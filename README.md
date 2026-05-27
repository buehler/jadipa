# Jadipa

Jadipa is a JSON DiffPatch project centered on standards-based JSON document mutation.

The Rust core currently provides JSON Pointer support ([RFC 6901](https://www.rfc-editor.org/rfc/rfc6901)) and JSON Patch support ([RFC 6902](https://www.rfc-editor.org/rfc/rfc6902)). The .NET binding exposes the patch API through a NuGet package backed by native Rust binaries.

## Repository Layout

- `crates/core`: Rust library for JSON pointers and patches.
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
Jadipa.Jadipa.ApplyPatchJson(string targetJson, string patchJson)
```

It returns the patched JSON as a compact string and throws `JadipaErrorException` for invalid target JSON, invalid patch documents, patch application failures, or serialization failures.
