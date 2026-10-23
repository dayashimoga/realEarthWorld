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

MANDATORY_PACKAGES=(
    "Windows:x86_64:$DIST_DIR/windows/realearth-windows-x64.zip:zip:Unsigned Windows PE32+ (Development release binary)"
    "Linux:x86_64:$DIST_DIR/linux/realearth-linux-x64.tar.gz:tar.gz:Unsigned Linux portable AppDir tarball"
    "macOS:Universal:$DIST_DIR/macos/realearth-macos-universal.tar.gz:tar.gz:Ad-hoc signed macOS universal tarball"
    "iOS:Simulator:$DIST_DIR/ios/realearth-ios-simulator.zip:zip:Unsigned iOS Simulator bundle"
    "DataRegion:San Francisco:$DIST_DIR/regions/realearth-region-san-francisco.tar.gz:tar.gz:Authentic surveyed OpenStreetMap + USGS 3DEP pilot dataset"
)

PASSED=0

for ENTRY in "${MANDATORY_PACKAGES[@]}"; do
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

    SIZE_MB=$(awk "BEGIN {printf \"%.2f\", $FILESIZE/1048576}")
    echo "OK (${SIZE_MB} MB) [SHA256: ${COMPUTED_HASH:0:16}...]"
    PASSED=$((PASSED + 1))
done

# Write unified SHA256SUMS file
SUMS_PATH="$DIST_DIR/SHA256SUMS"
> "$SUMS_PATH"
for ENTRY in "${MANDATORY_PACKAGES[@]}"; do
    IFS=":" read -r PLATFORM ARCH FILEPATH FORMAT SIGNING_NOTE <<< "$ENTRY"
    if [ -f "$FILEPATH" ]; then
        REL_PATH="${FILEPATH#$DIST_DIR/}"
        if command -v sha256sum >/dev/null 2>&1; then
            sha256sum "$FILEPATH" | awk -v r="$REL_PATH" '{print $1 "  " r}' >> "$SUMS_PATH"
        else
            shasum -a 256 "$FILEPATH" | awk -v r="$REL_PATH" '{print $1 "  " r}' >> "$SUMS_PATH"
        fi
    fi
done

echo "======================================================================"
echo "Certification Result: $PASSED Certified Packages Verified PASS"
echo "Unified Checksums Written: $SUMS_PATH"
echo "======================================================================"
