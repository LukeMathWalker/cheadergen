# Introduction

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
is a good primer on the building blocks `cheadergen` relies on: the C
calling convention, `#[no_mangle]` for stable symbol names, `#[repr(C)]`
for predictable struct layout, and the rules around passing data across
the boundary.

Read through it first if any of those terms feel unfamiliar: the
rest of this guide assumes you know what they do, and focuses on how
`cheadergen` can support your mixed C/Rust projects.

## Author

`cheadergen` (**C Header Gen**erator) is built and maintained by [Luca Palmieri](https://lpalmieri.com).

## User guide structure

This guide is structured as follows:

- **[Getting started](./getting-started/install.md)**.
  If you're new, start here. Install the CLI, then walk through the
  [Quickstart](./getting-started/quickstart.md).
- **Foundations**.
  The load-bearing concepts you'll meet using cheadergen:
  [target packages](./foundations/target-packages.md),
  [item annotations](./foundations/item-annotations.md), and
  [global configuration](./foundations/global-configuration.md).
- **How it works**.
  The internals: [the processing pipeline](./how-it-works/pipeline.md) and
  [generics and monomorphization](./how-it-works/generics-and-monomorphization.md).
- **What you get**.
  The shape of the generated output:
  [anatomy of a generated header](./what-you-get/header-structure.md) and
  [bundled vs partitioned output](./what-you-get/partitioning.md).
- **Limits and alternatives**.
  [Comparison with `cbindgen`](./limits-and-alternatives/vs-cbindgen.md),
  [what `cheadergen` can't do](./limits-and-alternatives/limitations.md), and
  [C++ support](./limits-and-alternatives/cpp-support.md).
- **How-to guides**.
  Recipes for common tasks:
  [wiring `cheadergen` into Cargo](./how-to/integrate-with-cargo.md),
  [generating headers across a workspace](./how-to/workspaces-and-multi-crate.md),
  [migrating from `cbindgen`](./how-to/migrate-from-cbindgen.md),
  and more.
- [**API references on docs.rs**](https://docs.rs/cheadergen).
  The canonical references for the options exposed by the
  [`#[cheadergen::config(...)]`](https://docs.rs/cheadergen/latest/cheadergen/attr.config.html)
  attribute and the
  [`cheadergen.toml`](https://docs.rs/cheadergen/latest/cheadergen/config_reference/index.html)
  config file.
