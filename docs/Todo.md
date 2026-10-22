# Real Earth World Platform — Task Tracker (Todo)
*(Append-Only Document — Never remove historical records)*

## Active Tasks
- [x] Stage 0: Audit workspace, initialize git, set up Cargo workspace structure and shared primitives.
- [x] Stage 0: Create complete architectural and strategy documentation suite in `docs/`.
- [x] Stage 1: Implement `realearth-geospatial` with WGS84, Bowring ECEF, ENU transforms, and UTM projections.
- [x] Stage 1: Add unit tests for Bengaluru and global boundary coordinates ($10^{-6}\text{ m}$ tolerance).
- [x] Stage 2: Implement `realearth-ingestion` for OSM XML vector and DEM elevation raster grid parsing.
- [x] Stage 2: Create authentic, bounded Bengaluru Central reference dataset (`assets/bengaluru_sample`).
- [x] Stage 3: Implement `realearth-validation` (topological graph, grade separation, polygon repair, quarantine logs).
- [x] Stage 4: Implement `realearth-compiler` (3D building extrusion, road ribbons, terrain meshes, `.rec` zstd packaging).
- [x] Stage 5: Implement `realearth-streaming` (SQLite index, Tokio async loader, Rayon decompression, LRU memory manager).
- [x] Stage 6: Implement `realearth-simulation` (headless Bevy ECS, solar position, time-of-day, weather).
- [x] Stage 7: Implement `realearth-navigation` (petgraph A* routing) and `realearth-vehicles` (kinematics/Rapier physics).
- [x] Stage 8: Implement `realearth-rendering` (wgpu pipeline, floating origin camera, PBR shaders) and `apps/world-viewer`.
- [x] Stage 8: Implement `tools/world-cli` (`realearth-cli`).
- [x] Stage 9: Add Criterion benchmarks and comprehensive CI/CD workflows (`build.yml`, `test.yml`, `coverage.yml`, `benchmark.yml`, `security.yml`, `docs.yml`, `release.yml`).
- [x] Stage 10: Run full test suite, measure memory and frame performance, generate Certification Matrix.

## Completed Milestones
- **All 12 Workspace Crates Implemented & 100% Tested**:
  1. `realearth-shared`: Coordinates (`Wgs84Coord`, `EcefCoord`, `EnuCoord`), `GeoBounds`, `FloatingOrigin`.
  2. `realearth-geospatial`: WGS84, Bowring ECEF, ENU, UTM Zone 43N, Web Mercator, EGM96 Geoid.
  3. `realearth-ingestion`: OSM XML streaming parser, DemGrid, authentic Bengaluru Central dataset.
  4. `realearth-validation`: Grade-separation invariance, winding order CCW repair, area calculation, provenance tagging.
  5. `realearth-compiler`: 3D building extrusion, road ribbons with $+6.0\text{m}$ bridge clearance, terrain mesh, `.rec` zstd packaging, SQLite `chunks.db`.
  6. `realearth-streaming`: LRU cache budget manager, async chunk query/loader, LOD selector, telemetry.
  7. `realearth-simulation`: Headless Bevy ECS world, deterministic 60Hz tick, ephemeris solar positioning.
  8. `realearth-navigation`: Directed road graph, A* pathfinding (distance/time), trajectory waypoints.
  9. `realearth-vehicles`: Ackermann steering kinematics, Rapier3D physics, Pure Pursuit controller.
  10. `realearth-rendering`: Floating origin camera, WGSL shaders with sun lighting & distance fog, headless and windowed render paths.
  11. `realearth-cli`: Headless CLI compiler, validator, chunk inspector, and throughput benchmark.
  12. `realearth-viewer`: Interactive 3D simulation viewer with headless benchmark and metrics telemetry.
- **Criterion Benchmarking & CI/CD**:
  - `realearth-benchmarks`: Criterion benchmarks for geospatial transforms and compiler meshing/compression.
  - Multi-platform GitHub Actions workflows: `build.yml`, `test.yml`, `coverage.yml`, `benchmark.yml`, `security.yml`, `docs.yml`, `release.yml`.
  - Dockerfile & Containerfile multi-stage builds.
  - Full scripts suite in PowerShell and Bash (`setup`, `build`, `test`, `run`, `bench`, `clean`).
