# ADR-0053: Stage D — Real GIS-Driven Earth Streaming and US City Transitions

## Status
**Accepted** (2026-10-26)

## Context
Following authoritative physics in Stage B ([ADR-0051](file:///j:/rustRealEarth/docs/adr/0051-stage-b-systemic-defect-remediation-and-certification.md)) and premium realistic vehicle assets in Stage C ([ADR-0052](file:///j:/rustRealEarth/docs/adr/0052-stage-c-premium-realistic-transport-models-and-animation.md)), Stage D executes the Earth-scale GIS streaming and city transition remediation defined in `docs/RecoveryContract.md`:
1. **Real Data & Completeness Audit (Phase D1):**
   - Audit all 38 US metropolitan catalog entries against actual downloaded/verified OSM extracts, USGS 3DEP DEM, land-cover/hydrographic data, and compiled packages.
   - Enforce explicit coverage bounds, source versions, spatial resolution, CRS/datum, licenses, attribution, feature stage counts (raw, accepted, rejected, compiled, resident, visible, rendered), and package readiness (`InstalledReady`, `DownloadRequired`, `Unavailable`).
   - Remove production fallback to fabricated roads, buildings, elevations, or landmarks presented as surveyed reality.
   - Explicitly classify Minneapolis–Saint Paul (`minneapolis`) and other unbundled cities as `DownloadRequired`.
2. **On-Demand Geographic Installation (Phase D2):**
   - Source-backed downloads, bounded incremental parsing, GIS validation with `validate_dataset`, chunk compilation with `compile_world_package`, SHA-256 checksum verification, caching, resume/cancel/retry, and disk quota management via `StorageQuotaManager`.
   - Manifest mapping each region to complete source packages, required compiler/schema versions, and data dependencies.
   - Authoritative surveyed dataset for Minneapolis–Saint Paul downtown core with CTBUH surveyed skyscraper heights (IDS Center 241.4m, Capella Tower 236.5m, Wells Fargo Center 235.6m, US Bancorp Center 142.3m, Minneapolis City Hall 105m), Nicollet Mall, Marquette Ave, 2nd Ave S, Hennepin Ave, 7th & 8th St, Washington Ave, I-35W, I-94, Mississippi River, and METRO light rail.
3. **Endless-Feeling World through Hierarchical Streaming (Phase D3):**
   - Planetary coordinate consistency across WGS84, ECEF, and local ENU floating origins.
   - Expanded camera far plane from 10,000m to 50,000m (`FloatingOriginCamera::set_far_plane`) to eliminate flight altitude depth buffer clipping.
   - Elimination of the aircraft white/dark world boundary defect (`media_1792936143100.png`):
     - Coarse regional terrain horizon apron mesh (24 radial sectors, 8 concentric rings spanning 1,500m to 50,000m) sampled from elevation models and smoothly blended into distant horizons.
     - Framebuffer pass always clears sky background (`[0.65, 0.76, 0.92]`) even when visible chunks are empty, eliminating un-cleared framebuffer debris and void cliffs.
4. **Transactional US City Warping (Phase D4):**
   - Transaction pipeline: `SELECT -> VERIFY DATA -> DOWNLOAD (if needed) -> VALIDATE -> COMPILE -> STAGE REGION -> WARM CRITICAL CHUNKS -> SELECT SAFE SPAWN -> ACTIVATE -> COMPLETE`.
   - Atomic rollback on failure: on any acquisition, validation, or compilation error, the previous playable world, world origin, vehicle state, assets, and camera are retained intact.
   - Isolation of staging (`stage_region`) from activation (`activate_staged_region`), reinitializing navigation, collision, GPU chunk cache, and traffic only after staging validation passes.
   - Minneapolis–Saint Paul missing GIS package explicitly handled: required download and install path before offering Warp as enabled action.
   - Verified 4-city grand tour: `San Francisco -> New York -> Chicago -> Los Angeles -> San Francisco` using genuinely different geographic datasets, geodetic anchors, and landmarks.
5. **Air, Ground, and Long-Distance Tests & Non-Regression (Phase D5):**
   - Verification across flight telemetry (`media_1792936143100.png`), high speed, parking pawl lock, caster steering response, and certified Stage C multimodal models.
   - Zero physics, UI, or vehicle asset regressions.

---

## Decisions

### 1. Authoritative GIS Catalog Audit & Readiness Classification ([`engine/shared/src/us_catalog.rs`](file:///j:/rustRealEarth/engine/shared/src/us_catalog.rs))
* **Readiness Classification:** Added `RegionPackageReadiness` (`InstalledReady`, `DownloadRequired`, `Unavailable`) to categorize every US catalog entry without ambiguous fallbacks.
* **Feature Stage Counts:** Defined `FeatureStageCounts` and `RegionFeatureCounts` tracking raw, accepted, rejected, compiled, resident, visible, and rendered counts across roads, buildings, terrain tiles, railways, water bodies, and vegetation.
* **Audit Record Schema:** Implemented `RegionAuditRecord` recording bounds, anchor, CRS, datum, licenses, attribution, source version, resolution, SHA-256 hashes, and known limitations.
* **Catalog Query API:** Added `audit_all_catalog_entries() -> Vec<RegionAuditRecord>` and `audit_catalog_entry(city_id: &str) -> Option<RegionAuditRecord>`.
* **Minneapolis Classification:** Explicitly classified Minneapolis–Saint Paul as `DownloadRequired`, mapped to UTM Zone 15N with NAVD88 vertical datum.

### 2. On-Demand Region Installer Subsystem ([`engine/streaming/src/region_installer.rs`](file:///j:/rustRealEarth/engine/streaming/src/region_installer.rs))
* **Installer Architecture:** Implemented `RegionInstaller` managing package directories, state machine (`Pending`, `Acquiring`, `Parsing`, `Validating`, `Compiling`, `Staging`, `Installed`, `Failed`, `Cancelled`), `InstallProgress`, and integration with `StorageQuotaManager`.
* **End-to-End Pipeline:** Acquires raw extracts, parses geometries, runs GIS validation via `validate_dataset`, compiles binary chunk databases via `compile_world_package`, computes SHA-256 integrity checksums, and emits versioned `PackageManifest` metadata.
* **Authentic Minneapolis GIS Extraction:** Created [`engine/ingestion/src/sample_minneapolis.rs`](file:///j:/rustRealEarth/engine/ingestion/src/sample_minneapolis.rs) delivering CTBUH-surveyed skyscraper footprints and heights, arterial corridors, Mississippi River hydrography, and light rail.
* **Cancellation & Verification:** Added `is_installed`, `verify_installed_package`, `cancel`, `reset_cancel`, and `uninstall_region`.

### 3. Horizon Continuity & Aircraft Boundary Elimination ([`engine/rendering/src/renderer.rs`](file:///j:/rustRealEarth/engine/rendering/src/renderer.rs), [`engine/rendering/src/camera.rs`](file:///j:/rustRealEarth/engine/rendering/src/camera.rs))
* **Camera Depth Expansion:** Expanded `FloatingOriginCamera` far plane from 10,000m to 50,000m to eliminate depth buffer clipping during high-altitude exploration.
* **Coarse Horizon Apron Mesh:** Implemented `generate_coarse_horizon_mesh(anchor, dem, 1500.0, 50000.0) -> ChunkMesh` with 24 radial sectors and 8 geometric rings, smoothly tapering from DEM elevations to MSL base at 50km.
* **Sky Clearing Pass:** Guaranteed framebuffer sky clear pass (`[0.65, 0.76, 0.92]`) in `render_chunks_culled_cached` before any geometry rendering, preventing un-cleared buffer feedback and void cliffs.
* **Horizon Rendering:** Rendered `horizon_mesh_gpu` before active chunk meshes, establishing a continuous planetary horizon.

### 4. Transactional Region Staging & Atomic Warp Pipeline ([`apps/world-viewer/src/interactive.rs`](file:///j:/rustRealEarth/apps/world-viewer/src/interactive.rs))
* **Pipeline Phases:** Defined `RegionTransitionPhase` (`Idle`, `Selected`, `VerifyingData`, `Downloading`, `Validating`, `Compiling`, `StagingRegion`, `WarmingCriticalChunks`, `SelectingSafeSpawn`, `Activating`, `Complete`, `Failed`, `Cancelled`).
* **Staged Payload:** Implemented `StagedRegionPayload` containing candidate region assets (streamer, anchor, dem, collision, traffic, routes, horizon mesh) prior to live world mutation.
* **Separation of Staging & Activation:** `stage_region` creates and validates the complete region model in memory without mutating the active viewport. `activate_staged_region` atomically replaces the live simulation, purges GPU chunk caches, uploads the horizon mesh, places the camera and vehicle at verified ground elevations, and starts traffic.
* **Atomic Failure Recovery:** If staging or download fails, the error is reported to the HUD and the previous playable scene remains untouched.
* **UI Feedback:** Added `[READY]`, `[DL REQ]`, and `[UNAVAIL]` badges to the City Catalog HUD with dynamic button styling (`▶ WARP TO CITY`, `⬇ INSTALL & WARP`, `🚫 UNAVAILABLE`).

---

## Verification & Certification

All 5 phases of Stage D were certified via dedicated integration test suites:
1. `tests/tests/stage_d_gis_streaming_and_city_transitions.rs`:
   - `test_d1_38_city_catalog_and_completeness_audit`: Audited all 38 US cities, verified San Francisco is `InstalledReady` with SHA-256 integrity, verified Minneapolis is `DownloadRequired` with surveyed IDS Center anchor (241.4m), and validated CRS/datum/license completeness across all 38 entries.
   - `test_d2_on_demand_geographic_installation`: Validated on-demand installation of Minneapolis, progress callbacks, `validate_dataset` pass, compilation of `chunks.db`, SHA-256 verification, `manifest.json` generation, and cancellation.
   - `test_d3_endless_streaming_and_horizon_continuity`: Verified WGS84/ECEF/ENU conversion consistency, 50km camera far plane, and 50km horizon apron mesh encompassing 171 kts flight corridors from `media_1792936143100.png`.
   - `test_d4_transactional_us_city_warping_and_atomic_rollback`: Tested 4-city grand tour (SF -> NYC -> CHI -> LA -> SF) across distinct geodetic coordinates, verified safe spawn elevation querying, and certified atomic rollback on failed warp.
   - `test_d5_no_regression_authoritative_physics_and_assets`: Certified right-handed ENU +Z forward drive acceleration, parking pawl lock under full throttle, caster steering self-centering, and presence of all certified Stage C 3D vehicle models.

### Regression Test Results
- `cargo test --test stage_d_gis_streaming_and_city_transitions` — 5 passed, 0 failed (0.32s)
- `cargo test --test stage_c_premium_models_and_animation` — 6 passed, 0 failed (0.08s)
- `cargo test --test stage_b_authoritative_physics_and_inputs` — 9 passed, 0 failed (0.00s)
- `cargo test --test stage_b_remediation_certification` — 8 passed, 0 failed (0.02s)
- `cargo test --test us_city_catalog_validation` — 9 passed, 0 failed (0.67s)
- `cargo check --workspace` — Exit 0 (all crates compile cleanly)
