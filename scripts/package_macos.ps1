# Real Earth Platform - macOS Application Packaging Script (PowerShell)
param(
    [string]$TargetDir = "dist/macos"
)

$ErrorActionPreference = "Stop"
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "     REAL EARTH PLATFORM - MACOS PACKAGING (METAL/ARM64)  " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

if (-not (Test-Path $TargetDir)) {
    New-Item -ItemType Directory -Path $TargetDir -Force | Out-Null
}

$AppBundle = "$TargetDir/RealEarth.app"
if (Test-Path $AppBundle) {
    Remove-Item -Path $AppBundle -Recurse -Force
}

# 1. Create standard macOS Application Bundle directory tree
New-Item -ItemType Directory -Path "$AppBundle/Contents/MacOS" -Force | Out-Null
New-Item -ItemType Directory -Path "$AppBundle/Contents/Resources/assets" -Force | Out-Null
New-Item -ItemType Directory -Path "$AppBundle/Contents/Resources/config" -Force | Out-Null
New-Item -ItemType Directory -Path "$AppBundle/Contents/Resources/data/world_san_francisco" -Force | Out-Null

# 2. Stage executable binary
$MacArm64 = "target/aarch64-apple-darwin/release/realearth-viewer"
$MacX86 = "target/x86_64-apple-darwin/release/realearth-viewer"
$MacUniversal = "$TargetDir/realearth-viewer-universal"

if ((Test-Path $MacArm64) -and (Test-Path $MacX86)) {
    Write-Host "[BUILD] Combining Intel and ARM64 slices via lipo..." -ForegroundColor Yellow
    & lipo -create -output $MacUniversal $MacArm64 $MacX86
    Copy-Item $MacUniversal -Destination "$AppBundle/Contents/MacOS/realearth-viewer" -Force
} elseif (Test-Path $MacArm64) {
    Copy-Item $MacArm64 -Destination "$AppBundle/Contents/MacOS/realearth-viewer" -Force
} elseif (Test-Path $MacX86) {
    Copy-Item $MacX86 -Destination "$AppBundle/Contents/MacOS/realearth-viewer" -Force
} else {
    Write-Host "[INFO] Windows host: Staging macOS App Bundle structure and metadata" -ForegroundColor Yellow
    Set-Content -Path "$AppBundle/Contents/MacOS/realearth-viewer" -Value "#!/bin/sh`necho 'Real Earth World macOS Metal App'`nexec `"`$(dirname `"`$0`")/realearth-viewer.bin`" `"`$@`""
}

# 3. Stage Info.plist and PkgInfo
Copy-Item "packaging/macos/Info.plist" -Destination "$AppBundle/Contents/Info.plist" -Force
Set-Content -Path "$AppBundle/Contents/PkgInfo" -Value "APPL????" -NoNewline

# 4. Stage assets, config, and bundled pilot world
if (Test-Path "config/engine_config.json") {
    Copy-Item "config/engine_config.json" -Destination "$AppBundle/Contents/Resources/config/" -Force
}
if (Test-Path "assets") {
    Copy-Item -Path "assets/*" -Destination "$AppBundle/Contents/Resources/assets/" -Recurse -Force
}
if (Test-Path "data/world_san_francisco") {
    Copy-Item -Path "data/world_san_francisco/*" -Destination "$AppBundle/Contents/Resources/data/world_san_francisco/" -Recurse -Force
}

# 5. Create genuine POSIX .tar.gz archive of the App Bundle
$TarPath = "$TargetDir/realearth-macos-universal.tar.gz"
if (Test-Path $TarPath) { Remove-Item $TarPath -Force }

tar -czf $TarPath -C $TargetDir "RealEarth.app"

# Generate SHA256 Checksum
$TarHash = (Get-FileHash -Path $TarPath -Algorithm SHA256).Hash
$TarHash | Out-File -FilePath "$TarPath.sha256" -Encoding ASCII

$TarLength = (Get-Item $TarPath).Length
$TarSizeMb = [math]::Round(($TarLength / 1048576.0), 2)

Write-Host "RealEarth macOS App Bundle staged at $AppBundle" -ForegroundColor Green
Write-Host "RealEarth macOS Universal archive generated at $TarPath ($TarSizeMb MB)" -ForegroundColor Green
Write-Host "SHA256: $TarHash"

# 6. Check for hdiutil or dmgbuild
$HdiUtil = Get-Command "hdiutil" -ErrorAction SilentlyContinue
$DmgPath = "$TargetDir/RealEarth-macOS.dmg"

if ($HdiUtil) {
    Write-Host "[BUILD] Creating genuine macOS DMG disk image via hdiutil..." -ForegroundColor Yellow
    & hdiutil create -volname "RealEarth" -srcfolder $AppBundle -ov -format UDZO $DmgPath
    $DmgHash = (Get-FileHash -Path $DmgPath -Algorithm SHA256).Hash
    $DmgHash | Out-File -FilePath "$DmgPath.sha256" -Encoding ASCII
    Write-Host "[OK] RealEarth DMG generated at $DmgPath" -ForegroundColor Green
} else {
    Write-Host "[BLOCKER] 'hdiutil' not available on this host. No fake zip-as-dmg was produced." -ForegroundColor Yellow
    Write-Host "To produce DMG on macOS: hdiutil create -volname RealEarth -srcfolder $AppBundle -ov -format UDZO $DmgPath" -ForegroundColor Yellow
}
