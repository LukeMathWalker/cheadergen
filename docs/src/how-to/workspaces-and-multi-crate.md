# Generate headers for a Cargo workspace

`cheadergen` was designed for Cargo workspaces with multiple FFI-facing
crates that share types. In partitioned mode (the default) the right shape
falls out automatically: each crate that contributes types gets its own
header, and consuming crates `#include` it.

This guide shows the common patterns. For the mental model of which
crates participate in a run and why, see
[Target packages](../foundations/target-packages.md).

## Select multiple target packages

Pass `--package` multiple times to generate headers for several workspace members
in one invocation:

```bash
cheadergen generate \
    --output-dir include \
    --package alpha \
    --package beta
```

Each `--package` argument names a workspace member that should produce a "target"
header (one with `extern "C"` function declarations, not just types).

Alternatively, omit `--package` and pass a workspace directory; `cheadergen` will
target every library member in that directory:

```bash
cheadergen generate \
    --output-dir include \
    /workspace/directory
```

## What gets generated

Given two targets `alpha` and `beta` that both use a `Config` type from a
workspace dependency `core`:

```
include/
├── alpha.h     # target — extern "C" functions and types unique to alpha
├── beta.h      # target — extern "C" functions and types unique to beta
└── core.h      # types-only — Config and other shared types
```

- Both `alpha.h` and `beta.h` start with `#include "core.h"`.
- `Config` is defined exactly once, in `core.h`.
- `core.h` has no `extern "C"` block — `core` isn't a target, just a type
  contributor.

A non-target dependency only gets a header if at least one target actually
uses its types. External crates (from crates.io) follow the same rules.

## Customising header filenames

By default a crate named `my-crate` produces `my_crate.h` (hyphens become
underscores). Override the
base name per package with `header_name`:

```toml
# cheadergen.toml
[package."core"]
header_name = "myproject_core"
```

Now `alpha.h` and `beta.h` will `#include "myproject_core.h"` and the file
on disk is renamed to match.

If a dependency name is ambiguous (multiple versions present), disambiguate
with `name@version`:

```toml
[package."internal-state@1"]
header_name = "state_v1"
```

## Pruning stale output

Add `--prune-orphans` to clean up any `*.h` files in the output directory
that this run didn't write. Useful when you remove types or packages:

```bash
cheadergen generate \
    --output-dir include \
    --prune-orphans \
    -p alpha \
    -p beta
```

Without `--prune-orphans`, removing a crate from the workspace leaves its
header behind as silent stale output.

`--prune-orphans` only acts on top-level files matching the language
extension (`*.h` for C); it never touches subdirectories
or unrelated files.

## Skipping empty headers

Some non-target crates contribute _no_ concrete C-visible types, only
generic definitions that get monomorphized into their consumers' headers.
By default `cheadergen` still writes an (empty) header file for those crates.
Pass `--skip-empty` to suppress the empty files:

```bash
cheadergen generate \
    --output-dir include \
    --skip-empty \
    --prune-orphans \
    .
```

`--skip-empty` is partitioned-only (it's rejected with `--bundle`).
Combining it with `--prune-orphans` ensures empty-by-deletion headers also
get cleaned up.

## See also

- [Bundled vs partitioned output](../what-you-get/partitioning.md) — the
  conceptual model behind the per-crate header layout.
- [Generics and monomorphization](../how-it-works/generics-and-monomorphization.md) —
  where `Wrapper<i32>` actually ends up.
- The [`cheadergen.toml` reference](https://docs.rs/cheadergen/latest/cheadergen/config_reference/index.html)
  for the full `[package.<name>]` schema.
