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

## [0.3.0] - 2026-10-21 — STAGE 3: Functional America San Francisco World Viewer
### Added
- **Architectural Decision Record**:
  - `docs/adr/0007-functional-san-francisco-world-viewer.md`: Documents Bevy ECS entity mapping, Gribb-Hartmann frustum culling, dedicated surface meshers, dual-mode floating camera, and deterministic offscreen PPM framebuffer export.
- **Surface Meshers (`realearth-compiler`)**:
  - Created `engine/compiler/src/water_mesher.rs`: Triangulates coastal water polygons (San Francisco Bay) locked to $0.0\text{m}$ MSL with ocean blue vertex coloration.
  - Created `engine/compiler/src/vegetation_mesher.rs`: Triangulates urban park areas (Sue Bierman Park, Portsmouth Square) with organic foliage green coloration and $+0.35\text{m}$ vertical offset over DEM to prevent Z-fighting.
  - Added `water` and `vegetation` mesh slots to `WorldChunk`, updating total vertex and triangle calculations.
  - Updated compiler pipeline to synthesize water and vegetation alongside terrain, roads, and buildings.
- **Frustum Culling & Shader Lighting (`realearth-rendering`)**:
  - Created `engine/rendering/src/culling.rs`: Implemented 6-plane normalized Gribb-Hartmann frustum extraction from the camera's `view_proj` matrix with `intersects_sphere` and `intersects_aabb` testing.
  - Added `camera.frustum()` method in `engine/rendering/src/camera.rs`.
  - Added `render_chunks_culled` in `engine/rendering/src/renderer.rs` issuing draw commands only for visible chunks and returning the visible count.
  - Enhanced WGSL shaders with Blinn-Phong specular solar highlights on reflective water and building glass, ambient base illumination, directional sun lighting, and distance fog.
- **Bevy ECS Scene Entities (`realearth-simulation`)**:
  - Added `ChunkEntity`, `TerrainMeshComponent`, `RoadMeshComponent`, `BuildingMeshComponent`, `WaterMeshComponent`, `VegetationMeshComponent`, and `BoundingSphere` components in `engine/simulation/src/components.rs`.
  - Added `spawn_chunk_entity` in `engine/simulation/src/sim_world.rs` to maintain synchronized ECS representations of active streamed chunks.
- **Stage 3 World Viewer Executable (`apps/world-viewer`)**:
  - Created `apps/world-viewer/src/camera_controller.rs`: Dual-mode camera navigation (`CameraMode::Fly` 6-DOF and `CameraMode::Walk` pedestrian ground clamping at $+1.8\text{m}$ eye level).
  - Added real-time WGS84 coordinate display and instant landmark jump navigation keys (`1`..`6` for San Francisco landmarks, `7` for Bengaluru Vidhana Soudha).
  - Created `apps/world-viewer/src/screenshot.rs`: Offscreen framebuffer staging buffer copy with 256-byte row alignment and Netpbm binary PPM export (`capture_texture_to_ppm`).
  - Updated `apps/world-viewer/src/main.rs`: Full CLI argument parsing (`--region`, `--headless`, `--benchmark`, `--camera-mode`, `--location`, `--frames`), Bevy ECS chunk sync, telemetry overlay, and automated tests.
  - Added unit tests: `test_camera_controller_fly_and_walk_modes`, `test_camera_controller_landmarks_and_coordinates`, and `test_frustum_culling_logic`.

## [0.4.0] - 2026-10-21 — STAGE 4: Comprehensive GIS Validation and Controlled Repair
### Added
- **Architectural Decision Record**:
  - `docs/adr/0008-comprehensive-gis-validation-and-controlled-repair.md`: Documents modular deep validators, transactional controlled repair engine, multi-format diagnostics, source-resolution tolerances, and compiler publication gates.
- **Extended Ingestion Data Models (`realearth-ingestion`)**:
  - Added `OsmRailwayFeature` and `RailwayType` (Mainline, LightRail, Subway, Tram, HighSpeed, Spur) supporting track geometry, layers, and gauge.
  - Added `OsmAerowayFeature` and `AerowayType` (Runway, Taxiway, Apron, Helipad, Gate, Terminal) with length/width metrics.
  - Added `OsmPortFeature` and `PortType` (Pier, Dock, Berth, Slipway, FerryTerminal) with depth and closed polygon boundaries.
  - Updated `IngestedDataset` and XML parser to seamlessly ingest railway, aeroway, and maritime infrastructure.
