use std::ops::ControlFlow;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::pointer::Pointer;

#[derive(Error, Debug, PartialEq)]
/// Errors that can occur while applying a JSON Patch operation.
pub enum ApplicationError {
    /// The operation cannot be applied to the target document.
    #[error("operation is not applicable: {0}")]
    NotApplicable(PatchOperation),

    /// The addressed JSON Pointer path does not exist.
    #[error("path not found: {0}")]
    PathNotFound(Pointer),

    /// The addressed array index is outside the valid range.
    #[error("array index out of bounds at path: {0}")]
    ArrayOutOfBounds(Pointer),

    /// The addressed array index is not valid JSON Patch array syntax.
    #[error("invalid array index at path: {0}")]
    ArraySyntaxError(Pointer),

    /// A `test` operation failed because the actual value did not equal the expected value.
    #[error(
        "test operation failed at path: {path}, expected value: {test_value}, actual value: {value}"
    )]
    TestFailed {
        /// The path whose value was tested.
        path: Pointer,
        /// The actual value found at `path`.
        value: Value,
        /// The expected value supplied by the operation.
        test_value: Value,
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Hash)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "op")]
/// A single JSON Patch operation as defined by RFC 6902.
pub enum PatchOperation {
    /// Adds `value` at `path`.
    ///
    /// Object members are inserted or replaced. Array targets insert at the
    /// addressed index, and `-` appends to the end.
    Add {
        /// Destination path.
        path: Pointer,
        /// Value to add.
        value: Value,
    },
    /// Removes the value at `path`.
    Remove {
        /// Path to remove.
        path: Pointer,
    },
    /// Replaces the existing value at `path` with `value`.
    Replace {
        /// Path to replace.
        path: Pointer,
        /// Replacement value.
        value: Value,
    },
    /// Moves the value from `from` to `path`.
    ///
    /// This is equivalent to removing the source value and adding it at the
    /// destination path.
    Move {
        /// Source path.
        from: Pointer,
        /// Destination path.
        path: Pointer,
    },
    /// Copies the value from `from` to `path`.
    Copy {
        /// Source path.
        from: Pointer,
        /// Destination path.
        path: Pointer,
    },
    /// Tests whether the value at `path` equals `value`.
    Test {
        /// Path to test.
        path: Pointer,
        /// Expected value.
        value: Value,
    },
}

