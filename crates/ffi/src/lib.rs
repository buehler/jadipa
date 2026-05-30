use boltffi::*;

/// Errors returned by Jadipa FFI functions.
///
/// The variants preserve which stage failed while keeping the payload FFI-safe
/// for generated bindings. The `message` field contains the underlying parser,
/// patch, application, or serialization error text.
#[error]
pub enum JadipaError {
    /// The input string could not be parsed as JSON.
    InvalidJson { message: String },
    /// The patch string could not be parsed as a valid JSON Patch or merge
    /// patch document.
    InvalidPatch { message: String },
    /// A JSON Patch operation could not be applied to the target document.
    PatchApplication { message: String },
    /// The patched JSON value could not be serialized back to a string.
    Serialization { message: String },
}

pub struct Patch;

#[export]
impl Patch {
    /// Applies a JSON Patch document to a target JSON document.
    ///
    /// `target_json` must contain a valid JSON value and `patch_json` must
    /// contain a valid JSON Patch operation array. The input target is not
    /// mutated; the returned string contains the patched JSON document.
    pub fn apply_json(target_json: &str, patch_json: &str) -> Result<String, JadipaError> {
        let target: serde_json::Value =
            serde_json::from_str(target_json).map_err(|err| JadipaError::InvalidJson {
                message: err.to_string(),
            })?;

        let patch = jadipa_core::patch::Patch::new(patch_json).map_err(|err| {
            JadipaError::InvalidPatch {
                message: err.to_string(),
            }
        })?;

        let patched = patch
            .apply(&target)
            .map_err(|err| JadipaError::PatchApplication {
                message: err.to_string(),
            })?;

        serde_json::to_string(&patched).map_err(|err| JadipaError::Serialization {
            message: err.to_string(),
        })
    }
}

pub struct MergePatch;

#[export]
impl MergePatch {
    /// Applies a JSON Merge Patch document to a target JSON document.
    ///
    /// Both inputs must contain valid JSON values. Object merge patches add,
    /// replace, recursively patch, or remove object members. Non-object merge
    /// patches replace the entire target document. The returned string contains
    /// the patched JSON document.
    pub fn apply_json(target_json: &str, patch_json: &str) -> Result<String, JadipaError> {
        let mut target: serde_json::Value =
            serde_json::from_str(target_json).map_err(|err| JadipaError::InvalidJson {
                message: err.to_string(),
            })?;

        let patch: serde_json::Value =
            serde_json::from_str(patch_json).map_err(|err| JadipaError::InvalidJson {
                message: err.to_string(),
            })?;

        jadipa_core::merge_patch::apply_mut(&mut target, &patch);

        serde_json::to_string(&target).map_err(|err| JadipaError::Serialization {
            message: err.to_string(),
        })
    }
}

pub struct Diff;

#[export]
impl Diff {
    /// Creates a JSON Patch document that transforms source JSON into target JSON.
    ///
    /// Both inputs must contain valid JSON values. The returned string contains
    /// a JSON Patch operation array generated from the two documents. The patch
    /// uses `add`, `remove`, and `replace` operations and can be applied to the
    /// source document to produce the target document.
    pub fn diff_json(source_json: &str, target_json: &str) -> Result<String, JadipaError> {
        let source: serde_json::Value =
            serde_json::from_str(source_json).map_err(|err| JadipaError::InvalidJson {
                message: err.to_string(),
            })?;
        let target: serde_json::Value =
            serde_json::from_str(target_json).map_err(|err| JadipaError::InvalidJson {
                message: err.to_string(),
            })?;

        let diff = jadipa_core::diff::diff(&source, &target);

        serde_json::to_string(&diff).map_err(|err| JadipaError::Serialization {
            message: err.to_string(),
        })
    }
}
