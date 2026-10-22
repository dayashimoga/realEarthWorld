#!/usr/bin/env bash
# Real Earth Platform — macOS Application Packaging Script (Bash)
set -euo pipefail

TARGET_DIR="dist/macos"
mkdir -p "$TARGET_DIR"

APP_BUNDLE="$TARGET_DIR/RealEarth.app"
rm -rf "$APP_BUNDLE"
mkdir -p "$APP_BUNDLE/Contents/MacOS" "$APP_BUNDLE/Contents/Resources/assets"

cargo build --workspace --release

cp target/release/realearth-viewer "$APP_BUNDLE/Contents/MacOS/"
cp packaging/macos/Info.plist "$APP_BUNDLE/Contents/"
printf "APPL????" > "$APP_BUNDLE/Contents/PkgInfo"
cp -r assets/* "$APP_BUNDLE/Contents/Resources/assets/"

TAR_PATH="$TARGET_DIR/realearth-macos-universal.tar.gz"
tar -czvf "$TAR_PATH" -C "$TARGET_DIR" RealEarth.app
shasum -a 256 "$TAR_PATH" > "$TAR_PATH.sha256"

echo "[SUCCESS] macOS App Bundle packaged at $TAR_PATH"
