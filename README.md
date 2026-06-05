# `cheadergen`

`cheadergen` is a CLI to generate C header files for Rust libraries that expose a
C-compatible API.

It is an alternative to
[`cbindgen`](https://github.com/mozilla/cbindgen) that uses
[`rustdoc-json`](https://rust-lang.github.io/rfcs/2963-rustdoc-json.html) as its
reflection mechanism instead of source parsing. Check out our [detailed comparison](docs/explanation/vs-cbindgen.md)
to see how they compare.

## Documentation

- [User guide](./docs/src/SUMMARY.md). Browse it locally via `just book-serve`.
- [`#[cheadergen::config(...)]`: per-item configuration reference](https://docs.rs/cheadergen/latest/cheadergen/attr.config.html) macro reference.
- [`cheadergen.toml`: global configuration reference](https://docs.rs/cheadergen/latest/cheadergen/config_reference/index.html).
