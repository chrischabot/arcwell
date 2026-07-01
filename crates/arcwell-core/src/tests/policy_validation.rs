use super::*;

#[test]
fn policy_validation_effect_accepts_known_values_rejects_unknown() {
    // CLAIM: validate_policy_effect only accepts the four defined policy effects.
    // ORACLE: known effects are Ok, an unrecognized string is Err.
    for effect in ["allow", "deny", "require_approval", "defer"] {
        assert!(
            crate::validate_policy_effect(effect).is_ok(),
            "expected {effect} to be accepted"
        );
    }
    assert!(crate::validate_policy_effect("nope").is_err());
}

#[test]
fn policy_validation_cost_scope_accepts_known_values_rejects_unknown() {
    // CLAIM: validate_cost_scope only accepts the four defined cost scopes.
    // ORACLE: known scopes are Ok, an unrecognized scope is Err.
    for scope in ["global", "package", "provider", "source"] {
        assert!(
            crate::validate_cost_scope(scope).is_ok(),
            "expected {scope} to be accepted"
        );
    }
    assert!(crate::validate_cost_scope("team").is_err());
}

#[test]
fn policy_validation_candidate_operation_accepts_known_values_rejects_unknown() {
    // CLAIM: validate_candidate_operation only accepts the four uppercase
    // memory-candidate operations.
    // ORACLE: known uppercase operations are Ok; a lowercase variant is Err
    // because matching is case-sensitive.
    for operation in ["ADD", "UPDATE", "DELETE", "NONE"] {
        assert!(
            crate::validate_candidate_operation(operation).is_ok(),
            "expected {operation} to be accepted"
        );
    }
    assert!(crate::validate_candidate_operation("add").is_err());
}

#[test]
fn policy_validation_digest_review_status_accepts_known_values_rejects_unknown() {
    // CLAIM: validate_digest_review_status only accepts approved/rejected.
    // ORACLE: known statuses are Ok, an unrecognized status is Err.
    for status in ["approved", "rejected"] {
        assert!(
            crate::validate_digest_review_status(status).is_ok(),
            "expected {status} to be accepted"
        );
    }
    assert!(crate::validate_digest_review_status("pending").is_err());
}

#[test]
fn policy_validation_action_rejects_empty_whitespace_and_overlong() {
    // CLAIM: validate_policy_action rejects empty/whitespace-only strings and
    // strings over 120 chars, but accepts a normal action string.
    // ORACLE: "" and "   " are Err; a 121-char string is Err; a short real
    // action string is Ok.
    assert!(crate::validate_policy_action("").is_err());
    assert!(crate::validate_policy_action("   ").is_err());
    let long_action = "x".repeat(121);
    assert!(crate::validate_policy_action(&long_action).is_err());
    assert!(crate::validate_policy_action("provider.network").is_ok());
}

#[test]
fn policy_validation_pattern_rejects_empty_and_overlong() {
    // CLAIM: validate_policy_pattern rejects empty strings and strings over
    // 240 chars, but accepts a normal wildcard pattern.
    // ORACLE: "" is Err; a 241-char string is Err; "github*" is Ok.
    assert!(crate::validate_policy_pattern("").is_err());
    let long_pattern = "x".repeat(241);
    assert!(crate::validate_policy_pattern(&long_pattern).is_err());
    assert!(crate::validate_policy_pattern("github*").is_ok());
}

#[test]
fn policy_validation_non_negative_cost_rejects_bad_values_accepts_good() {
    // CLAIM: validate_non_negative_cost rejects negative, non-finite, and
    // above-cap values, and accepts 0.0 and other in-range values.
    // ORACLE: -1.0, NaN, +Infinity, and 1_000_000.1 are Err; 0.0 and 12.5 are Ok.
    assert!(crate::validate_non_negative_cost(-1.0, "label").is_err());
    assert!(crate::validate_non_negative_cost(f64::NAN, "label").is_err());
    assert!(crate::validate_non_negative_cost(f64::INFINITY, "label").is_err());
    assert!(crate::validate_non_negative_cost(1_000_000.1, "label").is_err());
    assert!(crate::validate_non_negative_cost(0.0, "label").is_ok());
    assert!(crate::validate_non_negative_cost(12.5, "label").is_ok());
}

