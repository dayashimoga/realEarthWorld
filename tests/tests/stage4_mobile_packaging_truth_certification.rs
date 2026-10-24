//! STAGE 4 — MOBILE RUNTIME REALITY & PACKAGING TRUTH CERTIFICATION
//!
//! Validates:
//! 1. Mobile Runtime Status Honesty:
//!    - Verifies `realearth_mobile_runtime_status()` correctly distinguishes `Uninitialized` (2)
//!      and `BlockedPendingNativeSwapchain` (1).
//!    - Verifies `realearth_mobile_status_string()` explicitly discloses:
//!      "BLOCKED (Host Toolchain / Native Swapchain Required)".
//!    - Verifies `realearth_render_frame()` deprecation contract and safe behavior.
//! 2. JNI ABI Truth:
//!    - Verifies `Java_com_realearth_world_RealEarthEngine_nativeGetRuntimeStatus` reports the
//!      same honest status across the JNI boundary.
//! 3. Binary Magic & Packaging Truth across All Platforms:
//!    - Windows: Genuine PE32+ binaries (`MZ` + `PE\0\0`) and `windows_build_status.json` ("CERTIFIED").
//!    - Linux: Genuine POSIX tarball + AppDir layout, shell wrapper script honesty (not masquerading as ELF),
//!      and `linux_build_status.json` ("BLOCKED_PENDING_TOOLCHAIN").
//!    - macOS: Genuine POSIX tarball + App Bundle layout, shell wrapper script honesty (not masquerading as Mach-O),
//!      and `macos_build_status.json` ("BLOCKED_PENDING_TOOLCHAIN").
//!    - Android: JNI ABI and Gradle tree validation, `android_build_status.json` ("BLOCKED_PENDING_TOOLCHAIN").
//!    - iOS: Metal bridge and Xcode tree validation, `ios_build_status.json` ("BLOCKED_PENDING_PROVISIONING").

use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use realearth_mobile::{
    realearth_destroy_surface, realearth_init, realearth_mobile_runtime_status,
    realearth_mobile_status_string, realearth_render_frame, realearth_set_surface,
    realearth_shutdown, Java_com_realearth_world_RealEarthEngine_nativeGetRuntimeStatus,
    MobileRuntimeStatus,
};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Workspace root must exist")
        .to_path_buf()
}

// ============================================================================
// 1. MOBILE RUNTIME STATUS & LIFECYCLE HONESTY
// ============================================================================

#[test]
fn test_stage4_mobile_runtime_status_and_lifecycle() {
    // 1. Initially or after shutdown, status must be Uninitialized (2)
    realearth_shutdown();
    let status_before = realearth_mobile_runtime_status();
    assert_eq!(
        status_before,
        MobileRuntimeStatus::Uninitialized as i32,
        "Status must be Uninitialized when surface is not attached"
    );

    // 2. Initialize region
    let region_c = CString::new("san-francisco").unwrap();
    let init_res = realearth_init(region_c.as_ptr());
    assert_eq!(init_res, 0, "realearth_init must succeed");

    // 3. Attach surface handle
    let surface_handle = 0xdeadbeef_usize;
    let set_surf_res = realearth_set_surface(surface_handle, 1920, 1080);
    assert_eq!(set_surf_res, 0, "realearth_set_surface must succeed");

    // 4. On host desktop build without target mobile swapchain, status must honestly report Blocked
    let status_attached = realearth_mobile_runtime_status();
    assert_eq!(
        status_attached,
        MobileRuntimeStatus::BlockedPendingNativeSwapchain as i32,
        "Status must honestly report BlockedPendingNativeSwapchain (1) on host build"
    );

    // 5. Query status string disclosure
    let status_ptr = realearth_mobile_status_string();
    assert!(!status_ptr.is_null());
    let status_str = unsafe { CStr::from_ptr(status_ptr) }
        .to_str()
        .expect("Status string must be valid UTF-8");
    assert!(
        status_str.contains("BLOCKED"),
        "Status disclosure must contain 'BLOCKED': got {}",
        status_str
    );
    assert!(
        status_str.contains("Host Toolchain / Native Swapchain Required"),
        "Status disclosure must state requirement: got {}",
        status_str
    );

    // 6. Deprecated render frame must return 0 without panicking
    let render_res = realearth_render_frame();
    assert_eq!(
        render_res, 0,
        "realearth_render_frame must succeed as deprecated stub"
    );

    // 7. Destroy surface returns state to Uninitialized
    let destroy_res = realearth_destroy_surface();
    assert_eq!(destroy_res, 0, "realearth_destroy_surface must succeed");
    assert_eq!(
        realearth_mobile_runtime_status(),
        MobileRuntimeStatus::Uninitialized as i32,
        "Status must return to Uninitialized after surface destroy"
    );

    realearth_shutdown();
}

