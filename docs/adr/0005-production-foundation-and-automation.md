# ADR 0005: Production Foundation, Automation, and Schema Governance

## Status
Accepted

## Context
Stage 0 requires establishing a production-grade substrate for the Real Earth World Platform before initiating domain-specific GIS rendering and gameplay layers. The platform must guarantee:
1. Complete headless determinism and reproducibility across developer workstations (Windows, Linux, macOS) and containerized CI environments.
2. Pinned dependencies to eliminate supply chain drift.
3. Centralized error handling and tracing with zero stdout/stderr noise.
4. Binary and network schema evolution without breaking changes.
5. Automated quality gates enforcing strict linting, formatting, testing, and performance regression standards.

## Decisions

### 1. Cargo Workspace Dependency Inheritance
All workspace crates (`engine/*`, `tools/*`, `apps/*`, `benchmarks`, `tests`) inherit common dependencies (`glam`, `tokio`, `rayon`, `serde`, `tracing`, `wgpu`, `bevy_ecs`, `rapier3d`) from `workspace.dependencies` in the root `Cargo.toml`. Version pinning is strict and committed via `Cargo.lock`.

### 2. Tracing and Structured Telemetry
All engine crates utilize `tracing` (`trace!`, `debug!`, `info!`, `warn!`, `error!`). Direct `println!` or `eprintln!` calls are forbidden in engine code and restricted solely to interactive CLI entry points. Subscribers are configured via `tracing-subscriber` with `EnvFilter` support.

### 3. FlatBuffers Versioned Schema Governance
Spatial chunk definitions and network messages use FlatBuffers (`data/schemas/*.fbs`) with backward/forward binary compatibility:
- All fields have explicit default values or are optional tables.
- Structs are used only for fixed mathematical primitives (`Vec3f`, `GeoBounds`, `Wgs84Coord`).
- Schema changes must follow append-only semantics.

### 4. Container Workflows (Podman & Docker)
To decouple engine compilation and headless verification from host OS graphics stacks:
- Multi-stage `Dockerfile` and `Containerfile` are provided for headless world compilation, validation, and CLI tool execution.
- `compose.yml` provides a one-command container bringup/teardown workflow for continuous headless execution.

### 5. Automated CI/CD Seven-Workflow Architecture
GitHub Actions workflows enforce continuous validation across 7 dedicated jobs:
- `build.yml`: Cross-platform compilation (Windows MSVC, Linux GNU, macOS Universal).
- `test.yml`: Workspace tests, integration tests, and headless smoke tests.
- `coverage.yml`: LLVM source-based code coverage target ($\ge 90\%$).
- `benchmark.yml`: Criterion performance regression testing.
- `security.yml`: Dependency vulnerability scanning via `cargo-audit`.
- `docs.yml`: Crate documentation verification and publishing.
- `release.yml`: Release artifact packaging with SHA256 checksums and provenance.

### 6. Script Idempotency
All scripts in `scripts/` (`setup`, `bringup`, `teardown`, `build`, `test`, `run`, `bench`, `clean`) are written in dual-dialect (PowerShell for Windows, Bash for POSIX) and are guaranteed idempotent: running them multiple times produces identical, side-effect-free results.

## Consequences
- Engine crates compile headlessly on standard Linux container environments without requiring X11, Wayland, or physical GPU hardware.
- Developers and CI pipelines run identical quality gate checks.
- Binary formats and coordinate conventions remain versioned, traceable, and protected from ad-hoc schema modifications.
