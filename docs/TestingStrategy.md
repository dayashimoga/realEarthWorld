# Real Earth World Platform — Testing & Verification Strategy

## 1. Quality Gates & Acceptance Hierarchy

In strict compliance with Section 7 of the contract:
- **Core Testable Code Coverage**: $\ge 90\%$.
- **100% Pass Rate**: Zero tolerated test failures.
- **Zero Clippy Warnings**: Strict linting across the entire workspace (`cargo clippy --workspace --all-targets -- -D warnings`).
- **Formatting**: Zero formatting drift (`cargo fmt --check`).
- **Honest Status Reporting**: Test runs must clearly report `PASS`, `FAIL`, `BLOCKED`, `NOT TESTED`, and `HARDWARE-REQUIRED` status honestly. No simulated or mocked evidence presented as production validation.

---

## 2. Test Classification & Suites

### 2.1 Unit Tests (`cargo test --workspace --lib`)
- Mathematical accuracy: Geodetic $\leftrightarrow$ ECEF $\leftrightarrow$ ENU roundtrip precision tests with $10^{-6}\text{ m}$ tolerance.
- Floating origin translation invariance.
- Polygon triangulation and winding order correction.
- Binary chunk serialization and deserialization integrity.

### 2.2 Geospatial & Topological Validation Tests
- Grade separation invariant verification: Ensures overpasses and bridges never create spurious junction nodes with underlying ground roads.
- Road network connectivity: Verifies that primary roads form connected components without orphaned edges.
- Building footprint validity: Asserts that non-convex and polygon-with-holes shapes triangulate with zero inverted normals.

### 2.3 Headless Simulation & Determinism Tests
- Deterministic Bevy ECS execution: Given identical seed and initial state, step the simulation 1,000 frames and assert byte-identical state hashes.
- Solar calculation verification: Check solar elevation angle at Bengaluru coordinates for known solar noon and midnight timestamps.

### 2.4 Headless Rendering & Shader Tests
- wgpu headless off-screen rendering: Create an offscreen texture buffer, render a test chunk (terrain, road, building), and verify depth buffer and color buffer outputs.

### 2.5 Microbenchmarks & Performance Regression Tests
- Criterion benchmarks in `benchmarks/` tracking throughput, latency, and memory allocation.

---

## 3. Behavioral vs Internal Assumption Testing Mandate (Phase 0 Audit)

Following the Phase 0 forensic audit, the following testing anti-patterns are explicitly prohibited:

1. **Anti-Pattern: Stub Return Code Assertion**:
   - *Prohibited*: Asserting `assert_eq!(stub_fn(), 0)` when the function is a no-op that always returns `0`.
   - *Requirement*: Tests must assert on actual observable side effects (e.g. allocated vertex buffers, initialized GPU swapchains, real physics velocity changes).
2. **Anti-Pattern: Self-Validating Synthetic Generation**:
   - *Prohibited*: Asserting `dataset.buildings.len() > 0` on a generator that manufactures synthetic bounding boxes when authentic raw GIS files are missing.
   - *Requirement*: Validate physical disk existence of authentic raw OSM PBF/XML and GeoTIFF elevation files.
3. **Anti-Pattern: Conflating Offscreen Pipeline Throughput with Display Framerate**:
   - *Prohibited*: Reporting unthrottled headless render loop throughput (>3000 FPS) as proof of interactive 60 FPS gameplay.
   - *Requirement*: Isolate raw GPU pipeline latency from presented windowed swapchain pacing.

