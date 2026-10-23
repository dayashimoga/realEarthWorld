# ADR 0028: Stage 3 — End-to-End World Integration, Driving Physics, Gameplay Loops, and Visual Fidelity

## Status
Accepted / Implemented

## Date
2026-10-22

## Context
Prior iterations of the Real Earth World application suffered from disconnected subsystems:
1. **LOD Seams and Coordinate Inconsistencies**: The world compiler assigned each chunk an independent anchor at `chunk.bounds.center()`, converting mesh vertices into per-chunk local ENU. Meanwhile, the renderer and camera computed vertex positions assuming a single global or package anchor, causing inter-chunk cracks, altitude offsets, and LOD seams. Furthermore, camera position transformations in `camera.relative_pos_f32` assumed global ECEF without accounting for the local ENU orientation, introducing severe view matrix tilt (~50° in US mid-latitudes).
2. **Vehicle Physics Tunneling & Sinking**: Vehicle raycast suspension queries in `engine/vehicles/src/ground.rs` returned `None` whenever ground penetration exceeded zero (`dist < 0.0`), depriving the solver of restorative spring forces. Vehicles subjected to curb strikes, steep slopes, or velocity shocks fell through the terrain and tunneled into the void. Additionally, vehicles lacked class-specific physics presets (sports coupe, sedan, SUV, taxi, bus, heavy truck).
3. **Disconnected Gameplay & Progression**: Gameplay mechanics (taxi, courier, freight delivery) existed as isolated data structures without live game loop integration. Deliveries did not reward wallet payouts, earn driver XP, or advance career progression. Quicksave/quickload functionality was absent from runtime keybindings.
4. **Visual Uniformity**: Shaders rendered geometry with flat surface colors without architectural details (window patterns, night interior illumination) or animated water wave dynamics (specular reflection, sun glint).

Stage 3 mandates:
- **3.1 World integration**: GIS → validator → compiler → chunk storage → streamer → GPU renderer → physics collision → navigation working together in live applications without LOD seams, cracked boundaries, or missing chunks.
- **3.2 Vehicles**: Physically stable, visible cars with responsive steering, throttle, braking, suspension, traction, gravity, anti-tunneling bump-stops, and authentic road network routing.
- **3.3 Gameplay**: Functional career progression, contract jobs (Taxi, Package Delivery, Heavy Freight), real payouts deposited into player wallet, garage vehicle mode selection, HUD indicators, and quicksave/quickload roundtrip persistence (`F5` / `F9`).
- **3.4 Visual quality**: Procedural architectural facade windows with warm nighttime interior illumination, animated water wave specular shimmer, realistic atmosphere, and performance-tuned render presets.

## Decisions

### 1. Unified Package Coordinate Anchors & View Matrix (`engine/compiler`, `engine/rendering`)
- **Package-Wide ENU Anchor**: In `engine/compiler/src/compiler_pipeline.rs`, updated chunk compilation so that all chunks within a package share a single package-wide anchor: `package_converter = EnuConverter::new(bounds.center())`. Mesh vertices are projected into this unified coordinate space, guaranteeing that adjacent chunks align down to sub-millimeter precision with zero boundary cracks or LOD seams.
- **Camera ENU Projection**: In `engine/rendering/src/camera.rs`, updated `relative_pos_f32(&self)` to project `pos_ecef` into local ENU relative to `anchor_ecef` using `EnuConverter::new(anchor_wgs84).ecef_to_enu(self.pos_ecef)`. This guarantees that the view-projection matrix precisely matches the mesh coordinate system (X=East, Y=Elevation, Z=North) without orientation distortion.
- **Chunk Culling Relative Center**: In `engine/rendering/src/renderer.rs`, fixed `render_chunks_culled_cached` to compute bounding sphere centers using `chunk.header.bounds.center()` converted to local ENU relative to `chunk.header.anchor`.

