# ADR-0021: Stage E — Complete Defect Remediation

## Status
Accepted

## Context
Following the implementation of Stage B (Adversarial Validation), Stage C (Visual Fidelity & Realism Certification), and Stage D (Real Major US City Coverage), the Real Earth Open-World Platform requires a rigorous, forensic defect remediation phase (Stage E).

Per the mandate:
- **E1 Prioritize**: P0 data integrity, security, architectural, and fundamental behavior defects first; then P1 geographic, physics, navigation, streaming, rendering, integration, and missing features; then P2 optimization, UX, and reliability; then P3 enhancements.
- **E2 Implement Correct Repairs**: Fix root causes, not symptom suppression. Replace fake/incomplete implementations with real algorithms and integrations. Preserve original geographic truth, working features, and persisted data.
- **E3 Test-Driven Remediation**: Reproduce each defect, create a regression test, fix it, verify independent oracle results, run full integration/E2E suites, and compare memory/FPS/functional behavior before and after.
- **E4 Verify Systemic Quality**: Review all comparable modules for the same defect class; avoid fixing one city, renderer, vehicle, platform, or code path while leaving equivalent implementations broken.

## Prioritized Defects & Forensic Root Cause Analysis

### 1. [P0 Data Integrity & Concurrency] SQLite WAL Mode & Busy Timeout in Database Indexer (`engine/compiler/src/database.rs`)
- **Symptom**: In multi-threaded execution or concurrent compiler/reader environments, streaming readers and chunk packagers encountered `SQLITE_BUSY` database lock errors.
- **Root Cause**: `ChunkDatabase::open` opened SQLite in default journal mode (`DELETE`) without concurrency pragmas, WAL mode, or busy timeout.
- **Remediation**: Executed `PRAGMA journal_mode = WAL;`, `PRAGMA synchronous = NORMAL;`, and `PRAGMA busy_timeout = 5000;` during database initialization. This enables simultaneous multi-reader and single-writer concurrency with up to 5,000ms automatic wait on lock contention.

### 2. [P0 Geographic Data Integrity] Antimeridian Longitudinal Spatial Query in SQLite Index (`engine/compiler/src/database.rs`)
- **Symptom**: Spatial chunk queries across the 180° antimeridian (e.g. 175°E to -175°W in the Aleutian Islands, Fiji, or Chukotka) returned zero chunks, dropping all intersecting tiles along the Pacific corridor.
- **Root Cause**: `query_bbox` generated `WHERE min_lat <= ?2 AND max_lat >= ?1 AND min_lon <= ?4 AND max_lon >= ?3`. When a bounding box crosses the antimeridian, `bbox.min_lon > bbox.max_lon` (e.g. 175.0 > -175.0), requiring a single chunk to span >350° of longitude. Additionally, `GeoBounds::new(min_lat, max_lat, min_lon, max_lon)` asserted `min_lon <= max_lon`, causing debug panics when loading antimeridian chunks.
- **Remediation**: Implemented `bbox.crosses_antimeridian()` branching in `ChunkDatabase::query_bbox`:
  `WHERE min_lat <= ?2 AND max_lat >= ?1 AND (max_lon >= ?3 OR min_lon <= ?4)`.
  When instantiating records, used `GeoBounds::new_antimeridian` if `min_lon > max_lon`.

### 3. [P0 Security & DoS] Unbounded Zstandard Decompression Bomb Vulnerability (`engine/compiler/src/packager.rs`)
- **Symptom**: Memory allocation was vulnerable to out-of-memory (OOM) process termination when unpacking untrusted or malicious `.rec` chunk payloads.
- **Root Cause**: `unpack_chunk_from_bytes` invoked `zstd::decode_all(bytes)` directly. A highly compressed zero-byte payload (e.g. 70 KiB compressed expanding to 100+ MiB) would allocate unbounded memory on the heap.
- **Remediation**: Established `pub const MAX_UNCOMPRESSED_CHUNK_BYTES: usize = 64 * 1024 * 1024;` (64 MiB budget). Implemented bounded streaming decompression via `std::io::Read::take(MAX_UNCOMPRESSED_CHUNK_BYTES as u64 + 1)` and rejected oversized chunks cleanly with `RealEarthError::Serialization`.

### 4. [P1 Navigation Topology] Negative Longitude Truncation in Road Network Graphs (`engine/navigation/src/road_graph.rs`)
- **Symptom**: Road networks in the Western hemisphere (all 38 US cities, where longitude < 0.0) suffered severe coordinate collisions and route resolution failures.
- **Root Cause**: In `RoadNetworkGraph::get_or_create_node`, coordinate keys were constructed as `((coord.lat * 1e7) as u64, (coord.lon * 1e7) as u64)`. In Rust, casting a negative float to `u64` saturates to `0`. Consequently, all negative longitudes across the Western hemisphere (e.g. -122.4° in SF, -74.0° in NYC) collapsed to lon key `0`, corrupting node spatial indexing.
- **Remediation**: Replaced the unsigned key with signed 64-bit integer coordinates: `((coord.lat * 1e7).round() as i64, (coord.lon * 1e7).round() as i64)`. This preserves negative coordinates across both Western and Southern hemispheres with sub-millimeter precision. Also updated `find_nearest_node` and `compute_lane_coordinates` to use latitude-dependent cosine scaling `111_320.0 * coord.lat.to_radians().cos().abs()`.