- **Modular Deep Validators (`realearth-validation`)**:
  - `road_network_validator.rs`: Evaluates network graph connectivity, dead-end stubs, lane count sanity ($\ge 1$), speed limit ranges ($5 \le v \le 160$ km/h), one-way restrictions, acute hairpin junctions ($< 15^\circ$), duplicate vertex culling, and grade separation invariance.
  - `railway_validator.rs`: Evaluates track vertex counts, minimum curve radii, gauge sanity ($600 \le g \le 2140$ mm), and at-grade rail-highway crossings.
  - `building_deep_validator.rs`: Evaluates unclosed rings, CCW winding order, self-intersecting bow-tie / figure-8 rings, footprint ground area ($\ge 4.0\text{ m}^2$), DEM foundation slopes ($< 45^\circ$), coastal sea-level feasibility ($\ge 0.0\text{m}$ MSL), and inter-building footprint overlap collisions.
  - `terrain_water_validator.rs`: Evaluates DEM raster slope continuity, step-discontinuity seam cracks, and water polygon closure and CCW winding.
  - `aeroway_port_validator.rs`: Evaluates runway length ($\ge 300\text{m}$), standard 45m runway width, and closed port/pier polygons.
  - `chunk_integrity_validator.rs`: Evaluates coordinate continuity across local chunk grid boundaries ($< 1\text{mm}$) and duplicate feature IDs.
- **Controlled Reversible Repair Engine (`repair_engine.rs`)**:
  - Deterministic repairs with before/after geometry snapshots (`GeometrySnapshot`), explicit rule IDs, confidence scoring ($0.0$ to $1.0$), and reversible transactions.
  - Strict quarantine policy isolating ambiguous defects (confidence $< 0.80$); zero infrastructure fabrication.
- **Hierarchical Diagnostics & Multi-Format Reporting (`diagnostic.rs`, `report.rs`)**:
  - Severity classifications: `Blocker`, `Error`, `Warning`, `Info`.
  - Machine-readable JSON output (`to_json()`).
  - GitHub-flavored Markdown output (`to_markdown()`).
  - Standalone styled HTML dashboard with KPI cards and severity-badged tables (`to_html()`).
  - Geolocated 3D viewer debug overlays (`to_overlay()`).
  - Source-resolution-aware tolerance profiles: `Surveyed` (0.05m snap), `Satellite` (0.5m snap), `Coarse` (2.0m snap).
- **Compiler Publication Gate (`publication_gate.rs`)**:
  - `CompilerPublicationGate` enforcing zero Blocker defects and controlled Error quarantine budgets prior to world package compilation.
## [0.5.0] - 2026-10-21 — STAGE 5: Production World Streaming and Performance Optimization
### Added
- **Architectural Decision Record**:
  - `docs/adr/0009-production-world-streaming-and-performance-optimization.md`: Documents hierarchical spatial indexing, lookahead prefetching, prioritized cancellable Tokio I/O with Rayon parallel decoding, dual CPU/GPU memory accounting, collision-critical chunk pinning, LOD hysteresis, persistent GPU buffer caching, and planetary scale demonstration (100K and 1M managed objects).
- **Production World Streaming Engine (`realearth-streaming`)**:
  - `prefetch.rs`: Velocity- and direction-aware prefetching lookahead cone: $\mathbf{p}_{\text{future}} = \mathbf{p}_{\text{cam}} + \mathbf{v} \cdot t_{\text{lookahead}}$. Computes anisotropic bounding box covering omnidirectional view distance and forward trajectory lobe.
  - `lod.rs`: Multi-LOD heuristics and deadband distance hysteresis ($\Delta_{\text{hyst}} = 100\text{m}$) eliminating ping-pong popping at LOD0/LOD1/LOD2 boundaries.
  - `async_loader.rs`: Prioritized cancellable chunk loading queue (`ChunkLoadQueue`) with automatic cancellation of non-critical prefetch requests on camera teleportation.
  - Corruption recovery: `read_chunk_with_corruption_recovery` detects corrupt/truncated chunks and generates continuous safe fallback geometry with audit logging, preventing physics fall-throughs and runtime crashes.
  - `lru_cache.rs`: Dual CPU heap and GPU VRAM accounting with collision-critical chunk pinning (`pinned: bool`). Pinned chunks are guaranteed never to be evicted while active physics bodies or camera contacts reference them.
  - `streamer.rs`: Rayon data-parallel decoding across background worker threads with bounded backpressure (`max_in_flight_tasks: 16`), ensuring zero render-thread blocking.
