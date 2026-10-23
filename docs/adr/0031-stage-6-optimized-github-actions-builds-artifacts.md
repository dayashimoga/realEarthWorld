# ADR 0031: Stage 6 — Optimize GitHub Actions, Builds and Artifacts

## Status
Accepted

## Context
Prior iterations of the CI/CD and release packaging pipeline had several architectural inefficiencies and maintenance liabilities:
1. **Redundant & Fragmented Workflows**: Seven separate per-commit workflows (`audit.yml`, `benchmark.yml`, `build.yml`, `coverage.yml`, `docs.yml`, `security.yml`, `test.yml`) caused duplicate dependency fetching, repeated complete workspace compilation, and high runner resource consumption.
2. **Missing Aggregate Gate**: Branch protection rules required configuring 8+ individual matrix jobs, making it easy for skipped or silently failed jobs to escape notice without a single authoritative status check.
3. **Severe Artifact Storage Bloat**: Previous `test.yml` had configured `path: target/` in artifact upload steps, attempting to upload 10+ GB of compiled intermediate object code on every push, threatening GitHub storage limits and slowing down pipelines.
4. **ZIP-inside-ZIP Packaging Inconvenience**: Pointing `actions/upload-artifact@v4` at an existing archive (`dist/windows/realearth-windows-x64.zip`) resulted in GitHub creating an outer ZIP containing an inner ZIP, creating friction for users and automated smoke tests.
5. **Coupled Geographic Data & Binaries**: Releases lacked clean separation between application binaries and independently installable real geographic regional world packages.

## Decisions

### 1. Workflow Consolidation into Three High-Performance Pipelines
We replaced the fragmented per-commit workflows with three purpose-built, dedicated workflow files:
- **`.github/workflows/ci.yml`** (Per-Commit & PR):
  - Concurrency group with automatic cancellation of superseded pull request runs (`cancel-in-progress: ${{ github.event_name == 'pull_request' }}`).
  - `quick-gates`: Fast-fail gate checking `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo audit --deny warnings`, workspace crate license compliance, GIS sample dataset integrity, and strict rustdoc (`-D warnings`).
  - `test-and-coverage`: Complete workspace unit and integration test pass, 38-city GIS catalog validation, defect remediation regression, adversarial fuzzing, visual fidelity, performance/anti-fake certification, and `cargo-llvm-cov` core engine coverage ($\ge 90\%$).
  - `desktop-build`: Three-platform desktop matrix (`windows-latest`, `ubuntu-latest`, `macos-latest`), building release binaries, testing CLI `--help`, packaging flattened distributions, and uploading artifacts.
  - `mobile-targets`: Android NDK r26b cross-compilation and iOS targets on `macos-latest` runner, packaging Android JNI libraries and iOS application framework.
  - `package-verification`: Downloads platform artifacts, packages standalone region bundle, executes `verify_release_integrity.sh`, and executes CLI compile and viewer headless startup smoke tests.
  - `ci-success`: **One aggregate required status check** that evaluates the completion status of all preceding stages and fails if any stage failed, was skipped, or cancelled.
- **`.github/workflows/nightly.yml`** (Daily at 02:00 UTC & Manual):
  - Scheduled deep regression suite: 38-city US catalog validation, adversarial fuzzing, long-running soak traversal zero-memory-leak checks, 10M-object scale simulation, vehicle physics invariants, multi-transport simulation regression, and Criterion performance baselines.
  - Bounded 30-day retention on concise benchmark metrics.
- **`.github/workflows/release.yml`** (Tagged Releases `v*` & Manual Dispatch):
  - Gated production packaging for Windows, Linux, macOS, Android, iOS, and standalone geographic region packages.
  - Rigorous release integrity verification via `scripts/verify_release_integrity.sh`.
  - Direct attachment of ready-to-use artifacts to GitHub Releases using `softprops/action-gh-release@v2`.

### 2. Reusable Composite Action & Lockfile-Aware Caching
- Created `.github/actions/setup-rust/action.yml` encapsulating:
  - `dtolnay/rust-toolchain@stable` with configurable components (`rustfmt`, `clippy`, `llvm-tools-preview`) and target architectures.
  - `Swatinem/rust-cache@v2` with scoped cache keys based on `Cargo.lock` hashing, eliminating redundant dependency downloads and workspace recompilations.

### 3. Artifact Simplicity & Zero "ZIP-Inside-ZIP"
- Actions artifact uploads target flattened package directories (`path: dist/windows/`, `path: dist/linux/`, etc.) so GitHub produces only the single outer archive.
- GitHub Release publishing targets pre-packaged native archives (`.zip`, `.tar.gz`) directly, providing single-download release assets with no nested ZIPs.
- Authored `scripts/package_regions.ps1` and `scripts/package_regions.sh` to package geographic regions (`data/world_san_francisco/`) into standalone `dist/regions/realearth-region-san-francisco.zip` and `.tar.gz` with installation guides and checksums, completely decoupled from binary executables.

### 4. Cost Reduction & Diagnostic Size Budgets
- Completely eliminated `target/` directory uploads.
- Diagnostic uploads are strictly limited to concise reports: `lcov.info`, `target/llvm-cov/html/`, `target/performance_certification_report.json`, `dist/release_manifest.json`, and `dist/SHA256SUMS`.
- Enforced artifact retention policies:
  - 7-day retention for ephemeral intermediate build packages.
  - 14-day retention for pull request test coverage and verification reports.
  - 30-day retention for nightly benchmark baselines and certified production manifests.

### 5. Enforced Quality Gates & Honest Target Blocker Reporting
- Enforced strict binary magic verification (`PK\x03\x04` for ZIP, `\x1f\x8b` for GZIP/TAR.GZ, `\x7fELF` for Linux binaries) in `scripts/verify_release_integrity.ps1` and `.sh`.
- Prohibited fake or renamed placeholder packages (e.g. ZIPs renamed to `.apk` or `.AppImage`). Platform blockers (such as missing Apple Developer credentials or Android NDK on non-mobile hosts) are documented honestly in `dist/release_manifest.json` and build status JSONs.
- Verified test suite `tests/tests/release_integrity_certification.rs` (5/5 tests passing) validating workflow structure, quality gates, smoke tests, and release manifests.

## Consequences
- GitHub Actions execution time and bandwidth costs are minimized through lockfile-aware caching and targeted artifact uploads.
- Branch protection configuration is simplified to a single required status check (`ci-success`).
- Users receive clean, direct single-download packages and standalone geographic region packages with verifiable SHA-256 checksums.
