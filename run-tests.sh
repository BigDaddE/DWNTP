#!/bin/bash
set -e

# Ensure we are executing from the root of the project
cd "$(dirname "$0")"

echo "=========================================="
echo "      Running DWNTP Test Suite            "
echo "=========================================="

echo ""
echo "[1/3] Checking code formatting (cargo fmt)..."
cargo fmt -- --check

echo ""
echo "[2/3] Running linter (cargo clippy)..."
cargo clippy

echo ""
echo "[3/3] Executing unit and integration tests (cargo test)..."
cargo test

echo ""
echo "=========================================="
echo "      All tests passed successfully!      "
echo "=========================================="
