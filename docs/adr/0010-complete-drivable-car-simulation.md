# ADR 0010: Complete Drivable Car Simulation Architecture

## Status
Accepted

## Context
Stage 6 of the Global Engineering Contract requires implementing a **Complete Drivable Car Simulation** integrated with the Real Earth World Platform while preserving standalone world-viewer operation. 

Under our core mission (World First -> Simulation Second -> Gameplay Third), the vehicle simulation must:
1. **Decouple Physics from Rendering:** Physics must run in a deterministic, fixed-timestep loop (60 Hz) independent of the display refresh rate, with state interpolation to eliminate visual jitter.
2. **Employ Authentic Vehicle Dynamics:** Rapier3D provides generic rigid-body mechanics (gravity, collisions, friction), but does *not* supply an automotive raycast vehicle dynamics model. A dedicated automotive physics solver must model 4-wheel independent suspensions (springs and dampers), tire slip angles, lateral cornering stiffness, longitudinal tractive forces, brake torques, weight transfer, and slope reactions.
3. **Ensure Terrain and Road Contact Continuity:** Vehicles must traverse high-relief terrain, bridges, tunnels, ramps, and intersections without falling through meshes, hovering above surfaces, or penetrating buildings.
4. **Support Rule-of-the-Road Lane Traversal:** In Bengaluru (India), traffic adheres to Drive-on-Left (Left-Hand Traffic, LHT), while San Francisco (US) adheres to Drive-on-Right (Right-Hand Traffic, RHT). Navigation graphs must accurately place vehicles in their legal lanes.
5. **Support Multi-Agent Traffic & Player Control:** Responsive keyboard/controller inputs with Chase and Hood cameras, complemented by automated AI traffic agents following legal lane trajectories with collision avoidance.

## Decisions

### 1. Dedicated Automotive Dynamics Solver (`realearth-vehicles::dynamics`)
Instead of assuming rigid-body colliders suffice for automotive driving, we implement an automotive raycast suspension and tire dynamics model:
- **Chassis Body**: A Rapier3D dynamic rigid body with realistic mass ($1450\text{ kg}$ sedan baseline), inertia tensor, and aerodynamic drag.
- **Raycast Suspension Struts**: Four independent wheels (FL, FR, RL, RR) casting downward rays from chassis hardpoints:
  - Spring force: $F_{\text{spring}} = K_s \cdot \Delta x$ ($K_s \approx 35,000\text{ N/m}$)
  - Damping force: $F_{\text{damper}} = -K_d \cdot v_{\text{susp}}$ ($K_d \approx 3,200\text{ N}\cdot\text{s/m}$)
  - Total normal load: $F_z = \max(F_{\text{spring}} + F_{\text{damper}}, 0.0)$
- **Tire Traction & Friction Model**:
  - Longitudinal slip ratio $s_x$ and lateral slip angle $\alpha_y$.
  - Cornering force: $F_y = -C_\alpha \cdot \alpha_y$ clamped by friction circle $\sqrt{F_x^2 + F_y^2} \le \mu \cdot F_z$.
  - Drive modes: Automatic transmission (`Drive`, `Reverse`, `Neutral`, `Park`) with progressive throttle, anti-lock braking (ABS approximation), and handbrake locking.
- **Slope Response & Hill Holding**: Natural gravitational resistance on inclines; vehicle remains stationary on hills when braking without drifting through meshes.
- **Flip Recovery**: Automatic or manual (`R` key) attitude stabilization returning overturned vehicles to the nearest road surface.

### 2. Fixed-Timestep Bevy ECS Integration & Presentation Interpolation (`realearth-simulation`)
- The simulation runs at a deterministic $60\text{Hz}$ fixed timestep ($\Delta t = 1/60\text{ s}$).
- State snapshots (`VehiclePhysicsState`) record position, orientation quaternion, and wheel compressions at ticks $N-1$ and $N$.
- The renderer consumes interpolated transforms:
  $$\mathbf{p}_{\text{render}} = (1 - \alpha) \mathbf{p}_{N-1} + \alpha \mathbf{p}_N$$
  guaranteeing jitter-free visuals regardless of display refresh rate.

### 3. Collision Filtering & Drivable Mesh Continuity
- Rapier collision groups isolate layers:
  - Layer 0 (`VEHICLE`): dynamic vehicle bodies and raycasts.
  - Layer 1 (`ROAD`): drivable road ribbons with asphalt friction ($\mu = 0.85$).
  - Layer 2 (`TERRAIN`): continuous DEM ground colliders.
  - Layer 3 (`OBSTACLE`): buildings, walls, and roadside infrastructure.
- Safe Spawning: Vehicle spawns at road centerline elevation $+0.40\text{m}$ clearance with zero initial vertical velocity.

### 4. Lane-Aware Navigation & AI Traffic Flow (`realearth-navigation`)
- Road networks represent lanes with legal driving side offsets:
  - `RuleOfTheRoad::DriveOnLeft`: used for Bengaluru (India) datasets.
  - `RuleOfTheRoad::DriveOnRight`: used for US datasets.
- AI Traffic Agents: automated vehicles following road graph trajectories using the Intelligent Driver Model (IDM) for headway spacing and speed regulation.

### 5. Dual-Mode Viewer Operation (`apps/world-viewer`)
- If run with `--drive` (or when pressing `V` to toggle vehicle mode), the viewer spawns the drivable car and switches to third-person `FollowCamera` or `HoodCamera`.
- If run without `--drive`, the viewer maintains full backward compatibility as a 6-DOF flying and pedestrian exploration tool (`Fly` / `Walk` modes).

## Consequences
- Authentic vehicular handling with realistic weight transfer, body roll, braking dive, and traction limits.
- Zero floating or terrain penetration during high-velocity maneuvers or rough road crossings.
- Complete backward compatibility with the existing world-viewer architecture.
- Deterministic headless testing for autonomous navigation and physics verification.