- **Rendering Optimization & Graphics Quality Presets (`realearth-rendering`)**:
  - `preset.rs`: Graphics quality presets (`Low`, `Medium`, `High`, `Adaptive`). Includes dynamic adaptive frame-time regulator continuously maintaining target 60 FPS ($\le 16.67\text{ ms}$).
  - `renderer.rs`: Persistent GPU buffer caching (`GpuChunkBuffers`, `GpuChunkMesh`). Meshes are uploaded to VRAM once upon arrival and rendered via single-pass batched draw calls with zero per-frame buffer allocation churn.
  - `shaders.rs`: Enhanced WGSL shaders with procedural road markings (double yellow centerlines, white dashed lane dividers), Fresnel specular water reflections, day/night solar transitions, and atmospheric Rayleigh haze.
  - `uniforms.rs`: Extended `SceneUniforms` with `material_params` for runtime preset and lighting control.
- **Profiling, Telemetry & Scenarios (`apps/world-viewer`)**:
  - Integrated statistical percentile tracking: average framerate, p50 (median), p95, and p99 (tail latency).
  - Profiling scenarios: `--scenario <dense-city|highway|teleport|soak|standard>`.
  - Memory telemetry: tracking CPU resident heap (MB), GPU VRAM (MB), cold startup time (ms), draw calls, scene triangles, cache hits/misses, and prefetch cancellations.
- **Planetary Scaling & Regression Test Suite (`tests/tests/streaming_performance.rs`)**:
  - Verified 100,000 managed objects maintaining $\ge 60\text{ FPS}$ and $< 1024\text{ MB}$ RSS.
  - Verified 1,000,000 managed objects maintaining $\ge 60\text{ FPS}$ and $< 1024\text{ MB}$ RSS.
  - Verified high-speed highway traversal ($108\text{ km/h}$) with lookahead prefetching.
  - Verified rapid teleportation with instant cancellation of stale prefetch queues.
  - Verified LOD hysteresis preventing boundary ping-pong popping.
  - Verified collision-critical chunk pinning preventing eviction under extreme memory pressure.
  - Verified 300+ frame soak testing proving zero memory leaks and constant memory plateau.
- **Criterion Benchmark Suite (`benchmarks/benches/streaming_bench.rs`)**:
  - Microbenchmarks for predictive prefetch bounds calculation, LOD hysteresis selection, and LRU cache operations.

## [0.6.0] - 2026-10-21 — STAGE 6: Complete Drivable Car Simulation
### Added
- **Architectural Decision Record**:
  - `docs/adr/0010-complete-drivable-car-simulation.md`: Comprehensive documentation of fixed-timestep ECS simulation, physics/presentation separation with $\alpha$-interpolation, 4-wheel independent raycast suspension dynamics, Pacejka friction circle tire model, spatial collision filtering, Intelligent Driver Model (IDM) traffic simulation, and Bengaluru connected road network driving.
- **Fixed-Timestep Simulation & Physics Integration (`realearth-simulation`, `realearth-vehicles`)**:
  - `sim_world.rs`: Introduced `FixedTimestepAccumulator` resource running physics at 60 Hz independent of variable display framerates, with smooth accumulator carry-over.
  - `components.rs`: Introduced `InterpolatedTransform` component supporting linear translation interpolation ($\mathbf{p} = (1 - \alpha)\mathbf{p}_{N-1} + \alpha\mathbf{p}_N$) and shortest-path angular slerp.
  - `physics.rs`: Integrated Rapier3D physics engine with `QueryPipeline` and collision interaction groups (`GROUP_VEHICLE`, `GROUP_TERRAIN`, `GROUP_ROAD`, `GROUP_BUILDING`, `GROUP_OBSTACLE`).
  - Safe Spawning & Clamping: Implemented `spawn_safe_vehicle` clamping spawn elevation to the highest underlying road/terrain collider $+ \text{half\_height} + \text{clearance}$, preventing physics drop-throughs.
