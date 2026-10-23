# ADR 0026: Stage 1 — Forensic Repair: Live Interactive Surface Rendering Architecture

## Status
Accepted / Implemented

## Date
2026-10-22

## Context
A critical forensic defect was discovered in the desktop runtime of the Real Earth World application:
1. `apps/world-viewer/src/main.rs` unconditionally executed a 60-frame offscreen virtual benchmark before opening a desktop window.
2. The interactive loop in `apps/world-viewer/src/interactive.rs` created an empty `winit` window with no `wgpu::Surface`, no render pipeline, and no swapchain presentation. Redraw events requested further redraws in an unthrottled spin loop, resulting in a blank white window reporting deceptive ~579 FPS.
3. The real planetary world geometry (terrain, roads, buildings, water, vegetation), dynamic vehicle simulation, and HUD were never wired to the live display swapchain.

## Decision
1. **Decoupled CLI Architecture**:
   - Standard execution (`cargo run -p realearth-viewer` or launching `realearth-viewer.exe`) defaults directly to the interactive native window with live 3D surface presentation.
   - Headless benchmarking is strictly quarantined under the `--benchmark` and `--headless` CLI flags, ensuring offscreen benchmark metrics are never conflated with interactive window performance.

2. **Persistent `winit + wgpu` Surface Pipeline**:
   - The interactive application implements `winit::application::ApplicationHandler` with full lifecycle management across `resumed`, `window_event`, `about_to_wait`, and `suspended`.
   - On `resumed`: Dynamically acquires window handle, creates `wgpu::Surface`, selects high-performance adapter, creates logical device/queue, configures `SurfaceConfiguration` with `PresentMode::AutoVsync` (or `Fifo`), initializes reverse-Z depth texture (`TextureFormat::Depth32Float`), builds `RealEarthRenderer` from device, loads active streaming chunks, spawns player vehicle, and initializes 2D HUD overlay.
   - On `RedrawRequested`:
     - Checks for zero/minimized window dimensions (gracefully skipping render passes).
     - Fixed-timestep simulation tick (vehicle dynamics, IDM autonomous traffic, camera tracking).
     - Updates nearby streaming chunks and synchronizes ECS components with GPU buffer caches.
     - Acquires `SurfaceTexture` from swapchain, handling `SurfaceError::Timeout`, `SurfaceError::Outdated`, `SurfaceError::Lost`, and `SurfaceError::OutOfMemory` with automatic re-configuration.
     - Renders 3D planetary geometry (terrain, roads, buildings, water, vegetation) and vehicle meshes with depth testing.
     - Renders 2D screen-space HUD overlay with telemetry, speed, gear, coordinates, heading, and graphics presets.
     - Submits command buffers to queue and calls `frame.present()`.
     - Calls `window.request_redraw()` synchronized with display refresh to prevent busy-looping.

3. **2D Screen-Space HUD & 3D Procedural Vehicle Rendering**:
   - Created `engine/rendering/src/hud.rs` implementing `HudMesh`, `HudVertex`, and built-in 5x7 ASCII bitmap text font and telemetry bars, rendered through `HUD_SHADER_WGSL`.
   - Created `engine/rendering/src/vehicle_mesh.rs` implementing parameterized 3D procedural vehicle geometry (chassis, cabin, wheels, headlights, taillights) for player vehicles and autonomous traffic fleets.

4. **Independent Validation Suite**:
   - Implemented `tests/tests/interactive_rendering_certification.rs` containing 8 tests asserting nonempty San Francisco geometry, visible chunk culling, wgpu surface depth pipelines, camera-responsive frame changes, landmark detectability via spatial/depth assertions, HUD vertex generation, vehicle mesh rendering, and CLI mode separation invariance.

## Consequences
- **Positive**:
  - The Windows executable visibly renders navigable San Francisco streets, terrain, and buildings immediately upon startup.
  - Zero busy-looping: presentation throttled by display VSync.
  - Comprehensive crash resilience against window resizing, minimization, DPI scaling, and surface loss.
  - Strict separation between interactive display and headless profiling.
- **Negative / Operational Notes**:
  - Headless CI environments without physical display adapters must continue to use `--headless` or `--benchmark` mode, or mock adapters.
