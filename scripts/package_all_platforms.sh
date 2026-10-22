#!/usr/bin/env bash
# Real Earth Platform — Master Multi-Platform Packaging Script (Bash)
set -euo pipefail

echo "=========================================================="
echo "   REAL EARTH PLATFORM — FIVE-PLATFORM PACKAGE RELEASE    "
echo "=========================================================="

echo "[1/5] Compiling release binaries..."
cargo build --workspace --release

echo "[2/5] Packaging Linux x86_64..."
bash scripts/package_linux.sh

echo "[3/5] Packaging macOS Universal..."
if [ -f scripts/package_macos.sh ]; then
    bash scripts/package_macos.sh
fi

echo "[4/5] Packaging Android APK/AAB..."
if [ -f scripts/package_android.sh ]; then
    bash scripts/package_android.sh
fi

echo "[5/5] Packaging iOS IPA..."
if [ -f scripts/package_ios.sh ]; then
    bash scripts/package_ios.sh
fi

echo "=========================================================="
echo " [SUCCESS] Five-platform packaging complete in dist/      "
echo "=========================================================="
