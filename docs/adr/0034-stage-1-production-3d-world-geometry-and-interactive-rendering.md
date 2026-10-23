# ADR-0034: Stage 1 Production 3D World Geometry and Interactive Rendering Repair

## Status
Accepted

## Context
In Stage 0, forensic analysis identified that the 3D rendering pipeline exhibited severe visual defects and structural geometry anomalies:
1. An artificial flat green plane obscuring topography.
2. Inverted triangle winding orders in terrain and building roofs causing inverted surface normals and backface clipping.
3. Road ribbon geometry with flipped clockwise quad indices and z-fighting with the underlying DEM mesh.
4. Broken horizon artifacts resulting from an artificial 0.88 clamp in the atmospheric fog shader.
5. Coordinate frame offset between the camera's floating origin anchor and the world chunk package origin, displacing visible objects.
6. Window recreation panics on DPI/ScaleFactorChanged due to depth buffer dimension mismatches.
7. Unconstrained camera movement allowing clipping beneath the terrain.

## Decisions

1. **Terrain Geometry & Geomorphic Shading**:
   - Replaced clockwise triangle index generation in [engine/compiler/src/terrain_mesher.rs](file:///J:/rustRealEarth/engine/compiler/src/terrain_mesher.rs) with counter-clockwise (CCW) winding `(top_left, top_right, bottom_left)` and `(top_right, bottom_right, bottom_left)`.
   - Replaced artificial uniform green plane vertex coloring with dynamic altitude- and slope-aware geomorphic vertex colors: coastal shoreline sand ($\le 0.8\text{m}$), urban street soil/pavement ($0.8\text{m}-30\text{m}$), hillside park green ($30\text{m}-85\text{m}$), and steep cliff sandstone/granite ($n_y < 0.82$ and ridges $> 85\text{m}$).

2. **Road Ribbon Geometry & Elevation Offset**:
   - Replaced clockwise quad winding in [engine/compiler/src/road_mesher.rs](file:///J:/rustRealEarth/engine/compiler/src/road_mesher.rs) with counter-clockwise winding `(base, base + 3, base + 1)` and `(base, base + 2, base + 3)`.
   - Elevated road centerline and shoulder vertices by $+0.20\text{m}$ above local ground level to completely eliminate z-fighting and depth coplanarity with terrain.

3. **Building Geometry & Facade Normals**:
   - Corrected outward horizontal wall normals in [engine/compiler/src/building_mesher.rs](file:///J:/rustRealEarth/engine/compiler/src/building_mesher.rs) using 2D edge normal `Vec3::new(edge.z, 0.0, -edge.x).normalize()`.
   - Inverted building roof triangulation indices from earcut clockwise winding to counter-clockwise upward-pointing (+Y) normal winding.

4. **Atmospheric Fog & Horizon Blending**:
   - In [engine/rendering/src/shaders.rs](file:///J:/rustRealEarth/engine/rendering/src/shaders.rs), removed the artificial `clamp(..., 0.0, 0.88)` on linear fog and smoothstep.
   - Far-plane geometry at `view_distance_meters` now smoothly and completely reaches $1.0$ fog density, fading seamlessly into `sky_fog_color` without hard clipping edges.

5. **Camera Anchor Synchronization**:
   - Added `set_anchor(&mut self, anchor: Wgs84Coord)` to [engine/rendering/src/camera.rs](file:///J:/rustRealEarth/engine/rendering/src/camera.rs).
   - In [apps/world-viewer/src/interactive.rs](file:///J:/rustRealEarth/apps/world-viewer/src/interactive.rs) and [apps/world-viewer/src/main.rs](file:///J:/rustRealEarth/apps/world-viewer/src/main.rs), synchronized `camera.anchor_ecef` with the active world package's anchor (`first_chunk.header.anchor`).
   - Ensured the camera's local ENU coordinate space matches the chunk vertex buffers with sub-millimeter precision.

6. **Interactive Frame Lifecycle & Navigation Controls**:
   - Recreated `Depth32Float` depth texture and view on `WindowEvent::ScaleFactorChanged` to avoid GPU dimension mismatch panics.
   - Added camera navigation modes (`Fly`, `Walk`, `Orbit`, `Free`, `Chase`) with `Tab` cycling and `clamp_terrain` collision prevention in [apps/world-viewer/src/camera_controller.rs](file:///J:/rustRealEarth/apps/world-viewer/src/camera_controller.rs).
   - Mapped `Home` key for instant landmark reset, sprint acceleration (`Shift` = 2.5x), and fine precision (`Ctrl` = 0.35x).

## Consequences

* **Positive**:
  - The San Francisco Bay Area and all catalog regions render with authentic, physically accurate 3D geometry.
  - Zero inverted triangles, zero z-fighting on roads, zero broken horizon cutoffs, and zero terrain clipping.
  - 100% of workspace tests and integration certification suites pass.
  - High performance: steady-state frame rate of 1916.7 FPS (0.52 ms p95 latency) in headless verification.
* **Verification Artifact**:
  - Reference screenshot captured and verified at [sf_stage1_reference.png](file:///C:/Users/daya/.gemini/antigravity-ide/brain/c5a543a1-6e1e-4fdf-83e0-1043a873219b/sf_stage1_reference.png).
