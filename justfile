set positional-arguments

# Nightly toolchain used for `cargo rustdoc` JSON generation and for building
# UI test workspaces (which use unstable features like `sync_unsafe_cell`).
# Single source of truth is `rust-docs-toolchain` at the repo root, also
# read by `cheadergen_cli::metadata::DOCS_TOOLCHAIN`. Override locally with
# `CHEADERGEN_DOCS_TOOLCHAIN=...` if needed.
docs_toolchain := env_var_or_default("CHEADERGEN_DOCS_TOOLCHAIN", trim(shell("cat 'cheadergen_cli/rust-docs-toolchain'")))

# Format all files
# Use `just fmt check` to verify rather than format
fmt action="fmt":
    dprint {{ action }}

# Run linter
lint:
    cargo clippy --workspace --all-targets -- -D warnings

# Build docs and fail on warnings
doc:
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --document-private-items --lib

# Build the user guide (mdbook) into docs/book
book:
    mdbook build docs

# Serve the user guide locally with live reload
book-serve:
    mdbook serve docs --open

# Check that version-pinned URLs in docs/src/ match the workspace package version.
# Run automatically on release PRs; can be invoked locally before bumping a release.
docs-check-version *args:
    bash scripts/check_docs_version.sh {{ args }}

# Run tests
# Use `just test <nextest args>` to pass filters and flags
# For example:
#   just test -E 'test(=cbindgen::generate::c::plain::alias)'
# to run a single test.
test +args="":
    cargo nextest run --no-fail-fast --no-tests pass "$@"

# Run only cbindgen tests
test-cbindgen +args="":
    cargo nextest run --no-fail-fast -p cbindgen-ui-tests --no-tests pass "$@"

# Run only cheadergen tests
test-cheadergen +args="":
    cargo nextest run --no-fail-fast -p cheadergen-ui-tests --no-tests pass "$@"

# Run only generation tests (no compilation)
test-generate +args="":
    cargo nextest run --no-fail-fast -p cbindgen-ui-tests -p cheadergen-ui-tests --no-tests pass -E 'test(~::generate::)' "$@"

# Run only compilation tests (no generation)
test-compile +args="":
    cargo nextest run --no-fail-fast -p cbindgen-ui-tests -p cheadergen-ui-tests --no-tests pass -E 'test(~::compile::)' "$@"

# Run only symbol tests
test-symbol +args="":
    cargo nextest run --no-fail-fast -p cbindgen-ui-tests -p cheadergen-ui-tests --no-tests pass -E 'test(~::symbol::)' "$@"


# Compute project coverage by running tests with instrumentation enabled
# Report formats: html (default), codecov, lcov, text
#   just coverage          → HTML report in target/llvm-cov/html/
#   just coverage codecov  → codecov.json file (for Codecov upload)
#   just coverage lcov     → lcov.info file
#   just coverage text     → summary printed to stdout
coverage format="html":
    #!/usr/bin/env bash
    set -euo pipefail
    source <(cargo llvm-cov show-env --sh --no-cfg-coverage)
    cargo llvm-cov clean --workspace
    just test -E 'not test(~::compile::)'
    report_args=()
    case "{{ format }}" in
        html)    report_args+=(--html) ;;
        codecov) report_args+=(--codecov --output-path codecov.json) ;;
        lcov)    report_args+=(--lcov --output-path lcov.info) ;;
        text)    ;;
        *)       echo "Unknown format '{{ format }}'. Use: html, codecov, lcov, text" >&2; exit 1 ;;
    esac
    cargo llvm-cov report "${report_args[@]}"
    [[ "{{ format }}" == html ]] && echo "Report: target/llvm-cov/html/index.html" || true

# Show uncovered lines per file in compact format
# Runs coverage instrumentation, then parses lcov.info into collapsed line ranges
# Use `just uncovered <pattern>` to filter to files matching pattern, e.g. `just uncovered src/emit`
uncovered pattern="":
    #!/usr/bin/env bash
    set -euo pipefail
    just coverage lcov
    repo_root="$(pwd)/"
    awk -v pattern="{{ pattern }}" -v root="$repo_root" '
    /^SF:/ {
        file = substr($0, 4)
        if (index(file, root) == 1) file = substr(file, length(root) + 1)
        skip = (index(file, "ui-tests/") == 1)
        if (!skip && pattern != "") skip = (index(file, pattern) == 0)
        delete lines
        n = 0
    }
    /^DA:/ && !skip {
        split(substr($0, 4), a, ",")
        if (a[2] == 0) { lines[n++] = a[1] }
    }
    /^end_of_record/ && !skip && n > 0 {
        printf "%s: ", file
        start = lines[0]; end = lines[0]
        for (i = 1; i < n; i++) {
            if (lines[i] == end + 1) { end = lines[i] }
            else {
                printf "%s", (start == end) ? start : start "-" end
                printf ", "
                start = lines[i]; end = lines[i]
            }
        }
        printf "%s\n", (start == end) ? start : start "-" end
    }
    ' lcov.info

# Print cbindgen compatibility report for a variant (e.g. c/plain)
cbindgen-report +args:
    cargo run -p ui-tests -- cbindgen-report "$@"

# Run ui-tests commands (e.g. `just ui-tests new <name>`)
ui-tests +args:
    cargo run -p ui-tests -- "$@"

# Translate all cbindgen.toml configs to cheadergen.toml
ui-tests-translate-configs:
    cargo run -p ui-tests -- translate-configs

# Build the UI test workspaces with `-D warnings` to ensure they stay warning-free.
# Also runs `cargo doc` since cheadergen's tests rely on rustdoc output and
# any rustdoc warning would leak into snapshotted test stderr.
# Requires a nightly toolchain because some fixtures use unstable features.
# We `cd` into each workspace so cargo picks up its `.cargo/config.toml`
# (which suppresses the future-incompat report for known fixtures).
lint-ui-tests:
    cd ui-tests/cbindgen/tests/cbindgen/rust/cases && RUSTFLAGS="-D warnings" cargo +{{ docs_toolchain }} build --workspace
    cd ui-tests/cbindgen/tests/cbindgen/rust/cases && RUSTDOCFLAGS="-D warnings" cargo +{{ docs_toolchain }} doc --workspace --no-deps
    cd ui-tests/cheadergen/tests/cheadergen/rust/cases && RUSTFLAGS="-D warnings" cargo +{{ docs_toolchain }} build --workspace
    cd ui-tests/cheadergen/tests/cheadergen/rust/cases && RUSTDOCFLAGS="-D warnings" cargo +{{ docs_toolchain }} doc --workspace --no-deps

# Run all checks
verify: lint doc (fmt "check") test lint-ui-tests
