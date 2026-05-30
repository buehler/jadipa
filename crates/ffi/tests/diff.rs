use jadipa::{Diff, JadipaError, Patch};
use serde_json::json;

#[test]
fn creates_diff_patch_json() {
    let source_json = r#"{
        "name":"old",
        "tags":["stable","legacy"],
        "meta":{"enabled":false},
        "temporary":"remove-me"
    }"#;
    let target_json = r#"{
        "name":"new",
        "tags":["stable","legacy","ffi"],
        "meta":{"enabled":true}
    }"#;

    let patch_json = match Diff::diff_json(source_json, target_json) {
        Ok(value) => value,
        Err(_) => panic!("expected diff to be created"),
    };
    let patched_json = match Patch::apply_json(source_json, &patch_json) {
        Ok(value) => value,
        Err(_) => panic!("expected generated patch to apply"),
    };
    let patched: serde_json::Value = serde_json::from_str(&patched_json).unwrap();

    assert_eq!(
        patched,
        json!({
            "name": "new",
            "tags": ["stable", "legacy", "ffi"],
            "meta": {
                "enabled": true
            }
        })
    );
}

#[test]
fn invalid_source_json_returns_invalid_json_error() {
    let result = Diff::diff_json(r#"{"name":"old""#, r#"{"name":"new"}"#);

    match result {
        Err(JadipaError::InvalidJson { message }) => {
            assert!(message.contains("EOF"));
        }
        Ok(_) => panic!("expected invalid JSON error"),
        Err(_) => panic!("expected invalid JSON error"),
    }
}
