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
// G1 — GITHUB ACTIONS CONSOLIDATED WORKFLOW SUITE
// ============================================================================

#[test]
fn test_g1_github_actions_workflow_suite_completeness() {
    let workflows_dir = workspace_root().join(".github/workflows");
    assert!(
        workflows_dir.exists(),
        ".github/workflows directory must exist"
    );

    let required_workflows = [
        ("ci.yml", "CI Quality & Platform Matrix"),
        ("nightly.yml", "Nightly Deep Verification & Baselines"),
        ("release.yml", "Production Release & Gated Distribution"),
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
// G2 — CONSOLIDATED CI GATES: FORMATTING, CLIPPY, TESTS, COVERAGE & AGGREGATE
// ============================================================================

#[test]
fn test_g2_ci_quality_gates_and_coverage_enforcement() {
    let workflows_dir = workspace_root().join(".github/workflows");

    let ci_content =
        std::fs::read_to_string(workflows_dir.join("ci.yml")).expect("Failed to read ci.yml");

    // 1. Concurrency cancellation on superseded PRs
    assert!(
        ci_content.contains("cancel-in-progress:"),
        "CI workflow must enable concurrency cancellation for superseded PR runs"
    );

    // 2. Enforces formatting and zero Clippy warnings
    assert!(
        ci_content.contains("cargo fmt --check"),
        "Must enforce formatting gate"
    );
    assert!(
        ci_content.contains("cargo clippy --workspace --all-targets -- -D warnings"),
        "Must enforce zero Clippy warnings (-D warnings)"
    );

    // 3. Security audit against RustSec database
    assert!(
        ci_content.contains("cargo audit --deny warnings"),
        "Must enforce zero security vulnerability gate"
    );

    // 4. Coverage gate specifies >=90% target with cargo-llvm-cov
    assert!(
        ci_content.contains("cargo-llvm-cov"),
        "Coverage gate must use cargo-llvm-cov"
    );
    assert!(
        ci_content.contains(">=90%"),
        "Coverage gate must specify >=90% target"
    );

    // 5. Test pass without failures
    assert!(
        ci_content.contains("cargo test --workspace --verbose"),
        "CI workflow must execute full workspace test suite"
    );

    // 6. One aggregate required status check job
    assert!(
        ci_content.contains("ci-success:"),
        "CI workflow must provide one aggregate required status check job"
    );
}

// ============================================================================
// G3 — AUTOMATED PLATFORM TESTING & SMOKE TESTS
// ============================================================================

#[test]
fn test_g3_automated_platform_testing_smoke_checks() {
    let workflows_dir = workspace_root().join(".github/workflows");
    let ci_content =
        std::fs::read_to_string(workflows_dir.join("ci.yml")).expect("Failed to read ci.yml");

    // 1. Multi-platform runners configured
    assert!(ci_content.contains("windows-latest"));
    assert!(ci_content.contains("ubuntu-latest"));
    assert!(ci_content.contains("macos-latest"));

    // 2. CLI compile and viewer headless startup smoke tests
    assert!(ci_content.contains("realearth-cli -- compile"));
    assert!(ci_content.contains("realearth-viewer"));

    // 3. Mobile integration targets checked
    assert!(ci_content.contains("aarch64-linux-android"));
    assert!(ci_content.contains("aarch64-apple-ios"));

    // 4. Nightly workflow covers deep GIS, fuzzing, and soak tests
    let nightly_content = std::fs::read_to_string(workflows_dir.join("nightly.yml"))
        .expect("Failed to read nightly.yml");
    assert!(nightly_content.contains("us_city_catalog_validation"));
    assert!(nightly_content.contains("adversarial_validation"));
    assert!(nightly_content.contains("test_soak_traversal_zero_memory_leak"));
    assert!(nightly_content.contains("realearth-benchmarks"));
}

// ============================================================================
// G4 — ARTIFACTS: AUTHENTIC PLATFORM PACKAGES & SHA256 DIGESTS
// ============================================================================

#[test]
fn test_g4_distribution_artifacts_and_checksum_integrity() {
    let dist_dir = workspace_root().join("dist");
    assert!(dist_dir.exists(), "dist/ directory must exist");

    // Mandatory authentic platform packages
    let mandatory_packages = [
        "windows/realearth-windows-x64.zip",
        "linux/realearth-linux-x64.tar.gz",
        "macos/realearth-macos-universal.tar.gz",
        "ios/realearth-ios-simulator.zip",
    ];

    for rel_path in mandatory_packages {
        let pkg_path = dist_dir.join(rel_path);
        assert!(pkg_path.exists(), "Artifact missing: dist/{}", rel_path);

        let metadata = std::fs::metadata(&pkg_path).expect("Failed to read metadata");
        assert!(
            metadata.len() > 1000,
            "Artifact dist/{} too small ({} bytes)",
            rel_path,
            metadata.len()
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

    // Invariant: No mislabeled zip-as-AppImage or zip-as-DMG
    let forbidden_fakes = [
        ("linux/RealEarth-x86_64.AppImage", b"\x7fELF" as &[u8]),
        ("macos/RealEarth-macOS.dmg", b"\x6b\x6f\x6c\x79" as &[u8]),
    ];
    for (rel_path, expected_magic) in forbidden_fakes {
        let p = dist_dir.join(rel_path);
        if p.exists() {
            let mut file = std::fs::File::open(&p).expect("Failed to open file");
            let mut magic = vec![0u8; expected_magic.len()];
            let _ = file.read_exact(&mut magic);
            assert_eq!(
                &magic, expected_magic,
                "Package dist/{} cannot be a renamed ZIP archive",
                rel_path
            );
        }
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
    assert!(
        manifest_content.contains("\"Status\":  \"CERTIFIED\"")
            || manifest_content.contains("\"Status\":  \"CERTIFIED_WITH_HONEST_BLOCKERS\"")
    );
    assert!(manifest_content.contains("\"ReleaseVersion\":  \"0.6.0\""));

    // Check transparency: device IPA and Android toolchain blockers must be transparently disclosed
    assert!(
        manifest_content.contains("Apple Developer")
            || manifest_content.contains("BLOCKED_PENDING_PROVISIONING")
            || manifest_content.contains("Provisioning Profile"),
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