impl PatchOperation {
    pub(crate) fn apply(&self, target: &mut Value) -> Result<(), ApplicationError> {
        let path = self.get_path();
        let (parent, child) = pointer_split(path);
        let parent = parent
            .get_mut(target)
            .ok_or(ApplicationError::PathNotFound(parent.clone()))?;

        if matches!(
            self,
            PatchOperation::Add { .. } | PatchOperation::Remove { .. }
        ) && parent.is_array()
            && child.is_none()
        {
            return Err(ApplicationError::ArraySyntaxError(path.clone()));
        }

        match self {
            PatchOperation::Add { value, .. } => {
                // special case is array. otherwise just set the value.
                if parent.is_array() {
                    let parent = parent.as_array_mut().unwrap();
                    let prop = child.as_ref().unwrap();
                    let prop = prop.path_without_leading_slash();
                    if prop == "-" {
                        parent.push(value.clone());
                    } else {
                        let index: usize = prop
                            .parse()
                            .map_err(|_| ApplicationError::ArraySyntaxError(path.clone()))?;

                        if index > parent.len() {
                            return Err(ApplicationError::ArrayOutOfBounds(path.clone()));
                        }

                        parent.insert(index, value.clone());
                    }

                    return Ok(());
                }

                if child.is_none() {
                    *parent = value.clone();
                    return Ok(());
                }

                parent[child.unwrap().path_without_leading_slash()] = value.clone();

                Ok(())
            }
            PatchOperation::Remove { .. } => {
                if parent.is_array() {
                    let parent = parent.as_array_mut().unwrap();
                    let prop = child.as_ref().unwrap();
                    let prop = prop.path_without_leading_slash();
                    let index: usize = prop
                        .parse()
                        .map_err(|_| ApplicationError::ArraySyntaxError(path.clone()))?;

                    if index >= parent.len() {
                        return Err(ApplicationError::ArrayOutOfBounds(path.clone()));
                    }

                    parent.remove(index);
                    return Ok(());
                }

                if parent.is_object() && child.is_none() {
                    return Err(ApplicationError::PathNotFound(path.clone()));
                } else if parent.is_object() {
                    let parent = parent.as_object_mut().unwrap();
                    let key = child.as_ref().unwrap().tokens().join("/");
                    if !parent.contains_key(&key) {
                        return Err(ApplicationError::PathNotFound(path.clone()));
                    }

                    parent.remove(&key);
                    return Ok(());
                }

                // special case: removing the root is not valid because
                // it will return an invalid json document (empty/void).

                Err(ApplicationError::NotApplicable(self.clone()))
            }
            PatchOperation::Replace { value, .. } => {
                let target = path
                    .get_mut(target)
                    .ok_or(ApplicationError::PathNotFound(path.clone()))?;
                *target = value.clone();
                Ok(())
            }
            PatchOperation::Move { from, .. } => {
                let proper_prefix = {
                    let from_tokens = from.tokens();
                    let path_tokens = path.tokens();

                    from_tokens.len() < path_tokens.len()
                        && from_tokens
                            .iter()
                            .zip(path_tokens.iter())
                            .all(|(left, right)| left == right)
                };
                if proper_prefix {
                    return Err(ApplicationError::NotApplicable(self.clone()));
                }

                let mut clone = target.clone();

                // the RFC specifies the move operation as "remove then add".
                // thus, we execute a remove with optional failure, then add.
                let value = from
                    .get(&clone)
                    .ok_or(ApplicationError::PathNotFound(from.clone()))?
                    .clone();

                // this works because if the root (or parent) is an array or object,
                // remove will work. the only case where remove fails is when
                // the root is removed. so, we ignore the error and just "add"
                // over the root anyway.
                let remove = PatchOperation::Remove { path: from.clone() }.apply(&mut clone);
                match remove {
                    Ok(..) | Err(ApplicationError::NotApplicable(..)) => {}
                    Err(e) => return Err(e),
                }
                PatchOperation::Add {
                    path: path.clone(),
                    value: value,
                }
                .apply(&mut clone)?;

                *target = clone;

                Ok(())
            }
            PatchOperation::Copy { from, .. } => {
                let value = from
                    .get(target)
                    .ok_or(ApplicationError::PathNotFound(from.clone()))?
                    .clone();

                PatchOperation::Add {
                    path: path.clone(),
                    value: value,
                }
                .apply(target)
            }
            PatchOperation::Test { value, .. } => {
                // vut = value under test
                let vut = path
                    .get(target)
                    .ok_or(ApplicationError::PathNotFound(path.clone()))?;

                fn compare_values(left: &Value, right: &Value) -> bool {
                    match (left, right) {
                        (Value::Null, Value::Null) => true,
                        (Value::Bool(a), Value::Bool(b)) if a == b => true,
                        (Value::Number(a), Value::Number(b)) if a == b => true,
                        (Value::String(a), Value::String(b)) if a == b => true,
                        (Value::Array(a), Value::Array(b))
                            if a.len() == b.len()
                                && a.iter().zip(b.iter()).try_fold(true, |v, (a, b)| {
                                    if v && compare_values(a, b) {
                                        ControlFlow::Continue(true)
                                    } else {
                                        ControlFlow::Break(false)
                                    }
                                }) == ControlFlow::Continue(true) =>
                        {
                            true
                        }
                        (Value::Object(a), Value::Object(b))
                            // we can compare the map key by key since serdes "preserve order" feature is not enabled
                            if a.len() == b.len() && a.iter().zip(b.iter()).try_fold(true, |v, ((k_a, v_a), (k_b, v_b))|{
                                if v && k_a == k_b && compare_values(v_a, v_b){
                                    ControlFlow::Continue(true)
                                } else {
                                    ControlFlow::Break(false)
                                }
                            }) == ControlFlow::Continue(true) =>
                        {
                            true
                        }
                        _ => false,
                    }
                }

                if compare_values(vut, value) {
                    Ok(())
                } else {
                    Err(ApplicationError::TestFailed {
                        path: path.clone(),
                        value: vut.clone(),
                        test_value: value.clone(),
                    })
                }
            }
        }
    }

    fn get_path(&self) -> &Pointer {
        match self {
            PatchOperation::Add { path, .. } => path,
            PatchOperation::Remove { path } => path,
            PatchOperation::Replace { path, .. } => path,
            PatchOperation::Move { path, .. } => path,
            PatchOperation::Copy { path, .. } => path,
            PatchOperation::Test { path, .. } => path,
        }
    }
}

impl std::fmt::Display for PatchOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatchOperation::Add { path, value } => write!(f, "Add {}: {}", path, value),
            PatchOperation::Remove { path } => write!(f, "Remove {}", path),
            PatchOperation::Replace { path, value } => write!(f, "Replace {}: {}", path, value),
            PatchOperation::Move { from, path } => write!(f, "Move from {} to {}", from, path),
            PatchOperation::Copy { from, path } => write!(f, "Copy from {} to {}", from, path),
            PatchOperation::Test { path, value } => write!(f, "Test {}: {}", path, value),
        }
    }
}

