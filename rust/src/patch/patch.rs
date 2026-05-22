use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::patch::PatchOperation;

#[derive(Error, Debug)]
/// Errors that can occur while parsing or applying a JSON Patch document.
pub enum PatchError {
    /// The patch document could not be parsed as valid JSON Patch data.
    #[error("patch parse failed: {0}")]
    ParseError(#[from] serde_json::Error),

    /// Applying one of the patch operations failed.
    #[error("patch application failed: {0}")]
    ApplicationError(#[from] crate::patch::operations::ApplicationError),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Hash)]
/// A JSON Patch document.
///
/// The patch stores an ordered list of [`PatchOperation`] values. The order is
/// preserved because patch operations are applied sequentially by JSON Patch
/// processors.
pub struct Patch(Vec<PatchOperation>);

impl Patch {
    /// Creates an empty patch.
    ///
    /// This is equivalent to [`Patch::default`].
    pub fn empty() -> Self {
        Patch::default()
    }

    /// Parses a patch from a JSON string.
    ///
    /// The input must deserialize as a JSON array of [`PatchOperation`] values.
    /// Deserialization errors from `serde_json` are returned unchanged.
    pub fn new(patch_json: &str) -> Result<Self, PatchError> {
        let ops = serde_json::from_str::<Vec<PatchOperation>>(patch_json)?;
        Ok(Patch(ops))
    }

    /// Creates a patch from an existing operation list.
    ///
    /// The operation order is preserved exactly as provided.
    pub fn new_with(ops: Vec<PatchOperation>) -> Self {
        Patch(ops)
    }

    /// Applies this patch to `target` and returns the patched document.
    ///
    /// The input document is cloned before operations are applied, so `target`
    /// is not modified. Operations are applied in order and the first failure
    /// stops application.
    pub fn apply(&self, target: &Value) -> Result<Value, PatchError> {
        let mut target = target.clone();

        for op in &self.0 {
            op.apply(&mut target)?;
        }

        Ok(target)
    }
}

impl From<Vec<PatchOperation>> for Patch {
    /// Converts an operation list into a patch.
    ///
    /// The operation order is preserved exactly as provided.
    fn from(ops: Vec<PatchOperation>) -> Self {
        Patch(ops)
    }
}

impl TryFrom<String> for Patch {
    type Error = PatchError;

    /// Parses a patch from an owned JSON string.
    ///
    /// This delegates to [`Patch::new`].
    fn try_from(patch_json: String) -> Result<Self, Self::Error> {
        Patch::new(&patch_json)
    }
}

impl TryFrom<&str> for Patch {
    type Error = PatchError;

    /// Parses a patch from a borrowed JSON string.
    ///
    /// This delegates to [`Patch::new`].
    fn try_from(patch_json: &str) -> Result<Self, Self::Error> {
        Patch::new(patch_json)
    }
}

impl Default for Patch {
    /// Creates an empty patch.
    fn default() -> Self {
        Patch(vec![])
    }
}

impl std::fmt::Display for Patch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = serde_json::to_string(self).map_err(|_| std::fmt::Error)?;
        write!(f, "{}", result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pointer::Pointer;
    use serde_json::json;
    use yare::parameterized;

    #[test]
    fn empty_creates_patch_without_operations() {
        let patch = Patch::empty();

        assert_eq!(patch.0, Vec::<PatchOperation>::new());
    }

    #[parameterized(
        empty_patch = { "[]", vec![] },
        single_operation = {
            r#"[{"op":"add","path":"/foo","value":1}]"#,
            vec![PatchOperation::Add {
                path: Pointer::new("/foo"),
                value: json!(1),
            }]
        },
        multiple_operations = {
            r#"[
                {"op":"add","path":"/foo","value":1},
                {"op":"remove","path":"/foo"},
                {"op":"replace","path":"/foo","value":2},
                {"op":"move","from":"/foo","path":"/bar"},
                {"op":"copy","from":"/bar","path":"/baz"},
                {"op":"test","path":"/baz","value":2}
            ]"#,
            vec![
                PatchOperation::Add {
                    path: Pointer::new("/foo"),
                    value: json!(1),
                },
                PatchOperation::Remove {
                    path: Pointer::new("/foo"),
                },
                PatchOperation::Replace {
                    path: Pointer::new("/foo"),
                    value: json!(2),
                },
                PatchOperation::Move {
                    from: Pointer::new("/foo"),
                    path: Pointer::new("/bar"),
                },
                PatchOperation::Copy {
                    from: Pointer::new("/bar"),
                    path: Pointer::new("/baz"),
                },
                PatchOperation::Test {
                    path: Pointer::new("/baz"),
                    value: json!(2),
                },
            ]
        },
        extra_properties = {
            r#"[{"op":"add","path":"/foo","value":1, "foo": "bar"}]"#,
            vec![PatchOperation::Add {
                path: Pointer::new("/foo"),
                value: json!(1),
            }]
        },
    )]
    fn new_parses_patch_json(patch_json: &str, expected: Vec<PatchOperation>) {
        let patch = Patch::new(patch_json).unwrap();

        assert_eq!(patch.0, expected);
    }

