# Contributing

## Prerequisites

The following tools must be installed to work on this repository:

- [Rust](https://rustup.rs/) (stable + nightly)
  - Stable is used to compile the project.
  - Nightly is required for `rustdoc` invocations.
- [cargo-nextest](https://nexte.st/), test runner (`cargo install cargo-nextest`).
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov), for coverage (`cargo install cargo-llvm-cov` + `rustup +nightly component add llvm-tools-preview`).
- [dprint](https://dprint.dev/), code formatter.
- [just](https://just.systems/), command runner.
- [mdbook](https://rust-lang.github.io/mdBook/) (`cargo install mdbook`) — only needed when editing the user guide under `docs/`. Build with `just book` or live-preview with `just book-serve`.
- A C/C++ compiler (e.g. `gcc`/`g++` or `clang`/`clang++`) — needed to compile generated headers in tests.
- [Cython](https://cython.org/) — needed for Cython header generation tests. Install via `uv tool install cython` or `pip install cython`.
