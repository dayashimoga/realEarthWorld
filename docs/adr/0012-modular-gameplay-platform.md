# ADR 0012: Modular Gameplay Platform

## Status
Accepted

## Context
Stage 6 established a deterministic 60Hz vehicle dynamics simulation and AI traffic model, and Stage 7 extended the planetary platform with authentic multi-region packages (Bengaluru, Mysuru, Chennai, Hyderabad, South India Corridors, and San Francisco) and seamless streaming.

Stage 8 introduces **STAGE 8: Modular Gameplay Platform** as defined in `docs/ExecutionContract.md`. Under the Global Engineering Contract:
1. **World First -> Simulation Second -> Gameplay Third**: Gameplay must never mutate surveyed geographic truth, fabricate terrain/road topology, or couple to rendering.
2. **Decoupled Architecture**: Gameplay logic must run completely headlessly without GPU or display dependencies, functioning via pure event-driven APIs and state machines.
3. **Authentic Careers**: Missions and careers (Free Roam, Taxi/Passenger, Parcel Delivery, Heavy Freight) must use authentic geographic coordinates and landmarks on surveyed roads with verifiable routing and distance calculations.
4. **Reliability & State Persistence**: The platform must support versioned save/load files, robust schema migrations, failure/retry/recovery handling, and deterministic state progression across long sessions.

## Decision

### 1. Dedicated `realearth-gameplay` Engine Crate
We introduce `engine/gameplay` (`realearth-gameplay`) as an isolated, optional workspace member. It depends strictly on `realearth-shared`, `realearth-geospatial`, and `realearth-navigation`. Rendering, windowing, and GPU systems have zero knowledge of gameplay internals.

### 2. Event-Driven Architecture and Job State Machines
- **`JobStateMachine`**: Strict transition graph:
  $$\text{Available} \longrightarrow \text{Accepted} \longrightarrow \text{EnRouteToPickup} \longrightarrow \text{AtPickup} \longrightarrow \text{CargoLoaded} \longrightarrow \text{EnRouteToDropoff} \longrightarrow \text{AtDropoff} \longrightarrow \text{Completed}$$
  With explicit failure transitions: $\text{Failed}$ (timeout, cargo damage, vehicle destroyed, abandonment) and deterministic retry/recovery mechanics.
- **`GameplayEventBus`**: Dispatches strongly-typed events (`JobStarted`, `WaypointReached`, `InteractionTriggered`, `JobCompleted`, `CollisionPenalty`, `LevelUp`, `AchievementUnlocked`) to decouple simulation updates, UI rendering, telemetry, and diagnostics.

### 3. Four Playable Authentic Careers
1. **Free Roam**: Open exploration, landmark discovery, distance milestones, and clean-driving bonuses.
2. **Taxi & Passenger Transport**: Passenger pickup at authentic civic landmarks (e.g. Vidhana Soudha, Mysore Palace, Chennai Central, Charminar, Transamerica Pyramid), passenger boarding dwell, smooth-ride comfort scoring (measuring lateral/longitudinal jerk), dynamic tips, and destination drop-off.
3. **Parcel & Express Delivery**: Urgent time-sensitive deliveries between urban depots and commercial hubs with fragility ratings and deadline countdowns.
4. **Heavy Logistics & Freight**: Long-distance haulage across regional expressways and transport corridors (NH 275, NH 48, NH 44) with vehicle speed compliance, weight simulation, high payouts, and cargo security constraints.

### 4. Player Profile, Garage & Configurable Economy
- **`PlayerProfile`**: Tracks career levels, XP, reputation tiers (`Novice`, `Professional`, `Expert`, `Master`), wallet credits, licenses/endorsements, and career statistics.
- **`VehicleGarage`**: Multi-vehicle ownership, progression from entry-level sedans to luxury taxis, delivery vans, and heavy freight haulers with performance upgrades.
- **`EconomyEngine`**: Configurable rates for base fares, per-kilometer compensation calculated from topological road graph distances, speed bonuses, and penalty deductions.

### 5. Schema-Migrated Save/Load Engine
- **`SaveManager`**: Serializes versioned binary/JSON save slots.
- **`MigrationPipeline`**: Implements declarative forward migrations (e.g., $V1 \to V2$) ensuring forward-compatibility and zero data corruption across updates.
- **Roadside Recovery**: Vehicle recovery/towing service returning immobilized or flipped vehicles to the nearest valid surveyed road node.

### 6. Minimal-Click HUD & Viewer Integration
- Exposes responsive HUD data structures (`ActiveObjectiveHud`, `EarningsHud`, `JobsBrowserHud`, `DebriefModal`) consumed by both interactive windowed viewer and headless CI benchmark telemetry.

## Consequences
- **Positive**: Complete playable careers with zero compromise of geographic or physics integrity.
- **Positive**: 100% testable headlessly; automated regression tests can simulate hundreds of career jobs in milliseconds.
- **Positive**: Zero coupling to `wgpu`, `winit`, or rendering pipelines.
- **Negative**: Adds new crate `realearth-gameplay` and test suite `tests/tests/gameplay_careers.rs` requiring strict coverage and zero lint warnings.