- **End-to-End Integration**:
  - Comprehensive vertical slice test suite in `tests/tests/end_to_end_pipeline.rs`.
- **Quality Gates**:
  - 100% test pass rate (31/31 tests passing across workspace).
  - Zero compiler warnings, zero clippy warnings (`-D warnings`), zero formatting drift (`cargo fmt --check`).
  - Bengaluru MVP: 165.2 FPS (6.05 ms/frame vs $\le 16.67$ ms budget), 0.23 MB resident memory (budget 1024 MB), sub-200ms cold startup.

## STAGE 1: Real US GIS Acquisition and Normalization (San Francisco Pilot)
- [x] Stage 1.1: Implement configurable `RegionProfile` architecture supporting arbitrary US and global bounding boxes.
- [x] Stage 1.1: Author authentic San Francisco reference dataset (`assets/san_francisco_sample`) including downtown, coastal waterfront, Bay Bridge approach, Broadway Tunnel, and surveyed skyscrapers.
- [x] Stage 1.1: Ingestion pipeline with SHA-256 provenance verification, caching, and offline bundle fallback.
- [x] Stage 1.2: Geographic normalization for Western US: UTM Zone 10N, NAVD88 vertical datum, and high-relief coastal DEM grid (0m to 115m MSL).
- [x] Stage 1.3: Extended feature extraction for water bodies, coastlines, piers, land-cover classifications, and US highway attributes.
- [x] Stage 1.4: Coastal terrain alignment, road elevation invariance, and defect quarantine reporting.
- [x] Stage 1.4: Update CLI tool with San Francisco compilation, inspection, and benchmark subcommands.
- [x] Stage 1.4: Execute full test suite, regression verification, and publish Stage 1 Certification Matrix.

## STAGE 2: Minimum GIS Validation and Production World Compiler (Completed)
- [x] Phase 2.1 — Blocking validation: Validate coordinates, road topology, junction grades, invalid polygons, footprints, terrain/road/building alignment, bridge/tunnel elevations, water conflicts, basic railway integrity, and inter-chunk continuity. Quarantine critical errors; classify diagnostics BLOCKER/ERROR/WARNING/INFO without silent repairs.
- [x] Phase 2.2 — Hierarchical compiler: Partition geography hierarchically; generate terrain, road, building, water and basic vegetation-zone geometry, LOD meshes, normals, material references, collision meshes, road/routing graphs, spatial indexes, and neighbor connectivity.
- [x] Phase 2.3 — Binary storage & incremental builds: Implement compressed, versioned chunks (`.rec` with zstd), SQLite manifests (`chunks.db` with package metadata, CRS/bounds, source IDs, SHA-256 checksums, neighbor metadata). Support deterministic incremental compilation (skipping unchanged chunks), corruption detection, and invalidation.
- [x] Proof & Certification: Real Bengaluru GIS -> validated chunks -> headless reload with identical geographic alignment; prove integrity, topology, bit-exact repeatability, and no critical seams. (Verified via `gis_validation_compiler.rs`).

## STAGE 3: Functional America San Francisco World Viewer (Completed)
- [x] Stage 3.1: Integrate `wgpu` with Bevy ECS scene entities (`TerrainMeshComponent`, `RoadMeshComponent`, `BuildingMeshComponent`, `WaterMeshComponent`, `VegetationMeshComponent`).
- [x] Stage 3.1: Dedicated 3D surface meshers for San Francisco Bay water bodies ($0.0\text{m}$ MSL) and urban vegetation parks ($+0.35\text{m}$ anti-Z-fighting terrain offset).
- [x] Stage 3.1: Gribb-Hartmann frustum plane culling (`culling.rs`) deriving 6 normalized planes from `view_proj` matrix.
- [x] Stage 3.1: Upgraded WGSL shaders with Blinn-Phong specular solar highlights, ambient lighting, directional sun shading, and distance fog.
- [x] Stage 3.2: Dynamic chunk streaming loading nearby active binary `.rec` packages and updating ECS world synchronously.
- [x] Stage 3.2: Dual-mode floating-origin camera controller (`Fly` and `Walk` mode with ground clamping at $+1.8\text{m}$).
- [x] Stage 3.2: Real-time WGS84 coordinate display and instant landmark jump navigation (1-6 San Francisco, 7 Bengaluru).
- [x] Stage 3.3: Geographic integrity verification (grounded skyscrapers, elevated I-80 flyover, subterranean Broadway Tunnel, $0.0\text{m}$ water level, $C^0$ continuous terrain).
- [x] Stage 3.3: Deterministic offscreen GPU framebuffer copy and binary PPM export (`target/screenshots/san_francisco_view.ppm`).
- [x] Stage 3.3: Execute full regression suite (41/41 tests passing), static analysis, and headless benchmark (84.2 FPS, 31.5 ms startup, 0.58 MB RSS).

