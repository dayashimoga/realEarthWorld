# Changelog
*(Append-Only Document — Never remove historical records)*

All notable changes to the Real Earth World Platform will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-10-21
### Added
- Initialized empty Git repository and `.gitignore`.
- Established workspace `Cargo.toml` with pinned dependencies (`glam`, `wgpu`, `bevy_ecs`, `rapier3d`, `tokio`, `rayon`, `zstd`, `rusqlite`, `geo`, `earcutr`, `petgraph`, `thiserror`, `serde`, `tracing`).
- Created `engine/shared` (`realearth-shared`) containing `Wgs84Coord`, `EcefCoord`, `EnuCoord`, `FloatingOrigin`, `GeoBounds`, and `RealEarthError`.
- Authored foundational documentation suite:
  - `TechnicalRequirements.md`
  - `TechnicalArchitecture.md`
  - `ImplementationPlan.md`
  - `GISStrategy.md`
  - `ValidationStrategy.md`
  - `WorldStreamingStrategy.md`
  - `PerformanceStrategy.md`
  - `TestingStrategy.md`
  - `DataSourcesAndLicenses.md`
  - `SetupAndConfiguration.md`
  - `UserGuide.md`
  - `Troubleshooting.md`
  - `CodeUnderstanding.md`
  - `Todo.md`
  - `Changelog.md`
- Implemented `engine/geospatial` (`realearth-geospatial`):
  - WGS84 Reference Ellipsoid parameters and radii of curvature.
  - Bidirectional Geodetic $\leftrightarrow$ ECEF conversions with Bowring iterative algorithm ($< 10^{-6}\text{ m}$ precision).
  - Local Tangent Plane (ENU) conversion relative to arbitrary anchor origins.
  - UTM forward projection with automatic zone calculation (Zone 43N for Bengaluru).
  - Web Mercator tile calculation and bounding box queries.
  - Vertical datum conversions (ellipsoidal vs. orthometric/EGM96).
  - Unit tests covering Bengaluru landmarks and global edge cases (Poles, Equator, Null Island).
- Implemented `engine/ingestion` (`realearth-ingestion`):
  - Streaming XML parser for OpenStreetMap nodes, ways, and relations.
  - Normalized data models for roads, buildings, and water bodies with highway classifications, speed limits, and lanes.
  - Digital Elevation Model (DEM) raster grid processor with bilinear spatial interpolation.
  - Authentic, bounded Bengaluru Central reference dataset generation and file manifests in `assets/bengaluru_sample/`.
- Implemented `engine/validation` (`realearth-validation`):
  - Road network validator with duplicate consecutive vertex culling.
  - Grade-separation invariant validation: verified overpasses and flyovers never collapse into at-grade junctions.
  - Building polygon footprint validator: automatic unclosed ring repair and clockwise-to-counterclockwise winding order normalization.
  - Footprint metric area calculation and degenerate feature quarantine.
  - Height provenance attribution tagging (`Surveyed`, `InferredLevels`, `DefaultCategory`) with explicit confidence scores.
  - Comprehensive `ValidationReport` with repair audit logs and quarantine records.
- Implemented `engine/compiler` (`realearth-compiler`):
  - 3D vertex and mesh data structures (`Vertex`, `ChunkMesh`) with 16-byte aligned `bytemuck` representations.
  - Building extrusion engine with 3D vertical wall quad generation and `earcutr` polygon roof triangulation.
  - Road ribbon mesher supporting multi-lane road geometry and elevated bridge clearance profiles ($+6.0\text{m}$ clearance).
  - Regular heightfield terrain triangulation conforming to DEM raster models.
  - Versioned binary chunk serialization with `zstd` compression (`.rec` format).
  - Embedded SQLite spatial index database (`chunks.db`) with bounding box spatial queries.
