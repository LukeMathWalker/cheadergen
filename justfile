# Format all files
# Use `just fmt check` to verify rather than format
fmt action="fmt":
    dprint {{ action }}

# Run linter
lint:
    cargo clippy --workspace --all-targets -- -D warnings

# Run tests
# Use `just test <nextest args>` to pass filters and flags
# For example:
#   just test -E 'test(=cbindgen::generate::c::plain::alias)'
# to run a single test.
test +args="":
    cargo nextest run --no-tests pass {{ args }}

# Run only cbindgen tests
test-cbindgen +args="":
    cargo nextest run -p ui-tests --no-tests pass -E 'test(~cbindgen::)' {{ args }}

# Run only generation tests (no compilation)
test-generate +args="":
    cargo nextest run -p ui-tests --no-tests pass -E 'test(~::generate::)' {{ args }}

# Run xfail cbindgen tests (expected failures)
test-cbindgen-xfail +args="":
    cargo nextest run -p ui-tests --run-ignored ignored-only -E 'test(~cbindgen::)' {{ args }}

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
    just test
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

# Run all checks
verify: lint (fmt "check") test
