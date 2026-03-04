# Format all files
# Use `just fmt check` to verify rather than format
fmt action="fmt":
    dprint {{ action }}

# Run linter
lint:
    cargo clippy --all-targets -- -D warnings

# Run all checks
verify: lint (fmt "check")
