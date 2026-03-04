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

## Licensing

cheadergen's own code is licensed under APACHE-2.0.

The `tests/rust/` and `tests/expectations/` directories contain files vendored from
[mozilla/cbindgen](https://github.com/mozilla/cbindgen), which is licensed under MPL-2.0.
Those files retain their original license. See `LICENSE-MPL-2.0` in the repo root.