- **Realistic Automotive Vehicle Dynamics (`realearth-vehicles`)**:
  - `dynamics.rs`: 4-wheel independent raycast suspension model with compression velocity damping, spring stiffness ($35,000\text{ N/m}$), damping ($3,200\text{ N}\cdot\text{s/m}$), and dynamic weight transfer.
  - Pacejka & Friction Circle Tire Model: Combined longitudinal traction/braking and lateral cornering forces constrained by $\sqrt{F_x^2 + F_y^2} \le \mu F_z$.
  - Transmission Modes: `Drive` (with 60/40 rear/front torque split), `Reverse` (50% torque), `Neutral`, and `Park` (mechanical lock).
  - Smooth Zero-Speed Braking & Hill Hold: Eliminated Coulomb jitter on slopes via velocity-threshold ramp and static hill-hold lock when stationary under braking.
  - Flip & Crash Recovery: `recover_on_surface` rights overturned vehicles and zeros residual angular velocities without resetting position.
  - Deterministic Snapshotting: `snapshot` and `restore_snapshot` enabling rewinding, replay, and headless determinism.
  - `ground.rs`: `FunctionalGroundQuery` supporting analytical ground functions and bilinear raster DEM elevation queries.
  - `camera.rs`: `VehicleCameraController` implementing spring-damper `Chase`, first-person `Hood`, and front-bumper `Bumper` cameras.
- **Road Navigation & Autonomous Traffic Simulation (`realearth-navigation`, `realearth-vehicles`)**:
  - `road_graph.rs`: Added `RuleOfTheRoad` (`DriveOnLeft` for India/UK vs. `DriveOnRight` for US) and `LaneDescriptor` with lateral offset calculations.
  - `traffic.rs`: Multi-agent autonomous traffic simulation implementing the Intelligent Driver Model (IDM) for safe car-following headway and pure pursuit steering.
  - `sample_bengaluru.rs`: Added connected corridor roads (Cubbon Rd, Dr. Ambedkar Veedhi, Residency Rd) linking Vidhana Soudha, Cubbon Park, Kasturba Rd, MG Road, and Brigade Road into a fully traversable network.
- **World Viewer Integration (`apps/world-viewer`)**:
  - Added `--drive`, `--traffic <count>`, and `--vehicle-cam <chase|hood|bumper>` CLI flags.
  - Integrated real-time automotive telemetry HUD displaying speed (km/h), gear, RPM, position, suspension travel, tire normal forces, and active traffic fleet.
  - Preserved standalone world-viewer operation (runs fly/walk mode at $>600\text{ FPS}$ when `--drive` is omitted).
- **Test Suite (`tests/tests/vehicle_simulation.rs`)**:
  - Full end-to-end integration tests for slope climbing, hill holding, neutral rollback, crash/flip recovery, snapshot determinism, 4-car IDM convoy spacing, elevated flyover grade separation, and connected Bengaluru route driving.

## [0.7.0] - 2026-10-22 — STAGE 7: Geographic Expansion and Intercity Connectivity
### Added
- **Architectural Decision Record**:
  - `docs/adr/0011-geographic-expansion-and-intercity-connectivity.md`: Architecture specification for multi-region ingestion, transport corridors topology, decentralized regional catalogs, and multi-region spatial streaming with global memory accounting.
