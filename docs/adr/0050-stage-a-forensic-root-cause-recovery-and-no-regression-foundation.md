# ADR-0050: Stage A — Forensic Root-Cause Recovery and No-Regression Foundation

## Status
**Accepted** (2026-10-25)

## Context
Across previous development cycles, several critical vehicle dynamics, rendering, collision, and geographic defects reappeared or suffered incomplete fixes:
1. Car steering right when KeyA is pressed, alongside a 90-degree frame shift on road spawn.
2. Self-steering yaw divergence under zero input when moving down slopes.
3. Backward rollback on realistic hill slopes (e.g. San Francisco 8% grade) while in Drive (D1), which subsequently caused pressing `W` to trigger reverse service braking.
4. Building collision penetration calculating inward normals that permanently trap vehicles inside buildings.
5. Placeholder primitive fallback meshes used instead of glTF 3D assets, and heavy truck trailers occluding 90% of the chase camera viewport.
6. Trains accelerating and driving over terrain freely without railway track infrastructure.
7. Aircraft escaping the 2,500m geographic package boundary into void space within 15 seconds.
8. US city transitions failing for 32 of 38 catalog cities due to unbundled GIS packages on disk.

To permanently break the cycle of recurring regressions, Stage A mandates a strict forensic recovery process: baseline preservation, independent deterministic reproduction, machine-readable root-cause tracing, and a permanent automated test infrastructure prior to implementing additional features.

## Decision

1. **Non-Destructive Recovery Baseline**:
   - Preserved HEAD commit `d85d8677e5d87171d3d63bca2d897f1f41489ecf` on recovery branch `recovery/stage_a_forensic_foundation`.
   - Recorded compiler versions: `rustc 1.99.1`, `cargo 1.99.1`.
   - Recorded SHA-256 digests and byte sizes of release artifacts: `dist/windows/realearth-windows-x64.zip` (`948180155CE871B76FCA222939D19E47D6C4FB156E2185F45D8535EABA90D744`, 9,134,813 bytes).

2. **Full End-to-End Defect Reproduction**:
   - Authored [`tests/tests/stage_a_p0_defect_reproduction.rs`](file:///j:/rustRealEarth/tests/tests/stage_a_p0_defect_reproduction.rs) providing independent, deterministic automated reproduction of all 8 P0 defects.
   - Traced each bug through `event input` $\to$ `authoritative state` $\to$ `simulation` $\to$ `geographic collision` $\to$ `rendering` $\to$ `HUD`.

3. **Machine-Readable Defect Governance**:
   - Created [`docs/RootCauseMatrix.json`](file:///j:/rustRealEarth/docs/RootCauseMatrix.json) and [`docs/RootCauseMatrix.md`](file:///j:/rustRealEarth/docs/RootCauseMatrix.md) establishing exact source lines, symptoms, mathematical causes, and remediation strategies for all 8 defects.

4. **Permanent Regression Test Infrastructure**:
   - **Deterministic Input Replay:** [`tests/tests/stage_a_deterministic_replay.rs`](file:///j:/rustRealEarth/tests/tests/stage_a_deterministic_replay.rs) evaluating fixed 60Hz input sequences (300 frames), snapshot rewind and branching invariance, achieving $\Delta pos = 0.00\text{e}0\text{m}$.
   - **Graphics Regression Harness:** [`tests/tests/stage_a_graphics_regression.rs`](file:///j:/rustRealEarth/tests/tests/stage_a_graphics_regression.rs) auditing fixed landmark cameras, NDC projections, semantic geometry IDs, and viewport trailer clearances.
   - **Injected Defects Harness:** [`tests/tests/stage_a_injected_defects.rs`](file:///j:/rustRealEarth/tests/tests/stage_a_injected_defects.rs) intentionally injecting reversed steering, phantom throttle in Park, missing city packages, empty GPU meshes, and stale geographic cache, verifying that all 5 injected failure modes are caught.
   - **Artifact Smoke Testing:** [`tests/tests/stage_a_artifact_smoke.rs`](file:///j:/rustRealEarth/tests/tests/stage_a_artifact_smoke.rs) extracting packaged Windows zip into an isolated clean directory and validating standalone headless execution (`realearth-viewer.exe --headless --drive --frames 10`) with exit code 0.

5. **Governance Documentation**:
   - Published [`docs/RecoveryContract.md`](file:///j:/rustRealEarth/docs/RecoveryContract.md), [`docs/RegressionScenarios.md`](file:///j:/rustRealEarth/docs/RegressionScenarios.md), [`docs/ArtifactEvidence.md`](file:///j:/rustRealEarth/docs/ArtifactEvidence.md), and [`docs/StageAcceptance.md`](file:///j:/rustRealEarth/docs/StageAcceptance.md).

## Consequences
- **Traceability:** Distributed release packages are cryptographically tied to verified Git commits and authentic GIS packages.
- **Regression Protection:** All future feature changes must pass the 5 Stage A test suites before acceptance.
- **Readiness:** Stage A is fully satisfied, unlocking Stage B for targeted remediation.
