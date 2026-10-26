# ADR-0056: Stage G0 — Independent Production Certification Repair

## Status
**Accepted (Superseding ADR-0055 Production Approval)** (2026-10-26)

## Context
Following the Stage F delivery ([ADR-0055](file:///j:/rustRealEarth/docs/adr/0055-stage-f-production-performance-five-platform-certification.md)), real-world Windows gameplay testing exposed critical defects that contradicted the prior claim of production readiness:
1. **Vehicle Void Escape (Screenshot 1):** Apex GT Sports Coupe traveling at 172 km/h in San Francisco Bay Area escaped mapped chunk boundaries into an empty gray void.
2. **Aviation Zero-Airspeed Spawn Stall (Screenshot 2):** Aircraft spawned in midair (+350m AGL) with zero airspeed, triggering immediate aerodynamic stall warning and plummeting to ~16m MSL.
3. **Train Spawn in Bay Water (Screenshot 3):** Locomotive spawned floating directly in the open water of San Francisco Bay because downtown OSM lacks surveyed railway tracks.
4. **Motorcycle Primitive Box Model (Screenshot 4):** HyperSport RR-1000 was constructed entirely from crude axis-aligned boxes (`add_box_at`).
5. **Semi Trailer Monolithic Box (Screenshot 5):** TransContinental Semi 53ft trailer was generated as a single untextured flat box.
6. **Platform Distribution Stubs:** Linux and macOS packages contained shell scripts calling non-existent `.bin` files; iOS app bundle contained no executable; Android APK was missing and omitted from `dist/SHA256SUMS`.

An independent forensic audit revealed that Stage F achieved passing CI results using fabricated frame times (formula `0.65 + jitter + spike`), assigned memory literals (`current_rss_mb: 8.5`), optional platform archive checks (`if win_pkg.exists()`), synthetic geographic datasets, and static math hit-testing.

---

## Decisions

### 1. Formal Revocation of Stage F Production Approval
- The prior `PRODUCTION_CERTIFIED` status is formally superseded and revoked.
- Certification state is set to `STAGE_G0_AUDIT_REVOKED_REMEDIAL_GATES_ACTIVE`.
- The application shall NOT be certified for production while any P0 or P1 defects remain unresolved.

### 2. Mandatory 8-Tier Evidence Hierarchy
To eliminate false certifications, all tests and release claims must adhere to the 8-tier evidence hierarchy:
- `UNIT`: Isolated algorithms, math bounds, pure parsers.
- `CONTRACT`: Interface signatures, FFI types, serialization schemas.
- `SIMULATION`: Headless domain state machines stepping in-memory.
- `INTEGRATION`: Multi-crate interactions within engine processes.
- `GPU`: Hardware/driver wgpu rendering, shader pipelines, framebuffer depth analysis.
- `END_TO_END`: Real application execution with user input and subsystem integration.
- `DEVICE`: Physical target hardware execution (e.g. physical ARM64 device).
- `RELEASE`: Complete distributable package integrity (PE32+, ELF, Mach-O, APK, clean-dir launch).
- **Rule:** Never certify a stronger category using weaker evidence.

### 3. Reclassification of Stage F Test Suite
All tests in `tests/tests/stage_f_production_performance_and_five_platform_certification.rs` are formally reclassified with permanent audit banners:
- `test_f1`: Reclassified from RELEASE to **SIMULATION** (Fabricated frame durations & assigned constants).
- `test_f2`: Reclassified from GPU/RELEASE to **UNIT** (In-memory configuration math).
- `test_f3`: Reclassified from RELEASE to **CONTRACT** (File signatures and optional skips).
- `test_f4`: Reclassified from DEVICE to **INTEGRATION** (Host-only Windows C FFI execution).
- `test_f5`: Reclassified from RELEASE to **SIMULATION** (Headless stepping and catalog bounds).
- `test_f6`: Reclassified from END_TO_END to **UNIT** (Static math hit-testing).
- `test_f7`: Reclassified from END_TO_END to **INTEGRATION** (Manually injected coordinates).

### 4. Defect Reproduction Acceptance Suite (`tests/tests/stage_g0_acceptance_and_defect_reproduction.rs`)
Created 6 automated reproduction tests that isolate and verify the root causes of each screenshot defect:
- `test_defect_1_reproduce_unbounded_world_void_escape`
- `test_defect_2_reproduce_aviation_zero_speed_airborne_stall`
- `test_defect_3_reproduce_rail_spawn_in_water_without_tracks`
- `test_defect_4_reproduce_motorcycle_crude_box_primitives`
- `test_defect_5_reproduce_semi_trailer_untextured_box`
- `test_defect_6_reproduce_missing_platform_executables`

### 5. Deterministic Gameplay Input Recording & Replay (`realearth_shared::gameplay_recording`)
- Implemented synchronized recording and replay of raw keys, mouse buttons, vehicle state, transforms, camera poses, collision dynamics, and geographic context.
- Verified bitwise deterministic replay with $< 1\text{mm}$ position drift in `tests/tests/stage_g0_real_executable_validation.rs`.

### 6. Clean Working Directory Standalone Launch Verification
- In `tests/tests/stage_g0_real_executable_validation.rs`, extracted `dist/windows/realearth-windows-x64.zip` into an isolated clean temporary directory.
- Verified standalone launch and headless execution with zero access to source-tree paths, confirming genuine portability on Windows.

### 7. Real GPU Scene Depth & Semantic Regression Gate
- Built `SceneValidationMetrics` requiring $> 500$ triangles and $> 0.01$ depth buffer variance.
- Rejects empty void scenes and HUD-only viewports despite high 2D image entropy.

### 8. Mutation Testing Release Gates
- Implemented mutation tests injecting:
  - Inverted steering input (asserts angular divergence $> 0.1\text{ rad}$).
  - Continuous stuck throttle (asserts failure of $< 0.1\text{ km/h}$ stopping gate).
  - Missing city data (asserts explicit `Err` return).

### 9. Measured Telemetry over Hardcoded Samples
- Replaced synthetic pacing with real wall-clock elapsed durations (`Instant::now()`).
- Replaced hardcoded memory constants with OS resident set size (`get_current_process_memory()`).

### 10. Strict Multi-Platform Package Integrity (No Silent Skips)
- Linux: `BLOCKED_STUB_WRAPPER` (Shell script wrapper referencing non-existent ELF binary).
- macOS: `BLOCKED_STUB_WRAPPER` (Shell script wrapper referencing non-existent Mach-O binary).
- iOS: `BLOCKED_MISSING_BINARY` (App bundle contains no executable).
- Android: `BLOCKED_PENDING_TOOLCHAIN` (APK missing, omitted from `dist/SHA256SUMS`).
- Only Windows x86_64 has a verified standalone native PE32+ executable.

---

## Consequences & Next Steps

1. **Integrity Restored:** Production certification claims are now backed by measured evidence and strict validation.
2. **Clear Defect Ledger:** Remedial targets are clearly isolated and guarded by failing/acceptance tests.
3. **Release Gate Enforcement:** CI/CD will reject release builds until all platform binaries are natively compiled and gameplay defects are remediated.
