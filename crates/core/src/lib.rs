//! Jadipa (JSON DiffPatch)
//!
//! This crate provides tooling for JSON pointers, patches, and other utils.
//! It is built around `serde_json::Value` and exposes small modules
//! for standards-based JSON document addressing and mutation.
//!
//! The [`pointer`] module is always available and implements JSON Pointer
//! support as defined by [RFC 6901](https://www.rfc-editor.org/rfc/rfc6901).
//! Use [`pointer::Pointer`] to represent paths into a JSON document and to
//! resolve immutable or mutable values from a `serde_json::Value`.
//!
//! With the `patch` feature enabled, the [`patch`] module provides JSON Patch
//! support as defined by [RFC 6902](https://www.rfc-editor.org/rfc/rfc6902).
//! [`patch::Patch`] parses ordered patch documents and applies their
//! operations sequentially to JSON values.
//!
//! # Example
//!
//! ```
//! use jadipa::patch::Patch;
//! use serde_json::json;
//!
//! let target = json!({ "name": "old" });
//! let patch = Patch::new(r#"[{"op":"replace","path":"/name","value":"new"}]"#)?;
//! let patched = patch.apply(&target)?;
//!
//! assert_eq!(patched, json!({ "name": "new" }));
//! assert_eq!(target, json!({ "name": "old" }));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
#![cfg_attr(
    feature = "document-features",
    cfg_attr(doc, doc = ::document_features::document_features!())
)]

// #[cfg(feature = "diff")]
// pub mod diff;

// #[cfg(feature = "merge_patch")]
// pub mod merge_patch;

#[cfg(feature = "patch")]
pub mod patch;

pub mod pointer;
