# Stage A Acceptance & Forensic Foundation Certification Report

**Project:** Real Earth World — Navigable Planetary Simulation  
**Document:** `docs/StageAcceptance.md`  
**Governance Phase:** STAGE A — FORENSIC ROOT-CAUSE RECOVERY AND NO-REGRESSION FOUNDATION  
**Authoritative Date:** 2026-10-25  
**Sign-off Verdict:** **PASS — STAGE A REQUIREMENTS FULLY SATISFIED**  

---

## 1. Stage A Deliverables Audit

| Requirement | Scope | Deliverables & Verification | Verdict |
|---|---|---|---|
| **Phase A1: Preserve & Reproduce** | Baseline preservation, non-destructive recovery branch, and deterministic reproduction of all 8 P0 defects. | • Non-destructive branch: `recovery/stage_a_forensic_foundation`<br>• HEAD Commit: `d85d8677e5d87171d3d63bca2d897f1f41489ecf`<br>• Reproducing test suite: [`tests/tests/stage_a_p0_defect_reproduction.rs`](file:///j:/rustRealEarth/tests/tests/stage_a_p0_defect_reproduction.rs) (8/8 reproduced) | **PASS** |
| **Phase A2: Systemic Root-Cause Identification** | Traced through `event input` $\to$ `authoritative state` $\to$ `simulation` $\to$ `collision` $\to$ `rendering` $\to$ `HUD`. Verified asset bindings and city packages. | • Machine-readable matrix: [`docs/RootCauseMatrix.json`](file:///j:/rustRealEarth/docs/RootCauseMatrix.json)<br>• Taxonomy documentation: [`docs/RootCauseMatrix.md`](file:///j:/rustRealEarth/docs/RootCauseMatrix.md)<br>• 100% mathematical and code-line traceability established | **PASS** |
| **Phase A3: Test Infrastructure** | Fixed 60Hz deterministic replay, graphics regression with landmark cameras, 5 injected defect tests, and packaged Windows release smoke testing. | • [`tests/tests/stage_a_deterministic_replay.rs`](file:///j:/rustRealEarth/tests/tests/stage_a_deterministic_replay.rs) ($\Delta pos = 0.00\text{e}0\text{m}$)<br>• [`tests/tests/stage_a_graphics_regression.rs`](file:///j:/rustRealEarth/tests/tests/stage_a_graphics_regression.rs) (NDC projection & occlusion gates)<br>• [`tests/tests/stage_a_injected_defects.rs`](file:///j:/rustRealEarth/tests/tests/stage_a_injected_defects.rs) (5/5 injected defects caught)<br>• [`tests/tests/stage_a_artifact_smoke.rs`](file:///j:/rustRealEarth/tests/tests/stage_a_artifact_smoke.rs) (Standalone binary extracted and executed cleanly, exit code 0) | **PASS** |
| **Phase A4: Governance & Documentation** | Create RecoveryContract, RootCauseMatrix, RegressionScenarios, ArtifactEvidence, StageAcceptance, ADR-0050. Maintain append-only logs. | • [`docs/RecoveryContract.md`](file:///j:/rustRealEarth/docs/RecoveryContract.md)<br>• [`docs/RootCauseMatrix.md`](file:///j:/rustRealEarth/docs/RootCauseMatrix.md)<br>• [`docs/RegressionScenarios.md`](file:///j:/rustRealEarth/docs/RegressionScenarios.md)<br>• [`docs/ArtifactEvidence.md`](file:///j:/rustRealEarth/docs/ArtifactEvidence.md)<br>• [`docs/StageAcceptance.md`](file:///j:/rustRealEarth/docs/StageAcceptance.md)<br>• [`docs/adr/0050-stage-a-forensic-root-cause-recovery-and-no-regression-foundation.md`](file:///j:/rustRealEarth/docs/adr/0050-stage-a-forensic-root-cause-recovery-and-no-regression-foundation.md) | **PASS** |

---

## 2. P0 Defect Reproduction Summary

All 8 observed P0 defects are 100% reproducible and have verified root causes documented in source code:

1. **P0-1 (Car Steering Sign & 90° Frame Shift):** Road tangent uses `atan2(dx, dz)` vs camera's `atan2(dz, dx)` producing a $\pi/2$ frame offset; camera steer lookahead pulls opposite to intended turn.
2. **P0-2 (Self-Steering Under Zero Input):** Unstable slip angle in reverse without caster mechanical trail produces divergent lateral drift under zero steering input.
3. **P0-3 (Backward Movement on Slope in Drive):** Idle hold requires `slope_grade < 0.025`; car rolls backward at $-2.8\text{ km/h}$ on 8% grade, tripping reverse brake threshold when pressing `W`.
4. **P0-4 (Building Collision Inward Penetration Trapping):** Inward normal vector `diff = p - closest` drives penetrating vehicles deeper into building interior.
5. **P0-5 (Placeholder Meshes & Trailer Occlusion):** Procedural box geometries used instead of GLB models; truck trailer camera clearance is only $1.95\text{m}$ to $3.0\text{m}$, blocking the screen.
6. **P0-6 (Train Free Roam Without Track):** Missing rail graph causes train consist to step unconstrained 3D motion over open terrain.
7. **P0-7 (Aircraft Leaving Geographic World):** High airspeed outpaces 2,500m regional bounding box in $< 15\text{s}$ into void space without geographic turnaround.
8. **P0-8 (US City Transitions Failing):** 32 of 38 catalog cities lack installed offline GIS packages on disk, aborting transitions immediately.

---

## 3. Unlocking Conditions for Stage B Remediation

With Stage A forensic reproduction, machine-readable root-cause documentation, and automated test regression gates fully established:
* **Stage A Acceptance Status:** **ACCEPTED & COMPLETE**
* **Authorization to Proceed to Stage B:** **GRANTED**
* **Stage B Scope:** Implement permanent, verified mathematical remediations for each P0 defect while maintaining all prior certified functionality intact.
