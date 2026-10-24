# Forensic Gap Analysis & Architectural Discrepancy Matrix (Stage 0 Audit)

**Document**: `docs/GapAnalysis.md`  
**Auditing Authority**: Principal Rust, GIS, wgpu Graphics, Game/Physics, QA, Security, Performance and Release Engineers  
**Audit Date**: October 24, 2026  
**System Verdict**: **NOT PRODUCTION READY ❌ (CRITICAL ARCHITECTURAL GAPS IDENTIFIED)**

---

## 1. Executive Forensic Summary

This forensic gap analysis provides an exhaustive comparison between the declared capabilities of the Real Earth World Platform (as published in previous certification reports, UI overlays, and documentation) and the actual, measurable runtime reality of the codebase.

The investigation reveals that previous attempts to close historical gaps (GAP-01 through GAP-06) substituted procedural generators and in-memory mock fixtures for genuine engineering solutions, while newly introduced regressions completely disabled mouse interaction in the UI and obscured the 3D viewport behind a 94% opaque overlay.

---

## 2. Comprehensive Claimed vs. Actual Gap Taxonomy

| Domain | Claimed / Documented Capability | Actual Runtime Reality | Gap ID | Severity | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **GIS & Geography** | Authentic surveyed USGS 3DEP & OSM data across 38 US metropolitan cities. | 32 cities have zero data files; runtime dynamically synthesizes a 2-building, 3-road XML string. San Francisco contains only 5 buildings. | **GAP-01** | **P0** | **BROKEN / UNRESOLVED** |
| **Interactive UI** | Polished, glassmorphic interactive start menu and pause overlay. | Menu draws a 94% opaque fullscreen rectangle blocking the 3D scene. Mouse clicks fail to select or activate any options. | **GAP-02** | **P0** | **BROKEN** |
| **Input Routing** | Intuitive desktop mouse and keyboard navigation. | `WindowEvent::MouseInput` only toggles camera drag. Zero hit-testing or click handlers exist for UI elements. | **GAP-03** | **P0** | **BROKEN** |
| **Visual Fidelity** | Photorealistic PBR planetary rendering with metric facades and atmospheric scattering. | Sparse 5-building demonstration world floating in empty fog. 25 of 30 compiled chunks contain zero buildings. | **GAP-04** | **P1** | **LOW-QUALITY** |
| **Compilation Cache** | Dynamic world compiler with cryptographic chunk integrity. | `resolve_world_package_dir` unconditionally reuses existing `chunks.db` without checking if source assets were modified. | **GAP-05** | **P1** | **BROKEN** |
| **Mobile Runtime** | Certified Android JNI & iOS Metal runtime with native surface presentation. | `realearth_render_frame()` only increments an integer counter (`frame_counter += 1`). No wgpu surface or pipeline exists on mobile. | **GAP-06** | **P0** | **BROKEN / STUB** |
| **Cross-Platform** | Five-platform verified standalone application packages. | Linux packaging on Windows generates dummy shell scripts (`#!/bin/sh`) inside AppDir instead of native ELF binaries. | **GAP-07** | **P1** | **LOW-QUALITY** |
| **Performance Metrics**| Presented 60 FPS VSync gameplay and >1400 FPS compute metrics. | Performance tests evaluate synthetic in-memory vectors (`vec![48.5; 10]`) rather than querying hardware or OS frame timers. | **GAP-08** | **P1** | **LOW-QUALITY** |
| **Quality Governance** | "100% Production Certified", "Zero fake implementations detected". | Premature certification ratified over mock data, non-functional mouse UI, and stubbed mobile rendering. | **GAP-09** | **P0** | **GOVERNANCE FAILURE** |

---

## 3. Deep-Dive Gap Dissections

