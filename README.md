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

## What it does

You write Rust:

```rust
#[repr(C)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[unsafe(no_mangle)]
pub extern "C" fn distance(a: Point, b: Point) -> f64 {
    ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt()
}
```

`cheadergen` produces a header your C code can consume:

```c
typedef struct {
    double x;
    double y;
} Point;

double distance(Point a, Point b);
```

## New to C/Rust interoperability?

If you've never written `extern "C"` in Rust before, the
[FFI chapter of the Rust Nomicon](https://doc.rust-lang.org/nomicon/ffi.html)
is a good primer on the topic.

## Documentation

- [User guide](https://cheadergen.com)
- References:
  - [`#[cheadergen::config(...)]`: per-item configuration](https://docs.rs/cheadergen/latest/cheadergen/attr.config.html).
  - [`cheadergen.toml`: project-level configuration](https://docs.rs/cheadergen/latest/cheadergen/config_reference/index.html).
