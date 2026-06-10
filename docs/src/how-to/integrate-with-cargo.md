# Integrate with Cargo

This guide shows how to make `cheadergen` part of your normal Cargo workflow,
so generated headers stay in sync with your Rust source.

## Run `cheadergen` outside `cargo build`

`cheadergen` must be invoked as a standalone CLI command, not from a Cargo
`build.rs` script.

The reason is a Cargo deadlock: a `build.rs` runs while Cargo holds an
exclusive lock on the target directory for the in-flight build. `cheadergen`
internally shells out to `cargo +<nightly> rustdoc -- --output-format=json`,
which tries to acquire the same lock — and waits forever for the build that
spawned it to finish. There's no way around this from inside `build.rs`.

The right place for header generation is therefore one level up, in a
**command runner** (`just`, `make`, `nix`, your CI workflow, etc.) that
sequences `cheadergen` and `cargo build` as separate steps.

A minimal `justfile`:

```just
# Regenerate C headers
headers:
    cheadergen generate --output-dir include --prune-orphans

# Build the crate, refreshing headers first
build: headers
    cargo build --release
```

A minimal `Makefile`:

```makefile
.PHONY: headers build

headers:
	cheadergen generate --output-dir include --prune-orphans

build: headers
	cargo build --release
```

Either way: `cheadergen` finishes (and releases the target-dir lock) before
`cargo build` starts.

## Idempotent output: mtimes won't churn

`cheadergen` calls
[`write_header_if_changed`](https://github.com/LukeMathWalker/cheadergen/blob/main/cheadergen_cli/src/cli/generate.rs)
for every output file: if the new content is byte-identical to what's already
on disk, the file is left alone and its mtime is preserved.

Build systems that watch generated headers by mtime (Make, CMake, downstream
`cc` crates) won't see spurious rebuilds just because `cheadergen` ran with no
real change.

## Picking an output directory

There's no universal convention; pick what works for your project.

It's common to use a directory in the crate root (e.g. `/include`), committed alongside the Rust source.
Easy for downstream C consumers: they `-I path/to/crate/include`.

If you're using [partitioned mode](../what-you-get/partitioning.md), pass
`--prune-orphans` to keep the output directory clean as types are added and
removed.
