# `cheadergen`

`cheadergen` generates accurate C headers for Rust libraries that expose a C-compatible API.

`cheadergen` provides:

- **Multi-crate support.** One C header per crate, with
  cross-crate `#include`s wired up automatically.
- **Compiler-accurate type analysis.** Type information comes from
  [`rustdoc-json`](https://rust-lang.github.io/rfcs/2963-rustdoc-json.html),
  so the generated output mirrors what the Rust compiler actually sees.
- **Macro-aware.** Items defined by declarative or procedural macros
  are picked up automatically.

`cheadergen` is an alternative to
[`cbindgen`](https://github.com/mozilla/cbindgen). Check out our
[comparison page](https://cheadergen.com/limits-and-alternatives/vs-cbindgen.html)
for more details.

`cheadergen` (**C Header Gen**erator) is built and maintained by [Luca Palmieri](https://lpalmieri.com).

## Documentation

- [User guide](https://cheadergen.com)
- References:
  - [`#[cheadergen::config(...)]`: per-item configuration](https://docs.rs/cheadergen/latest/cheadergen/attr.config.html).
  - [`cheadergen.toml`: project-level configuration](https://docs.rs/cheadergen/latest/cheadergen/config_reference/index.html).