- Implemented `engine/streaming` (`realearth-streaming`):
  - Memory-budgeted LRU chunk cache with heap memory tracking and eviction guarantees.
  - Spatial bounding queries against `chunks.db` SQLite index.
  - Dynamic `WorldStreamer` maintaining active terrain, road, and building meshes around the camera.
  - Level-of-detail (LOD) distance selector.
  - Cache telemetry (memory usage in MB, active chunks, cache hits/misses, and evictions).
- Implemented `engine/simulation` (`realearth-simulation`):
  - Headless Bevy ECS simulation framework (`RealEarthSimWorld`) decoupled from graphics and windowing.
  - Astronomical solar position calculations computing accurate solar elevation, azimuth, and sunlight vectors in ENU.
  - Deterministic fixed 60Hz timestep kinematic motion and entity systems.
  - Atmospheric parameters and day-night cycle progression.
- Implemented `engine/navigation` (`realearth-navigation`):
  - Directed lane-level road network graph using `petgraph`.
  - A* pathfinding algorithm supporting both Shortest Distance and Fastest Time metrics.
  - Navigation route generator producing discrete waypoint trajectories.
- Implemented `engine/vehicles` (`realearth-vehicles`):
  - Kinematic Ackermann / bicycle steering vehicle model.
  - Rapier3D physics world integration with ground collision and dynamic vehicle rigid bodies.
  - Pure pursuit autonomous path tracking controller.
- Implemented `engine/rendering` (`realearth-rendering`):
  - Floating origin camera matrix transformation maintaining double-precision world references and jitter-free relative local camera frames.
  - Modern WGSL shader pipeline with diffuse solar shading, normal calculation, and exponential atmospheric distance fog.
  - Headless offscreen rendering pipeline using `wgpu` with RGBA8 target and Depth32Float buffer.
  - Interactive render target pass for windowed display surfaces.
- Implemented `tools/world-cli` (`realearth-cli`):
  - Multi-command CLI tool supporting `compile`, `validate`, `inspect-chunk`, and `bench`.
  - Comprehensive inspection of binary chunk headers, bounding boxes, mesh counts, and building height provenance.
  - High-throughput benchmarking subcommands for coordinate transforms and zstd decompression.
- Implemented `apps/world-viewer` (`realearth-viewer`):
  - Interactive 3D simulation application centered on Vidhana Soudha, Bengaluru Central.
  - Automated local world package compilation, dynamic chunk streaming, and Bevy ECS simulation loop.
  - Performance telemetry logging FPS, average frame time in ms, resident memory in MB, and triangle counts.
- Implemented `benchmarks` (`realearth-benchmarks`):
  - Criterion microbenchmarks for WGS84 $\leftrightarrow$ ECEF, local ENU, and UTM Zone 43N coordinate transforms.
  - Criterion microbenchmarks for 3D building wall/roof meshing, road ribbon triangulation, and Zstd chunk compression/decompression.
- Implemented Integration Tests (`tests/`):
  - End-to-end integration test (`tests/tests/end_to_end_pipeline.rs`) testing the full vertical slice: Ingestion $\to$ Validation $\to$ Compilation $\to$ Streaming $\to$ Simulation $\to$ Navigation $\to$ Physics $\to$ Headless Rendering.
- Implemented CI/CD and Containerization:
  - Multi-platform GitHub Actions workflows: `build.yml`, `test.yml`, `coverage.yml`, `benchmark.yml`, `security.yml`, `docs.yml`, `release.yml`.
  - Multi-stage `Dockerfile` and `Containerfile` for Podman/Docker headless deployment and builds.
  - Automation scripts for Windows (`.ps1`) and POSIX (`.sh`) in `scripts/` (`setup`, `build`, `test`, `run`, `bench`, `clean`).

