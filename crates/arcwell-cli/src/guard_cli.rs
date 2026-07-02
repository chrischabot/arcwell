use crate::*;
use std::process::Stdio;

/// Max bytes of diff/new-file content handed inline to the reviewer. Larger changes
/// are summarised and the reviewer is told to inspect specific files itself.
const GUARD_DIFF_INLINE_CAP: usize = 120_000;
const GUARD_NEW_FILE_CAP: usize = 8_000;
const GUARD_MAX_NEW_FILES: usize = 30;
/// Default number of times a session may be blocked before the gate hard-allows
/// (with a warning) to guarantee the agent can always eventually finish.
const GUARD_DEFAULT_MAX_BLOCKS: i64 = 3;

pub(crate) fn guard(store: Store, args: GuardCommand) -> Result<()> {
    match args.command {
        GuardSubcommand::CaptureGoal { event, goal } => {
            guard_capture_goal_hook(&store, &event, goal)
        }
        GuardSubcommand::StopReview { reviewer } => guard_stop_review_hook(&store, reviewer),
        GuardSubcommand::Status { session_id, limit } => {
            print_json(&store.guard_status(session_id.as_deref(), limit)?)
        }
        GuardSubcommand::Enable => {
            store.guard_set_enabled(true)?;
            print_json(&json!({ "ok": true, "guard_enabled": true }))
        }
        GuardSubcommand::Disable => {
            store.guard_set_enabled(false)?;
            print_json(&json!({ "ok": true, "guard_enabled": false }))
        }
    }
}

/// SessionStart / UserPromptSubmit hook: persist the user's stated goal so the
/// Stop-time review has a stable target to judge against.
fn guard_capture_goal_hook(
    store: &Store,
    event: &str,
    goal_override: Option<String>,
) -> Result<()> {
    let raw = read_stdin_lossy()?;
    let value: Value = serde_json::from_str(raw.trim()).unwrap_or(Value::Null);
    let session_id = guard_str(&value, "session_id").unwrap_or_else(|| "unknown".to_string());
    let cwd = guard_str(&value, "cwd");

    let goal = goal_override
        .or_else(|| hook_text_from_input(&raw))
        .map(|g| g.trim().to_string())
        .filter(|g| !g.is_empty());

    match goal {
        Some(goal) => {
            let id = store.guard_capture_goal(&session_id, cwd.as_deref(), event, &goal, None)?;
            print_json(&json!({ "ok": true, "event": event, "goal_id": id }))
        }
        // SessionStart with no prompt text is normal — nothing to capture.
        None => print_json(&json!({ "ok": true, "event": event, "skipped": "no goal text" })),
    }
}

