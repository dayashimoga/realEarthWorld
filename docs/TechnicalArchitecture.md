# Real Earth World Platform — Technical Architecture

## 1. Architectural Overview & Component Topology

The platform enforces a unidirectional, pipeline-driven architecture. Each subsystem is encapsulated in its own Rust crate within the Cargo workspace, with strictly defined interfaces.

```mermaid
graph TD
    subgraph Data Layer
        OSM[OpenStreetMap XML/PBF]
        DEM[Copernicus / NASA DEM Geotiff/HGT]
        LC[Land Cover Datasets]
    end

    subgraph "engine/ingestion (realearth-ingestion)"
        IngestOSM[OSM Parser & Feature Extractor]
        IngestDEM[DEM Rasterizer & Interpolator]
    end

    subgraph "engine/geospatial (realearth-geospatial)"
        GeoDatum[WGS84 Ellipsoid Math]
        ECEF[ECEF / ENU Coordinate Transforms]
        FloatOrigin[Floating Origin System]
    end

    subgraph "engine/validation (realearth-validation)"
        TopoVal[Topological & Junction Check]
        GradeVal[Grade Separation & Bridge Layer Validation]
        PolyRepair[Polygon Repair & Confidence Tagging]
        Quarantine[Defect Quarantine Log]
    end

    subgraph "engine/compiler (realearth-compiler)"
        MeshGen[Building & Road Mesh Generator]
        TerrainGen[Terrain Heightfield Triangulation]
        ChunkPack[Versioned Binary Chunk Packager]
    end

    subgraph Storage & Indexing
        SQLite[SQLite Spatial Index (chunks.db)]
        ChunksBin[Compressed Binary Chunks (.rec)]
    end

    subgraph "engine/streaming (realearth-streaming)"
        TokioIO[Tokio Async IO Worker]
        RayonDecomp[Rayon Parallel Decompressor]
        LRUCache[Memory-Budgeted LRU Chunk Cache]
    end

    subgraph "engine/simulation (realearth-simulation)"
        BevyECS[Bevy ECS World & Systems]
        TimeWeather[Solar Position & Weather Simulation]
    end

    subgraph "engine/physics & vehicles (realearth-vehicles)"
        RapierPhys[Rapier3D Physics Engine]
        VehicleDynamics[Ackermann & Kinematic Models]
    end

    subgraph "engine/navigation (realearth-navigation)"
        RoadGraph[Petgraph Lane-Level Routing]
        AStar[A* Pathfinding & Turn Restrictions]
    end

    subgraph "engine/rendering (realearth-rendering)"
        WgpuPipe[WGPU Render Pipeline]
        CameraSystem[Floating-Origin Camera]
        ShaderPass[Terrain, Road, & Building Shaders]
    end

    subgraph "apps/world-viewer (realearth-viewer)"
        AppMain[Winit Event Loop & UI HUD]
    end

    OSM --> IngestOSM
    DEM --> IngestDEM
    LC --> IngestOSM

    IngestOSM --> GeoDatum
    IngestDEM --> GeoDatum
    GeoDatum --> ECEF

    IngestOSM --> TopoVal
    IngestDEM --> TopoVal
    TopoVal --> GradeVal
    GradeVal --> PolyRepair
    PolyRepair --> Quarantine
    PolyRepair --> MeshGen

    MeshGen --> ChunkPack
    TerrainGen --> ChunkPack
    ECEF --> ChunkPack

    ChunkPack --> SQLite
    ChunkPack --> ChunksBin

    SQLite --> TokioIO
    ChunksBin --> TokioIO
    TokioIO --> RayonDecomp
    RayonDecomp --> LRUCache

    LRUCache --> BevyECS
    LRUCache --> WgpuPipe
    LRUCache --> RapierPhys
    LRUCache --> RoadGraph

    BevyECS --> TimeWeather
    BevyECS --> VehicleDynamics
    RapierPhys --> VehicleDynamics
    RoadGraph --> AStar
    AStar --> VehicleDynamics

    VehicleDynamics --> BevyECS
    BevyECS --> CameraSystem
    FloatOrigin --> CameraSystem
    CameraSystem --> WgpuPipe
    WgpuPipe --> ShaderPass
    ShaderPass --> AppMain
```

