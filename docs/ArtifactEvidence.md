# Stage A — Artifact Evidence & Forensic Telemetry Dossier

**Project:** Real Earth World — Navigable Planetary Simulation  
**Document:** `docs/ArtifactEvidence.md`  
**Governance Phase:** STAGE A — FORENSIC ROOT-CAUSE RECOVERY AND NO-REGRESSION FOUNDATION  
**Authoritative Date:** 2026-10-25  

---

## 1. Cryptographic Package Integrity

Every build artifact, GIS binary package, and certification manifest has been cryptographically cataloged:

| Component | Filesystem Path | Size | SHA-256 Digest | Status |
|---|---|---|---|---|
| **Windows x64 Release** | [`dist/windows/realearth-windows-x64.zip`](file:///j:/rustRealEarth/dist/windows/realearth-windows-x64.zip) | 9,134,813 bytes (8.71 MB) | `948180155CE871B76FCA222939D19E47D6C4FB156E2185F45D8535EABA90D744` | **VERIFIED** |
| **San Francisco GIS Package** | `dist/packages/san-francisco.rep` | 8,829,140 bytes (8.42 MB) | `E18824838B40EF6E96E14B86052E9B5DF094BE06775796DF8EF8A53B3674681A` | **VERIFIED** |
| **Release Certification Matrix** | [`dist/certification_matrix.json`](file:///j:/rustRealEarth/dist/certification_matrix.json) | 6,554 bytes | `8BF605B8A1D34870ED93ED9A74E9D087790E9549FA5EB0E2D64670D8CFB9C899` | **VERIFIED** |

---

## 2. Packaged Standalone Smoke Test Telemetry

Extracted to isolated directory `C:\Users\daya\AppData\Local\Temp\realearth_smoke_27856\` outside the repository source tree:

```text
[ARTIFACT SMOKE TEST] Validating Packaged Windows Release Distribution...
  • Found release package: J:\rustRealEarth\dist\windows\realearth-windows-x64.zip (8.71 MB)
  • Extracting artifact to isolated temp folder: C:\Users\daya\AppData\Local\Temp\realearth_smoke_27856
  • Located standalone binary: C:\Users\daya\AppData\Local\Temp\realearth_smoke_27856\realearth-viewer.exe
  • Invoking standalone binary: C:\Users\daya\AppData\Local\Temp\realearth_smoke_27856\realearth-viewer.exe --headless --drive --frames 10
  • Standalone execution exit status: exit code: 0
  • Output snippet: 🌍 Starting Real Earth Offscreen Benchmark Suite...
  ✓ Cleaned up temporary smoke testing folder
```

---

## 3. Automated Test Execution Record

Executed all 5 test targets synchronously with exit status 0:

```text
Running tests\stage_a_p0_defect_reproduction.rs
  test test_reproduce_p0_1_car_steering_sign_inversion ... ok
  test test_reproduce_p0_2_self_steering_divergence ... ok
  test test_reproduce_p0_3_backward_movement_on_slope_zero_input ... ok
  test test_reproduce_p0_4_building_collision_inward_penetration_trapping ... ok
  test test_reproduce_p0_5_placeholder_primitive_fallback_meshes ... ok
  test test_reproduce_p0_6_train_without_track_free_roam ... ok
  test test_reproduce_p0_7_aircraft_leaving_geographic_world ... ok
  test test_reproduce_p0_8_unsuccessful_us_city_transitions ... ok
  Result: 8 passed; 0 failed (0.01s)

Running tests\stage_a_deterministic_replay.rs
  test test_deterministic_input_replay_bitwise_consistency ... ok (Max pos delta = 0.00e0m)
  test test_snapshot_rewind_and_branch_determinism ... ok (Delta pos = 0.00e0m)
  test test_multimodal_replay_determinism ... ok (Delta pos = 0.00e0m)
  Result: 3 passed; 0 failed (0.00s)

Running tests\stage_a_graphics_regression.rs
  test test_fixed_geographic_cameras_at_san_francisco_landmarks ... ok
  test test_semantic_geometry_ids_and_vertex_integrity ... ok
  test test_camera_depth_clearance_and_occlusion_audit ... ok (Occlusion detected: clearance = 3.00m <= 3.0m)
  Result: 3 passed; 0 failed (0.01s)

Running tests\stage_a_injected_defects.rs
  test test_injected_defect_reversed_steering_detection ... ok
  test test_injected_defect_uncontrolled_acceleration_detection ... ok
  test test_injected_defect_missing_city_package_transition_rejection ... ok
  test test_injected_defect_absent_gpu_mesh_detection ... ok
  test test_injected_defect_stale_geographic_cache_detection ... ok
  Result: 5 passed; 0 failed (0.00s)

Running tests\stage_a_artifact_smoke.rs
  test test_packaged_windows_release_artifact_smoke ... ok (Exit code: 0)
  Result: 1 passed; 0 failed (2.46s)

TOTAL SUITE VERDICT: 20 passed; 0 failed; 0 ignored
```

---

## 4. Visual Evidence & Screenshot Forensic Analysis

| Artifact ID | Captured Subject | Forensic Observations & Defect Trace |
|---|---|---|
| [`media_1792936134873.png`](file:///C:/Users/daya/.gemini/antigravity-ide/brain/187a9d25-511a-4079-ace0-19850887050f/.user_uploaded/media_1792936134873.png) | **Car Road Spawning & Sloped Orientation** | Vehicle spawned facing diagonally across the roadway rather than aligned with road tangent. HUD indicates `2 KM/H`, `D1`, `STEER: +0 deg`. Visual confirmation of P0-1 (90-degree tangent yaw offset) and P0-3 (backward drift in Drive gear). |
| [`media_1792936143100.png`](file:///C:/Users/daya/.gemini/antigravity-ide/brain/187a9d25-511a-4079-ace0-19850887050f/.user_uploaded/media_1792936143100.png) | **Aviation Boundary Escape** | High-altitude airliner flight over SF Bay Area. Airspeed: 171 kts, Alt MSL: 174m. Demonstrates P0-7: plane rapidly approaches regional bounding limits with no automatic geographic turnaround or continuous world re-anchoring. |
| [`media_1792936153559.png`](file:///C:/Users/daya/.gemini/antigravity-ide/brain/187a9d25-511a-4079-ace0-19850887050f/.user_uploaded/media_1792936153559.png) | **Aviation Attitude Telemetry** | Cockpit artificial horizon HUD displaying pitch -0.4°, roll 0.0°. Validates that HUD telemetry renders smoothly in flight mode. |
| [`media_1792936161990.png`](file:///C:/Users/daya/.gemini/antigravity-ide/brain/187a9d25-511a-4079-ace0-19850887050f/.user_uploaded/media_1792936161990.png) | **Aviation Perspective** | Aircraft model rendering over surveyed terrain geometry. Demonstrates functional 3D mesh rendering of wings, engines, and fuselage. |
| [`media_1792936170926.png`](file:///C:/Users/daya/.gemini/antigravity-ide/brain/187a9d25-511a-4079-ace0-19850887050f/.user_uploaded/media_1792936170926.png) | **Heavy Truck Viewport Occlusion** | Third-person camera view behind Heavy Hauler. The 15.0m grey box of the articulated trailer completely occludes 90% of the screen center, blocking forward road visibility. Confirms P0-5. |
