# Jadipa

Jadipa is a .NET binding for the Jadipa JSON DiffPatch core library.

The package applies JSON Patch documents as defined by [RFC 6902](https://www.rfc-editor.org/rfc/rfc6902) to JSON values addressed with JSON Pointer paths as defined by [RFC 6901](https://www.rfc-editor.org/rfc/rfc6901). It also applies JSON Merge Patch documents as defined by [RFC 7396](https://www.rfc-editor.org/rfc/rfc7396).

## Installation

```sh
dotnet add package Jadipa
```

The package targets `net10.0` and ships native runtime assets for:

- `osx-arm64`
- `osx-x64`
- `linux-x64`
- `linux-arm64`
- `win-x64`

## JSON Patch Usage

```csharp
using Jadipa;

var targetJson = """
{
  "title": "Draft release notes",
  "status": "draft",
  "tags": ["release", "internal"],
  "metadata": {
    "owner": "core",
    "reviewed": false
  },
  "temporary": true
}
""";

var patchJson = """
[
  { "op": "test", "path": "/status", "value": "draft" },
  { "op": "replace", "path": "/status", "value": "published" },
  { "op": "add", "path": "/tags/-", "value": "json-patch" },
  { "op": "remove", "path": "/temporary" },
  { "op": "copy", "from": "/metadata/owner", "path": "/owner" },
  { "op": "move", "from": "/title", "path": "/headline" }
]
""";

try
{
    var patchedJson = Patch.ApplyJson(targetJson, patchJson);
    Console.WriteLine(patchedJson);
}
catch (JadipaErrorException ex)
{
    Console.Error.WriteLine(ex.Message);
}
```

`Patch.ApplyJson` returns a new compact JSON string. The input JSON string is not modified.

## JSON Patch

Patch documents must be JSON arrays. Operations are applied in array order, and application stops at the first failing operation.

Supported operations:

- `add`: inserts or replaces an object member, inserts into an array at an index, or appends to an array with `-`.
- `remove`: removes the value at `path`.
- `replace`: replaces the existing value at `path`.
- `move`: removes the value at `from` and adds it at `path`.
- `copy`: copies the value at `from` to `path`.
- `test`: checks that the value at `path` equals the supplied `value`.

## JSON Merge Patch Usage

```csharp
using Jadipa;

var targetJson = """
{
  "title": "Goodbye!",
  "author": {
    "givenName": "John",
    "familyName": "Doe"
  },
  "tags": ["example", "sample"],
  "content": "This will be unchanged"
}
""";

var patchJson = """
{
  "title": "Hello!",
  "phoneNumber": "+01-123-456-7890",
  "author": {
    "familyName": null
  },
  "tags": ["example"]
}
""";

try
{
    var patchedJson = MergePatch.ApplyJson(targetJson, patchJson);
    Console.WriteLine(patchedJson);
}
catch (JadipaErrorException ex)
{
    Console.Error.WriteLine(ex.Message);
}
```

`MergePatch.ApplyJson` returns a new compact JSON string. Object merge patches add, replace, recursively patch, or remove object members. A `null` value in an object patch removes that member. Non-object merge patches replace the entire target value. Arrays are replaced as complete values.

## JSON Pointer Paths

Patch paths use JSON Pointer syntax:

- The empty pointer `""` addresses the whole document.
- Non-empty paths use `/`-separated reference tokens, such as `/metadata/owner`.
- `~1` represents `/`.
- `~0` represents `~`.
- Array indexes are decimal strings, such as `/tags/0`.
- The `add` operation can append to arrays with `-`, such as `/tags/-`.

Example escaped paths:

```json
[
  { "op": "replace", "path": "/a~1b", "value": 1 },
  { "op": "replace", "path": "/m~0n", "value": 2 }
]
```

`/a~1b` addresses the member named `a/b`. `/m~0n` addresses the member named `m~n`.

## Errors

The binding throws `JadipaErrorException` when the Rust core returns an error. The original error value is available through `ex.Error`.

Error variants:

- `JadipaError.InvalidJson`: the target JSON could not be parsed.
- `JadipaError.InvalidPatch`: the patch document could not be parsed as JSON Patch or JSON Merge Patch.
- `JadipaError.PatchApplication`: a patch operation failed, for example because a path was missing, an array index was invalid, or a `test` operation failed.
- `JadipaError.Serialization`: the patched value could not be serialized back to JSON.

```csharp
try
{
    var patchedJson = Patch.ApplyJson("""{"name":"old"}""", """
    [
      { "op": "replace", "path": "/missing", "value": "new" }
    ]
    """);
}
catch (JadipaErrorException ex) when (ex.Error is JadipaError.PatchApplication)
{
    Console.Error.WriteLine(ex.Message);
}
```
