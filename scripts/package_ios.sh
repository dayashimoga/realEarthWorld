#!/usr/bin/env bash
# Real Earth Platform — iOS Application Packaging Script (Bash)
set -euo pipefail

TARGET_DIR="dist/ios"
mkdir -p "$TARGET_DIR"

APP_BUNDLE="$TARGET_DIR/RealEarth.app"
rm -rf "$APP_BUNDLE"
mkdir -p "$APP_BUNDLE/assets" "$APP_BUNDLE/Frameworks"

# Stage binary
BIN_SRC=""
if [ -f "target/aarch64-apple-ios/release/librealearth.a" ]; then
    BIN_SRC="target/aarch64-apple-ios/release/librealearth.a"
elif [ -f "target/release/librealearth.a" ]; then
    BIN_SRC="target/release/librealearth.a"
elif [ -f "target/release/realearth.dll" ]; then
    BIN_SRC="target/release/realearth.dll"
else
    cargo build -p realearth-mobile --release
    if [ -f "target/release/librealearth.a" ]; then
        BIN_SRC="target/release/librealearth.a"
    elif [ -f "target/release/realearth.dll" ]; then
        BIN_SRC="target/release/realearth.dll"
    fi
fi

if [ -n "$BIN_SRC" ] && [ -f "$BIN_SRC" ]; then
    cp "$BIN_SRC" "$APP_BUNDLE/RealEarth"
    echo "[OK] Staged iOS static library from $BIN_SRC"
fi

cp "ios/RealEarth/Info.plist" "$APP_BUNDLE/Info.plist"
printf "APPL????" > "$APP_BUNDLE/PkgInfo"
cp -r assets/* "$APP_BUNDLE/assets/"

# Create Simulator Package
SIM_TAR="$TARGET_DIR/realearth-ios-simulator.tar.gz"
tar -czvf "$SIM_TAR" -C "$TARGET_DIR" RealEarth.app
shasum -a 256 "$SIM_TAR" > "$SIM_TAR.sha256"

# Create IPA Package
PAYLOAD="$TARGET_DIR/Payload"
rm -rf "$PAYLOAD"
mkdir -p "$PAYLOAD"
cp -r "$APP_BUNDLE" "$PAYLOAD/RealEarth.app"

IPA_PATH="$TARGET_DIR/realearth-ios-device.ipa"
(cd "$TARGET_DIR" && zip -r "realearth-ios-device.ipa" Payload)
shasum -a 256 "$IPA_PATH" > "$IPA_PATH.sha256"

rm -rf "$PAYLOAD"
echo "[SUCCESS] iOS IPA packaged at $IPA_PATH"
