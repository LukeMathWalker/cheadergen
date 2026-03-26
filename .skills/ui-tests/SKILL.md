---
name: ui-tests
description: "Reference for the ui-tests infrastructure: test suites, directory layout, test.toml format, snapshot workflow, and common commands. Use when working with ui-tests cases, debugging test failures, or scaffolding new tests."
---

# UI-Tests Reference

The ui-tests infrastructure is split into four crates under `ui-tests/`:

| Crate | Path | Purpose |
|-------|------|---------|
| `cbindgen-ui-tests` | `ui-tests/cbindgen/` | cbindgen compatibility test suite |
| `cheadergen-ui-tests` | `ui-tests/cheadergen/` | cheadergen feature test suite |
| `ui-tests` | `ui-tests/cli/` | CLI binary (`just ui-tests new`, `cbindgen-report`, etc.) |
| `ui_tests_toolkit` | `ui-tests/toolkit/` | Shared build-time + runtime infrastructure |

## Cbindgen Normal Expectations Are Read-Only

**NEVER modify plain expectation files (`.c`, `.hpp`, `.pyx`, `.c.sym`) under `ui-tests/cbindgen/`.**
These are vendored golden references from mozilla/cbindgen (MPL-2.0). They define the
target output our codegen must match. Only `test.toml` files may be changed in the
cbindgen suite (to update variant statuses).

**Diff and stderr snapshots ARE mutable.** Files like `*.diff.c.snap` (header_diff output)
and `*.stderr.snap` (generation_fails stderr) under `ui-tests/cbindgen/` are insta snapshots
that evolve as cheadergen improves. Accept these via `mv .snap.new .snap`.

## Test Suites

| Suite | Cases path | License | Notes |
|-------|-----------|---------|-------|
| **cbindgen** | `ui-tests/cbindgen/tests/cbindgen/rust/cases/` | MPL-2.0 | Vendored from mozilla/cbindgen for compatibility |
| **cheadergen** | `ui-tests/cheadergen/tests/cheadergen/rust/cases/` | Apache-2.0 | New tests for cheadergen-specific features |

## Test Categories

Each case can produce tests in three categories:

- **generate** — Runs cheadergen and compares the output header against expectations
- **compile** — Compiles the generated header with a C/C++/Cython compiler
- **symbol** — Compares the symbol file output (`--symbol-file --no-header` flag)

## Directory Layout

```
ui-tests/cbindgen/tests/cbindgen/rust/cases/<name>/
├── Cargo.toml           # Workspace member package
├── src/lib.rs            # Rust library with extern "C" API
├── test.toml             # Variant status declarations
├── cbindgen.toml         # (cbindgen suite only) Original config
├── cheadergen.toml       # (optional) cheadergen config, passed via --config
└── expectations/
    ├── <name>.c               # Normal C/plain expectation (read-only plain file)
    ├── <name>.c.diff.snap     # header_diff: cheadergen's differing output (insta snapshot)
    ├── <name>.c.stderr.snap   # generation_fails: cheadergen's stderr (insta snapshot)
    ├── <name>.c.sym            # Normal symbol expectation (read-only plain file)
    ├── <name>.c.hash           # Compilation cache hash
    └── <name>.c.hash-cxx       # C++ compat compilation cache hash
```

Cheadergen test cases use insta `.snap` files for all expectations:

```
ui-tests/cheadergen/tests/cheadergen/rust/cases/<name>/
├── Cargo.toml
├── src/lib.rs
├── test.toml
└── expectations/
    ├── <name>.c.snap           # C/plain snapshot
    ├── <name>.c.stderr.snap    # (generation_fails only) stderr snapshot
    └── <name>.c.sym.snap       # Symbol file snapshot
```

## Crate-Level Documentation

Every cheadergen test case **must** have a crate-level doc comment (`//!`) at the top of `src/lib.rs` explaining what the test is checking.

## test.toml Format

Maps variant keys to statuses. Omitted variants default to normal (must pass).

