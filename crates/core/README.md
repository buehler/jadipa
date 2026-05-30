# Jadipa

Rust library for standards-based JSON document addressing and mutation.

The crate is built around `serde_json::Value` and currently exposes:

- `pointer`: JSON Pointer support as defined by [RFC 6901](https://www.rfc-editor.org/rfc/rfc6901).
- `patch`: JSON Patch support as defined by [RFC 6902](https://www.rfc-editor.org/rfc/rfc6902).
- `diff`: JSON Patch diff generation.
- `merge_patch`: JSON Merge Patch support as defined by [RFC 7396](https://www.rfc-editor.org/rfc/rfc7396).

## Install

```bash
cargo add jadipa
```

## Features

- `patch`: enables JSON Patch parsing and application.
- `diff`: enables JSON Patch diff generation; depends on `patch`.
- `merge_patch`: enables JSON Merge Patch application.

Default features: `patch`, `diff`.

## JSON Patch

```rust
use jadipa::patch::Patch;
use serde_json::json;

let target = json!({
    "name": "old",
    "tags": ["stable"]
});

let patch = Patch::new(
    r#"[
        {"op":"replace","path":"/name","value":"new"},
        {"op":"add","path":"/tags/-","value":"json-patch"}
    ]"#,
).unwrap();

let patched = patch.apply(&target).unwrap();

assert_eq!(patched, json!({
    "name": "new",
    "tags": ["stable", "json-patch"]
}));
assert_eq!(target, json!({
    "name": "old",
    "tags": ["stable"]
}));
```

Patch operations are applied in order. Application stops at the first failing operation. `apply` returns a patched clone and does not mutate the input value.

Supported operations: `add`, `remove`, `replace`, `move`, `copy`, `test`.

## JSON Patch Diff

The `diff` module creates a JSON Patch that transforms one `serde_json::Value` into another.

```rust
use jadipa::diff;
use serde_json::json;

let source = json!({
    "title": "Draft release notes",
    "status": "draft",
    "tags": ["release", "internal"],
    "temporary": true
});

let target = json!({
    "title": "Draft release notes",
    "status": "published",
    "tags": ["release", "json-patch", "internal"]
});

let patch = diff::diff(&source, &target);
let patched = patch.apply(&source).unwrap();

assert_eq!(patched, target);
assert_eq!(source["status"], "draft");
```

Generated patches use `add`, `remove`, and `replace` operations. Equal values return an empty patch. Objects are compared recursively. Arrays keep shared prefixes and suffixes, then replace, remove, or add values in the changed middle section. Generated patches are valid JSON Patch documents, but they are not guaranteed to be the shortest possible patches.

## JSON Merge Patch

Enable the `merge_patch` feature to use this module.

```rust
use jadipa::merge_patch;
use serde_json::json;

let target = json!({
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

let patched = merge_patch::apply(&target, &patch);

assert_eq!(patched, json!({
    "title": "Hello!",
    "author": {
        "givenName": "John"
    },
    "tags": ["example"]
}));
assert_eq!(target["title"], "Goodbye!");
```

Object merge patches add, replace, recursively patch, or remove object members. A `null` value in an object patch removes that member. Non-object merge patches replace the entire target value. Arrays are replaced as complete values.

Use `merge_patch::apply` to return a patched clone, or `merge_patch::apply_mut` to patch a `serde_json::Value` in place.

## JSON Pointer

```rust
use jadipa::pointer::Pointer;
use serde_json::json;

let document = json!({
    "items": ["first", "second"],
    "a/b": 1,
    "m~n": 2
});

assert_eq!(
    Pointer::new("/items/1").get(&document),
    Some(&json!("second"))
);
assert_eq!(
    Pointer::new("/a~1b").get(&document),
    Some(&json!(1))
);
assert_eq!(
    Pointer::new("/m~0n").get(&document),
    Some(&json!(2))
);
```

Pointer escaping follows RFC 6901:

- `~1` represents `/`.
- `~0` represents `~`.
- `""` addresses the whole document.

## Development

Run the core tests:

```sh
cargo test -p jadipa --all-features
```
