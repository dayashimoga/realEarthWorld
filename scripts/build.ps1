# Real Earth Platform — Build Script (PowerShell)
$ErrorActionPreference = "Stop"

Write-Host "[BUILD] Building Real Earth World Platform (Release Profile)..." -ForegroundColor Cyan
cargo build --workspace --release

Write-Host "[SUCCESS] Workspace build completed successfully!" -ForegroundColor Green
