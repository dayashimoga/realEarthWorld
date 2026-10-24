# GLOBAL ENGINEERING QUALITY CONTRACT — REAL EARTH WORLD PLATFORM

**Role Definition:** Act as Principal Rust, Game/Simulation, Graphics, GIS, Physics, Distributed Systems, Performance, Platform, QA, Build, Security, UX, and Documentation Engineers.  
**Governing Standard:** Forensic Integrity, Source-Grounded GIS, Independent Test Oracles, and Zero Fabrication.  
**Status:** BINDING ARCHITECTURAL CONTRACT (Updated Stage 0 Foundation)  

---

## 1. Prime Directive: Truth in Engineering

1. **Anti-Fabrication Invariant**: Never fabricate geographic facts, runtime capabilities, test outcomes, or platform readiness. Procedural generation, synthetic templates, mock stubs, and in-memory test arrays are strictly forbidden in production pipelines and certification suites.
2. **Revocation of Unverified Certification**: The platform must never be declared production-ready while major capabilities (e.g. real metropolitan GIS datasets, responsive mouse UI controls, native mobile GPU swapchains) remain stubs, mock fallbacks, or non-functional components.
3. **Evidence-Based Traceability**: Every requirement in `docs/FeatureQualityMatrix.md` and `docs/FeatureQualityMatrix.json` must be backed by genuine, executable, observable source code and tests. Deficiencies must be honestly categorized as `BROKEN`, `LOW-QUALITY`, `PARTIAL`, or `BLOCKED`.

---

## 2. Mandatory Architectural Stack

- **Core Language & Toolchain:** Rust Stable (`2021` edition); `cargo test`, `Criterion`, `cargo-llvm-cov`, `Clippy` (`-D warnings`), `rustfmt`.
- **Rendering & Presentation:** `wgpu` (WebGPU on DX12/Metal/Vulkan); `winit` for window management, event dispatch, and input routing.
- **Simulation & ECS:** `Bevy ECS` decoupled from rendering; `Rapier3D` for rigid-body physics, raycast suspension, and terrain heightfield collision.
- **Asynchronous Execution & Concurrency:** `Tokio` asynchronous runtime; `Rayon` data parallelism.
- **Data & Serialization:** Versioned schemas; `SQLite` indexing/metadata with WAL mode concurrency; compressed binary world chunks (`bincode` + `zstd`).
- **Geographic Data Sources:** Real OpenStreetMap vector data (ODbL 1.0); USGS 3DEP / Copernicus DEM rasters (Public Domain / CC0); licensed land cover.
- **Networking:** Optional authoritative UDP networking (`realearth-networking`) with client-side prediction and server reconciliation.

---

## 3. Strict Architectural Invariants

1. **Headless Execution Invariance**: The world compiler, GIS validation pipeline, navigation router, and physics simulation must execute headlessly with zero display server, window handle, or GPU presentation dependency.
2. **Decoupled Graphics & Presentation**: The rendering engine (`realearth-rendering`) must never own simulation, geographic validation, or gameplay logic. Simulation state must be queryable without rendering.
3. **Decoupled User Interface & Responsive Input**: The UI system must provide hit-testing, mouse click handling, hover state updates, and keyboard accessibility. Menus must never block or obscure the underlying 3D world behind opaque barriers without user dismissal.
4. **Independent Replaceability**: Physics, graphics, navigation, gameplay, and networking must remain independently replaceable without cross-cutting rewrites.
5. **Deterministic Foundation**: Use explicit APIs, immutable snapshots where appropriate, versioned schemas, deterministic simulation steps, bounded work queues, structured errors, and traceable state transitions.

---

## 4. Geographic Truth & Provenance Mandates

1. **Source-Grounded GIS**: Use real, licensed, provenance-tracked GIS data. Preserve WGS84 coordinates, appropriate projected coordinates (UTM), global ECEF/local ENU transformations, double-precision geographic positions (`f64`), floating origin camera systems, and vertical datums (NAVD88).
2. **No Mock Synthesis in Production**: Do not generate synthetic 2-building or 3-road XML strings and falsely label them "authoritative". Missing geographic regions must be explicitly cataloged as `CATALOG_DEFINITION_ONLY` until raw OSM/DEM files are acquired on disk.
3. **Source-Hash Cache Invalidation**: The world compilation pipeline must store source asset SHA-256 digests in package manifests. Startup routines must verify that compiled chunk databases match the current raw asset hashes; any discrepancy must trigger automated recompilation.
4. **Topological Invariance**: Validate polygon winding, grade separations, elevation limits, and network connectivity. Overpasses and bridges (Layer 1) must maintain vertical clearance and never collapse into planar road junctions.

---

## 5. Performance & Telemetry Standards

