# ADR-0057: Stage G1 — Authoritative Vehicle Physics and Controls Repair

## Status
**Accepted** (2026-10-26)

## Context
Following forensic audit Stage G0 ([ADR-0056](file:///j:/rustRealEarth/docs/adr/0056-stage-g0-independent-production-certification-repair.md)), detailed physical inspection and user reports revealed several critical defects in vehicle locomotion, control mapping, stationary stability, and collision boundaries:
1. **Unreliable Stopping & Continuous Backward Rolling:** When driving forward, pressing and holding the brake key (`[S]`) initially slowed the vehicle down, but as forward speed dropped below $0.35\text{ m/s}$ ($1.2\text{ km/h}$), input logic immediately dropped brake pressure to zero and engaged $85\%$ reverse throttle. Consequently, drivers attempting to stop were launched violently into continuous reverse motion.
2. **Spontaneous Turning & Unintended Creeping:** The engine torque converter applied an unconditional $8\%$ forward creep force in `TransmissionMode::Drive`. In conjunction with sub-milliradian steer smoothing residuals ($< 0.005\text{ rad}$), vehicles parked or idling on level ground slowly accelerated, self-steered, and drifted into obstacles.
3. **Building Penetration:** The bounding-circle collision resolution routine in `building_collision.rs` multiplied half-extents diagonal $R$ by an arbitrary factor of $0.85$. This ignored the outer $15\%$ of vehicle bumpers and corners, allowing vehicles to clip and penetrate building facades.
4. **Unsafe Spawning Trap:** The initial vehicle spawn fallback position defaulted to `(0.0, 0.0)`, which is directly in the center of the Transamerica Pyramid skyscraper footprint in San Francisco, spawning cars embedded inside solid geometry.
5. **Latched Input on Focus/Modal Transitions:** Opening menus, console windows, or modal dialogs (such as telemetry, navigation, or graphics settings) did not clear the keyboard buffer, resulting in latched throttle or steering commands after UI interaction.
6. **Ground Query Wheel Dropouts:** In `FunctionalGroundQuery::query_ground_contact`, penetration distances below $0.0$ returned `None`, causing wheels that dipped slightly below surface triangles to lose ground contact, drop out of raycasting, and fall through the world.

---

## Decisions

### 1. Directional Input Routing & Dwell Gating
In [`apps/world-viewer/src/interactive.rs`](file:///j:/rustRealEarth/apps/world-viewer/src/interactive.rs), input handling for forward/reverse travel was overhauled:
- **Service Braking to Complete Standstill:** While moving forward ($v_x > 0.05\text{ m/s}$), pressing `[S]` applies 100% service brake force until the vehicle reaches a complete stop ($v = 0.000\text{ m/s}$).
- **Standstill Hold:** Once stopped, holding `[S]` maintains the service brake at 100%, holding the vehicle immobile at zero velocity.
- **Intentional Reverse Gating:** Transition to reverse gear requires an explicit intentional action: either releasing and re-pressing `[S]` while at standstill, or holding `[S]` past a deliberate dwell threshold (`reverse_dwell_sec > 0.35s`).
- **Symmetric Braking in Reverse:** While reversing ($v_x < -0.05\text{ m/s}$), `[W]` commands service braking to a standstill rather than fighting reverse drive torque.

### 2. Elimination of Spontaneous Creep & Steer Snapping
- **Configurable Auto-Creep:** Added `pub auto_creep_enabled: bool` to `VehicleConfig` in [`engine/vehicles/src/dynamics.rs`](file:///j:/rustRealEarth/engine/vehicles/src/dynamics.rs), defaulting to `false`. Idle torque converter creep in Drive is strictly gated behind this configuration flag.
- **Steer Deadzone Snapping:** In `interactive.rs`, when the target steering input is zero, `smoothed_steer_input` is hard-snapped to `0.0` once residual steering drops below $0.005\text{ rad}$, eliminating sub-milliradian lateral drift.

### 3. Absolute Immobility in Park Mode
- In `TransmissionMode::Park`, linear velocity and angular velocity are unconditionally clamped to zero ($\vec{v} = \vec{0}$, $\vec{\omega} = \vec{0}$).
- Parking brake hold force was increased to $25,000\text{ N}$ across all vehicle classes.
- Updated [`engine/vehicles/src/multimodal/road_transport.rs`](file:///j:/rustRealEarth/engine/vehicles/src/multimodal/road_transport.rs) so that `MotorcycleModel::step`, `TransitBusModel::step`, and `HeavyTruckModel::step` also enforce absolute zero-velocity immobility in `TransmissionMode::Park`.

### 4. Hill-Hold Assist (HHA) & Suspension Equilibrium
- Enhanced Hill-Hold Assist in `dynamics.rs` to automatically engage on road grades up to $35\%$ ($19.3^\circ$) whenever throttle is zero and all wheels are grounded.
- During standstill hold, horizontal velocity components (`linear_velocity.x = 0; linear_velocity.z = 0`) are zeroed, while vertical suspension compliance (`linear_velocity.y`) is preserved unconstrained. This allows suspension springs and dampers to settle into true physical equilibrium without false force spikes.

### 5. Ground Contact Continuity
- In [`engine/vehicles/src/ground.rs`](file:///j:/rustRealEarth/engine/vehicles/src/ground.rs), `FunctionalGroundQuery::query_ground_contact` was updated to accept wheel penetration depths down to $-10.0\text{m}$, returning `Some((0.0.max(dist), normal))`. This matches other ground query implementations and prevents catastrophic wheel dropouts during sudden terrain elevation changes or high-speed impacts.

### 6. 100% Collision Containment & Safe Spawning
- **Collision Radius Correction:** In [`engine/vehicles/src/building_collision.rs`](file:///j:/rustRealEarth/engine/vehicles/src/building_collision.rs), removed the arbitrary `* 0.85` scale factor in `resolve_collision`. Bounding radius is strictly defined as $R = \sqrt{h_x^2 + h_z^2}$, ensuring full coverage of front/rear bumpers and corners.
- **Polygon Penetration Clearance:** Implemented `is_point_inside(&self, pos: Vec3, margin: f32) -> bool` and updated `query_clearance` to return $0.0\text{m}$ clearance if a point lies within any building polygon.
- **Safe Fallback Spawning:** In `interactive.rs`, replaced the hardcoded `(0.0, 0.0)` Transamerica Pyramid spawn with `find_safe_fallback_spawn_position()`, which inspects building geometry and guarantees $\ge 8.0\text{m}$ clearance from all building polygons.

### 7. Input Cleansing on Focus Loss & Modal Transitions
- Added `reset_all_inputs()` calls in `interactive.rs` to:
  - `dispatch_ui_action()` (UI clicks and interactions)
  - Modal hotkeys (`Escape`, `KeyT`, `KeyN`, `KeyJ`, `Tab`, `KeyG`, `KeyO`, `KeyU`, `KeyP`)
  - Window focus loss events
- Cleanses all keyboard state buffers, zeroing throttle, steer, brake, and handbrake latches.

### 8. Master Certification Suite (`stage_g1_vehicle_physics_and_controls_certification.rs`)
Created 8 comprehensive, quantitative automated tests in `tests/tests/stage_g1_vehicle_physics_and_controls_certification.rs`:
1. `test_stage_g1_stationary_park_60_second_slope_stability`: 3,600 physics frames (60s) on 15% and 25% slopes; asserts drift $< 10^{-4}\text{m}$ for car and motorcycle.
2. `test_stage_g1_zero_input_level_ground_hold_no_self_steering`: 1,800 physics frames (30s) in Drive on level ground; asserts displacement $< 10^{-4}\text{m}$ and yaw change $< 10^{-4}\text{ rad}$.
3. `test_stage_g1_canonical_control_signs_across_four_cardinal_headings`: North ($0^\circ$), East ($90^\circ$), South ($180^\circ$), West ($270^\circ$); confirms A always turns left and D always turns right.
4. `test_stage_g1_progressive_service_braking_and_stopping_distances`: Decelerates from 60 km/h and 100 km/h; asserts full stop without reverse surge and 0.000m drift over 180 standstill frames.
5. `test_stage_g1_building_collision_zero_penetration_and_safe_spawning`: High-speed head-on impact at 72 km/h; asserts zero polygon penetration and verifies safe spawn clearance $> 8.0\text{m}$.
6. `test_stage_g1_multimodal_class_appropriate_dynamics`: Verifies acceleration, mass response, turning limits, and Park mode stability across Motorcycle, Transit Bus, and Heavy Truck.
7. `test_stage_g1_focus_loss_and_state_reset_cleansing`: Injects active inputs, triggers focus loss / modal transitions, and asserts all control states snap to zero.
8. `test_stage_g1_fixed_step_accumulator_invariance_30_60_120_fps`: 3 seconds of high-speed steering stepped at 30, 60, and 120 FPS; asserts terminal trajectory variance $< 0.05\text{m}$.

---

## Consequences

1. **Rock-Solid Station-Keeping:** Vehicles parked or stopped at red lights remain completely motionless indefinitely on flat ground and slopes up to 35%, with zero spontaneous turning or creeping.
2. **Intuitive & Predictable Stopping:** Drivers can decelerate smoothly to a complete stop and hold position using `[S]` without any unexpected reverse surges.
3. **Watertight Building Collisions:** Vehicles cannot clip or penetrate building facades, and fallback spawns never trap players inside building interiors.
4. **Zero Latched Inputs:** Modal transitions, UI menus, and focus changes cleanly release all control axes.
5. **Preservation Verified:** All prior regression test suites (`stage_g0_...`, `stage_f_...`, and `stage_b_...`) pass 100% green alongside the new Stage G1 master certification suite.