#[test]
fn policy_validation_pattern_matches_covers_none_wildcard_prefix_and_exact() {
    // CLAIM: pattern_matches implements: None pattern matches anything; a
    // pattern with no value never matches; "*" matches anything; a "prefix*"
    // pattern matches by prefix; otherwise it is an exact match.
    // ORACLE: enumerated (pattern, value) -> bool pairs from the source.
    assert!(crate::pattern_matches(None, Some("x")));
    assert!(!crate::pattern_matches(Some("x"), None));
    assert!(crate::pattern_matches(Some("*"), Some("anything")));
    assert!(crate::pattern_matches(Some("git*"), Some("github")));
    assert!(!crate::pattern_matches(Some("git*"), Some("lab")));
    assert!(crate::pattern_matches(Some("exact"), Some("exact")));
    assert!(!crate::pattern_matches(Some("exact"), Some("other")));
}

#[test]
fn policy_validation_pattern_specificity_ranks_exact_over_prefix_over_wildcard() {
    // CLAIM: pattern_specificity scores an absent pattern and a wildcard as 0,
    // a prefix pattern as 1, and an exact match as 3; a non-matching pattern
    // also scores 0.
    // ORACLE: enumerated (pattern, value) -> score triples from the source.
    assert_eq!(crate::pattern_specificity(None, Some("x")), 0);
    assert_eq!(crate::pattern_specificity(Some("*"), Some("x")), 0);
    assert_eq!(crate::pattern_specificity(Some("git*"), Some("github")), 1);
    assert_eq!(crate::pattern_specificity(Some("exact"), Some("exact")), 3);
    assert_eq!(crate::pattern_specificity(Some("nomatch"), Some("x")), 0);
}

#[test]
fn policy_validation_effect_rank_orders_deny_above_require_approval_above_allow() {
    // CLAIM: effect_rank orders effects by how strongly they should win a
    // most-specific-rule tie-break: deny > require_approval > allow.
    // ORACLE: numeric ranks from the source satisfy that strict ordering.
    assert!(crate::effect_rank("deny") > crate::effect_rank("require_approval"));
    assert!(crate::effect_rank("require_approval") > crate::effect_rank("allow"));
}

fn wildcard_provider_rule() -> PolicyRule {
    PolicyRule {
        id: "test-rule".to_string(),
        effect: "deny".to_string(),
        action: "provider.network".to_string(),
        reason: "test".to_string(),
        package: None,
        provider: Some("*".to_string()),
        source: None,
        channel: None,
        subject: None,
        target: None,
        priority: 0,
        expires_at: None,
    }
}

fn base_request(action: &str, provider: Option<&str>) -> PolicyRequest {
    PolicyRequest {
        action: action.to_string(),
        package: None,
        provider: provider.map(|p| p.to_string()),
        source: None,
        channel: None,
        subject: None,
        target: None,
        projected_usd: None,
        metadata: Value::Null,
        untrusted_excerpt: None,
    }
}

#[test]
fn policy_validation_rule_matches_same_action_with_wildcard_provider() {
    // CLAIM: policy_rule_matches AND-combines per-field pattern matching; a
    // rule scoped to action "provider.network" with a wildcard provider
    // pattern matches any request with that same action.
    // ORACLE: the rule matches a "provider.network" request for provider
    // "openai".
    let rule = wildcard_provider_rule();
    let request = base_request("provider.network", Some("openai"));
    assert!(crate::policy_rule_matches(&rule, &request));
}

#[test]
fn policy_validation_rule_does_not_match_different_action() {
    // CLAIM: policy_rule_matches requires the action pattern to match; a rule
    // scoped to "provider.network" must not match an unrelated action.
    // ORACLE: the same rule does not match a "memory.capture" request.
    let rule = wildcard_provider_rule();
    let request = base_request("memory.capture", Some("openai"));
    assert!(!crate::policy_rule_matches(&rule, &request));
}
