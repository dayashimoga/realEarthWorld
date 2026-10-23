# Real Earth World Platform — Code Understanding & Architecture Guide

## 1. System Codebase Structure

The workspace follows a strict layered topology under `engine/`, `apps/`, `tools/`, and `tests/`:

```
j:/rustRealEarth/
├── Cargo.toml                  # Workspace root manifest (dependency pinning)
├── docs/                       # Architectural documentation, ADRs, strategies
│   ├── adr/                    # Architecture Decision Records
│   ├── TechnicalRequirements.md
│   ├── TechnicalArchitecture.md
│   ├── ImplementationPlan.md   # Append-only execution record
│   ├── GISStrategy.md
│   ├── ValidationStrategy.md
│   ├── WorldStreamingStrategy.md
│   ├── PerformanceStrategy.md
│   ├── TestingStrategy.md
│   ├── DataSourcesAndLicenses.md
│   ├── SetupAndConfiguration.md
│   ├── UserGuide.md
│   ├── Troubleshooting.md
│   ├── Changelog.md            # Append-only changelog
│   └── Todo.md                 # Append-only task tracker
├── engine/
│   ├── shared/                 # realearth-shared: Math, coords, floating origin, errors
│   ├── geospatial/             # realearth-geospatial: Ellipsoid math, Bowring ECEF, ENU, datums
│   ├── ingestion/              # realearth-ingestion: OSM XML/PBF, DEM raster grids
│   ├── validation/             # realearth-validation: Topology, grade separation, repair
│   ├── compiler/               # realearth-compiler: 3D mesh building, .rec packaging
│   ├── streaming/              # realearth-streaming: SQLite spatial index, Tokio+Rayon LRU
│   ├── simulation/             # realearth-simulation: Headless Bevy ECS, sun/weather
│   ├── vehicles/               # realearth-vehicles: Kinematics, Rapier3D physics
│   ├── navigation/             # realearth-navigation: Petgraph lane network, A* search
│   ├── rendering/              # realearth-rendering: wgpu pipeline, shaders, camera, HUD
│   ├── network/                # realearth-network: Optional snapshot sync
│   └── gameplay/               # realearth-gameplay: Careers, contracts, economy
├── bindings/
│   └── mobile/                 # realearth-mobile: JNI (Android) & C (iOS) native bindings
├── apps/
│   └── world-viewer/           # realearth-viewer: Desktop interactive viewer app (winit+wgpu)
├── tools/
│   └── world-cli/              # realearth-cli: Headless ingestion, compilation & inspection
├── android/                    # Android Studio / Gradle project (MainActivity, JNI bridge)
├── ios/                        # Xcode / Cocoa project (UIKit, Metal view controller)
├── dist/                       # Packaged five-platform distributions and SHA256SUMS
├── benchmarks/                 # realearth-benchmarks: Criterion performance benchmarks
├── assets/                     # Authentic Bengaluru & San Francisco datasets, shaders
├── data/                       # Precompiled regional world packages (chunks.db)
├── tests/                      # 23 Integration & certification test suites
└── .github/
    ├── actions/setup-rust/     # Reusable composite setup & cache action
    └── workflows/              # Consolidated CI/CD (ci.yml, nightly.yml, release.yml)
```

---

## 2. Key Data Flow Paths

1. **Ingestion & Validation**:
   `OSM XML / PBF + DEM Raster` $\to$ `realearth-ingestion` $\to$ `realearth-validation` $\to$ `Valid Features + Quarantine Log`.
2. **Compilation**:
   `Valid Features` $\to$ `realearth-compiler` $\to$ Mesh Extrusion (`earcutr`) $\to$ Terrain Meshing $\to$ Zstd `.rec` binary chunks + `chunks.db` SQLite index.
3. **Streaming**:
   Camera position $\to$ `realearth-streaming` $\to$ SQLite bounding query $\to$ Tokio disk read $\to$ Rayon decompression $\to$ LRU cache.
4. **Rendering**:
   `LRU Cache` $\to$ `realearth-rendering` $\to$ Vertex Buffers (rebased to `FloatingOrigin`) $\to$ wgpu draw calls + HUD $\to$ Surface Presentation.
5. **Simulation, Navigation & Gameplay**:
   Road network $\to$ `realearth-navigation` $\to$ A* waypoints $\to$ `realearth-vehicles` $\to$ Rapier ground physics $\to$ `realearth-gameplay` contracts $\to$ HUD Telemetry.