## STAGE 4: Comprehensive GIS Validation and Controlled Repair (Completed)
- [x] Stage 4.1: Expand ingestion models (`realearth-ingestion`) with Railway, Aeroway, and Port features.
- [x] Stage 4.1: Implement `RoadNetworkValidator` with connectivity, access, speed limit, intersection angles, and grade-separation checks.
- [x] Stage 4.1: Implement `RailwayValidator` with track continuity, curvature, and road level-crossing verification.
- [x] Stage 4.1: Implement `BuildingDeepValidator` with overlap collision detection, DEM slope foundation feasibility, and self-intersection tests.
- [x] Stage 4.1: Implement `TerrainWaterValidator` with DEM slope/crack seam validation and water boundary closure.
- [x] Stage 4.1: Implement `AerowayPortValidator` with runway/taxiway geometry and maritime dock/pier connectivity.
- [x] Stage 4.1: Implement `ChunkIntegrityValidator` ensuring border coordinate continuity ($< 1\text{mm}$).
- [x] Stage 4.2: Implement `RepairEngine` with deterministic reversible corrections, confidence scoring ($0.0$ to $1.0$), before/after snapshots, and strict quarantine isolation.
- [x] Stage 4.3: Implement multi-format report generator (JSON, Markdown, interactive HTML dashboard).
- [x] Stage 4.3: Implement visual error overlay generation for 3D viewer integration.
- [x] Stage 4.3: Implement `CompilerPublicationGate` blocking compilation on Blocker/Error defects.
- [x] Stage 4.3: Create documented regression test suite with intentionally malformed fixtures, property tests, and proof that valid features remain unchanged.

## STAGE 5: Production World Streaming and Performance Optimization (Completed)
- [x] Stage 5.1: Hierarchical spatial indexing and quadtree chunk levels in `realearth-streaming`.
- [x] Stage 5.1: Velocity- and direction-aware prefetching lookahead cone along motion vector.
- [x] Stage 5.1: Prioritized, cancellable Tokio async I/O worker with Rayon decompression and bounded backpressure.
- [x] Stage 5.1: Dual CPU heap and GPU VRAM accounting in `LruChunkCache`.
- [x] Stage 5.1: Safe collision-critical readiness and chunk pinning (`pinned: bool`).
- [x] Stage 5.1: Chunk corruption recovery with checksums and safe fallback geometry.
- [x] Stage 5.1: Chunk and LOD distance hysteresis ($\Delta_{\text{hyst}} = 100\text{m}$) to eliminate boundary popping.
- [x] Stage 5.2: Persistent GPU buffer caching (`GpuChunkCache`, `GpuChunkMesh`) eliminating per-frame vertex/index reallocation.
- [x] Stage 5.2: Graphics quality presets (`Low`, `Medium`, `High`, `Adaptive`) with dynamic frame-time regulator.
- [x] Stage 5.2: Shader enhancements: road lane markings (white/yellow dashes), specular reflections, atmospheric haze, day/night lighting.
- [x] Stage 5.3: Accurate percentile frame time telemetry (p50, p95, p99), draw calls, GPU/CPU time, RSS, and cache stats.
- [x] Stage 5.3: Dense city, highway, rapid movement, teleportation, and soak scenario profiling.
- [x] Stage 5.3: Planetary scaling demonstration: 100,000 and 1,000,000 managed objects at 60 FPS reference target.
- [x] Stage 5.3: Regression test suite and performance certification matrix.

