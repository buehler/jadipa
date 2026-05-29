# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0](https://github.com/buehler/jadipa/compare/v0.1.2...v0.2.0) (2026-05-29)


### ⚠ BREAKING CHANGES

* **ffi:** This changes the structure of the public interface of the FFI. As such, the ApplyJson functions now reside inside Patch and MergePatch static classes in csharp.

### Features

* **core:** add merge_patch to the core lib ([6e2256d](https://github.com/buehler/jadipa/commit/6e2256d50119a725c02aff0e67811bd9a8cae555))
* **ffi:** add merge patch to ffi and restructure the public interface of the ffi to map the functions correctly ([15c47e3](https://github.com/buehler/jadipa/commit/15c47e3fd587079ba09edec3a65f1eec2b780763))


### Bug Fixes

* **bindings/dotnet:** make dotnet build deterministic and upload debug symbols as well ([617d5a3](https://github.com/buehler/jadipa/commit/617d5a32ec9887b20a4eda86748d7a54ce21ca8d))

## [Unreleased]

## [0.1.2](https://github.com/buehler/jadipa/compare/v0.1.1...v0.1.2) - 2026-05-28

### Bug Fixes

- correctly deploy nuget pkg

## [0.1.1](https://github.com/buehler/jadipa/compare/v0.1.0...v0.1.1) - 2026-05-28

### Continuous Integration

- add release and build/test for github actions

### Documentation

- add readme to the core crate
