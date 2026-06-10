# Item annotations

`cheadergen` uses a procedural macro to customize header generation
on a per-item basis: [`#[cheadergen::config(...)]`][config_ref].

**`#[cheadergen::config(...)]`** can be applied to structs, enums, unions, type
aliases, functions, constants or statics. It's an _item-level_ attribute.

From time to time, you may need field-level configuration. That's done via
**`#[cheadergen(...)]`**, the _member-level_ attribute.
It only takes effect when the parent item also carries `#[cheadergen::config(...)]`; without it, the proc macro
never fires and the inner attribute is ignored.

Both attributes accept a comma-separated list of directives, just like `serde`
or `clap`:

```rust
#[cheadergen::config(export, opaque, rename = "Handle")]
pub struct InternalHandle { /* ... */ }
```

This page sketches the _categories_ of directive so you know what's available
and where to look. The
[attribute reference on docs.rs](https://docs.rs/cheadergen/latest/cheadergen/attr.config.html)
is the authoritative list, including the validation rules.

## The five categories

### 1. Inclusion control

Decides whether an item shows up in the header at all.

| Directive | Meaning                                                                    |
| --------- | -------------------------------------------------------------------------- |
| `export`  | Force inclusion of a type that isn't reachable from any `extern "C"` item. |
| `skip`    | Exclude an item even if FFI traversal would normally pull it in.           |

`export` is meaningful only on types. Functions and statics with
`extern "C"` + `#[no_mangle]` are auto-included; applying `export` to them is
a compile error.

`export` is idempotent: applying it to a type that is already reachable from
FFI has no inclusion effect, but the other directives on the same attribute
still apply.

### 2. Shape

Changes how a type or field is rendered in C.

| Directive              | Where        | Meaning                                                                  |
| ---------------------- | ------------ | ------------------------------------------------------------------------ |
| `opaque`               | type         | Emit as a forward declaration (`typedef struct Foo Foo;`) when included. |
| `field_names(a, b, …)` | tuple struct | Assign C field names to positional fields.                               |
| `bitfield = N`         | field        | Emit as a C bitfield of width N.                                         |
| `const_ptr`            | field        | Qualify the resulting C pointer as `const T *`.                          |

`opaque` on its own doesn't force inclusion: it only changes the rendering
_if_ the type is included for some other reason. Combine it with `export` if
you also want it to appear regardless of FFI reachability.

### 3. Naming

Renames identifiers as they cross the Rust/C boundary.

| Directive                   | Where                                  | Meaning                                                     |
| --------------------------- | -------------------------------------- | ----------------------------------------------------------- |
| `rename = "CName"`          | type, function, static, field, variant | Override the C name.                                        |
| `rename_all = "..."`        | struct, enum, union                    | Bulk-rename fields or variants using a casing rule.         |
| `rename_all_fields = "..."` | enum (with struct variants)            | Bulk-rename fields inside the enum's struct variants.       |
| `prefix_with_name`          | enum                                   | Prefix variant names with the enum name (e.g. `Status_Ok`). |

The casing rules accept the same string literals as `serde`:
`"camelCase"`, `"PascalCase"`, `"snake_case"`, `"SCREAMING_SNAKE_CASE"`. A
per-field/per-variant `rename = "..."` always beats the bulk
`rename_all`/`rename_all_fields` rule.

### 4. Validation

The proc macro rejects nonsensical combinations at compile time. A few of the
more useful checks:

- `export` and `skip` on the same item → error.
- `export` on a function or static → error.
- `opaque` on a function, static, or constant → error.
- `prefix_with_name` on a non-enum → error.
- `field_names` on a non-tuple struct → error.
- `rename_all_fields` on an enum without any struct variants → error.
- Unknown casing strings, unknown directives → error.

Errors fire when you compile your Rust crate, not when you run `cheadergen` —
so you find out as part of your normal `cargo build` loop.

### 5. Type-aware runtime checks

A few directives can't be fully validated by the proc macro because they
depend on `cheadergen`'s type lowering:

- `const_ptr` requires that the field's resolved C type _is_ a pointer (e.g.
  `*mut T`, `NonNull<T>`). If lowering produces a non-pointer, `cheadergen`
  fails at generation time with a clear error.

## How it works under the hood

When the proc macro fires, it rewrites each directive into a
`#[diagnostic::cheadergen::...]` attribute that can be seen in the JSON output of `rustdoc`. `cheadergen` then picks up those
attributes when indexing that crate.

You don't need to know any of this to _use_ the
attributes, but it's a cool trick that's worth sharing!

## See also

- The [`#[cheadergen::config(...)]` reference][config_ref].
  The canonical list with every directive's exact semantics.

[config_ref]: https://docs.rs/cheadergen/latest/cheadergen/attr.config.html
