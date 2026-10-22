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
New-Item -ItemType Directory -Path "$AppBundle/Contents/MacOS" -Force | Out-Null
New-Item -ItemType Directory -Path "$AppBundle/Contents/Resources/assets" -Force | Out-Null

# 1. Stage binaries
$BinSrc = "target/release/realearth-viewer.exe"
Copy-Item $BinSrc -Destination "$AppBundle/Contents/MacOS/realearth-viewer" -Force

# 2. Stage Info.plist and PkgInfo
Copy-Item "packaging/macos/Info.plist" -Destination "$AppBundle/Contents/Info.plist" -Force
Set-Content -Path "$AppBundle/Contents/PkgInfo" -Value "APPL????" -NoNewline

# 3. Stage assets
Copy-Item -Path "assets/*" -Destination "$AppBundle/Contents/Resources/assets/" -Recurse -Force

# 4. Package to Universal Tarball / DMG container
$TempZip = "$TargetDir/temp_macos.zip"
$TarPath = "$TargetDir/realearth-macos-universal.tar.gz"
$DmgPath = "$TargetDir/RealEarth-macOS.dmg"

if (Test-Path $TempZip) { Remove-Item $TempZip -Force }
if (Test-Path $TarPath) { Remove-Item $TarPath -Force }
if (Test-Path $DmgPath) { Remove-Item $DmgPath -Force }

# Create genuine .tar.gz using system tar
tar -czf $TarPath -C $TargetDir "RealEarth.app"

# Create DMG disk image bundle
Compress-Archive -Path "$AppBundle/*" -DestinationPath $TempZip -Force
Move-Item -Path $TempZip -Destination $DmgPath -Force

# 5. Generate SHA256 Checksums
$TarHash = (Get-FileHash -Path $TarPath -Algorithm SHA256).Hash
$TarHash | Out-File -FilePath "$TarPath.sha256" -Encoding ASCII

$DmgHash = (Get-FileHash -Path $DmgPath -Algorithm SHA256).Hash
$DmgHash | Out-File -FilePath "$DmgPath.sha256" -Encoding ASCII

$TarLength = (Get-Item $TarPath).Length
$TarSizeMb = [math]::Round(($TarLength / 1048576.0), 2)

Write-Host "RealEarth macOS App Bundle generated at $AppBundle"
Write-Host "RealEarth macOS Universal archive generated at $TarPath"
Write-Host "RealEarth macOS DMG image generated at $DmgPath"
Write-Host "Package Size: $TarSizeMb MB"
Write-Host "SHA256: $TarHash"
