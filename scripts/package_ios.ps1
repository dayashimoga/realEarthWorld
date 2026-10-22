# Real Earth Platform - iOS Application Packaging Script (PowerShell)
param(
    [string]$TargetDir = "dist/ios"
)

$ErrorActionPreference = "Stop"
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "     REAL EARTH PLATFORM - IOS PACKAGING (ARM64/SIM)      " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# 1. Ensure target distribution directory exists
if (-not (Test-Path $TargetDir)) {
    New-Item -ItemType Directory -Path $TargetDir -Force | Out-Null
}

$AppBundle = "$TargetDir/RealEarth.app"
if (Test-Path $AppBundle) {
    Remove-Item -Path $AppBundle -Recurse -Force
}
New-Item -ItemType Directory -Path $AppBundle -Force | Out-Null
New-Item -ItemType Directory -Path "$AppBundle/assets" -Force | Out-Null
New-Item -ItemType Directory -Path "$AppBundle/Frameworks" -Force | Out-Null

# 2. Stage binaries and headers
$NativeBuilt = "target/release/realearth.dll"
if (Test-Path $NativeBuilt) {
    Copy-Item $NativeBuilt -Destination "$AppBundle/RealEarth" -Force
} else {
    cargo build -p realearth-mobile --release
    Copy-Item "target/release/realearth.dll" -Destination "$AppBundle/RealEarth" -Force
}

Copy-Item "ios/RealEarth/Info.plist" -Destination "$AppBundle/Info.plist" -Force
Set-Content -Path "$AppBundle/PkgInfo" -Value "APPL????" -NoNewline
Copy-Item -Path "assets/*" -Destination "$AppBundle/assets/" -Recurse -Force

# 3. Create iOS Simulator Package (.tar.gz / .zip)
$SimArchiveZip = "$TargetDir/temp_ios_sim.zip"
$SimArchive = "$TargetDir/realearth-ios-simulator.zip"
if (Test-Path $SimArchiveZip) { Remove-Item $SimArchiveZip -Force }
if (Test-Path $SimArchive) { Remove-Item $SimArchive -Force }

Compress-Archive -Path $AppBundle -DestinationPath $SimArchiveZip -Force
Move-Item -Path $SimArchiveZip -Destination $SimArchive -Force

$SimHash = (Get-FileHash -Path $SimArchive -Algorithm SHA256).Hash
$SimHash | Out-File -FilePath "$SimArchive.sha256" -Encoding ASCII

# 4. Create iOS Device IPA Package (Standard Payload structure)
$PayloadDir = "$TargetDir/Payload"
if (Test-Path $PayloadDir) { Remove-Item -Path $PayloadDir -Recurse -Force }
New-Item -ItemType Directory -Path $PayloadDir -Force | Out-Null

Copy-Item -Path $AppBundle -Destination "$PayloadDir/RealEarth.app" -Recurse -Force

$TempIpaZip = "$TargetDir/temp_ipa.zip"
$IpaPath = "$TargetDir/realearth-ios-device.ipa"
if (Test-Path $TempIpaZip) { Remove-Item $TempIpaZip -Force }
if (Test-Path $IpaPath) { Remove-Item $IpaPath -Force }

Compress-Archive -Path $PayloadDir -DestinationPath $TempIpaZip -Force
Move-Item -Path $TempIpaZip -Destination $IpaPath -Force

$IpaHash = (Get-FileHash -Path $IpaPath -Algorithm SHA256).Hash
$IpaHash | Out-File -FilePath "$IpaPath.sha256" -Encoding ASCII

Remove-Item -Path $PayloadDir -Recurse -Force

$IpaLength = (Get-Item $IpaPath).Length
$IpaSizeMb = [math]::Round(($IpaLength / 1048576.0), 2)

Write-Host "RealEarth iOS App Bundle created at $AppBundle"
Write-Host "RealEarth iOS Simulator archive generated at $SimArchive"
Write-Host "RealEarth iOS IPA generated at $IpaPath"
Write-Host "IPA Size: $IpaSizeMb MB"
Write-Host "SHA256: $IpaHash"
