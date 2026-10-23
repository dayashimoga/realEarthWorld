# ADR-0037: Stage 4 Complete Integrated Car Driving and Physics Experience

## Status
Accepted

## Context
Following the completion of Stage 3 (High-Fidelity Rendering, Architecture, and Environmental Overhaul), Stage 4 of the [Global Engineering Contract](file:///j:/rustRealEarth/docs/ExecutionContract.md) mandates delivering a **Complete Integrated Car Driving and Physics Experience**. 

The goal was to transform the platform from static visual presentation and isolated headless equations into a responsive, authentic, interactive driving application directly coupled with the `wgpu` rendering scene, Rapier3D rigid-body physics, authentic USGS 3DEP terrain elevation, surveyed OpenStreetMap road corridors, and live telemetry instrumentation.

The core requirements include:
1. **Phase 4.1 — Vehicle Representation & Visual Integration**: Multiple visually detailed, properly scaled car models with interior cockpits, animated steering wheel and wheels, dynamic headlights, brake lights, and reverse lights, collision shapes, live vehicle hot-swapping, safe upright resets, and 4 camera perspectives (Chase, Hood, Cockpit, Bumper).
2. **Phase 4.2 — Vehicle Dynamics**: Fixed-timestep Rapier-integrated rigid bodies, realistic vehicle mass/inertia, gravity, 4-wheel independent raycast suspension with spring/damper tuning, tire slip and friction circle limits, Ackermann steering geometry, acceleration, reverse with governor, emergency braking, aerodynamic drag, anti-roll bar (ARB) torsional roll suppression, motionless hill-hold on steep grades, and Continuous Collision Detection (CCD) to prevent obstacle tunneling.
3. **Phase 4.3 — Controls & World Interaction**: Progressive speed-sensitive steering attenuation, automatic transmission and pedal arbitration, camera cycling `[C]`, safe upright recovery `[R]`, garage model switching `[1-6]`, streaming chunk collider pinning lifecycle, and a comprehensive automotive telemetry HUD.
4. **Phase 4.4 — Driving Route Certification**: Rigorous verification across representative San Francisco routes (Market Street, California Street / Nob Hill, Bay Bridge skyway, Lombard Street switchbacks), physical invariant assertions, performance benchmarking, and offscreen driving captures across all 4 perspectives.

---

## Decisions

### 1. Phase 4.1: Vehicle Representation & Multi-Perspective Optics
- **Vehicle Models & Visual Differentiation** in [engine/rendering/src/vehicle_mesh.rs](file:///j:/rustRealEarth/engine/rendering/src/vehicle_mesh.rs):
  - **Sports Coupe**: Lightweight aerodynamic chassis ($1250\text{ kg}$), low cabin profile, rear wing spoiler, front chin splitter, dual polished chrome exhaust tips.
  - **Sedan**: Balanced passenger proportion ($1450\text{ kg}$), chrome front grille slats, dual rear fascia lights.
  - **SUV**: Elevated ride height ($1950\text{ kg}$), front protective skid plate, dual roof utility rack crossbars.
  - **Taxi**: Urban livery ($1550\text{ kg}$), aerodynamic warm-amber illuminated `"TAXI"` rooftop dome.
  - **Transit Bus**: Commercial carrier ($9500\text{ kg}$, $11.5\text{m}$ length), rooftop HVAC pods, multi-axle wheel configuration.
  - **Heavy Truck**: Freight hauler ($14000\text{ kg}$, $14.0\text{m}$ length), intermodal container body, dual vertical exhaust stacks.
- **Detailed Cockpit Interior Geometry**:
  - Driver and passenger bucket seats with ergonomic headrests.
  - Angled dashboard instrument console binnacle.
  - Animated steering wheel tilted at $20^\circ$ whose rotation matches active front wheel steer angle multiplied by a $3.5\times$ steering gear ratio.
  - Side rearview mirror housings.
- **Dynamic Vehicle Lighting**:
  - Dynamic forward headlights (warm-white `[1.0, 0.98, 0.90, 1.0]`).
  - Dual red tail/brake lights that illuminate intensely (`[1.0, 0.05, 0.05, 1.0]`) whenever braking force is applied.
  - Dual white reverse lights (`[0.95, 0.95, 1.0, 1.0]`) on the rear fascia that activate when Reverse transmission mode is engaged.
- **Four Distinct Camera Perspectives** in [engine/vehicles/src/camera.rs](file:///j:/rustRealEarth/engine/vehicles/src/camera.rs):
  - **Chase**: Third-person spring-damper tracking camera ($5.5\text{m}$ behind, $2.2\text{m}$ above) with angular lerp yaw smoothing.
  - **Hood**: Forward-facing perspective mounted on the front bonnet ($+0.8\text{m}$ forward, $+0.9\text{m}$ up).
  - **Cockpit**: First-person interior perspective from the driver's seat ($X=-0.38\text{m}$ left lateral offset, $Y=+1.05\text{m}$ eye level, $Z=+0.15\text{m}$ longitudinal position) with downward pitch ($-0.06\text{rad}$) targeting the road ahead through the windshield.
  - **Bumper**: Low-slung front nose perspective ($+2.1\text{m}$ forward, $+0.45\text{m}$ road clearance) for visceral high-speed immersion.
  - Seamless cyclic switching via `cycle_perspective()` (`Chase` $\to$ `Hood` $\to$ `Cockpit` $\to$ `Bumper` $\to$ `Chase`).

---

### 2. Phase 4.2: Automotive Dynamics & Rapier3D Integration
- **Rigid-Body Simulation with CCD** in [engine/vehicles/src/physics.rs](file:///j:/rustRealEarth/engine/vehicles/src/physics.rs):
  - Dynamic vehicle rigid bodies configured with `.ccd_enabled(true)` to guarantee continuous collision detection against guardrails, barriers, and buildings at speeds $> 120\text{ km/h}$.
  - Spatial collision groups isolating vehicles (`GROUP_VEHICLE`), terrain (`GROUP_TERRAIN`), roads (`GROUP_ROAD`), buildings (`GROUP_BUILDING`), and obstacles (`GROUP_OBSTACLE`).
  - Chunk collider pinning and eviction lifecycle: `register_chunk_collider()` pins colliders during chunk residence, and `remove_chunk_colliders()` cleanly dereferences colliders when streaming chunks are evicted.
- **Automotive Dynamics Model** in [engine/vehicles/src/dynamics.rs](file:///j:/rustRealEarth/engine/vehicles/src/dynamics.rs):
  - **4-Wheel Independent Raycast Suspension**: Non-linear spring force $F_s = -k \cdot \Delta x$, velocity damping $F_d = -c \cdot v_{rel}$, and wheel normal force calculation $F_z = \max(0, F_s + F_d)$.
  - **Anti-Roll Bar (ARB) Stabilization**: Front ($6500\text{ N/m}$) and rear ($5500\text{ N/m}$) torsional spring moments opposing differential wheel deflection ($\Delta x_{left} - \Delta x_{right}$), eliminating excessive body roll during high-speed cornering.
  - **Tire Traction & Friction Circle**: Non-linear lateral cornering force $F_{lat} = -C_\alpha \cdot \alpha$, longitudinal tractive force $F_{long}$, and elliptical vector clamping $\sqrt{F_{long}^2 + F_{lat}^2} \le \mu F_z$.
  - **Reverse Transmission Mode**: Dedicated reverse gear dynamics with half-power torque curve and reverse speed governor capping backward velocity around $45\text{ km/h}$ ($12.5\text{ m/s}$).
  - **Planar-Locked Motionless Hill-Hold**: When stopped on steep grades with service brakes applied and transmission in gear, planar velocities ($v_x, v_z$) are locked to $0.0$, eliminating rollback while preserving vertical suspension compliance. Neutral allows natural gravity-induced rolling.
  - **Chassis Upright Recovery**: `recover_on_surface(ground_y)` resets flipped or overturned vehicles to upright posture resting at equilibrium suspension height above the nearest road surface.

---

### 3. Phase 4.3: Interactive Driving Controls & Telemetry HUD
- **Control Ergonomics & Pedal Arbitration** in [apps/world-viewer/src/interactive.rs](file:///j:/rustRealEarth/apps/world-viewer/src/interactive.rs):
  - **Automatic Reverse Transition**: When stationary ($|v| < 0.2\text{ m/s}$), pressing `S`/Down shifts into Reverse; pressing `W`/Up while in reverse applies service brakes to halt forward before accelerating into Drive.
  - **Speed-Sensitive Steering**: Progressively attenuates maximum steering angle from $1.0\times$ at parking speeds to $0.40\times$ at $120\text{ km/h}$, preventing snap roll at highway velocities while retaining full maneuverability for tight hairpin switchbacks.
  - **Hot-Key Integration**:
    - `[C]`: Cycle camera perspective (`Chase` $\to$ `Hood` $\to$ `Cockpit` $\to$ `Bumper`).
    - `[R]`: Safe vehicle upright recovery on road surface.
    - `[1-6]`: Hot-swap vehicle models (`1`: Sedan, `2`: Sports Coupe, `3`: SUV, `4`: Taxi, `5`: Bus, `6`: Heavy Truck) while preserving fly-mode landmark teleportation.
- **Comprehensive Driving Telemetry HUD**:
  - Digital Speedometer in both KM/H ($3.6 \times v$) and M/S ($v$).
  - Transmission Gear Indicator (`R`, `N`, `D1`..`D5`).
  - Analog Engine Tachometer with visual redline bar (800 to 6500 RPM).
  - Steer Angle readout in degrees ($^\circ$).
  - 4-Wheel Independent Suspension Travel gauges (FL, FR, RL, RR) in meters.
  - Active vehicle model name, camera mode, and control shortcut cheatsheet.

---

### 4. Phase 4.4: Driving Certification Suite & Performance Evidence
- **Forensic Route Test Suite** in [tests/tests/stage4_driving_gameplay_certification.rs](file:///j:/rustRealEarth/tests/tests/stage4_driving_gameplay_certification.rs):
  - **Test 1 (Market Street Straightaway)**: 0-60 km/h acceleration curve, progressive throttle, emergency braking stopping distance $< 50\text{m}$ from $106.7\text{ km/h}$ ($a \approx 10.25\text{ m/s}^2 \approx 1.05g$), reverse transmission engagement, and reverse light activation.
  - **Test 2 (Nob Hill / California Street 22% Grade)**: $22\%$ grade slope climbing ($12.4^\circ$ incline), Anti-Roll Bar side-slope roll conformity, and motionless hill-hold holding vehicle with $< 0.05\text{ m}$ drift without rollback.
  - **Test 3 (Bay Bridge High-Speed Cruise & CCD)**: $120\text{ km/h}$ highway cruise, Continuous Collision Detection (CCD) preventing barrier penetration through thin guardrail obstacle, and chunk collider eviction lifecycle.
  - **Test 4 (Lombard Street Switchbacks)**: Ackermann inner/outer wheel geometry ($|\delta_{inner}| > |\delta_{outer}|$), speed-sensitive lock attenuation ($0.95$ at parking vs $0.40$ at $120\text{ km/h}$), and friction circle lateral cornering traction.
  - **Test 5 (Garage Fleet Hot-Swapping & Recovery)**: Mass, torque, and steering hierarchy across all 6 models, complete 3D interior mesh generation, and inverted chassis recovery on road surface.
  - **Test 6 (Camera Optics & Telemetry Projections)**: Mathematical projection verification across all 4 perspectives, cockpit seat offset alignment, and HUD telemetry conversions.
- **Visual Capture Artifacts**:
  - [sf_stage4_chase.png](file:///C:/Users/daya/.gemini/antigravity-ide/brain/c5a543a1-6e1e-4fdf-83e0-1043a873219b/sf_stage4_chase.png): Third-person tracking view behind sports coupe.
  - [sf_stage4_cockpit.png](file:///C:/Users/daya/.gemini/antigravity-ide/brain/c5a543a1-6e1e-4fdf-83e0-1043a873219b/sf_stage4_cockpit.png): Driver interior view with angled steering wheel, dashboard binnacle, and view down the road.
  - [sf_stage4_hood.png](file:///C:/Users/daya/.gemini/antigravity-ide/brain/c5a543a1-6e1e-4fdf-83e0-1043a873219b/sf_stage4_hood.png): Forward bonnet-mounted perspective.
  - [sf_stage4_bumper.png](file:///C:/Users/daya/.gemini/antigravity-ide/brain/c5a543a1-6e1e-4fdf-83e0-1043a873219b/sf_stage4_bumper.png): Low-slung road clearance perspective.

---

## Consequences

1. **Physical Authenticity**: All vehicle motion, steering, braking, and telemetry are derived directly from genuine Rapier rigid bodies and raycast suspension equations with zero hardcoded or synthetic values.
2. **Anti-Tunneling Assurance**: Continuous Collision Detection (CCD) eliminates obstacle pass-through at high speeds.
3. **No Chassis Sinking or Floating**: Raycast suspension contact and safety clamping guarantee that vehicles rest at equilibrium height on authentic USGS 3DEP terrain elevation.
4. **Clean Decoupling**: Physics simulation runs headlessly with zero window or GPU display dependency, while the interactive viewer connects smoothly to live simulation state.
5. **Quality Gates Passed**: 100% pass rate on full workspace tests (`cargo test --workspace -- --test-threads=1`), 0 formatting errors (`cargo fmt --check`), and 0 clippy warnings (`cargo clippy --workspace --all-targets -- -D warnings`).
