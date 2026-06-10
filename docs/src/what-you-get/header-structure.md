# Anatomy of a generated header

Every header `cheadergen` writes follows the same fixed section order. This page
walks through that order top-to-bottom and points at the
[`cheadergen.toml`](https://docs.rs/cheadergen/latest/cheadergen/config_reference/index.html)
options that influence each section.

Knowing the structure helps when **reading a generated header** and if you're trying to
**customise the output**.

Options like `preamble`, `[header.<name>].include_guard`, and `[package.<name>]`
live in `cheadergen.toml`. See [Global configuration](../foundations/global-configuration.md) for how
the file is discovered and loaded; this page focuses on _which_ section
each option affects.

## The sections, in order

| #  | Section                                                                            | Configured by                                                                                                                   |
| -- | ---------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| 1  | Preamble (verbatim text)                                                           | `preamble`                                                                                                                      |
| 2  | Include guard or `#pragma once`                                                    | `[header.<name>].include_guard`, `pragma_once`                                                                                  |
| 3  | Autogen warning                                                                    | `autogen_warning`                                                                                                               |
| 4  | Standard `<...>` includes                                                          | `no_includes`, `usize_is_size_t` (adds `<stddef.h>`)                                                                            |
| 5  | User `includes`                                                                    | `includes`                                                                                                                      |
| 6  | Dependency-crate `#include`s                                                       | partitioned mode only (automatic)                                                                                               |
| 7  | After-includes (verbatim text)                                                     | `after_includes`                                                                                                                |
| 8  | `CHEADERGEN_ALIGNED(n)` macro                                                      | emitted automatically if any type has `#[repr(C, align(N))]`                                                                    |
| 9  | Type definitions (structs, unions, enums, typedefs) and their associated constants | `documentation`, `documentation_style`, `documentation_length`, `style`, `sort_by`, `[package.<name>] types`, `usize_is_size_t` |
| 10 | Free `const` items as `#define` macros                                             | `[constant].sort_by`, `constants` opt-in flags                                                                                  |
| 11 | `extern "C" { … }` open brace                                                      | `cpp_compat` (C-only)                                                                                                           |
| 12 | `static` declarations                                                              | `[static].sort_by`                                                                                                              |
| 13 | Function declarations                                                              | `[fn].sort_by`                                                                                                                  |
| 14 | `extern "C"` close brace                                                           | `cpp_compat`                                                                                                                    |
| 15 | Include guard close                                                                | `include_guard`                                                                                                                 |
| 16 | Trailer (verbatim text)                                                            | `trailer`                                                                                                                       |

> **Where this lives in the code.** The emission order is defined in
> [`cheadergen_cli/src/codegen.rs`](https://github.com/LukeMathWalker/cheadergen/blob/main/cheadergen_cli/src/codegen.rs)
> (`generate_c_header`). If a discrepancy ever creeps in between this page and
> the source, the source wins — please open an issue.

## A worked example

Given this Rust crate:

```rust
#[repr(C)]
pub struct Foo {}

impl Foo {
    #[cheadergen::config(export)]
    pub const U8: u8 = 1;
    #[cheadergen::config(export)]
    pub const GREETING: &str = "hello world";
}

#[unsafe(no_mangle)]
pub extern "C" fn root(x: Foo) {}
```

`cheadergen generate -l c -o include .` produces:

```c
/* [3] WARNING: this file was auto-generated by cheadergen from `demo`. Do not edit it manually. */

/* [4] Standard includes */
#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/* [9] Type definition, with its associated constants attached */
typedef struct {

} Foo;
#define Foo_U8 1
#define Foo_GREETING "hello world"

/* [13] Function declaration */
void root(Foo x);
```

(The bracketed numbers correspond to the section table above; `cheadergen` itself
doesn't emit them.)

## Notes on individual sections

### Preamble & trailer (1, 16)

Verbatim text. Useful for license blocks, project-wide banners, or compiler
pragmas that aren't part of `cheadergen`'s vocabulary. Both are simple
string-substitution, `cheadergen` does not parse what you put in them.

### Include guard vs `#pragma once` (2, 15)

If `[header.<name>].include_guard = "FOO_H"` is set, you get an
`#ifndef`/`#define`/`#endif` triple wrapping the file. If `pragma_once = true`
is set, you get `#pragma once`. The two are independent, you can use both or
neither.

### Autogen warning (3)

By default `cheadergen` synthesizes a warning that names the crate the header
came from (workspace-relative path for workspace members, `name@version`
otherwise). Set `autogen_warning = ""` to suppress it entirely; set it to a
string to override the default.

### Standard includes (4)

`cheadergen` always emits `<stdarg.h>`, `<stdbool.h>`, `<stdint.h>`, and
`<stdlib.h>` unless you set `no_includes = true`. `<stddef.h>` is added only
when any item in the header was lowered with `usize_is_size_t = true` (because
that flag introduces `size_t` and `ptrdiff_t`).

### Dependency includes (6)

In [partitioned mode](./partitioning.md) the dependency crate that defines a
referenced type gets its own header file, and the consuming header gets a
matching `#include "dep.h"`. You don't need to configure this manually in `cheadergen.toml`,
`cheadergen` computes the set of includes based on actual type usage.

### Type definitions (9)

The heart of the header. Each `#[repr(C)]` struct/enum/union (or anything
forced in via `#[cheadergen::config(export)]`) becomes a definition here.
Associated `const`s attached to a type are emitted immediately after the type
they belong to as `#define`s.

A topological sort guarantees that by-value type dependencies are defined
before their users; for ties, `sort_by` controls the secondary order.

### The `extern "C"` block (11, 14)

C++ consumers need an `extern "C"` block to disable name mangling. `cheadergen`
emits one around functions and statics when the `cpp_compat` option (or
`--cpp-compat` CLI flag) is set on a C header.

## Per-header configuration

Sections 1–16 can all be tuned globally or per individual output header via the
`[header.<name>]` section in `cheadergen.toml`. CLI flags (such as
`--cpp-compat`, `--bundle`) trump both. The full override priority is
documented in the
[config reference](https://docs.rs/cheadergen/latest/cheadergen/config_reference/index.html#override-priority).
