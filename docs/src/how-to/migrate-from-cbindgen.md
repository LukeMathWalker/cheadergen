# Migrate from `cbindgen`

This guide walks through moving an existing `cbindgen`-using crate to
`cheadergen`. Before you start, read
[Comparison with `cbindgen`](../limits-and-alternatives/vs-cbindgen.md) and
[Limitations](../limits-and-alternatives/limitations.md) so you know what features won't
carry over.

## 1. Install `cheadergen`

See [Install `cheadergen`](../getting-started/install.md).

## 2. Translate your `cbindgen.toml`

```bash
cheadergen config translate \
    --from cbindgen.toml \
    --to cheadergen.toml
```

The translator covers the option set `cheadergen` supports. Any `cbindgen`-only
options are reported and need a manual decision.

A spot-check, before:

```toml
# cbindgen.toml
language = "C"
include_guard = "MYLIB_H"
pragma_once = true

[parse]
parse_deps = true
```

after:

```toml
# cheadergen.toml
pragma_once = true

[header."mylib"]
include_guard = "MYLIB_H"
```

`cheadergen` doesn't have an equivalent of `parse_deps` because the rustdoc
pipeline always sees dependency types when they're reachable from `extern "C"`
items — the option simply isn't needed.

## 3. Rewrite `cbindgen` annotations

`cbindgen` reads doc-comment annotations like `/// cbindgen:rename-all=...`;
`cheadergen` uses proc-macro attributes. Sweep your source and translate them
using this table:

| `cbindgen` (doc comment)                     | `cheadergen` (proc-macro attr)                               |
| -------------------------------------------- | ------------------------------------------------------------ |
| `/// cbindgen:no-export`                     | `#[cheadergen::config(skip)]`                                |
| `/// cbindgen:ignore`                        | `#[cheadergen::config(skip)]`                                |
| `/// cbindgen:rename-all=ScreamingSnakeCase` | `#[cheadergen::config(rename_all = "SCREAMING_SNAKE_CASE")]` |
| `/// cbindgen:rename-all=SnakeCase`          | `#[cheadergen::config(rename_all = "snake_case")]`           |
| `/// cbindgen:rename-all=CamelCase`          | `#[cheadergen::config(rename_all = "camelCase")]`            |
| `/// cbindgen:field-names=[x, y]`            | `#[cheadergen::config(field_names(x, y))]`                   |
| `/// cbindgen:prefix-with-name`              | `#[cheadergen::config(prefix_with_name)]`                    |
| `/// cbindgen:bitfield` (on a field)         | `#[cheadergen(bitfield = N)]`                                |

You'll also need to add the crate dependency:

```toml
# Cargo.toml
[dependencies]
cheadergen = "0.2"
```

The proc-macro lives in the `cheadergen` crate; you don't need to depend on
`cheadergen_cli` (that's the CLI binary).

> **Tip.** The proc-macro attributes are _type-checked at compile time_. If
> you write `#[cheadergen::config(field_namse(x, y))]` (note the typo),
> `cargo build` will reject it. `cbindgen`'s doc-comment annotations would
> silently do nothing.

## 4. Regenerate and diff

```bash
cheadergen generate --output-dir include --bundle
```

Pass `--bundle` to match `cbindgen`'s "single file per crate" shape. Once you
have output, diff it against your committed `cbindgen` header:

```bash
diff -u old-include/mylib.h include/mylib.h
```

Expect differences in these places:

- **`#[cfg]`-gated items.** `cheadergen` sees only the items reachable for the
  current build target; `cbindgen` translates them to `#ifdef`s. There is no
  way to make these match — see [Limitations](../limits-and-alternatives/limitations.md).
- **Array length constants.** `[i32; FOO]` becomes `int32_t x[FOO]` in
  `cbindgen` and `int32_t x[10]` in `cheadergen`.
- **Whitespace.** Both tools format their output, but the rules differ;
  small whitespace diffs are normal.
- **Doc-comment formatting.** Doxygen-style vs plain C-comment choices are
  controlled by `documentation_style`.
- **Item ordering** may differ between `cheadergen` and `cbindgen`.

If anything else looks unexpectedly different, open an
[issue](https://github.com/LukeMathWalker/cheadergen/issues) with the input
and a diff.

## 5. Decide bundled vs partitioned

`--bundle` is the closest match to `cbindgen`'s behaviour. Once the bundled
output is right, consider whether
[partitioned mode](../what-you-get/partitioning.md) would suit your project
better:

- Workspaces with multiple FFI-facing crates almost always benefit.
- Single-crate libraries usually don't.

You can switch at any time; the only difference is the CLI flag (or the
`bundle` setting in `cheadergen.toml`).

## 6. Wire it into your build

If your project ran `cbindgen` from a `build.rs`, you'll need to relocate the
invocation: `cheadergen` can't run inside `build.rs` (it deadlocks against
Cargo's target-dir lock). Move header generation into a command runner that
sequences `cheadergen` before `cargo build`; see
[Integrate with Cargo](./integrate-with-cargo.md) for the recipe.

## Common migration pitfalls

- **You forgot to add `cheadergen` to `[dependencies]`.** Without it, the
  `#[cheadergen::config(...)]` attribute fails to resolve. Errors surface
  during `cargo build`.
- **You translated `cbindgen.toml` but never deleted it.** Both tools
  cheerfully ignore the other's config. Remove `cbindgen.toml` (or rename it
  to `cbindgen.toml.bak` until you're confident) to avoid confusion.
- **The required nightly isn't installed.** Follow the
  installation instructions in the error returned by `cheadergen generate` to
  install the missing toolchain.
