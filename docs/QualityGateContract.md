# STAGE H0 — FORMAL QUALITY GATE & GOVERNANCE CONTRACT
**Project:** Real Earth World Simulation  
**Document:** QualityGateContract.md  
**Effective Date:** October 26, 2026  
**Auditing Authority:** Principal Rust, Graphics, GIS, Simulation, QA & Release Board  
**Enforcement Level:** **MANDATORY & NON-BYPASSABLE FOR ALL SUBSEQUENT REPAIR STAGES (H1–H6)**

---

## 1. Purpose & Core Principles

This Quality Gate Contract establishes permanent, non-negotiable engineering standards to eliminate false certifications, mock validations, and unverified release claims.

In earlier stages (G0–G4), certification reports prematurely declared complete platform readiness while the actual Windows game executable suffered from reversed steering, disappearing worlds, invalid terrain, floating trains, geometric box vehicles, and broken city transitions.

Under this contract:
1. **No production certification or "PASS ✅" claim may be issued while ANY defect in `ActualGapMatrix.md` remains reproducible or any test in `stage_h0_forensic_runtime_quality_gates.rs` fails.**
2. **Every bug fix MUST be backed by a deterministic test that demonstrably FAILS before the fix and PASSES after the fix.**
3. **Compilation success, catalog counts, direction-agnostic assertions, and mock data are permanently rejected as substitutes for runtime quality.**

---

## 2. The 5 Mandatory Runtime Quality Gates

All CI runs, release builds, and certification audits MUST execute `tests/tests/stage_h0_forensic_runtime_quality_gates.rs` and achieve 100% PASS across the following 5 gates:

### Gate 1: Steering & Camera Directional Conformance (`test_h0_gate_steering_directional_conformance`)
- **Requirement:** Pressing `KeyA` (Steer/Strafe Left) or `ArrowLeft` (Rotate Left) while moving forward or looking North MUST deflect vehicle/camera displacement and heading toward the West (-X / negative longitude).
- **Failure Threshold:** Longitude delta `d_lon >= 0` or heading delta `heading_x >= 0`.
- **Prohibited Substitutes:** Checking `.abs() > 0.05` without verifying direction.

### Gate 2: Continuous World Streaming & Topology (`test_h0_gate_continuous_world_streaming_distance`)
- **Requirement:** Driving at highway speed (up to 390 km/h) for 3.5 km and beyond MUST maintain continuous active terrain chunks and collision readiness covering the vehicle.
- **Failure Threshold:** 0 chunks covering vehicle position, or `streamer.is_collision_ready(pos) == false`, or dropping to an empty horizon apron.
- **Prohibited Substitutes:** Checking chunk count within the initial 1.5km downtown sample area.

### Gate 3: Multimodal Spawning Track Topology (`test_h0_gate_multimodal_spawning_track_topology`)
- **Requirement:** A train consist MUST NOT spawn unless the regional dataset contains verified railway corridors and the spawn anchor is snapped within 5.0m of a genuine railway centerline. Spawning over automotive roads or open water without rail geometry is strictly rejected.
- **Failure Threshold:** Spawning when `dataset.railways.is_empty()` or nearest rail track distance `> 5.0m`.
- **Prohibited Substitutes:** Testing trains on isolated synthetic straight tracks.

### Gate 4: Vehicle Rendering & Visual Fidelity (`test_h0_gate_vehicle_rendering_visual_fidelity`)
- **Requirement:** Vehicles must render with authentic multi-material PBR meshes, distinct tinted glass, rubber tires, dark trim, and curved aerodynamic body panels.
- **Failure Threshold:** Fewer than 4 distinct surface color clusters in the final composed draw call, or curved surface normal ratio `<= 25.0%`.
- **Prohibited Substitutes:** Checking triangle counts (> 2,000) or file existence without evaluating visual surface properties.

### Gate 5: City Catalog Ingestion Integrity (`test_h0_gate_catalog_city_transition_integrity`)
- **Requirement:** Every city selectable in the UI catalog for gameplay transition MUST successfully ingest authentic geographic datasets without failing or returning missing package errors.
- **Failure Threshold:** Any catalog region returning `RealEarthError::Ingestion` or crashing on load.
- **Prohibited Substitutes:** Asserting `catalog.len() == 38` in memory without testing actual dataset loading.

---

## 3. Strict Prohibitions & Anti-Patterns

The following practices are formally prohibited across all future engineering stages:

| Prohibited Practice | Forensic Explanation | Mandatory Alternative |
| :--- | :--- | :--- |
| **Direction-Agnostic Assertions** | Using `assert!(delta.abs() > threshold)` which passes even when rotation is inverted. | Verify the algebraic sign and spatial direction vector (`assert!(d_lon < 0)`). |
| **Micro-Simulation Window** | Simulating only 20–60 frames (1 second) in downtown, masking bounding box termination. | Simulate full-mission trajectories across regional distances (>3.5km). |
| **Catalog Count Inflation** | Claiming 38 audited cities based solely on a hardcoded array in `us_catalog.rs`. | Test `load_authentic_city` for every single advertised city. Mark un-bundled cities as `[DOWNLOAD REQUIRED]`. |
| **Monochromatic Color Overrides** | Overwriting all vertices of a vehicle submesh with a single flat color. | Preserve material channels: tinted glass, black trim, rubber, and emissive lights. |
| **Mock / Procedural Masking** | Generating procedural noise terrain when authentic GIS data fails to load, claiming authentic USGS coverage. | Report truthful data provenance. Graceful fallbacks must clearly display `[SYNTHETIC FALLBACK]` in telemetry. |

---

## 4. Contract Acceptance Signature

- **Principal Systems & Rust Engineer:** Verified baseline failure suite and branch preservation.
- **Principal Graphics & 3D Asset Engineer:** Confirmed vehicle model pipeline audit and normal curvature gate.
- **Principal GIS & Cartography Engineer:** Confirmed 38-city audit and boundary termination root cause.
- **Principal QA & Release Engineer:** Formally rejected prior G0–G4 certification reports; marked unsupported claims UNVERIFIED.

*No stage completion or release build may be merged to `main` until this contract is fully satisfied.*
