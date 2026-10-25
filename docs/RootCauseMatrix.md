# Stage A — Root Cause Matrix & Forensic Defect Taxonomy

**Project:** Real Earth World — Navigable Planetary Simulation  
**Document:** `docs/RootCauseMatrix.md`  
**Machine-Readable Twin:** [`docs/RootCauseMatrix.json`](file:///j:/rustRealEarth/docs/RootCauseMatrix.json)  
**Governance Phase:** STAGE A — FORENSIC ROOT-CAUSE RECOVERY AND NO-REGRESSION FOUNDATION  
**Baseline HEAD Commit:** [`d85d8677e5d87171d3d63bca2d897f1f41489ecf`](file:///j:/rustRealEarth)  

---

## 1. Executive Summary

This document establishes the verified root-cause analysis for all 8 reported P0 defects. Each issue has been traced through the authoritative event flow:
$$\text{Event Input} \longrightarrow \text{Authoritative State} \longrightarrow \text{Simulation} \longrightarrow \text{Geographic Collision} \longrightarrow \text{Rendering} \longrightarrow \text{HUD Telemetry}$$

Every defect is backed by an automated reproduction test in [`tests/tests/stage_a_p0_defect_reproduction.rs`](file:///j:/rustRealEarth/tests/tests/stage_a_p0_defect_reproduction.rs).

---

## 2. Master Root Cause Matrix

| ID | Defect Name | Verified Root Cause | Source References | Reproducing Test Gate | Severity |
|---|---|---|---|---|---|
| **P0-1** | Car steering right when A is pressed / 90° frame shift | 1. Road spawn heading uses `atan2(dx, dz)` vs camera's `atan2(dz, dx)`, producing a 90° angular frame shift.<br>2. `steer_lookahead` in `camera.rs:111` applies negative offset along `car_right`, pulling camera left and throwing vehicle rightward across screen.<br>3. Discrepancy between wheel mesh rotation and steering wheel rotation sign. | [`apps/world-viewer/src/interactive.rs:1290`](file:///j:/rustRealEarth/apps/world-viewer/src/interactive.rs#L1290)<br>[`engine/vehicles/src/camera.rs:111`](file:///j:/rustRealEarth/engine/vehicles/src/camera.rs#L111)<br>[`engine/rendering/src/vehicle_mesh.rs:250`](file:///j:/rustRealEarth/engine/rendering/src/vehicle_mesh.rs#L250) | `test_reproduce_p0_1_car_steering_sign_inversion` | **P0** |
| **P0-2** | Self-steering divergence under zero input | Tire slip angle formula uses `(v_lateral / v_forward.abs()).atan()`. When rolling backward down a grade or in reverse with `steer = 0.0`, front wheels lack caster self-aligning trail, creating divergent positive-feedback yaw torque that causes self-steering. | [`engine/vehicles/src/dynamics.rs:491-495`](file:///j:/rustRealEarth/engine/vehicles/src/dynamics.rs#L491-L495) | `test_reproduce_p0_2_self_steering_divergence` | **P0** |
| **P0-3** | Backward movement on slope with zero input in Drive D1 | `is_flat_idle` requires `slope_grade < 0.025` (`dynamics.rs:739`). On realistic hill slopes (e.g. San Francisco at 8%), static friction hold is disabled and automatic transmission creep/hill-hold is missing, causing gravity rollback. In `interactive.rs:4674`, rolling backward trips `active_forward_speed < -0.3`, causing pressing `W` to apply service brakes (`brake = 0.95`) and force `gear_mode = Reverse`. | [`engine/vehicles/src/dynamics.rs:739`](file:///j:/rustRealEarth/engine/vehicles/src/dynamics.rs#L739)<br>[`apps/world-viewer/src/interactive.rs:4674`](file:///j:/rustRealEarth/apps/world-viewer/src/interactive.rs#L4674) | `test_reproduce_p0_3_backward_movement_on_slope_zero_input` | **P0** |
| **P0-4** | Building collision inward penetration trapping | In `building_collision.rs:24-29`, collision normal is calculated as `diff = p - closest`. When `p` is inside a building, `p - closest` points inward into the building interior. `BuildingCollisionWorld::resolve_collision` then adds `normal_3d * pen_depth` to `position`, pushing the vehicle deeper inside and trapping it permanently. | [`engine/vehicles/src/building_collision.rs:24-29`](file:///j:/rustRealEarth/engine/vehicles/src/building_collision.rs#L24-L29)<br>[`engine/vehicles/src/building_collision.rs:136-139`](file:///j:/rustRealEarth/engine/vehicles/src/building_collision.rs#L136-L139) | `test_reproduce_p0_4_building_collision_inward_penetration_trapping` | **P0** |
| **P0-5** | Placeholder primitive fallback meshes & camera occlusion | 1. 3D models in `assets/vehicles/*.glb` are never bound at runtime; rendering falls back to procedural primitive boxes in `vehicle_models.rs:3426`.<br>2. For `HeavyTruck`, trailer length is 15.0m while camera chase distance is 18.0m, placing camera within 1.95m of the tall trailer back wall and occluding 90% of the screen. | [`engine/rendering/src/vehicle_models.rs:3426`](file:///j:/rustRealEarth/engine/rendering/src/vehicle_models.rs#L3426)<br>[`engine/rendering/src/gltf_asset.rs:279`](file:///j:/rustRealEarth/engine/rendering/src/gltf_asset.rs#L279) | `test_reproduce_p0_5_placeholder_primitive_fallback_meshes` | **P0** |
| **P0-6** | Train simulation free roam without track geometry | When `res.rail_graph` is `None` or track pose is unlinked, fallback calls `self.train_consist.step(PHYSICS_DT)` (`interactive.rs:4828`), which simulates unconstrained 3D motion over open terrain and roadways rather than enforcing track adherence. | [`apps/world-viewer/src/interactive.rs:4827-4829`](file:///j:/rustRealEarth/apps/world-viewer/src/interactive.rs#L4827-L4829) | `test_reproduce_p0_6_train_without_track_free_roam` | **P0** |
| **P0-7** | Aircraft leaving geographic world bounds | Airliner accelerates to 70–200 m/s with zero geographic bounding clamping or floating origin regional re-anchoring. Within 15 seconds, displacement exceeds the 2,500m San Francisco package bounds, escaping into black void space. | [`engine/vehicles/src/multimodal/aviation.rs:337`](file:///j:/rustRealEarth/engine/vehicles/src/multimodal/aviation.rs#L337) | `test_reproduce_p0_7_aircraft_leaving_geographic_world` | **P0** |
| **P0-8** | US city transitions failing for catalog-only definitions | `authentic.rs:270-305` whitelists only 6 sample cities. The remaining 32 of 38 cities in the menu are metropolitan definitions only with zero pre-compiled chunk packages installed on disk. Invoking `verify_city_playable` immediately aborts transition with an error. | [`engine/ingestion/src/authentic.rs:270-305`](file:///j:/rustRealEarth/engine/ingestion/src/authentic.rs#L270-L305) | `test_reproduce_p0_8_unsuccessful_us_city_transitions` | **P0** |

---

## 3. End-to-End Tracing Breakdown

### 3.1 P0-1: Steering Sign & Camera Yaw Discrepancy
* **Event Input:** User presses KeyA (`KeyCode::KeyA`). In `interactive.rs:4593`, `steer_left = true`.
* **Authoritative State:** `target_steer = -0.65 * speed_factor`.
* **Simulation:** In `dynamics.rs:440`, `steer_rad = inputs.steer * max_steer_angle = -0.4225 rad`. Front wheels turn toward $-X$ (Left).
* **Geographic Collision / Road Tangent:** Road tangent heading was computed via `tangent.0.atan2(tangent.1)` (`dx.atan2(dz)`). This measures angle relative to $+Z$ (North). In contrast, the camera controller computes yaw via `forward.z.atan2(forward.x)`, measuring angle relative to $+X$ (East). This produces a 90-degree ($\pi/2$) frame shift at spawn.
* **Camera Lookahead:** In `camera.rs:111`, `steer_lookahead = car_right * (steer_angle * 1.5)`. Negative steer angle pulls the camera target to the left, which rotates the camera left and visually shifts the car to the right of the screen.

### 3.2 P0-3: Backward Hill Rollback in Drive
* **Event Input:** Vehicle placed on San Francisco hill (grade = 0.08, ~8%). Inputs: `throttle: 0.0, brake: 0.0, gear_mode: Drive`.
* **Authoritative State:** Driver has transmission in Drive, expecting idle forward creep or hill-hold.
* **Simulation:** In `dynamics.rs:739`, `is_flat_idle` evaluates to `false` because `slope_grade = 0.08 >= 0.025`. Because automatic transmission torque converter creep is absent, gravity acts unopposed and accelerates the car backward down the hill.
* **HUD / Control Interlock:** When the driver presses `W` to drive forward, `interactive.rs:4674` checks `active_forward_speed < -0.3`. Since the car is rolling backward at $-2.8\text{ km/h}$ ($-0.78\text{ m/s}$), this threshold is tripped, forcing `road_inputs.brake = 0.95` and `road_inputs.gear_mode = Reverse`. The car brakes aggressively and switches into Reverse gear.

### 3.3 P0-4: Building Inward Normal Trapping
* **Simulation & Collision:** In `building_collision.rs:24-29`:
  ```rust
  let diff = p - closest;
  let d = diff.length();
  let normal = diff / d;
  ```
  When the vehicle penetrates a building, `p` is *inside* the footprint polygon, and `closest` is on the exterior boundary edge. Therefore, `p - closest` points from the edge into the building interior.
* **Resolution Step:** In `building_collision.rs:136-139`:
  ```rust
  *position += normal_3d * pen_depth;
  ```
  Adding `normal_3d * pen_depth` pushes the vehicle deeper into the building, permanently trapping it.
