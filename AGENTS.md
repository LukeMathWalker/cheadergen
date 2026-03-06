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
# Run only generation tests, no compilation (faster feedback loop)
just test-generate
```

Tests require the `nightly` toolchain (`rustup toolchain install nightly`).
The test suite uses `cbindgen` to generate C/C++ headers from mini-crates under `tests/rust/cases/`
and compares the output against expected headers in `tests/expectations/`.

Use `just test-generate` when iterating on header generation logic — it skips compilation
and runs significantly faster. Use `just test-cbindgen` to scope down to the cbindgen suite
without running cheadergen tests. All three commands accept extra nextest args,
e.g. `just test -- -E 'test(~alias)'`.

## Licensing

cheadergen's own code is licensed under APACHE-2.0.

The `tests/rust/` and `tests/expectations/` directories contain files vendored from
[mozilla/cbindgen](https://github.com/mozilla/cbindgen), which is licensed under MPL-2.0.
Those files retain their original license. See `LICENSE-MPL-2.0` in the repo root.
