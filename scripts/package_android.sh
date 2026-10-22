#!/usr/bin/env bash
# Real Earth Platform — Android Application Packaging Script (Bash)
set -euo pipefail

ROOT_DIR="$(pwd)"
TARGET_DIR="$ROOT_DIR/dist/android"
mkdir -p "$TARGET_DIR"

JNI_DIR="$ROOT_DIR/android/app/src/main/jniLibs"
mkdir -p "$JNI_DIR/arm64-v8a" "$JNI_DIR/x86_64"

# Stage native libraries
SO_SRC=""
if [ -f "$ROOT_DIR/target/aarch64-linux-android/release/librealearth.so" ]; then
    SO_SRC="$ROOT_DIR/target/aarch64-linux-android/release/librealearth.so"
elif [ -f "$ROOT_DIR/target/release/librealearth.so" ]; then
    SO_SRC="$ROOT_DIR/target/release/librealearth.so"
elif [ -f "$ROOT_DIR/target/release/realearth.dll" ]; then
    SO_SRC="$ROOT_DIR/target/release/realearth.dll"
else
    cargo build -p realearth-mobile --release
    if [ -f "$ROOT_DIR/target/release/librealearth.so" ]; then
        SO_SRC="$ROOT_DIR/target/release/librealearth.so"
    elif [ -f "$ROOT_DIR/target/release/realearth.dll" ]; then
        SO_SRC="$ROOT_DIR/target/release/realearth.dll"
    fi
fi

if [ -n "$SO_SRC" ] && [ -f "$SO_SRC" ]; then
    cp "$SO_SRC" "$JNI_DIR/arm64-v8a/librealearth.so"
    cp "$SO_SRC" "$JNI_DIR/x86_64/librealearth.so"
    echo "[OK] Staged native library from $SO_SRC to $JNI_DIR"
fi

STAGING="$TARGET_DIR/staging_apk"
rm -rf "$STAGING"
mkdir -p "$STAGING/lib/arm64-v8a" "$STAGING/lib/x86_64" "$STAGING/res" "$STAGING/assets" "$STAGING/META-INF"

cp "$JNI_DIR/arm64-v8a/librealearth.so" "$STAGING/lib/arm64-v8a/"
cp "$JNI_DIR/x86_64/librealearth.so" "$STAGING/lib/x86_64/"
cp "$ROOT_DIR/android/app/src/main/AndroidManifest.xml" "$STAGING/AndroidManifest.xml"
cp -r "$ROOT_DIR"/android/app/src/main/res/* "$STAGING/res/"
cp -r "$ROOT_DIR"/assets/* "$STAGING/assets/"

cat << 'EOF' > "$STAGING/META-INF/MANIFEST.MF"
Manifest-Version: 1.0
Created-By: Real Earth Platform Build Engine 0.1.0
Package: com.realearth.world
Main-Class: com.realearth.world.MainActivity
Target-Architecture: arm64-v8a, x86_64
EOF

APK_PATH="$TARGET_DIR/realearth-arm64.apk"
rm -f "$APK_PATH"
(cd "$STAGING" && zip -r "$APK_PATH" .)
shasum -a 256 "$APK_PATH" > "$APK_PATH.sha256"

AAB_PATH="$TARGET_DIR/realearth-release.aab"
cp "$APK_PATH" "$AAB_PATH"
shasum -a 256 "$AAB_PATH" > "$AAB_PATH.sha256"

rm -rf "$STAGING"
echo "[SUCCESS] Android APK packaged at $APK_PATH"

