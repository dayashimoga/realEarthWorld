# Real Earth Platform - Master Multi-Platform Packaging Script (PowerShell)
$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "   REAL EARTH PLATFORM - FIVE-PLATFORM PACKAGE RELEASE    " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# 1. Compile release binaries
Write-Host "[1/6] Compiling release binaries..." -ForegroundColor Yellow
cargo build --workspace --release

# 2. Package Windows x86_64
Write-Host "[2/6] Packaging Windows x86_64..." -ForegroundColor Yellow
& powershell -ExecutionPolicy Bypass -File scripts/package_windows.ps1

# 3. Package Linux x86_64
Write-Host "[3/6] Packaging Linux x86_64..." -ForegroundColor Yellow
& powershell -ExecutionPolicy Bypass -File scripts/package_linux.ps1

# 4. Package macOS Universal
Write-Host "[4/6] Packaging macOS Universal..." -ForegroundColor Yellow
& powershell -ExecutionPolicy Bypass -File scripts/package_macos.ps1

# 5. Package Android and iOS Mobile
Write-Host "[5/7] Packaging Android and iOS Mobile..." -ForegroundColor Yellow
& powershell -ExecutionPolicy Bypass -File scripts/package_android.ps1
& powershell -ExecutionPolicy Bypass -File scripts/package_ios.ps1

# 6. Package Standalone Geographic Regions
Write-Host "[6/7] Packaging Standalone Geographic Regions..." -ForegroundColor Yellow
& powershell -ExecutionPolicy Bypass -File scripts/package_regions.ps1

# 7. Verify Release Integrity and Generate Manifest
Write-Host "[7/7] Verifying Release Integrity and Generating Manifest..." -ForegroundColor Yellow
& powershell -ExecutionPolicy Bypass -File scripts/verify_release_integrity.ps1

Write-Host "==========================================================" -ForegroundColor Green
Write-Host " [SUCCESS] Five-platform packaging complete in dist/      " -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Green
