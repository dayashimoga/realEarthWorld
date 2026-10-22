# ADR 0013: Extensible Multi-Transport Simulation

## Status
Accepted

## Context
Stages 6, 7, and 8 established realistic 4-wheel car dynamics, multi-region intercity transport packages, and a modular gameplay career framework.

Stage 9 introduces **STAGE 9: Extensible Multi-Transport Simulation** under `docs/ExecutionContract.md`. In accordance with the invariant:
> "Extensible Entity Systems: Support future cars, motorcycles, trucks, buses, trains, ships, and aircraft without architectural rewrites."
> "Ensure land/rail/water semantics remain independent."
> "Do not reuse road physics for aircraft."

The platform must support diverse transportation modalities operating concurrently across surveyed planetary geography:
1. **Multi-Class Road Transport**: Extending automotive models to motorcycles (2-wheel lean/camber dynamics), transit buses (high mass, air-suspension, passenger capacity, bus-lane restrictions), and heavy commercial trucks (multi-axle, high center-of-mass, rollover thresholds, cargo tonnage).
2. **Railway Simulation**: Dedicated track graphs, turnouts/switches, block signaling, train consist physics (Davis resistance, tractive effort, steel wheel-on-rail adhesion, pneumatic brake lines), and road level-crossings.
3. **Marine Simulation**: Hydrodynamic buoyancy solvers (Archimedes volume displacement, metacentric stability, wave-surface heave/pitch/roll), hull water resistance, propeller thrust, rudder drift, navigable waterways, and harbor port docking.
4. **Aviation Simulation**: 6-DOF aerodynamic flight model (lift, induced drag, parasite drag, thrust, pitching moment, aileron/elevator/rudder/flap controls, stall dynamics, ground effect), runway threshold/ILS glide-slope guidance, landing gear shock absorption, and high-altitude wide-horizon prefetching ranges.

## Decision

### 1. Unified Multi-Transport Architecture in `realearth-vehicles` and `realearth-navigation`
Rather than creating disconnected ad-hoc implementations, we expand `engine/vehicles` and `engine/navigation` with dedicated, decoupled submodules:
- `engine/vehicles/src/multimodal/`:
  - `motorcycle.rs`: In-line 2-wheel lean dynamics and camber thrust.
  - `bus.rs`: Multi-passenger pneumatic transit bus with swept-path steering.
  - `truck.rs`: Multi-axle commercial heavy hauler with cargo payload and rollover stability.
  - `train.rs`: Steel-wheel railway physics, tractive curves, Davis drag, and Westinghouse air brake propagation.
  - `marine.rs`: Hydrodynamic vessel physics with Archimedes buoyancy, water draft, righting moment, and propeller thrust.
  - `aviation.rs`: 6-DOF aerodynamic flight solver with lift/drag polar, control surfaces, stall modeling, and landing gear dynamics.
- `engine/navigation/src/`:
  - `rail_graph.rs`: Railway track network with turnouts, switches, block signals, and station platforms.
  - `waterway.rs`: Marine shipping fairway channels, harbor docks, and navigational buoys.
  - `air_navigation.rs`: Airport runways, taxiways, holding points, and ILS glideslope approach corridors.

### 2. Independent Land / Rail / Water / Air Semantics
- Road vehicles query DEM terrain and road ribbon colliders via `GroundQuery`.
- Trains are constrained to 1D arc-length $s$ along surveyed rail track splines with superelevation cant.
- Ships interact with the geodetic water surface datum ($0.0\text{m}$ MSL or inland lake elevations) with Archimedes volume displacement and wave heave.
- Aircraft operate in true 3D continuous airspace with barometric altitude and exponential air density lapse $\rho(h) = \rho_0 e^{-h/H}$, transitioning to runway ground interaction during touchdown and rollout.

### 3. High-Altitude Streaming Adaptation
Aircraft moving at $250\text{ m/s}$ ($900\text{ km/h}$) at $10,000\text{m}$ altitude require a wider visibility cone ($30\text{km} - 50\text{km}$). We introduce high-altitude LOD selection in streaming lookahead cones to maintain $\ge 60\text{ FPS}$ without VRAM thrashing.

## Consequences
- **Positive**: Complete multi-transport platform honoring authentic physics for all modes without compromising geographic reality.
- **Positive**: 100% testable in headless simulation test suites; zero GPU or window dependencies.
- **Positive**: Zero cross-modal code coupling: railway semantics do not touch road routing, and flight aerodynamics never reuse wheel friction.
- **Negative**: Expands the vehicle physics solver surface, requiring rigorous regression tests and benchmark validation for each category.
