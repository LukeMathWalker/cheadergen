# The processing pipeline

When you run `cheadergen generate`, your crate flows through six stages before
a single byte of C lands on disk. Understanding the order helps you reason
about _why_ a given item did (or did not!) end up in the generated C header(s).

## 1. Invoke `rustdoc-json`

`cheadergen` invokes `cargo +<nightly> doc [...] --output-format=json`
for the target package. The Rust compiler does the heavy
lifting: macro expansion, module resolution, type checking, name resolution.

## 2. Filter FFI items

`cheadergen` scans the JSON file emitted by `rustdoc` to determine which items
are exposed to C:

- Functions annotated with `extern "C"` and `#[no_mangle]`.
- Statics annotated with `#[no_mangle]`.
- Anything that explicitly opted into C visibility via
  [`#[cheadergen::config(export)]`](./annotations-overview.md).

> **Caution!** `cheadergen` will only pick up items annotated via
> `#[cheadergen::config(export)]` if they are defined in a _target_ package.
> I.e. if the defining package was passed to `cheadergen generate` via the
> `--package` flag (or implicitly via `--workspace`, if it's a local crate)

## 3. Compute the crate closure

`cheadergen` then tries to determine the _crate closure_: the set of Rust types
that are reachable from any of the items collected by the previous step (e.g. a struct used
as input parameter in one of your `extern "C"` functions).

Keep in mind that a function signature in your crate can reference a type defined in a
_different_ crate (a workspace member, a dependency from crates.io, etc.).
`cheadergen` walks each external reference and pulls in the defining crate's
rustdoc JSON on demand (recursively) until every reachable type has a
definition.

This is also why dependency types can end up in your generated header at all,
and why the [partitioned output mode](./partitioning.md) gives each defining
crate its own header file.

The JSON files for third-party dependencies are cached on disk in
`cheadergen`'s cache directory.
Subsequent runs try to reuse them whenever possible.

> **Tip.** You can pre-warm the cache for an entire workspace with
> `cheadergen cache warm`, inspect the cache directory with
> `cheadergen cache show dir`, and clear it with `cheadergen cache clear`.
> See [Manage the rustdoc cache](../how-to/manage-the-rustdoc-cache.md) for
> the dedicated guide.

## 4. Build the IR

The raw `rustdoc-types` items are translated into `cheadergen`'s own intermediate
representation (IR). The IR is closer to C than to Rust: pointers, primitives,
tagged unions and bitfields are all explicit, and naming concerns
(`rename`, `prefix_with_name`, casing rules) are resolved here.

## 5. Transform standard types

A few well-known Rust types are special-cased so the resulting C is idiomatic:

- `Option<&T>` and `Option<&mut T>` lower to `*const T` / `*mut T`.
- `NonNull<T>` lowers to `*mut T` (or `*const T` with `#[cheadergen(const_ptr)]`).
- `usize` / `isize` become `uintptr_t` / `intptr_t` (or `size_t` / `ptrdiff_t`
  if `usize_is_size_t` is set).
- etc.

## 6. Emit

The final IR is rendered into one or more C headers. The exact shape (the
order of sections, what gets `#include`d, whether you get one file or multiple) is
controlled by your `cheadergen.toml` and CLI flags. See
[Anatomy of a generated header](./header-structure.md) and
[Bundled vs partitioned output](./partitioning.md) for more details.
