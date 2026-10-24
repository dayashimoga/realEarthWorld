# Real Earth Platform — Root Cause Analysis (Forensic Audit Stage 0)

**Document**: `docs/RootCauseAnalysis.md`  
**Auditing Authority**: Principal Rust, GIS, wgpu Graphics, Game/Physics, QA, Security, Performance and Release Engineers  
**Date**: October 24, 2026  
**Status**: ACTIVE FORENSIC AUDIT RECORD  

---

## 1. Executive Summary & Diagnostic Overview

A comprehensive, forensic investigation of the `realEarthWorld` codebase was conducted following user inspection of the running application. At runtime, the application presents:
1. **Extremely sparse San Francisco geometry**: A grey, foggy, barren landscape with approximately 5 isolated buildings floating on a low-resolution grid, flanked by a single road strip.
2. **A massive semi-opaque start menu**: A 94% opaque dark backdrop obscuring approximately 95% of the viewport, locking the user out of immediate 3D visualization.
3. **Nonfunctional mouse selection**: Mouse clicks anywhere on the menu buttons or UI options fail to register, select, or trigger any action; only keyboard directional controls (`W`/`S`/`Up`/`Down`/`Enter`) navigate the menu.
4. **Unverifiable claims of 38 US cities**: The UI prominently claims full surveyed coverage for 38 US cities with USGS 3DEP topography, OSM road networks, and multimodal transit, stamped with a `"v1.0.0-STAGE6-CERTIFIED"` badge.
5. **Stale compiled chunks and self-validating test suites**: Tests certify 100% production readiness while testing synthetic in-memory fixtures or dummy stubs.

This document identifies the **exact root causes**, line numbers, architectural decisions, and systemic failures that produced these defects.

---

## 2. Root Cause 1: Sparse San Francisco 3D Geometry

### Observed Symptom
When navigating the San Francisco pilot region, the world appears virtually empty: Salesforce Tower, Transamerica Pyramid, 555 California Street, Ferry Building, and Coit Tower stand isolated in a featureless plane with vast empty voids, missing all downtown street blocks, residential zones, commercial skyscrapers, piers, and urban density.

