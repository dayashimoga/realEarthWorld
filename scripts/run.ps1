# Real Earth Platform — Run Script (PowerShell)
$ErrorActionPreference = "Stop"

Write-Host "🌍 Launching Real Earth World Platform..." -ForegroundColor Cyan

# 1. Compile world package if missing
if (-not (Test-Path "target/world_bengaluru/chunks.db")) {
    Write-Host "📦 Compiling Bengaluru Central world package..." -ForegroundColor Yellow
    cargo run -p realearth-cli -- compile --output target/world_bengaluru
}

# 2. Launch World Viewer
Write-Host "🚀 Launching Real Earth World Viewer..." -ForegroundColor Green
cargo run -p realearth-viewer