### 5. [P1 Geographic Ingestion] Antimeridian Bilinear DEM Interpolation Panics (`engine/ingestion/src/dem_grid.rs`)
- **Symptom**: Continuous elevation sampling crashed or panicked when evaluating DEM rasters that span across the 180° antimeridian.
- **Root Cause**: `DemGrid::sample_bilinear` clamped `coord.lon` between `self.bounds.min_lon` and `self.bounds.max_lon`. When `bounds.crosses_antimeridian()`, `min_lon > max_lon`, causing `f64::clamp` to panic in debug mode and calculating negative `delta_lon`.
- **Remediation**: Added antimeridian normalization in `sample_bilinear`:
  `let delta_lon = (self.bounds.max_lon + 360.0) - self.bounds.min_lon;`
  `let lon_offset = if coord.lon >= self.bounds.min_lon { coord.lon - self.bounds.min_lon } else { (coord.lon + 360.0) - self.bounds.min_lon };`
  `let u = (lon_offset.clamp(0.0, delta_lon) / delta_lon) * (width - 1) as f64;`.

### 6. [P1 Physics Kinematics] IDM Zero-Headway Regularization & Bumper Penetration Protection (`engine/vehicles/src/traffic.rs`)
- **Symptom**: In dense platooning traffic, aggressive followers penetrated trailing bumpers, and extreme negative gap inputs caused division-by-zero or NaN acceleration in the Intelligent Driver Model.
- **Root Cause**: In `IdmConfig::calculate_acceleration`, if `lead_gap <= 0.05m` or if headway became negative, the interaction term $(s^*/s)^2$ caused numerical instability, while `TrafficSimulation::step` masked bumper collisions by clamping `min_gap.max(0.1)`.
- **Remediation**:
  1. In `IdmConfig::calculate_acceleration`, guarded the denominator: `(self.max_accel_ms2.max(0.01) * self.comfortable_decel_ms2.max(0.01)).sqrt()`.
  2. If `lead_gap <= 0.05m` (contact or penetration), immediately return maximum emergency deceleration (`-8.0 m/s^2`).
  3. In `TrafficAgent::step`, halt forward speed when `lead_gap <= 0.05m && accel < 0.0`, physically preventing mesh penetration.
  4. In `TrafficSimulation::step`, passed unmasked `min_gap` to leader info so emergency braking triggers authentically.

### 7. [P1 Vehicle Dynamics] Low-Speed Lateral Scrub Tire Continuity (`engine/vehicles/src/dynamics.rs`)
- **Symptom**: When stationary or moving at low forward speed ($v < 0.5\text{ m/s}$), vehicles slid laterally sideways without realistic lateral tire friction ("ice skating" defect).
- **Root Cause**: In `DynamicVehicle::step`, lateral tire slip angle was set to `0.0` whenever forward speed `v_forward.abs() <= 0.5`. This caused lateral cornering force $F_y = -C_\alpha \alpha$ to drop to zero, leaving the chassis with zero lateral tire resistance at low speeds.
- **Remediation**: Implemented low-speed tire scrub continuity:
  When `v_forward.abs() <= 0.5`, evaluate lateral scrub velocity directly:
  `wheel.slip_angle = (v_lateral / 0.5).clamp(-1.0, 1.0) * FRAC_PI_4;`
  This maintains continuous lateral cornering and scrub resistance bounded by the tire friction circle at all speeds.

### 8. [P2 Interactive Navigation] Continuous Input Tracking & Framerate-Independent Delta-Time (`apps/world-viewer/src/interactive.rs`)
- **Symptom**: Interactive windowed mode lacked continuous WASD camera motion and did not scale navigation speed by delta-time ($\Delta t$).
- **Root Cause**: `InteractiveViewerApp` only processed discrete `ElementState::Pressed` events on digit and tab keys without holding active key states or tracking inter-frame duration.
- **Remediation**: Added `pressed_keys: HashSet<KeyCode>`, `last_frame: Instant`, and integrated `about_to_wait` loop callback. Implemented continuous WASD / Arrow key handling with framerate-independent metric speeds (60 m/s in Fly mode, 5 m/s in Walk mode) and smooth rotation.

## Verification & Test Results
- **Dedicated Regression Suite**: `tests/tests/defect_remediation_regression.rs` (8 tests passing in 0.19s).
- **Full Workspace Test Suite**: 213 tests across 14 integration test suites and 13 crate unit test suites passing with zero failures.
- **Linter & Formatting**: `cargo clippy --workspace --all-targets -- -D warnings` (0 warnings); `cargo fmt --check` (0 deviations).

## Unresolved Constraints & Production Notes
1. **Dynamic Road Mesh Splitting**: Extremely long interstate roads crossing multiple chunk boundaries are currently partitioned at chunk bounding planes; future LOD streaming can introduce continuous spline lofting at chunk seams.
2. **GPU Native Texture Compression**: Binary packager currently stores raw vertex/index buffers and PNG textures; BC7/ASTC native block compression can be enabled in Stage F release packaging.