### Forensic Source Analysis
1. **Inspection of Asset Payload**:
   - Location: [`assets/san_francisco_sample/san_francisco_downtown.osm.xml`](file:///j:/rustRealEarth/assets/san_francisco_sample/san_francisco_downtown.osm.xml)
   - The file size is merely **9,034 bytes** (267 lines of XML).
   - A complete count of entities in this file reveals:
     * **Buildings**: Exactly **5 buildings** (lines 9, 27, 45, 63, 81).
     * **Roads**: Exactly **7 road segments** (Market St, Embarcadero, California St, Montgomery St, I-80 approach, Broadway tunnel, Sansome St).
     * **Water bodies**: Exactly **1 polygon** (San Francisco Bay).
     * **Land cover**: Exactly **2 small green polygons** (Salesforce Park, Pioneer Park).
2. **Hardcoded Sample Generator**:
   - Location: [`engine/ingestion/src/sample_san_francisco.rs`](file:///j:/rustRealEarth/engine/ingestion/src/sample_san_francisco.rs#L21-L109)
   - Function `create_san_francisco_dataset()` manually hardcodes the identical 5 buildings and 7 roads as Rust structs.
3. **Low-Resolution DEM Grid**:
   - Location: [`assets/san_francisco_sample/usgs_3dep_san_francisco.bin`](file:///j:/rustRealEarth/assets/san_francisco_sample/usgs_3dep_san_francisco.bin)
   - The binary DEM file is exactly 16,384 bytes ($64 \times 64 \times 4\text{ bytes}$ `f32`). For a $2.78\text{ km} \times 2.50\text{ km}$ bounding box, each DEM sample covers $\approx 40\text{ meters}$, obliterating curbs, hillsides, and detailed slope gradients.
4. **Compiler Chunk Slicing**:
   - Location: [`engine/compiler/src/compiler_pipeline.rs`](file:///j:/rustRealEarth/engine/compiler/src/compiler_pipeline.rs)
   - The compiler partitions the bounding box into $500\text{m}$ spatial chunks. Out of 30 generated chunks, 25 chunks contain zero buildings, and many contain zero roads. Only the terrain heightfield is meshed.

### Root Cause Verdict
The application does not ingest authentic, complete OpenStreetMap extracts or real high-resolution USGS 3DEP LiDAR rasters for San Francisco. It ingests an ultra-minimalist, hand-crafted 5-building fixture originally written for initial unit tests, which was erroneously promoted to the production dataset and falsely labeled "Authoritative USGS 3DEP & OSM Extract".

---

## 3. Root Cause 2: Large Semi-Opaque Menu Obscuring the Viewport

### Observed Symptom
Upon launching `realearth-viewer` with default arguments, the screen is covered by an overwhelming dark blue-gray card overlay that obscures nearly the entire 3D planetary simulation, leaving only peripheral glimpses of the background.

### Forensic Source Analysis
1. **Default Mode Routing**:
   - Location: [`apps/world-viewer/src/main.rs`](file:///j:/rustRealEarth/apps/world-viewer/src/main.rs#L274)
   ```rust
   let is_drive = args.iter().any(|a| a == "--drive" || a == "-d") || mode_arg.is_some();
   ```
   - When launched with no arguments (the standard user entry point), `is_drive` evaluates to `false`.
2. **Initial UI State Assignment**:
   - Location: [`apps/world-viewer/src/interactive.rs`](file:///j:/rustRealEarth/apps/world-viewer/src/interactive.rs#L477-L481)
   ```rust
   ui_state: if is_drive {
       AppUiState::InGame
   } else {
       AppUiState::StartScreen
   },
   ```
   - This sets `self.ui_state = AppUiState::StartScreen`.
3. **Fullscreen Opaque Backdrop Rendering**:
   - Location: [`engine/rendering/src/hud.rs`](file:///j:/rustRealEarth/engine/rendering/src/hud.rs#L875-L876)
   ```rust
   // Fullscreen dark gradient backdrop
   self.add_rect(0.0, 0.0, screen_w, screen_h, [0.03, 0.04, 0.07, 0.94]);
   ```
   - The HUD renderer draws a rectangle covering `[0, 0, screen_w, screen_h]` with alpha `0.94` (94% opaque).
   - In addition, lines 945–1060 draw two large opaque panels (menu card width: up to 520px; feature highlight card width: up to $(screen\_w - 600\text{px})$), visually dominating the window.

### Root Cause Verdict
The application treats the interactive viewer as a console game title screen rather than a planetary GIS viewer. It forces an almost totally opaque (94% opacity) modal barrier over the 3D scene on startup, without an option to minimize or dismiss it via mouse click or view-through transparency.

---

## 4. Root Cause 3: Nonfunctional Mouse Selection & Missing Hit-Testing

### Observed Symptom
Clicking with the left or right mouse button over any of the 7 start menu options ("CONTINUE CAREER", "FREE ROAM SIMULATION", "CAREER CONTRACTS", etc.) does nothing. The cursor does not highlight options on hover, and clicking does not activate the options.

### Forensic Source Analysis
1. **Window Event Mouse Input Routing**:
   - Location: [`apps/world-viewer/src/interactive.rs`](file:///j:/rustRealEarth/apps/world-viewer/src/interactive.rs#L1623-L1630)
   ```rust
   WindowEvent::MouseInput { state, button, .. } => {
       if button == MouseButton::Right || button == MouseButton::Left {
           self.mouse_drag_active = state == ElementState::Pressed;
           if !self.mouse_drag_active {
               self.last_cursor_pos = None;
           }
       }
   }
   ```
   - `WindowEvent::MouseInput` **only** toggles `self.mouse_drag_active`.
   - There is **zero** UI hit-testing logic. No bounding box check is performed against menu items, buttons, or dialog controls.
2. **Cursor Motion Routing**:
   - Location: [`apps/world-viewer/src/interactive.rs`](file:///j:/rustRealEarth/apps/world-viewer/src/interactive.rs#L1632-L1643)
   ```rust
   WindowEvent::CursorMoved { position, .. } => {
       if self.mouse_drag_active {
           if let Some((prev_x, prev_y)) = self.last_cursor_pos {
               let dx = (position.x - prev_x) as f32;
               let dy = (position.y - prev_y) as f32;
               let sensitivity = 0.0035;
               self.camera_ctrl.rotate(-dx * sensitivity, -dy * sensitivity);
           }
           self.last_cursor_pos = Some((position.x, position.y));
       }
   }
   ```
   - Cursor movement is only tracked when mouse drag is active, and only feeds into camera rotation. Cursor position is never routed to the UI layout engine to update hover states.
3. **Keyboard-Only UI State Machine**:
   - Location: [`apps/world-viewer/src/interactive.rs`](file:///j:/rustRealEarth/apps/world-viewer/src/interactive.rs#L1674-L1733)
   - Menu navigation is hardcoded exclusively to `KeyCode::ArrowUp`, `KeyCode::ArrowDown`, `KeyCode::KeyW`, `KeyCode::KeyS`, and activation to `KeyCode::Enter` or `KeyCode::Space`.

### Root Cause Verdict
Mouse UI interaction is completely unimplemented. The UI system renders static 2D screen-space vertex geometry without maintaining an interactive element hierarchy, bounding boxes, or click-dispatch handlers. Left-clicking inadvertently initiates 3D camera rotation underneath the opaque menu rather than selecting the clicked UI element.

---

## 5. Root Cause 4: Unverifiable Claims of 38 US Cities & Synthetic Geography Generation

### Observed Symptom
The UI claims: *"Browse 38 high-precision US surveyed cities... Real surveyed USGS 3DEP topography & OSM road networks... v1.0.0-STAGE6-CERTIFIED"*. Documentation asserts: *"38 US Metropolitan regions categorized across Priority 1, 2, and 3 tiers with exact bounds... USGS 3DEP/NLCD/NHD provenance, ODbL 1.0 licensing... VERIFIED"*.

### Forensic Source Analysis
1. **Absence of Real Data Assets**:
   - Location: `assets/`
   - Checking directory contents reveals:
     * `assets/bengaluru_sample/`
     * `assets/san_francisco_sample/`
   - There are **no assets whatsoever** for New York, Los Angeles, Chicago, Seattle, Miami, Boston, Washington D.C., Atlanta, Dallas, Houston, Phoenix, Philadelphia, Denver, or any of the other 36 cataloged cities.
2. **Synthetic XML Generation Masquerading as Authoritative Acquisition**:
   - Location: [`engine/ingestion/src/authentic.rs`](file:///j:/rustRealEarth/engine/ingestion/src/authentic.rs#L239-L242)
   ```rust
   // Fallback: Generate bit-exact authoritative OSM XML bundle with verified real-world features
   let content = generate_authoritative_city_osm_xml(city);
   let hash = compute_sha256(content.as_bytes());
   Ok((content, hash))
   ```
   - When a requested city file is not found on disk (which is true for all cities except San Francisco), it falls back to `generate_authoritative_city_osm_xml(city)`.
3. **Inspection of `generate_authoritative_city_osm_xml()`**:
   - Location: [`engine/ingestion/src/authentic.rs`](file:///j:/rustRealEarth/engine/ingestion/src/authentic.rs#L286-L450)
   - This function procedurally constructs an XML string with:
     * Exactly 1 anchor skyscraper box (4 nodes)
     * Exactly 1 civic building box (4 nodes)
     * Exactly 1 North-South arterial way (3 nodes)
     * Exactly 1 East-West arterial way (3 nodes)
     * Exactly 1 Interstate highway overpass (3 nodes)
   - It computes the SHA-256 hash of this newly generated in-memory string and returns it as `meta.source_osm_sha256`, claiming cryptographic provenance!
4. **Self-Validating Catalog Readiness Audit**:
   - Location: [`engine/validation/src/city_catalog_validator.rs`](file:///j:/rustRealEarth/engine/validation/src/city_catalog_validator.rs#L95-L118)
   ```rust
   let road_coverage: f32 = if validated.roads.len() >= 3 && has_drivable_route {
       100.0
   } else ...
   let bld_coverage: f32 = if validated.buildings.iter().any(|b| ...) {
       100.0
   } else ...
   ```
   - If a city has at least 3 roads and at least 1 building, the validator awards **100% road coverage** and **100% building coverage**, resulting in the reported *"97.8% mean readiness"* across all 38 cities.

### Root Cause Verdict
The 38 US cities claim is an engineered illusion. 32 of the 38 cities have zero source data files and rely on a 2-building, 3-road procedural XML template generated at runtime. The validation logic was artificially calibrated to treat 3 roads and 1 building as "100% coverage", and previous reports improperly certified this as "100% authentic and verified".

---

## 6. Root Cause 5: Stale Compiled Chunks & Chunk Cache Invalidation Failure

### Observed Symptom
Modifying or updating raw OSM XML or DEM assets does not alter the rendered world or recompile the world package; stale geometry persists indefinitely.

### Forensic Source Analysis
1. **Cache-First Path Resolution**:
   - Location: [`apps/world-viewer/src/interactive.rs`](file:///j:/rustRealEarth/apps/world-viewer/src/interactive.rs#L310-L347)
   ```rust
   pub fn resolve_world_package_dir(slug: &str) -> PathBuf {
       let bundled = PathBuf::from(format!("data/world_{}", slug));
       if bundled.join("chunks.db").exists() {
           return bundled;
       }
       ...
   }
   ```
2. **Unconditional Re-use Without Hash Verification**:
   - Location: [`apps/world-viewer/src/interactive.rs`](file:///j:/rustRealEarth/apps/world-viewer/src/interactive.rs#L836-L849)
   ```rust
   if !dir.join("chunks.db").exists() {
       // Only compiles if chunks.db is completely absent!
       compile_world_package(&val, &dem, &dir, &options)?;
   }
   ```
   - The existence of `chunks.db` is used as a permanent bypass flag.
   - The compilation step never checks whether the underlying OSM XML hash or DEM binary hash in `assets/` matches the manifest record in `chunks.db`.
   - `data/world_san_francisco/chunks.db` was committed with precompiled chunks from an earlier build, preventing any asset improvements from ever being compiled or seen.

### Root Cause Verdict
The world compilation pipeline lacks source-hash cache invalidation. A pre-existing `chunks.db` file permanently suppresses compilation of fresh geographic assets.

---

## 7. Root Cause 6: Mobile Runtime Stubs Masquerading as Certified Packaging

### Observed Symptom
Documentation and tests claim: *"REQ-027: Android JNI & Surface Rendering VERIFIED; REQ-028: iOS Metal Bridge & UIKit Project VERIFIED"*.

### Forensic Source Analysis
1. **Inspection of Mobile Render Callback**:
   - Location: [`bindings/mobile/src/lib.rs`](file:///j:/rustRealEarth/bindings/mobile/src/lib.rs#L500-L510)
   ```rust
   #[no_mangle]
   pub extern "C" fn realearth_render_frame() -> c_int {
       let mut surface = match SURFACE_STATE.lock() {
           Ok(g) => g,
           Err(_) => return -1,
       };
       if !surface.is_attached || surface.is_paused {
           return 0;
       }
       surface.frame_counter += 1;
       0
   }
   ```
   - `realearth_render_frame()` does **nothing except increment an integer counter** (`frame_counter += 1`).
   - There is no `wgpu::Instance`, no `wgpu::Device`, no `wgpu::Surface`, no command encoder, no shader execution, and no pixel presentation.
2. **Tautological Test Assertion**:
   - Location: [`tests/tests/five_platform_application_parity.rs`](file:///j:/rustRealEarth/tests/tests/five_platform_application_parity.rs)
   - The test attaches a dummy surface handle (`0xDEADBEEF`), calls `realearth_render_frame()`, asserts that the return code is `0`, and certifies mobile GPU presentation.
3. **Linux AppDir Staging on Windows**:
   - Location: [`scripts/package_linux.ps1`](file:///j:/rustRealEarth/scripts/package_linux.ps1#L36-L41)
   - On Windows, instead of cross-compiling an ELF binary, it writes a plain text shell script:
     `Set-Content -Path "$AppDir/usr/bin/realearth-viewer" -Value "#!/bin/sh..."`
   - It tars this directory and certifies Linux packaging as complete.

### Root Cause Verdict
The mobile rendering implementation is a no-op stub. Cross-platform packaging tests assert on the existence of file archives and return codes rather than executable platform verification, creating a false impression of multi-platform readiness.

---

## 8. Root Cause 7: Self-Validating Test Oracles & Synthetic Test Fixtures

### Observed Symptom
All 29 integration test suites pass (100% pass rate) and report 92.4% code coverage, despite major functional defects in runtime presentation, GIS coverage, and UI interactivity.

### Forensic Source Analysis
1. **Synthetic Telemetry in Performance Tests**:
   - Location: [`tests/tests/performance_anti_fake_certification.rs`](file:///j:/rustRealEarth/tests/tests/performance_anti_fake_certification.rs#L42-L49)
   ```rust
   // 1. Simulate 60 frame durations: 10 initial shader compilation warmup frames
   // followed by 50 steady-state presented frames.
   let mut frame_durations = vec![48.5; 10];
   for i in 0..50 {
       let jitter = (i % 5) as f64 * 0.4;
       frame_durations.push(14.0 + jitter); // 14.0 - 15.6 ms
   }
   let pacing = FramePacingMetrics::compute(&frame_durations, 10);
   ```
   - The performance test does not render frames or query the OS clock. It populates an in-memory `Vec<f64>` with hardcoded numbers between 14.0 and 15.6 ms, feeds it to a statistical helper, and asserts that the average is under 16.67 ms.
2. **Absence of Negative UI Tests**:
   - There are zero tests that instantiate `InteractiveViewerApp`, fire mouse events, and verify that the active UI state or menu selection changes.

### Root Cause Verdict
The test suite suffers from "tautological certification": tests validate internal mathematical formulas on synthetic inputs rather than probing real runtime systems, real hardware clocks, real file formats, or real user input pipelines.

---

## 9. Comprehensive Root Cause Matrix

| Defect ID | Defect Description | Primary Root Cause File | Technical Root Cause Mechanism | Severity |
| :--- | :--- | :--- | :--- | :--- |
| **RC-01** | Sparse San Francisco Geometry | [`assets/san_francisco_sample/san_francisco_downtown.osm.xml`](file:///j:/rustRealEarth/assets/san_francisco_sample/san_francisco_downtown.osm.xml) | Asset contains only 5 buildings and 7 roads; real OSM dataset never ingested. | **P0** |
| **RC-02** | Large Semi-Opaque Menu | [`engine/rendering/src/hud.rs:L876`](file:///j:/rustRealEarth/engine/rendering/src/hud.rs#L876) | Fullscreen backdrop drawn with 0.94 alpha covering entire screen on default startup. | **P1** |
| **RC-03** | Nonfunctional Mouse Selection | [`apps/world-viewer/src/interactive.rs:L1623`](file:///j:/rustRealEarth/apps/world-viewer/src/interactive.rs#L1623) | MouseInput only sets `mouse_drag_active`; zero hit-testing or click dispatching exists. | **P0** |
| **RC-04** | 38 Cities Unverifiable Claims | [`engine/ingestion/src/authentic.rs:L286`](file:///j:/rustRealEarth/engine/ingestion/src/authentic.rs#L286) | 32 cities have no files; dynamically generated 2-building template labeled "authoritative". | **P0** |
| **RC-05** | Stale Compiled Chunks | [`apps/world-viewer/src/interactive.rs:L836`](file:///j:/rustRealEarth/apps/world-viewer/src/interactive.rs#L836) | `chunks.db` existence unconditionally skips compilation without checking source hashes. | **P1** |
| **RC-06** | Mobile Render No-Op Stubs | [`bindings/mobile/src/lib.rs:L500`](file:///j:/rustRealEarth/bindings/mobile/src/lib.rs#L500) | `realearth_render_frame` only increments an integer counter; no wgpu pipeline exists. | **P1** |
| **RC-07** | Self-Validating Tests | [`tests/tests/performance_anti_fake_certification.rs:L42`](file:///j:/rustRealEarth/tests/tests/performance_anti_fake_certification.rs#L42) | Tests evaluate synthetic in-memory vectors and stubs rather than real runtime outcomes. | **P1** |
| **RC-08** | Premature Production Certification | [`docs/FeatureQualityMatrix.md:L6`](file:///j:/rustRealEarth/docs/FeatureQualityMatrix.md#L6) | Declared "100% PRODUCTION CERTIFIED" despite stubs, mock data, and broken UI. | **P0** |

---

## 10. Required Action Items (Remediation Prerequisites)

1. **Purge Synthetic Data Claims**: Formally revoke the "100% production certified" rating across all documentation.
2. **Implement Real Mouse Hit-Testing**: Add 2D bounding-box hit detection for UI panels and buttons in `interactive.rs` and `hud.rs`.
3. **Make Start Menu Dismissible & Transparent**: Replace the 94% opaque fullscreen overlay with a modern, glassmorphic, non-blocking HUD that allows immediate viewing of the 3D world, with one-click dismissal.
4. **Acquire & Ingest Real San Francisco GIS Data**: Ingest an authentic multi-thousand-building OpenStreetMap dataset and authentic USGS 3DEP LiDAR elevation raster for San Francisco.
5. **Implement Source-Hash Invalidation in Chunk Compiler**: Force recompilation whenever the source data SHA-256 does not match the compiled manifest.
6. **Honestly Disclose Platform Status**: Categorize mobile rendering as `BLOCKED / STUB` until genuine wgpu surface pipelines are linked to native window handles.
7. **Deploy Real Executable Validation Infrastructure**: Replace synthetic test oracles with tests that fail on mock data and pass only on real, verified systems.
