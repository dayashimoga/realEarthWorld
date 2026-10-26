# STAGE G0 — REQUIREMENT VERIFICATION MATRIX (RVM)

**Audit Version:** Stage G0 Independent Production Certification Repair  
**Date:** 2026-10-26  
**Taxonomy:** Strict 8-Tier Hierarchy (`UNIT`, `CONTRACT`, `SIMULATION`, `INTEGRATION`, `GPU`, `END_TO_END`, `DEVICE`, `RELEASE`)  
**Certification Status:** `STAGE_G0_AUDIT_REVOKED_REMEDIAL_GATES_ACTIVE`

---

## 1. 8-Tier Evidence Hierarchy Rules

To eliminate certification fraud and misleading quality claims:
1. **Never certify a stronger category using weaker evidence.** (e.g. A `SIMULATION` test using synthetic math loops cannot certify `RELEASE` performance; a `CONTRACT` header check cannot certify `DEVICE` readiness).
2. **Missing platform packages must be marked explicitly `BLOCKED`, never silently skipped.**
3. **Performance numbers must be measured wall-clock and OS memory values, never fabricated constants.**

---

## 2. Requirement Verification & Reclassification Table

| Req ID | Requirement Description | Stage F Prior Claim | Stage G0 Reclassified Level | Verification Test & Evidence Target | Current Audit Status | Audit Notes & Remedial Requirements |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **REQ-PERF-01** | Desktop 60 FPS frame pacing ($p_{95} \le 16.67\text{ms}$) | RELEASE | **SIMULATION** (Prior) / **END_TO_END** (G0) | `tests/tests/stage_g0_measured_performance_and_platform_evidence.rs::test_stage_g0_measured_presentation_and_resource_telemetry` | **PASS (Measured)** | Prior Stage F used synthetic loop `0.65 + jitter + spike`. Stage G0 measures real `Instant::now()` wall-clock timestamps. |
| **REQ-PERF-02** | Process RSS Memory budget ($\le 512\text{ MB}$) | RELEASE | **SIMULATION** (Prior) / **END_TO_END** (G0) | `tests/tests/stage_g0_measured_performance_and_platform_evidence.rs::test_stage_g0_measured_presentation_and_resource_telemetry` | **PASS (Measured)** | Prior Stage F assigned literal `current_rss_mb: 8.5`. Stage G0 queries real OS memory via `get_current_process_memory()`. |
| **REQ-PERF-03** | Cold startup wall-clock latency ($\le 200\text{ms}$) | RELEASE | **SIMULATION** (Prior) / **RELEASE** (G0) | `tests/tests/stage_g0_measured_performance_and_platform_evidence.rs::test_stage_g0_measured_presentation_and_resource_telemetry` | **PASS (Measured)** | Real wall-clock elapsed startup time measured. |
| **REQ-PKG-WIN** | Windows x86_64 Portable Distribution | RELEASE | **RELEASE** | `tests/tests/stage_g0_real_executable_validation.rs::test_stage_g0_clean_working_directory_execution` | **PASS** | Validated PE32+ AMD64 binary in extracted standalone directory without source-tree fallback. |
| **REQ-PKG-LNX** | Linux x86_64 AppImage Package | RELEASE | **CONTRACT** | `tests/tests/stage_g0_measured_performance_and_platform_evidence.rs::test_stage_g0_strict_five_platform_package_audit` | **BLOCKED** | Shell script wrapper calling non-existent `realearth-viewer.bin`. Real ELF missing. |
| **REQ-PKG-MAC** | macOS Universal Application Bundle | RELEASE | **CONTRACT** | `tests/tests/stage_g0_measured_performance_and_platform_evidence.rs::test_stage_g0_strict_five_platform_package_audit` | **BLOCKED** | Shell script wrapper in `RealEarth.app/Contents/MacOS`. Real Mach-O missing. |
| **REQ-PKG-IOS** | iOS Simulator Application Bundle | RELEASE | **CONTRACT** | `tests/tests/stage_g0_measured_performance_and_platform_evidence.rs::test_stage_g0_strict_five_platform_package_audit` | **BLOCKED** | App bundle contains no binary executable. |
| **REQ-PKG-AND** | Android ARM64 APK/AAB Package | RELEASE | **RELEASE** | `tests/tests/stage_g0_measured_performance_and_platform_evidence.rs::test_stage_g0_strict_five_platform_package_audit` | **BLOCKED** | File missing from `dist/android/`; `android_build_status.json` states toolchain blocked. Omitted from `dist/SHA256SUMS`. |
| **REQ-FFI-MOB** | Mobile C FFI & JNI Contract Parity | DEVICE | **INTEGRATION** | `tests/tests/stage_f_production_performance_and_five_platform_certification.rs::test_f4_mobile_c_ffi_and_jni_parity_contract` | **PASS (Host-Side Only)** | Validates C FFI export symbols on host Windows x86_64. Does not validate physical mobile execution. |
| **REQ-PHYS-01** | Vehicle Physics Slope & Parking Invariance | SIMULATION | **SIMULATION** | `tests/tests/stage_f_production_performance_and_five_platform_certification.rs::test_f5_automated_release_gates_physics_gis_and_entropy` | **PASS** | In-memory physics stepping verified caster stability and parking lock. |
| **REQ-INPUT-01** | Simulated Real Input: Steering & Stopping | END_TO_END | **END_TO_END** | `tests/tests/stage_g0_real_executable_validation.rs::test_stage_g0_simulated_real_input_steering_and_stopping` | **PASS** | Dynamic vehicle responds to real steering yaw delta and stops to $< 0.1\text{ km/h}$ under brake. |
| **REQ-REC-01** | Deterministic Input Recording & Replay | END_TO_END | **END_TO_END** | `tests/tests/stage_g0_real_executable_validation.rs::test_stage_g0_deterministic_gameplay_recording_and_replay` | **PASS** | Bitwise deterministic recording and replay of keys, transforms, camera, and collision. |
| **REQ-SCENE-01**| Real GPU Scene Depth & Entropy Regression | GPU | **GPU** | `tests/tests/stage_g0_real_executable_validation.rs::test_stage_g0_gpu_scene_regression_reject_hud_only_and_void` | **PASS** | Rejects HUD-only text over flat void despite high 2D entropy; requires $> 500$ triangles and depth variance. |
| **REQ-MUT-01**  | Mutation Testing: Reversed Steering Gate | SIMULATION | **SIMULATION** | `tests/tests/stage_g0_real_executable_validation.rs::test_stage_g0_mutation_testing_reversed_steering` | **PASS** | Properly triggers failure when inverted steering mutation is injected. |
| **REQ-MUT-02**  | Mutation Testing: Continuous Throttle Gate| SIMULATION | **SIMULATION** | `tests/tests/stage_g0_real_executable_validation.rs::test_stage_g0_mutation_testing_continuous_stuck_throttle` | **PASS** | Properly triggers braking failure gate when stuck throttle is injected. |
| **REQ-MUT-03**  | Mutation Testing: Missing City Data Gate | CONTRACT | **CONTRACT** | `tests/tests/stage_g0_real_executable_validation.rs::test_stage_g0_mutation_testing_missing_city_data` | **PASS** | Rejects fake-success for non-existent city ID. |
| **DEF-01**      | Geofence Escape into Infinite Void | END_TO_END | **END_TO_END** | `tests/tests/stage_g0_acceptance_and_defect_reproduction.rs::test_defect_1_reproduce_unbounded_world_void_escape` | **REMEDIATED & VERIFIED** | Implemented 2200m regional ground boundary geofence in `interactive.rs:4870-4945` clamping Car, Motorcycle, TransitBus, HeavyTruck. |
| **DEF-02**      | Aircraft Midair Zero-Airspeed Spawn Stall | SIMULATION | **SIMULATION** | `tests/tests/stage_g0_acceptance_and_defect_reproduction.rs::test_defect_2_reproduce_aviation_zero_speed_airborne_stall` | **REMEDIATED & VERIFIED** | Initialized safe flying airspeed (75 m/s airliner, 42 m/s GA) on airborne spawn in `aviation.rs:100-185`. |
| **DEF-03**      | Locomotive Floating in Water Without Tracks| INTEGRATION | **INTEGRATION** | `tests/tests/stage_g0_acceptance_and_defect_reproduction.rs::test_defect_3_reproduce_rail_spawn_in_water_without_tracks` | **REMEDIATED & VERIFIED** | Guarded train mode against spawn without surveyed railway tracks in `interactive.rs:1770-1790`, `2148-2158`, `2914-2924`. |
| **DEF-04**      | HyperSport RR-1000 Primitive Box Chassis | GPU | **UNIT** | `tests/tests/stage_g0_acceptance_and_defect_reproduction.rs::test_defect_4_reproduce_motorcycle_crude_box_primitives` | **REMEDIATED & VERIFIED** | Redesigned LOD0 chassis with aerodynamic fairings, winglets, trellis frame, and 4-pipe exhaust (2,224 triangles). |
| **DEF-05**      | TransContinental Semi Monolithic Trailer Box | GPU | **UNIT** | `tests/tests/stage_g0_acceptance_and_defect_reproduction.rs::test_defect_5_reproduce_semi_trailer_untextured_box` | **REMEDIATED & VERIFIED** | Upgraded 53ft trailer with 26 embossed corrugated ribs per side, dual swing doors, ICC bumper, landing gear (1,792 triangles). |

---

## 3. Summary of Gate Status

- **Automated Tests Passing:** 100% of Stage G0 forensic defect reproduction, real executable validation, and measured performance tests pass.
- **Defects Remediated:** All 5 screenshot defects (DEF-01 to DEF-05) have been verified and remediated in production engine code.
- **Production Readiness:** `UNVERIFIED / BLOCKED (P0 Packaging Gates Active)`. While the Windows standalone PE32+ application is verified in clean directory execution, the overall release CANNOT be marked `PRODUCTION_CERTIFIED` because Linux, macOS, iOS, and Android distributions remain blocked stubs. Truth in advertising is maintained.
