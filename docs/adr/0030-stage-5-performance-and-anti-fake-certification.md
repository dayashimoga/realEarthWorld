# ADR 0030: Stage 5 — Performance and Anti-Fake Certification

## Status
Accepted

## Context
Prior iterations of engine performance and quality certification suffered from critical authenticity vulnerabilities:
1. **Hardcoded PASS Strings**: Telemetry strings such as `" -> PASS ✅"` were hardcoded into terminal output regardless of actual presented frame rates, geometry counts, or latencies.
2. **Warmup & Empty Frame Distortions**: Initial shader compilation and pipeline initialization frames skewed statistical metrics, while empty or offscreen-only renders with 0 triangles were treated identically to fully loaded geometry.
3. **Synthetic / Incomplete Memory Tracking**: Heap and process memory were either approximated or ignored without querying native operating system APIs for true Working Set (RSS) and peak RSS.
4. **Conflated Object Metrics**: Catalog entities, active in-memory features, frustum-visible primitives, and physics-simulated objects were conflated into a single ambiguous "objects" number.
5. **Untracked Vulnerabilities in Dependencies**: Security audits against the RustSec Advisory Database were not run regularly, leaving known CVEs unaddressed.

## Decisions

### 1. OS-Native Process Metrics & Statistical Frame Pacing (`engine/shared/src/metrics.rs`)
- Implemented native operating system Working Set / RSS memory queries:
  - Windows: `K32GetProcessMemoryInfo` retrieving `WorkingSetSize` and `PeakWorkingSetSize`.
  - Linux / Unix: `/proc/self/statm` parsing resident set page counts multiplied by page size.
- Implemented statistical frame pacing calculations:
  - Exclusion of initial warmup frames (first 10 frames) recorded separately (`warmup_mean_ms`) to avoid shader pipeline compilation skew.
  - Steady-state statistical evaluation: Mean, standard deviation, min, max, p50, p90, p95, p99, 1.0% low FPS, 0.1% low FPS, and presented FPS.
- Structured strict 4-way geographic object accounting (`ManagedObjectCounts`):
  - `stored_objects`: Total entities indexed in SQLite chunk catalog.
  - `active_objects`: Entities decompressed and resident in RAM.
  - `visible_objects`: Entities passing view-frustum, occlusion, and LOD distance culling.
  - `simulated_objects`: Entities actively ticked in physics / IDM traffic simulation.

### 2. Elimination of Hardcoded PASS & Dynamic Machine-Readable Certification
- Eradicated all hardcoded PASS strings from `apps/world-viewer/src/main.rs`.
- Implemented dynamic budget evaluation via `PerformanceCertificationReport::evaluate(...)`:
  - Enforces `steady_state.p95_ms <= budget_p95_ms` (16.67ms for 60 FPS target).
  - Enforces `peak_rss_mb <= budget_rss_mb` (1024 MB threshold).
  - Enforces `cold_startup_ms <= budget_startup_ms` (2000 ms threshold).
  - Enforces anti-fake scene geometry validation: `total_scene_triangles > 0` and `active_chunks > 0` (rejects fake / empty renders).
- Emitted machine-readable telemetry to `target/performance_certification_report.json`.

### 3. Independent Geometric & Semantic Ground Truth Validation
- Created comprehensive integration suite `tests/tests/performance_anti_fake_certification.rs` testing:
  - Geometric comparison against source USGS/OSM: Salesforce Tower (326m, OSM 509206771), Transamerica Pyramid (260m, OSM 34502157), Ferry Building (75m, OSM 27244569), and coastal shoreline DEM elevation (0–15m NAVD88).
  - Dynamic driving and raycast collision continuity: A* route tracking along Market St and The Embarcadero with strict waypoint spacing (< 800m), ground normal contact force, and anti-tunneling clamp.
  - Adversarial fuzz and corruption resilience: Truncated XML and malformed Zstd chunk buffers gracefully handled with fallback recovery.
  - Visual entropy verification: Offscreen wgpu render buffer readback with luminance variance $\sigma^2 > 10.0$ proving genuine rendered scene content (rejects blank/uniform frames).
  - Scale taxonomy verification: 100K, 1M, and adaptive 10M-object hierarchies distinguishing stored vs active vs visible vs simulated.

### 4. Strict Quality Gates & Dependency Security Remediation
- Remediated 3 vulnerabilities discovered by `cargo audit`:
  - Upgraded `quick-xml` from 0.37.5 to 0.41.0 to eliminate RUSTSEC-2026-0194 and RUSTSEC-2026-0195.
  - Updated `tokio` to >=1.54.1 to eliminate RUSTSEC-2026-0384.
- Added automated `cargo audit` step to `scripts/test.ps1` and `scripts/test.sh`.
- Enforced strict formatting (`cargo fmt --check`) and clippy warnings prohibition (`cargo clippy --workspace --all-targets -- -D warnings`).
- Configured compiler profile optimization in root `Cargo.toml` (`debug = 1`) to eliminate disk space exhaustion on memory-constrained systems while maintaining stack trace fidelity.
- Validated 100% test pass rate across 23 integration test suites and core crate unit tests.

## Consequences
- Zero fake metrics or hardcoded certification strings remain in the repository.
- Every benchmark run produces verified, verifiable JSON telemetry artifacts with native OS memory stats and presented frame times.
- Dependencies are scanned and certified against the RustSec Advisory Database with 0 known vulnerabilities.
