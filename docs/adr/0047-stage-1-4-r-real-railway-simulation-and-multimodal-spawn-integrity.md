# ADR-0047: Stage 1.4-R — Real Railway Simulation and Multimodal Spawn Integrity

## Status
**Accepted** (2026-10-25)

## Context
During inspection of railway simulation, multimodal fleet deployment, and geographical scene transitions, critical fidelity gaps were identified:

1. **Straight-Line Fake Train Motion**:
   - In [`apps/world-viewer/src/interactive.rs`](file:///j:/rustRealEarth/apps/world-viewer/src/interactive.rs#L4361-L4369), train motion was previously executed via open-field translation: `self.train_position += forward * (self.train_consist.velocity_mps * PHYSICS_DT)`, allowing locomotives to drive completely off rails across terrain, through skyscrapers, and over water.
   - Rail network topology surveyed in [`engine/navigation/src/rail_graph.rs`](file:///j:/rustRealEarth/engine/navigation/src/rail_graph.rs) was ignored during interactive driving.

2. **Rigid Carriage Rendering**:
   - In [`apps/world-viewer/src/interactive.rs`](file:///j:/rustRealEarth/apps/world-viewer/src/interactive.rs#L4996-L5016), trailing passenger carriages were positioned along a rigid linear vector behind the locomotive yaw (`loco_pos + -Quat(yaw)*Z * offset`), rather than articulating independently along track curve centerlines.

3. **Multimodal Spawning Safety Deficiencies**:
   - Spawning did not enforce modality-specific infrastructure validation:
     - Passenger cars and motorcycles were spawned without checking for legal driving lanes or collision clearance.
     - Transit buses and heavy semi-trucks could spawn onto narrow single-lane paths or pedestrian footways.
     - Marine vessels (ferries) could be placed on dry inland hills (e.g. 15m elevation in SF hills or Mysore) without verifying navigable water depth ($\le 1.0\text{m}$ MSL).
     - Aircraft lacked explicit runway checks or safe airborne corridor altitude initialization (+350m AGL).
     - Railway consists could be spawned in regions lacking any surveyed rail infrastructure by fabricating fictional tracks.

## Decision

1. **Authentic Track-Constrained Railway Simulation**:
   - Extended `RailNetworkGraph` and `RailTrackSegment` in [`engine/navigation/src/rail_graph.rs`](file:///j:/rustRealEarth/engine/navigation/src/rail_graph.rs):
     - Added `sample_pose_enu(&self, s_meters, anchor) -> (RailTrackPose, bool)`: Computes exact 3D position, yaw tangent, elevation gradient pitch, cant angle, and instantaneous curvature ($\kappa = \frac{d\theta}{ds}$).
     - Added `route_cross_track_distance(&self, route_segments, point, anchor) -> f32` to measure physical deviation from track centerline.
     - Added `sample_route_pose(&self, route_segments, distance, anchor) -> Option<(RailTrackPose, usize, f64, bool)>` supporting multi-segment routes, turnouts, and terminus buffer stops.
     - Created authentic San Francisco Caltrain corridor: 6 nodes (`SF_4TH_KING`, `SF_MISSION_BAY_TURNOUT`, `SF_22ND_STREET`, `SF_MISSION_BAY_YARD`, `SF_BAYSHORE`, `SF_MILLBRAE_SFO`), 5 tracks, mainline routes, and yard siding routes.

2. **Consist Route Binding & 3D Articulation**:
   - In [`engine/vehicles/src/multimodal/train.rs`](file:///j:/rustRealEarth/engine/vehicles/src/multimodal/train.rs):
     - Implemented `TrainConsist::new_caltrain_diesel()`: Authentic MPI MP36PH-3C diesel-electric locomotive + 4 Bombardier BiLevel commuter coaches (351.5 tons, 127 km/h line speed).
     - Implemented `bind_route()`, `update_track_poses()`, and `step_along_route()`.
     - Each carriage calculates its precise cumulative arclength offset ($s - \sum \Delta s_i$) and samples its own independent 3D pose (`carriage_poses`), enabling realistic curve articulation, coupler spacing ($25.9\text{m}$), and independent yaw angles.

3. **Authentic Multimodal Spawning Infrastructure Validation**:
   - In [`apps/world-viewer/src/interactive.rs`](file:///j:/rustRealEarth/apps/world-viewer/src/interactive.rs):
     - `find_road_spawn_candidate(min_lanes, max_slope)`: Validates surveyed road geometry, minimum lane count ($\ge 1$ for cars/motorcycles, $\ge 2$ for buses/trucks), legal heading, and building collision clearance.
     - `find_rail_spawn_candidate()`: Validates surveyed `RailNetworkGraph` track segments, platform alignments, and Caltrain mainline routes.
     - `find_marine_spawn_candidate()`: Validates surveyed port facilities or sea-level water polygons ($\le 1.0\text{m}$ MSL). In dry / inland regions, returns explicit rejection `NAVIGABLE WATER REQUIRED / NO VALID SPAWN`.
     - `find_aviation_spawn_candidate()`: Validates surveyed runway/taxiway alignments, or establishes safe open-sky airborne corridor at $+350\text{m}$ AGL above terrain.
     - **Honest Rejection**: If a region contains no surveyed railway infrastructure, the system refuses to invent fictitious tracks and issues an authoritative rejection:
       `RAIL DATA REQUIRED / NO VALID SPAWN: Region contains no surveyed railway infrastructure. Fictitious rail paths are prohibited.`

4. **Interactive Railway Route Selector Modal**:
   - In [`engine/rendering/src/hud.rs`](file:///j:/rustRealEarth/engine/rendering/src/hud.rs) and [`apps/world-viewer/src/interactive.rs`](file:///j:/rustRealEarth/apps/world-viewer/src/interactive.rs):
     - Added `AppUiState::RailRouteModal` toggled via `[T]` key in `ActiveTransportMode::Train`.
     - Renders modal showing line name, departure station, destination station, platform track, route length in km, and constituent track segment breakdown.
     - Up/Down navigation, Enter/Space to dispatch consist to route starting platform.

## Verification

The implementation was validated against a 6-part master certification suite in [`tests/tests/stage1_4_real_railway_simulation_and_multimodal_spawn_certification.rs`](file:///j:/rustRealEarth/tests/tests/stage1_4_real_railway_simulation_and_multimodal_spawn_certification.rs):
- **Part 1**: Track-Constrained 3D Kinematics — Zero cross-track error ($< 0.05\text{m}$), 3D tangent curvature calculation, and Davis train resistance verified.
- **Part 2**: Multi-Carriage Consist Articulation — 5-car consist (MPI MP36PH-3C + 4 Bombardier coaches) independent curve yaw and coupler distance verified.
- **Part 3**: Switch Routing & ATP Emergency Braking — Turnout switching verified; Red signal violation triggers ATP emergency braking ($P_{cyl} \ge 3.5\text{ bar}$, throttle cut, $v = 0.0$).
- **Part 4**: Station Platform Stopping & Terminus Safety — Verified station lookup (`4th & King`, `22nd St`), platform stopping, and buffer stop clamping.
- **Part 5**: Multimodal Spawn Integrity — Rejection of unverified rail and inland marine validated without fake geometry.
- **Part 6**: Multi-Framerate Replay Invariance — 30, 60, and 120 FPS simulations verified within $< 1.5\%$ variance.

Workspace compliance:
- `cargo test --workspace` passed 100%.
- `cargo clippy --workspace --all-targets -- -D warnings` passed with 0 warnings.
- `cargo fmt --check` passed with 0 diffs.
