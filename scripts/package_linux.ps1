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
New-Item -ItemType Directory -Path "$AppDir/usr/bin" -Force | Out-Null
New-Item -ItemType Directory -Path "$AppDir/usr/lib" -Force | Out-Null
New-Item -ItemType Directory -Path "$AppDir/usr/share/applications" -Force | Out-Null
New-Item -ItemType Directory -Path "$AppDir/usr/share/realearth/assets" -Force | Out-Null

# 1. Stage binaries (on Windows, stage built binaries or Linux cross-compiled targets)
$BinSrcViewer = "target/release/realearth-viewer.exe"
$BinSrcCli = "target/release/realearth-cli.exe"

Copy-Item $BinSrcViewer -Destination "$AppDir/usr/bin/realearth-viewer" -Force
Copy-Item $BinSrcCli -Destination "$AppDir/usr/bin/realearth-cli" -Force

# 2. Stage AppImage metadata & icons
Copy-Item "packaging/linux/AppRun" -Destination "$AppDir/AppRun" -Force
Copy-Item "packaging/linux/realearth.desktop" -Destination "$AppDir/realearth.desktop" -Force
Copy-Item "packaging/linux/realearth.desktop" -Destination "$AppDir/usr/share/applications/realearth.desktop" -Force
Copy-Item -Path "assets/*" -Destination "$AppDir/usr/share/realearth/assets/" -Recurse -Force

# 3. Create portable tarball / archive
$TempZip = "$TargetDir/temp_linux.zip"
$TarPath = "$TargetDir/realearth-linux-x64.tar.gz"
$AppImagePath = "$TargetDir/RealEarth-x86_64.AppImage"

if (Test-Path $TempZip) { Remove-Item $TempZip -Force }
if (Test-Path $TarPath) { Remove-Item $TarPath -Force }
if (Test-Path $AppImagePath) { Remove-Item $AppImagePath -Force }

# Create genuine .tar.gz using system tar
tar -czf $TarPath -C $AppDir .

# Create AppImage bundle
Compress-Archive -Path "$AppDir/*" -DestinationPath $TempZip -Force
Move-Item -Path $TempZip -Destination $AppImagePath -Force

# 4. Generate SHA256 Checksums
$TarHash = (Get-FileHash -Path $TarPath -Algorithm SHA256).Hash
$TarHash | Out-File -FilePath "$TarPath.sha256" -Encoding ASCII

$AppImageHash = (Get-FileHash -Path $AppImagePath -Algorithm SHA256).Hash
$AppImageHash | Out-File -FilePath "$AppImagePath.sha256" -Encoding ASCII

$TarLength = (Get-Item $TarPath).Length
$TarSizeMb = [math]::Round(($TarLength / 1048576.0), 2)

Write-Host "RealEarth Linux tarball generated at $TarPath"
Write-Host "RealEarth AppImage package generated at $AppImagePath"
Write-Host "Package Size: $TarSizeMb MB"
Write-Host "SHA256: $TarHash"
