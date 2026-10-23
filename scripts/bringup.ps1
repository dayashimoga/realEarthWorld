# Real Earth Platform — Container Bring-up Script (PowerShell)
# Starts headless compilation and validation services via Podman or Docker Compose
$ErrorActionPreference = "Stop"

Write-Host "[CONTAINER] Starting Real Earth containerized environment..." -ForegroundColor Cyan

# Detect container runtime: podman or docker
$composeCmd = $null
if (Get-Command podman-compose -ErrorAction SilentlyContinue) {
    $composeCmd = "podman-compose"
} elseif (Get-Command docker -ErrorAction SilentlyContinue) {
    $composeCmd = "docker compose"
} elseif (Get-Command podman -ErrorAction SilentlyContinue) {
    $composeCmd = "podman compose"
} elseif (Test-Path "$env:LOCALAPPDATA\Programs\Podman\podman.exe") {
    $composeCmd = "& `"$env:LOCALAPPDATA\Programs\Podman\podman.exe`" compose"
}

if ($null -eq $composeCmd) {
    Write-Host "[WARN] Neither Podman Compose nor Docker Compose found on PATH." -ForegroundColor Yellow
    Write-Host "[INFO] Skipping container orchestration. Native headless builds remain operational via scripts/build.ps1." -ForegroundColor Gray
    exit 0
}

Write-Host "[RUN] Using compose runtime: $composeCmd" -ForegroundColor Green
Invoke-Expression "$composeCmd up --build -d"

Write-Host "[SUCCESS] Containers brought up successfully!" -ForegroundColor Green
