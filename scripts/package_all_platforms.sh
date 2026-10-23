#!/usr/bin/env bash
# Real Earth Platform — Master Multi-Platform Packaging Script (Bash)
set -euo pipefail

echo "=========================================================="
echo "   REAL EARTH PLATFORM — FIVE-PLATFORM PACKAGE RELEASE    "
echo "=========================================================="

echo "[1/6] Compiling release binaries..."
cargo build --workspace --release

echo "[2/6] Packaging Linux x86_64..."
bash scripts/package_linux.sh

echo "[3/6] Packaging macOS Universal..."
if [ -f scripts/package_macos.sh ]; then
    bash scripts/package_macos.sh
fi

echo "[4/6] Packaging Android APK/AAB..."
if [ -f scripts/package_android.sh ]; then
    bash scripts/package_android.sh
fi

echo "[5/7] Packaging iOS Simulator..."
if [ -f scripts/package_ios.sh ]; then
    bash scripts/package_ios.sh
fi

echo "[6/7] Packaging Standalone Geographic Regions..."
if [ -f scripts/package_regions.sh ]; then
    bash scripts/package_regions.sh
fi

echo "[7/7] Verifying Release Integrity..."
if [ -f scripts/verify_release_integrity.sh ]; then
    bash scripts/verify_release_integrity.sh
fi

echo "=========================================================="
echo " [SUCCESS] Five-platform packaging complete in dist/      "
echo "=========================================================="
