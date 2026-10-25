# ADR-0046: Stage 1.3-R — Permanent Vehicle Control, Motion and Braking Repair

## Status
**Accepted** (2026-10-25)

## Context
During high-speed gameplay, multimodal navigation, and multi-window desktop usage, users observed several persistent vehicle control, motion, and braking anomalies across all 7 transport modes (Cars, Motorcycles, Buses, Heavy Trucks, Trains, Marine Vessels, and Aircraft):

1. **Stale Inputs and Runaway Acceleration on Focus Loss**: When the viewer window lost OS focus (e.g., Alt-Tab, clicking another window, opening pause or garage modals), keys in `pressed_keys` remained stuck in the active set, causing vehicles to continuously accelerate, steer, or run away without user input.
2. **Inverted Aerodynamic and Hydrodynamic Control Moments**:
   - In [`engine/vehicles/src/multimodal/marine.rs`](file:///j:/rustRealEarth/engine/vehicles/src/multimodal/marine.rs), rudder lift was calculated such that left rudder (`rudder = -1.0`) produced a positive turning moment, incorrectly turning the vessel starboard (+X) instead of port (-X).
   - In [`engine/vehicles/src/multimodal/aviation.rs`](file:///j:/rustRealEarth/engine/vehicles/src/multimodal/aviation.rs), left stick aileron deflection (`aileron = -0.7`) produced a negative roll moment, banking the aircraft rightward instead of leftward.
3. **Road Transport Reverse Drive & Speed Clamping**:
   - In [`engine/vehicles/src/multimodal/road_transport.rs`](file:///j:/rustRealEarth/engine/vehicles/src/multimodal/road_transport.rs), `TransitBusModel`, `HeavyTruckModel`, and `MotorcycleModel` clamped speed to `self.speed_kph = (self.speed_kph + ...).max(0.0)`, completely blocking reverse motion and corrupting signed velocity feedback.
   - Forward speed was calculated globally using `self.vehicle` (Car) rather than the active transport mode's own orientation and linear velocity.
4. **Train Reverser Flutter**:
   - In [`apps/world-viewer/src/interactive.rs`](file:///j:/rustRealEarth/apps/world-viewer/src/interactive.rs), braking to a standstill while holding `S` caused the train reverser to flip continuously at 60 Hz (`reverser = -reverser`), causing locomotive shuttering and invalid brake pipe dynamics.
5. **Standstill Creep vs. Coasting**:
   - Lack of an authoritative vehicle state machine led to ambiguity between aerodynamic/rolling resistance coasting, active service braking, and standstill hold in Park mode.

## Decision

1. **Authoritative Vehicle State Lifecycle**:
   - Introduced `AuthoritativeVehicleState` in [`engine/vehicles/src/multimodal/mod.rs`](file:///j:/rustRealEarth/engine/vehicles/src/multimodal/mod.rs):
     ```rust
     #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
     pub enum AuthoritativeVehicleState {
         Parked,
         Idle,
         Driving,
         Braking,
         Reversing,
         Rail,
         Marine,
         Flight,
         Paused,
     }
     ```
   - Re-exported `AuthoritativeVehicleState` in `realearth_vehicles::AuthoritativeVehicleState`.
   - Updated each 60 Hz physics tick in `InteractiveViewerApp` to maintain authoritative state tracking.

2. **Window Focus & Input Clearing**:
   - Implemented `reset_all_inputs()` in `InteractiveViewerApp`:
     - Clears `pressed_keys` HashSet.
     - Resets `smoothed_steer_input = 0.0`.
     - Deactivates `mouse_drag_active = false`.
     - Resets throttle inputs for multimodal vehicles.
   - Connected `reset_all_inputs()` to `WindowEvent::Focused(false)`, `WindowEvent::Occluded(true)`, window minimization, mode switching (`Digit1`..`Digit7`), and roadside recovery.

3. **Physics Model Repairs**:
   - **Marine Vessel**: Corrected yaw moment sign in `marine.rs`:
     ```rust
     let rudder_arm = -self.length_meters * 0.45;
     let yaw_moment = -rudder_lift * rudder_arm;
     ```
     Verified: Left rudder turns vessel to port (heading decreases); right rudder turns vessel to starboard (heading increases).
   - **Aircraft**: Corrected aileron roll moment sign in `aviation.rs`:
     ```rust
     let roll_moment = -q_s * self.wingspan_meters * (0.35 * aileron_authority);
     ```
     Verified: Left stick rolls left ($\omega_z > 0$); right stick rolls right ($\omega_z < 0$).
   - **Road Transport**: Replaced `.max(0.0)` speed clamps in `road_transport.rs` with signed longitudinal speed `forward_speed = linear_velocity.dot(forward)`. Added proper reverse tractive effort, resistance opposing motion direction, zero-crossing clamp under braking, and absolute standstill hold in Park mode.
   - **Railway Train**: Decoupled train reverser from continuous brake-hold logic. Reverser is exclusively controlled by single-edge `KeyCode::KeyR` events, with `S` key providing smooth service brake deceleration to zero.

4. **Interactive Viewer Decoupling**:
   - Evaluated `active_forward_speed` using the currently active vehicle model's transform and velocity rather than defaulting to `self.vehicle`.
   - Tied 3D mesh brake and reverse lighting directly to authoritative state (`is_road_braking` and `is_road_reversing`).
   - Fed actual driver throttle and brake inputs to `AudioTelemetry`.

5. **Certification Test Suite**:
   - Authored master certification test suite [`tests/tests/stage1_3_vehicle_control_motion_and_braking_certification.rs`](file:///j:/rustRealEarth/tests/tests/stage1_3_vehicle_control_motion_and_braking_certification.rs) covering 6 rigorous parts:
     - Part 1: Steering direction fidelity at all 4 compass headings (North, East, South, West).
     - Part 2: Signed velocity, reverse transmission, and full braking on road modes.
     - Part 3: Railway train single-edge reverser, service braking, and emergency brake venting to 0 bar.
     - Part 4: Standstill anti-drift hold (<2mm over 1200 frames), 18% slope hold, coasting vs active service stopping.
     - Part 5: Action router input clearing on focus loss and mode isolation.
     - Part 6: Multi-framerate (30 / 60 / 120 FPS) fixed-timestep replay bitwise invariance.

## Consequences

- **Safety & Determinism**: Complete elimination of stuck keys, uncontrolled runaway motion, and mode cross-talk.
- **Physical Realism**: True reverse driving, realistic coasting under aerodynamic drag and rolling resistance, and authentic train reverser operation.
- **Visual & Audio Polish**: Accurate brake and reverse lighting on all 3D vehicle meshes, authentic engine RPM synthesis, and clean HUD cluster telemetry.
- **Compatibility**: All existing rendering, GIS streaming, terrain DEM, camera controls, career contracts, and UI modals remain 100% intact.