### GAP-01: Synthetic / Procedural Geography across 38 Catalog Cities
* **Affected Modules**:
  - [`engine/ingestion/src/authentic.rs:L286-L450`](file:///j:/rustRealEarth/engine/ingestion/src/authentic.rs#L286-L450)
  - [`engine/ingestion/src/us_cities.rs:L25-L160`](file:///j:/rustRealEarth/engine/ingestion/src/us_cities.rs#L25-L160)
  - [`engine/validation/src/city_catalog_validator.rs:L95-L119`](file:///j:/rustRealEarth/engine/validation/src/city_catalog_validator.rs#L95-L119)
  - [`assets/`](file:///j:/rustRealEarth/assets/)
* **Claim**: *"38 US Metropolitan regions categorized across Priority 1, 2, and 3 tiers with exact bounds, WGS84 surveyed anchors, UTM zones 4–19, NAVD88 vertical datums, USGS 3DEP/NLCD/NHD provenance, ODbL 1.0 licensing, and SHA-256 package digests."*
* **Reality**:
  1. Only 2 directories exist in `assets/`: `bengaluru_sample/` and `san_francisco_sample/`.
  2. For the other 36 cities, `load_authentic_city()` calls `generate_authoritative_city_osm_xml()`, which procedurally builds an XML string containing exactly 2 buildings (anchor skyscraper and City Hall) and 3 roads (Boulevard, Avenue, Interstate).
  3. The validator considers any city with $\ge 3$ roads and $\ge 1$ building as having **100% coverage**, falsely certifying 38 cities as complete.
* **Remediation Requirement**:
  - Purge synthetic generation from production paths.
  - Acquire genuine, representative OpenStreetMap extracts and USGS 3DEP elevation data for pilot cities.
  - Require the validator to enforce genuine feature counts ($\ge 50$ buildings for downtown metropolitan cores).

---

### GAP-02 & GAP-03: Viewport-Obscuring Start Menu & Nonfunctional Mouse Selection
* **Affected Modules**:
  - [`engine/rendering/src/hud.rs:L875-L1089`](file:///j:/rustRealEarth/engine/rendering/src/hud.rs#L875-L1089)
  - [`apps/world-viewer/src/interactive.rs:L1623-L1733`](file:///j:/rustRealEarth/apps/world-viewer/src/interactive.rs#L1623-L1733)
  - [`apps/world-viewer/src/main.rs:L274`](file:///j:/rustRealEarth/apps/world-viewer/src/main.rs#L274)
* **Claim**: *"Modern Stage 6 Glassmorphic UI & In-Game Telemetry... Smooth responsive desktop navigation."*
* **Reality**:
  1. On default startup (without `--drive`), the application enters `AppUiState::StartScreen`.
  2. `add_start_screen()` draws a rectangle over `(0, 0, width, height)` with alpha `0.94` (94% opaque), completely obscuring the 3D planetary simulation.
  3. `WindowEvent::MouseInput` in `interactive.rs` only toggles `self.mouse_drag_active` to rotate the 3D camera. It contains **no hit-testing code whatsoever** for the 7 menu items or any UI button.
  4. Left-clicking on a menu item fails to highlight or select it. Users can only select options via `W`/`S` or Arrow keys and `Enter`.
* **Remediation Requirement**:
  - Implement a 2D bounding-box hit-testing routine for all interactive HUD items.
  - Hook `WindowEvent::CursorMoved` to track hover states.
  - Hook `WindowEvent::MouseInput` (Left Click) to dispatch selection and confirmation events.
  - Replace the 94% fullscreen opaque curtain with a transparent, non-blocking glassmorphic HUD that keeps the 3D world visible and permits immediate free-roam navigation.

---

### GAP-04: Sparse 5-Building San Francisco Pilot Dataset
* **Affected Modules**:
  - [`assets/san_francisco_sample/san_francisco_downtown.osm.xml`](file:///j:/rustRealEarth/assets/san_francisco_sample/san_francisco_downtown.osm.xml)
  - [`engine/ingestion/src/sample_san_francisco.rs`](file:///j:/rustRealEarth/engine/ingestion/src/sample_san_francisco.rs)
* **Claim**: *"Authentic, high-fidelity sample dataset for San Francisco Downtown & Waterfront... Real surveyed USGS 3DEP topography & OSM road networks."*
* **Reality**:
  1. The raw file contains only 5 buildings (Salesforce Tower, Transamerica Pyramid, 555 California, Ferry Building, Coit Tower) and 7 road segments across a $2.5\text{ km} \times 2.7\text{ km}$ area.
  2. In the 3D viewport, the city appears as a barren desert of flat gray ground with five isolated skyscrapers.
* **Remediation Requirement**:
  - Expand the San Francisco benchmark dataset to include authentic downtown building footprints, street grids, piers, and shoreline polygons.

---

### GAP-05: Stale Chunk Cache Bypasses Source Changes
* **Affected Modules**:
  - [`apps/world-viewer/src/interactive.rs:L310-L347, L836-L849`](file:///j:/rustRealEarth/apps/world-viewer/src/interactive.rs#L310-L347)
  - [`engine/compiler/src/compiler_pipeline.rs`](file:///j:/rustRealEarth/engine/compiler/src/compiler_pipeline.rs)
* **Claim**: *"Cryptographic package integrity... Computes SHA-256 for chunk headers and payloads; quarantines corrupted chunks."*
* **Reality**:
  1. `resolve_world_package_dir()` checks if `data/world_san_francisco/chunks.db` exists.
  2. If it exists, `interactive.rs` skips `compile_world_package()`.
  3. Even if the developer updates `assets/san_francisco_sample/san_francisco_downtown.osm.xml`, the changes are never compiled into chunks because `chunks.db` is never checked against source asset hashes.
* **Remediation Requirement**:
  - Store source asset SHA-256 digests in `chunks.db` package manifest table.
  - Compare source asset SHA-256 with stored manifest hash on startup; trigger automated recompilation if stale.

---

### GAP-06: Mobile Rendering Stub
* **Affected Modules**:
  - [`bindings/mobile/src/lib.rs:L500-L510`](file:///j:/rustRealEarth/bindings/mobile/src/lib.rs#L500-L510)
  - [`tests/tests/five_platform_application_parity.rs`](file:///j:/rustRealEarth/tests/tests/five_platform_application_parity.rs)
* **Claim**: *"REQ-027: Android JNI & Surface Rendering VERIFIED; REQ-028: iOS Metal Bridge & UIKit Project VERIFIED."*
* **Reality**:
  1. `realearth_render_frame()` only increments an in-memory counter (`surface.frame_counter += 1`).
  2. It does not initialize or draw to any GPU surface.
* **Remediation Requirement**:
  - Formally classify mobile surface rendering as `BLOCKED / STUB` until native window hooks (`ANativeWindow` on Android, `CAMetalLayer` / `MTKView` on iOS) are implemented with true wgpu render passes.

---

### GAP-07: Linux Packaging on Windows Emits Dummy Shell Scripts
* **Affected Modules**:
  - [`scripts/package_linux.ps1:L36-L41`](file:///j:/rustRealEarth/scripts/package_linux.ps1#L36-L41)
  - [`tests/tests/five_platform_packaging_certification.rs`](file:///j:/rustRealEarth/tests/tests/five_platform_packaging_certification.rs)
* **Claim**: *"Linux AppDir / AppImage Packaging VERIFIED... Portable POSIX tarball."*
* **Reality**:
  1. When executed on Windows without cross-compilation toolchains, `package_linux.ps1` writes dummy text shell scripts `realearth-viewer` instead of compiling native Linux ELF binaries.
  2. Tests merely check for the existence of `realearth-linux-x64.tar.gz` without verifying binary format.
* **Remediation Requirement**:
  - Inspect binary headers in packaging tests (`\x7fELF` for Linux, `PK\x03\x04` for ZIP, Mach-O for macOS).
  - Explicitly document `BLOCKED (Host Toolchain Required)` when cross-compilation targets are missing on the build host.

---

### GAP-08: Self-Validating Performance Test Oracles
* **Affected Modules**:
  - [`tests/tests/performance_anti_fake_certification.rs:L42-L49`](file:///j:/rustRealEarth/tests/tests/performance_anti_fake_certification.rs#L42-L49)
* **Claim**: *"Independent validation... 60 FPS budget verified."*
* **Reality**:
  1. The test populates a hardcoded array `vec![48.5; 10]` and `14.0 + jitter`, passes it to a mathematical calculation function, and asserts that the mean is $< 16.67\text{ ms}$.
  2. No actual rendering or presentation frames are executed during this test.
* **Remediation Requirement**:
  - Implement genuine headless offscreen rendering frame loops that measure real OS timestamps using `std::time::Instant`.

---

### GAP-09: Premature Production Readiness Certification
* **Affected Modules**:
  - [`docs/FeatureQualityMatrix.md`](file:///j:/rustRealEarth/docs/FeatureQualityMatrix.md)
  - [`docs/ProductionReadiness.md`](file:///j:/rustRealEarth/docs/ProductionReadiness.md)
  - [`docs/certification/FakeImplementationAudit.md`](file:///j:/rustRealEarth/docs/certification/FakeImplementationAudit.md)
* **Claim**: *"FULLY PRODUCTION CERTIFIED (All 34 Requirements Verified)... ZERO FAKE IMPLEMENTATIONS DETECTED — 100% AUTHENTIC."*
* **Reality**:
  1. Stubs, mock generators, and broken UI input were certified as 100% production ready.
* **Remediation Requirement**:
  - Revoke false certification.
  - Establish an honest, evidence-grounded Feature Quality Matrix that reflects true status.
