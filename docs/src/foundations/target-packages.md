# Target packages

When you run `cheadergen generate`, the first thing it has to figure out is
**which Rust packages to generate headers for**. That set is called the
**target packages**.

This page explains what a target package is, how the set is computed, and
why it matters when you reach for `#[cheadergen::config(export)]`.

## Target packages vs dependency packages

A **target package** is a package that `cheadergen` treats as a
header-emission target. Its `extern "C"` functions, `#[no_mangle]` statics,
and items annotated with `#[cheadergen::config(export)]` all get pulled into
the output.

A **dependency package** is anything else `cheadergen` reaches while
following type references: workspace siblings of a target, third-party
crates from crates.io, the standard library. Its _types_ may still appear in
the output (because a target's signature uses them), but its `extern "C"`
items and `#[cheadergen::config(export)]` annotations are **ignored**.

The distinction is local to a single `cheadergen generate` invocation. The
same crate can be a target in one run and a dependency in another, depending
on the flags you pass.

## How the target set is computed

`cheadergen generate` accepts three kinds of selectors. They combine in the
order below; the final set is what gets fed to the rest of the pipeline.

### `--input-dir` (repeatable)

```bash
cheadergen generate --output-dir include --input-dir <PATH>
```

`<PATH>` must be a **directory**: every workspace member whose manifest
lives under that directory is selected. Pointing at the workspace root
selects every workspace member; pointing at a subdirectory scopes the
selection to the crates inside it.

Pass `--input-dir` multiple times to combine several directories in a single
run. Each path must live inside the workspace rooted at the current working
directory; a path outside that workspace, or one that contains no workspace
members, is an error.

### `--package` / `-p` (repeatable)

```bash
cheadergen generate --output-dir include -p alpha -p beta
```

Adds workspace members by name on top of whatever `--input-dir` selected.
Each name must match a workspace member; unknown names produce an error.

If you pass `--package` flags without any `--input-dir`, the directory-based
step is skipped entirely; only the named packages end up in the target set.

### `--exclude` (repeatable)

```bash
cheadergen generate --output-dir include --exclude internal-tools
```

Subtracts workspace members by name from the set produced by the steps
above. Excluding a package that isn't in the set is an error.

### Library-only filter

After all three selectors run, `cheadergen` drops any package without a
library build target: `cheadergen` can only generate headers for library
crates. If you _explicitly_ named a binary-only package via `-p`, that's an
error; if a binary-only package was picked up implicitly (via `--input-dir`
or the default), it's silently skipped with a warning.

### Default

With no `--input-dir`, no `--package`, and no `--exclude`, `cheadergen` mirrors
Cargo's default:

- If the current working directory sits **inside a workspace member's
  directory**, only that member becomes a target. (When several candidates
  contain cwd — e.g. a hybrid workspace where the root is a package _and_
  has nested members — the closest one wins.)
- Otherwise the workspace's `[workspace] default-members` are used. For a
  virtual workspace that expands to every member; for a non-virtual
  workspace it's the root package.

So `cheadergen generate` run from `crates/alpha/` targets only `alpha`,
while the same command run from the workspace root targets every member
(or whichever `default-members` configures).

## Why this matters for `#[cheadergen::config(export)]`

`#[cheadergen::config(export)]` only takes effect when the defining package
is a target. Annotations on a non-target package are silently ignored.

In practice that means:

- If a type defined in a **third-party crate** carries an `export`
  annotation upstream, `cheadergen` won't pick it up; that crate isn't in
  your target set.
- If a type defined in a **workspace sibling** carries `export`, you need
  to make sure that sibling is in the target set (via `--input-dir`, an
  explicit `-p`, or a cwd that resolves to it).
- If `cheadergen` "doesn't see" an annotation you added, the most common
  cause is that the defining package isn't a target in this invocation.

Items that don't rely on `#[cheadergen::config(export)]` for inclusion (`extern "C"` functions, `#[no_mangle]` statics) follow the same rule: they
only become entry points when they live in a target package.

## A worked example

Consider this workspace:

```
my-workspace/
├── Cargo.toml          # [workspace] members = ["alpha", "beta", "core", "cli"]
├── alpha/              # library, has extern "C" functions
├── beta/               # library, has extern "C" functions
├── core/               # library, shared types only — no extern "C"
└── cli/                # binary crate
```

Different invocations produce different target sets:

| Invocation                                                      | Target packages     | Notes                                                                               |
| --------------------------------------------------------------- | ------------------- | ----------------------------------------------------------------------------------- |
| `cheadergen generate …` (run from the workspace root)           | `alpha, beta, core` | `cli` dropped (binary-only, warning).                                               |
| `cheadergen generate …` (run from `alpha/`)                     | `alpha`             | Default. `core`'s types may still appear if `alpha` uses them.                      |
| `cheadergen generate … --input-dir alpha`                       | `alpha`             | Same selection, explicit form; works from any cwd inside the workspace.             |
| `cheadergen generate … -p alpha -p beta`                        | `alpha, beta`       | `core` is _not_ a target; any `#[cheadergen::config(export)]` in `core` is ignored. |
| `cheadergen generate … --exclude cli` (from the workspace root) | `alpha, beta, core` | Same as the default; explicit `--exclude` avoids the binary warning.                |

## See also

- [The processing pipeline](../how-it-works/pipeline.md). What happens after the target
  set is resolved.
- [Bundled vs partitioned output](../what-you-get/partitioning.md). How the target set
  interacts with the choice of output mode.
- [Generate headers for a Cargo workspace](../how-to/workspaces-and-multi-crate.md). Recipes for common workspace shapes.
