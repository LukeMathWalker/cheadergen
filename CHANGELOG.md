# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### ‼️ Breaking changes

- `opaque` is now a standalone directive on `#[cheadergen::config(...)]` rather
  than an argument to `export`. Write `export, opaque` to force-include a type
  as an opaque forward declaration (previously `export(opaque)`), or `opaque`
  on its own to mark a type as opaque only when it is otherwise included in
  the header.

## [0.1.4](https://github.com/LukeMathWalker/cheadergen/compare/0.1.3...0.1.4) - 2026-05-11

### ⛰️ Features

- Support Rust types with custom alignment (by @LukeMathWalker)

### Contributors

- @LukeMathWalker

## [0.1.3](https://github.com/LukeMathWalker/cheadergen/compare/0.1.2...0.1.3) - 2026-05-10

### 🐛 Bug Fixes

- Ensure deterministic headers in case of name conflicts (by @LukeMathWalker)

### Contributors

- @LukeMathWalker

## [0.1.2](https://github.com/LukeMathWalker/cheadergen/compare/0.1.1...0.1.2) - 2026-05-05

### ‼️ Breaking changes

- Constants must opt-in to be included in the generated header, via an annotation (by @LukeMathWalker)

### Contributors

- @LukeMathWalker

## [0.1.1](https://github.com/LukeMathWalker/cheadergen/compare/0.1.0...0.1.1) - 2026-05-04

### ⛰️ Features

- Add support for usize_is_size_t configuration as a global knob as well as a per-package knob (by @LukeMathWalker)
- Support rename_all and rename_all_fields directives (by @LukeMathWalker)
- Add new macros attributes, without compiler support (by @LukeMathWalker)
- Add support for MaybeUninit (by @LukeMathWalker)
- Add support for UnsafeCell (by @LukeMathWalker)
- Add support for arrays with const generic length (by @LukeMathWalker)
- Simplify ManuallyDrop<T> to T (by @LukeMathWalker)
- Implement a first version of null-pointer optimization (by @LukeMathWalker)
- Add parameter names to function pointer declarations when we have them on the Rust side (by @LukeMathWalker)
- Add type-level simplifications to handle null pointer optimization (by @LukeMathWalker)
- Add support for simple function pointers (by @LukeMathWalker)
- Make enum variant prefix configurable (by @LukeMathWalker)
- Add support for inherent associated constants (by @LukeMathWalker)
- Handle type aliases (by @LukeMathWalker)
- Add support for str and char constants. Broaden our test suite for computed constants and other numeric types (by @LukeMathWalker)
- Add support for repr transparent (by @LukeMathWalker)
- Include private fields in emitted definitions (by @LukeMathWalker)
- Add support for unions (by @LukeMathWalker)
- Add an underscore to Rust variant names and fields that are also keywords in C or C++ (by @LukeMathWalker)
- Handle 'Self' in structs and enums. Sort type definitions in source order. (by @LukeMathWalker)
- Add support for standalone constants (by @LukeMathWalker)
- Emit monomorphised versions of generic types when used with concrete parameters in function signatures (by @LukeMathWalker)
- Add support for function arguments as expected by the 'both' style (by @LukeMathWalker)
- Emit C++-compatible headers for enums and tagged unions (by @LukeMathWalker)
- Make ZST more precise by matching on full path and package id. Test all known variants. (by @LukeMathWalker)
- Rough support for fieldless enums and tagged unions (by @LukeMathWalker)
- Add support for documentation config options (by @LukeMathWalker)
- Add support for generating struct definition out of repr(C) Rust structs (by @LukeMathWalker)
- Make sorting configurable, either by name or by source order. (by @LukeMathWalker)
- Support for statics (by @LukeMathWalker)
- Take generics into account when generating the C name of monomorphised generic Rust types (by @LukeMathWalker)
- Emit forward declarations for any user-defined type. Stepping stone (by @LukeMathWalker)
- Add a translation command to convert a cbindgen config file into a cheadergen config file (by @LukeMathWalker)
- Honor basic codegen options (by @LukeMathWalker)
- Add config parsing system (by @LukeMathWalker)
- Generate plain C header files for simple functions (by @LukeMathWalker)
- Add --symbol-file to export discovered symbol names. Add new tests to check exported symbols match expectations. (by @LukeMathWalker)
- Collect exported symbols, be they functions or statics (by @LukeMathWalker)
- Identify the target crate and compute its JSON documentation (by @LukeMathWalker)

### 🐛 Bug Fixes

- Emit a blank line between braces for struct definitions with no fields (by @LukeMathWalker)
- Use typedef union for opaque unions (by @LukeMathWalker)
- Use GlobalItemId to ensure correct cross-crate resolution of items and docs (by @LukeMathWalker)
- Emit type definitions in topological order to ensure all types have their by-value dependencies defined beforehand (by @LukeMathWalker)
- Use the correct style for pointer fields in struct definitions (by @LukeMathWalker)
- Add trailing new line to header docs when cbindgen does (by @LukeMathWalker)
- Coverage report now works as expected (by @LukeMathWalker)
- Honor export attribute when emitting the symbol file (by @LukeMathWalker)

### 📚 Documentation

- Document cheadergen.toml sections (by @LukeMathWalker)
- Document the desired macro surface for annotations (by @LukeMathWalker)

### 🧪 Testing

- Mark lifetime_arg as passing (by @LukeMathWalker)
- Exercise generic monomorphisation with and without default type parameters (by @LukeMathWalker)
- Enhance ui-tests translate-config to emit a granular report on which tests use unsupported cbindgen configuration options (by @LukeMathWalker)
- Mark more cbindgen tests as passing (by @LukeMathWalker)
- Speed up config translation by compiling once rather than going through cargo run for each discovered cbindgen.toml file (by @LukeMathWalker)
- After config translation, report only which files have changed (by @LukeMathWalker)
- Add a new mechanism to automatically translate a cbindgen config file into a cheadergen one. Keep track of ignored fields. (by @LukeMathWalker)
- Warm up rustdoc cache ahead of test execution, thus minimizing contention (by @LukeMathWalker)
- Bring back the cheadergen custom test suite with a failing test (by @LukeMathWalker)
- Use cheadergen in tests, rather than cbindgen. Track which tests are expected to fail/succeeds. (by @LukeMathWalker)

### Contributors

- @LukeMathWalker
