# The processing pipeline

When you run `cheadergen generate`, your crate flows through six stages before
a single byte of C lands on disk. Understanding the order helps you reason
about _why_ a given item did (or did not!) end up in the generated C header(s).

## 1. Invoke `rustdoc-json`

`cheadergen` invokes `cargo +<nightly> doc [...] --output-format=json`
for each [target package](../foundations/target-packages.md). The Rust compiler does the
heavy lifting: macro expansion, module resolution, type checking, name
resolution.

## 2. Filter FFI items

`cheadergen` scans the JSON file emitted by `rustdoc` to determine which items
are exposed to C:

- Functions annotated with `extern "C"` and `#[no_mangle]`.
- Statics annotated with `#[no_mangle]`.
- Anything that explicitly opted into C visibility via
  [`#[cheadergen::config(export)]`](../foundations/item-annotations.md).

> **Caution!** `cheadergen` only picks up items annotated via
> `#[cheadergen::config(export)]` when the defining package is a
> [target package](../foundations/target-packages.md). Annotations in non-target
> packages (workspace siblings you didn't select, third-party
> dependencies, the standard library) are silently ignored.

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
and why the [partitioned output mode](../what-you-get/partitioning.md) gives each defining
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

A handful of well-known Rust types are special-cased so the resulting C is
idiomatic:

| Rust input                                                           | C output                    | Notes                                                                                                     |
| -------------------------------------------------------------------- | --------------------------- | --------------------------------------------------------------------------------------------------------- |
| `Box<T>`                                                             | `T *`                       |                                                                                                           |
| `NonNull<T>`                                                         | `T *`                       | Becomes `const T *` if the field is annotated with `#[cheadergen(const_ptr)]`.                            |
| `Option<&T>` / `Option<&mut T>`                                      | `const T *` / `T *`         |                                                                                                           |
| `Option<Box<T>>` / `Option<NonNull<T>>`                              | `T *`                       | Null-pointer optimization; same shape as the inner pointer.                                               |
| `Option<fn(...)>`                                                    | `fn(...)`                   | Function pointers are inherently nullable.                                                                |
| `Option<W>` where `W` is `#[repr(transparent)]` wrapping an NPO type | `W`                         | Null-pointer optimization is carried through transparent wrappers.                                        |
| `ManuallyDrop<T>` / `UnsafeCell<T>` / `MaybeUninit<T>`               | `T`                         | Zero-cost wrappers are stripped before emission.                                                          |
| `PhantomData<T>` / `PhantomPinned`                                   | _(field skipped)_           | Zero-sized types aren't emitted.                                                                          |
| `usize` / `isize`                                                    | `uintptr_t` / `intptr_t`    | They become `size_t` / `ptrdiff_t` if `usize_is_size_t` is set to `true`, either globally or per-package. |
| `std::ffi::c_int`, `c_char`, …                                       | C's native `int`, `char`, … | The full `core::ffi` / `std::ffi` C primitive aliases are recognised.                                     |

## 6. Emit

The final IR is rendered into one or more C headers. The exact shape (the
order of sections, what gets `#include`d, whether you get one file or multiple) is
controlled by your `cheadergen.toml` and CLI flags. See
[Anatomy of a generated header](../what-you-get/header-structure.md) and
[Bundled vs partitioned output](../what-you-get/partitioning.md) for more details.
