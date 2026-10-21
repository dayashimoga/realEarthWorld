# Real Earth Platform — Benchmark Script (PowerShell)
$ErrorActionPreference = "Stop"

Write-Host "[BENCH] Running Real Earth Performance Benchmarks..." -ForegroundColor Cyan

# 1. CLI Throughput Benchmarks
cargo run -p realearth-cli -- bench

# 2. Criterion Microbenchmarks Smoke Test
cargo bench --bench geospatial_bench --bench compiler_bench -- --test

Write-Host "[SUCCESS] Benchmarks completed successfully!" -ForegroundColor Green