---

## 2. Crate Responsibilities & Interfaces

| Crate | Directory | Primary Role | Key Dependencies |
|---|---|---|---|
| `realearth-shared` | `engine/shared` | Core math, coordinates (`Wgs84Coord`, `EcefCoord`, `EnuCoord`), floating origin, bounds, errors | `glam`, `serde`, `thiserror` |
| `realearth-geospatial` | `engine/geospatial` | Ellipsoid math, ECEF Bowring algorithm, ENU rotation matrices, tile math, geohashes | `realearth-shared`, `glam` |
| `realearth-ingestion` | `engine/ingestion` | Ingests OSM XML/PBF and DEM grids; extracts ways, nodes, heights | `realearth-geospatial`, `realearth-shared` |
| `realearth-validation` | `engine/validation` | Enforces planar graph rules, grade separation, polygon topology, confidence tagging | `realearth-geospatial`, `realearth-shared`, `geo` |
| `realearth-compiler` | `engine/compiler` | Generates 3D building extrusions, road meshes, terrain meshes, compresses chunks with zstd | `realearth-validation`, `earcutr`, `zstd`, `bincode` |
| `realearth-streaming` | `engine/streaming` | SQLite spatial index, Tokio async loader, Rayon decompression, LRU memory manager | `rusqlite`, `tokio`, `rayon`, `realearth-compiler` |
| `realearth-simulation` | `engine/simulation` | Headless Bevy ECS simulation, deterministic time-of-day, solar azimuth, weather cycle | `bevy_ecs`, `realearth-shared` |
| `realearth-vehicles` | `engine/vehicles` | Vehicle kinematics, Ackermann steering, Rapier 3D colliders & rigidbodies | `rapier3d`, `bevy_ecs`, `realearth-shared` |
| `realearth-navigation` | `engine/navigation` | Lane-level directed graph using `petgraph`, A* pathfinding, turn restrictions | `petgraph`, `realearth-shared` |
| `realearth-rendering` | `engine/rendering` | Headless & windowed `wgpu` renderer, floating-origin camera, shaders, debug passes | `wgpu`, `bytemuck`, `realearth-shared` |
| `realearth-network` | `engine/network` | Optional QUIC networking protocol, snapshot serialization, delta compression | `serde`, `bincode` |
| `realearth-viewer` | `apps/world-viewer` | Interactive viewer application with winit window, camera controls, HUD overlay | `winit`, `realearth-rendering`, `realearth-streaming` |
| `realearth-cli` | `tools/world-cli` | CLI for ingestion, compilation, validation reports, and inspection | `clap`, `realearth-compiler` |

---

## 3. Coordinate System & Floating Origin Strategy

Planetary scale simulations suffer from IEEE 754 32-bit floating-point precision loss when rendering objects far from the origin ($0, 0, 0$). At the Earth's radius ($\sim 6,378,137\text{ m}$), a 32-bit float provides a spatial precision of approximately $0.5\text{ to }1.0\text{ meter}$, causing visual vertex jitter and physics instabilities.

### Solution:
1. **World Storage & Physics Representation**: Double-precision (`f64`, `glam::DVec3`) in ECEF coordinates.
2. **Local Chunk Representation**: Local East-North-Up (ENU) coordinates relative to the chunk anchor.
3. **Floating Origin for Rendering**:
   - The camera maintains an absolute ECEF position $\mathbf{P}_{\text{cam}} \in \mathbb{R}^3$ in `f64`.
   - When submitting transforms to the GPU, vertex positions are translated relative to the active floating origin anchor $\mathbf{A}_{\text{origin}}$:
     $$\mathbf{P}_{\text{rel}} = \text{cast\_f32}(\mathbf{P}_{\text{world}} - \mathbf{A}_{\text{origin}})$$
   - When camera distance from $\mathbf{A}_{\text{origin}}$ exceeds $5,000\text{ meters}$, the floating origin resets to the current camera position, shifting the scene origin transparently.