/// Stop hook: the cross-model gate. Allows (stdout silent) or blocks
/// (`{"decision":"block","reason":...}` on stdout, which the runtime feeds back).
fn guard_stop_review_hook(store: &Store, reviewer_override: Option<String>) -> Result<()> {
    let raw = read_stdin_lossy()?;
    let value: Value = serde_json::from_str(raw.trim()).unwrap_or(Value::Null);
    let session_id = guard_str(&value, "session_id").unwrap_or_else(|| "unknown".to_string());
    let cwd = guard_str(&value, "cwd")
        .or_else(|| std::env::var("CLAUDE_PROJECT_DIR").ok())
        .unwrap_or_else(|| ".".to_string());
    let last_message = guard_str(&value, "last_assistant_message").unwrap_or_default();
    let stop_active = value
        .get("stop_hook_active")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    // (1) Recursion guard: the reviewer subprocess runs with this set, so a
    // reviewer that itself triggers a Stop hook never recurses.
    if std::env::var("ARCWELL_GUARD_DISABLE").as_deref() == Ok("1") {
        return guard_allow("disabled via ARCWELL_GUARD_DISABLE");
    }
    // (2) Persisted kill switch.
    if !store.guard_enabled()? {
        return guard_allow("guard disabled");
    }
    // (3) The runtime's one-shot re-entrancy flag.
    if stop_active {
        return guard_allow("stop_hook_active");
    }

    let worker = guard_detect_worker();
    let reviewer = reviewer_override
        .or_else(|| std::env::var("ARCWELL_GUARD_REVIEWER").ok())
        .map(|r| r.trim().to_lowercase())
        .filter(|r| !r.is_empty())
        .unwrap_or_else(|| guard_other_model(&worker));

    // (4) Default-allow when there are no real code changes (status/report turns).
    let Some(repo_root) = guard_git_root(&cwd) else {
        return guard_allow("not a git repository");
    };
    let changes = guard_collect_changes(&repo_root);
    if changes.trim().is_empty() {
        return guard_allow("no code changes");
    }

    // (5) Bounded iteration counter: never trap the agent in a block loop.
    let max_blocks = std::env::var("ARCWELL_GUARD_MAX_BLOCKS")
        .ok()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(GUARD_DEFAULT_MAX_BLOCKS);
    let streak = store.guard_block_streak(&session_id)?;
    if streak >= max_blocks {
        store.guard_record_review(
            &session_id,
            None,
            streak + 1,
            &worker,
            &reviewer,
            "capped",
            "iteration cap reached; allowing with warning",
            None,
        )?;
        eprintln!(
            "[arcwell-guard] iteration cap ({max_blocks}) reached for this session — allowing finish. Review the work manually."
        );
        return guard_allow("iteration cap reached");
    }

    // (6) Run the independent reviewer.
    let goal = store.guard_active_goal(&session_id)?;
    let goal_id = goal
        .as_ref()
        .and_then(|g| g.get("id"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let prompt = guard_build_prompt(
        &worker,
        &reviewer,
        goal.as_ref(),
        &last_message,
        &changes,
        &repo_root,
    );
    let diff_summary =
        guard_run_git(&repo_root, &["diff", "--stat", "HEAD"]).map(|s| guard_truncate(&s, 1_000));

    match guard_run_reviewer(&reviewer, &repo_root, &prompt) {
        Ok((true, reason)) => {
            store.guard_record_review(
                &session_id,
                goal_id.as_deref(),
                streak + 1,
                &worker,
                &reviewer,
                "allow",
                &reason,
                diff_summary.as_deref(),
            )?;
            guard_allow(&reason)
        }
        Ok((false, reason)) => {
            store.guard_record_review(
                &session_id,
                goal_id.as_deref(),
                streak + 1,
                &worker,
                &reviewer,
                "block",
                &reason,
                diff_summary.as_deref(),
            )?;
            print_json(&json!({
                "decision": "block",
                "reason": format!(
                    "arcwell-guard stop-gate ({reviewer} reviewed {worker}'s work): {reason}\n\
                     Fix this before finishing. To bypass: `arcwell guard disable` or set ARCWELL_GUARD_DISABLE=1."
                ),
            }))
        }
        Err(err) => {
            store.guard_record_review(
                &session_id,
                goal_id.as_deref(),
                streak + 1,
                &worker,
                &reviewer,
                "error",
                &err.to_string(),
                diff_summary.as_deref(),
            )?;
            // Strict mode blocks on a broken reviewer; default fails OPEN so a flaky
            // judge can never wedge the session or burn a block loop.
            if std::env::var("ARCWELL_GUARD_STRICT").as_deref() == Ok("1") {
                print_json(&json!({
                    "decision": "block",
                    "reason": format!(
                        "arcwell-guard could not complete the {reviewer} review (strict mode): {err}. \
                         Run it manually or set ARCWELL_GUARD_DISABLE=1 to bypass."
                    ),
                }))
            } else {
                eprintln!("[arcwell-guard] review unavailable ({err}); allowing (fail-open).");
                guard_allow("review error (fail-open)")
            }
        }
    }
}

/// Allow = no decision on stdout (the runtime lets the turn end). Notes go to stderr.
fn guard_allow(reason: &str) -> Result<()> {
    eprintln!("[arcwell-guard] allow: {reason}");
    Ok(())
}

fn guard_str(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .filter(|s| !s.is_empty())
}

/// Which runtime is executing this hook. Override with `ARCWELL_GUARD_WORKER`.
fn guard_detect_worker() -> String {
    if let Ok(worker) = std::env::var("ARCWELL_GUARD_WORKER") {
        let worker = worker.trim().to_lowercase();
        if !worker.is_empty() {
            return worker;
        }
    }
    let is_codex = std::env::var_os("CODEX_SANDBOX").is_some()
        || std::env::var_os("CODEX_HOME").is_some()
        || std::env::vars_os().any(|(k, _)| k.to_string_lossy().starts_with("CODEX_"));
    if is_codex {
        "codex".to_string()
    } else {
        "claude".to_string()
    }
}

fn guard_other_model(worker: &str) -> String {
    match worker {
        "codex" => "claude".to_string(),
        _ => "codex".to_string(),
    }
}

fn guard_git_root(cwd: &str) -> Option<String> {
    let out = guard_run_git(cwd, &["rev-parse", "--show-toplevel"])?;
    let root = out.trim();
    (!root.is_empty()).then(|| root.to_string())
}

/// All uncommitted work: tracked diff vs HEAD plus the bodies of new untracked files.
/// Including untracked files is essential — a brand-new parallel system (e.g. a second
/// gateway) is entirely new files that `git diff` alone would never show.
fn guard_collect_changes(repo_root: &str) -> String {
    let mut out = String::new();
    if let Some(diff) = guard_run_git(repo_root, &["--no-pager", "diff", "HEAD"])
        && !diff.trim().is_empty()
    {
        out.push_str("# Tracked changes (git diff HEAD)\n");
        out.push_str(&diff);
        out.push('\n');
    }
    if let Some(list) = guard_run_git(repo_root, &["ls-files", "--others", "--exclude-standard"]) {
        let files: Vec<&str> = list.lines().filter(|l| !l.trim().is_empty()).collect();
        if !files.is_empty() {
            out.push_str("\n# New untracked files\n");
            for path in files.iter().take(GUARD_MAX_NEW_FILES) {
                out.push_str(&format!("\n--- NEW FILE: {path} ---\n"));
                match std::fs::read_to_string(std::path::Path::new(repo_root).join(path)) {
                    Ok(body) => out.push_str(&guard_truncate(&body, GUARD_NEW_FILE_CAP)),
                    Err(_) => out.push_str("(binary or unreadable)\n"),
                }
            }
            if files.len() > GUARD_MAX_NEW_FILES {
                out.push_str(&format!(
                    "\n(... {} more new files not shown; inspect with `git status`)\n",
                    files.len() - GUARD_MAX_NEW_FILES
                ));
            }
        }
    }
    out
}

fn guard_run_git(cwd: &str, args: &[&str]) -> Option<String> {
    let output = ProcessCommand::new("git")
        .arg("-C")
        .arg(cwd)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).to_string())
}

