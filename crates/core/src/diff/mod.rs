//! JSON diff support.
//!
//! This module compares two [`serde_json::Value`] documents and produces a
//! [`Patch`] that transforms the source document into the target document.
//! The generated patch uses `add`, `remove`, and `replace` operations.
//!
//! Object members are compared recursively. Array comparison keeps shared
//! prefixes and suffixes, then replaces, removes, or adds items in the changed
//! middle section. The result is valid JSON Patch, but it is not guaranteed to
//! be the shortest possible patch.
//!
//! # Example
//!
//! ```
//! use jadipa::diff;
//! use serde_json::json;
//!
//! let source = json!({ "name": "old", "tags": ["a", "b"] });
//! let target = json!({ "name": "new", "tags": ["a", "b", "c"] });
//!
//! let patch = diff::diff(&source, &target);
//! let patched = patch.apply(&source)?;
//!
//! assert_eq!(patched, target);
//! assert_eq!(source["name"], "old");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use serde_json::Value;

use crate::{
    patch::{Patch, PatchOperation},
    pointer::Pointer,
};

/// Creates a JSON Patch that transforms `source` into `target`.
///
/// Equal values return an empty patch. Objects and arrays are compared
/// recursively, and scalar or type-changing differences are emitted as
/// `replace` operations. Object keys are escaped as JSON Pointer reference
/// tokens, so generated paths are valid for keys containing `/` or `~`.
///
/// Array diffs preserve equal prefixes and suffixes before emitting operations
/// for the changed middle section. This keeps common append, prepend, insert,
/// and truncate cases compact, but the generated patch is not a globally
/// minimal edit script.
///
/// # Example
///
/// ```
/// use jadipa::diff;
/// use serde_json::json;
///
/// let source = json!({ "enabled": false, "items": [1, 2, 4] });
/// let target = json!({ "enabled": true, "items": [1, 2, 3, 4] });
///
/// let patch = diff::diff(&source, &target);
///
/// assert_eq!(patch.apply(&source)?, target);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn diff(source: &Value, target: &Value) -> Patch {
    if source == target {
        // things are equal. Serde performs deep equality.
        return Patch::empty();
    }

    let mut operations = Vec::new();
    let mut path = String::new();

    rec_diff(&mut operations, &mut path, source, target);

    Patch::new_with(operations)
}

fn push_key(path: &mut String, key: &str) -> usize {
    let old_len = path.len();

    path.push('/');

    for ch in key.chars() {
        match ch {
            '~' => path.push_str("~0"),
            '/' => path.push_str("~1"),
            _ => path.push(ch),
        }
    }

    old_len
}

// functionally, the same function as push_key, but does not need to allocate
// a string if used with numbers.
fn push_index(path: &mut String, index: usize) -> usize {
    use std::fmt::Write;

    let old_len = path.len();
    path.push('/');
    write!(path, "{index}").unwrap();
    old_len
}

