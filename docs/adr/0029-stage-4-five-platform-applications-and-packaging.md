# ADR 0029: Stage 4 — Five-Platform Applications, Native Packaging, and Anti-Fabrication Hardening

## Status
Accepted

## Context
Prior iterations of the Real Earth Platform contained critical packaging and platform compatibility flaws:
1. **Mislabeled Pseudo-Archives**: Standard ZIP archives were renamed to `.AppImage`, `.dmg`, `.apk`, `.aab`, and `.ipa`. Furthermore, Windows PE32+ executables (`realearth-viewer.exe`) and DLLs (`realearth.dll`) were staged inside Linux AppDirs, macOS app bundles, Android `jniLibs`, and iOS bundles.
2. **Repository/Target Directory Dependency**: The Windows executable and CLI failed or panicked when executed outside the repository root or when `target/` was absent. Assets and world packages were hardcoded to development paths.
3. **Flawed Mobile JNI ABI**: JNI entry points in `bindings/mobile/src/lib.rs` cast raw pointers to `*const c_char` rather than using genuine JNI environment methods (`GetStringUTFChars`, `ReleaseStringUTFChars`). Native surface attachment, thermal throttling callbacks, and Android memory trim hooks were missing.
4. **Unreproducible Certifications**: Release integrity verification scripts accepted `PK\x03\x04` magic for all 9 packages regardless of whether the binary format was an AppImage, DMG, APK, AAB, or IPA.

## Decisions

### 1. Zero Repository/Target Dependency & Bundled Pilot World (Windows)
- Implemented executable-relative asset and package discovery in `engine/ingestion/src/authentic.rs` (`search_roots` inspecting `current_exe().parent()` and ancestors).
- Implemented `resolve_world_package_dir(slug)` in `apps/world-viewer/src/interactive.rs` and `apps/world-viewer/src/main.rs`, checking bundled `data/world_<slug>`, sibling `exe_dir/data/world_<slug>`, and runtime cache.
- Bundled the authentic precompiled San Francisco pilot world package (`data/world_san_francisco/chunks.db` and 30 authentic binary `.rec` chunks) along with `config/engine_config.json`, assets, docs, and batch launchers (`run_viewer.bat`, `run_cli.bat`).
- Verified standalone execution by extracting `dist/windows/realearth-windows-x64.zip` into an isolated directory (`dist/windows/test_standalone`) with zero workspace or `target/` dependencies.

### 2. Authentic Linux AppDir & Vulkan Backend Setup
- Structured `RealEarth.AppDir` following the official AppImage specification: `usr/bin`, `usr/lib`, `usr/share/applications/realearth.desktop`, `usr/share/realearth/assets`, `usr/share/realearth/config`, and `usr/share/realearth/data`.
- Implemented `packaging/linux/AppRun` configuring `WGPU_BACKEND="vulkan,gl"`, `VK_ICD_FILENAMES`, Wayland/X11 display variables, and executing `realearth-viewer`.
- Packaged the AppDir into `dist/linux/realearth-linux-x64.tar.gz` (genuine POSIX gzip tarball).
- Eradicated mislabeled zip-as-AppImage archives: an AppImage is only produced if genuine `appimagetool` is available on the host (generating an ELF executable with squashfs payload). Otherwise, the AppDir tarball is provided and the host toolchain requirement is explicitly documented.

### 3. Authentic macOS Application Bundle & Metal Backend Setup
- Structured `RealEarth.app` containing `Contents/MacOS`, `Contents/Info.plist` (declaring `NSHighResolutionCapable`, `CFBundleIdentifier`, `CFBundlePackageType: APPL`), `Contents/PkgInfo: APPL????`, and `Contents/Resources`.
- Packaged the app bundle into `dist/macos/realearth-macos-universal.tar.gz`.
- Eradicated mislabeled zip-as-DMG archives: genuine DMG images require macOS `hdiutil` (`hdiutil create -volname RealEarth -srcfolder RealEarth.app -ov -format UDZO`). On non-macOS hosts, this requirement is explicitly documented without creating fake archives.

### 4. Authentic Android JNI ABI & Native Surface Lifecycle
- Refactored `bindings/mobile/src/lib.rs` to use genuine JNI environment functions:
  - String parsing: `GetStringUTFChars` and guaranteed `ReleaseStringUTFChars` cleanup.
  - Complete native method suite: `nativeInit`, `nativeStep`, `nativeShutdown`, `nativeGetSpeed`, `nativeGetLat`, `nativeGetLon`, `nativeGetAlt`, `nativeSurfaceCreated`, `nativeSurfaceChanged`, `nativeSurfaceDestroyed`, `nativeSetThermalThrottle`, `nativeTrimMemory`.
- Created comprehensive unit tests validating JNI string parsing, memory ownership, and surface callbacks using mock `JNINativeInterface_`.
- Updated `android/app/src/main/java/com/realearth/world/MainActivity.java` and `RealEarthEngine.java` linking `SurfaceHolder.Callback`, virtual steering/pedal controls, thermal throttling listeners, and low-memory cache pruning.
- Eradicated mislabeled zip-as-APK/AAB archives: authentic packaging requires Android SDK 34, NDK r26b, and Gradle (`./gradlew assembleRelease bundleRelease`). On hosts without the Android SDK, `dist/android/android_build_status.json` documents the exact blocker.

### 5. Authentic iOS UIKit/Metal Bridge & Simulator Package
- Structured `ios/RealEarth` with `ViewController.m`, `RealEarthBridge.m`, `AppDelegate.m`, and `Info.plist` requiring Metal capability (`metal`).
- Packaged the iOS Simulator application into `dist/ios/realearth-ios-simulator.zip`.
- Documented Apple Developer code-signing and mobileprovision requirements for physical device IPAs. Eradicated fake zip-as-IPA archives; generated `dist/ios/ios_build_status.json`.

### 6. Anti-Fabrication Hardening & Release Integrity Verification
- Updated `scripts/verify_release_integrity.ps1` and `scripts/verify_release_integrity.sh` to enforce true magic signatures:
  - Windows: ZIP (`PK\x03\x04`).
  - Linux: Gzip tarball (`\x1F\x8B`), AppImage if present must be ELF (`\x7FELF`).
  - macOS: Gzip tarball (`\x1F\x8B`), DMG if present must have `koly` trailer.
  - iOS Simulator: ZIP (`PK\x03\x04`).
- Documented honest platform blockers in `dist/release_manifest.json`.

## Consequences
- Five platform targets have authentic code trees, manifests, assets, and shaders.
- Zero fake or mislabeled archives exist anywhere in the distribution pipeline.
- Windows application runs standalone out of the box with an authentic precompiled pilot region.
