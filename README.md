# `cheadergen`

`cheadergen` is a CLI to generate C header files for Rust libraries that expose a
C-compatible API.

It is an alternative to
[`cbindgen`](https://github.com/mozilla/cbindgen) that uses
[`rustdoc-json`](https://rust-lang.github.io/rfcs/2963-rustdoc-json.html) as its
reflection mechanism instead of source parsing. Check out our [comparison page](https://cheadergen.com/explanation/vs-cbindgen.html)
for more details.

## Documentation

- [User guide](https://cheadergen.com).
- References:
  - [`#[cheadergen::config(...)]`: per-item configuration](https://docs.rs/cheadergen/latest/cheadergen/attr.config.html) macro reference.
  - [`cheadergen.toml`: project-level configuration](https://docs.rs/cheadergen/latest/cheadergen/config_reference/index.html).
