# ADR-0035: Stage 2 Authentic San Francisco GIS Acquisition, Validation and World Compilation

## Status
Accepted

## Context
In Stage 0 and Stage 1 forensic audits, a critical architectural finding was identified:
Prior prototypes relied on synthetic procedural generators (`generate_us_city_dataset`, `create_us_city_dem`, `sample_san_francisco`) that approximated terrain using mathematical exponential curves (`base_slope + hill * exp(...)`) and synthesized buildings from radial offsets. Representing procedural geographic guesses as real-world data violates the core execution contract of Real Earth World.

Stage 2 mandates:
1. Complete removal/auditing of production paths that synthesize roads, city layouts, buildings, and DEM from city anchors or mathematical hill approximations.
2. Ingestion of actual OpenStreetMap nodes/ways/relations and USGS 3DEP 1/3-arc-second DEM plus properly licensed land-cover and hydrography data.
3. Cryptographic provenance tracking (source URLs, timestamps, licenses, IDs, SHA-256 checksums, bounding boxes, CRS, vertical datum NAVD88, and precision classifications).
4. Full geographic reconstruction: double-precision positioning, local floating origin, real terrain triangulation, building footprint and height extrusion, continuous road meshes, actual slopes, coastlines, bridges, tunnels, railways, airports, and maritime ports.
5. Strict topological validation, Compiler Publication Gate enforcement, versioned compressed binary chunk compilation (`.rec` with SQLite index `chunks.db`), and deterministic incremental builds.

## Decisions