fn rec_diff(ops: &mut Vec<PatchOperation>, path: &mut String, source: &Value, target: &Value) {
    match (source, target) {
        (Value::Object(source), Value::Object(target)) => {
            for (key, source_value) in source {
                let old_len = push_key(path, key);

                match target.get(key) {
                    Some(target_value) => {
                        rec_diff(ops, path, source_value, target_value);
                    }
                    None => {
                        ops.push(PatchOperation::Remove {
                            path: Pointer::new(path.clone()),
                        });
                    }
                }

                path.truncate(old_len);
            }

            for (key, target_value) in target {
                if source.get(key).is_none() {
                    let old_len = push_key(path, key);

                    ops.push(PatchOperation::Add {
                        path: Pointer::new(path.clone()),
                        value: target_value.clone(),
                    });

                    path.truncate(old_len);
                }
            }
        }
        (Value::Array(source), Value::Array(target)) => {
            let source_len = source.len();
            let target_len = target.len();

            let mut prefix = 0;
            while prefix < source_len && prefix < target_len && source[prefix] == target[prefix] {
                prefix += 1;
            }

            let mut suffix = 0;
            while suffix < source_len - prefix
                && suffix < target_len - prefix
                && source[source_len - 1 - suffix] == target[target_len - 1 - suffix]
            {
                suffix += 1;
            }

            let source_mid_len = source_len - prefix - suffix;
            let target_mid_len = target_len - prefix - suffix;
            let paired_len = source_mid_len.min(target_mid_len);

            for offset in 0..paired_len {
                let index = prefix + offset;
                let old_len = push_index(path, index);

                rec_diff(ops, path, &source[index], &target[index]);

                path.truncate(old_len);
            }

            if source_mid_len > target_mid_len {
                for offset in (target_mid_len..source_mid_len).rev() {
                    let index = prefix + offset;
                    let old_len = push_index(path, index);

                    ops.push(PatchOperation::Remove {
                        path: Pointer::new(path.clone()),
                    });

                    path.truncate(old_len);
                }
            }

            if target_mid_len > source_mid_len {
                for offset in source_mid_len..target_mid_len {
                    let index = prefix + offset;
                    let old_len = push_index(path, index);

                    ops.push(PatchOperation::Add {
                        path: Pointer::new(path.clone()),
                        value: target[index].clone(),
                    });

                    path.truncate(old_len);
                }
            }
        }
        _ if source != target => {
            ops.push(PatchOperation::Replace {
                path: Pointer::new(path.clone()),
                value: target.clone(),
            });
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use yare::parameterized;

    #[parameterized(
        empty_patch = { json!("j"), json!("j"), Patch::empty() },
        replace_root_element = { json!(42), json!(true), Patch::new(r#"[
                {"op":"replace","path":"","value":true}
            ]"#).unwrap() },
        empty_objects = {
            json!({}),
            json!({}),
            Patch::empty()
        },
        equal_objects = {
            json!({"a": 1, "b": true, "c": null}),
            json!({"a": 1, "b": true, "c": null}),
            Patch::empty()
        },
        add_root_member = {
            json!({"a": 1}),
            json!({"a": 1, "b": 2}),
            Patch::new(r#"[
                {"op":"add","path":"/b","value":2}
            ]"#).unwrap()
        },
        remove_root_member = {
            json!({"a": 1, "b": 2}),
            json!({"a": 1}),
            Patch::new(r#"[
                {"op":"remove","path":"/b"}
            ]"#).unwrap()
        },
        replace_root_member = {
            json!({"a": 1, "b": 2}),
            json!({"a": 1, "b": 3}),
            Patch::new(r#"[
                {"op":"replace","path":"/b","value":3}
            ]"#).unwrap()
        },
        add_nested_member = {
            json!({"a": {"b": 1}}),
            json!({"a": {"b": 1, "c": 2}}),
            Patch::new(r#"[
                {"op":"add","path":"/a/c","value":2}
            ]"#).unwrap()
        },
        remove_nested_member = {
            json!({"a": {"b": 1, "c": 2}}),
            json!({"a": {"b": 1}}),
            Patch::new(r#"[
                {"op":"remove","path":"/a/c"}
            ]"#).unwrap()
        },
        replace_nested_member = {
            json!({"a": {"b": {"c": 1}}}),
            json!({"a": {"b": {"c": 2}}}),
            Patch::new(r#"[
                {"op":"replace","path":"/a/b/c","value":2}
            ]"#).unwrap()
        },
        replace_object_with_scalar_member = {
            json!({"a": {"b": 1}}),
            json!({"a": false}),
            Patch::new(r#"[
                {"op":"replace","path":"/a","value":false}
            ]"#).unwrap()
        },
        replace_scalar_with_object_member = {
            json!({"a": false}),
            json!({"a": {"b": 1}}),
            Patch::new(r#"[
                {"op":"replace","path":"/a","value":{"b":1}}
            ]"#).unwrap()
        },
        escaped_object_keys = {
            json!({"a/b": {"c~d": 1}}),
            json!({"a/b": {"c~d": 2, "e/f": 3}}),
            Patch::new(r#"[
                {"op":"replace","path":"/a~1b/c~0d","value":2},
                {"op":"add","path":"/a~1b/e~1f","value":3}
            ]"#).unwrap()
        },
        empty_object_key = {
            json!({"": 1}),
            json!({"": 2}),
            Patch::new(r#"[
                {"op":"replace","path":"/","value":2}
            ]"#).unwrap()
        },
        multiple_root_object_changes = {
            json!({"a": 1, "b": 2, "c": 3}),
            json!({"a": 10, "c": 3, "d": 4}),
            Patch::new(r#"[
                {"op":"replace","path":"/a","value":10},
                {"op":"remove","path":"/b"},
                {"op":"add","path":"/d","value":4}
            ]"#).unwrap()
        },
        equal_arrays = {
            json!([1, 2, 3]),
            json!([1, 2, 3]),
            Patch::empty()
        },
        replace_array_item = {
            json!([1, 2, 3]),
            json!([1, 9, 3]),
            Patch::new(r#"[
                {"op":"replace","path":"/1","value":9}
            ]"#).unwrap()
        },
        append_array_item = {
            json!([1, 2]),
            json!([1, 2, 3]),
            Patch::new(r#"[
                {"op":"add","path":"/2","value":3}
            ]"#).unwrap()
        },
        prepend_array_item = {
            json!([2, 3]),
            json!([1, 2, 3]),
            Patch::new(r#"[
                {"op":"add","path":"/0","value":1}
            ]"#).unwrap()
        },
        insert_array_item = {
            json!([1, 2, 4]),
            json!([1, 2, 3, 4]),
            Patch::new(r#"[
                {"op":"add","path":"/2","value":3}
            ]"#).unwrap()
        },
        remove_array_item = {
            json!([1, 2, 3, 4]),
            json!([1, 2, 4]),
            Patch::new(r#"[
                {"op":"remove","path":"/2"}
            ]"#).unwrap()
        },
        truncate_array_items = {
            json!([1, 2, 3, 4]),
            json!([1, 2]),
            Patch::new(r#"[
                {"op":"remove","path":"/3"},
                {"op":"remove","path":"/2"}
            ]"#).unwrap()
        },
        remove_array_prefix_items = {
            json!([1, 2, 3, 4]),
            json!([3, 4]),
            Patch::new(r#"[
                {"op":"remove","path":"/1"},
                {"op":"remove","path":"/0"}
            ]"#).unwrap()
        },
        replace_and_append_array_items = {
            json!([1, 2, 3]),
            json!([1, 8, 9, 10]),
            Patch::new(r#"[
                {"op":"replace","path":"/1","value":8},
                {"op":"replace","path":"/2","value":9},
                {"op":"add","path":"/3","value":10}
            ]"#).unwrap()
        },
        replace_and_remove_array_items = {
            json!([1, 2, 3, 4]),
            json!([1, 8]),
            Patch::new(r#"[
                {"op":"replace","path":"/1","value":8},
                {"op":"remove","path":"/3"},
                {"op":"remove","path":"/2"}
            ]"#).unwrap()
        },
        reverse_array_replaces_each_item = {
            json!([1, 2, 3, 4]),
            json!([4, 3, 2, 1]),
            Patch::new(r#"[
                {"op":"replace","path":"/0","value":4},
                {"op":"replace","path":"/1","value":3},
                {"op":"replace","path":"/2","value":2},
                {"op":"replace","path":"/3","value":1}
            ]"#).unwrap()
        },
        array_nested_in_object = {
            json!({"items": [1, 2, 4]}),
            json!({"items": [1, 2, 3, 4]}),
            Patch::new(r#"[
                {"op":"add","path":"/items/2","value":3}
            ]"#).unwrap()
        },
        object_nested_in_array = {
            json!([{"a": 1, "b": 2}]),
            json!([{"a": 1, "b": 3, "c": 4}]),
            Patch::new(r#"[
                {"op":"replace","path":"/0/b","value":3},
                {"op":"add","path":"/0/c","value":4}
            ]"#).unwrap()
        },
        replace_array_with_object = {
            json!([1, 2]),
            json!({"a": 1}),
            Patch::new(r#"[
                {"op":"replace","path":"","value":{"a":1}}
            ]"#).unwrap()
        },
        replace_object_with_array = {
            json!({"a": 1}),
            json!([1, 2]),
            Patch::new(r#"[
                {"op":"replace","path":"","value":[1,2]}
            ]"#).unwrap()
        },
    )]
    fn calculcate_correct_diff(source: Value, target: Value, result: Patch) {
        let calculated_diff = diff(&source, &target);
        assert_eq!(calculated_diff, result);
    }
}