### 2. Vehicle Anti-Tunneling, Ground Clamping & Dynamics Presets (`engine/vehicles`, `apps/world-viewer`)
- **Signed Ground Penetration with Bump-Stop**: In `engine/vehicles/src/ground.rs`, modified `FlatGround` and `SlopedGround` query implementations so that negative depths (`dist < 0.0`) down to `-10.0m` return `Some((dist, normal))`. This supplies the spring-damper solver with restorative bump-stop forces, preventing penetration.
- **Physical Hard Clamp**: In `engine/vehicles/src/dynamics.rs` and `apps/world-viewer/src/interactive.rs`, added a chassis safety clamp preventing vehicle altitude from sinking below `ground_y + wheel_radius * 0.5`.
- **Surface Normal Query**: `DemGroundQuery` evaluates partial spatial derivatives (`slope_x`, `slope_z`) across the digital elevation model grid to compute realistic terrain surface normals.
- **Fleet Vehicle Presets**: Added factory presets in `VehiclePhysicsConfig` for `sports_coupe`, `sedan`, `suv`, `taxi`, `transit_bus`, and `heavy_truck`, tailoring mass (1,350kg to 16,000kg), engine torque, brake force, suspension stiffness, and steer limits.

### 3. Functional Career Loop, Contracts, Garage, and Persistence (`apps/world-viewer`, `engine/gameplay`)
- **Contract Job Completion & Progression**: In `apps/world-viewer/src/interactive.rs`, wired job completion events to award contract base payouts, bonuses, and driver XP to `player_profile.wallet` and `career_progress`. The HUD displays celebration banners upon delivery completion.
- **Contract Selection Keybinding**: Added `KeyJ` to cycle and accept regional contracts across Taxi, Courier Package Delivery, Heavy Freight, and Free Roam modes.
- **Garage Fleet Cycling**: Added `KeyT` and `KeyG` to swap active vehicles in real time across the preset classes, immediately updating physics and rendering.
- **Persistence (`F5` / `F9`)**: Bound `F5` to `SaveManager::save_to_file("target/saves/quicksave.json")` and `F9` to `SaveManager::load_from_file`, preserving vehicle position, speed, wallet balance, active job, and career progression across sessions.

### 4. Shaders, Architectural Facades & Water Shimmer (`engine/rendering/src/shaders.rs`)
- **Procedural Window Grids**: Implemented architectural window patterns in the fragment shader for vertical building facades (`abs(normal.y) < 0.25`), with warm yellow illumination under nighttime conditions (`sun_elevation <= 0.0`).
- **Animated Water Wave Shimmer**: Implemented animated wave specular reflections and directional sun glints using world coordinates and time uniforms (`is_water > 0.5`).

### 5. Automated Certification Suite (`tests/tests/end_to_end_driving_gameplay_certification.rs`)
Implemented 7 automated regression tests:
1. `test_stage3_1_world_integration_and_coordinate_continuity`: Verifies unified ENU anchoring, boundary alignment, camera projection, and zero seams.
2. `test_stage3_2_vehicle_dynamics_anti_tunneling_and_hill_holding`: Verifies spring equilibrium, hill-holding brake torque, and anti-tunneling clamp under severe downward shock.
3. `test_stage3_3_autonomous_traffic_and_platoon_collision_avoidance`: Verifies Intelligent Driver Model (IDM) car-following, braking, and zero rear-end collisions.
4. `test_stage3_4_gameplay_progression_payouts_and_save_reload_persistence`: Verifies job completion, wallet deposit, progression tracking, and `SaveManager` JSON quicksave/quickload roundtrip.
5. `test_stage3_5_garage_vehicle_modes_and_physics_differentiation`: Verifies vehicle fleet presets (sports coupe, taxi, transit bus, heavy truck).
6. `test_stage3_6_visual_quality_shaders_atmosphere_and_presets`: Verifies procedural facade grids, night illumination, water glint, and Rayleigh atmospheric haze.
7. `test_stage3_7_multi_city_authentic_us_road_routing`: Verifies authentic navigation across San Francisco, New York, Los Angeles, Chicago, and Houston.

## Consequences
- **Positive**:
  - The entire world pipeline (GIS → compiler → chunks → streamer → renderer → physics → gameplay) functions seamlessly in live desktop runs.
  - Zero chunk cracks or camera disorientation.
  - Vehicles drive stably with realistic suspension, slope response, and crash recovery.
  - Complete gameplay loop with active contracts, financial payouts, garage fleet cycling, and quicksave/quickload persistence.
  - All 156+ workspace tests pass with 0 warnings.
- **Verification**:
  - `cargo test --test end_to_end_driving_gameplay_certification` (7/7 tests passed).
  - `cargo test --workspace` (100% passed across all crates).
  - `cargo clippy --workspace --all-targets -- -D warnings` (clean).
  - `cargo fmt --check` (clean).
