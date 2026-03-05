#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

cargo metadata --all-features --format-version 1 \
    --manifest-path "$SCRIPT_DIR/rust/cases/Cargo.toml" \
    > "$SCRIPT_DIR/rust/cases/metadata.json"

cargo metadata --all-features --format-version 1 \
    --manifest-path "$SCRIPT_DIR/rust/workspace/Cargo.toml" \
    > "$SCRIPT_DIR/rust/workspace/metadata.json"

echo "CHEADERGEN_CASES_METADATA=$SCRIPT_DIR/rust/cases/metadata.json" >> "$NEXTEST_ENV"
echo "CHEADERGEN_WORKSPACE_METADATA=$SCRIPT_DIR/rust/workspace/metadata.json" >> "$NEXTEST_ENV"
