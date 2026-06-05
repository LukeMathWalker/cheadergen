# Introduction

`cheadergen` is a CLI to generate C header files for Rust libraries that expose a
C-compatible API.

It is an alternative to
[`cbindgen`](https://github.com/mozilla/cbindgen) that uses
[`rustdoc-json`](https://rust-lang.github.io/rfcs/2963-rustdoc-json.html) as its
reflection mechanism instead of source parsing. Check out our [detailed comparison](explanation/vs-cbindgen.md)
to see how they compare.

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

## How this site is organised

This guide complements the API references on
[docs.rs](https://docs.rs/cheadergen):

- The [`#[cheadergen::config(...)]`](https://docs.rs/cheadergen/latest/cheadergen/attr.config.html)
  attribute reference covers every directive you can apply to a Rust item.
- The [`cheadergen.toml`](https://docs.rs/cheadergen/latest/cheadergen/config_reference/index.html)
  config reference covers every option the TOML file accepts.

Use this book for the parts those references don't address:

- **[Getting started](./getting-started/install.md)**: install the CLI, run
  it for the first time.
- **Explanation**: understand how `cheadergen` processes your crates. Understand the
  [pipeline](./explanation/pipeline.md), the
  [shape of a generated header](./explanation/header-structure.md), the
  [bundled vs partitioned decision](./explanation/partitioning.md), and
  [what it can't do](./explanation/limitations.md).
- **How-to guides**:
  [wiring `cheadergen` into Cargo](./how-to/integrate-with-cargo.md),
  [migrating from `cbindgen`](./how-to/migrate-from-cbindgen.md), and assessing
  [whether to commit generated headers](./how-to/commit-generated-headers.md).
