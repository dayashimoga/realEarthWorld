//! STAGE A — ARTIFACT-BASED PACKAGED RELEASE SMOKE TEST
//!
//! Validates the packaged Windows release distribution:
//! 1. Verifies existence and integrity of `dist/windows/realearth-windows-x64.zip`.
//! 2. Extracts zip into an isolated clean directory outside the repository source tree.
//! 3. Launches the standalone `realearth-viewer.exe` binary in headless driving mode (`--headless --drive --frames 10`).
//! 4. Asserts that the standalone application executes cleanly without requiring repository-relative resources.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

fn extract_zip_archive(zip_path: &Path, extract_dir: &Path) -> bool {
    let zip_str = zip_path.to_str().unwrap();
    let dir_str = extract_dir.to_str().unwrap();

    #[cfg(target_os = "windows")]
    {
        // 1. Try Windows built-in tar (bsdtar)
        if let Ok(mut child) = Command::new("tar")
            .args(["-xf", zip_str, "-C", dir_str])
            .spawn()
        {
            if let Ok(status) = child.wait() {
                if status.success() {
                    return true;
                }
            }
        }

        // 2. Try tar.exe explicitly
        if let Ok(mut child) = Command::new("tar.exe")
            .args(["-xf", zip_str, "-C", dir_str])
            .spawn()
        {
            if let Ok(status) = child.wait() {
                if status.success() {
                    return true;
                }
            }
        }

        // 3. Try PowerShell Expand-Archive
        if let Ok(mut child) = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                    zip_path.display(),
                    extract_dir.display()
                ),
            ])
            .spawn()
        {
            if let Ok(status) = child.wait() {
                if status.success() {
                    return true;
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        // 1. Try unzip (standard on Linux / macOS CI runners)
        if let Ok(mut child) = Command::new("unzip")
            .args(["-q", "-o", zip_str, "-d", dir_str])
            .spawn()
        {
            if let Ok(status) = child.wait() {
                if status.success() {
                    return true;
                }
            }
        }

        // 2. Try python3 -m zipfile
        if let Ok(mut child) = Command::new("python3")
            .args(["-m", "zipfile", "-e", zip_str, dir_str])
            .spawn()
        {
            if let Ok(status) = child.wait() {
                if status.success() {
                    return true;
                }
            }
        }

        // 3. Try python -m zipfile
        if let Ok(mut child) = Command::new("python")
            .args(["-m", "zipfile", "-e", zip_str, dir_str])
            .spawn()
        {
            if let Ok(status) = child.wait() {
                if status.success() {
                    return true;
                }
            }
        }

        // 4. Try tar
        if let Ok(mut child) = Command::new("tar")
            .args(["-xf", zip_str, "-C", dir_str])
            .spawn()
        {
            if let Ok(status) = child.wait() {
                if status.success() {
                    return true;
                }
            }
        }
    }

    false
}

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

    let extracted = extract_zip_archive(&zip_path, &temp_extract_dir);
    assert!(
        extracted,
        "Extraction of distribution ZIP failed on this platform: {}",
        zip_path.display()
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

    let is_windows = cfg!(target_os = "windows");
    let has_wine = if !is_windows {
        Command::new("wine")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    } else {
        false
    };

    if is_windows || has_wine {
        println!(
            "  • Invoking standalone binary: {} --headless --drive --frames 10{}",
            exe_path.display(),
            if has_wine {
                " (via Wine emulation)"
            } else {
                ""
            }
        );
        let mut cmd = if is_windows {
            Command::new(&exe_path)
        } else {
            let mut c = Command::new("wine");
            c.arg(&exe_path);
            c
        };
        cmd.current_dir(exe_work_dir)
            .args(["--headless", "--drive", "--frames", "10"]);

        let run_output = cmd
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
    } else {
        println!("  • Running on non-Windows host without Wine; validating PE32+ binary integrity & structure...");
        let mut exe_file = fs::File::open(&exe_path).expect("Open extracted exe");
        let mut dos_header = [0u8; 64];
        exe_file
            .read_exact(&mut dos_header)
            .expect("Read DOS header");
        assert_eq!(
            &dos_header[0..2],
            b"MZ",
            "Executable must have DOS MZ magic"
        );

        let pe_offset = u32::from_le_bytes([
            dos_header[0x3C],
            dos_header[0x3D],
            dos_header[0x3E],
            dos_header[0x3F],
        ]) as usize;

        let mut exe_full = Vec::new();
        fs::File::open(&exe_path)
            .expect("Open exe")
            .read_to_end(&mut exe_full)
            .expect("Read exe bytes");

        assert!(
            pe_offset + 6 <= exe_full.len(),
            "PE header offset must be within file"
        );
        assert_eq!(
            &exe_full[pe_offset..pe_offset + 4],
            b"PE\x00\x00",
            "Executable must have PE signature"
        );
        let machine = u16::from_le_bytes([exe_full[pe_offset + 4], exe_full[pe_offset + 5]]);
        assert_eq!(
            machine, 0x8664,
            "Windows binary must be AMD64 (x86_64 PE32+)"
        );
        println!(
            "  ✓ Verified authentic Windows x86_64 PE32+ binary structure (size: {} bytes)",
            exe_full.len()
        );
    }

    // Clean up temporary directory
    let _ = fs::remove_dir_all(&temp_extract_dir);
    println!("  ✓ Cleaned up temporary smoke testing folder");
}
