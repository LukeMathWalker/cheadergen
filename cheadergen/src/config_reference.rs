//! Reference for `cheadergen.toml`.
//!
//! `cheadergen.toml` is a [TOML](https://toml.io) file that controls how the
//! `cheadergen` CLI generates C/C++ headers for a workspace. It is optional —
//! when omitted, every option falls back to its default. Pass it via
//! `cheadergen generate --config path/to/cheadergen.toml`.
//!
//! This page documents every option the file accepts. For per-item annotations
//! (placed on Rust types and functions in your source code), see
//! [`config`](super::config) instead.
//!
//! # Override priority
//!
//! Several options can be set in more than one place. When the same option is
//! set in multiple sections, the most specific value wins:
//!
//! 1. CLI flags (e.g. `--style`, `--cpp-compat`, `--bundle`) — highest priority.
//! 2. `[header.<name>.c]` / `[header.<name>.cxx]` — per-header, language-specific.
//! 3. `[header.<name>]` — per-header, language-agnostic.
//! 4. `[c]` / `[cxx]` — global, language-specific.
//! 5. Top-level keys — lowest priority.
//!
//! Unknown keys are rejected at parse time, so typos surface as errors instead
//! of silently doing nothing.
//!
//! # Top-level keys
//!
//! These keys live at the root of the file. They apply to every generated
//! header in every language unless overridden by a more specific section.
//!
//! ### `preamble`
//!
//! - **Type**: string
//! - **Default**: none
//!
//! Verbatim text prepended to each generated header (typically a license block).
//!
//! ```toml
//! preamble = """
//! /* Copyright (c) 2026 Acme Corp. All rights reserved. */
//! """
//! ```
//!
//! ### `trailer`
//!
//! - **Type**: string
//! - **Default**: none
//!
//! Verbatim text appended at the end of each generated header.
//!
//! ### `autogen_warning`
//!
//! - **Type**: string
//! - **Default**: none
//!
//! Warning text emitted between major sections of the generated header to
//! discourage manual edits.
//!
//! ```toml
//! autogen_warning = "/* WARNING: this file is auto-generated. Do not edit. */"
//! ```
//!
//! ### `pragma_once`
//!
//! - **Type**: bool
//! - **Default**: `false`
//!
//! When `true`, emit `#pragma once` at the top of each header. Independent of
//! the per-header [`include_guard`](#headernameinclude_guard).
//!
//! ### `usize_is_size_t`
//!
//! - **Type**: bool
//! - **Default**: `false`
//!
//! When `true`, Rust's `usize` and `isize` translate to C's `size_t` and
//! `ptrdiff_t` instead of the default `uintptr_t` and `intptr_t`.
//!
//! Resolution is per *defining* package. The setting on a `[package.<name>]`
//! section (see [`[package.<name>].usize_is_size_t`](#packagenameusize_is_size_t))
//! takes priority over this global value for items defined in that package —
//! including generic instantiations, which follow the generic's defining
//! crate. Items in your own crate (`extern "C"` functions, statics, struct
//! fields, etc.) use this global value unless you list your own crate under
//! `[package.<name>]` to override it.
//!
//! ### `includes`
//!
//! - **Type**: array of string
//! - **Default**: `[]`
//!
//! Extra `#include` directives to emit in every header.
//!
//! Entries wrapped in angle brackets become system includes; everything else
//! is rendered with double quotes.
//!
//! ```toml
//! includes = ["<stddef.h>", "my_types.h"]
//! ```
//!
//! Renders as:
//!
//! ```c
//! #include <stddef.h>
//! #include "my_types.h"
//! ```
//!
//! ### `no_includes`
//!
//! - **Type**: bool
//! - **Default**: `false`
//!
//! When `true`, suppress the language's default includes (e.g. `<stdint.h>`,
//! `<stdbool.h>` for C). Useful when the consumer provides its own freestanding
//! type definitions.
//!
//! ### `after_includes`
//!
//! - **Type**: string
//! - **Default**: none
//!
//! Verbatim text inserted immediately after the `#include` block. A common use
//! is to predeclare types referenced from a [`skip`](#packagenametypes)ped
//! dependency:
//!
//! ```toml
//! after_includes = """
//! typedef struct MyTypeTag MyType;
//! """
//! ```
//!
//! ### `documentation`
//!
//! - **Type**: bool
//! - **Default**: `true`
//!
//! When `true`, Rust doc comments (`///`) are emitted as comments in the
//! generated header. Set to `false` to strip them entirely.
//!
//! ### `documentation_style`
//!
//! - **Type**: string — one of `auto`, `c`, `c99`, `doxy`, `cxx`
//! - **Default**: `auto`
//!
//! Comment syntax used for emitted doc comments. See
//! [Enumerated values](#enumerated-values) for the meaning of each variant.
//!
//! ### `documentation_length`
//!
//! - **Type**: string — one of `full`, `short`
//! - **Default**: `full`
//!
//! How much of each Rust doc comment to emit. `short` keeps only the first
//! paragraph (up to the first blank line); `full` keeps everything.
//!
//! ### `sort_by`
//!
//! - **Type**: string — one of `source_order`, `name`
//! - **Default**: `source_order`
//!
//! Default emission order for items of every kind (functions, statics,
//! constants). Can be overridden per kind via [`[fn]`](#fn-section),
//! [`[static]`](#static-section), or [`[constant]`](#constant-section).
//!
//! ### `bundle`
//!
//! - **Type**: bool
//! - **Default**: `false`
//!
//! Emission mode toggle:
//!
//! - `false` (partitioned mode): one header per crate. Cross-crate type
//!   references become `#include` directives.
//! - `true` (bundle mode): one combined header per target package, inlining
//!   types from every dependency.
//!
//! Bundle mode is incompatible with [`[package.<name>].header_name`](#packagenameheader_name).
//! The CLI flag `--bundle` overrides this setting.
//!
//! # Item-kind sections
//!
//! These sections override [`sort_by`](#sort_by) for a specific item kind, and
//! the `[enum]` section additionally controls C-enum naming.
//!
//! ## `[fn]` section
//!
//! ### `[fn].sort_by`
//!
//! - **Type**: string — one of `source_order`, `name`
//! - **Default**: inherits the top-level [`sort_by`](#sort_by) (which itself
//!   defaults to `source_order`)
//!
//! Emission order for `extern "C"` functions.
//!
//! ## `[static]` section
//!
//! ### `[static].sort_by`
//!
//! - **Type**: string — one of `source_order`, `name`
//! - **Default**: inherits the top-level [`sort_by`](#sort_by)
//!
//! Emission order for exported `static` items.
//!
//! ## `[constant]` section
//!
//! ### `[constant].sort_by`
//!
//! - **Type**: string — one of `source_order`, `name`
//! - **Default**: inherits the top-level [`sort_by`](#sort_by)
//!
//! Emission order for exported `const` items.
//!
//! ## `[enum]` section
//!
//! ### `[enum].prefix_with_name`
//!
//! - **Type**: bool
//! - **Default**: `false`
//!
//! When `true`, every variant of every C enum is prefixed with the enum's name
//! and an underscore. C enums share a global namespace, so this is a common
//! way to avoid name collisions.
//!
//! ```toml
//! [enum]
//! prefix_with_name = true
//! ```
//!
//! With this set, an enum `Status { Ok, Error }` emits variants `Status_Ok`
//! and `Status_Error` instead of bare `Ok`/`Error`. Individual enums can opt
//! out via `#[cheadergen::config(prefix_with_name = false)]`.
//!
//! # `[package.<name>]` sections
//!
//! Per-dependency-package configuration. The key after `package.` is a Cargo
//! crate name, optionally suffixed with `@<version-req>` to disambiguate when
//! the same crate name appears at multiple versions in the dependency graph:
//!
//! ```toml
//! [package.my-dep]
//! types = "opaque"
//!
//! [package."serde@1.0"]
//! types = "skip"
//! ```
//!
//! ### `[package.<name>].types`
//!
//! - **Type**: string — one of `opaque`, `skip`
//! - **Default**: emit definitions normally
//!
//! Controls what is emitted for types from this dependency:
//!
//! - `opaque` — emit only forward declarations. Consumers can hold pointers
//!   to these types but cannot inspect their layout.
//! - `skip` — emit nothing. Consumers must supply the type definitions
//!   themselves (e.g. via [`after_includes`](#after_includes) or a separate
//!   header in [`includes`](#includes)).
//!
//! ### `[package.<name>].header_name`
//!
//! - **Type**: string
//! - **Default**: derived from the crate name
//!
//! Override the on-disk base name of the header generated for this package
//! (partitioned mode only). Must be a bare filename: no path separators, no
//! file extension — the language extension (`.h`, `.hpp`) is appended
//! automatically.
//!
//! ```toml
//! [package.my-internal-crate]
//! header_name = "internal"
//! ```
//!
//! Rejected in [`bundle`](#bundle) mode. Two packages cannot share the same
//! `header_name`.
//!
//! ### `[package.<name>].usize_is_size_t`
//!
//! - **Type**: bool
//! - **Default**: inherits the top-level [`usize_is_size_t`](#usize_is_size_t)
//!
//! Override the global `usize_is_size_t` for items defined in this package.
//! Use this to flip the spelling for a specific dependency without affecting
//! the rest of the workspace:
//!
//! ```toml
//! [package.my-dep]
//! usize_is_size_t = true
//! ```
//!
//! Resolution is per *defining* package, so a generic from this dependency
//! (e.g. `Vec<usize>`) follows the package's setting wherever it is
//! instantiated.
//!
//! # `[header.<name>]` sections
//!
//! Per-generated-header overrides. The key after `header.` matches a generated
//! header's base name (the crate name, or the value of
//! [`header_name`](#packagenameheader_name) if one was set):
//!
//! ```toml
//! [header.my-lib]
//! include_guard = "MY_LIB_H"
//! preamble = "/* Custom preamble for my-lib only. */"
//! ```
//!
//! Every common option from the top level can be overridden here. Only options
//! listed below behave specially.
//!
//! ### `[header.<name>].include_guard`
//!
//! - **Type**: string
//! - **Default**: none
//!
//! Custom `#ifndef`/`#define` include guard macro. Available **only** in
//! `[header.<name>]` sections — setting it at the top level or in a language
//! section is a parse error.
//!
//! ### Common option overrides
//!
//! The following keys, if set in a `[header.<name>]` section, override the
//! top-level value for that header only. Each behaves exactly as documented
//! at the top level:
//!
//! - [`preamble`](#preamble)
//! - [`trailer`](#trailer)
//! - [`autogen_warning`](#autogen_warning)
//! - [`pragma_once`](#pragma_once)
//! - [`includes`](#includes)
//! - [`no_includes`](#no_includes)
//! - [`after_includes`](#after_includes)
//! - [`documentation`](#documentation)
//! - [`documentation_style`](#documentation_style)
//! - [`documentation_length`](#documentation_length)
//! - [`sort_by`](#sort_by)
//!
//! ### Nested sections
//!
//! `[header.<name>]` may contain its own `[header.<name>.fn]`,
//! `[header.<name>.static]`, `[header.<name>.constant]`,
//! `[header.<name>.enum]`, `[header.<name>.c]`, and `[header.<name>.cxx]`
//! sub-sections. Each accepts the same keys as its top-level counterpart and
//! overrides the global value for that header.
//!
//! # `[c]` and `[cxx]` sections
//!
//! Language-specific overrides. Settings in `[c]` apply only when generating
//! C output; settings in `[cxx]` apply only to C++ output.
//!
//! Note that C++ and Cython output are currently rejected at validation time —
//! `[cxx]` is parsed but unused, and the only legal `--lang` value today is `c`.
//!
//! ### `[c].style`
//!
//! - **Type**: string — one of `both`, `tag`, `type`
//! - **Default**: `both`
//!
//! Declaration style for C structs and enums:
//!
//! - `both` — `typedef struct MyType { ... } MyType;` (tag plus typedef).
//! - `tag` — `struct MyType { ... };` (tag only; consumers write `struct MyType`).
//! - `type` — `typedef struct { ... } MyType;` (typedef only; no tag name).
//!
//! Equivalent to the `--style` CLI flag, which takes precedence when set.
//!
//! ### `[c].cpp_compat`
//!
//! - **Type**: bool
//! - **Default**: `false`
//!
//! When `true`, wrap generated declarations in an `extern "C"` block guarded
//! by `#ifdef __cplusplus`, so the header is safe to include from C++.
//! Equivalent to the `--cpp-compat` CLI flag.
//!
//! ### Common option overrides
//!
//! The same overridable keys listed under
//! [`[header.<name>]`](#common-option-overrides) — `preamble`, `trailer`,
//! `autogen_warning`, `pragma_once`, `includes`, `no_includes`,
//! `after_includes`, `documentation`, `documentation_style`,
//! `documentation_length` — can also be set in `[c]` or `[cxx]` to apply only
//! to that language's output.
//!
//! # Enumerated values
//!
//! ## `sort_by`
//!
//! - `source_order` — emit items in the order they appear in their Rust
//!   source file. The default.
//! - `name` — sort items alphabetically by name.
//!
//! ## `documentation_style`
//!
//! - `auto` — pick a sensible style for the target language: C-style block
//!   comments (`/** ... */`) for C output, C++ line comments (`///`) for C++.
//!   The default.
//! - `c` — always use C-style block comments: `/** ... */`.
//! - `c99` — always use C99/C++ line comments: `// ...`.
//! - `doxy` — Doxygen block comments: `/** ... */`.
//! - `cxx` — C++ triple-slash comments: `/// ...`.
//!
//! ## `documentation_length`
//!
//! - `full` — emit the entire Rust doc comment. The default.
//! - `short` — emit only the first paragraph (up to the first blank line).
//!
//! ## `package.<name>.types`
//!
//! - `opaque` — emit forward declarations only.
//! - `skip` — emit nothing for types from this package.
//!
//! ## `[c].style`
//!
//! - `both` — emit both a tag and a typedef. The default.
//! - `tag` — emit only the tag.
//! - `type` — emit only the typedef.
//!
//! # Worked example
//!
//! A `cheadergen.toml` for a workspace that ships two crates (`engine` and
//! `engine-utils`), depends on `serde` (whose types should be opaque), and
//! wants C++ compatibility:
//!
//! ```toml
//! preamble = """
//! /* Copyright (c) 2026 Acme Corp. */
//! """
//! autogen_warning = "/* AUTOGENERATED — do not edit. */"
//! pragma_once = true
//!
//! [c]
//! style = "both"
//! cpp_compat = true
//!
//! [enum]
//! prefix_with_name = true
//!
//! [package.serde]
//! types = "opaque"
//!
//! [header.engine]
//! include_guard = "ACME_ENGINE_H"
//! includes = ["<stddef.h>"]
//!
//! [header.engine-utils]
//! include_guard = "ACME_ENGINE_UTILS_H"
//! ```