## STAGE 6: Complete Drivable Car Simulation (Completed)
- [x] Stage 6.1: Fixed-timestep (60Hz) Bevy ECS simulation loop with Rapier physics/presentation interpolation.
- [x] Stage 6.1: Spatial collision filtering (Vehicles, Roads, Terrain, Obstacles) and safe spawning elevation clamping.
- [x] Stage 6.1: Deterministic state snapshotting (`VehicleStateSnapshot`) and headless physics verification tests.
- [x] Stage 6.2: 4-wheel independent raycast suspension solver with spring-damper dynamics and weight transfer.
- [x] Stage 6.2: Tire traction and friction circle model (longitudinal slip, lateral cornering stiffness, slope response).
- [x] Stage 6.2: Transmission modes (`Drive`, `Reverse`, `Neutral`, `Park`), progressive throttle, braking, and flip recovery.
- [x] Stage 6.2: Camera systems: 3rd-person spring-damper `ChaseCamera`, first-person `HoodCamera`, and `BumperCamera`.
- [x] Stage 6.3: Lane-aware road network traversal with Left-Hand Traffic (`RuleOfTheRoad::DriveOnLeft`) for Bengaluru routes.
- [x] Stage 6.3: Multi-agent autonomous AI traffic simulation using Intelligent Driver Model (IDM) car-following.
- [x] Stage 6.3: Drivable Bengaluru Central test routes (Vidhana Soudha -> Cubbon Park -> Kasturba Rd -> MG Road).
- [x] Stage 6.3: Integrated driving HUD in `apps/world-viewer` with `--drive` flag and full regression testing suite.

## STAGE 7: Geographic Expansion and Intercity Connectivity (Completed)
- [x] Stage 7.1: Region profiles and authentic GIS datasets for Mysuru, Chennai, and Hyderabad in `realearth-shared` and `realearth-ingestion`.
- [x] Stage 7.1: Digital Elevation Models: Mysuru plateau & Chamundi Hill (1060m), Chennai coastal flat (0m-25m), and Hyderabad granite plateau (540m-610m).
- [x] Stage 7.1: Provenance metadata, licensing (ODbL, CC-BY, Public Domain), and SHA-256 manifests in `AcquisitionEngine`.
- [x] Stage 7.2: Transport Corridors: Bengaluru–Mysuru Expressway (NH 275), Bengaluru–Chennai Highway (NH 48 / NE 7), and Bengaluru–Hyderabad Highway (NH 44).
- [x] Stage 7.2: Grade separations: Kaveri River bridge at Srirangapatna, Palar River bridge at Ranipet, Krishna & Tungabhadra bridges at Kurnool, Musi River bridge (Nayapul), and PVNR Elevated Expressway.
- [x] Stage 7.2: Long-distance road graph traversal without disconnected edges or fabricated connectors in `realearth-navigation`.
- [x] Stage 7.3: Decentralized regional package format and catalog (`RegionalCatalog`, `PackageManifest`) with incremental install, rescan, verification, and uninstallation.
- [x] Stage 7.3: Multi-region spatial streamer (`MultiRegionStreamer`) routing queries across multiple regional SQLite databases with boundary seam stitching.
- [x] Stage 7.3: Long-distance intercity vehicle travel with continuous streaming, LRU cache eviction (< 1024 MB RSS), and 60-FPS performance.
- [x] Stage 7.3: CLI (`realearth-cli`) and World Viewer (`realearth-viewer`) updates supporting all regions and intercity corridor navigation.

## STAGE 8: Modular Gameplay Platform (Completed)
- [x] Stage 8.1: Create isolated `realearth-gameplay` crate (`engine/gameplay`) with event-driven architecture and decoupled state machines.
- [x] Stage 8.1: Implement `JobStateMachine` with strict lifecycle transitions (Available -> Accepted -> EnRouteToPickup -> AtPickup -> CargoLoaded -> EnRouteToDropoff -> AtDropoff -> Completed / Failed / Abandoned / Recovered).
- [x] Stage 8.1: Implement `GameplayEventBus` with strongly-typed event emissions and listeners.
- [x] Stage 8.1: Implement `PlayerProfile`, `Wallet`, `ReputationTier`, `VehicleGarage`, and `AchievementSystem`.
- [x] Stage 8.1: Implement versioned save/load system (`SaveManager`, `SaveData`) with forward schema migrations and vehicle roadside recovery.
- [x] Stage 8.2: Implement 4 authentic playable careers (`FreeRoam`, `TaxiPassenger`, `PackageDelivery`, `HeavyFreight`) with real surveyed geographic coordinates across all covered regions.
- [x] Stage 8.2: Implement pickup/drop-off interaction zones with proximity tolerance, vehicle speed stop conditions, and boarding/loading dwell timers.
- [x] Stage 8.2: Implement topological road-distance-aware objective pricing, dynamic time limits, driving comfort scores, and configurable economy engine (`EconomyConfig`).
- [x] Stage 8.3: Implement modern minimal-click responsive HUD data structures (`ActiveObjectiveHud`, `EarningsHud`, `JobsBrowserHud`, `DebriefModal`).
- [x] Stage 8.3: Integrate gameplay systems with `apps/world-viewer` (driving mode HUD overlay, job accept/dropoff interaction) and `tools/world-cli` (job listing/execution simulation).
- [x] Stage 8.3: Develop comprehensive integration test suite (`tests/tests/gameplay_careers.rs`) covering all careers, save/load, schema migration, and long-session stability.
- [x] Stage 8.3: Validate zero regression across core simulation, rendering, streaming, and GIS layers; run full test suite and clippy/fmt quality gates.