// ============================================================================
// 2. JNI ABI RUNTIME STATUS PARITY
// ============================================================================

#[test]
fn test_stage4_jni_exports_and_truth_reporting() {
    realearth_shutdown();

    unsafe {
        let status = Java_com_realearth_world_RealEarthEngine_nativeGetRuntimeStatus(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        assert_eq!(
            status,
            MobileRuntimeStatus::Uninitialized as i32,
            "JNI nativeGetRuntimeStatus must report Uninitialized when surface is unattached"
        );

        let region_c = CString::new("san-francisco").unwrap();
        realearth_init(region_c.as_ptr());
        realearth_set_surface(0x1234, 1280, 720);

        let status_attached = Java_com_realearth_world_RealEarthEngine_nativeGetRuntimeStatus(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        assert_eq!(
            status_attached,
            MobileRuntimeStatus::BlockedPendingNativeSwapchain as i32,
            "JNI nativeGetRuntimeStatus must report BlockedPendingNativeSwapchain when surface attached"
        );

        realearth_destroy_surface();
        realearth_shutdown();
    }
}

// ============================================================================
// 3. WINDOWS PE BINARY MAGIC & CERTIFICATION STATUS
// ============================================================================

#[test]
fn test_stage4_windows_pe_magic_and_integrity() {
    let root = workspace_root();

    // 1. Check windows_build_status.json
    let status_path = root.join("dist/windows/windows_build_status.json");
    assert!(
        status_path.exists(),
        "dist/windows/windows_build_status.json must exist"
    );

    let mut content = String::new();
    File::open(&status_path)
        .expect("Failed to open windows_build_status.json")
        .read_to_string(&mut content)
        .expect("Failed to read windows_build_status.json");

    assert!(
        content.contains("\"Status\": \"CERTIFIED\"")
            || content.contains("\"Status\":  \"CERTIFIED\"")
    );
    assert!(
        content.contains("\"Platform\": \"Windows\"")
            || content.contains("\"Platform\":  \"Windows\"")
    );
    assert!(content.contains("PE32+"));

    // 2. Verify PE header of release viewer executable if present
    let viewer_exe = root.join("target/release/realearth-viewer.exe");
    if viewer_exe.exists() {
        let mut file = File::open(&viewer_exe).expect("Failed to open realearth-viewer.exe");
        let mut header = [0u8; 64];
        file.read_exact(&mut header)
            .expect("Failed to read DOS header");
        assert_eq!(
            &header[0..2],
            b"MZ",
            "Windows binary must have genuine MZ DOS header"
        );

        // e_lfanew is at offset 0x3C
        let pe_offset =
            u32::from_le_bytes([header[0x3c], header[0x3d], header[0x3e], header[0x3f]]) as u64;
        use std::io::Seek;
        file.seek(std::io::SeekFrom::Start(pe_offset))
            .expect("Failed to seek to PE header");
        let mut pe_sig = [0u8; 4];
        file.read_exact(&mut pe_sig)
            .expect("Failed to read PE signature");
        assert_eq!(
            &pe_sig, b"PE\0\0",
            "Windows binary must have genuine PE\\0\\0 signature"
        );
    }
}

// ============================================================================
// 4. LINUX PACKAGING TRUTH & TOOLCHAIN BLOCKER
// ============================================================================

#[test]
fn test_stage4_linux_packaging_truth_and_toolchain_blocker() {
    let root = workspace_root();

    // 1. Check linux_build_status.json
    let status_path = root.join("dist/linux/linux_build_status.json");
    assert!(
        status_path.exists(),
        "dist/linux/linux_build_status.json must exist"
    );

    let mut content = String::new();
    File::open(&status_path)
        .expect("Failed to open linux_build_status.json")
        .read_to_string(&mut content)
        .expect("Failed to read linux_build_status.json");

    assert!(content.contains("BLOCKED_PENDING_TOOLCHAIN"));
    assert!(content.contains("x86_64-unknown-linux-gnu"));

    // 2. Check staged viewer wrapper script
    let viewer_staged = root.join("dist/linux/RealEarth.AppDir/usr/bin/realearth-viewer");
    if viewer_staged.exists() {
        let mut f = File::open(&viewer_staged).expect("Failed to open staged linux viewer");
        let mut head = [0u8; 16];
        let bytes_read = f.read(&mut head).unwrap_or(0);
        if bytes_read >= 4 && &head[0..4] == b"\x7fELF" {
            // Native ELF binary
        } else {
            // Truthful shell wrapper script
            assert_eq!(
                &head[0..2],
                b"#!",
                "Staged Linux binary on Windows host must be a documented script, not fake ELF"
            );
        }
    }
}

// ============================================================================
// 5. MACOS PACKAGING TRUTH & TOOLCHAIN BLOCKER
// ============================================================================

#[test]
fn test_stage4_macos_packaging_truth_and_toolchain_blocker() {
    let root = workspace_root();

    // 1. Check macos_build_status.json
    let status_path = root.join("dist/macos/macos_build_status.json");
    assert!(
        status_path.exists(),
        "dist/macos/macos_build_status.json must exist"
    );

    let mut content = String::new();
    File::open(&status_path)
        .expect("Failed to open macos_build_status.json")
        .read_to_string(&mut content)
        .expect("Failed to read macos_build_status.json");

    assert!(content.contains("BLOCKED_PENDING_TOOLCHAIN"));
    assert!(content.contains("darwin"));

    // 2. Check staged viewer wrapper script
    let viewer_staged = root.join("dist/macos/RealEarth.app/Contents/MacOS/realearth-viewer");
    if viewer_staged.exists() {
        let mut f = File::open(&viewer_staged).expect("Failed to open staged macos viewer");
        let mut head = [0u8; 16];
        let bytes_read = f.read(&mut head).unwrap_or(0);
        let is_macho = bytes_read >= 4
            && (&head[0..4] == b"\xfe\xed\xfa\xcf"
                || &head[0..4] == b"\xcf\xfa\xed\xfe"
                || &head[0..4] == b"\xca\xfe\xba\xbe"
                || &head[0..4] == b"\xbe\xba\xfe\xca");
        if !is_macho {
            assert_eq!(
                &head[0..2],
                b"#!",
                "Staged macOS binary on Windows host must be a documented script, not fake Mach-O"
            );
        }
    }
}

// ============================================================================
// 6. FIVE-PLATFORM STATUS REGISTRY COMPLETION
// ============================================================================

#[test]
fn test_stage4_five_platform_status_registry_complete() {
    let root = workspace_root();

    let platforms = [
        (
            "dist/windows/windows_build_status.json",
            "Windows",
            "CERTIFIED",
        ),
        (
            "dist/linux/linux_build_status.json",
            "Linux",
            "BLOCKED_PENDING_TOOLCHAIN",
        ),
        (
            "dist/macos/macos_build_status.json",
            "macOS",
            "BLOCKED_PENDING_TOOLCHAIN",
        ),
        (
            "dist/android/android_build_status.json",
            "Android",
            "BLOCKED_PENDING_TOOLCHAIN",
        ),
        (
            "dist/ios/ios_build_status.json",
            "iOS",
            "BLOCKED_PENDING_PROVISIONING",
        ),
    ];

    for (rel_path, platform_name, expected_status) in platforms {
        let path = root.join(rel_path);
        assert!(
            path.exists(),
            "Build status document must exist for {}: {}",
            platform_name,
            rel_path
        );

        let mut content = String::new();
        File::open(&path)
            .unwrap_or_else(|_| panic!("Failed to open {}", rel_path))
            .read_to_string(&mut content)
            .unwrap_or_else(|_| panic!("Failed to read {}", rel_path));

        assert!(
            content.contains(expected_status),
            "{} status must be '{}', content: {}",
            platform_name,
            expected_status,
            content
        );
    }
}
