#!/usr/bin/env bash
# Precompute `cargo metadata` for cbindgen test cases ahead of test execution.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

# Build the cheadergen binary so tests can invoke it.
cargo build --bin cheadergen
echo "CARGO_BIN_EXE_cheadergen=$WORKSPACE_ROOT/target/debug/cheadergen" >> "$NEXTEST_ENV"

cargo metadata --all-features --format-version 1 \
    --manifest-path "$SCRIPT_DIR/cbindgen/rust/cases/Cargo.toml" \
    > "$SCRIPT_DIR/cbindgen/rust/cases/metadata.json"

cargo metadata --all-features --format-version 1 \
    --manifest-path "$SCRIPT_DIR/cbindgen/rust/workspace/Cargo.toml" \
    > "$SCRIPT_DIR/cbindgen/rust/workspace/metadata.json"

echo "CBINDGEN_CASES_METADATA=$SCRIPT_DIR/cbindgen/rust/cases/metadata.json" >> "$NEXTEST_ENV"
echo "CBINDGEN_WORKSPACE_METADATA=$SCRIPT_DIR/cbindgen/rust/workspace/metadata.json" >> "$NEXTEST_ENV"

# Pre-warm the rustdoc JSON cache to avoid cargo target-dir lock contention
# when tests run in parallel.
CHEADERGEN="$WORKSPACE_ROOT/target/debug/cheadergen"
"$CHEADERGEN" warm-cache --metadata "$SCRIPT_DIR/cbindgen/rust/cases/metadata.json"
"$CHEADERGEN" warm-cache --metadata "$SCRIPT_DIR/cbindgen/rust/workspace/metadata.json"
