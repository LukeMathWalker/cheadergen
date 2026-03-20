# Annotations

cheadergen uses proc-macro attributes to let users control how Rust items appear in
the generated C/C++ header. This document specifies the annotation surface, its
encoding, and the rules for each directive.

## User-facing attributes

There are two attributes, used at different levels:

- **Item-level:** `#[cheadergen::config(...)]` — applied to structs, enums, unions,
  type aliases, functions, or statics.
- **Field/variant-level:** `#[cheadergen(...)]` — applied to struct fields or enum
  variants. Requires an item-level `#[cheadergen::config(...)]` on the parent item
  (without it the proc macro never fires).

Using one attribute rather than many (`export`, `rename`, `skip`, …) keeps
directives composable:

```rust
#[cheadergen::config(export(opaque), rename = "Handle")]
pub struct InternalHandle { /* ... */ }
```

This mirrors how `serde`, `clap`, and other Rust ecosystems use a single
container attribute.

## Directives

### Item-level

Applied via `#[cheadergen::config(...)]` to structs, enums, unions, type aliases,
functions, or statics.

| Directive                  | Applies to                                        | Semantics                                                                                               |
| -------------------------- | ------------------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| `export`                   | struct, enum, union, type alias                   | Force inclusion in the header with a full definition, even if not reachable from any `extern "C"` item. |
| `export(opaque)`           | struct, enum, union, type alias                   | Force inclusion as an opaque forward declaration only.                                                  |
| `skip`                     | struct, enum, union, type alias, function, static | Exclude the item from the header even if discovered via FFI traversal.                                  |
| `rename = "CName"`         | struct, enum, union, type alias, function, static | Override the C name emitted in the header.                                                              |
| `prefix_with_name`         | enum                                              | Prefix each variant name with the enum name (e.g. `Status_Ok`).                                         |
| `prefix_with_name = false` | enum                                              | Explicitly disable variant prefixing for this enum.                                                     |
| `field_names(a, b, …)`     | struct (tuple only)                               | Assign C field names to positional fields.                                                              |

`export` is only meaningful on types. Functions and statics with `extern "C"` +
`#[no_mangle]` are auto-included; applying `export` to them is a compile error.

`export` is idempotent: applying it to a type that is already reachable via FFI
traversal has no extra effect on inclusion, but the remaining directives
(`rename`, etc.) still apply. This avoids needing a separate "modify-only" form
for types that are already discovered.

### Field-level

Applied to struct fields via `#[cheadergen(...)]`.

| Directive           | Semantics                                    |
| ------------------- | -------------------------------------------- |
| `rename = "c_name"` | Override the C field name.                   |
| `bitfield = N`      | Emit the field as a C bitfield with width N. |

### Variant-level

Applied to enum variants via `#[cheadergen(...)]`.

| Directive           | Semantics                    |
| ------------------- | ---------------------------- |
| `rename = "C_NAME"` | Override the C variant name. |

## Syntax examples

```rust
// Force-include a type that isn't reachable from any extern "C" item
#[cheadergen::config(export)]
#[repr(C)]
pub struct Config {
    pub width: u32,
    pub height: u32,
}

// Rename the C type and a field; mark a field as a bitfield
#[cheadergen::config(export, rename = "CConfig")]
#[repr(C)]
pub struct Config2 {
    #[cheadergen(rename = "raw_width")]
    pub width: u32,
    #[cheadergen(bitfield = 8)]
    pub flags: u8,
}

// Opaque forward declaration
#[cheadergen::config(export(opaque))]
pub struct OpaqueHandle {
    _inner: u64,
}

// Exclude a function from the header
#[cheadergen::config(skip)]
#[unsafe(no_mangle)]
pub extern "C" fn internal_helper() {}

// Customize an already-discovered type (no export needed)
#[cheadergen::config(rename = "CPoint")]
#[repr(C)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

// Prefix enum variants with the enum name → Status_Ok, Status_Error
#[cheadergen::config(prefix_with_name)]
#[repr(C)]
pub enum Status {
    Ok,
    Error,
    Pending,
}

// Disable prefix for a specific enum
#[cheadergen::config(prefix_with_name = false)]
#[repr(C)]
pub enum Flags {
    A,
    B,
    C,
}

// Assign C field names to a tuple struct
#[cheadergen::config(field_names(x, y))]
#[repr(C)]
pub struct Point2D(f64, f64);

// Rename a specific variant
#[cheadergen::config(export)]
#[repr(C)]
pub enum Color {
    Red,
    #[cheadergen(rename = "COLOR_GREEN")]
    Green,
    Blue,
}
```

