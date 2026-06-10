# Comparison with `cbindgen`

[`cbindgen`](https://github.com/mozilla/cbindgen) is the established tool for
generating C/C++ headers from Rust. `cheadergen` is a younger alternative,
built on different foundations.

This page explains where they differ and how to decide
which one fits your project.

## Why a a new tool?

`cbindgen` parses your Rust source files to infer the shape of the C API for
your project. It's an approach with quite a few downsides:

- **Limited module resolution and type-level analysis.**
- **No macro expansion** without resorting to `cargo expand` or ad-hoc code paths.
- **Visibility into other crates is shallow**.

In other words, `cbindgen` can't tap into the rich view that's available to the
Rust compiler for a given crate. This limitation can create significant friction in
real world projects; enough friction to build an alternative, `cheadergen`!

`cheadergen` relies on the Rust compiler for compile-time reflection, via
[`rustdoc-json`](https://rust-lang.github.io/rfcs/2963-rustdoc-json.html).
That gives it module resolution, type checking, accurate cross-crate
references, and macro expansion for free.

But it'd be dishonest to state that `cheadergen` is a clear-cut improvement
for _all_ projects: there are some downsides and limitations you should be aware of
before choosing one or the other for your next project.

## Behavioural differences

### Annotation syntax

`cbindgen` uses **doc-comment annotations**:

```rust
/// cbindgen:rename-all=ScreamingSnakeCase
#[repr(C)]
pub enum Status { Ok, Error }
```

`cheadergen` uses **proc-macro attributes** that participate in normal Rust
compilation:

```rust
#[cheadergen::config(rename_all = "SCREAMING_SNAKE_CASE")]
#[repr(C)]
pub enum Status { Ok, Error }
```

## `cheadergen` downsides

### `#[cfg]` attributes

`cbindgen` translates `#[cfg(...)]` into `#ifdef ...` directives so a single
header works across platforms. **`cheadergen` can't do this** (see
[Limitations](./limitations.md)).

### Constants in array lengths

`cbindgen` preserves named constants in array-length positions
(`int32_t x[FOO]`). `cheadergen` receives the array length as a resolved
literal value from rustdoc JSON, so it emits `int32_t x[10]` and loses the
constant's name in that position only.

### Doc generation

`cheadergen` must generate JSON documentation for all packages that appear
in the C API exposed by your Rust project. This implies a `cargo doc` invocation
that, in most cases, is going to be slower than parsing source code via `syn`
(the approach taken by `cbindgen`).

The slowdown is mitigated via a disk-based cache for third-party crates, but
it's worth calling out as a downside compared to `cbindgen`.

Furthermore, this requires you have to a `nightly` toolchain installed,
at least on dev machines. This issue may go away in the future,
once `rustdoc-json` stabilizes.

## When you'd pick `cbindgen`

- You need `#[cfg]`-to-`#ifdef` translation (Firefox-style multi-platform
  headers).
- You need C++-idiomatic output (`enum class`, custom constructors,
  `derive-ostream`, etc.).
- You need Cython output
- You can't use a `nightly` toolchain for build-time tools.

## When you'd pick `cheadergen`

- You're on a Cargo workspace with multiple FFI-facing crates and want
  partitioned headers that share types coherently.
- You value type-checked, IDE-friendly annotations over doc-comment strings.
- Your platform set is narrow enough that the missing `#[cfg]` handling
  isn't a blocker.

If you're using `cbindgen` on a project, and you want to migrate over,
check out our [migration guide](../how-to/migrate-from-cbindgen.md).
