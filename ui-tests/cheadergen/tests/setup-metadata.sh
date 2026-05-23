#!/usr/bin/env bash
# Precompute `cargo metadata` for cheadergen test cases ahead of test execution.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

# Build the cheadergen binary so tests can invoke it.
cargo build --bin cheadergen
echo "CARGO_BIN_EXE_cheadergen=$WORKSPACE_ROOT/target/debug/cheadergen" >> "$NEXTEST_ENV"

cargo metadata --all-features --format-version 1 \
    --manifest-path "$SCRIPT_DIR/cheadergen/rust/cases/Cargo.toml" \
    > "$SCRIPT_DIR/cheadergen/rust/cases/metadata.json"

echo "CHEADERGEN_CASES_METADATA=$SCRIPT_DIR/cheadergen/rust/cases/metadata.json" >> "$NEXTEST_ENV"

# Pre-warm the rustdoc JSON cache to avoid cargo target-dir lock contention
# when tests run in parallel. The env var opts into caching workspace-package
# rustdoc — that's off by default because the cache key doesn't yet track
# every input rustdoc output can depend on (e.g. files pulled in via
# `include_str!`). The test suite doesn't exercise those edge cases.
export __CHEADERGEN_CACHE_WORKSPACE_DOCS=1
CHEADERGEN="$WORKSPACE_ROOT/target/debug/cheadergen"
"$CHEADERGEN" cache warm --metadata "$SCRIPT_DIR/cheadergen/rust/cases/metadata.json"
