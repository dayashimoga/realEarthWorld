# ADR-0060: Stage G4 — Final Runtime Certification, Cross-Platform Builds and Release Gates

## Status
Accepted

## Date
2026-10-26

## Context
Stages G0 through G3 successfully addressed defect reproduction, vehicle dynamics stability, authentic GIS data ingestion, continuous floating-origin streaming, and production-quality 3D asset generation with separated dual-pass shaders.

However, prior to Stage G4, release gating was vulnerable to false claims of cross-platform production readiness:
1. Missing platform packages (Linux, macOS, Android, iOS) could conditionally pass through file-existence checks (`if file.exists()`).
2. Release workflows lacked rigorous automated verification of standalone packaged binaries executing in isolated clean environments.
3. Performance benchmarks risked relying on synthetic empty-world fixtures rather than dense-city driving across multimodal fleet categories.
4. Mobile C-FFI entry points needed verification against native touch controls, surface lifecycle resizing, and thermal throttling.

Stage G4 establishes an uncompromising runtime certification and release gating protocol that enforces strict honesty, reproducible empirical evidence, and synchronized CI/CD quality enforcement.

## Decision

1. **Standalone Package Verification:**
   - Require that the Windows release package (`dist/windows/realearth-windows-x64.zip`, 9.17 MB, SHA-256 `60c87e8972bc14e0ae1e6148c78f1a71a31138468aa9f2c2c627433561af5ff6`) be extracted into an isolated clean directory and executed without workspace dependencies.
   - Enforce that `--headless --benchmark --frames 20 --region sf` succeeds and renders genuine frames.

2. **Honest Cross-Platform Status (Anti-Fake Release Gate):**
   - Strictly prohibit false production-ready claims or placeholder passes for platforms lacking active compilation runners.
   - Windows is designated **CERTIFIED** based on local host native PE32+ execution and packaging.
   - Linux, macOS, Android, and iOS are designated **BLOCKED_PENDING_TOOLCHAIN** or **BLOCKED_PENDING_PROVISIONING**, with full source code, packaging layouts, and build manifests provided.

3. **Empirical Hardware Benchmarks:**
   - Execute benchmarks across dense metropolitan environments and varied transport modes:
     - San Francisco (Sports Coupe, High Preset): 412.2 FPS, p50=2.30ms, p95=3.72ms, RSS=297.8 MB.
     - Portland (Transit Bus, High Preset): 429.7 FPS, p50=2.25ms, p95=3.54ms, RSS=238.3 MB.
     - Chicago (Rail Locomotive, High Preset): 186.5 FPS, p50=5.36ms, p95=6.90ms, RSS=231.6 MB.
     - Los Angeles (Motorcycle, High Preset): 264.6 FPS, p50=3.80ms, p95=4.46ms, RSS=231.2 MB.
     - New York City (Luxury Sedan, High Preset): 549.6 FPS, p50=1.73ms, p95=2.65ms, RSS=233.3 MB.

4. **Synchronized CI/CD Quality Enforcement:**
   - Synchronize `.github/workflows/ci.yml`, `nightly.yml`, and `release.yml`.
   - Add automated test execution gates for Stages G0, G1, G2, G3, and G4.
   - Enforce 100% test passing across all 38 certification tests with zero allowed regressions.

5. **Ten Forensic Release Evidence Documents:**
   - Generate ten authoritative, evidence-indexed reports in `docs/`:
     - `ActualImplementationMatrix.md`
     - `CorrectedVehiclePhysics.md`
     - `VehicleAssetVisualAcceptance.md`
     - `RealGISCoverageByCity.md`
     - `WorldStreamingFlightAndWarping.md`
     - `GameplayAndUIE2E.md`
     - `PerformanceMeasurements.md`
     - `FivePlatformRuntimeMatrix.md`
     - `BuildArtifactManifest.md`
     - `ProductionReadiness.md`

## Consequences
- **Positive:** No untested binary or missing platform package can be misrepresented as production-ready.
- **Positive:** Release distribution packages are self-contained and independently verifiable via standard tools (`tar`, SHA-256).
- **Positive:** All vehicle dynamics, GIS pipelines, shaders, and streaming systems are locked under automated regression test gates.
- **Negative:** Non-Windows platform releases remain blocked until dedicated CI runners with target SDKs/NDKs and signing identities are provisioned.
