# ADR-0049: Stage 1.6-R Premium Multimodal Vehicle Visual & Physical Animation Overhaul

## Status
Accepted / Certified

## Context
In previous releases, while the underlying vehicle dynamics, Pacejka tire models, locomotive tractive physics, and DEM terrain clamping functioned with physical authority, visual representations for several vehicle classes (transit buses, locomotives, passenger coaches, freight semitrailers, aircraft, and ferries) relied on simplistic placeholder shapes or rudimentary boxes.

Specifically:
1. The low-floor urban transit bus visually lacked distinctive curved windshield aerodynamics, cabin passenger aisles, interior seating rows, and bi-fold boarding doors.
2. The locomotive and passenger rail cars lacked authentic North American diesel-electric visual anatomy (wide-nose safety cabs, 2-axle/3-axle articulated bogies with flanged steel wheelsets conforming to standard 1,435mm gauge, AAR knuckle couplers, undercarriage fuel tanks/air reservoirs, and dual-level Bombardier BiLevel carbody profiles).
3. Physical kinematic and dynamic animations were incomplete: wheels spun and steered, but bogies did not articulate with track curvature, motorcycles did not lean into turns, passenger doors did not open at transit stops, aircraft propellers and turbofans were static, and marine radar antennas did not rotate.

## Decision
We implemented a comprehensive, multimodal vehicle visual and physical animation overhaul that preserves 100% of the underlying simulation, vehicle physics, collision detection, and camera systems.

### 1. High-Fidelity 3D Vehicle Fleet Modeling
All 10 distinct vehicle assets across 12 visual classes were redesigned from first principles with authentic proportions, non-intersecting geometries, and rich PBR materials:
- **Apex GT Sports Coupe**: 4.45m × 1.92m × 1.25m, curved hood scoop, aerodynamic decklid spoiler, quad exhaust, slotted rotors, and Brembo-red calipers.
- **Executive Luxury Sedan**: 4.85m × 1.86m × 1.45m, sculpted 3-box notchback profile, chrome window surrounds, multi-spoke alloy wheels.
- **Overland Explorer SUV**: 4.90m × 1.98m × 1.75m, high ground-clearance 2-box rugged wagon, lower perimeter cladding, roof rails.
- **Metro Yellow Cab**: 4.85m × 1.86m × 1.52m, municipal taxi livery, illuminated roof medallion lantern.
- **TransContinental Semi & 53ft Trailer**: 19.4m total length, Class 8 highway tractor with dual tandem drive axles, vertical chrome stacks, and an articulated 53ft dry van semitrailer pivoting on the fifth-wheel kingpin plate.
- **CityLine 40ft Low-Floor Transit Bus**: 11.9m × 2.50m × 3.10m, raked aerodynamic panoramic front glass, amber LED destination matrix header, interior cabin aisle with passenger seating rows, grab poles, and dual bi-fold entrance/exit doors.
- **Pacific Diesel-Electric Locomotive**: 19.9m × 3.15m × 4.60m, authentic North American wide-nose safety cab, twin 2-axle track-articulated bogies with flanged steel wheelsets conforming to standard 1,435mm gauge, knuckle couplers, undercarriage diesel fuel tank, and cyclops high-intensity headlight.
- **Bombardier BiLevel Commuter Rail Coach**: 27.3m × 3.00m × 4.40m, octagonal stainless steel carbody, dual-level passenger windows, end gangway diaphragms, and articulated railway bogies.
- **Golden Gate Passenger Ferry**: 42.2m × 10.7m × 8.5m, high-speed catamaran with twin sponson hulls, razor wave-piercing knife bows, observation bridge, stern waterjet propulsion nozzles, and rotating open-array radar scanner mast.
- **SkyCourier B737 Narrow-Body Airliner**: 35.3m × 34.5m × 9.4m, swept wings with blended winglets, twin underwing turbofan nacelles with spinning titanium fan blades.
- **Skyhawk C172 General Aviation Aircraft**: 8.1m × 11.0m × 3.1m, high-wing strut-braced monoplane with spinning nose propeller and tricycle landing gear.

### 2. Multi-Node Semantic Submesh Architecture
Every vehicle model is decomposed into semantically typed submesh nodes conforming to Khronos glTF 2.0 conventions:
- `ChassisBody`, `GlassGreenhouse`, `InteriorCockpit`, `InteriorPassengerSeats`
- `WheelFrontLeft`, `WheelFrontRight`, `WheelRearLeft`, `WheelRearRight`
- `TrainBogieFront`, `TrainBogieRear`, `TrainWheelset`, `TrainCoupler`
- `BusDoorFront`, `BusDoorCenter`
- `TruckArticulatedTrailer`
- `AircraftPropeller`, `AircraftTurbofanFan`, `AircraftControlSurface`
- `MarineRadarMast`, `MarineWaterjets`
- `MotorcycleLeanBody`

### 3. Physical Kinematic & Dynamic Animation Pipeline
The rendering pipeline evaluates submesh articulation dynamically per frame without affecting authoritative physics:
$$\mathbf{p}_{\text{world}} = \mathbf{p}_{\text{entity}} + \mathbf{R}_{\text{entity}} \cdot \left(\mathbf{p}_{\text{pivot}} + \mathbf{t}_{\text{local}} + \mathbf{R}_{\text{local}} \cdot \mathbf{p}_{\text{rel}}\right)$$
1. **Wheel Spin & Steering**: $v = \omega r$ rotation combined with Ackermann steering yaw and suspension compression.
2. **Motorcycle Dynamic Lean**: Roll banking angle applied to `MotorcycleLeanBody` as a function of lateral cornering acceleration.
3. **Transit Bus Door Articulation**: Bi-fold doors slide and rotate outward when vehicle speed drops below $1\text{ km/h}$ at stops.
4. **Railway Bogie Curvature Articulation**: Bogies independently articulate around their vertical pivot axes to negotiate track curvature.
5. **Articulated Semitrailer Yaw**: Fifth-wheel hitch articulation rotates the 53ft trailer relative to tractor heading.
6. **Propulsion Dynamics**: Continuous angular rotation of aircraft propellers, turbofan fan blades, and marine radar antennas.

### 4. Level of Detail (LOD) & glTF 2.0 Export Pipeline
- **LOD0 High**: Full visual fidelity with detailed rims, calipers, seats, and curved fillets ($< 5,000$ triangles).
- **LOD1 Medium**: Silhouette-preserving simplification with reduced cylinder tessellation ($< 2,000$ triangles).
- **LOD2 Low**: High-efficiency distant fleet impostor meshes ($< 1,000$ triangles).
- **Batch GLB Export**: `export_all_certified_models_to_glb_directory` outputs valid Khronos glTF 2.0 binary `.glb` containers with PBR material definitions, embedded buffers, and semantic scene nodes, verifiable by `GltfParser::parse_glb`.

## Consequences
- Authentic visual fidelity for all vehicle modes with zero placeholder shapes.
- Physical animations are 100% bound to authoritative simulation states with zero independent drift.
- Full compatibility with the existing headless rendering, simulation replay, and multi-platform packaging pipelines.
