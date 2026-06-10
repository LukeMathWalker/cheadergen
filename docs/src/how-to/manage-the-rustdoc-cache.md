# Manage the rustdoc cache

`cheadergen` invokes `cargo +<nightly> doc --output-format=json` under
the hood, which is the slowest step of the whole pipeline. To keep iteration
fast, `cheadergen` keeps the resulting JSON in a per-crate on-disk cache and
reuses it on subsequent runs.

You normally don't have to touch the cach, it builds up incrementally as
you run `cheadergen generate`. This page documents the `cheadergen cache`
subcommands for the cases where you do.

## Inspect the cache location

```bash
cheadergen cache show dir
```

Prints the absolute path to the cache directory and exits. The location is
platform-specific.

Use this in CI configs to feed the cache directory into your runner's cache
action.

## Pre-warm the cache

```bash
cheadergen cache warm                    # all workspace members in cwd
cheadergen cache warm path/to/Cargo.toml # one crate
cheadergen cache warm -p alpha -p beta   # specific workspace members
```

`cache warm` issues a single batched `cargo doc -p crate1 -p crate2 ...`
invocation, maximising parallelism within `cargo`.

## Clear the cache

```bash
cheadergen cache clear
```

Removes the cache directory and exits.

In practice you rarely need this. Reasons you might:

- The cache directory has grown large enough to matter on a developer
  machine and you want a fresh start.
- You're chasing a reproducibility bug and want to confirm the issue
  isn't a stale cache entry.

## See also

- [The processing pipeline](../how-it-works/pipeline.md) for where the cache
  fits into a `generate` invocation.
- [Integrate with Cargo](./integrate-with-cargo.md) for the broader
  workflow `cheadergen` slots into.
