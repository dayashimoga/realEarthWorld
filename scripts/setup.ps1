# Real Earth Platform — Setup Script (PowerShell)
# Idempotent environment and dependency verification

$ErrorActionPreference = "Stop"

Write-Host "🌍 Real Earth World Platform — Environment Setup" -ForegroundColor Cyan

# 1. Verify Rust toolchain
$rustcVersion = rustc --version
if ($LASTEXITCODE -ne 0) {
    Write-Error "Rust toolchain is missing. Please install Rust from https://rustup.rs"
}
Write-Host "✅ Detected Rust: $rustcVersion" -ForegroundColor Green

# 2. Check cargo components (clippy, rustfmt)
Write-Host "🔍 Checking cargo components..." -ForegroundColor Yellow
rustup component add clippy rustfmt

# 3. Create cache and data directories
$dirs = @("target/world_bengaluru", "data/manifests", "data/schemas", "assets/bengaluru_sample")
foreach ($dir in $dirs) {
    if (-not (Test-Path $dir)) {
        New-Item -ItemType Directory -Path $dir -Force | Out-Null
        Write-Host "📁 Created directory: $dir" -ForegroundColor Gray
    }
}

Write-Host "🎉 Environment setup completed successfully!" -ForegroundColor Green
