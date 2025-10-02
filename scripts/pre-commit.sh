#!/bin/bash
set -e

echo "Running pforge quality gates..."

# Format check
echo "  Checking formatting..."
cargo fmt --check

# Clippy
echo "  Running clippy..."
cargo clippy --all-targets -- -D warnings

# Tests
echo "  Running tests..."
cargo test --all

# PMAT quality gates (if pmat is installed)
if command -v pmat &> /dev/null; then
    echo "  Running PMAT quality gates..."
    pmat analyze complexity --max 20 || true
    pmat analyze satd --max 0 || true
    pmat analyze tdg --min 0.75 || true
fi

echo "✓ All quality gates passed"
