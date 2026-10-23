#!/usr/bin/env bash
# Real Earth Platform — macOS Application Packaging Script (Bash)
set -euo pipefail

TARGET_DIR="dist/macos"
mkdir -p "$TARGET_DIR"

APP_BUNDLE="$TARGET_DIR/RealEarth.app"
rm -rf "$APP_BUNDLE"
mkdir -p "$APP_BUNDLE/Contents/MacOS" "$APP_BUNDLE/Contents/Resources/assets" \
         "$APP_BUNDLE/Contents/Resources/config" "$APP_BUNDLE/Contents/Resources/data/world_san_francisco"

# Binary staging
if [ -f "target/release/realearth-viewer" ]; then
    cp target/release/realearth-viewer "$APP_BUNDLE/Contents/MacOS/"
    chmod +x "$APP_BUNDLE/Contents/MacOS/realearth-viewer"
fi

cp packaging/macos/Info.plist "$APP_BUNDLE/Contents/"
printf "APPL????" > "$APP_BUNDLE/Contents/PkgInfo"
[ -f "config/engine_config.json" ] && cp config/engine_config.json "$APP_BUNDLE/Contents/Resources/config/"
[ -d "assets" ] && cp -r assets/* "$APP_BUNDLE/Contents/Resources/assets/"
[ -d "data/world_san_francisco" ] && cp -r data/world_san_francisco/* "$APP_BUNDLE/Contents/Resources/data/world_san_francisco/"

TAR_PATH="$TARGET_DIR/realearth-macos-universal.tar.gz"
tar -czf "$TAR_PATH" -C "$TARGET_DIR" RealEarth.app
sha256sum "$TAR_PATH" | awk '{print $1}' > "$TAR_PATH.sha256"

echo "[SUCCESS] macOS App Bundle packaged at $TAR_PATH"

if command -v hdiutil >/dev/null 2>&1; then
    DMG_PATH="$TARGET_DIR/RealEarth-macOS.dmg"
    hdiutil create -volname "RealEarth" -srcfolder "$APP_BUNDLE" -ov -format UDZO "$DMG_PATH"
    sha256sum "$DMG_PATH" | awk '{print $1}' > "$DMG_PATH.sha256"
    echo "[SUCCESS] Real macOS DMG generated at $DMG_PATH"
else
    echo "[NOTE] hdiutil not found. Genuine App Bundle packaged into $TAR_PATH."
fi
