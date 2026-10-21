#!/usr/bin/env bash
# Real Earth Platform — Benchmark Script (POSIX / Linux / macOS)
set -euo pipefail

echo "⚡ Running Real Earth Performance Benchmarks..."

cargo run -p realearth-cli -- bench
cargo bench -p realearth-benchmarks

echo "✅ Benchmarks completed successfully!"
