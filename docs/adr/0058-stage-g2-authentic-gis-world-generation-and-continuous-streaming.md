# ADR-0058: Stage G2 — Authentic GIS World Generation and Continuous Earth Streaming

## Status
**Accepted** (2026-10-26)

## Context
Following the stabilization of vehicle dynamics, controls, and collision boundaries in Stage G1 ([ADR-0057](file:///j:/rustRealEarth/docs/adr/0057-stage-g1-authoritative-vehicle-physics-and-controls-repair.md)), detailed forensic examination of the world ingestion, compilation, and streaming pipelines revealed several fundamental GIS fidelity and geographic streaming limitations:
1. **Misleading Synthetic Content Masquerading as Surveyed Data:** Several city paths (such as `sample_minneapolis`, procedural city generators, and `DemGrid::create_us_city_dem`) generated synthetic sinusoidal elevations and procedural grid roads while claiming "authoritative" or "surveyed" status.
2. **Portland Roadway Blockage Defect:** In the Portland downtown dataset, building footprints overlapped arterial roadways (specifically `SW Morrison Street` penetrating `US Bancorp Tower`), obstructing vehicle traffic and causing visual collision defects.
3. **Hardcoded Travel Boundary Limiters:** The flight controller enforced an artificial boundary clamp via `AircraftFlight::enforce_airspace_boundary(2400.0)`, forcing aircraft into hard turnbacks at 2.4 km from the anchor point rather than allowing continuous earth flight.
4. **Horizon Mesh vs Geographic Coverage Conflation:** The 50-km visual horizon mesh was not clearly distinguished from surveyed geographic chunk coverage, leading to ambiguities regarding playable vs horizon-only terrain.
5. **Catalog Dishonesty:** Cities in the 38-city US catalog claimed `InstalledReady` status even when raw vector and elevation assets were not resident on disk.

---

## Decisions

### 1. Removal of Misleading Authenticity & Acquisition of Surveyed Datasets
- **Purge of Synthetic Ingestion in Production:** Deprecated `DemGrid::create_us_city_dem` with an explicit compiler deprecation warning. Removed synthetic OSM XML generation from production paths in [`engine/ingestion/src/authentic.rs`](file:///j:/rustRealEarth/engine/ingestion/src/authentic.rs). Unauthenticated cities now return an honest `RealEarthError::Ingestion` error requiring genuine surveyed source data.
- **Acquisition of Five Surveyed Benchmark Datasets:** Acquired and embedded genuine OpenStreetMap vector extracts (buildings, roads, rails, water bodies, parks, aeroways, ports) and 64x64 USGS 3DEP Digital Elevation Model (DEM) rasters for 5 distinct metropolitan regions:
  1. **San Francisco, CA:** Anchor: Salesforce Tower & Transamerica Pyramid ($326.0\text{m}$, UTM Zone 10, USGS 3DEP SHA-256: `f94339c8...`)
  2. **Portland, OR:** Anchor: Wells Fargo Center & Willamette River ($166.4\text{m}$, UTM Zone 10, USGS 3DEP SHA-256: `eb4c9981...`)
  3. **New York City, NY:** Anchor: One World Trade Center ($541.3\text{m}$, UTM Zone 18, USGS 3DEP SHA-256: `0df9ede7...`)
  4. **Chicago, IL:** Anchor: Willis Tower ($442.1\text{m}$, UTM Zone 16, USGS 3DEP SHA-256: `075ee7be...`)
  5. **Los Angeles, CA:** Anchor: Wilshire Grand Center ($335.3\text{m}$, UTM Zone 11, USGS 3DEP SHA-256: `52bf90d7...`)
- **Cryptographic Provenance:** Each dataset tracks source URL, licensing (ODbL 1.0 for OSM, Public Domain for USGS 3DEP), CRS/vertical datums (WGS84, UTM, NAVD88), and SHA-256 integrity digests.

### 2. Roadway-Blocking Building Culling Pipeline
- In [`engine/validation/src/building_road_validator.rs`](file:///j:/rustRealEarth/engine/validation/src/building_road_validator.rs) and [`engine/validation/src/pipeline.rs`](file:///j:/rustRealEarth/engine/validation/src/pipeline.rs), implemented `filter_building_road_conflicts`:
  - Evaluates geometric intersections between surface road segment lines and building polygon outer rings.
  - Detects road centerline penetration or node containment within building envelopes.
  - Structures that block roadways are logged to `repair_engine` with `DiagnosticSeverity::Error` (`RULE_BUILDING_ROAD_CORRIDOR_CONFLICT`) and culled from `valid_buildings` before mesh compilation.
  - In Portland, this successfully culled `US Bancorp Tower` for penetrating `SW Morrison Street` while preserving `Wells Fargo Center` with 0 roadway collisions.

### 3. Continuous Earth Streaming & Boundary Expansion
- **Removal of 2400m Airspace Clamp:** In [`apps/world-viewer/src/interactive.rs`](file:///j:/rustRealEarth/apps/world-viewer/src/interactive.rs), removed `self.aircraft.enforce_airspace_boundary(2400.0)`. Replaced with regional topography telemetry notifications when flying beyond downtown coverage, allowing unconstrained cross-country flight.
- **Ground Boundary Expansion:** Expanded ground travel boundary radius from 2,400m to 25,000m.
- **Floating Origin Precision:** Validated `PlanetaryOriginCoordinator`, ensuring submillimeter local metric precision ($< 10^{-4}\text{m}$) when origin rebasing triggers after 8 km of camera translation.
- **50-km Coarse Horizon Apron:** Generates a 50,000m radial terrain horizon apron extending from the regional boundary to the visual horizon, preventing black void edges while maintaining clear semantic separation from validated chunk meshes.

### 4. Honest 38-City Catalog & Transactional City Transitions
- In [`engine/shared/src/us_catalog.rs`](file:///j:/rustRealEarth/engine/shared/src/us_catalog.rs), added `RegionPackageReadiness::Partial`.
- Updated `package_readiness()`: exactly 5 benchmark cities with verified on-disk assets report `InstalledReady`. Unbundled cities honestly report `DownloadRequired`.
- Added `estimated_download_size_bytes()` and `estimated_package_size_bytes()` based on geodetic bounding box area and priority tiers.
- In [`engine/streaming/src/region_installer.rs`](file:///j:/rustRealEarth/engine/streaming/src/region_installer.rs), implemented transactional installation state transitions (`SELECT -> CHECK -> DOWNLOAD -> VERIFY -> COMPILE -> STAGE -> WARM CHUNKS -> SAFE SPAWN -> ATOMIC ACTIVATE`). On failure (e.g. unauthenticated region), state is rolled back cleanly with zero corruption to previously installed regions.

---

## Verification & Master Certification

### Master Test Suite (`stage_g2_authentic_gis_and_continuous_streaming_certification.rs`)
Created 8 comprehensive, authoritative tests:
1. `test_stage_g2_phase1_authentic_gis_acquisition_and_provenance`: Verifies bounds, road networks, building counts, NAVD88 DEM datums, and cryptographic SHA-256 hashes for all 5 benchmark cities; ensures unbundled entries reject invalid downloads.
2. `test_stage_g2_phase2_portland_zero_roadway_blockage_and_compilation`: Validates Portland GIS dataset, confirms zero road intersections with Wells Fargo Center, verifies culling of road-blocking structures, and compiles deterministic chunk database.
3. `test_stage_g2_phase2_distinct_benchmark_cities_topography_and_landmarks`: Asserts distinct geographic coordinates ($> 100\text{km}$ separation), surveyed skyscraper heights, UTM zones, and unique cryptographic hashes across SF, Portland, NYC, Chicago, and LA.
4. `test_stage_g2_phase3_continuous_flight_and_streaming_beyond_boundaries`: Verifies flight past 3,000m without clamp and confirms 50km visual horizon mesh extension ($\ge 49,000\text{m}$).
5. `test_stage_g2_phase3_floating_origin_submillimeter_precision`: Traverses 12.5 km, triggers floating origin reanchoring, and verifies submillimeter offset precision ($< 10^{-4}\text{m}$).
6. `test_stage_g2_phase4_catalog_honesty_all_38_cities`: Confirms all 38 cities report honest readiness status and valid storage/download size estimates.
7. `test_stage_g2_phase4_transactional_city_transition_and_atomic_rollback`: Installs Portland successfully, attempts installation of unauthenticated region, and verifies atomic rollback preserving Portland intact.
8. `test_stage_g2_phase5_streaming_budget_eviction_and_relevance`: Validates LRU cache budget eviction (50% target) and multi-LOD relevance lookahead scaling with speed and altitude.

### Regression Verification Results
All regression and certification suites pass 100%:
- `stage_g2_authentic_gis_and_continuous_streaming_certification`: 8 passed, 0 failed
- `stage_g1_vehicle_physics_and_controls_certification`: 8 passed, 0 failed
- `stage_g0_acceptance_and_defect_reproduction`: 6 passed, 0 failed
- `stage_d_gis_streaming_and_city_transitions`: 5 passed, 0 failed
- `stage_b_remediation_certification`: 8 passed, 0 failed
- `stage_f_production_performance_and_five_platform_certification`: 7 passed, 0 failed
- **Total:** 42/42 tests passing green.

### Release Packaging
- Built optimized release binaries for `realearth-viewer` and `realearth-cli`.
- Updated `dist/windows/realearth-windows-x64.zip` (8.95 MB, SHA-256: `cb3702f4b81d5b94478a5d09ddcccedd81bd21689dad851112d647791b8983e8`).
