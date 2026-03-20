#!/usr/bin/env bash
set -euo pipefail
cargo build --bin cheadergen
WORKSPACE_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
echo "CHEADERGEN_BIN=$WORKSPACE_ROOT/target/debug/cheadergen" >> "$NEXTEST_ENV"