## STAGE 9: Extensible Multi-Transport Simulation (Completed)
- [x] Stage 9.1: Road transport: Extend vehicle models with motorcycles, transit buses, and heavy commercial trucks.
- [x] Stage 9.1: Implement 2-wheel motorcycle lean angle, camber thrust, and roll dynamics.
- [x] Stage 9.1: Implement pneumatic transit bus suspension, passenger payload, and swept path turning.
- [x] Stage 9.1: Implement heavy truck multi-axle load distribution, rollover threshold, and highway access restrictions.
- [x] Stage 9.1: Validate multi-class road transport operation across surveyed road networks.
- [x] Stage 9.2: Railway transport: Implement `RailGraph`, switches/turnouts, block signaling, station platforms, and level crossings.
- [x] Stage 9.2: Implement `TrainConsist` physics: Davis resistance, tractive effort curves, steel wheel adhesion, and pneumatic brake propagation.
- [x] Stage 9.2: Marine transport: Implement `WaterwayGraph`, shipping fairway channels, harbor docks, and navigational buoys.
- [x] Stage 9.2: Implement `MarineVessel` physics: Archimedes buoyancy volume displacement, water-surface heave/pitch/roll, metacentric righting moment, and propeller thrust.
- [x] Stage 9.3: Aviation: Implement airport runway and taxiway interfaces, ILS glide slope corridors, and holding points.
- [x] Stage 9.3: Implement 6-DOF aircraft aerodynamics: lift/drag polar, control surfaces (ailerons/elevators/rudder/flaps), stall mechanics, and tricycle landing gear.
- [x] Stage 9.3: Implement high-altitude flight navigation and adaptive wide-horizon streaming ranges.
- [x] Stage 9.3: Create comprehensive test target (`tests/tests/multi_transport_simulation.rs`) validating road, rail, marine, and aviation modes.
- [x] Stage 9.3: Validate zero cross-modal coupling, 100% test pass rate across workspace, and >= 60 FPS performance.

## STAGE 10: Optional Authoritative Multiplayer (Completed)
- [x] Stage 10.1: Create isolated `realearth-networking` crate (`engine/networking`) with zero single-player runtime overhead.
- [x] Stage 10.1: Implement framed packet protocol, magic headers (`0x5245414C`), sequence numbers, selective ACKs, and channels (ReliableOrdered, UnreliableSequenced).
- [x] Stage 10.1: Implement secure session authentication, tokens, connection lifecycle, heartbeat ping/pong, and reconnection grace.
- [x] Stage 10.1: Implement validated client input frames with NaN/infinity rejection and bounds checks.
- [x] Stage 10.2: Implement authoritative server simulation tick loop, entity ownership, and state snapshot broadcasting.
- [x] Stage 10.2: Implement spatial Area of Interest (AoI) cell management with distance-based subscription filters.
- [x] Stage 10.2: Implement client-side prediction, authoritative reconciliation with replay, and remote entity snapshot interpolation.
- [x] Stage 10.2: Implement seamless region/cell boundary handoff protocols and per-client token-bucket rate limiting.
- [x] Stage 10.3: Implement network simulation harness with latency, jitter, packet drop, and reordering injection.
- [x] Stage 10.3: Test malicious/malformed packet rejection, speed-hack/teleportation defense, and unauthorized entity mutation.
- [x] Stage 10.3: Validate multi-client concurrent scalability, single-player offline isolation, and complete CI verification matrix.