- **Regional Ingestion & Normalization (`engine/shared`, `engine/ingestion`)**:
  - `region.rs`: Added authentic regional profiles for Mysuru Urban (`KA`), Chennai Central (`TN`), Hyderabad Central (`TG`), and Intercity Corridors (`KA-TN-TG`) with WGS84 bounding boxes, UTM zones (43N/44N), and EGM96 geoid datums.
  - `dem_grid.rs`: Implemented authentic DEM raster models for Mysuru (Deccan plateau + Chamundi Hill 1065m peak), Chennai (coastal flat 0m at Bay of Bengal to 25m inland), Hyderabad (granite plateau 535m, Golconda ridge 610m, Musi River basin 505m), and South India Intercity Corridors (regional river basins and plateau gradients).
  - Authentic Datasets:
    - `sample_mysuru.rs`: Mysore Palace, Chamundi Temple, Mysuru Junction, Devaraja Market, Sayyaji Rao Road, Bengaluru-Nilgiri Rd (NH 766), Mysuru Outer Ring Road (ORR) Layer 1 elevated bypass, and Karanji Lake.
    - `sample_chennai.rs`: Chennai Central Station, Ripon Building, Fort St. George, LIC Building, Poonamallee High Road (EVR Periyar Salai), Anna Salai, Kamarajar Salai, Napier Bridge (Layer 1 elevated viaduct over Cooum River mouth), and Marina Coastline.
    - `sample_hyderabad.rs`: Charminar, Mecca Masjid, HITEC City Cyber Towers, Golconda Fort, PVNR Elevated Expressway (Layer 1 elevated viaduct), Inner Ring Road, Tank Bund Promenade, Nayapul Bridge (Layer 1 elevated over Musi River), and Hussain Sagar lake.
  - `acquisition.rs`: Updated acquisition pipelines with ODbL 1.0, CC-BY 4.0, and Public Domain licensing, provider provenance, and SHA-256 integrity verification manifests.
- **Transport Corridors & Grade Separations (`engine/ingestion/src/sample_corridors.rs`)**:
  - Compiled connecting highways:
    - NH 275 (Bengaluru-Mysuru Expressway): 10-lane access-controlled highway with Ramanagara/Channapatna bypasses and Srirangapatna Kaveri River Bridge (Layer 1 elevated bridge over Kaveri River).
    - NH 48 / NE 7 (Bengaluru-Chennai Highway): 6-lane trunk expressway via Kolar, Mulbagal, Palamaner Eastern Ghats pass, Chittoor, Ranipet Palar River Bridge (Layer 1), Sriperumbudur, and Poonamallee.
    - NH 44 (Bengaluru-Hyderabad North-South Expressway): 6-lane national expressway via Hebbal, Devanahalli, Chikballapur, Anantapur, Kurnool Tungabhadra River Bridge (Layer 1), Shadnagar, Shamshabad, and PVNR Elevated Expressway.
  - City Center Feeder Links: Built 4 zero-gap urban connectors linking Bengaluru MG Road, Mysuru K.R. Circle, Chennai EVR Periyar Salai, and Hyderabad Charminar directly to intercity expressway termini.
- **Decentralized Regional Packaging & Catalogs (`engine/compiler/src/regional_catalog.rs`)**:
  - Implemented `PackageManifest`, `PackageState`, and `RegionalCatalog` supporting independent regional packaging, disk scanning, incremental installation, SHA-256 verification, and uninstallation.
  - Added spatial coverage queries (`find_regions_covering`) resolving intersecting regions for any bounding box.
- **Multi-Region Streaming & Global LRU Eviction (`engine/streaming/src/multi_region.rs`)**:
  - Implemented `MultiRegionStreamer` dispatching bounding box prefetch requests across multiple mounted regional SQLite databases.
  - Namespaced chunk keys (`ChunkKey::for_region`) preventing coordinate collisions across regional origin offsets.
  - Shared LRU cache budget enforcement across all regions with collision-critical chunk pinning and eviction guarantees.
- **CLI & World Viewer Enhancements (`tools/world-cli`, `apps/world-viewer`)**:
  - `world-cli`: Added `catalog` subcommand (`--list`, `--verify`, `--install-manifest`) and updated `compile`/`validate` to support `--region <sf|bengaluru|mysuru|chennai|hyderabad|corridors>`.
  - `world-viewer`: Added multi-region support and 9 fast-travel landmark hotkeys (1..9): Transamerica, Salesforce, Vidhana Soudha, Mysore Palace, Chennai Central & Marina, Charminar & Hussain Sagar, Kaveri River Bridge, Palar River Bridge, and PVNR Expressway.
- **Dedicated Test Target (`tests/tests/intercity_connectivity.rs`)**:
  - 7 comprehensive integration tests verifying all 5 regional packages compilation and SHA-256 integrity, continuous A* road network routing across all 3 major highway corridors without topological discontinuities, incremental catalog packaging, and bounded memory stability during 50-step high-speed drives.