fn pointer_split(p: &Pointer) -> (Pointer, Option<Pointer>) {
    let tokens = p.raw_tokens();
    if tokens.is_empty() {
        return (Pointer::new(""), None);
    }

    let parent_tokens = &tokens[..tokens.len() - 1];
    if parent_tokens.is_empty() {
        return (
            Pointer::new(""),
            Some(Pointer::new(format!("/{}", tokens[0]))),
        );
    }

    let child_token = &tokens[tokens.len() - 1];
    (
        Pointer::new(format!("/{}", parent_tokens.join("/"))),
        Some(Pointer::new(format!("/{}", child_token))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use yare::parameterized;

    #[parameterized(
        root = { "", "", None },
        root_member = { "/foo", "", Some("/foo") },
        nested_member = { "/foo/bar", "/foo", Some("/bar") },
        array_index = { "/foo/0", "/foo", Some("/0") },
        array_append = { "/foo/-", "/foo", Some("/-") },
        empty_member_at_root = { "/", "", Some("/") },
        empty_nested_member = { "/foo/", "/foo", Some("/") },
        escaped_tokens = { "/a~1b/c~0d", "/a~1b", Some("/c~0d") },
    )]
    fn pointer_split_returns_parent_and_child_property(
        path: &str,
        expected_parent: &str,
        expected_child: Option<&str>,
    ) {
        let (parent, child) = pointer_split(&Pointer::new(path));

        assert_eq!(parent, Pointer::new(expected_parent));
        assert_eq!(child, expected_child.map(Pointer::new));
    }

    #[parameterized(
        add_to_end_of_array = {
            PatchOperation::Add { path: "/arr/-".into(), value: json!(4) },
            json!({"arr": [1,2,3]}),
            json!({"arr": [1,2,3,4]})
        },
        add_other_type_to_end_of_array = {
            PatchOperation::Add { path: "/arr/-".into(), value: json!(true) },
            json!({"arr": [1,2,3]}),
            json!({"arr": [1,2,3,true]})
        },
        add_at_start_of_array = {
            PatchOperation::Add { path: "/arr/0".into(), value: json!(0) },
            json!({"arr": [1,2,3]}),
            json!({"arr": [0,1,2,3]})
        },
        add_in_middle_of_array = {
            PatchOperation::Add { path: "/arr/1".into(), value: json!(1.5) },
            json!({"arr": [1,2,3]}),
            json!({"arr": [1,1.5,2,3]})
        },
        add_to_array_at_root = {
            PatchOperation::Add { path: "/-".into(), value: json!(4) },
            json!([1,2,3]),
            json!([1,2,3,4])
        },
        add_to_object = {
            PatchOperation::Add { path: "/foo".into(), value: json!(1) },
            json!({}),
            json!({"foo": 1})
        },
        add_object_member_rfc = {
            PatchOperation::Add { path: "/baz".into(), value: json!("qux") },
            json!({"foo": "bar"}),
            json!({"foo": "bar", "baz": "qux"})
        },
        add_replaces_existing_object_member = {
            PatchOperation::Add { path: "/foo".into(), value: json!("qux") },
            json!({"foo": "bar"}),
            json!({"foo": "qux"})
        },
        add_to_nested_object = {
            PatchOperation::Add { path: "/foo/baz".into(), value: json!("qux") },
            json!({"foo": {"bar": 1}}),
            json!({"foo": {"bar": 1, "baz": "qux"}})
        },
        add_nested_member_object_rfc = {
            PatchOperation::Add { path: "/child".into(), value: json!({"grandchild": {}}) },
            json!({"foo": "bar"}),
            json!({"foo": "bar", "child": {"grandchild": {}}})
        },
        add_replace_root = {
            PatchOperation::Add { path: "".into(), value: json!(1) },
            json!({}),
            json!(1)
        },
        add_replaces_root_string = {
            PatchOperation::Add { path: "".into(), value: json!("qux") },
            json!("bar"),
            json!("qux")
        },
        add_replaces_root_bool = {
            PatchOperation::Add { path: "".into(), value: json!(false) },
            json!(true),
            json!(false)
        },
        add_replaces_root_null = {
            PatchOperation::Add { path: "".into(), value: json!({"foo": "bar"}) },
            json!(null),
            json!({"foo": "bar"})
        },
        remove_object_member = {
            PatchOperation::Remove { path: "/foo".into() },
            json!({"foo": "bar", "baz": "qux"}),
            json!({"baz": "qux"})
        },
        remove_object_member_rfc = {
            PatchOperation::Remove { path: "/baz".into() },
            json!({"baz": "qux", "foo": "bar"}),
            json!({"foo": "bar"})
        },
        remove_nested_member_rfc = {
            PatchOperation::Remove { path: "/a/b/c".into() },
            json!({"a": {"b": {"c": "foo", "d": "bar"}}}),
            json!({"a": {"b": {"d": "bar"}}})
        },
        remove_nested_object_member = {
            PatchOperation::Remove { path: "/foo/bar".into() },
            json!({"foo": {"bar": true, "baz": false}}),
            json!({"foo": {"baz": false}})
        },
        remove_empty_object_member_name = {
            PatchOperation::Remove { path: "/".into() },
            json!({"": "empty", "foo": "bar"}),
            json!({"foo": "bar"})
        },
        remove_escaped_object_member_name = {
            PatchOperation::Remove { path: "/a~1b/c~0d".into() },
            json!({"a/b": {"c~d": 1, "e": 2}}),
            json!({"a/b": {"e": 2}})
        },
        remove_first_array_item = {
            PatchOperation::Remove { path: "/arr/0".into() },
            json!({"arr": [1, 2, 3]}),
            json!({"arr": [2, 3]})
        },
        remove_middle_array_item = {
            PatchOperation::Remove { path: "/arr/1".into() },
            json!({"arr": [1, 2, 3]}),
            json!({"arr": [1, 3]})
        },
        remove_array_element_rfc = {
            PatchOperation::Remove { path: "/foo/1".into() },
            json!({"foo": ["bar", "qux", "baz"]}),
            json!({"foo": ["bar", "baz"]})
        },
        remove_last_array_item = {
            PatchOperation::Remove { path: "/arr/2".into() },
            json!({"arr": [1, 2, 3]}),
            json!({"arr": [1, 2]})
        },
        remove_array_item_at_root = {
            PatchOperation::Remove { path: "/1".into() },
            json!(["foo", "bar", "baz"]),
            json!(["foo", "baz"])
        },
        replace_object_member_rfc = {
            PatchOperation::Replace { path: "/baz".into(), value: json!("boo") },
            json!({"baz": "qux", "foo": "bar"}),
            json!({"baz": "boo", "foo": "bar"})
        },
        replace_nested_member_rfc = {
            PatchOperation::Replace { path: "/a/b/c".into(), value: json!(42) },
            json!({"a": {"b": {"c": "foo", "d": "bar"}}}),
            json!({"a": {"b": {"c": 42, "d": "bar"}}})
        },
        replace_object_member_with_object = {
            PatchOperation::Replace { path: "/foo".into(), value: json!({"bar": true}) },
            json!({"foo": "bar", "baz": "qux"}),
            json!({"foo": {"bar": true}, "baz": "qux"})
        },
        replace_empty_object_member_name = {
            PatchOperation::Replace { path: "/".into(), value: json!("replaced") },
            json!({"": "empty", "foo": "bar"}),
            json!({"": "replaced", "foo": "bar"})
        },
        replace_escaped_object_member_name = {
            PatchOperation::Replace { path: "/a~1b/c~0d".into(), value: json!(false) },
            json!({"a/b": {"c~d": true, "e": 2}}),
            json!({"a/b": {"c~d": false, "e": 2}})
        },
        replace_array_element = {
            PatchOperation::Replace { path: "/arr/1".into(), value: json!("two") },
            json!({"arr": [1, 2, 3]}),
            json!({"arr": [1, "two", 3]})
        },
        replace_array_element_with_array = {
            PatchOperation::Replace { path: "/arr/0".into(), value: json!(["nested"]) },
            json!({"arr": [1, 2, 3]}),
            json!({"arr": [["nested"], 2, 3]})
        },
        replace_array_element_at_root = {
            PatchOperation::Replace { path: "/1".into(), value: json!("bar") },
            json!(["foo", "qux", "baz"]),
            json!(["foo", "bar", "baz"])
        },
        replace_root_with_string = {
            PatchOperation::Replace { path: "".into(), value: json!("root") },
            json!({"foo": "bar"}),
            json!("root")
        },
        replace_root_with_number = {
            PatchOperation::Replace { path: "".into(), value: json!(42) },
            json!({"foo": "bar"}),
            json!(42)
        },
        replace_root_with_bool = {
            PatchOperation::Replace { path: "".into(), value: json!(false) },
            json!({"foo": "bar"}),
            json!(false)
        },
        replace_root_with_null = {
            PatchOperation::Replace { path: "".into(), value: json!(null) },
            json!({"foo": "bar"}),
            json!(null)
        },
        replace_array_root_with_string = {
            PatchOperation::Replace { path: "".into(), value: json!("root") },
            json!([1, 2, 3]),
            json!("root")
        },
        move_object_member_rfc = {
            PatchOperation::Move { from: "/foo/waldo".into(), path: "/qux/thud".into() },
            json!({"foo": {"bar": "baz", "waldo": "fred"}, "qux": {"corge": "grault"}}),
            json!({"foo": {"bar": "baz"}, "qux": {"corge": "grault", "thud": "fred"}})
        },
        move_array_element_rfc = {
            PatchOperation::Move { from: "/foo/1".into(), path: "/foo/3".into() },
            json!({"foo": ["all", "grass", "cows", "eat"]}),
            json!({"foo": ["all", "cows", "eat", "grass"]})
        },
        move_object_member_to_new_member = {
            PatchOperation::Move { from: "/foo".into(), path: "/baz".into() },
            json!({"foo": "bar"}),
            json!({"baz": "bar"})
        },
        move_object_member_replaces_existing_member = {
            PatchOperation::Move { from: "/foo".into(), path: "/baz".into() },
            json!({"foo": "bar", "baz": "qux"}),
            json!({"baz": "bar"})
        },
        move_nested_object_member = {
            PatchOperation::Move { from: "/foo/bar".into(), path: "/baz/qux".into() },
            json!({"foo": {"bar": true}, "baz": {}}),
            json!({"foo": {}, "baz": {"qux": true}})
        },
        move_escaped_object_member_name = {
            PatchOperation::Move { from: "/a~1b/c~0d".into(), path: "/target".into() },
            json!({"a/b": {"c~d": 1, "e": 2}}),
            json!({"a/b": {"e": 2}, "target": 1})
        },
        move_array_element_to_start = {
            PatchOperation::Move { from: "/arr/2".into(), path: "/arr/0".into() },
            json!({"arr": [1, 2, 3]}),
            json!({"arr": [3, 1, 2]})
        },
        move_array_element_to_end = {
            PatchOperation::Move { from: "/arr/0".into(), path: "/arr/-".into() },
            json!({"arr": [1, 2, 3]}),
            json!({"arr": [2, 3, 1]})
        },
        move_child_replaces_parent = {
            PatchOperation::Move { from: "/foo/bar".into(), path: "/foo".into() },
            json!({"foo": {"bar": "baz"}, "qux": true}),
            json!({"foo": "baz", "qux": true})
        },
        move_array_child_replaces_parent_element = {
            PatchOperation::Move { from: "/arr/0/1".into(), path: "/arr/0".into() },
            json!({"arr": [[1, 2], 3]}),
            json!({"arr": [2, [1], 3]})
        },
        move_root_replaced_by_child = {
            PatchOperation::Move { from: "/foo".into(), path: "".into() },
            json!({"foo": {"bar": "baz"}, "qux": true}),
            json!({"bar": "baz"})
        },
        move_root_replaced_by_primitive_child = {
            PatchOperation::Move { from: "/foo".into(), path: "".into() },
            json!({"foo": "bar", "qux": true}),
            json!("bar")
        },
        move_root_replaced_by_array_child_primitive = {
            PatchOperation::Move { from: "/arr/0".into(), path: "".into() },
            json!({"arr": [1, 2, 3]}),
            json!(1)
        },
        copy_object_member_rfc = {
            PatchOperation::Copy { from: "/a/b/c".into(), path: "/a/b/e".into() },
            json!({"a": {"b": {"c": "foo", "d": "bar"}}}),
            json!({"a": {"b": {"c": "foo", "d": "bar", "e": "foo"}}})
        },
        copy_object_member_to_new_member = {
            PatchOperation::Copy { from: "/foo".into(), path: "/baz".into() },
            json!({"foo": "bar"}),
            json!({"foo": "bar", "baz": "bar"})
        },
        copy_object_member_replaces_existing_member = {
            PatchOperation::Copy { from: "/foo".into(), path: "/baz".into() },
            json!({"foo": "bar", "baz": "qux"}),
            json!({"foo": "bar", "baz": "bar"})
        },
        copy_nested_object_member = {
            PatchOperation::Copy { from: "/foo/bar".into(), path: "/baz/qux".into() },
            json!({"foo": {"bar": true}, "baz": {}}),
            json!({"foo": {"bar": true}, "baz": {"qux": true}})
        },
        copy_escaped_object_member_name = {
            PatchOperation::Copy { from: "/a~1b/c~0d".into(), path: "/target".into() },
            json!({"a/b": {"c~d": 1, "e": 2}}),
            json!({"a/b": {"c~d": 1, "e": 2}, "target": 1})
        },
        copy_array_element_to_start = {
            PatchOperation::Copy { from: "/arr/2".into(), path: "/arr/0".into() },
            json!({"arr": [1, 2, 3]}),
            json!({"arr": [3, 1, 2, 3]})
        },
        copy_array_element_to_end = {
            PatchOperation::Copy { from: "/arr/0".into(), path: "/arr/-".into() },
            json!({"arr": [1, 2, 3]}),
            json!({"arr": [1, 2, 3, 1]})
        },
        copy_array_element_to_index_at_end = {
            PatchOperation::Copy { from: "/arr/0".into(), path: "/arr/3".into() },
            json!({"arr": [1, 2, 3]}),
            json!({"arr": [1, 2, 3, 1]})
        },
        copy_child_to_root = {
            PatchOperation::Copy { from: "/foo".into(), path: "".into() },
            json!({"foo": {"bar": "baz"}, "qux": true}),
            json!({"bar": "baz"})
        },
        copy_primitive_child_to_root = {
            PatchOperation::Copy { from: "/foo".into(), path: "".into() },
            json!({"foo": "bar", "qux": true}),
            json!("bar")
        },
        copy_root_to_object_member = {
            PatchOperation::Copy { from: "".into(), path: "/copy".into() },
            json!({"foo": "bar"}),
            json!({"foo": "bar", "copy": {"foo": "bar"}})
        }
    )]
    fn apply_operation(op: PatchOperation, target: Value, expected: Value) {
        let mut target = target;
        op.apply(&mut target).expect("Testing Expect");

        assert_eq!(target, expected);
    }

    #[parameterized(
        test_object_member_rfc = {
            PatchOperation::Test { path: "/baz".into(), value: json!("qux") },
            json!({"baz": "qux", "foo": [ "a", 2, "c" ]})
        },
        test_array_element_rfc = {
            PatchOperation::Test { path: "/foo/1".into(), value: json!(2) },
            json!({"baz": "qux", "foo": [ "a", 2, "c" ]})
        },
        test_root_object = {
            PatchOperation::Test { path: "".into(), value: json!({"foo": "bar"}) },
            json!({"foo": "bar"})
        },
        test_root_string = {
            PatchOperation::Test { path: "".into(), value: json!("foo") },
            json!("foo")
        },
        test_root_number = {
            PatchOperation::Test { path: "".into(), value: json!(42) },
            json!(42)
        },
        test_root_bool = {
            PatchOperation::Test { path: "".into(), value: json!(true) },
            json!(true)
        },
        test_root_null = {
            PatchOperation::Test { path: "".into(), value: json!(null) },
            json!(null)
        },
        test_array_deep_equal = {
            PatchOperation::Test { path: "/foo".into(), value: json!(["a", {"b": [1, true, null]}]) },
            json!({"foo": ["a", {"b": [1, true, null]}]})
        },
        test_object_deep_equal = {
            PatchOperation::Test { path: "/foo".into(), value: json!({"bar": {"baz": [1, 2]}}) },
            json!({"foo": {"bar": {"baz": [1, 2]}}})
        },
        test_escaped_object_member_name = {
            PatchOperation::Test { path: "/a~1b/c~0d".into(), value: json!(1) },
            json!({"a/b": {"c~d": 1}})
        },
        test_strings_and_numbers_rfc_number = {
            PatchOperation::Test { path: "/~01".into(), value: json!(10) },
            json!({"/": 9, "~1": 10})
        },
        test_strings_and_numbers_rfc_string = {
            PatchOperation::Test { path: "/~01".into(), value: json!("10") },
            json!({"/": 9, "~1": "10"})
        }
    )]
    fn apply_test_operation(op: PatchOperation, target: Value) {
        let mut target = target;
        let expected = target.clone();

        op.apply(&mut target).expect("Testing Expect");

        assert_eq!(target, expected);
    }

    #[parameterized(
        add_to_missing_array = {
            PatchOperation::Add { path: "/missing/0".into(), value: json!(4) },
            json!({"arr": [1,2,3]}),
            ApplicationError::PathNotFound("/missing".into())
        },
        add_to_missing_nested_array = {
            PatchOperation::Add { path: "/parent/missing/0".into(), value: json!(4) },
            json!({"parent": {"arr": [1,2,3]}}),
            ApplicationError::PathNotFound("/parent/missing".into())
        },
        add_to_missing_object_parent_rfc = {
            PatchOperation::Add { path: "/baz/bat".into(), value: json!("qux") },
            json!({"foo": "bar"}),
            ApplicationError::PathNotFound("/baz".into())
        },
        add_to_missing_nested_object_parent = {
            PatchOperation::Add { path: "/foo/baz/bat".into(), value: json!("qux") },
            json!({"foo": {}}),
            ApplicationError::PathNotFound("/foo/baz".into())
        },
        add_with_missing_array_index = {
            PatchOperation::Add { path: "".into(), value: json!(4) },
            json!([1,2,3]),
            ApplicationError::ArraySyntaxError("".into())
        },
        add_with_non_numeric_array_index = {
            PatchOperation::Add { path: "/arr/foo".into(), value: json!(4) },
            json!({"arr": [1,2,3]}),
            ApplicationError::ArraySyntaxError("/arr/foo".into())
        },
        add_with_negative_array_index = {
            PatchOperation::Add { path: "/arr/-1".into(), value: json!(4) },
            json!({"arr": [1,2,3]}),
            ApplicationError::ArraySyntaxError("/arr/-1".into())
        },
        add_past_array_end = {
            PatchOperation::Add { path: "/arr/4".into(), value: json!(4) },
            json!({"arr": [1,2,3]}),
            ApplicationError::ArrayOutOfBounds("/arr/4".into())
        },
        remove_missing_object_member = {
            PatchOperation::Remove { path: "/missing".into() },
            json!({"foo": "bar"}),
            ApplicationError::PathNotFound("/missing".into())
        },
        remove_missing_nested_object_member = {
            PatchOperation::Remove { path: "/foo/missing".into() },
            json!({"foo": {"bar": "baz"}}),
            ApplicationError::PathNotFound("/foo/missing".into())
        },
        remove_from_missing_parent = {
            PatchOperation::Remove { path: "/missing/foo".into() },
            json!({"foo": "bar"}),
            ApplicationError::PathNotFound("/missing".into())
        },
        remove_from_empty_path_in_array = {
            PatchOperation::Remove { path: "".into() },
            json!([1, 2, 3]),
            ApplicationError::ArraySyntaxError("".into())
        },
        remove_with_non_numeric_array_index = {
            PatchOperation::Remove { path: "/arr/foo".into() },
            json!({"arr": [1, 2, 3]}),
            ApplicationError::ArraySyntaxError("/arr/foo".into())
        },
        remove_with_append_array_index = {
            PatchOperation::Remove { path: "/arr/-".into() },
            json!({"arr": [1, 2, 3]}),
            ApplicationError::ArraySyntaxError("/arr/-".into())
        },
        remove_with_negative_array_index = {
            PatchOperation::Remove { path: "/arr/-1".into() },
            json!({"arr": [1, 2, 3]}),
            ApplicationError::ArraySyntaxError("/arr/-1".into())
        },
        remove_past_array_end = {
            PatchOperation::Remove { path: "/arr/3".into() },
            json!({"arr": [1, 2, 3]}),
            ApplicationError::ArrayOutOfBounds("/arr/3".into())
        },
        remove_from_object_root = {
            PatchOperation::Remove { path: "".into() },
            json!({"foo": "bar"}),
            ApplicationError::PathNotFound("".into())
        },
        remove_from_string = {
            PatchOperation::Remove { path: "/foo".into() },
            json!("bar"),
            ApplicationError::NotApplicable(PatchOperation::Remove { path: "/foo".into() })
        },
        replace_missing_object_member = {
            PatchOperation::Replace { path: "/missing".into(), value: json!("bar") },
            json!({"foo": "bar"}),
            ApplicationError::PathNotFound("/missing".into())
        },
        replace_missing_nested_object_member = {
            PatchOperation::Replace { path: "/foo/missing".into(), value: json!("bar") },
            json!({"foo": {"bar": "baz"}}),
            ApplicationError::PathNotFound("/foo/missing".into())
        },
        replace_from_missing_parent = {
            PatchOperation::Replace { path: "/missing/foo".into(), value: json!("bar") },
            json!({"foo": "bar"}),
            ApplicationError::PathNotFound("/missing".into())
        },
        replace_with_non_numeric_array_index = {
            PatchOperation::Replace { path: "/arr/foo".into(), value: json!("bar") },
            json!({"arr": [1, 2, 3]}),
            ApplicationError::PathNotFound("/arr/foo".into())
        },
        replace_with_append_array_index = {
            PatchOperation::Replace { path: "/arr/-".into(), value: json!("bar") },
            json!({"arr": [1, 2, 3]}),
            ApplicationError::PathNotFound("/arr/-".into())
        },
        replace_with_negative_array_index = {
            PatchOperation::Replace { path: "/arr/-1".into(), value: json!("bar") },
            json!({"arr": [1, 2, 3]}),
            ApplicationError::PathNotFound("/arr/-1".into())
        },
        replace_past_array_end = {
            PatchOperation::Replace { path: "/arr/3".into(), value: json!("bar") },
            json!({"arr": [1, 2, 3]}),
            ApplicationError::PathNotFound("/arr/3".into())
        },
        replace_child_of_string = {
            PatchOperation::Replace { path: "/foo".into(), value: json!("bar") },
            json!("qux"),
            ApplicationError::PathNotFound("/foo".into())
        },
        move_from_missing_source = {
            PatchOperation::Move { from: "/missing".into(), path: "/foo".into() },
            json!({"foo": "bar"}),
            ApplicationError::PathNotFound("/missing".into())
        },
        move_to_missing_parent = {
            PatchOperation::Move { from: "/foo".into(), path: "/missing/bar".into() },
            json!({"foo": "bar"}),
            ApplicationError::PathNotFound("/missing".into())
        },
        move_to_child_of_source = {
            PatchOperation::Move { from: "/foo".into(), path: "/foo/bar".into() },
            json!({"foo": {"baz": "qux"}}),
            ApplicationError::NotApplicable(PatchOperation::Move {
                from: "/foo".into(),
                path: "/foo/bar".into()
            })
        },
        move_with_non_numeric_array_destination = {
            PatchOperation::Move { from: "/foo".into(), path: "/arr/foo".into() },
            json!({"foo": "bar", "arr": [1, 2, 3]}),
            ApplicationError::ArraySyntaxError("/arr/foo".into())
        },
        move_past_array_end = {
            PatchOperation::Move { from: "/foo".into(), path: "/arr/4".into() },
            json!({"foo": "bar", "arr": [1, 2, 3]}),
            ApplicationError::ArrayOutOfBounds("/arr/4".into())
        },
        copy_from_missing_source = {
            PatchOperation::Copy { from: "/missing".into(), path: "/foo".into() },
            json!({"foo": "bar"}),
            ApplicationError::PathNotFound("/missing".into())
        },
        copy_to_missing_parent = {
            PatchOperation::Copy { from: "/foo".into(), path: "/missing/bar".into() },
            json!({"foo": "bar"}),
            ApplicationError::PathNotFound("/missing".into())
        },
        copy_with_non_numeric_array_destination = {
            PatchOperation::Copy { from: "/foo".into(), path: "/arr/foo".into() },
            json!({"foo": "bar", "arr": [1, 2, 3]}),
            ApplicationError::ArraySyntaxError("/arr/foo".into())
        },
        copy_past_array_end = {
            PatchOperation::Copy { from: "/foo".into(), path: "/arr/4".into() },
            json!({"foo": "bar", "arr": [1, 2, 3]}),
            ApplicationError::ArrayOutOfBounds("/arr/4".into())
        },
        test_missing_path = {
            PatchOperation::Test { path: "/missing".into(), value: json!("bar") },
            json!({"foo": "bar"}),
            ApplicationError::PathNotFound("/missing".into())
        },
        test_object_member_rfc_failure = {
            PatchOperation::Test { path: "/baz".into(), value: json!("bar") },
            json!({"baz": "qux"}),
            ApplicationError::TestFailed {
                path: "/baz".into(),
                value: json!("qux"),
                test_value: json!("bar")
            }
        },
        test_string_number_mismatch = {
            PatchOperation::Test { path: "/foo".into(), value: json!("1") },
            json!({"foo": 1}),
            ApplicationError::TestFailed {
                path: "/foo".into(),
                value: json!(1),
                test_value: json!("1")
            }
        },
        test_bool_mismatch = {
            PatchOperation::Test { path: "/foo".into(), value: json!(false) },
            json!({"foo": true}),
            ApplicationError::TestFailed {
                path: "/foo".into(),
                value: json!(true),
                test_value: json!(false)
            }
        },
        test_null_mismatch = {
            PatchOperation::Test { path: "/foo".into(), value: json!(false) },
            json!({"foo": null}),
            ApplicationError::TestFailed {
                path: "/foo".into(),
                value: json!(null),
                test_value: json!(false)
            }
        },
        test_array_length_mismatch = {
            PatchOperation::Test { path: "/foo".into(), value: json!([1, 2]) },
            json!({"foo": [1, 2, 3]}),
            ApplicationError::TestFailed {
                path: "/foo".into(),
                value: json!([1, 2, 3]),
                test_value: json!([1, 2])
            }
        },
        test_array_value_mismatch = {
            PatchOperation::Test { path: "/foo".into(), value: json!([1, 3]) },
            json!({"foo": [1, 2]}),
            ApplicationError::TestFailed {
                path: "/foo".into(),
                value: json!([1, 2]),
                test_value: json!([1, 3])
            }
        },
        test_object_value_mismatch = {
            PatchOperation::Test { path: "/foo".into(), value: json!({"bar": 2}) },
            json!({"foo": {"bar": 1}}),
            ApplicationError::TestFailed {
                path: "/foo".into(),
                value: json!({"bar": 1}),
                test_value: json!({"bar": 2})
            }
        },
        test_object_missing_key_mismatch = {
            PatchOperation::Test { path: "/foo".into(), value: json!({"bar": 1}) },
            json!({"foo": {"bar": 1, "baz": 2}}),
            ApplicationError::TestFailed {
                path: "/foo".into(),
                value: json!({"bar": 1, "baz": 2}),
                test_value: json!({"bar": 1})
            }
        }
    )]
    fn apply_operation_error(op: PatchOperation, target: Value, expected_error: ApplicationError) {
        let mut target = target;
        let result = op.apply(&mut target);
        assert!(result.is_err());
        let error = result.err().unwrap();
        assert_eq!(error, expected_error);
    }

    #[parameterized(
        add_child_to_string = {
            PatchOperation::Add { path: "/foo".into(), value: json!("qux") },
            json!("bar")
        },
        add_child_to_number = {
            PatchOperation::Add { path: "/foo".into(), value: json!("qux") },
            json!(1)
        },
        add_child_to_bool = {
            PatchOperation::Add { path: "/foo".into(), value: json!("qux") },
            json!(true)
        },
    )]
    #[should_panic(expected = "cannot access key")]
    fn apply_operation_panics_when_adding_child_to_native_value(op: PatchOperation, target: Value) {
        let mut target = target;
        op.apply(&mut target).unwrap();
    }
}
