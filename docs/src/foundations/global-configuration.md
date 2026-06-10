# Global configuration

Beyond CLI flags, `cheadergen` reads an optional **`cheadergen.toml`**. It's
where you keep project-wide customisation: anything you'd rather not
retype on every invocation, or anything that doesn't have a CLI flag at all.

This page covers how the file is loaded and what categories of options it
holds. For the exhaustive option-by-option schema, see the canonical
[`cheadergen.toml` reference][config_ref] on docs.rs.

## How it's loaded

There is **no auto-discovery**. `cheadergen` only reads a config file when
you point at one explicitly:

```bash
cheadergen generate --config cheadergen.toml --output-dir include
```

Without `--config`, `cheadergen` runs with the same defaults as an empty
`cheadergen.toml`. Every option in the reference has a documented default.

The file's location on disk doesn't matter. A workspace can store its
config anywhere convenient (`cheadergen.toml` at the workspace root,
`tools/cheadergen.toml`, etc.) and pass the path through `--config`.

## What it can configure

The config file is grouped by scope. At a glance:

- **Top-level keys** are global emission defaults: `preamble`, `after_includes`,
  `autogen_warning`, `pragma_once`, `no_includes`, `documentation`,
  `documentation_style`, `documentation_length`, `style`, `cpp_compat`,
  `usize_is_size_t`, `bundle`, `sort_by`, …
- **`[header.<name>]`** introduces overrides scoped to a single output header (e.g.
  `[header.alpha]` only affects `alpha.h`). Accepts most top-level keys
  plus `include_guard`.
- **`[package.<name>]`** introduces overrides scoped to a single Rust package (e.g.
  `[package."my-crate"]`). Controls `header_name`, `types = "opaque" |
  "skip"`, `usize_is_size_t`, and per-package emission tweaks.
- **`[fn]`, `[static]`, `[constant]`** set the default sort order and emission rules for
  function declarations, static items, and constant macros.

Each of those subsections has its own page in the
[reference][config_ref] with the full list of keys and accepted values.

## CLI vs config precedence

CLI flags trump the config file: `--bundle`, `--cpp-compat`, `--style`, and
the rest override the corresponding TOML key when both are set. Per-header
sections (`[header.<name>]`) in turn override the top-level keys. The full
override priority is documented in the
[override priority section][override_priority] of the reference.

## See also

- [Anatomy of a generated header](../what-you-get/header-structure.md) — which sections
  each option influences.
- [Target packages](./target-packages.md) — `[package.<name>]` overrides
  apply regardless of target/dependency status, but a few directives only
  make sense for target packages.
- [`cheadergen.toml` reference][config_ref] — canonical, exhaustive schema.

[config_ref]: https://docs.rs/cheadergen/latest/cheadergen/config_reference/index.html
[override_priority]: https://docs.rs/cheadergen/latest/cheadergen/config_reference/index.html#override-priority