    #[test]
    fn new_returns_error_for_invalid_json() {
        let result = Patch::new(r#"{"op":"add"}"#);

        assert!(result.is_err());
    }

    #[test]
    fn new_with_preserves_operations() {
        let ops = vec![
            PatchOperation::Add {
                path: Pointer::new("/foo"),
                value: json!(1),
            },
            PatchOperation::Test {
                path: Pointer::new("/foo"),
                value: json!(1),
            },
        ];

        let patch = Patch::new_with(ops.clone());

        assert_eq!(patch.0, ops);
    }

    #[test]
    fn from_vec_preserves_operations() {
        let ops = vec![
            PatchOperation::Remove {
                path: Pointer::new("/foo"),
            },
            PatchOperation::Copy {
                from: Pointer::new("/foo"),
                path: Pointer::new("/bar"),
            },
        ];

        let patch = Patch::from(ops.clone());

        assert_eq!(patch.0, ops);
    }

    #[test]
    fn try_from_string_parses_patch_json() {
        let patch = Patch::try_from(String::from(
            r#"[{"op":"replace","path":"/foo","value":2}]"#,
        ))
        .unwrap();

        assert_eq!(
            patch.0,
            vec![PatchOperation::Replace {
                path: Pointer::new("/foo"),
                value: json!(2),
            }]
        );
    }

    #[test]
    fn try_from_str_parses_patch_json() {
        let patch = Patch::try_from(r#"[{"op":"move","from":"/foo","path":"/bar"}]"#).unwrap();

        assert_eq!(
            patch.0,
            vec![PatchOperation::Move {
                from: Pointer::new("/foo"),
                path: Pointer::new("/bar"),
            }]
        );
    }

    #[test]
    fn default_creates_patch_without_operations() {
        let patch = Patch::default();

        assert_eq!(patch.0, Vec::<PatchOperation>::new());
    }

    #[test]
    fn apply_adds_to_object_and_returns_new_object() {
        let patch = Patch::new_with(vec![PatchOperation::Add {
            path: Pointer::new("/baz"),
            value: json!("qux"),
        }]);
        let target = json!({"foo": "bar"});

        let result = patch.apply(&target).unwrap();

        assert_eq!(result, json!({"foo": "bar", "baz": "qux"}));
    }

    #[test]
    fn apply_returns_error_without_modifying_original_object() {
        let patch = Patch::new_with(vec![
            PatchOperation::Add {
                path: Pointer::new("/baz"),
                value: json!("qux"),
            },
            PatchOperation::Add {
                path: Pointer::new("/missing/path"),
                value: json!(1),
            },
        ]);
        let target = json!({"foo": "bar"});

        let result = patch.apply(&target);

        assert!(result.is_err());
        assert_eq!(target, json!({"foo": "bar"}));
    }
}
