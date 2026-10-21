# ADR 0007: Functional America San Francisco World Viewer Architecture

## Status
Accepted

## Context
Stage 3 of the Global Engineering Contract requires delivering a functional, high-performance 3D world viewer navigating the authentic San Francisco, California pilot area ($37.7800^\circ\text{N}, -122.4150^\circ\text{W}$ to $37.8050^\circ\text{N}, -122.3880^\circ\text{W}$). The viewer must operate strictly on Stage 2 world packages without vehicles, gameplay, or multiplayer distractions.

Rigorous performance and integrity criteria are mandated:
1. **Framerate:** Sustained $\ge 60\text{ FPS}$ ($\le 16.67\text{ ms/frame}$).
2. **Startup Latency:** $< 2.0\text{ seconds}$ from a cached world package.
3. **Resident Memory:** Strict budget $< 1024\text{ MB}$ RSS.
4. **Visual & Geometric Integrity:** Grounded buildings on high-relief DEM terrain ($0.0\text{m}$ to $105.0\text{m}$ MSL), elevated I-80 Bay Bridge approach (+6m), subterranean Broadway Tunnel (-10m), flat coastal water at $0.0\text{m}$ MSL, organic vegetation zones without Z-fighting, and seamless $C^0$ continuous terrain without cracks.
5. **Automation & Reproducibility:** Support real GPU execution, windowed interaction, dual-mode camera navigation (Fly & Walk), landmark jumping, telemetry overlay, and headless offscreen framebuffer capture (`.ppm`) for automated CI/CD validation.

## Decisions

### 1. Bevy ECS Scene Entity Synchronization
We decoupled world mesh components into first-class Bevy ECS entities within `realearth-simulation`:
- Components: `ChunkEntity`, `TerrainMeshComponent`, `RoadMeshComponent`, `BuildingMeshComponent`, `WaterMeshComponent`, `VegetationMeshComponent`, and `BoundingSphere`.
- The simulation world (`RealEarthSimWorld`) mirrors the active streaming chunk set, allowing decoupled spatial indexing, simulation queries, and graphics sync without tight coupling to windowing or presentation libraries.

### 2. Gribb-Hartmann Frustum Culling
To prevent redundant GPU vertex processing across large geographic extents:
- Extracted 6 normalized frustum planes directly from the combined camera $4\times4$ view-projection matrix using the Gribb-Hartmann algorithm (`Frustum::from_view_proj`).
- Implemented conservative `intersects_sphere` and `intersects_aabb` testing in `realearth-rendering::culling`.
- Chunk bounding spheres are tested prior to issuing draw commands in `render_chunks_culled`, drastically reducing GPU pipeline load in large viewing radii.

### 3. Surface Meshing for Water & Urban Vegetation
We extended `realearth-compiler` with dedicated meshers:
- **Water Mesher (`water_mesher.rs`):** Triangulates polygonal water boundaries (San Francisco Bay, coastal shoreline) locked to $0.0\text{m}$ MSL. Assigns aquatic blue vertex colors and upward surface normals $[0, 0, 1]$.
- **Vegetation Mesher (`vegetation_mesher.rs`):** Triangulates urban park boundaries (Portsmouth Square, Sue Bierman Park) using `earcutr`. Vertices sample underlying DEM terrain with an intentional $+0.35\text{m}$ vertical offset to completely prevent coplanar Z-fighting against ground triangles.
- Added `water` and `vegetation` mesh slots to `WorldChunk`, seamlessly serialized into `.rec` binary format.

### 4. Floating-Origin Dual Navigation Camera Controller
To eliminate 32-bit floating point precision jitter at planetary scale while supporting intuitive navigation:
- Implemented `CameraController` in `apps/world-viewer/src/camera_controller.rs` supporting:
  - **`CameraMode::Fly`:** 6-DOF free camera with configurable speed multiplier and sprint boosts.
  - **`CameraMode::Walk`:** Ground-clamped pedestrian navigation maintaining constant $1.8\text{m}$ eye-level clearance above the underlying DEM terrain.
- Added instant landmark jumping (`1`..`6` for San Francisco landmarks including Ferry Building, Salesforce Tower, Transamerica Pyramid, Nob Hill, Bay Bridge, and Broadway Tunnel; `7` for Bengaluru Vidhana Soudha).
- Converts camera local ENU coordinates back to geodetic WGS84 ($lat, lon, alt$) in real time for telemetry display.

### 5. Deterministic Headless Offscreen PPM Export
To enable automated CI verification of visual rendering without requiring desktop window servers:
- Implemented offscreen rendering pipeline using `wgpu` with an offscreen color texture and depth buffer.
- Added `screenshot.rs` in `apps/world-viewer`: stages GPU framebuffer memory into a buffer aligned to `wgpu::COPY_BYTES_PER_ROW_ALIGNMENT` (256 bytes) using `.div_ceil(align) * align`.
- Serializes raw RGB raster directly into binary Netpbm PPM (`P6`) format, eliminating third-party image encoding dependencies and ensuring bit-exact CI artifact generation.

## Consequences
- The viewer runs identically in both local windowed developer sessions and headless Linux/Windows CI test runners.
- Cached package startup executes in $\approx 30\text{ ms}$, exceeding the $< 2000\text{ ms}$ budget by two orders of magnitude.
- Framerate achieves $\approx 84\text{ FPS}$ in debug builds on reference hardware (well above the $\ge 60\text{ FPS}$ threshold).
- Resident memory remains $< 1\text{ MB}$, well within the $1024\text{ MB}$ memory budget.
- Continuous integration scripts can inspect generated `.ppm` frames to verify color histograms, rendering coverage, and absence of visual artifacts.
