# Real Earth World Platform — Technical Requirements

## 1. System Mission & Core Philosophy
The Real Earth World Platform is a production-grade, geographically accurate, planetary-scale simulation and graphics engine built in Rust.
The system adheres to three non-negotiable architectural mandates:
1. **WORLD FIRST → SIMULATION SECOND → GAMEPLAY THIRD**: The geographic foundation is absolute. Simulation operates on validated physical reality; gameplay is an optional consumer.
2. **GEOGRAPHIC TRUTH**: Every element is grounded in real, licensed, provenance-tracked geospatial datasets (WGS84, ECEF, ENU, DEM elevations, OSM topology). Never fabricate geographic facts. Inferred attributes (such as heights derived from level counts) must be explicitly flagged with confidence metrics.
3. **HEADLESS & INDEPENDENT SUBSYSTEMS**: The world compiler, GIS validation, and simulation run headlessly without GPU or windowing requirements. Rendering, physics, navigation, gameplay, and networking remain independently testable and replaceable.

---

## 2. Functional Requirements (FR)

### FR-01: Geodetic & Coordinate Transformations
- High-precision conversion between:
  - WGS84 Geodetic coordinates $(\text{latitude}, \text{longitude}, \text{altitude})$.
  - Earth-Centered Earth-Fixed (ECEF) Cartesian coordinates $(X, Y, Z)$ in double precision (`f64`).
  - East-North-Up (ENU) Local Tangent Plane coordinates relative to local chunk/region origins.
- Precision threshold: Geodetic round-trip conversion error must not exceed $10^{-6}$ meters ($1\ \mu\text{m}$).
- Floating-origin transformation system to eliminate single-precision floating-point precision jitter at planetary scale when submitting vertex positions to GPU shaders.

### FR-02: Geospatial Ingestion
- Ingest OpenStreetMap (OSM) vector data (nodes, ways, relations) covering roads, lanes, bridges, tunnels, layer tags, and building footprints.
- Ingest Digital Elevation Models (DEM) from Copernicus 30m / NASA SRTM sources.
- Support bilinear and bicubic elevation interpolation for any arbitrary WGS84 coordinate.
- Ingest land cover classification data (water bodies, parks, built-up areas).

### FR-03: GIS Normalization & Topological Validation
- Enforce topological graph consistency: roads meeting at junctions must share topological junction nodes.
- Grade separation validation: Overpasses, bridges, and flyovers with distinct vertical layers (`layer != 0`, `bridge = yes`) must not create spurious at-grade intersections with ground-level highways.
- Polygon validity: Building footprints and water body boundaries must be verified for self-intersection, orientation (counter-clockwise exterior rings), and planarity.
- Quarantine and repair: Unrecoverable geometric errors must be isolated into a quarantine log with detailed failure reasons rather than silently dropped or erroneously distorted.

### FR-04: World Compiler & Binary Chunk Pipeline
- Divide the geographic domain into standardized, bounded spatial chunks (e.g. 500m $\times$ 500m local tiles).
- Generate 3D triangle meshes for:
  - Extruded building geometries with accurate roof and facade vertices, surface normals, and texture coordinates.
  - Road ribbons with realistic cross-sections, lane dividers, curbs, and elevation conforming to the underlying terrain or bridge profile.
  - Terrain heightfields with seamless borders between adjacent chunks.
- Serialize compiled chunks into versioned binary packages (`.rec` — Real Earth Chunk) compressed using `zstd`.
- Index chunks in an embedded SQLite database (`chunks.db`) with spatial bounds, LOD levels, feature counts, and byte offsets.

### FR-05: Real-Time World Streaming
- Asynchronous streaming runtime powered by `tokio` (I/O) and `rayon` (decompression & mesh building).
- Priority-driven chunk loading queue based on Euclidean camera distance and viewing frustum.
- Strict memory budget enforcement: Configurable upper limit (default 1024 MB RAM for Bengaluru MVP) with LRU eviction for out-of-range chunks.
- Seamless level-of-detail (LOD) transitions without geometry cracking.

### FR-06: Headless Bevy ECS Simulation
- Simulation state decoupled entirely from graphics and windowing.
- Fixed time-step simulation loop (default 60 Hz).
- Deterministic time-of-day progression with solar azimuth and elevation calculation based on geographic latitude and UTC timestamp.
- Dynamic weather state (clear, rain, fog) feeding atmospheric parameters.

### FR-07: Vehicle Kinematics & Physics
- Vehicle entity representation using Bevy ECS.
- Support kinematic Ackermann steering models and dynamic multi-body physics integration via `rapier3d`.
- Road surface adhesion, grade calculation, and collision detection against terrain and building boundaries.

### FR-08: Road Navigation & Routing
- Directed lane-level graph representation using `petgraph`.
- A* shortest and fastest path search factoring in road classifications, speed limits, turn restrictions, and one-way streets.
- Waypoint sequence generation for autonomous traffic simulation.

### FR-09: Rendering Pipeline (wgpu)
- Native WebGPU/wgpu rendering architecture supporting Vulkan, DirectX 12, and Metal backends.
- Camera system operating relative to the active floating origin.
- Forward+/PBR shader pipeline rendering terrain, roads, and buildings with directional sun lighting, ambient occlusion approximation, and distance fog.
- Real-time diagnostic overlays: wireframe toggles, chunk bounding box visualization, FPS counter, frame time graphs, and draw call counters.

---

## 3. Non-Functional Requirements (NFR)

### NFR-01: Performance & Resource Limits (Bengaluru MVP)
- **First Usable Frame**: $\le 2.0\text{ seconds}$ from cold start with locally cached compiled package.
- **Process Memory (RSS)**: $\le 1024\text{ MB}$ under standard Bengaluru Central reference scenario.
- **Target Frame Rate**: Sustained 60 FPS ($16.67\text{ ms}$ budget per frame) on reference mid-tier desktop hardware.
- **P95 Frame Time**: $\le 18.0\text{ ms}$; **P99 Frame Time**: $\le 22.0\text{ ms}$.
- **Zero Streaming Stutter**: Chunk decompression and GPU upload must be staggered or backgrounded so as not to stall the main render loop.

### NFR-02: Scaling Targets
- Level 1: 100,000 managed geospatial entities at 60 FPS.
- Level 2: 1,000,000 managed entities with hierarchical spatial partitioning and LOD culling.
- Level 3: 10,000,000 managed entities enabled by background streaming and out-of-core database indexing.

### NFR-03: Quality & Code Integrity
- Test coverage for core testable crates: $\ge 90\%$.
- Zero Clippy warnings under strict lints (`-D warnings`).
- Zero formatting discrepancies (`cargo fmt --check`).
- Fully automated test execution across Windows, Linux, and macOS runners in CI.
- No mocked or fake success reports: all tests must execute genuine code paths on real or rigorous synthetic datasets.
