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
    cargo +nightly nextest run -p ui-tests {{ args }}

# Run only cbindgen tests
test-cbindgen +args="":
    cargo +nightly nextest run -p ui-tests -E 'test(~^cbindgen::)' {{ args }}

# Run only generation tests (no compilation)
test-generate +args="":
    cargo +nightly nextest run -p ui-tests -E 'test(~::generate::)' {{ args }}

# Run all checks
verify: lint (fmt "check") test
