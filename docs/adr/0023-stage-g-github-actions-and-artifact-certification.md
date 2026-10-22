# ADR-0023: Stage G — GitHub Actions & Artifact Certification

## Status
Accepted

## Date
2026-10-22

## Context
Following the completion of Stage F (Five-Platform Application Completeness), Stage G establishes a production-grade continuous integration, continuous delivery (CI/CD), and forensic artifact verification pipeline across all supported operating systems.

To eliminate false-green builds, unverified binary claims, and supply-chain drift, Stage G establishes:
1. **G1 — Workflow Matrix**: Implementation of 8 dedicated GitHub Actions workflows (`audit.yml`, `build.yml`, `test.yml`, `coverage.yml`, `benchmark.yml`, `security.yml`, `docs.yml`, `release.yml`) targeting Windows (`windows-latest`), Linux (`ubuntu-latest`), and macOS (`macos-latest`) runners with Android SDK/NDK and Apple Xcode environments.
2. **G2 — CI Quality Gates**: Strict gate enforcement requiring `cargo fmt --check`, zero Clippy warnings (`-D warnings`), 100% test pass rate across 16 test suites, >=90% core code coverage, zero security CVEs, and valid GIS manifests.
3. **G3 — Automated Platform Testing**: Automated smoke test verification across all supported desktop and mobile form factors, validating CLI geospatial compilation, headless renderer initialization, and JNI/UIKit bindings.
4. **G4 — Downloadable Versioned Artifacts**: Packaging and checksum generation for all 9 platform deliverables in `dist/`.
5. **G5 — Release Integrity & Anti-Tampering**: Verification of archive magic headers (`PK\x03\x04`, `\x1f\x8b`), cryptographic SHA-256 matching, strict fail-safe gates (`if-no-files-found: error`), and transparent disclosure of code signing status (explicitly distinguishing unsigned/ad-hoc builds from production signed App Store/Google Play releases).

## Decisions

### 1. Dedicated Workflow Isolation
Rather than bundling all checks into a monolithic CI job, each operational domain is cleanly isolated into a dedicated workflow:
- `audit.yml`: Code formatting, Clippy warnings, GIS dataset integrity, and crate license compliance.
- `build.yml`: Cross-platform matrix build across Windows, Linux, macOS, Android (ARM64), and iOS (ARM64).
- `test.yml`: 3-runner OS matrix executing unit tests, integration suites, and CLI/viewer smoke tests.
- `coverage.yml`: LLVM-cov coverage generation producing LCOV and HTML reports with threshold gates.
- `benchmark.yml`: Performance throughput regressions and Criterion execution.
- `security.yml`: RustSec Advisory Database dependency vulnerability scans.
- `docs.yml`: Strict Rustdoc compilation with zero warnings and API documentation publishing.
- `release.yml`: Multi-stage packaging, release integrity verification, and GitHub Release publication.

### 2. Strict Fail-Safe Upload Policy
All artifact upload steps across `build.yml`, `coverage.yml`, `docs.yml`, and `release.yml` specify:
```yaml
if-no-files-found: error
```
This guarantees that missing or misnamed packages immediately fail the build rather than producing silent false-green CI runs.

### 3. Transparent Code Signing Disclosures
In accordance with honest software engineering principles, release packages and manifests must never misrepresent unsigned builds:
- Windows: Portable ZIP container containing unsigned PE32+ binaries for developer and portable evaluation.
- Linux: Standalone AppImage and compressed tarball without proprietary GPG wrapping.
- macOS: Ad-hoc signed DMG and universal tarball; notarization explicitly documented as requiring Apple Developer credentials.
- Android: Self-signed debug keystore APK and AAB for emulator/developer deployment.
- iOS: Unsigned device IPA container; physical device sideloading transparently documented as requiring an active Apple Developer Team Provisioning Profile. Unsigned Simulator `.zip` bundle provided for zero-credential Xcode Simulator execution.

### 4. Release Integrity Audit Automation
The scripts [`scripts/verify_release_integrity.ps1`](file:///j:/rustRealEarth/scripts/verify_release_integrity.ps1) and [`scripts/verify_release_integrity.sh`](file:///j:/rustRealEarth/scripts/verify_release_integrity.sh) programmatically verify every package in `dist/`, enforce archive magic numbers, recompute SHA-256 digests, and generate [`dist/release_manifest.json`](file:///j:/rustRealEarth/dist/release_manifest.json).

## Consequences & Verification
- 8 dedicated GitHub Actions workflows active in `.github/workflows/`.
- 16 integration test suites passing in `tests/tests/`, including the dedicated [`release_integrity_certification.rs`](file:///j:/rustRealEarth/tests/tests/release_integrity_certification.rs).
- All 9 platform distribution archives certified with exact SHA-256 matching.
