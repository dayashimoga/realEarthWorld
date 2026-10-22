# ADR-0022: Stage F — Five-Platform Application Completeness & Parity Certification

## Status
Accepted

## Date
2026-10-22

## Context
Following the successful completion of Stage B (Adversarial Validation), Stage C (Visual Fidelity Certification), Stage D (US City Catalog), and Stage E (Complete Defect Remediation), the Real Earth Platform required genuine multi-platform deployment packages across all five supported operating systems:
1. **Windows (F1)**: Desktop x86_64 application with DirectX 12/wgpu HAL, native input, controllers, and portable ZIP distribution.
2. **Linux (F2)**: Desktop x86_64 application with Vulkan HAL, AppImage packaging, XDG desktop entry, and tarball distribution.
3. **macOS (F3)**: Desktop universal application with Metal HAL, high-DPI Retina scaling, Info.plist bundle configuration, DMG disk image, and tarball distribution.
4. **Android (F4)**: Mobile ARM64 application with Rust engine integration (`librealearth.so` C FFI / JNI), `SurfaceHolder` native surface lifecycle, virtual touch steering/pedal controls, adaptive thermal throttling, low-memory response, and APK/AAB distribution.
5. **iOS (F5)**: Mobile ARM64 application with Metal UIKit bridge, `MTKView` 60Hz `CADisplayLink` rendering, Dynamic Island safe area insets, `didReceiveMemoryWarning` cache pruning, Simulator `.app`, and signed device `.ipa` distribution.
6. **Cross-Platform Parity (F6)**: Unified physics, navigation, region selection, gameplay, persistence, graphics profiles, and telemetry verification across all five operating systems.

## Decisions

### 1. Unified Native Core Architecture
- The core simulation, geospatial coordinates, chunk streaming, vehicle dynamics, navigation, and gameplay remain 100% unified in the `realearth-*` Rust crates.
- Platform-specific bindings are cleanly isolated:
  - Desktop (Windows, Linux, macOS) leverage `apps/world-viewer` using `winit` and `wgpu`.
  - Mobile (Android, iOS) leverage `bindings/mobile` exposing stable C ABI and JNI entry points with zero overhead.

### 2. Mobile Surface Lifecycle & Virtual Controls (Android F4)
- **Native Surface**: Implemented in [`MainActivity.java`](file:///j:/rustRealEarth/android/app/src/main/java/com/realearth/world/MainActivity.java) via `SurfaceHolder.Callback` (`surfaceCreated`, `surfaceChanged`, `surfaceDestroyed`), ensuring zero render attempts while surfaces are backgrounded.
- **Virtual Driving Controls**: Dual-input scheme combining continuous steer seekbar (`-1.0` to `+1.0`) with touch-and-hold virtual pedals for throttle and service brake.
- **Thermal Management**: Real-time `PowerManager.OnThermalStatusChangedListener` actively regulates render loop interval:
  - `THERMAL_STATUS_NONE`: 60 FPS (16.67 ms)
  - `THERMAL_STATUS_MODERATE`: 45 FPS (22.22 ms)
  - `THERMAL_STATUS_SEVERE`: 30 FPS (33.33 ms)
  - `THERMAL_STATUS_EMERGENCY`: 20 FPS (50.00 ms)
- **Memory Pressure**: `onTrimMemory(TRIM_MEMORY_RUNNING_CRITICAL)` and `onLowMemory()` invoke engine cache pruning to safeguard against out-of-memory termination.

### 3. iOS Metal UIKit Architecture (iOS F5)
- **Metal Pipeline**: [`ViewController.m`](file:///j:/rustRealEarth/ios/RealEarth/ViewController.m) hosts an `MTKView` driven by a 60 Hz `CADisplayLink` synchronized with the display refresh.
- **Safe Area Insets**: Layout constraints bind controls to `view.safeAreaLayoutGuide`, dynamically adapting to notch and Dynamic Island geometry (top inset 59 pt on iPhone 15 Pro).
- **Lifecycle & Memory**: App transitions (`applicationDidEnterBackground`, `applicationWillEnterForeground`) pause and resume the simulation clock; `didReceiveMemoryWarning` purges unreferenced chunk caches.

### 4. Deterministic Packaging & Checksum Verification
Automated cross-platform packaging scripts in PowerShell (`scripts/package_*.ps1`) and POSIX Bash (`scripts/package_*.sh`) generate production bundles with SHA-256 integrity digests:
- Windows: `dist/windows/realearth-windows-x64.zip`
- Linux: `dist/linux/RealEarth-x86_64.AppImage` & `realearth-linux-x64.tar.gz`
- macOS: `dist/macos/RealEarth-macOS.dmg`, `RealEarth.app`, & `realearth-macos-universal.tar.gz`
- Android: `dist/android/realearth-arm64.apk` & `realearth-release.aab`
- iOS: `dist/ios/realearth-ios-device.ipa` & `realearth-ios-simulator.zip`

### 5. Transparent Physical Device Limitations
- Android packaging validates ARM64 APK and AAB structural compliance, Dalvik DEX/JNI ABI compatibility, and emulator execution.
- iOS packaging produces clean `.app` Simulator bundles and signed `.ipa` containers. Deployment to physical Apple hardware requires an active Apple Developer Team Provisioning Profile.

## Consequences & Parity Verification
- Bit-exact vehicle dynamics: Identical linear velocity, angular velocity, and suspension forces between mobile and desktop test instances.
- Clamped mobile profiles: Low graphics preset scales view distance to 1500m (vs 4000m on High) while preserving 60 FPS nominal frame-time targets.
- All 6 platform completeness and parity regression tests pass under [`tests/tests/five_platform_application_parity.rs`](file:///j:/rustRealEarth/tests/tests/five_platform_application_parity.rs).
