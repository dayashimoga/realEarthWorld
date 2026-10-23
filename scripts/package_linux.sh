#!/usr/bin/env bash
# Real Earth Platform — Linux x86_64 Application Packaging Script (Bash)
set -euo pipefail

TARGET_DIR="dist/linux"
mkdir -p "$TARGET_DIR"

APPDIR="$TARGET_DIR/RealEarth.AppDir"
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin" "$APPDIR/usr/lib" "$APPDIR/usr/share/applications" \
         "$APPDIR/usr/share/realearth/assets" "$APPDIR/usr/share/realearth/config" \
         "$APPDIR/usr/share/realearth/data/world_san_francisco"

# Stage binaries if compiled
if [ -f "target/release/realearth-viewer" ]; then
    cp target/release/realearth-viewer "$APPDIR/usr/bin/"
    cp target/release/realearth-cli "$APPDIR/usr/bin/"
    chmod +x "$APPDIR/usr/bin/realearth-viewer" "$APPDIR/usr/bin/realearth-cli"
fi

cp packaging/linux/AppRun "$APPDIR/AppRun"
chmod +x "$APPDIR/AppRun"
cp packaging/linux/realearth.desktop "$APPDIR/realearth.desktop"
cp packaging/linux/realearth.desktop "$APPDIR/usr/share/applications/"
[ -f "config/engine_config.json" ] && cp config/engine_config.json "$APPDIR/usr/share/realearth/config/"
[ -d "assets" ] && cp -r assets/* "$APPDIR/usr/share/realearth/assets/"
[ -d "data/world_san_francisco" ] && cp -r data/world_san_francisco/* "$APPDIR/usr/share/realearth/data/world_san_francisco/"

TAR_PATH="$TARGET_DIR/realearth-linux-x64.tar.gz"
tar -czf "$TAR_PATH" -C "$TARGET_DIR" RealEarth.AppDir
sha256sum "$TAR_PATH" | awk '{print $1}' > "$TAR_PATH.sha256"

echo "[SUCCESS] Linux portable AppDir package generated at $TAR_PATH"

if command -v appimagetool >/dev/null 2>&1; then
    APPIMAGE_PATH="$TARGET_DIR/RealEarth-x86_64.AppImage"
    appimagetool "$APPDIR" "$APPIMAGE_PATH"
    sha256sum "$APPIMAGE_PATH" | awk '{print $1}' > "$APPIMAGE_PATH.sha256"
    echo "[SUCCESS] Real Linux AppImage generated at $APPIMAGE_PATH"
else
    echo "[NOTE] appimagetool not found. Genuine AppDir packaged into $TAR_PATH."
fi
