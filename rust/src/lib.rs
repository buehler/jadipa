//! MOD DOC
#![cfg_attr(
    feature = "document-features",
    cfg_attr(doc, doc = ::document_features::document_features!())
)]

#[cfg(feature = "diff")]
pub mod diff;

#[cfg(feature = "merge_patch")]
pub mod merge_patch;

#[cfg(feature = "patch")]
pub mod patch;

pub mod pointer;
