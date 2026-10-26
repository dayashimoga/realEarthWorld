//! STAGE A — ARTIFACT-BASED PACKAGED RELEASE SMOKE TEST
//!
//! Validates the packaged Windows release distribution:
//! 1. Verifies existence and integrity of `dist/windows/realearth-windows-x64.zip`.
//! 2. Extracts zip into an isolated clean directory outside the repository source tree.
//! 3. Launches the standalone `realearth-viewer.exe` binary in headless driving mode (`--headless --drive --frames 10`).
//! 4. Asserts that the standalone application executes cleanly without requiring repository-relative resources.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn test_packaged_windows_release_artifact_smoke() {
    println!("\n📦 [ARTIFACT SMOKE TEST] Validating Packaged Windows Release Distribution...");

    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Workspace root")
        .to_path_buf();
    let zip_path = workspace_root
        .join("dist")
        .join("windows")
        .join("realearth-windows-x64.zip");

    assert!(
        zip_path.exists(),
        "Packaged artifact must exist at {}",
        zip_path.display()
    );

    let zip_metadata = fs::metadata(&zip_path).expect("Read zip metadata");
    let zip_size_mb = zip_metadata.len() as f64 / (1024.0 * 1024.0);
    println!(
        "  • Found release package: {} ({:.2} MB)",
        zip_path.display(),
        zip_size_mb
    );
    assert!(
        zip_metadata.len() > 8_000_000,
        "Package must be complete (> 8MB)"
    );

    // Create isolated temporary directory
    let temp_extract_dir =
        std::env::temp_dir().join(format!("realearth_smoke_{}", std::process::id()));
    if temp_extract_dir.exists() {
        let _ = fs::remove_dir_all(&temp_extract_dir);
    }
    fs::create_dir_all(&temp_extract_dir).expect("Create clean temporary directory");

    println!(
        "  • Extracting artifact to isolated temp folder: {}",
        temp_extract_dir.display()
    );

    // Extract using Windows built-in tar.exe
    let extract_output = Command::new("tar.exe")
        .args([
            "-xf",
            zip_path.to_str().unwrap(),
            "-C",
            temp_extract_dir.to_str().unwrap(),
        ])
        .output()
        .expect("Execute tar.exe to extract zip artifact");

    assert!(
        extract_output.status.success(),
        "tar.exe extraction failed: {}",
        String::from_utf8_lossy(&extract_output.stderr)
    );

    // Locate extracted realearth-viewer.exe
    let exe_path = if temp_extract_dir.join("realearth-viewer.exe").exists() {
        temp_extract_dir.join("realearth-viewer.exe")
    } else if temp_extract_dir
        .join("realearth-windows-x64")
        .join("realearth-viewer.exe")
        .exists()
    {
        temp_extract_dir
            .join("realearth-windows-x64")
            .join("realearth-viewer.exe")
    } else {
        panic!(
            "realearth-viewer.exe not found in extracted artifact at {}",
            temp_extract_dir.display()
        );
    };

    let exe_work_dir = exe_path.parent().unwrap();
    println!("  • Located standalone binary: {}", exe_path.display());
    assert!(exe_path.exists(), "Standalone binary must exist");

    // Execute standalone binary in headless driving mode strictly within its own extracted directory
    println!(
        "  • Invoking standalone binary: {} --headless --drive --frames 10",
        exe_path.display()
    );
    let run_output = Command::new(&exe_path)
        .current_dir(exe_work_dir)
        .args(["--headless", "--drive", "--frames", "10"])
        .output()
        .expect("Failed to execute extracted realearth-viewer.exe");

    let stdout_str = String::from_utf8_lossy(&run_output.stdout);
    let stderr_str = String::from_utf8_lossy(&run_output.stderr);

    println!(
        "  • Standalone execution exit status: {}",
        run_output.status
    );
    if !stdout_str.is_empty() {
        println!(
            "  • Output snippet: {}",
            stdout_str.lines().next().unwrap_or("")
        );
    }

    // Verify exit status
    assert!(
        run_output.status.success(),
        "Extracted standalone binary failed with status {}: stderr={}",
        run_output.status,
        stderr_str
    );

    // Clean up temporary directory
    let _ = fs::remove_dir_all(&temp_extract_dir);
    println!("  ✓ Cleaned up temporary smoke testing folder");
}
