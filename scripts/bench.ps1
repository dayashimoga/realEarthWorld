# Real Earth Platform — Benchmark Script (PowerShell)
$ErrorActionPreference = "Stop"

Write-Host "⚡ Running Real Earth Performance Benchmarks..." -ForegroundColor Cyan

# 1. CLI Throughput Benchmarks
cargo run -p realearth-cli -- bench

# 2. Criterion Microbenchmarks
cargo bench -p realearth-benchmarks

Write-Host "✅ Benchmarks completed successfully!" -ForegroundColor Green
