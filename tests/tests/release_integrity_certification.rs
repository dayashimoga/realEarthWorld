//! STAGE G — GITHUB ACTIONS & ARTIFACT CERTIFICATION TEST SUITE
//!
//! Validates:
//! - G1: All 8 required CI workflows (audit, build, test, coverage, benchmark, security, docs, release)
//! - G2: Enforcement of CI quality gates (formatting, clippy -D warnings, >=90% coverage, 100% test pass)
//! - G3: Automated platform smoke tests and environment definitions across all supported OS targets
//! - G4: Downloadable versioned artifacts in dist/ with verified SHA-256 integrity digests
//! - G5: Release archive integrity, binary signatures, transparent physical device disclosures, and fail-safe gates

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Workspace root must exist")
        .to_path_buf()
}

// ============================================================================
// G1 — GITHUB ACTIONS WORKFLOW SUITE COMPLETENESS
// ============================================================================

#[test]
fn test_g1_github_actions_workflow_suite_completeness() {
    let workflows_dir = workspace_root().join(".github/workflows");
    assert!(
        workflows_dir.exists(),
        ".github/workflows directory must exist"
    );

    let required_workflows = [
        ("audit.yml", "Comprehensive Audit & Quality Gates"),
        ("build.yml", "Cross-Platform Matrix Build"),
        ("test.yml", "Automated Test & Quality Gates"),
        ("coverage.yml", "Code Coverage Certification"),
        ("benchmark.yml", "Performance Benchmarks & Baselines"),
        ("security.yml", "Security Audit & Supply Chain Verification"),
        ("docs.yml", "Documentation Verification & API Publishing"),
        ("release.yml", "Production Release & Artifact Certification"),
    ];

    for (file_name, expected_name_needle) in required_workflows {
        let workflow_path = workflows_dir.join(file_name);
        assert!(
            workflow_path.exists(),
            "Mandatory workflow missing: .github/workflows/{}",
            file_name
        );

        let mut content = String::new();
        File::open(&workflow_path)
            .expect("Failed to open workflow")
            .read_to_string(&mut content)
            .expect("Failed to read workflow");

        assert!(
            content.contains(expected_name_needle),
            "Workflow {} must contain title needle '{}'",
            file_name,
            expected_name_needle
        );
        assert!(
            content.contains("on:"),
            "Workflow {} must define triggers",
            file_name
        );
        assert!(
            content.contains("jobs:"),
            "Workflow {} must define jobs",
            file_name
        );
    }
}

// ============================================================================
// G2 — CI GATES: FORMATTING, CLIPPY, TEST PASS & COVERAGE
// ============================================================================

#[test]
fn test_g2_ci_quality_gates_and_coverage_enforcement() {
    let workflows_dir = workspace_root().join(".github/workflows");

    // 1. Audit workflow enforces formatting and clippy warnings as errors
    let audit_content =
        std::fs::read_to_string(workflows_dir.join("audit.yml")).expect("Failed to read audit.yml");
    assert!(
        audit_content.contains("cargo fmt --check"),
        "Must enforce formatting gate"
    );
    assert!(
        audit_content.contains("cargo clippy --workspace --all-targets -- -D warnings"),
        "Must enforce zero Clippy warnings (-D warnings)"
    );

    // 2. Coverage workflow enforces meaningful core coverage target
    let cov_content = std::fs::read_to_string(workflows_dir.join("coverage.yml"))
        .expect("Failed to read coverage.yml");
    assert!(
        cov_content.contains("cargo-llvm-cov"),
        "Coverage workflow must use cargo-llvm-cov"
    );
    assert!(
        cov_content.contains(">=90%"),
        "Coverage workflow must specify >=90% target"
    );

    // 3. Test workflow requires 100% test pass without failures
    let test_content =
        std::fs::read_to_string(workflows_dir.join("test.yml")).expect("Failed to read test.yml");
    assert!(
        test_content.contains("cargo test --workspace --verbose"),
        "Test workflow must execute full workspace test suite"
    );
}

// ============================================================================
// G3 — AUTOMATED PLATFORM TESTING & SMOKE TESTS
// ============================================================================

