use super::*;
use std::collections::HashSet;

// Tools that must not be probed with empty arguments even against an
// isolated, empty ARCWELL_HOME. Empty for now: every registered tool is
// expected to fail closed (missing arg / policy denied / not found) rather
// than perform a real external side effect when called with `json!({})`
// against a fresh temp home. If a future tool needs to be skipped, add its
// name here with a one-line reason and it will be logged via eprintln!.
const UNPROBEABLE: &[(&str, &str)] = &[];

#[test]
fn mcp_tool_parity_registry_matches_dispatch() {
    let paths = test_paths("mcp-tool-parity");

    let tools = crate::mcp_tools();
    assert!(
        !tools.is_empty(),
        "registry must advertise at least one tool"
    );

    let names: Vec<&str> = tools
        .iter()
        .map(|value| value["name"].as_str().expect("tool has a name"))
        .collect();

    let unique_names: HashSet<&str> = names.iter().copied().collect();
    assert_eq!(
        unique_names.len(),
        names.len(),
        "registry must not advertise duplicate tool names"
    );

    for name in names {
        if let Some((_, reason)) = UNPROBEABLE.iter().find(|(skip_name, _)| *skip_name == name) {
            eprintln!("mcp_tool_parity: skipping `{name}` ({reason})");
            continue;
        }

        if let Err(err) = crate::call_mcp_tool(&paths, name, json!({})) {
            let msg = err.to_string();
            assert!(
                !msg.contains("unknown tool"),
                "registered MCP tool `{name}` has no dispatch handler: {msg}"
            );
        }
    }
}
