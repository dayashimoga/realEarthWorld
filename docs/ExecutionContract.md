# GLOBAL ENGINEERING CONTRACT — REAL EARTH WORLD PLATFORM

**Role Definition:** Act as Principal Rust, Game/Simulation, Graphics, GIS, Physics, Distributed Systems, Performance, Platform, QA, Build, Security, UX, and Documentation Engineers.

**Mission:** Build a production-grade, geographically accurate, high-performance Earth-scale world platform. **WORLD FIRST → SIMULATION SECOND → GAMEPLAY THIRD.** Start with a bounded real-world Bengaluru region; expand incrementally. Every stage must deliver functioning, independently testable software with no regression.

---

## 1. Mandatory Stack

- **Core Language & Toolchain:** Rust Stable; `cargo test`, `Criterion`, `cargo-llvm-cov`, `Clippy`, `rustfmt`.
- **Rendering & Simulation:** `wgpu` rendering; `Bevy ECS`; `Rapier3D` physics.
- **Asynchronous Execution & Concurrency:** `Tokio` asynchronous runtime; `Rayon` data parallelism.
- **Data & Serialization:** `FlatBuffers` versioned schemas; `SQLite` indexing/metadata; compressed binary world chunks (`zstd`).
- **Geographic Data Sources:** OpenStreetMap roads/buildings; Copernicus/NASA DEM; licensed land-cover datasets.
- **Networking:** Optional QUIC networking (`realearth-network`).
- **CI/CD & Containers:** GitHub Actions; Podman/Docker development workflows.
- **Dependency Policy:** Pin compatible dependencies and commit lockfiles.

---

## 2. Architecture

Implement isolated layers:

$$\text{Data Acquisition} \longrightarrow \text{GIS Normalization} \longrightarrow \text{Validation/Repair} \longrightarrow \text{World Compiler} \longrightarrow \text{Binary Chunk Storage} \longrightarrow \text{Streaming} \longrightarrow \text{Rendering/Navigation/Simulation} \longrightarrow \text{Optional Gameplay} \longrightarrow \text{Optional Multiplayer}$$

### Strict Architectural Invariants:
1. **Headless Execution:** World compiler and simulation must run headlessly without any window, display server, or GPU display dependency.
2. **Decoupled Graphics:** Rendering must never own simulation, geographic validation, or gameplay logic.
3. **Independent Replaceability:** Physics, graphics, navigation, gameplay, and networking must remain independently replaceable without cross-cutting rewrites.
4. **Optional Features:** Networking and gameplay are optional dependencies.
5. **Deterministic Foundation:** Use explicit APIs, immutable snapshots where appropriate, versioned schemas, deterministic operations, bounded work queues, structured errors, and traceable state transitions.
6. **Extensible Entity Systems:** Support future cars, motorcycles, trucks, buses, trains, ships, and aircraft without architectural rewrites.

---

## 3. Geographic Truth

1. **Provenance & Coordinate Precision:** Use real, licensed, provenance-tracked GIS data. Preserve WGS84 references, appropriate projected coordinates (e.g. UTM Zone 43N for Bengaluru), global ECEF/local ENU transformations, double-precision geographic positions (`f64`), floating origin camera systems, vertical datum metadata (ellipsoid vs EGM96 geoid), and accurate elevation/scale.
2. **Feature Preservation:** Preserve real roads, intersections, turn restrictions, bridges, flyovers, tunnels, buildings, railways, airports, ports, rivers, water bodies, terrain, vegetation zones, highways, and inter-city connections.
3. **No Fabrication Rule:** Never fabricate geographic facts. Procedural appearance, textures, vegetation, and missing-height approximations are permitted only as explicitly labeled visual/inferred data, never falsely represented as surveyed reality.
4. **Topological Invariance:** Validate topology, elevation, footprints, seams, graph connectivity, collision, and provenance before publishing world packages. Treat grade-separated road crossings correctly (e.g., bridges/overpasses must maintain vertical clearance and never collapse into planar junctions). Repair only when safe, traceable, confidence-aware, and reversible; otherwise report/quarantine defects.

---

## 4. Performance