1. **Measured vs Synthetic Metrics**: Frame pacing, latency, memory RSS, and GPU allocations must be measured against real OS clocks (`std::time::Instant`) and genuine process memory APIs. Never test performance by populating hardcoded mock arrays (`vec![48.5; 10]`).
2. **Presentation vs Throughput Separation**: Clearly distinguish presented windowed FPS (VSync-bounded, 60 Hz) from unthrottled offscreen compute throughput.
3. **Resource Budgets on Reference Hardware**:
   - Cold startup: $\le 2.0\text{ seconds}$ from compiled local chunk cache.
   - Working set memory: $\le 1024\text{ MB}$ RSS.
   - Steady-state frame time: $\le 16.67\text{ ms}$ (60 FPS) at High graphics preset on documented reference desktop hardware.
4. **Telemetry Transparency**: The application must report real-time presented FPS, frame time variance, active chunk count, resident memory, and frustum-culled triangle counts.

---

## 6. Testing Strategy & Independent Oracles

1. **Mandatory Quality Gates**:
   - Core testable code coverage $\ge 90\%$; report line and branch coverage.
   - 100% pass rate for required tests.
   - Zero Clippy warnings under `-D warnings` and zero formatting deviations.
   - Zero unapproved security vulnerabilities under `cargo audit`.
2. **Independent Test Oracles**: Tests must validate against independent mathematical equations, external surveyed ground truth, or genuine physical simulations. Assertions on internal stub return codes (e.g. `assert_eq!(stub(), 0)`) are strictly forbidden.
3. **Executable Failure Verification**: Every reported defect must be accompanied by an executable regression test that fails against the defective code and passes only when properly remediated.
4. **Platform-Specific Truth**: Packaging tests must verify genuine binary formats (ELF headers for Linux, Mach-O headers for macOS, PE headers for Windows). When cross-compilation toolchains are absent on the build host, the status must be honestly recorded as `BLOCKED (Host Toolchain Required)`.

---

## 7. Documentation & Append-Only History

1. **Append-Only Tracking**: All changes to `ImplementationPlan.md`, `Changelog.md`, and `Todo.md` must be strictly append-only. Historical stages, decisions, and past audit findings must be preserved for forensic accountability.
2. **Architectural Decision Records (ADRs)**: Every significant architectural shift or remediation stage must be documented with a numbered ADR in `docs/adr/`.
3. **Continuous Maintenance**: Maintain up-to-date documentation: `TechnicalRequirements.md`, `TechnicalArchitecture.md`, `GISStrategy.md`, `ValidationStrategy.md`, `UserGuide.md`, `Troubleshooting.md`, `RootCauseAnalysis.md`, `GapAnalysis.md`, and `RemediationPlan.md`.

---

## 8. Execution Directives for Subsequent Stages

For EVERY subsequent development or remediation stage:
1. **Audit Dependencies**: Read this contract, verify actual code behavior, and audit existing defects before modifying code.
2. **Update Tracking**: Append stage goals and tasks to `ImplementationPlan.md`, `Changelog.md`, and `Todo.md` before coding.
3. **Implement Vertical Slices**: Complete functioning, independently testable implementations.
4. **Write Real Tests**: Implement executable tests with independent oracles that verify real-world correctness.
5. **Measure Real Metrics**: Measure performance and memory using real system timers.
6. **No Regressions**: Verify that existing working functionality remains intact.
7. **Honest Reporting**: Update `docs/FeatureQualityMatrix.md` and `docs/FeatureQualityMatrix.json`. If a capability cannot be completed due to missing external prerequisites, report it honestly as `BLOCKED`.

---

## 9. Stage 0.6 Ratification: Geographic Collision, Vehicle Spawning and Real City Switching

1. **Phase 1 — Building Footprint & Road Corridor Repair**: Verified 0 penetrations in San Francisco via [`test_stage0_6_diagnostic_audit_building_road_collisions`](file:///j:/rustRealEarth/tests/tests/stage0_6_geographic_collision_and_spawning_certification.rs).
2. **Phase 2 — Vehicle Garage & Multimodal Spawning**: Verified validated spawning across all 10 multimodal vehicle classes with chase camera synchronization and contract cleansing via [`test_stage0_6_vehicle_garage_selection_and_spawning_all_ten_classes`](file:///j:/rustRealEarth/tests/tests/stage0_6_geographic_collision_and_spawning_certification.rs).
3. **Phase 3 — Real World City Switching**: Verified asynchronous state machine switching across San Francisco, New York, Chicago, and Los Angeles, double-precision ECEF floating-origin re-anchoring, GPU cache purging, and rejection of catalog-only definitions without bundled GIS packages via [`test_stage0_6_real_world_city_switching_sf_nyc_chicago_la`](file:///j:/rustRealEarth/tests/tests/stage0_6_geographic_collision_and_spawning_certification.rs) and [`test_stage0_6_catalog_only_missing_package_rejection`](file:///j:/rustRealEarth/tests/tests/stage0_6_geographic_collision_and_spawning_certification.rs).
4. **Phase 4 — Automated Test Suite & Codebase Quality**: All workspace tests pass (`cargo test --workspace` 100% pass), zero compiler warnings under `cargo clippy --workspace --all-targets -- -D warnings`, and clean formatting under `cargo fmt --check`.

