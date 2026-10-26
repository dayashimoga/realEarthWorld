# ADR-0054: Stage E — Complete Multimodal Navigation, User Experience, and Transportation Gameplay

## Status
**Accepted** (2026-10-26)

## Context
Following authoritative physics in Stage B ([ADR-0051](file:///j:/rustRealEarth/docs/adr/0051-stage-b-systemic-defect-remediation-and-certification.md)), premium realistic vehicle assets in Stage C ([ADR-0052](file:///j:/rustRealEarth/docs/adr/0052-stage-c-premium-realistic-transport-models-and-animation.md)), and real GIS streaming in Stage D ([ADR-0053](file:///j:/rustRealEarth/docs/adr/0053-stage-d-real-gis-streaming-and-us-city-transitions.md)), Stage E executes the multimodal navigation, user experience, and gameplay platform remediation defined in `docs/RecoveryContract.md`:

1. **Transport-Specific Navigation & Spawning (Phase E1):**
   - **Road Vehicles (Cars, Motorcycles, Buses, Heavy Trucks):** Valid lane, parking, and depot spawns, collision clearances, legal road routes, and off-road physics.
   - **Locomotives & Rail Consists:** Strict rail constraint enforcement. Locomotives spawn solely on authenticated rail segments at suitable stations, sidings, or depots. Motion is governed by track segment IDs, path distance, switch states, and 3D rail geometry. Rejects straight-line train movement over open terrain or through buildings.
   - **Watercraft:** Spawn exclusively on navigable water polygons, docks, slips, and port fairways with marine buoyancy, hydrodynamic resistance, and passenger deck boarding.
   - **Aircraft:** Spawn on surveyed runway thresholds and designated apron stands. Support full aerodynamic climb/cruise/approach envelopes, 6-DOF physics, and glideslope guidance.

2. **UI/UX & Mouse Repair (Phase E2):**
   - **DPI-Invariant Hit-Testing:** Standardized all UI click detection to physical window coordinates scaled by DPI factor, ensuring pixel-accurate hit-testing across 720p, 1080p, 1152p (1.25x), 1440p (1.5x), and 4K UHD (2.0x).
   - **Modal Exclusivity & Zero Control Leakage:** When modal dialogs (Pause Menu, Garage, Job Browser, City Catalog, Rail Route Selector, Multiplayer, Debrief) are open, world vehicle driving inputs and mouse look are completely decoupled and blocked.
   - **Elimination of Top-Right HUD Overlaps:** Relocated the multiplayer status chip to `y = 54.0` (below the top control buttons at `y = 16.0..46.0`), and shifted the active mission HUD card to `y = 94.0` (below the multiplayer chip), enforcing strictly disjoint vertical coordinate bounds.
   - **Elimination of Aviation PFD Minimap Overlap:** Moved the aviation Primary Flight Display (PFD) from the bottom-right corner (`width_f - 390.0`) to the bottom-left corner (`16.0`), uncrowding the tactical GPS minimap (`width_f - 85.0, height_f - 85.0`) and matching all other vehicle instrument clusters.
   - **Elimination of Bottom HUD Collisions:** Replaced static overlapping footer labels with an interactive glassmorphic control dock (`InGameHudLayout::compute`), and lifted contextual vehicle action prompts to `height_f - 72.0` so they sit cleanly above the dock.

3. **Engaging Transportation Gameplay (Phase E3):**
   - **Multimodal Career System:** Seven diverse career tracks (`TaxiPassenger`, `TransitBus`, `PackageDelivery`, `HeavyFreight`, `RailEngineer`, `FerryCaptain`, `AviationPilot`) and `FreeRoam`.
   - **Dynamic City Job Generation:** Implemented dynamic contract generation across all 38 US metropolitan areas in `JobCatalogGenerator::jobs_for_region`, computing authentic pickup and dropoff coordinates strictly inside surveyed city bounds with route distances exceeding 1,000 meters.
   - **Service Boarding & Dwell Sequences:** Authentic passenger boarding and cargo loading state machines with dwell timers, chime/horn audio events, and passenger comfort ratings.
   - **Save & Resume Persistence:** `SaveManager` roundtrip persistence supporting profile progress, wallet balance, reputation tiers, and geographic coordinates.

4. **Complete Playable Journeys (Phase E4):**
   - Seven master end-to-end integration journeys certified in `tests/tests/stage_e_multimodal_gameplay_and_ux.rs`.

---

## Decisions

### 1. Transport-Specific Infrastructure Navigation & Kinematics ([`engine/vehicles/src/multimodal/`](file:///j:/rustRealEarth/engine/vehicles/src/multimodal/))
* **Rail Graph Constraints:** Enforced `requires_track_pose` in `TrainConsist`. If a rail consist lacks an active route or track pose, emergency brakes are triggered and velocity is zeroed. Track following is evaluated via spline interpolation along `RailTrackSegment` geometry, maintaining cross-track error < 0.05m.
* **Transit Bus Mass & Passenger Comfort:** Modeled `TransitBusModel` with 12,500 kg unladen mass plus 75 kg per passenger (up to 75 passengers, 18,125 kg gross mass), dual-circuit pneumatic air brakes, and dynamic passenger comfort rating deductions during excessive lateral acceleration (> 0.25g).
* **6-DOF Aerodynamic Airliner:** Modeled commercial airliner flight dynamics with atmospheric density lapse rate, dynamic pressure, NACA lift curve ($C_{L0}=0.24, C_{L\alpha}=5.5$), induced drag polar, ground effect cushioning, control surface authority, and stall flow separation.

### 2. Glassmorphic UI/UX and DPI-Invariant Hit-Testing ([`apps/world-viewer/src/interactive.rs`](file:///j:/rustRealEarth/apps/world-viewer/src/interactive.rs), [`engine/rendering/src/hud.rs`](file:///j:/rustRealEarth/engine/rendering/src/hud.rs))
* **Disjoint Top-Right Bounds:**
  - Pause / Camera Buttons: $y \in [16.0, 46.0]$
  - Multiplayer Status Chip: $y \in [54.0, 86.0]$
  - Active Mission HUD Card: $y \in [94.0, 194.0]$
* **Bottom Screen Layout:**
  - Aviation PFD: $x = 16.0, y = \text{height} - 280.0$, width = 260.0, height = 210.0 (bottom-left)
  - Vehicle Instrument Cluster: $x = 16.0, y = \text{height} - 160.0$ (bottom-left)
  - Interactive In-Game Control Dock: centered horizontally at $y = \text{height} - 42.0$
  - Contextual Action Prompt: $y = \text{height} - 72.0$ (elevated cleanly above the control dock)
  - Tactical GPS Minimap: $x = \text{width} - 85.0, y = \text{height} - 85.0$ (bottom-right, completely uncrowded)
* **DPI Invariance:** Layout computations in `InGameHudLayout`, `PauseMenuLayout`, `GarageLayout`, `JobBrowserLayout`, `CatalogLayout`, `RailRouteLayout`, `MultiplayerLayout`, and `DebriefLayout` derive positions from physical window bounds and scale factors, validated across 720p, 1080p, 1152p (1.25x), 1440p (1.5x), and 4K UHD (2.0x).

### 3. Dynamic Non-Placeholder Geographic Job Generation ([`engine/gameplay/src/destinations.rs`](file:///j:/rustRealEarth/engine/gameplay/src/destinations.rs))
* **Surveyed Bounds Proportioning:** Calculated civic plazas, commercial centers, bus depots, transit centers, logistics parks, freight terminals, and airport waypoints proportionally using `city.bounds.min_lat + lat_span * fraction`.
* **Zero Boundary Overflows:** Replaced absolute degree offsets with proportional multipliers ($0.20 \le \text{fraction} \le 0.85$), mathematically guaranteeing that every generated destination lies strictly inside the surveyed city bounding box.
* **Authentic Route Distances:** Ensured every contract route distance exceeds 1,000 meters while honoring metropolitan arterial interstates.

---

## Verification & Certification

All 7 master journeys were certified in [`tests/tests/stage_e_multimodal_gameplay_and_ux.rs`](file:///j:/rustRealEarth/tests/tests/stage_e_multimodal_gameplay_and_ux.rs):
1. **Journey 1 (`test_e1_san_francisco_sports_car_drive_and_stop`):**
   - Sports coupe spawns on road surface at equilibrium ride height.
   - Accelerates under full throttle past 25 km/h (engine RPM > 2500, forward gear engaged).
   - Comes to complete halt under service brakes (< 0.2 km/h).
   - Locks firmly in Park mode under parking pawl ($\Delta \text{dist} < 0.01\text{m}$).
2. **Journey 2 (`test_e2_detailed_transit_bus_passenger_service_route`):**
   - Transit bus boards passengers (35 -> 42 passengers, mass 15,650 kg) emitting passenger chime.
   - Cruises along arterial route reaching > 15 km/h while preserving high passenger comfort rating ($\ge 4.5$).
   - Halts smoothly at next stop platform under pneumatic air brakes with safe reservoir pressure ($\ge 6.0$ bar).
3. **Journey 3 (`test_e3_locomotive_consist_rail_track_following_and_station_stop`):**
   - Caltrain commuter consist (351 metric tons) accelerates along surveyed 4th & King mainline past 15 km/h.
   - Follows track geometry strictly with cross-track error $< 0.05\text{m}$.
   - Approaches station platform and halts completely (< 0.2 km/h) under Westinghouse air brake pressure ($> 1.5$ bar).
4. **Journey 4 (`test_e4_aircraft_airport_departure_and_streamed_flight`):**
   - Commercial airliner executes takeoff climb with 5-degree pitch trim and flaps.
   - Sustains positive rate of climb climbing above 350m MSL with terrain clearance > 300m AGL.
   - Remains completely unstalled within valid aerodynamic flight envelope ($V > 100\text{ kts}$).
5. **Journey 5 (`test_e5_installed_us_city_transition_and_multimodal_spawning`):**
   - Verified Minneapolis–Saint Paul catalog entry (`MSP`, IDS Center anchor).
   - Verified generation of all multimodal careers (`FreeRoam`, `TaxiPassenger`, `TransitBus`, `HeavyFreight`, `PackageDelivery`, `AviationPilot`).
   - Validated that 100% of generated pickup and dropoff coordinates are strictly inside Minneapolis bounds with authentic distances > 1,000m.
6. **Journey 6 (`test_e6_mission_completion_wallet_rewards_save_and_resume`):**
   - Player profile initializes with starting wallet balance $500.0.
   - Accepts San Francisco taxi contract, drives to pickup, dwells for boarding, drives to dropoff, dwells for unloading.
   - Receives full payout with 5-star rating, updates wallet balance to $500.0 + payout, and saves to disk.
   - Reloads save file from disk, verifying roundtrip persistence of profile, wallet, completed jobs, and career tiers.
7. **Journey 7 (`test_e7_mouse_and_ui_interaction_at_multiple_dpi_scales`):**
   - Verified mouse click hit-testing determinism across 720p (1.0x), 1080p (1.0x), 1152p (1.25x), 1440p (1.5x), and 4K UHD (2.0x).
   - Validated hit-testing across Pause Menu, Start Screen, Garage, Job Browser, City Catalog, Rail Route Selector, Multiplayer Status, Debrief Modal, and In-Game HUD Control Dock.
   - Verified zero layout bounding-box collisions across all UI chips, cards, instruments, and minimap.

### Non-Regression Certification Battery
- `cargo test --test stage_e_multimodal_gameplay_and_ux` — **7 passed, 0 failed (0.02s)**
- `cargo test --test stage_d_gis_streaming_and_city_transitions` — **5 passed, 0 failed (0.55s)**
- `cargo test --test stage_c_premium_models_and_animation` — **6 passed, 0 failed (0.07s)**
- `cargo test --test stage_b_authoritative_physics_and_inputs` — **9 passed, 0 failed (0.00s)**
- `cargo test --test stage_b_remediation_certification` — **8 passed, 0 failed (0.02s)**
- `cargo test --test us_city_catalog_validation` — **9 passed, 0 failed (1.21s)**
- `cargo check --workspace` — **Exit 0 (clean compilation across all crates and tools)**
