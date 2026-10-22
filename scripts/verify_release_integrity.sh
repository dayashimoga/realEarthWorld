#!/usr/bin/env bash
# Real Earth Platform - Release Integrity and Artifact Certification Script (Bash)
set -euo pipefail

DIST_DIR="${1:-dist}"
MANIFEST_PATH="${2:-dist/release_manifest.json}"

echo "======================================================================"
echo "       REAL EARTH PLATFORM - RELEASE INTEGRITY CERTIFICATION          "
echo "======================================================================"

if [ ! -d "$DIST_DIR" ]; then
    echo "ERROR: Distribution directory '$DIST_DIR' does not exist! Run packaging first." >&2
    exit 1
fi

PACKAGES=(
    "Windows:x86_64:$DIST_DIR/windows/realearth-windows-x64.zip:zip:Unsigned Windows PE32+ (Development binary)"
    "Linux:x86_64:$DIST_DIR/linux/RealEarth-x86_64.AppImage:appimage:Unsigned Linux AppImage bundle"
    "Linux:x86_64:$DIST_DIR/linux/realearth-linux-x64.tar.gz:tar.gz:Unsigned Linux portable tarball"
    "macOS:Universal:$DIST_DIR/macos/RealEarth-macOS.dmg:dmg:Ad-hoc signed macOS DMG"
    "macOS:Universal:$DIST_DIR/macos/realearth-macos-universal.tar.gz:tar.gz:Ad-hoc signed macOS universal tarball"
    "Android:ARM64-v8a:$DIST_DIR/android/realearth-arm64.apk:apk:Self-signed Android debug keystore"
    "Android:Universal:$DIST_DIR/android/realearth-release.aab:aab:Self-signed Android AAB release bundle"
    "iOS:ARM64 Device:$DIST_DIR/ios/realearth-ios-device.ipa:ipa:Unsigned device IPA"
    "iOS:Simulator:$DIST_DIR/ios/realearth-ios-simulator.zip:zip:Unsigned Simulator bundle"
)

PASSED=0
TOTAL=${#PACKAGES[@]}

for ENTRY in "${PACKAGES[@]}"; do
    IFS=":" read -r PLATFORM ARCH FILEPATH FORMAT SIGNING_NOTE <<< "$ENTRY"
    FILENAME=$(basename "$FILEPATH")
    printf "Verifying: %-35s (%s %s)... " "$FILENAME" "$PLATFORM" "$ARCH"

    if [ ! -f "$FILEPATH" ]; then
        echo "FAILED! (File missing)"
        exit 1
    fi

    FILESIZE=$(stat -c%s "$FILEPATH" 2>/dev/null || stat -f%z "$FILEPATH" 2>/dev/null || wc -c < "$FILEPATH")
    if [ "$FILESIZE" -le 0 ]; then
        echo "FAILED! (Empty file)"
        exit 1
    fi

    SHA_FILE="$FILEPATH.sha256"
    if [ ! -f "$SHA_FILE" ]; then
        echo "FAILED! (Missing SHA256 file)"
        exit 1
    fi

    RECORDED_HASH=$(tr -d '[:space:]' < "$SHA_FILE" | awk '{print tolower($1)}')
    if command -v sha256sum >/dev/null 2>&1; then
        COMPUTED_HASH=$(sha256sum "$FILEPATH" | awk '{print tolower($1)}')
    else
        COMPUTED_HASH=$(shasum -a 256 "$FILEPATH" | awk '{print tolower($1)}')
    fi

    if [ "$RECORDED_HASH" != "$COMPUTED_HASH" ]; then
        echo "FAILED! (Hash mismatch: recorded $RECORDED_HASH vs computed $COMPUTED_HASH)"
        exit 1
    fi

    SIZE_MB=$(echo "scale=2; $FILESIZE / 1048576" | bc 2>/dev/null || awk "BEGIN {printf \"%.2f\", $FILESIZE/1048576}")
    echo "OK (${SIZE_MB} MB) [SHA256: ${COMPUTED_HASH:0:16}...]"
    PASSED=$((PASSED + 1))
done

echo "======================================================================"
echo "Certification Result: $PASSED / $TOTAL Packages Certified PASS"
echo "======================================================================"