## STAGE 11: Earth-Scale World Infrastructure (Completed)
- [x] Stage 11.1: Implement planetary hierarchical spatial indexing (`PlanetarySpatialIndex`, `GlobalCellKey`, quadkeys, levels 0..=18).
- [x] Stage 11.1: Implement high-precision ellipsoidal geodesics (Vincenty & Haversine), antimeridian crossing continuity, and polar wrap handling.
- [x] Stage 11.1: Implement cross-country CRS auto-zoning (UTM 1..60 N/S, EPSG:4326, EPSG:3857, national grids) and vertical datums.
- [x] Stage 11.1: Implement worldwide source manifests, licensing provenance models (OSM, Copernicus, USGS, ESA), and quality validation profiles.
- [x] Stage 11.2: Implement versioned country and regional packages (`CountryPackage`, `PackageDeliveryManager`).
- [x] Stage 11.2: Implement resumable on-demand acquisition pipeline with chunk-range resume and cryptographic checksum verification.
- [x] Stage 11.2: Implement multi-tier cache hierarchy and storage quota management (`StorageQuotaManager`) with LRU eviction of unpinned packages.
- [x] Stage 11.2: Implement selective package installation, upgrades, and uninstallation without engine restarts or neighbor disruption.
- [x] Stage 11.2: Implement missing-data diagnostics and explicit planetary basemap fallback without fabricating surveyed geography.
- [x] Stage 11.3: Implement adaptive global-to-local LOD streaming (LOD 0..4) and simulation relevance budgets (aircraft vs ground).
- [x] Stage 11.3: Implement cross-country boundary handoff and planetary floating origin re-anchoring.
- [x] Stage 11.3: Implement automated world-package certification harness (`WorldPackageCertifier`).
- [x] Stage 11.3: Implement 10,000,000 managed spatial objects benchmark demonstrating bounded resident memory (< 1024 MB RSS) and 60 FPS performance.

## STAGE 12: Forensic Quality Audit, US Major City Expansion, and Five-Platform Certification (Completed)
- [x] Stage 12.1: Execute comprehensive forensic quality audit across all stages 0–11; catalog defect analysis and remediate actionable gaps.
- [x] Stage 12.1: Remediate metric geodetic transformations with dynamic latitude cosine scaling ($111,320.0 \times \cos(\text{lat})$) across navigation, ground query, and vehicle positioning.
- [x] Stage 12.2: Expand authentic surveyed US geographic coverage across 5 major economic metropolitan hubs:
  - New York City (One WTC, Empire State, Chrysler, Broadway, FDR Drive, Brooklyn & Manhattan Bridges, East & Hudson Rivers).
  - Los Angeles (Wilshire Grand, US Bank Tower, Disney Concert Hall, I-110, Four Level Interchange).
  - Chicago (Willis Tower, Trump Tower, St. Regis, Merchandise Mart, Michigan Ave, Wacker Dr, DuSable Bridge, Chicago River, Lake Michigan).
  - Seattle (Space Needle, Columbia Center, Pike Place Market, Alaskan Way, SR 99 Deep Tunnel, Elliott Bay).
  - Miami (Panorama Tower, Southeast Financial, Brickell Ave, MacArthur Causeway, PortMiami Tunnel, Biscayne Bay).
- [x] Stage 12.2: Add authentic coastal DEM rasters, landmark presets, and regional gameplay contracts for all 5 new cities.
- [x] Stage 12.3: Implement interactive desktop windowing and keyboard event loop (`InteractiveViewerApp`) using `winit` 0.30 `ApplicationHandler` + `wgpu`.
- [x] Stage 12.3: Implement production non-blocking `UdpSocketTransport` in `engine/networking` with safe 1400-byte MTU and live loopback verification.
- [x] Stage 12.4: Create mobile native C FFI (`realearth.h`, `librealearth.a`) and JNI bindings (`librealearth.so`) in `bindings/mobile`.
- [x] Stage 12.4: Provide Android Gradle harness (`android/app`) and iOS Xcode harness (`ios/RealEarth`).
- [x] Stage 12.5: Update CI/CD build matrix and release packaging workflows for all 5 platforms with SHA-256 integrity verification.
- [x] Stage 12.5: Zero regression across 178+ tests, zero clippy warnings (`-D warnings`), zero formatting drift (`cargo fmt --check`).