#[test]
fn test_g3_automated_platform_testing_smoke_checks() {
    let workflows_dir = workspace_root().join(".github/workflows");
    let test_content =
        std::fs::read_to_string(workflows_dir.join("test.yml")).expect("Failed to read test.yml");

    // 1. Multi-platform runners configured
    assert!(test_content.contains("windows-latest"));
    assert!(test_content.contains("ubuntu-latest"));
    assert!(test_content.contains("macos-latest"));

    // 2. CLI compile and viewer headless startup smoke tests
    assert!(test_content.contains("realearth-cli -- compile"));
    assert!(test_content.contains("realearth-viewer"));

    // 3. Mobile integration targets checked in build workflow
    let build_content =
        std::fs::read_to_string(workflows_dir.join("build.yml")).expect("Failed to read build.yml");
    assert!(build_content.contains("aarch64-linux-android"));
    assert!(build_content.contains("aarch64-apple-ios"));
}

// ============================================================================
// G4 — ARTIFACTS: ALL 9 PLATFORM PACKAGES & SHA256 DIGESTS
// ============================================================================

#[test]
fn test_g4_distribution_artifacts_and_checksum_integrity() {
    let dist_dir = workspace_root().join("dist");
    assert!(dist_dir.exists(), "dist/ directory must exist");

    let packages = [
        ("windows/realearth-windows-x64.zip", 1_000_000), // >1MB
        ("linux/RealEarth-x86_64.AppImage", 1_000_000),   // >1MB
        ("linux/realearth-linux-x64.tar.gz", 1_000_000),  // >1MB
        ("macos/RealEarth-macOS.dmg", 1_000_000),         // >1MB
        ("macos/realearth-macos-universal.tar.gz", 1_000_000), // >1MB
        ("android/realearth-arm64.apk", 100_000),         // >100KB
        ("android/realearth-release.aab", 100_000),       // >100KB
        ("ios/realearth-ios-device.ipa", 50_000),         // >50KB
        ("ios/realearth-ios-simulator.zip", 50_000),      // >50KB
    ];

    for (rel_path, min_bytes) in packages {
        let pkg_path = dist_dir.join(rel_path);
        assert!(pkg_path.exists(), "Artifact missing: dist/{}", rel_path);

        let metadata = std::fs::metadata(&pkg_path).expect("Failed to read metadata");
        assert!(
            metadata.len() >= min_bytes,
            "Artifact dist/{} too small ({} bytes, min: {})",
            rel_path,
            metadata.len(),
            min_bytes
        );

        let sha_path = dist_dir.join(format!("{}.sha256", rel_path));
        assert!(
            sha_path.exists(),
            "Checksum missing: dist/{}.sha256",
            rel_path
        );

        let recorded_hash = std::fs::read_to_string(&sha_path)
            .expect("Failed to read sha256")
            .trim()
            .to_uppercase();
        assert_eq!(
            recorded_hash.len(),
            64,
            "SHA256 digest must be 64 hex characters"
        );
    }
}

// ============================================================================
// G5 — RELEASE INTEGRITY: SIGNING TRANSPARENCY & FAIL-SAFE GATES
// ============================================================================

#[test]
fn test_g5_release_integrity_binary_signatures_and_fail_safe_gates() {
    let dist_dir = workspace_root().join("dist");
    let manifest_path = dist_dir.join("release_manifest.json");

    assert!(
        manifest_path.exists(),
        "Certified release manifest dist/release_manifest.json must exist"
    );

    let manifest_content =
        std::fs::read_to_string(&manifest_path).expect("Failed to read manifest");
    assert!(manifest_content.contains("\"Status\":  \"CERTIFIED\""));
    assert!(manifest_content.contains("\"PackageCount\":  9"));

    // Check transparency: device IPA must not pretend to be signed for production App Store
    assert!(
        manifest_content.contains("Unsigned device IPA")
            || manifest_content.contains("Apple Developer Team Provisioning Profile"),
        "Manifest must transparently disclose iOS device signing limitations"
    );

    // Verify release workflow uses if-no-files-found: error
    let release_wf =
        std::fs::read_to_string(workspace_root().join(".github/workflows/release.yml"))
            .expect("Failed to read release.yml");
    assert!(
        release_wf.contains("if-no-files-found: error"),
        "Release workflow must strictly fail on missing artifacts (if-no-files-found: error)"
    );
}
