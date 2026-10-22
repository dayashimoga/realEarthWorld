#!/usr/bin/env bash
# Real Earth Platform — Linux x86_64 Application Packaging Script (Bash)
set -euo pipefail

TARGET_DIR="dist/linux"
mkdir -p "$TARGET_DIR"

APPDIR="$TARGET_DIR/RealEarth.AppDir"
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin" "$APPDIR/usr/lib" "$APPDIR/usr/share/applications" "$APPDIR/usr/share/realearth/assets"

cargo build --workspace --release

cp target/release/realearth-viewer "$APPDIR/usr/bin/"
cp target/release/realearth-cli "$APPDIR/usr/bin/"
cp packaging/linux/AppRun "$APPDIR/AppRun"
chmod +x "$APPDIR/AppRun"
cp packaging/linux/realearth.desktop "$APPDIR/realearth.desktop"
cp packaging/linux/realearth.desktop "$APPDIR/usr/share/applications/"
cp -r assets/* "$APPDIR/usr/share/realearth/assets/"

TAR_PATH="$TARGET_DIR/realearth-linux-x64.tar.gz"
tar -czvf "$TAR_PATH" -C "$TARGET_DIR" RealEarth.AppDir
shasum -a 256 "$TAR_PATH" > "$TAR_PATH.sha256"

echo "[SUCCESS] Linux package generated at $TAR_PATH"
