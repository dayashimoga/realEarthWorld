#!/usr/bin/env bash
# Real Earth Platform — Cleanup Script (POSIX / Linux / macOS)
set -euo pipefail

echo "🧹 Cleaning Real Earth build artifacts and cached worlds..."

cargo clean
rm -rf target/world_bengaluru

echo "✅ Workspace cleaned successfully!"
