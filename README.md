# Jadipa

Jadipa is a JSON DiffPatch project centered on standards-based JSON document mutation.

The Rust core currently provides JSON Pointer support ([RFC 6901](https://www.rfc-editor.org/rfc/rfc6901)), JSON Patch support ([RFC 6902](https://www.rfc-editor.org/rfc/rfc6902)), JSON Patch diff generation, and JSON Merge Patch support ([RFC 7396](https://www.rfc-editor.org/rfc/rfc7396)). The .NET binding exposes the patch, diff, and merge patch APIs through a NuGet package backed by native Rust binaries.

## Repository Layout

- `crates/core`: Rust library for JSON pointers, JSON Patch, JSON Patch diff generation, and JSON Merge Patch.
- `crates/ffi`: FFI layer used to expose core functionality to bindings.
- `bindings/dotnet/Jadipa`: .NET package project.
- `bindings/dotnet/Jadipa.Tests`: .NET binding tests.

## Requirements

- Rust toolchain with Cargo
- .NET SDK 10.x
- `boltffi_cli` when regenerating bindings

```sh
cargo install boltffi_cli
```

## Rust

Run the Rust test suite:

```sh
cargo test --all-features
```

Build the native FFI library for the current platform:

```sh
cargo build -p jadipa-ffi --release
```

## .NET Bindings

Regenerate the C# binding code after changing the FFI surface:

```sh
cd crates/ffi
boltffi generate csharp
```

Pack the NuGet package:

```sh
cd bindings/dotnet/Jadipa
dotnet pack --configuration Release --output packages/
```

Run the .NET tests:

```sh
cd bindings/dotnet/Jadipa.Tests
dotnet test
```

The NuGet package includes native assets for `osx-arm64`, `osx-x64`, `linux-x64`, `linux-arm64`, and `win-x64`. CI builds those native binaries before packing the package.

## Current API Surface

The .NET package exposes:

```csharp
Jadipa.Patch.ApplyJson(string targetJson, string patchJson)
Jadipa.Diff.DiffJson(string sourceJson, string targetJson)
Jadipa.MergePatch.ApplyJson(string targetJson, string patchJson)
```

`Patch.ApplyJson` and `MergePatch.ApplyJson` return the patched JSON as a compact string. `Diff.DiffJson` returns a compact JSON Patch operation array that transforms `sourceJson` into `targetJson`; the generated patch uses `add`, `remove`, and `replace` operations.

The methods throw `JadipaErrorException` for invalid JSON, invalid patch documents, patch application failures where applicable, or serialization failures. `Patch.ApplyJson` expects a JSON Patch operation array. `MergePatch.ApplyJson` expects any valid JSON Merge Patch value.
