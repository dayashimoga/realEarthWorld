#!/usr/bin/env bash
# Real Earth Platform — iOS Application Packaging Script (Bash)
set -euo pipefail

ROOT_DIR="$(pwd)"
TARGET_DIR="$ROOT_DIR/dist/ios"
mkdir -p "$TARGET_DIR"

APP_BUNDLE="$TARGET_DIR/RealEarth.app"
rm -rf "$APP_BUNDLE"
mkdir -p "$APP_BUNDLE/assets" "$APP_BUNDLE/Frameworks"

# Stage binary
BIN_SRC=""
if [ -f "$ROOT_DIR/target/aarch64-apple-ios/release/librealearth.a" ]; then
    BIN_SRC="$ROOT_DIR/target/aarch64-apple-ios/release/librealearth.a"
elif [ -f "$ROOT_DIR/target/release/librealearth.a" ]; then
    BIN_SRC="$ROOT_DIR/target/release/librealearth.a"
elif [ -f "$ROOT_DIR/target/release/realearth.dll" ]; then
    BIN_SRC="$ROOT_DIR/target/release/realearth.dll"
else
    cargo build -p realearth-mobile --release
    if [ -f "$ROOT_DIR/target/release/librealearth.a" ]; then
        BIN_SRC="$ROOT_DIR/target/release/librealearth.a"
    elif [ -f "$ROOT_DIR/target/release/realearth.dll" ]; then
        BIN_SRC="$ROOT_DIR/target/release/realearth.dll"
    fi
fi

if [ -n "$BIN_SRC" ] && [ -f "$BIN_SRC" ]; then
    cp "$BIN_SRC" "$APP_BUNDLE/RealEarth"
    echo "[OK] Staged iOS static library from $BIN_SRC"
fi

cp "$ROOT_DIR/ios/RealEarth/Info.plist" "$APP_BUNDLE/Info.plist"
printf "APPL????" > "$APP_BUNDLE/PkgInfo"
cp -r "$ROOT_DIR"/assets/* "$APP_BUNDLE/assets/"

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
rm -f "$IPA_PATH"
(cd "$TARGET_DIR" && zip -r "$IPA_PATH" Payload)
shasum -a 256 "$IPA_PATH" > "$IPA_PATH.sha256"

# Create Simulator Zip
SIM_ZIP="$TARGET_DIR/realearth-ios-simulator.zip"
rm -f "$SIM_ZIP"
(cd "$TARGET_DIR" && zip -r "$SIM_ZIP" RealEarth.app)
shasum -a 256 "$SIM_ZIP" > "$SIM_ZIP.sha256"

rm -rf "$PAYLOAD"
echo "[SUCCESS] iOS IPA packaged at $IPA_PATH"

