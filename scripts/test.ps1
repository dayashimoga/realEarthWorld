# Real Earth Platform — Test Script (PowerShell)
# Full quality gate verification: fmt, clippy, unit, integration, and benchmark tests
$ErrorActionPreference = "Stop"

Write-Host "[TEST] Running Real Earth Quality Gate Verification..." -ForegroundColor Cyan

Write-Host "[1/5] Code Formatting (cargo fmt --check)..." -ForegroundColor Yellow
cargo fmt --check
if ($LASTEXITCODE -ne 0) { throw "Formatting check failed" }

Write-Host "[2/5] Static Analysis (cargo clippy -D warnings)..." -ForegroundColor Yellow
cargo clippy --workspace --all-targets -- -D warnings
if ($LASTEXITCODE -ne 0) { throw "Clippy check failed" }

Write-Host "[3/5] Full Workspace Test Suite (cargo test)..." -ForegroundColor Yellow
cargo test --workspace
if ($LASTEXITCODE -ne 0) { throw "Workspace unit tests failed" }

Write-Host "[4/5] Integration End-to-End Suite..." -ForegroundColor Yellow
cargo test -p realearth-tests
if ($LASTEXITCODE -ne 0) { throw "Integration tests failed" }

Write-Host "[5/6] Benchmark Smoke Suite (cargo bench -- --test)..." -ForegroundColor Yellow
cargo bench --bench geospatial_bench --bench compiler_bench -- --test
if ($LASTEXITCODE -ne 0) { throw "Benchmark test verification failed" }

Write-Host "[6/6] Security Audit (cargo audit)..." -ForegroundColor Yellow
cargo audit
if ($LASTEXITCODE -ne 0) { throw "Security audit check failed" }

Write-Host "[SUCCESS] ALL QUALITY GATES PASSED (100% test success, 0 warnings, 0 fmt errors, 0 security vulnerabilities)!" -ForegroundColor Green
