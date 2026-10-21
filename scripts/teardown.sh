#!/usr/bin/env bash
# Real Earth Platform — Container Teardown Script (POSIX / Linux / macOS)
set -euo pipefail

echo "🛑 Tearing down Real Earth containerized environment..."

COMPOSE_CMD=""
if command -v podman-compose &> /dev/null; then
    COMPOSE_CMD="podman-compose"
elif command -v docker &> /dev/null && docker compose version &> /dev/null; then
    COMPOSE_CMD="docker compose"
elif command -v podman &> /dev/null; then
    COMPOSE_CMD="podman compose"
fi

if [ -z "$COMPOSE_CMD" ]; then
    echo "⚠️ No container compose runtime found. Nothing to tear down."
    exit 0
fi

echo "🛑 Executing: $COMPOSE_CMD down -v"
$COMPOSE_CMD down -v

echo "✅ Containers torn down successfully!"
