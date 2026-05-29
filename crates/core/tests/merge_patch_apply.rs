#![cfg(feature = "merge_patch")]

use jadipa::merge_patch::{apply, apply_mut};
use serde_json::json;

#[test]
fn applies_rfc_example_without_mutating_input() {
    let target = json!({
        "title": "Goodbye!",
        "author": {
            "givenName": "John",
            "familyName": "Doe"
        },
        "tags": ["example", "sample"],
        "content": "This will be unchanged"
    });
    let patch = json!({
        "title": "Hello!",
        "phoneNumber": "+01-123-456-7890",
        "author": {
            "familyName": null
        },
        "tags": ["example"]
    });

    let patched = apply(&target, &patch);

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
    assert_eq!(
        target,
        json!({
            "title": "Goodbye!",
            "author": {
                "givenName": "John",
                "familyName": "Doe"
            },
            "tags": ["example", "sample"],
            "content": "This will be unchanged"
        })
    );
}

#[test]
fn apply_mut_applies_rfc_example_in_place() {
    let mut target = json!({
        "title": "Goodbye!",
        "author": {
            "givenName": "John",
            "familyName": "Doe"
        },
        "tags": ["example", "sample"],
        "content": "This will be unchanged"
    });
    let patch = json!({
        "title": "Hello!",
        "phoneNumber": "+01-123-456-7890",
        "author": {
            "familyName": null
        },
        "tags": ["example"]
    });

    apply_mut(&mut target, &patch);

    assert_eq!(
        target,
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