1. **Initial Bengaluru MVP:** First usable frame under 2 seconds from a locally cached compiled package, under 1 GB process RAM, 60 FPS ($16.67\text{ ms/frame}$) on documented reference hardware.
2. **Planetary Scaling Targets:** 100K and 1M managed objects at 60 FPS; 10M managed objects through adaptive streaming and hierarchical LOD. Do not equate managed objects with simultaneously visible/simulated objects.
3. **Telemetry & Profiling:** Measure CPU/GPU utilization, GPU memory, RSS, startup, p50/p95/p99 frame times, streaming stalls, cache performance, draw calls, and geometry complexity.
4. **Budget Management:** Target $16.67\text{ ms/frame}$ for 60 FPS. Optimize based on profiling; avoid blocking I/O, unbounded caches, excess allocation, lock contention, and redundant GPU work.
5. **Reproducibility:** Define hardware, resolution, graphics preset, dataset, release profile, reproducible benchmark scenes, and statistically meaningful regression tolerances.

---

## 5. Repository Structure

Cargo workspace organized into explicit crates:
- `engine/{ingestion,geospatial,validation,compiler,streaming,rendering,simulation,vehicles,navigation,network,shared}`
- `apps/world-viewer`
- `tools`, `tests`, `benchmarks`, `assets`, `data/manifests`, `data/schemas`, `scripts`
- `docs` and `docs/adr`
- `.github/workflows`

Clear crate boundaries with typed interfaces.

---

## 6. CI/CD and Artifacts

- **Seven Workflows:** `build.yml`, `test.yml`, `coverage.yml`, `benchmark.yml`, `security.yml`, `docs.yml`, and `release.yml`.
- **Quality Gates:** Formatting (`cargo fmt --check`), static analysis (`cargo clippy -- -D warnings`), unit/integration/GIS/headless tests, dependency/security checks, documentation validation (`cargo doc`), coverage and controlled performance regression tests.
- **Cross-Platform Target Matrix:** Windows x64, Linux x64, and macOS arm64/x64 binaries. Native runners for GPU/Desktop validation; containerize development and headless tooling whenever feasible.
- **Automated Provenance:** Upload executables, checksums, build provenance, test results, coverage, benchmarks, GIS validation, security, performance reports, and documentation.
- **Script Suite:** Idempotent one-command setup, bringup, teardown, build, test, run, benchmark, cleanup, and teardown scripts with no unnecessary local installs.

---

## 7. Quality Gates

- Core testable code coverage $\ge 90\%$; report line/branch coverage and justified exclusions.
- 100% pass rate for required tests.
- Zero required formatting/lint violations.
- Zero unapproved critical/high security vulnerabilities.
- No undisclosed correctness, FPS, latency, memory, or GPU regressions.
- Unit, integration, property/fuzz, geospatial, rendering, physics, streaming, end-to-end, platform, and soak testing as applicable.
- Real data validation and genuine executable smoke tests.
- No fake success, placeholders, silent skips, or simulated evidence presented as production validation.
- Honestly separate PASS, FAIL, BLOCKED, NOT TESTED, and HARDWARE-REQUIRED status.

---

## 8. Documentation

Maintain:
`TechnicalRequirements.md`, `TechnicalArchitecture.md`, `ImplementationPlan.md`, `CodeUnderstanding.md`, `GISStrategy.md`, `ValidationStrategy.md`, `WorldStreamingStrategy.md`, `PerformanceStrategy.md`, `TestingStrategy.md`, `DataSourcesAndLicenses.md`, `SetupAndConfiguration.md`, `UserGuide.md`, `Troubleshooting.md`, `Changelog.md`, `Todo.md`, plus ADRs and ASCII/Mermaid architecture diagrams.

`ImplementationPlan`, `Changelog`, and `Todo` must preserve history append-only. Document requirements-to-code-to-test traceability, actual status, decisions, limitations, and configuration.

---

## 9. Execution Rules

For EACH stage:
1. Audit the existing repository, completed features, dependencies, known defects, and regression risks.
2. Update requirements, ADRs, implementation plan, and append-only TODO before coding.
3. Implement phases sequentially, completing actual executable vertical slices.
4. Write meaningful tests and benchmarks; ensure independent module boundaries.
5. Verify actual behavior using real data; measure performance and memory.
6. Preserve all existing working features; fix defects before adding dependent functionality.
7. Re-run relevant and full regression suites.
8. Update documentation, changelog, coverage, benchmarks, and evidence.
9. Generate a certification matrix: requirement, source, test, environment, dataset, expected/actual, evidence, result, limitations.
10. Finish with completed, failed, blocked, pending, performance results, downloadable CI artifacts, and next-stage readiness.

**Never claim implementation, execution, or production certification without verifiable evidence. Never move to the next stage with unresolved blocking acceptance criteria.**

**Priorities:** Geographic Accuracy → Data Integrity → Functional Correctness → Stability → Memory → Streaming → FPS → Visual Fidelity → Features → Multiplayer → Planetary Scale.