### Generation Variants

| Key               | Language | Style | C++ compat |
| ----------------- | -------- | ----- | ---------- |
| `"c/plain"`       | C        | plain | no         |
| `"c/tag"`         | C        | tag   | no         |
| `"c/both"`        | C        | both  | no         |
| `"c/compat"`      | C        | plain | yes        |
| `"c/tag_compat"`  | C        | tag   | yes        |
| `"c/both_compat"` | C        | both  | yes        |
| `"cpp/plain"`     | C++      | plain | —          |
| `"cython/plain"`  | Cython   | plain | —          |
| `"cython/tag"`    | Cython   | tag   | —          |

### Symbol Key

`"symbol"` — controls the symbol file generation test.

### Status Values

| Status              | Meaning |
| ------------------- | ------- |
| _(omitted)_         | Normal — test must pass, output matches expectation |
| `"header_diff"`     | cheadergen succeeds but output differs from cbindgen expectation; differing output is snapshotted |
| `"generation_fails"`| cheadergen fails (non-zero exit); stderr is snapshotted |
| `"skip"`            | Ignored — `#[ignore]` attribute, does not run |
| `"exclude"`         | No test function generated at all |

### Example

```toml
"c/plain" = "header_diff"
"cpp/plain" = "generation_fails"
"cython/plain" = "exclude"
"cython/tag" = "exclude"
```

## Common Commands

```bash
# Fast feedback — generation tests only (no compilation)
just test-generate

# cbindgen suite (generate + compile)
just test-cbindgen

# cheadergen suite
just test-cheadergen

# All tests
just test

# Filter to specific cases (nextest filter syntax)
just test-generate -E 'test(~alias)'
just test -E 'test(=cbindgen::generate::c::plain::alias)'

# Scaffold a new cheadergen test case
just ui-tests new <name>

# Write structured JUnit XML results (alongside normal output)
just test-generate --profile machine
# Then read target/nextest/machine/junit.xml for per-test results
```

## Config Files

- **cbindgen cases**: Have `cbindgen.toml` (original). May also have `cheadergen.toml` (translated via `just ui-tests translate-configs`).
- **cheadergen cases**: Use `cheadergen.toml` only.
- If a config file is present in the case directory, it is passed to cheadergen via `--config`.

## Snapshot Workflow

When output changes, tests fail and insta writes `.snap.new` files next to existing `.snap` files:

1. Review the `.snap.new` files to verify changes are expected
2. Accept snapshots by moving the `.snap.new` file over the `.snap` file: `mv foo.snap.new foo.snap`
3. Re-run tests to confirm

**IMPORTANT**: Do not use `cargo insta` commands (`cargo insta review`, `cargo insta accept`, etc.). Always move `.snap.new` files directly.

**Cbindgen normal expectations are read-only plain files** — never modify `.c`, `.hpp`, `.pyx`, or `.c.sym` files under `ui-tests/cbindgen/`. If cheadergen output now matches a cbindgen expectation, the test will tell you to remove the `header_diff` marker from `test.toml`.

**Cbindgen diff/stderr snapshots are mutable** — `.diff.*.snap` and `.stderr.snap` files under `ui-tests/cbindgen/` are expected to change as cheadergen improves. Accept these normally.

## Compilation Cache

Compile tests use hash-based caching (`.hash` / `.hash-cxx` files next to expectations). The hash covers: file content, language/style/compat flags, compiler path, compiler flags, and `testing-helpers.h`. Disable with `CHEADERGEN_NO_COMPILE_CACHE=1`.

## Code Generation (build.rs)

Each test crate has its own `build.rs` that discovers test cases, reads `test.toml`, and generates a module tree:

```
mod <suite>::generate::<lang>::<style>::<case_name>
mod <suite>::compile::<lang>::<style>::<case_name>
mod <suite>::symbol::<case_name>
```

Each module contains test functions using macros: `generate_variant!`, `compile_variant!`, `symbol_test!`.
