# Bundled vs partitioned output

`cheadergen` has two output modes: **bundled** and **partitioned**

**Bundled** emits a self-contained header for a single target crate, with
every reachable dependency type inlined. It can't be used if you're trying
to generate C headers for _multiple_ Rust crates at once.

**Partitioned** (the default) generates a C header for each _defining_ crate,
with cross-crate references wired up automatically via `#include` directives.
Partitioned supports generating C headers for multiple Rust crates at once.

This page explains the model, the trade-offs, and the knobs you can turn.

## The two modes at a glance

|                  | Bundled                                                   | Partitioned                           |
| ---------------- | --------------------------------------------------------- | ------------------------------------- |
| CLI flag         | `--bundle`                                                | (default)                             |
| Targets per run  | Exactly one                                               | One or more                           |
| Files produced   | A single combined header                                  | One header per _defining_ crate       |
| Dependency types | Inlined into the output                                   | Defined once, included from consumers |
| Best for         | Single-crate libraries that ship a single header artifact | Workspaces with shared FFI types      |

## Bundled mode

Bundled mode is a single-target mode. Pass `--bundle` (or set
`bundle = true` in `cheadergen.toml`) with exactly one target package and
`cheadergen` will produce one self-contained header. Every type the target
references (whether it's defined in the target itself, in a workspace
member, or in a crates.io dependency) is inlined into that single output
file.

If you select more than one target package with `--bundle`, `cheadergen`
errors out: two bundles built from overlapping dependency graphs would
duplicate the shared types in a way C can't tolerate, and the tool refuses
to produce colliding output silently.

This is the right choice when:

- You ship exactly one C-facing library.
- You want a single artifact your downstream consumers can drop into their
  build with no questions.
- You're migrating from `cbindgen` and want output shape parity (`cbindgen`'s
  defaults match this).

## Partitioned mode

Without `--bundle`, `cheadergen` treats types as belonging to their **defining
crate** and produces one header per crate that contributes types. Cross-crate
references are wired up automatically:

- If target crate `A` uses `BType` by value, `A.h` gets `#include "b.h"` and
  `B.h` gets the full definition of `BType` (and any other types `A` actually
  uses from `B`).
- Crates that contribute _only_ types (no `extern "C"` of their own) get a
  "types-only" header, with no function declarations.
- Types from crates the target doesn't actually use don't appear at all.
  Partitioning is demand-driven.

### Include vs forward-declare

The decision is **per dependency crate**, not per type:

- If _any_ type from crate `B` is used **by-value** in `A.h`, `A.h` emits
  `#include "b.h"`.
- If _every_ `B` type used in `A.h` is referenced only **behind a pointer**,
  `A.h` emits a forward declaration (`typedef struct BType BType;`) and no
  `#include`.

This rule keeps the include graph shallow: `cheadergen` never both includes and
forward-declares from the same dependency.

### Opaque packages stay inline

When

```toml
[package.<name>] 
types = "opaque"`
```

is used, the package's types are
forward-declared in each consuming header. No header file is
generated for the opaque package.

### Skipped packages don't generate anything

When

```toml
[package.<name>] 
types = "skip"`
```

is used, `cheadergen` emits no header for the
package and assumes you'll supply your own (typically via `includes` in
`cheadergen.toml`). Use this for packages whose C headers come from somewhere
outside `cheadergen`'s control.

## Generic types in partitioned mode

`Wrapper<i32>` doesn't belong to the crate that _defines_ `Wrapper<T>`, it
belongs to whichever crate _consumes_ the instantiation. This rule avoids
synthetic cross-crate include dependencies and circular includes. See
[Generics and monomorphization](./generics-and-monomorphization.md) for the
full treatment.

## The partitioned-only CLI flags

These flags only apply when generating partitioned output (`cheadergen` rejects
them in combination with `--bundle`):

### `--prune-orphans`

After generation, delete any `*.h` / `*.hpp` files in `--output-dir` that
`cheadergen` didn't write in this run. Useful in CI to catch stale headers from
removed types or packages. Without `--prune-orphans`, those files just sit
there.

### `--skip-empty`

Skip writing headers that would otherwise contain no declarations (no types,
constants, statics, or functions). Combined with `--prune-orphans` this
ensures empty-by-deletion headers are cleaned up rather than rewritten as
empty stubs.

## Choosing between modes

Start with partitioned (the default). Switch to bundled if:

- You're packaging a single shared library and the downstream contract is "one
  header, one `.so`".
- The crate has no workspace siblings that would also need their own headers.
- You're migrating an existing `cbindgen` pipeline and want a small diff against
  the original output.

Stay on partitioned when:

- You have multiple FFI-facing crates that share types and would otherwise
  duplicate them.
- You want changes to a shared type to flow through one canonical header
  rather than appearing in N independent copies.
- You're building a workspace-wide SDK where consumers may pick and choose
  which headers they need.
