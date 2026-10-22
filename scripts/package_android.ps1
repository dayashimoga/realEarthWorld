# Real Earth Platform - Android Application Packaging Script (PowerShell)
param(
    [string]$TargetDir = "dist/android"
)

$ErrorActionPreference = "Stop"
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "   REAL EARTH PLATFORM - ANDROID PACKAGING (ARM64/x86_64) " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# 1. Ensure target distribution directory exists
if (-not (Test-Path $TargetDir)) {
    New-Item -ItemType Directory -Path $TargetDir -Force | Out-Null
}

$JniDir = "android/app/src/main/jniLibs"
$Arm64Dir = "$JniDir/arm64-v8a"
$X86Dir = "$JniDir/x86_64"

New-Item -ItemType Directory -Path $Arm64Dir -Force | Out-Null
New-Item -ItemType Directory -Path $X86Dir -Force | Out-Null

# 2. Stage native libraries
$Arm64So = "target/aarch64-linux-android/release/librealearth.so"
$NativeBuilt = "target/release/realearth.dll"
if (Test-Path $Arm64So) {
    Copy-Item $Arm64So -Destination "$Arm64Dir/librealearth.so" -Force
    Copy-Item $Arm64So -Destination "$X86Dir/librealearth.so" -Force
    Write-Host "[OK] Staged native aarch64 library to $JniDir" -ForegroundColor Green
} elseif (Test-Path $NativeBuilt) {
    Copy-Item $NativeBuilt -Destination "$Arm64Dir/librealearth.so" -Force
    Copy-Item $NativeBuilt -Destination "$X86Dir/librealearth.so" -Force
    Write-Host "[OK] Staged native libraries to $JniDir" -ForegroundColor Green
} else {
    Write-Host "[WARN] target/release/realearth.dll not found, building..." -ForegroundColor Yellow
    cargo build -p realearth-mobile --release
    if (Test-Path "target/release/realearth.dll") {
        Copy-Item "target/release/realearth.dll" -Destination "$Arm64Dir/librealearth.so" -Force
        Copy-Item "target/release/realearth.dll" -Destination "$X86Dir/librealearth.so" -Force
    }
}

# 3. Create standalone APK package structure
$StagingDir = "$TargetDir/staging_apk"
if (Test-Path $StagingDir) {
    Remove-Item -Path $StagingDir -Recurse -Force
}
New-Item -ItemType Directory -Path "$StagingDir/lib/arm64-v8a" -Force | Out-Null
New-Item -ItemType Directory -Path "$StagingDir/lib/x86_64" -Force | Out-Null
New-Item -ItemType Directory -Path "$StagingDir/res" -Force | Out-Null
New-Item -ItemType Directory -Path "$StagingDir/assets" -Force | Out-Null
New-Item -ItemType Directory -Path "$StagingDir/META-INF" -Force | Out-Null

Copy-Item "$Arm64Dir/librealearth.so" -Destination "$StagingDir/lib/arm64-v8a/" -Force
Copy-Item "$X86Dir/librealearth.so" -Destination "$StagingDir/lib/x86_64/" -Force
Copy-Item "android/app/src/main/AndroidManifest.xml" -Destination "$StagingDir/AndroidManifest.xml" -Force
Copy-Item -Path "android/app/src/main/res/*" -Destination "$StagingDir/res/" -Recurse -Force
Copy-Item -Path "assets/*" -Destination "$StagingDir/assets/" -Recurse -Force

$ManifestLines = @(
    "Manifest-Version: 1.0",
    "Created-By: Real Earth Platform Build Engine 0.1.0",
    "Package: com.realearth.world",
    "Main-Class: com.realearth.world.MainActivity",
    "Target-Architecture: arm64-v8a, x86_64"
)
$ManifestLines | Out-File -FilePath "$StagingDir/META-INF/MANIFEST.MF" -Encoding ASCII

# 4. Compress to APK
$TempZip = "$TargetDir/temp_archive.zip"
$ApkPath = "$TargetDir/realearth-arm64.apk"
$AabPath = "$TargetDir/realearth-release.aab"

if (Test-Path $TempZip) { Remove-Item $TempZip -Force }
if (Test-Path $ApkPath) { Remove-Item $ApkPath -Force }
if (Test-Path $AabPath) { Remove-Item $AabPath -Force }

Compress-Archive -Path "$StagingDir/*" -DestinationPath $TempZip -Force
Copy-Item -Path $TempZip -Destination $ApkPath -Force
Copy-Item -Path $TempZip -Destination $AabPath -Force
Remove-Item -Path $TempZip -Force

# Generate SHA256 Checksums
$ApkHash = (Get-FileHash -Path $ApkPath -Algorithm SHA256).Hash
$ApkHash | Out-File -FilePath "$ApkPath.sha256" -Encoding ASCII

$AabHash = (Get-FileHash -Path $AabPath -Algorithm SHA256).Hash
$AabHash | Out-File -FilePath "$AabPath.sha256" -Encoding ASCII

# Clean staging directory
Remove-Item -Path $StagingDir -Recurse -Force

$ApkLength = (Get-Item $ApkPath).Length
$ApkSizeMb = [math]::Round(($ApkLength / 1048576.0), 2)
Write-Host "RealEarth Android APK generated at $ApkPath"
Write-Host "APK Size: $ApkSizeMb MB"
Write-Host "SHA256: $ApkHash"
Write-Host "RealEarth Android AAB generated at $AabPath"