fn guard_truncate(text: &str, cap: usize) -> String {
    if text.len() <= cap {
        return text.to_string();
    }
    let mut end = cap;
    while !text.is_char_boundary(end) && end > 0 {
        end -= 1;
    }
    format!(
        "{}\n... [truncated {} bytes]\n",
        &text[..end],
        text.len() - end
    )
}

fn guard_build_prompt(
    worker: &str,
    reviewer: &str,
    goal: Option<&Value>,
    last_message: &str,
    changes: &str,
    repo_root: &str,
) -> String {
    let goal_block = match goal {
        Some(g) => {
            let text = g.get("goal").and_then(Value::as_str).unwrap_or("");
            let criteria = g
                .get("success_criteria")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .map(|s| format!("\nStated success criteria: {s}"))
                .unwrap_or_default();
            format!("{text}{criteria}")
        }
        None => "(No explicit goal was captured for this session. Judge whether the change is internally complete and not a workaround.)".to_string(),
    };

    let last_block = if last_message.trim().is_empty() {
        "(none captured)".to_string()
    } else {
        guard_truncate(last_message.trim(), 6_000)
    };

    let (changes_block, changes_note) = if changes.len() > GUARD_DIFF_INLINE_CAP {
        (
            guard_truncate(changes, GUARD_DIFF_INLINE_CAP),
            "\nThe change set is large and was truncated above — inspect specific files yourself with read-only git commands before deciding.",
        )
    } else {
        (changes.to_string(), "")
    };

    let guardrails = guard_load_guardrails(repo_root)
        .map(|g| format!("\nPROJECT GUARDRAILS (owned components — must not be duplicated or bypassed):\n{g}\n"))
        .unwrap_or_default();

    format!(
        r#"You are {reviewer}, performing an adversarial stop-gate review of work just produced by {worker} in a coding session. Decide whether the session may END, or whether the work is incomplete, wrong, or a workaround that must be fixed first.

Be skeptical: your job is to BREAK confidence in the change, not to validate it. Do not give credit for good intent, partial fixes, or likely follow-up work. Happy-path-only counts as a real weakness.

DO NOT TRUST THE WORKER'S REPORT. Treat the assistant's summary and any stated rationale ("kept it simple", "per YAGNI", "will follow up") as UNVERIFIED CLAIMS. A stated rationale never downgrades a finding. Verify against the actual repository state below, not the prose.

Watch specifically for WORKAROUNDS, which are the highest-priority block reason:
- a NEW parallel/duplicate system instead of fixing or extending the owned one (check the new files);
- bypassing an existing component because it lacked a capability, instead of adding the capability;
- disabling, ignoring, or stubbing something to make an error or a test go away;
- faking/hardcoding a result instead of implementing it.

THE STATED GOAL / DEFINITION OF DONE:
{goal_block}

WHAT {worker} CLAIMED IT DID (last assistant message — unverified):
{last_block}

THE ACTUAL UNCOMMITTED CHANGES (in {repo_root}):
{changes_block}{changes_note}
{guardrails}
You may run read-only git commands to inspect further. Do NOT edit anything.

Decide:
- ALLOW if the change genuinely accomplishes the stated goal, OR the turn made no real code change, OR you find no blocking issue.
- BLOCK only if the change is incomplete, incorrect, or a workaround that still needs fixing to meet the goal.

Respond with EXACTLY ONE line, nothing before it:
ALLOW: <one-sentence reason>
or
BLOCK: <specific, actionable reason naming exactly what to fix>"#
    )
}