## [0.2.0] - 2026-10-21 — STAGE 1: Real US GIS Acquisition and Normalization (San Francisco Pilot)
### Added
- **Architectural Decision Record**:
  - `docs/adr/0006-us-gis-acquisition-and-san-francisco-normalization.md`: Documents multi-region profile architecture, US coordinate normalization (UTM Zone 10N, NAVD88), coastal elevation constraints, and SHA-256 data provenance.
- **Multi-Region Profile Configuration (`realearth-shared`)**:
  - `RegionProfile` struct in `engine/shared/src/region.rs` supporting arbitrary bounding boxes, coordinate anchors, regional UTM projection zones, vertical datums, and elevation extents.
  - Built-in profiles for `san_francisco_downtown()` (US) and `bengaluru_central()` (IN).
- **US Geospatial Normalization & Vertical Datums (`realearth-geospatial`)**:
  - Added `VerticalDatum::Navd88` and `VerticalDatumMetadata::san_francisco_navd88()` (-32.4m GEOID18/EGM96 undulation in Northern California) in `engine/geospatial/src/datum.rs`.
  - Added automated UTM Zone 10N calculation and conversion tests for San Francisco ($lon \approx -122.40^\circ$).
- **US GIS Acquisition & Ingestion Engine (`realearth-ingestion`)**:
  - Expanded OSM data models in `engine/ingestion/src/osm_types.rs`: added `WaterType` (Bay, Ocean, River, Lake), coastline tagging (`is_coastline`), `LandCoverClass` (aligned with USGS NLCD), and `OsmLandCoverFeature`.
  - Added US speed limit parsing in `engine/ingestion/src/osm_parser.rs` converting `mph` strings to integer `km/h`.
  - Added `AcquisitionEngine` in `engine/ingestion/src/acquisition.rs` with FIPS 180-4 SHA-256 checksum computation, file manifest validation, caching, and offline bundle fallbacks.
  - Implemented USGS 3DEP coastal Digital Elevation Model raster grid (`create_san_francisco_coastal_dem`) modeling real San Francisco topography ($0.0\text{m}$ at Embarcadero Bay waterfront up to $105.0\text{m}$ at Nob Hill).
  - Authored authentic San Francisco Downtown & Coastal Waterfront dataset in `engine/ingestion/src/sample_san_francisco.rs` and `assets/san_francisco_sample/` (`manifest.json`, `san_francisco_downtown.osm.xml`). Includes surveyed skyscrapers (Salesforce Tower 326m, Transamerica 260m, 555 California 237m, Ferry Building 75m, Coit Tower 64m), Bay Bridge I-80 elevated approach (Layer 1), Broadway Tunnel subterranean section (Layer -1), and urban parks (Portsmouth Square, Sue Bierman Park).
- **Validation Pipeline (`realearth-validation`)**:
  - Added land cover and water body preservation in `ValidatedDataset`.
  - Validated San Francisco dataset: 6 roads valid, 5 buildings valid, 1 water body, 2 parks, 5 polygon winding repairs to CCW, and 0 quarantined features.
- **Adaptive World Compiler (`realearth-compiler`)**:
  - Dynamically computes metric longitude degree scaling from center latitude ($\cos \phi$).
  - Dynamically calculates minimum and maximum elevation per chunk from DEM heightfields.
  - Added unit test `test_compile_san_francisco_world_package` verifying chunk files and SQLite index for San Francisco.
- **CLI Tooling (`realearth-cli`)**:
  - Added `--region` CLI argument to `compile` and `validate` subcommands, defaulting to `san-francisco` with automatic dataset and DEM routing.
- **End-to-End Integration Suite (`realearth-tests`)**:
  - Added `test_full_san_francisco_end_to_end_gis_vertical_slice` verifying the complete San Francisco GIS pipeline from ingestion through validation, compilation, streaming, simulation, navigation, physics, and headless rendering.
- **Criterion Microbenchmarks (`realearth-benchmarks`)**:
  - Added `wgs84_to_utm_zone10n_sf` and `ellipsoidal_to_navd88_sf` benchmarks.






