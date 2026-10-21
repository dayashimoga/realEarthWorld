# Real Earth Platform — Cleanup Script (PowerShell)
$ErrorActionPreference = "Stop"

Write-Host "[CLEAN] Cleaning Real Earth build artifacts and cached worlds..." -ForegroundColor Cyan

cargo clean

if (Test-Path "target/world_bengaluru") {
    Remove-Item -Recurse -Force "target/world_bengaluru"
    Write-Host "[CLEAN] Removed target/world_bengaluru" -ForegroundColor Gray
}

Write-Host "[SUCCESS] Workspace cleaned successfully!" -ForegroundColor Green