## [0.8.0] - 2026-10-22 — STAGE 8: Modular Gameplay Platform
### Added
- **Architectural Decision Record**:
  - `docs/adr/0012-modular-gameplay-platform.md`: Architectural specification for independent event-driven gameplay platform, decoupled job state machines, player progression, vehicle garage upgrades, save/load schema migrations, and minimal-click HUD systems.
- **Dedicated Gameplay Crate (`engine/gameplay`, `realearth-gameplay`)**:
  - `events.rs`: Strongly-typed `GameplayEvent` variants and thread-safe `GameplayEventBus` enabling decoupled observer callbacks for UI, telemetry, and audio systems.
  - `career.rs`: Four authentic playable careers (`FreeRoam`, `TaxiPassenger`, `PackageDelivery`, `HeavyFreight`) with payout/experience multipliers and `ReputationTier` hierarchy (`Novice`, `Professional`, `Expert`, `Master`).
  - `jobs.rs`: Comprehensive `JobStateMachine` handling full contract lifecycles (`Available`, `Accepted`, `EnRouteToPickup`, `AtPickup`, `Loaded`, `EnRouteToDropoff`, `AtDropoff`, `Completed`, `Failed`, `Abandoned`) with proximity zone checks, vehicle stop condition detection ($< 2.0\text{ km/h}$), dwell boarding timers (2.0s), ride comfort tracking (lateral/longitudinal jerk), dynamic customer tips, and cargo integrity degradation.
  - `destinations.rs`: Surveyed geographic destination perimeters and `JobCatalogGenerator` generating 9 authentic default contracts connecting surveyed landmarks across Bengaluru, Mysuru, Chennai, Hyderabad, South India Corridors, and San Francisco.
  - `economy.rs`: Configurable `EconomyConfig` and `Wallet` managing liquid funds, debit/credit validations, and itemized transaction logs.
  - `profile.rs`: `PlayerProfile` tracking career XP, levels, reputation per career, odometer travel distance, clean-driving streaks, and landmark discoveries.
  - `garage.rs`: `VehicleGarage` managing player fleet across `VehicleCategory` (`EconomySedan`, `LuxuryCab`, `CargoVan`, `HeavyTruck`) with engine, handling, and comfort upgrade tiers.
  - `achievements.rs`: `AchievementRegistry` tracking driving milestones and career achievements (`FIRST_FARE`, `SMOOTH_CHAUFFEUR`, `EXPRESS_COURIER`, `DECCAN_EXPRESS`, `CHENNAI_PORT_HAUL`, `HYDERABAD_EXPRESS`, `CENTURION_KM`, `CLEAN_STREAK`, `LANDMARK_EXPLORER`).
  - `recovery.rs`: `RecoveryDispatcher` roadside assistance towing service safely repositioning immobilized or flipped vehicles onto the nearest surveyed road node with wallet fee deduction and event logging.
  - `save.rs`: `SaveManager` and `SaveMigrationEngine` serializing game state to versioned JSON with forward schema migrations ($V1 \to V2$) and zero data loss.
  - `hud.rs`: `GameplayHudModel` synthesizing minimal-click responsive HUD state (`ActiveObjectiveHud`, `EarningsHud`, `JobBrowserHud`, `DebriefModal`).
- **CLI & World Viewer Enhancements (`tools/world-cli`, `apps/world-viewer`)**:
  - `realearth-cli`: Added `job` subcommand (`--list`, `--inspect <id>`, `--simulate <id>`) enabling deterministic headless contract execution and payout validation.
  - `realearth-viewer`: Added `--career <taxi|delivery|freight|free-roam>` and `--job <id>` flags, live gameplay HUD overlay integration, and comprehensive Stage 8 telemetry output.
- **Dedicated Test Target (`tests/tests/gameplay_careers.rs`)**:
  - 8 comprehensive integration tests covering real surveyed destinations, end-to-end taxi ride with dwell boarding and 5-star comfort, urgent package delivery speed bonuses, heavy freight cargo integrity across Kaveri Bridge, landmark discovery achievements, roadside recovery towing, persistent save/load with V1 to V2 schema migration, and 100-job continuous soak stability.