---

## 4. Binary Chunk Package Specification (`.rec`)

A Real Earth Chunk (`.rec`) contains:
- **Header**: Magic bytes `RECH`, schema version (e.g. `1`), chunk coordinate $(X_{\text{chunk}}, Y_{\text{chunk}})$, bounding box (WGS84 min/max), elevation range (min/max).
- **Terrain Block**: Triangulated heightfield grid vertices (position, normal, UV) and index buffer.
- **Road Block**: Road ribbon segments, width, lane markings, bridge elevation profiles.
- **Building Block**: Extruded polygon meshes with wall vertices, roof triangles, height confidence tag (`Surveyed`, `InferredLevels`, `DefaultCategory`), and OSM building ID.
- **Compression**: Entire chunk body compressed with `zstd` level 3 for high-speed real-time decompression.

---

## 5. Cross-Platform Runtime & Packaging Architecture

The platform targets five distinct operating systems using native abstractions:
- **Windows (x86_64)**: Native executable with DirectX 12 / Vulkan / wgpu backend. Resolves assets relative to `current_exe().parent()` enabling zero-repository standalone portable execution. Bundled pilot world allows offline startup.
- **Linux (x86_64)**: Compliant `RealEarth.AppDir` specification with `AppRun` configuring Vulkan ICDs and Wayland/X11 display variables. Distributed as portable `.tar.gz` and buildable to SquashFS AppImage.
- **macOS (Universal)**: Universal binary (`aarch64` Apple Silicon + `x86_64` Intel) packaged in standard `RealEarth.app` bundle with `Info.plist` (Metal API, Retina high-DPI scaling) and `PkgInfo: APPL????`.
- **Android (ARM64 / x86_64)**: Native JNI shared library (`librealearth.so`) in `bindings/mobile/` interfacing with Android `SurfaceView` via `SurfaceHolder.Callback`. Features virtual driving controls, thermal status listening, and low-memory cache pruning.
- **iOS (ARM64 / Simulator)**: Cocoa/UIKit project in `ios/RealEarth/` with `CAMetalLayer` rendering bridge (`RealEarthBridge.m`) and iOS Simulator package.

---

## 6. Empirical Performance, Pacing & Anti-Fake Telemetry

- **OS-Native Memory Accounting**: Queries native kernel working sets (`K32GetProcessMemoryInfo` on Windows, `/proc/self/statm` on Linux) tracking true RSS and peak RSS.
- **Statistical Frame Pacing with Warmup Separation**: Monotonic clock timing isolating the first 10 warmup frames (`warmup_mean_ms`) to prevent shader compilation skew. Computes Mean, Std Dev, Min, Max, p50, p90, p95, p99, 1% Low FPS, 0.1% Low FPS, and Presented FPS.
- **Anti-Fake Mathematical Gate**: Dynamic evaluation in `PerformanceCertificationReport::evaluate(...)` rejecting empty frames ($0$ triangles), latency overruns ($> 16.67\text{ ms}$), and memory spikes ($> 1024\text{ MB}$).

---

## 7. Decoupled Standalone Geographic Regional Distribution

Geographic world datasets are decoupled from executable binaries and packaged as standalone regional archives:
- Packaged via `scripts/package_regions.ps1` and `scripts/package_regions.sh` into `dist/regions/realearth-region-<id>.zip` and `.tar.gz`.
- Contains SQLite `chunks.db`, surveyed `.rec` binary tiles, installation guide, and individual `.sha256` digests registered in `dist/SHA256SUMS`.

