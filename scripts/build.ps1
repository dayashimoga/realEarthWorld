# Real Earth Platform — Build Script (PowerShell)
$ErrorActionPreference = "Stop"

Write-Host "🔨 Building Real Earth World Platform (Release Profile)..." -ForegroundColor Cyan
cargo build --workspace --release

Write-Host "✅ Workspace build completed successfully!" -ForegroundColor Green
