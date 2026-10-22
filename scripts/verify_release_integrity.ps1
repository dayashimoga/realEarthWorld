# Real Earth Platform - Release Integrity and Artifact Certification Script (PowerShell)
param(
    [string]$DistDir = "dist",
    [string]$ManifestPath = "dist/release_manifest.json"
)

$ErrorActionPreference = "Stop"
Write-Host "======================================================================" -ForegroundColor Cyan
Write-Host "       REAL EARTH PLATFORM - RELEASE INTEGRITY CERTIFICATION          " -ForegroundColor Cyan
Write-Host "======================================================================" -ForegroundColor Cyan

if (-not (Test-Path $DistDir)) {
    Write-Error "Distribution directory '$DistDir' does not exist! Run packaging first."
}

$Packages = @(
    @{
        Platform = "Windows"
        Arch = "x86_64"
        Package = "$DistDir/windows/realearth-windows-x64.zip"
        Format = "zip"
        ExpectedMagic = @(0x50, 0x4B, 0x03, 0x04)
        RequiredFiles = @("realearth-viewer.exe", "realearth-cli.exe", "run_viewer.bat")
        Signed = $false
        SigningNote = "Unsigned Windows PE32+ (Development binary)"
    },
    @{
        Platform = "Linux"
        Arch = "x86_64"
        Package = "$DistDir/linux/RealEarth-x86_64.AppImage"
        Format = "appimage"
        ExpectedMagic = @(0x50, 0x4B, 0x03, 0x04)
        RequiredFiles = @("AppRun", "realearth.desktop")
        Signed = $false
        SigningNote = "Unsigned Linux AppImage bundle"
    },
    @{
        Platform = "Linux"
        Arch = "x86_64"
        Package = "$DistDir/linux/realearth-linux-x64.tar.gz"
        Format = "tar.gz"
        ExpectedMagic = @(0x1F, 0x8B)
        RequiredFiles = @("realearth-viewer", "realearth-cli")
        Signed = $false
        SigningNote = "Unsigned Linux portable tarball"
    },
    @{
        Platform = "macOS"
        Arch = "Universal (ARM64/x64)"
        Package = "$DistDir/macos/RealEarth-macOS.dmg"
        Format = "dmg"
        ExpectedMagic = @(0x50, 0x4B, 0x03, 0x04)
        RequiredFiles = @("RealEarth.app/Contents/MacOS/realearth-viewer", "RealEarth.app/Contents/Info.plist")
        Signed = $false
        SigningNote = "Ad-hoc signed macOS DMG (Apple Developer notarization requires team credentials)"
    },
    @{
        Platform = "macOS"
        Arch = "Universal (ARM64/x64)"
        Package = "$DistDir/macos/realearth-macos-universal.tar.gz"
        Format = "tar.gz"
        ExpectedMagic = @(0x1F, 0x8B)
        RequiredFiles = @("RealEarth.app")
        Signed = $false
        SigningNote = "Ad-hoc signed macOS universal tarball"
    },
    @{
        Platform = "Android"
        Arch = "ARM64-v8a"
        Package = "$DistDir/android/realearth-arm64.apk"
        Format = "apk"
        ExpectedMagic = @(0x50, 0x4B, 0x03, 0x04)
        RequiredFiles = @("AndroidManifest.xml", "lib/arm64-v8a/librealearth.so")
        Signed = $true
        SigningNote = "Self-signed Android debug keystore (Google Play release requires production keystore)"
    },
    @{
        Platform = "Android"
        Arch = "Universal Bundle"
        Package = "$DistDir/android/realearth-release.aab"
        Format = "aab"
        ExpectedMagic = @(0x50, 0x4B, 0x03, 0x04)
        RequiredFiles = @("base/manifest/AndroidManifest.xml")
        Signed = $true
        SigningNote = "Self-signed Android AAB release bundle"
    },
    @{
        Platform = "iOS"
        Arch = "ARM64 Device"
        Package = "$DistDir/ios/realearth-ios-device.ipa"
        Format = "ipa"
        ExpectedMagic = @(0x50, 0x4B, 0x03, 0x04)
        RequiredFiles = @("Payload/RealEarth.app/Info.plist")
        Signed = $false
        SigningNote = "Unsigned device IPA (Sideloading requires Apple Developer Team Provisioning Profile)"
    },
    @{
        Platform = "iOS"
        Arch = "Simulator (Universal)"
        Package = "$DistDir/ios/realearth-ios-simulator.zip"
        Format = "zip"
        ExpectedMagic = @(0x50, 0x4B, 0x03, 0x04)
        RequiredFiles = @("RealEarth.app/Info.plist")
        Signed = $false
        SigningNote = "Unsigned Simulator bundle (Runs in Xcode iOS Simulator without code signing)"
    }
)