## Encoding via diagnostic attributes

Users never write `#[diagnostic::...]` directly. The `#[cheadergen::config(...)]`
proc macro (in `cheadergen_macros`) translates each directive into a
`#[diagnostic::cheadergen::...]` attribute that rustdoc preserves in its JSON
output as `Attribute::Other(String)`.

This is the same encoding trick used by [Pavex](https://github.com/LukeMathWalker/pavex).

### Translation table

| User writes                | Proc macro emits                                     |
| -------------------------- | ---------------------------------------------------- |
| `export`                   | `#[diagnostic::cheadergen::export]`                  |
| `export(opaque)`           | `#[diagnostic::cheadergen::export(opaque)]`          |
| `skip`                     | `#[diagnostic::cheadergen::skip]`                    |
| `rename = "Foo"`           | `#[diagnostic::cheadergen::rename("Foo")]`           |
| `prefix_with_name`         | `#[diagnostic::cheadergen::prefix_with_name]`        |
| `prefix_with_name = false` | `#[diagnostic::cheadergen::prefix_with_name(false)]` |
| `field_names(x, y)`        | `#[diagnostic::cheadergen::field_names(x, y)]`       |
| `bitfield = 8`             | `#[diagnostic::cheadergen::bitfield(8)]`             |

### Field and variant attributes

The parent item's proc macro invocation processes the full item token tree.
When it encounters `#[cheadergen(...)]` on a field or variant, it
rewrites it to the corresponding `#[diagnostic::cheadergen::...]` form and
strips the original. The compiler never sees `#[cheadergen(...)]` on
fields or variants — only the diagnostic encoding survives into rustdoc JSON.

### Reading annotations back

The `CheadergenVisitor` in the indexer inspects `Attribute::Other` strings
during crate traversal, looking for the `diagnostic::cheadergen::` prefix.
Item-level directives (`export`, `skip`, `rename`, `prefix_with_name`,
`field_names`) are captured during indexing. Field-level directives (`rename`,
`bitfield`) are read from `field_item.attrs` during type resolution, since
struct fields are already accessed individually at that point.

## Validation

The proc macro rejects invalid usage at compile time:

- `export` on a function or static → error.
- `opaque` outside of `export(...)` → error (it's not a standalone directive).
- `prefix_with_name` on a non-enum → error.
- `field_names` on a non-tuple struct → error.
- `bitfield` on a non-field → error.
- Unknown directives → error.
- `export` and `skip` on the same item → error.

## Relationship to cbindgen annotations

cbindgen uses doc-comment annotations (`/// cbindgen:rename-all=ScreamingSnakeCase`).
cheadergen covers the C-relevant subset with proper proc-macro attributes:

| cbindgen                            | cheadergen                                                    |
| ----------------------------------- | ------------------------------------------------------------- |
| `/// cbindgen:no-export` / `ignore` | `skip`                                                        |
| `/// cbindgen:rename-all=...`       | `rename = "..."` (per-item; global rename strategy is config) |
| `/// cbindgen:field-names=[x, y]`   | `field_names(x, y)`                                           |
| `/// cbindgen:prefix-with-name`     | `prefix_with_name`                                            |
| `/// cbindgen:bitfield`             | `bitfield = N`                                                |

C++-only cbindgen annotations (`derive-eq`, `derive-ostream`, `enum-class`,
constructor/destructor attributes, etc.) are out of scope for the initial
implementation.