/// Optional per-project sharpener: `.arcwell-guardrails.md` listing owned capabilities
/// the agent must not duplicate or bypass.
fn guard_load_guardrails(repo_root: &str) -> Option<String> {
    let path = std::path::Path::new(repo_root).join(".arcwell-guardrails.md");
    let body = std::fs::read_to_string(path).ok()?;
    let body = body.trim();
    (!body.is_empty()).then(|| guard_truncate(body, GUARD_NEW_FILE_CAP))
}

/// Run the opposite model as a non-interactive reviewer and parse its ALLOW:/BLOCK: line.
/// `ARCWELL_GUARD_DISABLE=1` is set in the child so the reviewer never recurses into its
/// own stop-gate.
fn guard_run_reviewer(reviewer: &str, cwd: &str, prompt: &str) -> Result<(bool, String)> {
    let mut command = match reviewer {
        "codex" => {
            let mut c = ProcessCommand::new("codex");
            c.args(["exec", prompt]);
            c
        }
        "claude" => {
            let mut c = ProcessCommand::new("claude");
            c.args(["-p", prompt]);
            c
        }
        other => bail!("unknown reviewer '{other}' (expected 'claude' or 'codex')"),
    };
    command
        .current_dir(cwd)
        .env("ARCWELL_GUARD_DISABLE", "1")
        .stdin(Stdio::null());

    let output = command
        .output()
        .with_context(|| format!("spawning reviewer '{reviewer}' (is it on PATH?)"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "reviewer '{reviewer}' exited with {}: {}",
            output.status,
            stderr.trim()
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let line = line.trim();
        if let Some(reason) = line.strip_prefix("ALLOW:") {
            return Ok((true, reason.trim().to_string()));
        }
        if let Some(reason) = line.strip_prefix("BLOCK:") {
            return Ok((false, reason.trim().to_string()));
        }
    }
    bail!("reviewer '{reviewer}' returned no ALLOW:/BLOCK: verdict")
}
