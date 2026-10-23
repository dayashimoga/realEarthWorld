# Forensic Gap Analysis & Architectural Remediation Roadmap (Phase 0 Audit)

**Document:** `docs/GapAnalysis.md`  
**Auditing Authority:** Principal Rust, Game, GIS, Graphics, Physics Architect & QA Board  
**Audit Date:** October 23, 2026  
**System Verdict:** **NOT PRODUCTION READY ❌**

---

## 1. Executive Summary

This forensic gap analysis provides an unsparing, evidence-grounded assessment of the current state of the Real Earth World Platform. While earlier audit iterations verified that the core single-player desktop simulation runs stably in San Francisco and Bengaluru, this audit identifies critical discrepancies between declared capabilities and actual source-level implementations.

Under the updated **Global Engineering Execution Contract**, no feature may be certified as complete if it relies on synthetic fallbacks where surveyed data was expected, operates through stubbed lifecycle callbacks, or remains completely disconnected from the runnable application.

---

## 2. Priority 0 (Critical Blockers)

### GAP-01: Synthetic / Procedural Geography across 32 of 38 Catalog Cities
* **Classification:** `LOW-QUALITY` / `UNVERIFIED` (Severity: **P0**)
* **Affected Module:** [engine/ingestion/src/us_cities.rs](file:///j:/rustRealEarth/engine/ingestion/src/us_cities.rs)
* **Forensic Finding:** In `us_cities.rs`, only 6 cities (`san-francisco`, `new-york`, `los-angeles`, `chicago`, `seattle`, `miami`) possess hand-curated surveyed templates. The remaining 32 cities default to lines 43–385:
  ```rust
  if dataset.buildings.is_empty() {
      // Synthesize authentic surveyed features if not already present in deep dataset
      let building_half_lat = 0.00045;
      ...
      // Generated landmark building and synthetic grid cross streets
  }
  ```
  Furthermore, authentic raw OpenStreetMap XML files and GeoTIFF elevation rasters only physically exist in `assets/` for `san_francisco_sample` and `bengaluru_sample`.
* **Root Cause:** Ingestion pipelines for the broader catalog relied on programmatic bounding box synthesis rather than disk-backed OSM/DEM data acquisition.
* **Remediation Plan:**
  1. Acquire or ingest authentic OSM PBF/XML excerpts and USGS 3DEP elevation rasters for all 38 cities.
  2. Deprecate and remove parametric synthetic grid generation from production runtime paths.
  3. Update `city_catalog_validator.rs` to verify physical raw file existence and real topology rather than internal generator output.

---

### GAP-02: Android Mobile Surface Rendering and Terrain Elevation Stubs
* **Classification:** `LOW-QUALITY` / `MISSING` (Severity: **P0**)
* **Affected Modules:** [bindings/mobile/src/lib.rs](file:///j:/rustRealEarth/bindings/mobile/src/lib.rs), [android/app/src/main/java/com/realearth/world/MainActivity.java](file:///j:/rustRealEarth/android/app/src/main/java/com/realearth/world/MainActivity.java)
* **Forensic Finding:**
  1. The native JNI surface lifecycle methods in `bindings/mobile/src/lib.rs` (lines 354–395) do not initialize WGPU or render frames:
     ```rust
     pub unsafe extern "C" fn Java_com_realearth_world_RealEarthEngine_nativeSurfaceCreated(
         _env: *mut JNIEnv, _class: jclass, surface: jobject
     ) -> jint {
         if let Ok(mut attached) = SURFACE_ATTACHED.lock() { *attached = true; }
         0
     }
     ```
  2. Mobile vehicle simulation executes against `FlatGround` (a synthetic mathematical plane $y = 0.0$) rather than real DEM elevation grids.
  3. `MainActivity.java` sets up a `SurfaceView` and starts a thread stepping the simulation, but no pixels are rendered to the hardware display by the native engine.
* **Root Cause:** Mobile bindings were created as an ABI and JNI interface contract harness without wiring the underlying `wgpu::Instance::create_surface` through Android `ANativeWindow_fromSurface`.
* **Remediation Plan:**
  1. Link `ndk-sys` / `android` native window APIs to obtain `ANativeWindow` from the Java `Surface` jobject.
  2. Initialize a genuine `wgpu::Surface` and swapchain on Android.
  3. Wire mobile engine context to load real DEM tiles for the active region.

---

### GAP-03: Premature "Production Ready" Certification Declaration
* **Classification:** `GOVERNANCE_VIOLATION` (Severity: **P0**)
* **Affected Modules:** [docs/ProductionReadiness.md](file:///j:/rustRealEarth/docs/ProductionReadiness.md), [docs/certification_dashboard.html](file:///j:/rustRealEarth/docs/certification_dashboard.html)
* **Forensic Finding:** Previous documentation declared the platform "100% Production Ready" and "Version 1.0.0 Ratified" despite GAP-01, GAP-02, and disconnected multiplayer networking.
* **Root Cause:** Quality gates evaluated whether existing automated unit tests passed, but tests asserted on internal stub returns rather than end-to-end user-observable realities.
* **Remediation Plan:** Formally update `ProductionReadiness.md` to declare the system **NOT PRODUCTION READY**, establishing explicit physical gates for production ratification.

---

## 3. Priority 1 (High Quality Deficiencies)

### GAP-04: Multiplayer Networking Completely Disconnected from Desktop Viewer
* **Classification:** `PARTIAL` / `DISCONNECTED` (Severity: **P1**)
* **Affected Modules:** [engine/networking/src/](file:///j:/rustRealEarth/engine/networking/src/), [apps/world-viewer/src/](file:///j:/rustRealEarth/apps/world-viewer/src/)
* **Forensic Finding:** The `realearth-networking` crate contains complete UDP transport, token-bucket rate limiting, interest management, and client prediction logic with passing unit tests. However, `apps/world-viewer` has zero imports or integration with `realearth-networking`. The interactive viewer is strictly single-player offline.
* **Remediation Plan:**
  1. Add an optional `--multiplayer <server-addr>` CLI argument to `world-viewer`.
  2. Wire the viewer ECS loop to send local vehicle input frames and interpolate remote vehicle entities received from the authoritative server.

---

### GAP-05: Misleading Headless Framerate Metrics vs Interactive Presentation
* **Classification:** `LOW-QUALITY` (Severity: **P1**)
* **Affected Module:** [apps/world-viewer/src/main.rs](file:///j:/rustRealEarth/apps/world-viewer/src/main.rs)
* **Forensic Finding:** In headless benchmark mode, `realearth-viewer` reports "5,866.5 FPS (0.17 ms)". This measurement executes an unthrottled loop rendering to a small offscreen target without swapchain presentation, window compositing, or VSync. In interactive windowed mode, the display is locked to VSync (60 Hz / 16.67 ms). Quoting 5000+ FPS misrepresents real-world gameplay performance.
* **Remediation Plan:**
  1. Clearly label headless metrics as "Raw Offscreen Render Pipeline Throughput (Unpresented)".
  2. Instrument interactive windowed mode to measure presented frame pacing, swapchain presentation latency, and 1% low FPS under actual OS window manager compositing.

---

### GAP-06: Test Suite Internal Assumption Bias
* **Classification:** `LOW-QUALITY` (Severity: **P1**)
* **Affected Modules:** [tests/tests/five_platform_application_parity.rs](file:///j:/rustRealEarth/tests/tests/five_platform_application_parity.rs), [tests/tests/final_production_certification.rs](file:///j:/rustRealEarth/tests/tests/final_production_certification.rs)
* **Forensic Finding:** Several tests validate that stub functions return their own hardcoded return values rather than validating real functional effects.
  - `test_f4_android_jni_contract_and_virtual_driving_controls` asserts that `nativeSurfaceCreated` returns `0`, which passes because the stub simply returns `0`.
  - `test_i3_gis_accuracy_and_38_city_catalog_integrity` asserts `buildings.len() > 0`, which passes because the generator synthetically creates a box building when none exists.
* **Remediation Plan:** Replace stub assertion tests with behavior verification (e.g. validating vertex buffers, actual mesh bounds, and physical coordinate alignment).

---

## 4. Priority 2 (Medium Functional Polish)

### GAP-07: Multi-Modal Vehicle Physics Disconnected from Interactive Controls
* **Classification:** `PARTIAL` (Severity: **P2**)
* **Affected Modules:** [engine/vehicles/src/multimodal/](file:///j:/rustRealEarth/engine/vehicles/src/multimodal/), [apps/world-viewer/src/interactive.rs](file:///j:/rustRealEarth/apps/world-viewer/src/interactive.rs)
* **Forensic Finding:** `RoadTransportSimulation`, `MarineSimulation`, `AviationSimulation`, and `RailSimulation` exist as standalone structs with basic formulas, but the viewer only allows driving a car (`DynamicVehicle`). Trains, ships, and airplanes cannot be controlled or rendered as drivable vehicles in the interactive viewer.
* **Remediation Plan:** Expose multi-modal vehicle selection in the interactive viewer and wire corresponding flight/watercraft input handlers.

---

### GAP-08: Traffic NPC Visual Mesh Simplification
* **Classification:** `PARTIAL` (Severity: **P2**)
* **Affected Modules:** [engine/vehicles/src/traffic.rs](file:///j:/rustRealEarth/engine/vehicles/src/traffic.rs), [engine/rendering/src/vehicle_mesh.rs](file:///j:/rustRealEarth/engine/rendering/src/vehicle_mesh.rs)
* **Forensic Finding:** Autonomous traffic simulation simulates NPC vehicle movement along road graphs, but renders them as simplified bounding boxes without articulated wheels or chassis models.
* **Remediation Plan:** Instantiate proper vehicle mesh models with wheel rotations for all active traffic entities within the camera frustum.

---

## 5. Governance & Remediation Schedule

| Phase | Milestone | Focus Areas | Target Acceptance |
| :--- | :--- | :--- | :--- |
| **Phase 1** | **Forensic Truth & Governance** | Update ExecutionContract, FeatureQualityMatrix, GapAnalysis, ADR-0033. | All gaps classified; zero false production-ready claims. |
| **Phase 2** | **Mobile Native Surface Integration** | ANativeWindow integration, genuine WGPU Android swapchain, DEM terrain. | Authentic frame rendering on Android SurfaceView. |
| **Phase 3** | **Multiplayer Viewer Integration** | Wire `realearth-networking` to `world-viewer` with `--multiplayer`. | Multi-client vehicle synchronization in live viewer. |
| **Phase 4** | **Authentic 38-City GIS Ingestion** | Ingest raw OSM & DEM data for all 38 cities; purge synthetic fallback. | 100% surveyed GIS data across all cataloged cities. |
| **Phase 5** | **Multi-Modal Drivable Vehicles** | Wire aviation, marine, and rail physics to viewer input and cameras. | Drivable flight and watercraft in interactive viewer. |
