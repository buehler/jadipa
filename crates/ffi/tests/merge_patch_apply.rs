use jadipa::{JadipaError, MergePatch};
use serde_json::json;

#[test]
fn applies_merge_patch_json() {
    let result = MergePatch::apply_json(
        r#"{
            "title":"Goodbye!",
            "author":{"givenName":"John","familyName":"Doe"},
            "tags":["example","sample"],
            "content":"This will be unchanged"
        }"#,
        r#"{
            "title":"Hello!",
            "phoneNumber":"+01-123-456-7890",
            "author":{"familyName":null},
            "tags":["example"]
        }"#,
    );

    let patched_json = match result {
        Ok(value) => value,
        Err(_) => panic!("expected merge patch to apply"),
    };
    let patched: serde_json::Value = serde_json::from_str(&patched_json).unwrap();

    assert_eq!(
        patched,
        json!({
            "title": "Hello!",
            "author": {
                "givenName": "John"
            },
            "tags": ["example"],
            "content": "This will be unchanged",
            "phoneNumber": "+01-123-456-7890"
        })
    );
}

#[test]
fn invalid_merge_patch_json_returns_invalid_json_error() {
    let result = MergePatch::apply_json(r#"{"name":"old"}"#, r#"{"name":"new""#);

    match result {
        Err(JadipaError::InvalidJson { message }) => {
            assert!(message.contains("EOF"));
        }
        Ok(_) => panic!("expected invalid JSON error"),
        Err(_) => panic!("expected invalid JSON error"),
    }
}