## [0.9.0] - 2026-10-22 — STAGE 9: Extensible Multi-Transport Simulation
### Added
- **Architectural Decision Record**:
  - `docs/adr/0013-extensible-multi-transport-simulation.md`: Architectural specification for extensible multi-transport simulation across road, railway, marine, and aviation modalities with strict decoupling of physical semantics.
- **Multimodal Road Transport (`engine/vehicles/src/multimodal/road_transport.rs`)**:
  - `RoadVehicleCategory`: Functional classifications for `Motorcycle`, `CitySedan`, `TransitBus`, and `HeavyTruck`.
  - `RoadAccessRestriction`: Operational corridor restrictions (`OpenAccess`, `BusOnly`, `NoHeavyTrucks`, `MotorcycleOnly`, `ExpresswayOnly`).
  - `MotorcycleModel`: 2-wheeled agile dynamic motorcycle model with roll lean dynamics, equilibrium lean solver ($\tan\phi = v^2 / (R g)$), spring-damper stabilization, camber thrust, and low-speed crash recovery.
  - `TransitBusModel`: Heavy passenger transit bus with pneumatic suspension, passenger mass payload ($75\text{ kg}$ per passenger), dual-chamber air brake reservoir pressure dynamics, and swept-path tail off-tracking calculation (`rear_tail_position`).
  - `HeavyTruckModel`: Multi-axle commercial heavy hauler with gross vehicle mass up to $40,000\text{ kg}$, fifth-wheel trailer articulation kinematics ($d\theta_{\text{trailer}}/dt = -(v/L)\sin\theta - \omega_{\text{tractor}}$), dynamic rollover threshold calculation ($a_{\text{crit}} = \frac{T}{2 h_{\text{com}}} g$), rollover detection, and upright recovery.
- **Railway Track Networks & Train Consist Dynamics (`engine/navigation/src/rail_graph.rs`, `engine/vehicles/src/multimodal/train.rs`)**:
  - `RailNetworkGraph`: Dedicated track topology with `RailNode`, `RailTrackSegment`, `RailGauge` (Broad, Standard, Metre), turnout/switch points (`SwitchState`: Straight, Diverging, Locked), automatic block signaling (`SignalAspect`: Clear, Approach, Stop), and road/rail level crossings (`RailLevelCrossing`).
  - `TrainConsist`: Longitudinal railway physics with `TrainCar`, `LocomotiveType` (Electric, DieselElectric, MultipleUnit), continuous rated power curves ($P = F \cdot v$), low-speed tractive effort limits, steel wheel-on-rail adhesion thresholds ($\mu \approx 0.32$), wheel slip detection, Davis resistance equation ($R = A + Bv + Cv^2$), and Westinghouse pneumatic air brake pipe acoustic wave propagation with Automatic Train Protection (ATP) emergency tripping at stop signals.
- **Marine Waterway Fairways & Hydrodynamic Buoyancy (`engine/navigation/src/waterway.rs`, `engine/vehicles/src/multimodal/marine.rs`)**:
  - `WaterwayGraph`: Marine fairway navigation with `WaterwayNode`, `ShippingChannel`, `WaterwayCategory` (OpenSea, DredgedChannel, InlandCanal, HarborBasin), navigation aid buoys (`NavigationAids`: PortLateral, StarboardLateral, SafeWater, CardinalNorth, SectorLight), and `MarinePort` with draft-clearance checking and container berth assignment (`PortBerth`).
  - `MarineVessel`: Hydrodynamic vessel simulation with `MarineVesselType` (CargoContainerShip, PassengerFerry, HarborTugboat), Archimedes buoyancy volume displacement, equilibrium water draft ($T = M / (\rho A C_b)$), transverse metacentric righting moment ($M = mg \cdot GM \sin\phi$), pitch trim stability, harmonic wave-surface interaction, hull hydrodynamic skin friction/wave-making drag, propeller slipstream wash, and rudder hydrofoil lift turning moments.
