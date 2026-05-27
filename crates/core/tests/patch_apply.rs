use jadipa::patch::{Patch, PatchError};
use serde_json::json;

#[test]
fn applies_multi_operation_patch_without_mutating_input() {
    let target = json!({
        "name": "old",
        "tags": ["stable", "legacy"],
        "meta": {
            "enabled": true,
            "owner": "core"
        },
        "temporary": "remove-me"
    });
    let patch = Patch::new(
        r#"[
            {"op":"test","path":"/meta/enabled","value":true},
            {"op":"replace","path":"/name","value":"new"},
            {"op":"add","path":"/tags/-","value":"patched"},
            {"op":"remove","path":"/temporary"},
            {"op":"copy","from":"/meta/owner","path":"/owner"},
            {"op":"move","from":"/tags/0","path":"/primary_tag"}
        ]"#,
    )
    .unwrap();

    let patched = patch.apply(&target).unwrap();

    assert_eq!(
        patched,
        json!({
            "name": "new",
            "tags": ["legacy", "patched"],
            "meta": {
                "enabled": true,
                "owner": "core"
            },
            "owner": "core",
            "primary_tag": "stable"
        })
    );
    assert_eq!(
        target,
        json!({
            "name": "old",
            "tags": ["stable", "legacy"],
            "meta": {
                "enabled": true,
                "owner": "core"
            },
            "temporary": "remove-me"
        })
    );
}

#[test]
fn invalid_patch_document_returns_parse_error() {
    let result = Patch::new(r#"{"op":"add","path":"/name","value":"new"}"#);

    assert!(matches!(result, Err(PatchError::ParseError(_))));
}

#[test]
fn missing_target_path_returns_application_error() {
    let target = json!({"name": "old"});
    let patch = Patch::new(r#"[{"op":"replace","path":"/missing","value":"new"}]"#).unwrap();

    let err = patch.apply(&target).unwrap_err();

    assert!(err.to_string().contains("patch application failed"));
    assert!(err.to_string().contains("/missing"));
    assert_eq!(target, json!({"name": "old"}));
}

#[test]
fn failing_test_operation_returns_application_error_without_mutating_input() {
    let target = json!({"status": "draft", "items": [1, 2]});
    let patch = Patch::new(
        r#"[
            {"op":"test","path":"/status","value":"published"},
            {"op":"add","path":"/items/-","value":3}
        ]"#,
    )
    .unwrap();

    let err = patch.apply(&target).unwrap_err();

    assert!(err.to_string().contains("test operation failed"));
    assert!(err.to_string().contains("/status"));
    assert_eq!(target, json!({"status": "draft", "items": [1, 2]}));
}
