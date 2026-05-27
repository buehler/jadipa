use boltffi::*;

#[error]
pub enum JadipaError {
    InvalidJson { message: String },
    InvalidPatch { message: String },
    PatchApplication { message: String },
    Serialization { message: String },
}

#[export]
pub fn apply_patch_json(target_json: &str, patch_json: &str) -> Result<String, JadipaError> {
    let target: serde_json::Value =
        serde_json::from_str(target_json).map_err(|err| JadipaError::InvalidJson {
            message: err.to_string(),
        })?;

    let patch =
        jadipa_core::patch::Patch::new(patch_json).map_err(|err| JadipaError::InvalidPatch {
            message: err.to_string(),
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
