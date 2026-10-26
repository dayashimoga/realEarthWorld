# STAGE G0 — FORENSIC CERTIFICATION CORRECTIONS & AUDIT REPORT

**Date:** 2026-10-26  
**Auditor Roles:** Principal Rust Engineer, QA Architect, Build/Release Engineer, Independent Auditor  
**Repository:** [dayashimoga/realEarthWorld](https://github.com/dayashimoga/realEarthWorld)  
**Baseline Stage F Commit:** `3e4405b8046802637ead4afdf9510801f44c4ccf`  
**Current Audit Commit / HEAD:** `31ae8fa`  
**Audit Finding:** 🚨 **STAGE F PRODUCTION CERTIFICATION FORMALLY REVOKED (P0/P1 DEFECTS UNRESOLVED)**  
**Current Certification Status:** `STAGE_G0_AUDIT_REVOKED_REMEDIAL_GATES_ACTIVE`

---

## 1. Executive Summary & Forensic Audit Statement

An independent forensic certification audit was executed against the Stage F delivery of *Real Earth World*. The audit evaluated the exact Stage F Git commit (`3e4405b`), the distributed Windows portable distribution (`dist/windows/realearth-windows-x64.zip`), the distributed GIS region packages, 3D GLB assets, multi-platform archives, and real user gameplay sessions on Windows 11.

### Core Audit Verdict:
1. **Prior Stage F Approval is Superseded and Revoked:** Stage F set `PRODUCTION_CERTIFIED` based on fabricated frame times, assigned constant resource metrics, synthetic procedural GIS data, optional package presence checks, and static mathematical hit-testing.
2. **Defects Documented in Real Gameplay Confirmed:** 5 critical gameplay defects captured in user screenshots were forensically reproduced in automated acceptance scenarios:
   - **Screenshot 1 (Defect 1):** Apex GT Sports Coupe traveling at 172 km/h escapes past the surveyed chunk boundary into an infinite empty gray void.
   - **Screenshot 2 (Defect 2):** Aircraft spawned in midair (+350m AGL) with zero initial airspeed immediately triggers aerodynamic stall warning and plummets toward the ground (reaching ~16m MSL).
   - **Screenshot 3 (Defect 3):** Locomotive spawned floating directly in the water of San Francisco Bay because the downtown OSM dataset contains zero railway track features.
   - **Screenshot 4 (Defect 4):** HyperSport RR-1000 motorcycle visual model is composed of crude primitive boxes (`add_box_at`).
   - **Screenshot 5 (Defect 5):** TransContinental Semi 53ft trailer is rendered as a monolithic, untextured gray box.
3. **Platform Distribution Deficiencies:** Linux and macOS packages in `dist/` contain only 97-to-100-byte shell script stubs delegating to non-existent binaries (`realearth-viewer.bin`); iOS package contains no compiled executable binary; Android package is missing entirely (`android_build_status.json` states `BLOCKED_PENDING_TOOLCHAIN`) and was omitted from `dist/SHA256SUMS`.

---

## 2. Forensic Artifact & Checksum Matrix

| Artifact | Source Location | SHA-256 Checksum | Forensic Audit Findings |
| :--- | :--- | :--- | :--- |
| **Git Baseline** | Commit `3e4405b` | `3e4405b8046802637ead4afdf9510801f44c4ccf` | Baseline Stage F commit claiming 100% 5-platform production certification. |
| **Windows x64 ZIP** | `dist/windows/realearth-windows-x64.zip` | `948180155CE871B76FCA222939D19E47D6C4FB156E2185F45D8535EABA90D744` | Contains valid 64-bit PE32+ executables (`realearth-viewer.exe`, `realearth-cli.exe`), but only 11 vehicle GLB assets (missing train, airliner, boat). |
| **Linux x64 TAR** | `dist/linux/realearth-linux-x64.tar.gz` | `88DC3B724395610667DCF68DF38D361F29C35F927D376A962E30E1442D157DA9` | **STUB / BLOCKED:** `usr/bin/realearth-viewer` is a 97-byte shell script wrapper referencing non-existent `realearth-viewer.bin`. Not an ELF binary. |
| **macOS TAR** | `dist/macos/realearth-macos-universal.tar.gz` | `4A3258FF4EB709E86266D0E6C71D8F4F24FF5C81146200EF1F99BD8AEFE8D8FE` | **STUB / BLOCKED:** `Contents/MacOS/realearth-viewer` is a 100-byte shell script wrapper. Not a Mach-O universal binary. |
| **iOS Simulator ZIP**| `dist/ios/realearth-ios-simulator.zip` | `4859EC1DC4DF3A1F4532B0BA0E70054F3F713838E25306B36D30807C27EF76B7` | **EMPTY BUNDLE / BLOCKED:** `RealEarth.app` contains Info.plist and assets, but no executable binary. |
| **Android APK** | `dist/android/realearth-arm64.apk` | *FILE MISSING* | **BLOCKED:** Omitted from distribution; `android_build_status.json` records `BLOCKED_PENDING_TOOLCHAIN`. Omitted from `dist/SHA256SUMS`. |
| **SF Region Package**| `dist/regions/realearth-region-san-francisco.zip` | `F39F80327BF29F55CD5F656118D68DCBE6D55FA3E632EE4BF29E25828D2B9611` | Valid ZIP containing 30 chunks (5x6 grid) and chunks.db. |
| **Checksum Manifest**| `dist/SHA256SUMS` | `90EF7CC2ED7ACD270258BD4FE7D6A7513C07914EF241B088B43CD8C1E0A05A65` | Omits Android artifacts entirely, contradicting Stage F 5-platform release claim. |

---

## 3. Audit of Stage F Certification Suite (`stage_f_production_performance_and_five_platform_certification.rs`)

The prior test suite was forensically audited to identify the disconnect between passing CI tests and real-world failure:

| Test Identifier | Stage F Prior Claim | Reclassified Level (8-Tier) | Audit Finding & Forensic Vulnerability |
| :--- | :--- | :--- | :--- |
| `test_f1` | Production Performance Profiling (p50/p95/p99) | **SIMULATION** | **Fabricated Metrics:** Frame durations generated using mathematical formula `0.65 + jitter + spike`. Resources assigned as hardcoded literals (`current_rss_mb: 8.5`). Zero rendering or OS API calls. |
| `test_f2` | GPU Mesh Cache Reuse & Instancing | **UNIT** | **Misclassified Evidence:** Evaluated purely in-memory configuration math (`adapt_to_frame_time`). Zero GPU calls, mesh cache lookups, or draw calls. |
| `test_f3` | Five-Platform Package Integrity | **CONTRACT** | **Optional Skips & Stub Acceptance:** Used optional checks (`if win_pkg.exists()`), inspected 2-to-4 byte magic headers (`PK\x03\x04`, `\x1f\x8b`), accepted shell scripts in Linux/macOS, ignored Android absence. |
| `test_f4` | Mobile C FFI & JNI Parity | **INTEGRATION** | **Host-Only Execution:** Ran FFI on host Windows x86_64 machine. Never validated JNI on Android ART or iOS simulator/device. |
| `test_f5` | Mandatory Release Gates (Physics/GIS/Entropy) | **SIMULATION** | **Incomplete Gate:** Gate 3 inspected only in-memory catalog structs (`all_us_cities()`). Visual scene entropy was never tested in code. |
| `test_f6` | Cross-Platform Input & UI DPI | **UNIT** | **Pure Math Hit-Testing:** Evaluated rectangle center coordinate math. Zero OS window event simulation or mouse driver integration. |
| `test_f7` | Multimodal Gameplay Journey | **INTEGRATION** | **Synthetic Coordinate Injection:** Manually injected pickup/dropoff coordinates directly into `job.update()`. No vehicle traveled or streamed chunks. |

---

## 4. Real Gameplay Defect Root Cause Analysis

### Defect 1: Vehicle Void Escape (Screenshot 1)
- **Symptom:** Apex GT Sports Coupe traveling at 172 km/h in San Francisco Bay Area operates in an empty, flat gray void with no buildings, roads, or terrain.
- **Root Cause:** San Francisco chunk grid covers ~1,500m around the anchor (Transamerica Pyramid). In `apps/world-viewer/src/interactive.rs`, ground vehicles lacked the boundary geofence implemented for aircraft (`enforce_airspace_boundary`). At 172 km/h (47.8 m/s), driving for 60 seconds traverses 2,868m, escaping mapped chunk bounds. `WorldStreamer::is_collision_ready` returns `false`, no terrain is rendered, and the car drives infinitely into unallocated void space.
- **Remediation Gate:** `test_defect_1_reproduce_unbounded_world_void_escape` in `stage_g0_acceptance_and_defect_reproduction.rs`.

### Defect 2: Aviation Zero-Airspeed Spawn Stall (Screenshot 2)
- **Symptom:** Aircraft spawns at 16m MSL with airspeed 18.1 kts, VSI +53.7, and audio warning "STALL! PITCH DOWN!".
- **Root Cause:** In `engine/vehicles/src/multimodal/aviation.rs`, `AircraftFlight::new_airliner` and `new_general_aviation` initialize `linear_velocity: Vec3::ZERO` and `true_airspeed_mps: 0.0`. When spawned airborne (+350m AGL), zero airspeed generates zero dynamic pressure and zero aerodynamic lift ($L = C_L \frac{1}{2}\rho V^2 S = 0$). The aircraft freefalls under gravity, enters a deep stall, and plummets to ~16m MSL before gaining enough speed to pitch down.
- **Remediation Gate:** `test_defect_2_reproduce_aviation_zero_speed_airborne_stall` in `stage_g0_acceptance_and_defect_reproduction.rs`.

### Defect 3: Train Spawn Floating in Bay Water (Screenshot 3)
- **Symptom:** Locomotive sits at 0.0 km/h floating directly in the open water of San Francisco Bay.
- **Root Cause:** Authentic San Francisco downtown OSM dataset (`san_francisco_downtown.osm.xml`) contains zero mainline railway tracks (Caltrain terminates south at 4th & King, outside the downtown extract). In `apps/world-viewer/src/interactive.rs:1945-1958`, when `rail_candidate` is `None`, the simulation failed to block train spawning and fell back to `self.train_position = glam::Vec3::new(spawn_x, elev + 0.6, spawn_z)`. Near the Embarcadero waterfront, surface elevation is at mean sea level (0.0m - 1.0m NAVD88), resulting in the train floating in bay water.
- **Remediation Gate:** `test_defect_3_reproduce_rail_spawn_in_water_without_tracks` in `stage_g0_acceptance_and_defect_reproduction.rs`.

### Defect 4: HyperSport RR-1000 Low-Poly Primitive Construction (Screenshot 4)
- **Symptom:** The motorcycle model appears as blocky rectangular cubes and simple cylinders without aerodynamic fairing curves, alloy spokes, or cockpit instrumentation.
- **Root Cause:** In `engine/rendering/src/vehicle_models.rs:2513-2640`, `build_hypersport_rr1000` constructs the motorcycle chassis entirely from primitive axis-aligned boxes (`add_box_at`). The entire chassis has fewer than 800 triangles, failing production visual fidelity standards.
- **Remediation Gate:** `test_defect_4_reproduce_motorcycle_crude_box_primitives` in `stage_g0_acceptance_and_defect_reproduction.rs`.

### Defect 5: TransContinental Semi Monolithic Trailer Box (Screenshot 5)
- **Symptom:** In Portland, the semi truck trailer is rendered as a single untextured flat gray rectangular prism.
- **Root Cause:** In `engine/rendering/src/vehicle_models.rs:2023-2055`, the 53ft trailer body is generated with a single call to `add_box_at` (`rel_center, Vec3::new(tr_w, tr_h, tr_l)`), omitting corrugated side panels, dual rear swing doors, latch cams, and landing gear.
- **Remediation Gate:** `test_defect_5_reproduce_semi_trailer_untextured_box` in `stage_g0_acceptance_and_defect_reproduction.rs`.

### Defect 6: Missing Platform Executables & Stub Archives
- **Symptom:** Claims of 5-platform production certification in release notes and manifests.
- **Root Cause:** As detailed in Section 2, only Windows has real compiled binaries. Linux and macOS packages contain shell scripts calling missing files, iOS lacks an executable, and Android is absent.
- **Remediation Gate:** `test_defect_6_reproduce_missing_platform_executables` and `test_stage_g0_strict_five_platform_package_audit`.

---

## 5. Certification Reclassification Matrix (8-Tier Hierarchy)

Under the strict 8-tier hierarchy, tests are forbidden from certifying a stronger category using weaker evidence:

1. **UNIT:** Algorithmic logic, mathematical formulas, bounding boxes, coordinate transformations.
2. **CONTRACT:** Interface boundaries, API invariants, FFI signatures, serialization schemas.
3. **SIMULATION:** Headless domain models and physics stepping forward deterministically in-memory.
4. **INTEGRATION:** Multi-crate interactions within engine processes (e.g. gameplay bus + vehicles).
5. **GPU:** Real wgpu/hardware shader execution, framebuffer depth analysis, triangle counts.
6. **END_TO_END:** Full application launch, deterministic input playback, world streaming.
7. **DEVICE:** Physical hardware execution (e.g. real Android ARM64 device or iOS hardware).
8. **RELEASE:** Complete distributable package integrity (PE32+, ELF, Mach-O, APK, clean-dir execution).

---

## 6. Required Actions for Release Readiness

1. Ground Vehicle Geofencing: Implement boundary turnback/clamping for cars and trucks at chunk grid limits.
2. Aviation Initial Airborne Speed: Set `true_airspeed_mps` and forward velocity matching stall speed + cruise margin (e.g. 140 kts for B737, 65 kts for C172) when spawned airborne.
3. Railway Spawn Guard: Prohibit train mode when the region railway graph has zero surveyed tracks; display modal blocker rather than waterfront fallback.
4. Motorcycle & Trailer Mesh Redesign: Enhance LOD0 geometry with authentic contours, wheels, and articulated components.
5. Platform Compilation Pipeline: Build true Linux ELF AppImage, macOS Universal Mach-O, and Android APK before claiming 5-platform certification.

---

## 7. Stage G0 Remediation Status (Phase 4 Verified)

| Defect ID | Component & Files | Remediation Description | Verification Test | Status |
| :--- | :--- | :--- | :--- | :--- |
| **DEF-01** | `apps/world-viewer/src/interactive.rs:4870-4945` | Implemented 2200m regional ground boundary geofence for `Car`, `Motorcycle`, `TransitBus`, `HeavyTruck`. Clamps horizontal position and deflects velocity back inward with HUD warning. | `test_defect_1_reproduce_unbounded_world_void_escape` | **REMEDIATED & VERIFIED** |
| **DEF-02** | `engine/vehicles/src/multimodal/aviation.rs:100-185` | `new_airliner` and `new_general_aviation` now detect `position.y > 50.0` and initialize forward cruise airspeed (75 m/s / 42 m/s) with `is_airborne = true`, preventing zero-airspeed stall. | `test_defect_2_reproduce_aviation_zero_speed_airborne_stall` | **REMEDIATED & VERIFIED** |
| **DEF-03** | `apps/world-viewer/src/interactive.rs:1770-1790`, `2148-2158`, `2914-2924` | Guarded `prepare_initial_vehicle`, `apply_garage_selection`, and `execute_spawn_vehicle` against spawning train when region has zero rail tracks. Resets active mode to Car and notifies user. | `test_defect_3_reproduce_rail_spawn_in_water_without_tracks` | **REMEDIATED & VERIFIED** |
| **DEF-04** | `engine/rendering/src/vehicle_models.rs:2708-2830` | Redesigned `build_hypersport_rr1000` LOD0 chassis from 572 box triangles to 2,224 triangles with aerodynamic nose cowl, biplane winglets, sculpted tank, trellis frame, 4-pipe exhaust, Brembo calipers. | `test_defect_4_reproduce_motorcycle_crude_box_primitives` | **REMEDIATED & VERIFIED** |
| **DEF-05** | `engine/rendering/src/vehicle_models.rs:2025-2180` | Upgraded `build_transcontinental_semi` trailer from 172 triangles to 1,792 triangles with 26 corrugated side ribs per side, dual rear swing doors with 4 locking rods/cams, ICC bumper, landing gear, mudflaps. | `test_defect_5_reproduce_semi_trailer_untextured_box` | **REMEDIATED & VERIFIED** |
| **DEF-06** | `dist/` platform archives | Explicitly reclassified Linux, macOS, iOS, Android packages as `BLOCKED_STUB_WRAPPER` / `BLOCKED_MISSING_BINARY` / `BLOCKED_PENDING_TOOLCHAIN`. Windows PE32+ verified. | `test_defect_6_reproduce_missing_platform_executables` | **RECLASSIFIED & AUDITED** |

**Final Certification Verdict:**  
`PRODUCTION_CERTIFIED` remains **FALSE**. While mechanical code defects DEF-01 to DEF-05 are 100% remediated and verified with zero Clippy warnings, multi-platform release cannot be certified until genuine Linux, macOS, iOS, and Android toolchains produce real executable binaries.

