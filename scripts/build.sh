#!/usr/bin/env bash
# Real Earth Platform — Build Script (POSIX / Linux / macOS)
set -euo pipefail

echo "🔨 Building Real Earth World Platform (Release Profile)..."
cargo build --workspace --release

echo "✅ Workspace build completed successfully!"
