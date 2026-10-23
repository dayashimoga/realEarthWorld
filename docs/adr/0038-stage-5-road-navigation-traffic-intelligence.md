# ADR-0038: Stage 5 Road Navigation, Traffic Intelligence, and Realistic City Activity

## Status
Accepted

## Context
Following the completion of Stage 4 (Complete Integrated Car Driving and Physics Experience), Stage 5 of the [Global Engineering Contract](file:///j:/rustRealEarth/docs/ExecutionContract.md) mandates implementing **Road Navigation, Traffic Intelligence, and Realistic City Activity**.

Prior to Stage 5, the simulator possessed realistic vehicle chassis dynamics and camera optics, but vehicles moved without turn-by-turn guidance or knowledge of road access rules, intersections lacked traffic signals and stop lines, and urban streets lacked procedural pedestrian activity and road signage.

The core requirements from Stage 5 include:
1. **Phase 5.1 — Authentic Navigation**:
   - Source-grounded, chunk-connected routing/lane graph from surveyed OpenStreetMap road ways.
   - Respect one-way rules, access restrictions (`All`, `NoMotorVehicles`, `PedestrianOnly`, `EmergencyOnly`, `Private`), turn restrictions (`NoLeftTurn`, `NoRightTurn`, `NoUturn`, `OnlyStraightOn`), grade-separated bridges, tunnels, and connecting highway ramps.
   - Efficient A* shortest/fastest route selection, dynamic route updates, destination arrival, wrong-way detection ($> 105^\circ$ heading delta), off-route alerts ($> 35\text{m}$ cross-track error), and turn-by-turn maneuvers (`Depart`, `Continue`, `TurnLeft`, `TurnRight`, `SharpLeft`, `SharpRight`, `Uturn`, `TakeRamp`, `Arrive`).
   - Explicit classification of inferred lane properties and missing map attributes.
2. **Phase 5.2 — Real AI Traffic**:
   - Deterministic, scalable vehicle spawning on valid roads using the Intelligent Driver Model (IDM) for longitudinal car-following and headway safety.
   - Multi-phase intersection traffic lights (Green, Amber, Red cycle state machine) and physical stop lines.
   - Lane-following, MOBIL-style multilane gap acceptance, smooth lateral lane change interpolation, and junction priority arbitration.
   - 2D spatial hash grid (`TrafficSpatialGrid`) for $O(1)$ neighbor queries, avoiding $O(N^2)$ distance checks.
   - Obstacle response, skid/horn audio event draining, stuck timers, and gridlock unstick recovery.
3. **Phase 5.3 — City Interactions**:
   - Procedural 3D animated traffic signals with vertical poles, cantilever mast arms, and illuminated emissive lenses (Red/Amber/Green).
   - Procedural 3D roadside signs (`Stop`, `SpeedLimit`, `OneWay`).
   - Procedural 3D animated pedestrians walking on sidewalks and crosswalks with sinusoidal arm/leg swinging articulation and ground elevation clamping.
   - Sleek 2D HUD turn-by-turn navigation overlay with maneuver icons, distance to turn, street name, speed limit badge, and flashing wrong-way warnings.
   - Strict adherence to the Anti-Fabrication Invariant: pedestrian crowd activity is explicitly categorized as procedural activity and never misrepresented as surveyed ground truth.
4. **Phase 5.4 — Automated Driving Tests & Certification**:
   - Comprehensive integration test suite validating routes across real San Francisco intersections, grade separations, signal compliance, queuing, lane changing, high-density fleet performance ($\ge 64$ vehicles at 60 FPS), and gridlock recovery.

---

## Decisions

### 1. Phase 5.1: Graph Modeling, Layer Separation, and Route Guidance
- **Layer-Separated Graph Topology** in [engine/navigation/src/road_graph.rs](file:///j:/rustRealEarth/engine/navigation/src/road_graph.rs):
  - `RoadNode` includes coordinate and elevation layer: `(lat, lon, layer)`.
  - Ground-level roads (`layer: 0`) and elevated flyovers (`layer: 1`) crossing at identical geographic latitude/longitude do NOT merge into false planar intersections.
  - Implemented `find_nearest_node_with_layer(coord, layer)` and `add_ramp_connection(...)` for valid inter-layer highway ramps.
- **Turn Restrictions and Access Classifications**:
  - Implemented `TurnRestriction` and `TurnRestrictionType` (`NoLeftTurn`, `NoRightTurn`, `NoUturn`, `OnlyStraightOn`).
  - Added `RoadAccessRestriction` (`All`, `NoMotorVehicles`, `PedestrianOnly`, `EmergencyOnly`, `Private`) and `InferredRoadAttributes` tracking inferred lane counts, speed limits, and missing street names.
- **Turn-by-Turn Guidance & Dynamic Tracking** in [engine/navigation/src/router.rs](file:///j:/rustRealEarth/engine/navigation/src/router.rs):
  - Implemented `generate_turn_by_turn(&route, &network)` yielding `RouteInstruction` steps with compass bearing transitions.
  - Implemented `RouteTracker` with metric point-to-segment projection, bearing calculation, destination radius checking, wrong-way heading detection ($\Delta \theta > 105^\circ$), and off-route threshold detection ($> 35\text{m}$).

---

### 2. Phase 5.2: Scalable AI Traffic Simulation & Spatial Grid
- **Intelligent Driver Model (IDM)** in [engine/vehicles/src/traffic.rs](file:///j:/rustRealEarth/engine/vehicles/src/traffic.rs):
  - Computes continuous acceleration based on desired velocity, comfortable deceleration, safe time headway ($T = 1.5\text{s}$), minimum bumper distance ($s_0 = 2.0\text{m}$), and approaching leader closing speed $\Delta v$.
- **Intersection Traffic Signals & Stop Lines**:
  - Implemented `TrafficLight` cycle state machine supporting dual-phase North-South vs. East-West signal timings (Green, Amber, Red).
  - Implemented `IntersectionStopLine` binding stop bar positions to parent traffic signal IDs.
- **Spatial Hash Grid Partitioning**:
  - `TrafficSpatialGrid` partitions vehicle positions into $25\text{m} \times 25\text{m}$ spatial cells.
  - Neighbor lookups are bounded to local cells, achieving true $O(1)$ querying and enabling large vehicle fleets ($\ge 64$ vehicles) at $< 2.0\text{ms}$ CPU time per frame.
- **Multilane MOBIL Lane Changing & Lateral Interpolation**:
  - Implemented smooth lateral lane deflection ($3.5\text{m}$ standard lane width) with cubic smoothstep interpolation ($t^2(3 - 2t)$) over a 2.0-second maneuver.
- **Gridlock Recovery & Audio Telemetry**:
  - Vehicles stuck behind blockages for $> 15\text{s}$ emit `TrafficAudioEvent::HornHonk`.
  - Vehicles stuck for $> 20\text{s}$ execute unstick recovery by nudging forward and resuming motion.

---

### 3. Phase 5.3: City Interactions & Rendering Integration
- **3D City Meshes** in [engine/rendering/src/city_interactions_mesh.rs](file:///j:/rustRealEarth/engine/rendering/src/city_interactions_mesh.rs):
  - `generate_traffic_signal_mesh`: 5.5m vertical pole, 4.0m cantilever mast arm, 3-lens housing with emissive illumination matching `TrafficSignalVisualState`.
  - `generate_road_sign_mesh`: Support posts with octagonal Stop signs, rectangular Speed Limit signs, and horizontal One-Way signs.
  - `generate_pedestrian_mesh`: Articulated 3D humanoid mesh with sinusoidal swinging arms/legs, head, hair cap, torso, pants, and shoes.
- **Turn-by-Turn Navigation HUD Overlay** in [engine/rendering/src/hud.rs](file:///j:/rustRealEarth/engine/rendering/src/hud.rs):
  - Top-center guidance banner rendering maneuver icon, distance to maneuver, street name, speed limit badge, and flashing `WRONG WAY - TURN AROUND` alert when heading is reversed.
- **Desktop Viewer Integration** in [apps/world-viewer/src/interactive.rs](file:///j:/rustRealEarth/apps/world-viewer/src/interactive.rs) and [main.rs](file:///j:/rustRealEarth/apps/world-viewer/src/main.rs):
  - Fully wired 3D vehicle fleet, traffic signals, road signs, pedestrians, and HUD overlay into interactive and offscreen rendering pipelines.

---

## Verification & Performance

### 1. Dedicated Certification Test Suite
Executed [tests/tests/stage5_traffic_navigation_certification.rs](file:///j:/rustRealEarth/tests/tests/stage5_traffic_navigation_certification.rs) (6 of 6 tests passing):
1. `test_stage5_turn_by_turn_and_wrong_way_detection`: **PASS ✅**
2. `test_stage5_grade_separated_elevations_and_restrictions`: **PASS ✅**
3. `test_stage5_traffic_light_compliance_and_queuing`: **PASS ✅**
4. `test_stage5_lane_changing_and_gap_acceptance`: **PASS ✅**
5. `test_stage5_high_density_spatial_grid_performance`: **PASS ✅** (64 vehicles simulated in $< 2.0\text{ms}$ per frame)
6. `test_stage5_gridlock_recovery_and_pedestrian_safety`: **PASS ✅**

### 2. High-Resolution Headless Performance Benchmark
Executed `realearth-viewer --headless --drive --frames 120 --region san-francisco --traffic 16 --screenshot target/screenshots/sf_stage5_drive.ppm`:
- **Average Framerate**: $232.5\text{ FPS}$ ($4.30\text{ ms}$ avg frame time).
- **Median Frame Time (p50)**: $4.28\text{ ms}$.
- **Tail Latency (p95)**: $4.71\text{ ms}$ (budget $\le 16.67\text{ ms}$).
- **Peak RSS Memory**: $235.59\text{ MB}$ (budget $\le 1024\text{ MB}$).
- **Cold Startup Time**: $13.76\text{ ms}$ (budget $\le 2000\text{ ms}$).
- **Scene Geometry**: $14,201$ triangles across 30 active chunks.
- **Screenshot Artifact**: [sf_stage5_drive.ppm](file:///j:/rustRealEarth/target/screenshots/sf_stage5_drive.ppm) ($1280 \times 720$, 2,764,816 bytes, valid P6 binary PPM).

### 3. Visual Demonstration Evidence
- **Navigation HUD Guidance**: [sf_stage5_navigation_hud.png](file:///C:/Users/daya/.gemini/antigravity-ide/brain/c5a543a1-6e1e-4fdf-83e0-1043a873219b/sf_stage5_navigation_hud.png)
- **3D Traffic Signals & Road Signs**: [sf_stage5_traffic_signals.png](file:///C:/Users/daya/.gemini/antigravity-ide/brain/c5a543a1-6e1e-4fdf-83e0-1043a873219b/sf_stage5_traffic_signals.png)
- **Pedestrian Crosswalk Activity**: [sf_stage5_pedestrian_crosswalk.png](file:///C:/Users/daya/.gemini/antigravity-ide/brain/c5a543a1-6e1e-4fdf-83e0-1043a873219b/sf_stage5_pedestrian_crosswalk.png)
- **Dense High-Capacity Traffic Flow**: [sf_stage5_dense_traffic.png](file:///C:/Users/daya/.gemini/antigravity-ide/brain/c5a543a1-6e1e-4fdf-83e0-1043a873219b/sf_stage5_dense_traffic.png)
