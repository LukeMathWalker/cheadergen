# What this project is

`cheadergen` generates C/C++ header files from Rust libraries that expose a `pub extern "C"` API.

Key documents:

- [Reflection Engine](design/reflection_engine.md) covers how `cheadergen` leverages `rustdoc-json` for reflection,
  including its advantages/disavantages with respect to the approach used in `cbindegen`.
- [Processing Pipeline](design/processing_pipeline.md) describe the different processing stages within `cheadergen`,
  going from CLI invocation to generated C/C++ header file.

## Code Conventions

- Rust Edition 2024.
- The project must compile on the latest `stable`. Only the `rustdoc` invocation requires `nightly`.

## Version Control System

Use Jujutsu (`jj`), if the repository is configured for it. Fallback to `git` otherwise.

## Common Commands

### All checks

```bash
just verify
```

### Formatting

```bash
# Format all files in the repository
just fmt
# Check if files are formatted
just fmt check
```

### Linting

```bash
just lint
```

### Testing

```bash
# Run all tests
just test
# Run only cbindgen tests (generate + compile)
just test-cbindgen
# Run only cheadergen tests
just test-cheadergen
# Run only generation tests, no compilation (faster feedback loop)
just test-generate
```

The test suites live in two separate crates:

- **`ui-tests/cbindgen/`** — cbindgen compatibility suite. Normal expectations are plain files
  (read-only, vendored from cbindgen). Tests marked `header_diff` capture cheadergen's
  differing output as insta snapshots. Tests marked `generation_fails` assert that
  cheadergen fails and snapshot stderr.
- **`ui-tests/cheadergen/`** — cheadergen test suite. Expectations are insta `.snap` files
  that can be updated as cheadergen evolves.

Both suites generate C/C++ headers from mini-crates under `tests/*/rust/cases/`
and compare the output against expected headers in each case's `expectations/` directory.

Use `just test-generate` when iterating on header generation logic — it skips compilation
and runs significantly faster. Use `just test-cbindgen` or `just test-cheadergen` to scope
down to a single suite. All commands accept extra nextest args,
e.g. `just test -E 'test(~alias)'`.

#### Structured test output

Pass `--profile machine` to any test command to write JUnit XML alongside normal output:

    just test-generate --profile machine

Then read `target/nextest/machine/junit.xml` for structured results. The XML contains one `<testcase>` per test with:

- **name** and **classname** — the test identity
- **time** — execution duration in seconds
- **`<failure>`** — present only for failed tests, contains the failure message and captured output

Example: a passing test:

    <testcase name="cbindgen::generate::c::plain::alias" classname="cbindgen-ui-tests::tests" time="0.200"/>

Example: a failing test:

    <testcase name="cbindgen::generate::c::plain::alias" classname="cbindgen-ui-tests::tests" time="0.200">
      <failure message="assertion failed" type="test">
        thread 'cbindgen::generate::c::plain::alias' panicked at ...
        Output mismatch: ...
      </failure>
    </testcase>

### Scaffolding new test cases

```bash
just ui-tests new <name>
```

Creates a new cheadergen test case under `ui-tests/cheadergen/tests/cheadergen/rust/cases/<name>/`
with a starter `Cargo.toml`, `src/lib.rs`, and `test.toml`.

## Licensing

cheadergen's own code is licensed under APACHE-2.0.

The `ui-tests/cbindgen/` directory contains test cases and expectations vendored from
[mozilla/cbindgen](https://github.com/mozilla/cbindgen), which is licensed under MPL-2.0.
Those files retain their original license. See `LICENSE-MPL-2.0` in the repo root.
