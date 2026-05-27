//! JSON Patch support.
//!
//! This module provides [`Patch`] and [`PatchOperation`] for parsing and
//! applying JSON Patch documents as defined by [RFC 6902](https://www.rfc-editor.org/rfc/rfc6902).
//! JSON Patch describes changes to a JSON document as an ordered list of
//! operations such as `add`, `remove`, `replace`, `move`, `copy`, and `test`.
//!
//! Patch operations use JSON Pointer paths to address values in the target
//! document. Operations are applied sequentially, and application stops at the
//! first failing operation.
mod operations;
mod patch;

pub use operations::PatchOperation;
pub use patch::{Patch, PatchError};
