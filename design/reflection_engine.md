# Reflection Engine

`cheadergen` is a ground-up reimplementation of [cbindgen](https://github.com/mozilla/cbindgen),
using **rustdoc-json** as its reflection mechanism instead of syn-based source parsing.

## `cbindgen` and `syn`'s Limitations

The original tool, `cbindegen`, parses Rust source with `syn`, which means it has no module resolution, no
type-checking, cannot handle macros without nightly `cargo expand`, and maps types purely by name.
cheadergen replaces that with `rustdoc-json`, the compiler's own understanding of the crate's API.

## `rustdoc-json` Limitations

### Conditional Compilation

`rustdoc-json` only emits items for the _current_ compilation target. Items behind `#[cfg]` predicates
that are false for the active target simply don't appear in the output.

`cbindgen`'s killer feature for Firefox is translating `#[cfg]` into `#ifdef` directives for
platform-agnostic headers. That capability is **out of scope** for `cheadergen`'s initial phase.
Tests that exercise cfg-to-ifdef translation are expected failures. Don't try to solve this:
a future hybrid architecture (`rustdoc-json` for types + `syn` for cfg attributes) is the planned
resolution.

## Reference Implementations

- [Pavex](https://github.com/LukeMathWalker/pavex). One of the most mature production users of rustdoc-json
  as a reflection engine. Study its caching, lazy deserialization, and cross-crate resolution.
