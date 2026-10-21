# Real Earth Platform — Setup Script (PowerShell)
# Idempotent environment and dependency verification
$ErrorActionPreference = "Stop"

Write-Host "[SETUP] Real Earth World Platform - Environment Setup" -ForegroundColor Cyan

# 1. Verify Rust toolchain
$rustcVersion = rustc --version
if ($LASTEXITCODE -ne 0) {
    Write-Error "Rust toolchain is missing. Please install Rust from https://rustup.rs"
}
Write-Host "[OK] Detected Rust: $rustcVersion" -ForegroundColor Green

# 2. Check cargo components (clippy, rustfmt)
Write-Host "[CHECK] Verifying cargo components..." -ForegroundColor Yellow
rustup component add clippy rustfmt

# 3. Create cache and data directories
$dirs = @("target/world_bengaluru", "data/manifests", "data/schemas", "assets/bengaluru_sample")
foreach ($dir in $dirs) {
    if (-not (Test-Path $dir)) {
        New-Item -ItemType Directory -Path $dir -Force | Out-Null
        Write-Host "[DIR] Created directory: $dir" -ForegroundColor Gray
    }
}

Write-Host "[SUCCESS] Environment setup completed successfully!" -ForegroundColor Green
