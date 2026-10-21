# Real Earth Platform — Container Teardown Script (PowerShell)
$ErrorActionPreference = "Stop"

Write-Host "[CONTAINER] Tearing down Real Earth containerized environment..." -ForegroundColor Cyan

$composeCmd = $null
if (Get-Command podman-compose -ErrorAction SilentlyContinue) {
    $composeCmd = "podman-compose"
} elseif (Get-Command docker -ErrorAction SilentlyContinue) {
    $composeCmd = "docker compose"
} elseif (Get-Command podman -ErrorAction SilentlyContinue) {
    $composeCmd = "podman compose"
}

if ($null -eq $composeCmd) {
    Write-Host "[INFO] No container compose runtime found. Nothing to tear down." -ForegroundColor Gray
    exit 0
}

Write-Host "[RUN] Executing: $composeCmd down -v" -ForegroundColor Yellow
Invoke-Expression "$composeCmd down -v"

Write-Host "[SUCCESS] Containers torn down successfully!" -ForegroundColor Green