- **Aviation Air Navigation & 6-DOF Flight Aerodynamics (`engine/navigation/src/air_navigation.rs`, `engine/vehicles/src/multimodal/aviation.rs`)**:
  - `AirNavigationDatabase`: Aeronautical navigation database with `Airport`, `AirportRunway`, magnetic headings, threshold coordinates, airway corridors (`AirCorridorSegment`), and Instrument Landing System (`IlsGuidance`) calculating horizontal localizer and 3.0° glideslope approach deviations.
  - `AircraftFlight`: 6-DOF aircraft aerodynamic flight solver with `AviationCategory` (CommercialAirliner, GeneralAviation, BusinessJet), barometric altitude air density lapse ($\rho(h) = \rho_0 e^{-h/8500}$), dynamic pressure, angle of attack ($\alpha$) and sideslip ($\beta$), non-linear lift polar with post-stall flow separation modeling, ground effect induced drag reduction ($h < b$), control surfaces (elevators, ailerons, rudder, flaps, spoilers), turbofan engine thrust, and tricycle landing gear oleo-pneumatic strut shock compression. Validated zero coupling to road wheel friction models.
- **Dedicated Multi-Transport Integration Test Target (`tests/tests/multi_transport_simulation.rs`)**:
  - Comprehensive integration test suite validating road transport dynamics and access policies, railway track topology and train consist physics, marine fairway navigation and hydrodynamic buoyancy, airport runway ILS approach and 6-DOF flight mechanics, and concurrent multi-modal simulation throughput exceeding $1000\text{ Hz}$ in headless simulation.

## [0.10.0] - 2026-10-22 — STAGE 10: Optional Authoritative Multiplayer
### Added
- **Architectural Decision Record**:
  - `docs/adr/0014-optional-authoritative-multiplayer.md`: Comprehensive architectural specification for optional authoritative multiplayer networking, client-side prediction, Area of Interest spatial partitioning, and anti-cheat defenses without degrading offline single-player independence.
- **Dedicated Networking Crate (`engine/networking`, `realearth-networking`)**:
  - `protocol.rs`: Framing protocol with magic bytes `0x5245414C`, version negotiation (`PROTOCOL_VERSION = 1`), sequence numbering, selective acknowledgment bitmasks, channel segregation (`ReliableOrdered`, `UnreliableSequenced`), serialized message types (`ConnectRequest`, `ConnectAccept`, `HeartbeatPing`, `HeartbeatPong`, `ClientInput`, `StateSnapshot`, `Disconnect`), and strict input frame sanitization rejecting NaNs, infinities, and out-of-range inputs.
  - `session.rs`: Authenticated client session management (`ClientSession`), cryptographic session tokens, connection lifecycle state machine (`Disconnected`, `Connecting`, `Handshaking`, `Connected`, `Reconnecting`, `Terminated`), heartbeat polling with 15-second reconnection grace period, and per-client token-bucket rate limiting (120 Hz burst cap).
  - `interest.rs`: Spatial Area of Interest (AoI) cell management (`InterestManager`, `SpatialCellId`) with grid-based spatial hashing and distance-based query filtering (500m - 1000m ground, 2000m marine, 30km aviation) ensuring efficient multi-player bandwidth consumption.
  - `replication.rs`: Client-side prediction engine (`ClientPredictor`) enabling zero perceived control latency, circular input replay ring buffers, authoritative reconciliation against server state divergence, and jitter-buffered remote entity snapshot interpolation (`RemoteEntityInterpolator`).
  - `server.rs`: Authoritative server simulation tick loop (`AuthoritativeServer`), entity ownership control registry, anti-cheat validation (acceleration clamping, speed-hack rejection, unauthorized entity mutation defense), spatial boundary handoff detection, and tailored per-client AoI snapshot broadcasting.
  - `harness.rs`: Network impairment simulation harness (`SimulatedNetworkLink`, `NetworkImpairmentConfig`) with configurable artificial latency, jitter, packet loss drop rate, duplication, and packet reordering.
- **Dedicated Multiplayer Integration Test Target (`tests/tests/multiplayer_networking.rs`)**:
  - 7 comprehensive integration tests covering packet framing and token-bucket rate limiting, spatial AoI culling and cell boundary handoffs, client prediction and reconciliation under divergence, remote entity snapshot interpolation, anti-cheat defense against malicious inputs and unauthorized entity mutations, network impairment simulation under 15% packet loss, and 16-player concurrent scaling exceeding 500 Hz tick rates with zero single-player socket overhead.

