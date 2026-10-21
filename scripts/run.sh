#!/usr/bin/env bash
# Real Earth Platform — Run Script (POSIX / Linux / macOS)
set -euo pipefail

echo "🌍 Launching Real Earth World Platform..."

if [ ! -f "target/world_bengaluru/chunks.db" ]; then
    echo "📦 Compiling Bengaluru Central world package..."
    cargo run -p realearth-cli -- compile --output target/world_bengaluru
fi

echo "🚀 Launching Real Earth World Viewer..."
cargo run -p realearth-viewer
