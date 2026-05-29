//! JSON Merge Patch support.
//!
//! JSON Merge Patch describes changes to a JSON document using a patch value
//! whose shape mostly mirrors the target document. Object members in the patch
//! are added to the target when missing, replace existing non-object values,
//! and recursively patch existing object values.
//!
//! A `null` value in an object patch removes that member from the target. If
//! the patch itself is not an object, it replaces the entire target. Arrays are
//! replaced as complete values; merge patch cannot update individual array
//! elements.
//!
//! The media type for JSON Merge Patch documents is
//! `application/merge-patch+json`.
//!
//! # Example
//!
//! ```
//! use jadipa::merge_patch;
//! use serde_json::json;
//!
//! let target = json!({
//!     "title": "Goodbye!",
//!     "author": {
//!         "givenName": "John",
//!         "familyName": "Doe"
//!     },
//!     "tags": ["example", "sample"]
//! });
//! let patch = json!({
//!     "title": "Hello!",
//!     "author": {
//!         "familyName": null
//!     },
//!     "tags": ["example"]
//! });
//!
//! let patched = merge_patch::apply(&target, &patch);
//!
//! assert_eq!(
//!     patched,
//!     json!({
//!         "title": "Hello!",
//!         "author": {
//!             "givenName": "John"
//!         },
//!         "tags": ["example"]
//!     })
//! );
//! assert_eq!(target["title"], "Goodbye!");
//! ```

use serde_json::{Map, Value};

/// Applies a JSON Merge Patch to `target` in place.
///
/// If `patch` is an object, its members are merged into `target`. Existing
/// object members are patched recursively, missing members are added, and
/// members whose patch value is `null` are removed. If `target` is not an
/// object and `patch` is an object, `target` is first replaced with an empty
/// object.
///
/// If `patch` is not an object, it replaces the entire target value. Arrays are
/// therefore replaced as complete values instead of being merged element by
/// element.
///
/// # Example
///
/// ```
/// use jadipa::merge_patch;
/// use serde_json::json;
///
/// let mut target = json!({
///     "title": "Goodbye!",
///     "author": {
///         "givenName": "John",
///         "familyName": "Doe"
///     }
/// });
/// let patch = json!({
///     "title": "Hello!",
///     "author": {
///         "familyName": null
///     }
/// });
///
/// merge_patch::apply_mut(&mut target, &patch);
///
/// assert_eq!(
///     target,
///     json!({
///         "title": "Hello!",
///         "author": {
///             "givenName": "John"
///         }
///     })
/// );
/// ```
pub fn apply_mut(target: &mut Value, patch: &Value) {
    let Some(patch_object) = patch.as_object() else {
        *target = patch.clone();
        return;
    };

    if !target.is_object() {
        *target = Value::Object(Map::new());
    }
    let target_object = target.as_object_mut().unwrap();

    for (key, patch_value) in patch_object {
        if patch_value.is_null() {
            target_object.remove(key);
            continue;
        }

        let target_value = target_object.entry(key.clone()).or_insert(Value::Null);
        apply_mut(target_value, patch_value);
    }
}

/// Applies a JSON Merge Patch to `target` and returns the patched value.
///
/// This function clones `target` before applying `patch`, so the input value is
/// not modified. Use [`apply_mut`] when the patch should be applied directly to
/// an existing [`Value`].
///
/// The merge semantics are the same as [`apply_mut`]: object patches add,
/// replace, recursively patch, or remove object members, while non-object
/// patches replace the entire target value.
///
/// # Example
///
/// ```
/// use jadipa::merge_patch;
/// use serde_json::json;
///
/// let target = json!({
///     "title": "Goodbye!",
///     "tags": ["example", "sample"]
/// });
/// let patch = json!({
///     "title": "Hello!",
///     "tags": ["example"]
/// });
///
/// let patched = merge_patch::apply(&target, &patch);
///
/// assert_eq!(
///     patched,
///     json!({
///         "title": "Hello!",
///         "tags": ["example"]
///     })
/// );
/// assert_eq!(
///     target,
///     json!({
///         "title": "Goodbye!",
///         "tags": ["example", "sample"]
///     })
/// );
/// ```
pub fn apply(target: &Value, patch: &Value) -> Value {
    let mut target = target.clone();
    apply_mut(&mut target, patch);
    target
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use yare::parameterized;

    #[parameterized(
        replace_existing_member = {
            json!({"a": "b"}),
            json!({"a": "c"}),
            json!({"a": "c"})
        },
        add_new_member = {
            json!({"a": "b"}),
            json!({"b": "c"}),
            json!({"a": "b", "b": "c"})
        },
        remove_existing_member = {
            json!({"a": "b"}),
            json!({"a": null}),
            json!({})
        },
        remove_one_of_multiple_members = {
            json!({"a": "b", "b": "c"}),
            json!({"a": null}),
            json!({"b": "c"})
        },
        replace_array_with_scalar = {
            json!({"a": ["b"]}),
            json!({"a": "c"}),
            json!({"a": "c"})
        },
        replace_scalar_with_array = {
            json!({"a": "c"}),
            json!({"a": ["b"]}),
            json!({"a": ["b"]})
        },
        recursively_patch_object = {
            json!({"a": {"b": "c"}}),
            json!({"a": {"b": "d", "c": null}}),
            json!({"a": {"b": "d"}})
        },
        replace_array_member = {
            json!({"a": [{"b": "c"}]}),
            json!({"a": [1]}),
            json!({"a": [1]})
        },
        replace_array_target_with_array_patch = {
            json!(["a", "b"]),
            json!(["c", "d"]),
            json!(["c", "d"])
        },
        replace_object_target_with_array_patch = {
            json!({"a": "b"}),
            json!(["c"]),
            json!(["c"])
        },
        replace_object_target_with_null_patch = {
            json!({"a": "foo"}),
            json!(null),
            json!(null)
        },
        replace_object_target_with_string_patch = {
            json!({"a": "foo"}),
            json!("bar"),
            json!("bar")
        },
        preserve_null_member_when_patch_adds_member = {
            json!({"e": null}),
            json!({"a": 1}),
            json!({"e": null, "a": 1})
        },
        object_patch_replaces_non_object_target = {
            json!([1, 2]),
            json!({"a": "b", "c": null}),
            json!({"a": "b"})
        },
        missing_nested_null_removal_does_not_create_member = {
            json!({}),
            json!({"a": {"bb": {"ccc": null}}}),
            json!({"a": {"bb": {}}})
        }
    )]
    fn applies_rfc_appendix_a_examples(target: Value, patch: Value, expected: Value) {
        assert_eq!(apply(&target, &patch), expected);
    }

    #[test]
    fn apply_does_not_mutate_input() {
        let target = json!({
            "title": "Goodbye!",
            "author": {
                "givenName": "John",
                "familyName": "Doe"
            }
        });
        let patch = json!({
            "title": "Hello!",
            "author": {
                "familyName": null
            }
        });

        let patched = apply(&target, &patch);

        assert_eq!(
            patched,
            json!({
                "title": "Hello!",
                "author": {
                    "givenName": "John"
                }
            })
        );
        assert_eq!(
            target,
            json!({
                "title": "Goodbye!",
                "author": {
                    "givenName": "John",
                    "familyName": "Doe"
                }
            })
        );
    }

    #[test]
    fn apply_mut_updates_target_in_place() {
        let mut target = json!({
            "title": "Goodbye!",
            "author": {
                "givenName": "John",
                "familyName": "Doe"
            },
            "tags": ["example", "sample"]
        });
        let patch = json!({
            "title": "Hello!",
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
                "tags": ["example"]
            })
        );
    }
}
