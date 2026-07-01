use super::*;

#[test]
fn secret_redaction_redacts_provider_prefixed_tokens() {
    // CLAIM: tokens shaped like known provider secrets are always redacted.
    // ORACLE: redact_secret_token returns the literal marker for each provider prefix.
    for token in [
        "sk-abc123def456",
        "xoxb-1-2-abc",
        "ghp_0123456789abcdef",
        "github_pat_11ABANE",
        "AKIAIOSFODNN7EXAMPLE",
    ] {
        assert_eq!(
            crate::redact_secret_token(token),
            "[REDACTED]",
            "expected {token} to be redacted"
        );
    }
}

#[test]
fn secret_redaction_redacts_assignment_shaped_tokens() {
    // CLAIM: key=value and query-parameter assignment shapes for secret-bearing
    // keys are redacted regardless of the value's own shape.
    // ORACLE: redact_secret_token returns the literal marker for each assignment token.
    for token in [
        "token=deadbeefcafe",
        "password=hunter2",
        "api_key=xyz",
        "?access_token=zzz",
        "&secret=qqq",
    ] {
        assert_eq!(
            crate::redact_secret_token(token),
            "[REDACTED]",
            "expected {token} to be redacted"
        );
    }
}

#[test]
fn secret_redaction_redacts_high_entropy_32_char_token() {
    // CLAIM: an opaque 32+ char alphanumeric token is treated as high-entropy
    // secret material even without a recognizable prefix or assignment shape.
    // ORACLE: a 32-char [A-Za-z0-9] string becomes the redaction marker.
    let token = "a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6";
    assert_eq!(token.len(), 32);
    assert_eq!(crate::redact_secret_token(token), "[REDACTED]");
}

#[test]
fn secret_redaction_leaves_bearer_literal_unchanged() {
    // CLAIM: the bare word "bearer" (any case) is a scheme label, not a secret,
    // and must pass through unchanged.
    // ORACLE: redact_secret_token is the identity function for "bearer"/"Bearer".
    assert_eq!(crate::redact_secret_token("bearer"), "bearer");
    assert_eq!(crate::redact_secret_token("Bearer"), "Bearer");
}

#[test]
fn secret_redaction_leaves_lowercase_dotted_identifier_unchanged() {
    // CLAIM: lowercase dotted/underscored identifiers that are not secret-shaped
    // (e.g. package/bundle ids) are an intentional carve-out and stay unchanged.
    // ORACLE: redact_secret_token is the identity function for "com.example.service".
    assert_eq!(
        crate::redact_secret_token("com.example.service"),
        "com.example.service"
    );
}

#[test]
fn secret_redaction_leaves_plain_word_unchanged() {
    // CLAIM: an ordinary short word is never redacted.
    // ORACLE: redact_secret_token is the identity function for "hello".
    assert_eq!(crate::redact_secret_token("hello"), "hello");
}

#[test]
fn secret_redaction_leaves_31_char_alphanumeric_unchanged() {
    // CLAIM: the high-entropy heuristic requires at least 32 characters, so a
    // 31-char alphanumeric string must not be redacted.
    // ORACLE: redact_secret_token does not produce the redaction marker.
    let token = "a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p";
    assert_eq!(token.len(), 31);
    assert_ne!(crate::redact_secret_token(token), "[REDACTED]");
}

#[test]
fn secret_redaction_lowercase_akia_prefix_is_case_sensitive() {
    // CLAIM: the AKIA provider prefix match is case-sensitive by design, so a
    // lowercase "akia..." string is not caught by that specific rule. This
    // documents current behavior rather than asserting it is fully unredacted
    // by every rule (it is also short enough to dodge the entropy heuristic).
    // ORACLE: redact_secret_token does not produce the redaction marker for the
    // lowercase variant of a well-known AKIA example key.
    let token = "akiaiosfodnn7example";
    assert_eq!(token.len(), 20);
    assert_ne!(crate::redact_secret_token(token), "[REDACTED]");
}

#[test]
fn secret_redaction_text_wrapper_redacts_token_but_preserves_scheme_word() {
    // CLAIM: redact_secret_like_text redacts secret-shaped tokens word-by-word
    // while leaving non-secret scheme words like "Bearer" intact.
    // ORACLE: the output contains the redaction marker and still contains "Bearer".
    let input = "authorization: Bearer sk-livetoken1234567890";
    let output = crate::redact_secret_like_text(input);
    assert!(
        output.contains("[REDACTED]"),
        "expected redaction marker in {output}"
    );
    assert!(
        output.contains("Bearer"),
        "expected scheme word preserved in {output}"
    );
}

#[test]
fn secret_redaction_json_recurses_into_nested_objects() {
    // CLAIM: redact_secret_like_json mutates a JSON tree in place, redacting
    // secret-shaped string leaves anywhere in the structure while leaving
    // ordinary string values untouched.
    // ORACLE: a nested "token" field bearing a provider-prefixed secret becomes
    // the redaction marker; a sibling plain string is unchanged.
    let mut v = json!({"a": {"token": "sk-abc123def456ghi"}, "b": "plain"});
    crate::redact_secret_like_json(&mut v);
    assert_eq!(v["a"]["token"], "[REDACTED]");
    assert_eq!(v["b"], "plain");
}
