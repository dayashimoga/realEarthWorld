#!/usr/bin/env bash
# Real Earth Platform — Container Bring-up Script (POSIX / Linux / macOS)
set -euo pipefail

echo "🐳 Starting Real Earth containerized environment..."

COMPOSE_CMD=""
if command -v podman-compose &> /dev/null; then
    COMPOSE_CMD="podman-compose"
elif command -v docker &> /dev/null && docker compose version &> /dev/null; then
    COMPOSE_CMD="docker compose"
elif command -v podman &> /dev/null; then
    COMPOSE_CMD="podman compose"
fi

if [ -z "$COMPOSE_CMD" ]; then
    echo "⚠️ Neither Podman Compose nor Docker Compose found on PATH."
    echo "ℹ️ Skipping container orchestration. Native headless builds remain operational via scripts/build.sh."
    exit 0
fi

echo "🚀 Using compose runtime: $COMPOSE_CMD"
$COMPOSE_CMD up --build -d

echo "✅ Containers brought up successfully!"
