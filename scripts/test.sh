#!/usr/bin/env bash
# Real Earth Platform — Test Script (POSIX / Linux / macOS)
set -euo pipefail

echo "🧪 Running Real Earth Quality Gate Verification..."

echo "1. Code Formatting (cargo fmt --check)..."
cargo fmt --check

echo "2. Static Analysis (cargo clippy -D warnings)..."
cargo clippy --workspace --all-targets -- -D warnings

echo "3. Full Workspace Test Suite (cargo test)..."
cargo test --workspace

echo "4. Integration End-to-End Suite..."
cargo test -p realearth-tests

echo "5. Benchmark Smoke Suite (cargo bench -- --test)..."
cargo bench --bench geospatial_bench --bench compiler_bench -- --test

echo "6. Security Audit (cargo audit)..."
cargo audit

echo "🎉 ALL QUALITY GATES PASSED (100% test success, 0 warnings, 0 fmt errors, 0 security vulnerabilities)!"
