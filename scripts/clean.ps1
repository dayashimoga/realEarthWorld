# Real Earth Platform — Cleanup Script (PowerShell)
$ErrorActionPreference = "Stop"

Write-Host "🧹 Cleaning Real Earth build artifacts and cached worlds..." -ForegroundColor Cyan

cargo clean

if (Test-Path "target/world_bengaluru") {
    Remove-Item -Recurse -Force "target/world_bengaluru"
    Write-Host "🗑️ Removed target/world_bengaluru" -ForegroundColor Gray
}

Write-Host "✅ Workspace cleaned successfully!" -ForegroundColor Green
