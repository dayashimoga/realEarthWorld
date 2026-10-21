#!/usr/bin/env bash
# Real Earth Platform — Setup Script (POSIX / Linux / macOS)
# Idempotent environment and dependency verification
set -euo pipefail

echo "🌍 Real Earth World Platform — Environment Setup"

# 1. Verify Rust toolchain
if ! command -v rustc &> /dev/null; then
    echo "❌ Rust toolchain is missing. Please install Rust from https://rustup.rs"
    exit 1
fi
echo "✅ Detected Rust: $(rustc --version)"

# 2. Check cargo components
echo "🔍 Checking cargo components..."
rustup component add clippy rustfmt

# 3. Create cache and data directories
mkdir -p target/world_bengaluru data/manifests data/schemas assets/bengaluru_sample

echo "🎉 Environment setup completed successfully!"
