//! JSON Pointer support.
//!
//! This module provides [`Pointer`], a small wrapper around JSON Pointer strings
//! as defined by [RFC 6901](https://www.rfc-editor.org/rfc/rfc6901). JSON
//! Pointers identify values inside a JSON document using `/`-separated
//! reference tokens.
//!
//! Token escaping follows [RFC 6901, section 3](https://www.rfc-editor.org/rfc/rfc6901#section-3):
//! `~1` represents `/`, and `~0` represents `~`. Value lookup delegates to
//! [`serde_json::Value::pointer`] and [`serde_json::Value::pointer_mut`].

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
/// A parsed JSON Pointer as defined by [RFC 6901](https://www.rfc-editor.org/rfc/rfc6901).
///
/// The pointer stores decoded reference tokens. For example, `/foo/0` is stored
/// as `["foo", "0"]`, and escape sequences are decoded according to the RFC.
pub struct Pointer(String);

impl Pointer {
    /// Creates a new JSON Pointer from a pointer string.
    ///
    /// The empty string represents the whole document and produces no reference
    /// tokens. For non-empty pointers, the leading slash separates tokens. RFC
    /// escape sequences are decoded in order: `~1` becomes `/`, then `~0`
    /// becomes `~`.
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    /// Returns the decoded reference tokens for this JSON Pointer.
    ///
    /// The empty pointer returns an empty vector. For non-empty pointers, the
    /// leading slash is ignored, the remaining string is split on `/`, and RFC
    /// escape sequences are decoded in order: `~1` becomes `/`, then `~0`
    /// becomes `~`.
    pub fn tokens(&self) -> Vec<String> {
        let path = self.0.as_str();
        if path.is_empty() {
            return Vec::new();
        }

        path.strip_prefix('/')
            .unwrap_or(path)
            .split('/')
            .map(|p| p.replace("~1", "/").replace("~0", "~"))
            .collect()
    }

    /// Returns the value addressed by this pointer in `object`.
    ///
    /// This delegates to [`serde_json::Value::pointer`]. It returns `None` when
    /// the path does not exist or when the stored pointer string is not valid
    /// JSON Pointer syntax.
    pub fn get<'a>(&self, object: &'a Value) -> Option<&'a Value> {
        object.pointer(&self.0)
    }

    /// Returns a mutable value addressed by this pointer in `object`.
    ///
    /// This delegates to [`serde_json::Value::pointer_mut`]. It returns `None`
    /// when the path does not exist or when the stored pointer string is not
    /// valid JSON Pointer syntax.
    pub fn get_mut<'a>(&self, object: &'a mut Value) -> Option<&'a mut Value> {
        object.pointer_mut(&self.0)
    }

    pub(crate) fn path_without_leading_slash(&self) -> String {
        self.0.strip_prefix('/').unwrap_or(&self.0).to_string()
    }

    pub(crate) fn raw_tokens(&self) -> Vec<String> {
        let path = self.0.as_str();
        if path.is_empty() {
            return Vec::new();
        }

        path.strip_prefix('/')
            .unwrap_or(path)
            .split('/')
            .map(|s| s.to_string())
            .collect()
    }
}

impl std::fmt::Display for Pointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Pointer {
    fn from(path: &str) -> Self {
        Self::new(path)
    }
}

impl From<String> for Pointer {
    fn from(path: String) -> Self {
        Self::new(path)
    }
}

impl AsRef<str> for Pointer {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use yare::parameterized;

    #[parameterized(
        empty_pointer = { "" },
        root_member = { "/foo" },
    )]
    fn new_pointer(path: &str) {
        let pointer = Pointer::new(path);

        assert_eq!(pointer.0, path);
    }

    #[parameterized(
        empty_pointer = { "", vec![] },
        root_member = { "/foo", vec!["foo"] },
        nested_member = { "/foo/0", vec!["foo", "0"] },
        slash_only = { "/", vec![""] },
        slash_replacement = { "/a~1b", vec!["a/b"] },
        tilde_replacement = { "/m~0n", vec!["m~n"] },
        replacement_order = { "/~01", vec!["~1"] },
        percent_character = { "/c%d", vec!["c%d"] },
        caret_character = { "/e^f", vec!["e^f"] },
        pipe_character = { "/g|h", vec!["g|h"] },
        backslash_character = { "/i\\j", vec!["i\\j"] },
        quote_character = { "/k\"l", vec!["k\"l"] },
        space_character = { "/ ", vec![" "] },
    )]
    fn correct_pathes_are_extracted(path: &str, expected: Vec<&str>) {
        let pointer = Pointer::new(path);

        assert_eq!(pointer.tokens(), expected);
    }

    #[test]
    fn get_returns_pointed_value() {
        let object = json!({
            "foo": ["bar", "baz"],
            "a/b": 1,
        });
        let pointer = Pointer::new("/foo/1");

        assert_eq!(pointer.get(&object), Some(&json!("baz")));
        assert_eq!(Pointer::new("/a~1b").get(&object), Some(&json!(1)));
        assert_eq!(Pointer::new("/missing").get(&object), None);
    }

    #[test]
    fn get_mut_returns_pointed_value() {
        let mut object = json!({
            "foo": ["bar", "baz"],
        });
        let pointer = Pointer::new("/foo/1");

        *pointer.get_mut(&mut object).unwrap() = json!("updated");

        assert_eq!(object, json!({ "foo": ["bar", "updated"] }));
        assert_eq!(Pointer::new("/missing").get_mut(&mut object), None);
    }
}