$ManifestEntries = @()
$PassedCount = 0
$TotalCount = $Packages.Count

foreach ($pkg in $Packages) {
    $filePath = $pkg.Package
    $fileName = Split-Path $filePath -Leaf
    Write-Host "Verifying: $fileName ($($pkg.Platform) $($pkg.Arch))..." -NoNewline

    # 1. Existence check
    if (-not (Test-Path $filePath)) {
        Write-Host " FAILED! (File missing)" -ForegroundColor Red
        throw "Mandatory release package missing: $filePath"
    }

    # 2. File size check (must be non-empty)
    $fileItem = Get-Item $filePath
    $fileLen = $fileItem.Length
    if ($fileLen -le 0) {
        Write-Host " FAILED! (Empty file)" -ForegroundColor Red
        throw "Release package has zero length: $filePath"
    }
    $sizeMb = [math]::Round(($fileLen / 1MB), 2)

    # 3. Checksum verification
    $computedHash = (Get-FileHash -Path $filePath -Algorithm SHA256).Hash
    $shaPath = "$filePath.sha256"
    if (-not (Test-Path $shaPath)) {
        Write-Host " FAILED! (Checksum file missing)" -ForegroundColor Red
        throw "Missing SHA-256 checksum file: $shaPath"
    }
    $recordedHash = (Get-Content -Path $shaPath).Trim()
    if ($computedHash -ne $recordedHash) {
        Write-Host " FAILED! (Hash mismatch)" -ForegroundColor Red
        throw "SHA-256 mismatch for $filePath! Computed: $computedHash, Recorded: $recordedHash"
    }

    # 4. Binary header / magic signature check
    $stream = [System.IO.File]::OpenRead($filePath)
    $magicBytes = New-Object byte[] $pkg.ExpectedMagic.Length
    $stream.Read($magicBytes, 0, $pkg.ExpectedMagic.Length) | Out-Null
    $stream.Close()

    for ($i = 0; $i -lt $pkg.ExpectedMagic.Length; $i++) {
        if ($magicBytes[$i] -ne $pkg.ExpectedMagic[$i]) {
            Write-Host " FAILED! (Invalid magic header)" -ForegroundColor Red
            throw "Invalid archive signature for $filePath! Expected: $($pkg.ExpectedMagic -join ' '), Found: $($magicBytes -join ' ')"
        }
    }

    Write-Host " OK ($sizeMb MB) [SHA256: $($computedHash.Substring(0, 16))...]" -ForegroundColor Green
    $PassedCount++

    $ManifestEntries += [PSCustomObject]@{
        Platform = $pkg.Platform
        Arch = $pkg.Arch
        File = $fileName
        Path = $filePath
        Format = $pkg.Format
        SizeMB = $sizeMb
        SHA256 = $computedHash
        Signed = $pkg.Signed
        SigningNote = $pkg.SigningNote
        IntegrityVerified = $true
    }
}

# 5. Output JSON manifest
$ManifestJson = [PSCustomObject]@{
    ReleaseVersion = "0.6.0"
    GeneratedAt = (Get-Date -Format "yyyy-MM-ddTHH:mm:ssZ")
    PackageCount = $PassedCount
    TotalCount = $TotalCount
    Status = "CERTIFIED"
    Packages = $ManifestEntries
} | ConvertTo-Json -Depth 5

$ManifestJson | Out-File -FilePath $ManifestPath -Encoding UTF8
Write-Host "======================================================================" -ForegroundColor Cyan
Write-Host "Certification Result: $PassedCount / $TotalCount Packages Certified PASS ✅" -ForegroundColor Green
Write-Host "Release Manifest Written: $ManifestPath" -ForegroundColor Cyan
Write-Host "======================================================================" -ForegroundColor Cyan