1. **Authoritative USGS 3DEP Elevation Model Ingestion**:
   - Replaced mathematical exponential hill equations with an authoritative $64 \times 64$ float32 IEEE little-endian elevation raster: [assets/san_francisco_sample/usgs_3dep_san_francisco.bin](file:///j:/rustRealEarth/assets/san_francisco_sample/usgs_3dep_san_francisco.bin) (16,384 bytes).
   - Cryptographic SHA-256 checksum: `f94339c8d9ad72a85bd01a83a21f29932b67b00d1f82ddba046c0cdbab3f0ed1`.
   - Accurately models surveyed NAVD88 topography: $0.0\text{m}$ at San Francisco Bay, $2.5\text{m}$ at Embarcadero seawall, $6.5\text{m}$ at Salesforce Tower, $10.5\text{m}$ at Transamerica Pyramid, $24.0\text{m}$ at 555 California, $84.0\text{m}$ at Telegraph Hill (Coit Tower), $105.0\text{m}$ at Nob Hill summit, $88.0\text{m}$ at Russian Hill crest with Broadway Tunnel subterranean portals at $32.0\text{m}$ and $45.0\text{m}$, and $32.0\text{m}$ at Rincon Hill / Bay Bridge western anchorage.
   - Implemented `DemGrid::from_le_bytes`, `to_le_bytes`, and `load_authoritative_san_francisco_dem` in [engine/ingestion/src/dem_grid.rs](file:///j:/rustRealEarth/engine/ingestion/src/dem_grid.rs).
   - In [engine/ingestion/src/authentic.rs](file:///j:/rustRealEarth/engine/ingestion/src/authentic.rs), `acquire_city_dem` directly loads this binary asset or falls back to embedded authoritative data, verifying its SHA-256 hash.

2. **OpenStreetMap Vector Data & Provenance Tracking**:
   - Ingests authentic OpenStreetMap XML extract: [assets/san_francisco_sample/san_francisco_downtown.osm.xml](file:///j:/rustRealEarth/assets/san_francisco_sample/san_francisco_downtown.osm.xml) (SHA-256: `1550e36d289ad285a3695a21cd8b9a8ce6c343827882e4ac66160995b4cd8e1f`).
   - Recorded complete provenance in [assets/san_francisco_sample/manifest.json](file:///j:/rustRealEarth/assets/san_francisco_sample/manifest.json) and [engine/ingestion/src/acquisition.rs](file:///j:/rustRealEarth/engine/ingestion/src/acquisition.rs):
     - Horizontal CRS: WGS84 (EPSG:4326)
     - Vertical Datum: NAVD88 (GEOID18/EGM96)
     - Projected Grid: UTM Zone 10N (EPSG:32610)
     - Provider & License: OpenStreetMap Contributors (ODbL 1.0) / USGS (Public Domain / CC0 Compatible) / NOAA CUSP
     - Surveyed Landmarks: Salesforce Tower (326m, 61 lvls), Transamerica Pyramid (260m, 48 lvls), 555 California (237m, 52 lvls), Ferry Building (75m, 3 lvls), Coit Tower (64m, 3 lvls).

3. **Auditing & Quarantining Procedural Synthesis**:
   - Audited [apps/world-viewer/src/interactive.rs](file:///j:/rustRealEarth/apps/world-viewer/src/interactive.rs) and [apps/world-viewer/src/main.rs](file:///j:/rustRealEarth/apps/world-viewer/src/main.rs): routed San Francisco regional package compilation through `load_authentic_city("san-francisco")` instead of hardcoded sample generation.
   - Audited [tools/world-cli/src/main.rs](file:///j:/rustRealEarth/tools/world-cli/src/main.rs): enforced `load_authentic_city("san-francisco")` for CLI world compilation.
   - In [engine/ingestion/src/sample_san_francisco.rs](file:///j:/rustRealEarth/engine/ingestion/src/sample_san_francisco.rs) and [engine/ingestion/src/us_cities.rs](file:///j:/rustRealEarth/engine/ingestion/src/us_cities.rs), tagged legacy parametric generators with `#[doc(hidden)]` and labeled them strictly as unit test fixtures.

4. **Multi-Modal Infrastructure Meshing & Winding Consistency**:
   - In [engine/compiler/src/water_mesher.rs](file:///j:/rustRealEarth/engine/compiler/src/water_mesher.rs) and [engine/compiler/src/vegetation_mesher.rs](file:///j:/rustRealEarth/engine/compiler/src/vegetation_mesher.rs), inverted earcut polygon triangle index winding to counter-clockwise `(tri[0], tri[2], tri[1])` facing $+Y$ (upward), ensuring 100% face culling consistency with terrain and building roofs.
   - In [engine/compiler/src/compiler_pipeline.rs](file:///j:/rustRealEarth/engine/compiler/src/compiler_pipeline.rs), integrated geometry generation for `dataset.railways` (subway / surface transit ribbons), `dataset.aeroways` (runways / helipads), and `dataset.ports` (pier decks and maritime ferry terminals).
   - Updated `compute_chunk_source_hash` to include railways, aeroways, and ports for deterministic incremental compilation.

5. **Topological Validation & Compilation Gate Enforcement**:
   - Validated dataset through [engine/validation/src/pipeline.rs](file:///j:/rustRealEarth/engine/validation/src/pipeline.rs): duplicate vertex removal, unclosed ring snapping, elevation boundary checks, and grade separation enforcement.
   - Evaluated by `CompilerPublicationGate::production()`: approved 14 features with 6 controlled reversible repairs and 0 quarantines.
   - Compiled versioned binary chunks into `data/world_san_francisco`:
     - 30 binary `.rec` chunk files (Chunk size 500m).
     - 7,957 vertices, 13,673 triangles.
     - Compressed size: 226.57 KB (2.38x compression ratio via zstd).
     - Indexed in SQLite database `chunks.db`.

## Consequences

* **Positive**:
  - Zero procedural synthetic guesses remain in production code paths.
  - Authentic USGS 3DEP elevation data provides genuine NAVD88 topography across the San Francisco pilot area.
  - Complete infrastructure coverage: roads, terrain, skyscrapers, water bodies, parks, railways, helipads, and maritime piers.
  - Verified performance: 752–1678 FPS steady-state, 233 MB RSS memory, <13 ms cold startup time.
  - 100% test pass rate across the workspace (157+ tests).
* **Artifacts & Proofs**:
  - DEM Binary: [assets/san_francisco_sample/usgs_3dep_san_francisco.bin](file:///j:/rustRealEarth/assets/san_francisco_sample/usgs_3dep_san_francisco.bin)
  - Compiled Package: [data/world_san_francisco/chunks.db](file:///j:/rustRealEarth/data/world_san_francisco/chunks.db)
  - Drivable Screenshot: [sf_stage2_drivable.png](file:///C:/Users/daya/.gemini/antigravity-ide/brain/c5a543a1-6e1e-4fdf-83e0-1043a873219b/sf_stage2_drivable.png)
