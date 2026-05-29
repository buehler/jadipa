use jadipa::{JadipaError, Patch};
use serde_json::json;

#[test]
fn applies_patch_json() {
    let result = Patch::apply_json(
        r#"{"name":"old","tags":["stable"]}"#,
        r#"[
            {"op":"replace","path":"/name","value":"new"},
            {"op":"add","path":"/tags/-","value":"ffi"}
        ]"#,
    );

    let patched_json = match result {
        Ok(value) => value,
        Err(_) => panic!("expected patch to apply"),
    };
    let patched: serde_json::Value = serde_json::from_str(&patched_json).unwrap();

    assert_eq!(patched, json!({"name":"new","tags":["stable","ffi"]}));
}

#[test]
fn invalid_target_json_returns_invalid_json_error() {
    let result = Patch::apply_json(
        r#"{"name":"old""#,
        r#"[{"op":"replace","path":"/name","value":"new"}]"#,
    );

    match result {
        Err(JadipaError::InvalidJson { message }) => {
            assert!(message.contains("EOF"));
        }
        Ok(_) => panic!("expected invalid JSON error"),
        Err(_) => panic!("expected invalid JSON error"),
    }
}

#[test]
fn invalid_patch_document_returns_invalid_patch_error() {
    let result = Patch::apply_json(
        r#"{"name":"old"}"#,
        r#"{"op":"replace","path":"/name","value":"new"}"#,
    );

    match result {
        Err(JadipaError::InvalidPatch { message }) => {
            assert!(message.contains("patch parse failed"));
        }
        Ok(_) => panic!("expected invalid patch error"),
        Err(_) => panic!("expected invalid patch error"),
    }
}

#[test]
fn missing_target_path_returns_patch_application_error() {
    let result = Patch::apply_json(
        r#"{"name":"old"}"#,
        r#"[{"op":"replace","path":"/missing","value":"new"}]"#,
    );

    match result {
        Err(JadipaError::PatchApplication { message }) => {
            assert!(message.contains("/missing"));
        }
        Ok(_) => panic!("expected patch application error"),
        Err(_) => panic!("expected patch application error"),
    }
}
