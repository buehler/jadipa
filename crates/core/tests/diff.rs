#![cfg(feature = "diff")]

use jadipa::{diff, patch::Patch};
use serde_json::json;

#[test]
fn diff_patch_transforms_source_without_mutating_it() {
    let source = json!({
        "name": "old",
        "tags": ["stable", "legacy"],
        "meta": {
            "enabled": false,
            "owner": "core"
        },
        "temporary": "remove-me"
    });
    let target = json!({
        "name": "new",
        "tags": ["stable", "legacy", "patched"],
        "meta": {
            "enabled": true,
            "owner": "core"
        }
    });

    let patch = diff::diff(&source, &target);
    let patched = patch.apply(&source).unwrap();

    assert_eq!(patched, target);
    assert_eq!(
        source,
        json!({
            "name": "old",
            "tags": ["stable", "legacy"],
            "meta": {
                "enabled": false,
                "owner": "core"
            },
            "temporary": "remove-me"
        })
    );
}

#[test]
fn equal_documents_produce_empty_patch() {
    let source = json!({
        "name": "same",
        "items": [1, {"nested": true}]
    });

    let patch = diff::diff(&source, &source);

    assert_eq!(patch, Patch::empty());
}
