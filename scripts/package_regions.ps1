# Real Earth Platform - Standalone Geographic Region Packaging Script (PowerShell)
param(
    [string]$DistDir = "dist/regions"
)

$ErrorActionPreference = "Stop"
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "   REAL EARTH - GEOGRAPHIC REGION PACKAGE GENERATION      " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

if (-not (Test-Path $DistDir)) {
    New-Item -ItemType Directory -Path $DistDir -Force | Out-Null
}

$RegionSource = "data/world_san_francisco"
if (-not (Test-Path $RegionSource)) {
    throw "Region source directory '$RegionSource' does not exist!"
}

$StagingDir = "$DistDir/staging_san_francisco"
if (Test-Path $StagingDir) {
    Remove-Item -Path $StagingDir -Recurse -Force
}
New-Item -ItemType Directory -Path "$StagingDir/world_san_francisco" -Force | Out-Null

Write-Host "[COPY] Staging San Francisco Bay Area pilot region..." -ForegroundColor Yellow
Copy-Item -Path "$RegionSource/*" -Destination "$StagingDir/world_san_francisco/" -Recurse -Force

# Add installation guide
$Readme = @"
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
"@
$Readme | Out-File -FilePath "$StagingDir/README.md" -Encoding UTF8

# 1. Create ZIP package
$ZipPath = "$DistDir/realearth-region-san-francisco.zip"
if (Test-Path $ZipPath) { Remove-Item $ZipPath -Force }
Add-Type -AssemblyName System.IO.Compression.FileSystem
$FullStaging = [System.IO.Path]::GetFullPath($StagingDir)
$FullZip = [System.IO.Path]::GetFullPath($ZipPath)
[System.IO.Compression.ZipFile]::CreateFromDirectory($FullStaging, $FullZip, [System.IO.Compression.CompressionLevel]::Optimal, $false)

$ZipHash = (Get-FileHash -Path $ZipPath -Algorithm SHA256).Hash
$ZipHash | Out-File -FilePath "$ZipPath.sha256" -Encoding ASCII
$ZipSizeMb = [math]::Round(((Get-Item $ZipPath).Length / 1MB), 2)
Write-Host "Generated: $ZipPath ($ZipSizeMb MB) [SHA256: $ZipHash]" -ForegroundColor Green

# 2. Create TAR.GZ package if tar is available
if (Get-Command tar -ErrorAction SilentlyContinue) {
    $TarGzPath = "$DistDir/realearth-region-san-francisco.tar.gz"
    if (Test-Path $TarGzPath) { Remove-Item $TarGzPath -Force }
    tar -czf $TarGzPath -C $StagingDir .
    $TarHash = (Get-FileHash -Path $TarGzPath -Algorithm SHA256).Hash
    $TarHash | Out-File -FilePath "$TarGzPath.sha256" -Encoding ASCII
    $TarSizeMb = [math]::Round(((Get-Item $TarGzPath).Length / 1MB), 2)
    Write-Host "Generated: $TarGzPath ($TarSizeMb MB) [SHA256: $TarHash]" -ForegroundColor Green
}

Remove-Item -Path $StagingDir -Recurse -Force
Write-Host "[SUCCESS] Standalone geographic region packaging complete!" -ForegroundColor Green
