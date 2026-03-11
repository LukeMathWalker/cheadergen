---
name: ui-tests
description: "Reference for the ui-tests infrastructure: test suites, directory layout, test.toml format, snapshot workflow, and common commands. Use when working with ui-tests cases, debugging test failures, or scaffolding new tests."
---

# UI-Tests Reference

The `ui-tests` crate contains the project's integration test infrastructure. It generates hundreds of test functions from minimal test case definitions, using `build.rs` code generation, insta snapshots, and compilation caching.

## Test Suites

Two test suites, each under `ui-tests/tests/`:

| Suite | Path | License | Notes |
|---|---|---|---|
| **cbindgen** | `tests/cbindgen/rust/cases/` | MPL-2.0 | Vendored from mozilla/cbindgen for compatibility |
| **cheadergen** | `tests/cheadergen/rust/cases/` | Apache-2.0 | New tests for cheadergen-specific features |

## Test Categories

Each case can produce tests in three categories:

- **generate** — Runs cheadergen and snapshots the output header (insta `.snap` files)
- **compile** — Compiles the generated header with a C/C++/Cython compiler
- **symbol** — Snapshots the symbol file output (`--symbol-file --no-header` flag)

## Directory Layout

```
ui-tests/tests/<suite>/rust/cases/<name>/
├── Cargo.toml           # Workspace member package
├── src/lib.rs            # Rust library with extern "C" API
├── test.toml             # Variant status declarations
├── cbindgen.toml         # (cbindgen suite only) Original config
├── cheadergen.toml       # (optional) cheadergen config, passed via --config
└── expectations/
    ├── <name>.c.snap           # C/plain snapshot
    ├── <name>_tag.c.snap       # C/tag snapshot
    ├── <name>.cpp.snap         # C++/plain snapshot
    ├── <name>.c.sym.snap       # Symbol file snapshot
    ├── <name>.c.snap.hash      # Compilation cache hash
    └── <name>.c.snap.hash-cxx  # C++ compat compilation cache hash
```

## test.toml Format

Maps variant keys to statuses. Omitted variants default to normal (must pass).

### Generation Variants

| Key | Language | Style | C++ compat |
|---|---|---|---|
| `"c/plain"` | C | plain | no |
| `"c/tag"` | C | tag | no |
| `"c/both"` | C | both | no |
| `"c/compat"` | C | plain | yes |
| `"c/tag_compat"` | C | tag | yes |
| `"c/both_compat"` | C | both | yes |
| `"cpp/plain"` | C++ | plain | — |
| `"cython/plain"` | Cython | plain | — |
| `"cython/tag"` | Cython | tag | — |

### Symbol Key

`"symbol"` — controls the symbol file generation test.

### Status Values

| Status | Meaning |
|---|---|
| *(omitted)* | Normal — test is generated and must pass |
| `"xfail"` | Expected failure — test runs, it must fail |
| `"skip"` | Ignored — `#[ignore]` attribute, does not run |
| `"exclude"` | No test function generated at all |

### Example

```toml
"c/plain" = "xfail"
"cython/plain" = "exclude"
"cython/tag" = "exclude"
"symbol" = "skip"
```

## Common Commands

```bash
# Fast feedback — generation tests only (no compilation)
just test-generate

# cbindgen suite (generate + compile), skips xfail tests
just test-cbindgen

# Full cheadergen suite, skips xfail tests
just test-cheadergen

# All tests, skips xfail tests
just test

# Runs expected-failure tests
just test-cbindgen-xfail
just test-symbol-xfail

# Filter to specific cases (nextest filter syntax)
just test-generate -E 'test(~alias)'
just test -E 'test(=cbindgen::generate::c::plain::alias)'

# Scaffold a new cheadergen test case
just ui-tests new <name>
```

## Config Files

- **cbindgen cases**: Have `cbindgen.toml` (original). May also have `cheadergen.toml` (translated via `just ui-tests translate-configs`).
- **cheadergen cases**: Use `cheadergen.toml` only.
- If a config file is present in the case directory, it is passed to cheadergen via `--config`.

## Snapshot Workflow

Snapshots use **insta**. When output changes:

1. Tests fail and insta writes `.snap.new` files next to existing `.snap` files
2. Review the `.snap.new` files to verify changes are expected
3. Accept snapshots by moving the `.snap.new` file over the `.snap` file: `mv foo.snap.new foo.snap`
4. Re-run tests to confirm

**IMPORTANT**: Do not use `cargo insta` commands (`cargo insta review`, `cargo insta accept`, etc.). Always move `.snap.new` files directly.

## Compilation Cache

Compile tests use hash-based caching (`.hash` / `.hash-cxx` files next to snapshots). The hash covers: file content, language/style/compat flags, compiler path, compiler flags, and `testing-helpers.h`. Disable with `CHEADERGEN_NO_COMPILE_CACHE=1`.

## Code Generation (build.rs)

`build.rs` discovers test cases, reads `test.toml`, and generates a module tree:

```
mod <suite>::generate::<lang>::<style>::<case_name>
mod <suite>::compile::<lang>::<style>::<case_name>
mod <suite>::symbol::<case_name>
```

Each module contains test functions using macros: `generate_variant!`, `compile_variant!`, `symbol_test!`.
