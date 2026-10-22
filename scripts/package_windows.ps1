# Real Earth Platform - Windows x86_64 Application Packaging Script (PowerShell)
param(
    [string]$TargetDir = "dist/windows"
)

$ErrorActionPreference = "Stop"
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "    REAL EARTH PLATFORM - WINDOWS PACKAGING (x86_64)      " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

if (-not (Test-Path $TargetDir)) {
    New-Item -ItemType Directory -Path $TargetDir -Force | Out-Null
}

$StagingDir = "$TargetDir/staging_windows"
if (Test-Path $StagingDir) {
    Remove-Item -Path $StagingDir -Recurse -Force
}
New-Item -ItemType Directory -Path $StagingDir -Force | Out-Null
New-Item -ItemType Directory -Path "$StagingDir/assets" -Force | Out-Null
New-Item -ItemType Directory -Path "$StagingDir/data" -Force | Out-Null
New-Item -ItemType Directory -Path "$StagingDir/docs" -Force | Out-Null

# 1. Ensure binaries are built
if (-not (Test-Path "target/release/realearth-viewer.exe")) {
    Write-Host "[BUILD] Compiling release binaries..." -ForegroundColor Yellow
    cargo build --workspace --release
}

# 2. Stage binaries
Copy-Item "target/release/realearth-viewer.exe" -Destination "$StagingDir/" -Force
Copy-Item "target/release/realearth-cli.exe" -Destination "$StagingDir/" -Force
if (Test-Path "target/release/realearth.dll") {
    Copy-Item "target/release/realearth.dll" -Destination "$StagingDir/" -Force
}

# 3. Stage assets, data, and docs
Copy-Item -Path "assets/*" -Destination "$StagingDir/assets/" -Recurse -Force
Copy-Item -Path "data/*" -Destination "$StagingDir/data/" -Recurse -Force
Copy-Item "docs/UserGuide.md" -Destination "$StagingDir/docs/" -Force
Copy-Item "docs/SetupAndConfiguration.md" -Destination "$StagingDir/docs/" -Force

# 4. Create launcher scripts
$ViewerBat = @"
@echo off
echo Starting Real Earth World Viewer (DirectX 12 / Vulkan / wgpu)...
start "" "%~dp0realearth-viewer.exe" --region san-francisco
"@
$ViewerBat | Out-File -FilePath "$StagingDir/run_viewer.bat" -Encoding ASCII

$CliBat = @"
@echo off
"%~dp0realearth-cli.exe" %*
"@
$CliBat | Out-File -FilePath "$StagingDir/run_cli.bat" -Encoding ASCII

# 5. Compress to portable ZIP
$ZipPath = "$TargetDir/realearth-windows-x64.zip"
if (Test-Path $ZipPath) {
    Remove-Item $ZipPath -Force
}

Compress-Archive -Path "$StagingDir/*" -DestinationPath $ZipPath -Force
Remove-Item -Path $StagingDir -Recurse -Force

# 6. Generate SHA256 Checksum
$ZipHash = (Get-FileHash -Path $ZipPath -Algorithm SHA256).Hash
$ZipHash | Out-File -FilePath "$ZipPath.sha256" -Encoding ASCII

$ZipLength = (Get-Item $ZipPath).Length
$ZipSizeMb = [math]::Round(($ZipLength / 1048576.0), 2)

Write-Host "RealEarth Windows portable package generated at $ZipPath"
Write-Host "Size: $ZipSizeMb MB"
Write-Host "SHA256: $ZipHash"
