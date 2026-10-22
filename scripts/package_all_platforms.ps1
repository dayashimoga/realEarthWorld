# Real Earth Platform — Master Multi-Platform Packaging Script (PowerShell)
$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "   REAL EARTH PLATFORM — FIVE-PLATFORM PACKAGE RELEASE    " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# 1. Compile release binaries
Write-Host "[1/5] Compiling release binaries..." -ForegroundColor Yellow
cargo build --workspace --release

# 2. Package Windows x86_64
Write-Host "[2/5] Packaging Windows x86_64..." -ForegroundColor Yellow
& powershell -ExecutionPolicy Bypass -File scripts/package_windows.ps1

# 3. Package Linux x86_64
Write-Host "[3/5] Packaging Linux x86_64 & AppImage..." -ForegroundColor Yellow
& powershell -ExecutionPolicy Bypass -File scripts/package_linux.ps1

# 4. Package macOS Universal
Write-Host "[4/5] Packaging macOS Universal & DMG..." -ForegroundColor Yellow
& powershell -ExecutionPolicy Bypass -File scripts/package_macos.ps1

# 5. Package Android & iOS Mobile
Write-Host "[5/5] Packaging Android APK/AAB & iOS IPA..." -ForegroundColor Yellow
& powershell -ExecutionPolicy Bypass -File scripts/package_android.ps1
& powershell -ExecutionPolicy Bypass -File scripts/package_ios.ps1

Write-Host "==========================================================" -ForegroundColor Green
Write-Host " [SUCCESS] All 5 platforms packaged successfully into dist/ " -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Green
