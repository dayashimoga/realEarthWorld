#!/usr/bin/env bash
# Real Earth Platform - Standalone Geographic Region Packaging Script (Bash)
set -euo pipefail

DIST_DIR="${1:-dist/regions}"
REGION_SOURCE="data/world_san_francisco"

echo "=========================================================="
echo "   REAL EARTH - GEOGRAPHIC REGION PACKAGE GENERATION      "
echo "=========================================================="

if [ ! -d "$REGION_SOURCE" ]; then
    echo "ERROR: Region source directory '$REGION_SOURCE' does not exist!" >&2
    exit 1
fi

mkdir -p "$DIST_DIR"
STAGING_DIR="$DIST_DIR/staging_san_francisco"
rm -rf "$STAGING_DIR"
mkdir -p "$STAGING_DIR/world_san_francisco"

echo "[COPY] Staging San Francisco Bay Area pilot region..."
cp -r "$REGION_SOURCE"/* "$STAGING_DIR/world_san_francisco/"

cat << 'EOF' > "$STAGING_DIR/README.md"
# Real Earth Geographic Region Package: San Francisco Bay Area (Pilot)
- **Region ID:** san-francisco
- **Format:** SQLite Catalog (chunks.db) + Zstd Binary Chunks (.rec)
- **NAVD88 Elevation:** 0.0m - 105.0m MSL
- **Chunks:** 30 authentic surveyed tiles

## Installation
Extract the 'world_san_francisco' directory into your Real Earth 'data/' folder:
  <realearth_install_dir>/data/world_san_francisco/

Then launch the viewer:
  realearth-viewer --region san-francisco
EOF

# 1. Create TAR.GZ package
TAR_PATH="$DIST_DIR/realearth-region-san-francisco.tar.gz"
tar -czf "$TAR_PATH" -C "$STAGING_DIR" .

if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$TAR_PATH" | awk '{print toupper($1)}' > "$TAR_PATH.sha256"
else
    shasum -a 256 "$TAR_PATH" | awk '{print toupper($1)}' > "$TAR_PATH.sha256"
fi

# 2. Create ZIP package if zip is available
if command -v zip >/dev/null 2>&1; then
    ZIP_PATH="$DIST_DIR/realearth-region-san-francisco.zip"
    (cd "$STAGING_DIR" && zip -r -q "$OLDPWD/$ZIP_PATH" .)
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$ZIP_PATH" | awk '{print toupper($1)}' > "$ZIP_PATH.sha256"
    else
        shasum -a 256 "$ZIP_PATH" | awk '{print toupper($1)}' > "$ZIP_PATH.sha256"
    fi
fi

rm -rf "$STAGING_DIR"
echo "[SUCCESS] Standalone geographic region packaging complete in $DIST_DIR!"
