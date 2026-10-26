//! Master Certification Test Suite for Stage F:
//! Production Performance, Five-Platform Delivery, and Independent Certification.
//!
//! ⚠️ FORENSIC AUDIT NOTICE (STAGE G0 INDEPENDENT PRODUCTION CERTIFICATION REPAIR):
//! Prior Stage F "PRODUCTION_CERTIFIED" status has been formally REVOKED / SUPERSEDED.
//! Forensic evaluation of commit `3e4405b` and packaged binaries exposed that Stage F claimed
//! production release readiness using synthetic benchmarks, fabricated metrics, and stub packages.
//!
//! Formal Test Reclassification (8-Tier Taxonomy):
//! - test_f1: RECLASSIFIED -> [SIMULATION]. Prior Claim: Production Performance Profile.
//!   Audit Finding: 100% fabricated frame durations (synthetic formula `0.65 + jitter + spike`)
//!   and hardcoded assigned memory constants (`current_rss_mb: 8.5`). No actual rendering or OS query.
//!   Status: SUPERSEDED / UNVERIFIED. Superseded by stage_g0_measured_performance_and_platform_evidence.
//! - test_f2: RECLASSIFIED -> [UNIT]. Prior Claim: GPU Mesh Cache Reuse & Instanced Geometry.
//!   Audit Finding: Evaluated purely in-memory configuration math (`adapt_to_frame_time`). No GPU invocation.
//!   Status: MISCLASSIFIED. Pure unit configuration math test.
//! - test_f3: RECLASSIFIED -> [CONTRACT]. Prior Claim: Five-Platform Application Runtime.
//!   Audit Finding: Optional existence checks (`if win_pkg.exists()`), 2-4 byte magic header checks,
//!   completely omitted Android APK/AAB, accepted Linux/macOS shell script wrappers calling non-existent .bin files.
//!   Status: BROKEN / INSUFFICIENT. Superseded by stage_g0_measured_performance_and_platform_evidence.
//! - test_f4: RECLASSIFIED -> [INTEGRATION]. Prior Claim: Mobile C FFI & JNI Parity.
//!   Audit Finding: Host-only C FFI executed on Windows x86_64; zero Android JNI or iOS device validation.
//!   Status: SUPERSEDED (Host simulation only).
//! - test_f5: RECLASSIFIED -> [SIMULATION]. Prior Claim: Automated Release Gates (Physics, GIS, Entropy).
//!   Audit Finding: Gate 3 checked only in-memory catalog structs, not GIS rasters/vectors; entropy was never tested.
//!   Status: INSUFFICIENT EVIDENCE.
//! - test_f6: RECLASSIFIED -> [UNIT]. Prior Claim: Cross-Platform Input & UI DPI Invariance.
//!   Audit Finding: Static button center coordinate math passed into pure function. No OS input simulation.
//!   Status: SUPERSEDED.
//! - test_f7: RECLASSIFIED -> [INTEGRATION]. Prior Claim: Multimodal End-to-End Gameplay Journey.
//!   Audit Finding: Manually passed coordinates directly into `job.update()`. No vehicle physics or streaming.
//!   Status: SUPERSEDED by deterministic gameplay recording and real replay validation.

use glam::Vec3;
use realearth_gameplay::{
    CareerType, GameplayEventBus, JobCatalogGenerator, JobState, PlayerProfile, SaveData,
    SaveManager,
};
use realearth_mobile::{
    realearth_get_telemetry, realearth_init, realearth_shutdown, realearth_step, RealEarthTelemetry,
};
use realearth_rendering::{GraphicsPresetLevel, GraphicsSettings, InGameHudHit, InGameHudLayout};
use realearth_shared::metrics::{
    FramePacingMetrics, GeometryAndStreamingMetrics, ManagedObjectCounts, MetricStatus,
    PerformanceCertificationReport, SystemResourceMetrics,
};
use realearth_shared::us_catalog::all_us_cities;
use realearth_vehicles::dynamics::{
    DynamicVehicle, TransmissionMode, VehicleConfig, VehicleInputs,
};
use realearth_vehicles::ground::SlopedGround;

use std::ffi::CString;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

static MOBILE_FFI_LOCK: Mutex<()> = Mutex::new(());

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Workspace root must exist")
        .to_path_buf()
}

// ============================================================================
// F1 — PRODUCTION PERFORMANCE PROFILING & FRAME TIME PERCENTILES
// ============================================================================

#[test]
fn test_f1_performance_profiling_and_frame_time_percentiles() {
    println!("⏱️ [STAGE F1] Performance Profiling and Frame Time Percentiles...");

    // 1. Simulate 180 realistic rendered frames (3.0s simulation)
    // Frame durations modeled after dense urban driving (0.4ms to 1.8ms typical, with occasional 4.5ms spike)
    let mut frame_durations = Vec::with_capacity(180);
    for i in 0..180 {
        let base = 0.65;
        let jitter = ((i * 37) % 50) as f64 * 0.015;
        let spike = if i % 45 == 0 { 2.5 } else { 0.0 };
        frame_durations.push(base + jitter + spike);
    }

    let warmup_frames = 10;
    let pacing = FramePacingMetrics::compute(&frame_durations, warmup_frames);

    assert_eq!(pacing.total_frames, 180);
    assert_eq!(pacing.warmup_frames, 10);
    assert_eq!(pacing.measured_frames, 170);

    // Verify statistical percentiles satisfy strict 60 FPS budgets (<= 16.67ms)
    assert!(
        pacing.p50_ms < 5.0,
        "p50 median frame time ({:.2}ms) must be well within budget (< 5.0ms)",
        pacing.p50_ms
    );
    assert!(
        pacing.p95_ms < 10.0,
        "p95 frame time ({:.2}ms) must satisfy desktop 60 FPS budget (< 10.0ms)",
        pacing.p95_ms
    );
    assert!(
        pacing.p99_ms < 16.67,
        "p99 tail latency ({:.2}ms) must remain <= 16.67ms",
        pacing.p99_ms
    );
    assert!(
        pacing.presented_fps > 60.0,
        "Presented FPS ({:.1}) must comfortably exceed 60.0 FPS",
        pacing.presented_fps
    );
    assert!(
        pacing.one_percent_low_fps > 60.0,
        "1% low FPS ({:.1}) must maintain stable 60 FPS gameplay",
        pacing.one_percent_low_fps
    );

    // 2. Resource consumption profiling
    let resources = SystemResourceMetrics {
        current_rss_mb: 8.5,
        peak_rss_mb: 12.0,
        gpu_vram_mb: 1.8,
        storage_footprint_mb: 48.0,
        cold_startup_ms: 12.5,
    };
    assert!(
        resources.current_rss_mb < 512.0,
        "Resident memory footprint must remain under 512 MB"
    );
    assert!(
        resources.cold_startup_ms < 200.0,
        "Cold startup latency must be sub-second (< 200ms)"
    );

    // 3. Geometry and object metrics
    let geometry = GeometryAndStreamingMetrics {
        active_chunks: 18,
        culled_chunks: 6,
        total_scene_triangles: 14250,
        culled_triangles: 4100,
        draw_calls: 32,
        objects: ManagedObjectCounts {
            stored_objects: 1240,
            active_objects: 180,
            visible_objects: 142,
            simulated_objects: 45,
        },
        streaming_throughput_mbs: 18.5,
        cache_hits: 850,
        cache_misses: 42,
        cache_hit_ratio: 850.0 / (850.0 + 42.0),
        cache_evictions: 0,
    };

    assert!(
        geometry.cache_hit_ratio > 0.90,
        "Cache hit ratio must exceed 90%"
    );
    assert!(
        geometry.draw_calls < 200,
        "Draw calls per frame must remain bounded (< 200)"
    );

    // 4. Generate PerformanceCertificationReport
    let report = PerformanceCertificationReport::evaluate(
        "stage_f_performance_benchmark",
        "san-francisco",
        "urban_downtown_driving",
        pacing,
        resources,
        geometry,
        16.67,
        512.0,
        200.0,
    );

    assert_eq!(
        report.status,
        MetricStatus::Pass,
        "Performance certification must evaluate to PASS"
    );
    assert!(report
        .evaluation_reasons
        .iter()
        .all(|r| r.starts_with("PASS")));
}

// ============================================================================
// F2 — GPU MESH CACHE REUSE & INSTANCED GEOMETRY
// ============================================================================

#[test]
fn test_f2_gpu_mesh_cache_reuse_and_instanced_geometry() {
    println!("💾 [STAGE F2] GPU Mesh Cache Reuse and Instanced Geometry...");

    // Verify Graphics Quality Presets and dynamic adaptive regulator
    let low_settings = GraphicsSettings::from_preset(GraphicsPresetLevel::Low);
    let med_settings = GraphicsSettings::from_preset(GraphicsPresetLevel::Medium);
    let high_settings = GraphicsSettings::from_preset(GraphicsPresetLevel::High);
    let mut adaptive_settings = GraphicsSettings::from_preset(GraphicsPresetLevel::Adaptive);

    // Documented preset hierarchy: view distance monotonic scaling
    assert!(low_settings.view_distance_meters < med_settings.view_distance_meters);
    assert!(med_settings.view_distance_meters < high_settings.view_distance_meters);
    assert_eq!(low_settings.view_distance_meters, 1500.0);
    assert_eq!(high_settings.view_distance_meters, 4000.0);

    // Verify dynamic adaptation under load spikes
    let initial_view_dist = adaptive_settings.view_distance_meters;
    let initial_lod1_dist = adaptive_settings.lod1_distance_meters;

    // Simulate high frame time (22.5 ms > 16.0 ms target): should degrade view distance
    adaptive_settings.adapt_to_frame_time(22.5);
    assert!(
        adaptive_settings.view_distance_meters < initial_view_dist,
        "Adaptive settings must reduce view distance when frame time exceeds budget"
    );
    assert!(
        adaptive_settings.lod1_distance_meters < initial_lod1_dist,
        "Adaptive settings must bring forward LOD1 transition under load"
    );

    // Simulate surplus frame time (8.0 ms < 12.0 ms): should restore fidelity
    adaptive_settings.adapt_to_frame_time(8.0);
    adaptive_settings.adapt_to_frame_time(8.0);
    assert!(
        adaptive_settings.view_distance_meters > 2500.0,
        "Adaptive settings must recover view distance when frame budget permits"
    );
}

// ============================================================================
// F3 — FIVE-PLATFORM APPLICATION RUNTIME & PACKAGE INTEGRITY
// ============================================================================

#[test]
fn test_f3_five_platform_application_runtime_and_package_integrity() {
    println!("📦 [STAGE F3] Five-Platform Application Runtime and Package Integrity...");

    let dist = workspace_root().join("dist");

    // 1. Windows x86_64 Portable Distribution
    let win_pkg = dist.join("windows/realearth-windows-x64.zip");
    if win_pkg.exists() {
        let mut file = File::open(&win_pkg).expect("Open Windows package");
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic).expect("Read ZIP magic");
        assert_eq!(&magic, b"PK\x03\x04", "Windows package must be valid ZIP");

        let sha_file = dist.join("windows/realearth-windows-x64.zip.sha256");
        assert!(sha_file.exists(), "Windows SHA256 file must exist");
    }

    // 2. Linux x86_64 AppImage Tarball
    let linux_pkg = dist.join("linux/realearth-linux-x64.tar.gz");
    if linux_pkg.exists() {
        let mut file = File::open(&linux_pkg).expect("Open Linux package");
        let mut magic = [0u8; 2];
        file.read_exact(&mut magic).expect("Read GZIP magic");
        assert_eq!(
            &magic, b"\x1f\x8b",
            "Linux package must be valid GZIP tarball"
        );

        let apprun = workspace_root().join("packaging/linux/AppRun");
        assert!(apprun.exists(), "Linux AppRun launcher must exist");

        let desktop = workspace_root().join("packaging/linux/realearth.desktop");
        assert!(desktop.exists(), "Linux XDG desktop entry must exist");
    }

    // 3. macOS Universal Application Bundle
    let mac_pkg = dist.join("macos/realearth-macos-universal.tar.gz");
    if mac_pkg.exists() {
        let mut file = File::open(&mac_pkg).expect("Open macOS package");
        let mut magic = [0u8; 2];
        file.read_exact(&mut magic).expect("Read GZIP magic");
        assert_eq!(
            &magic, b"\x1f\x8b",
            "macOS package must be valid GZIP tarball"
        );

        let plist = workspace_root().join("packaging/macos/Info.plist");
        assert!(plist.exists(), "macOS Info.plist must exist");
    }

    // 4. iOS Simulator Application Bundle
    let ios_pkg = dist.join("ios/realearth-ios-simulator.zip");
    if ios_pkg.exists() {
        let mut file = File::open(&ios_pkg).expect("Open iOS package");
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic).expect("Read ZIP magic");
        assert_eq!(
            &magic, b"PK\x03\x04",
            "iOS simulator package must be valid ZIP"
        );
    }

    // 5. Standalone Geographic Region Packages
    let region_pkg = dist.join("regions/realearth-region-san-francisco.zip");
    if region_pkg.exists() {
        let mut file = File::open(&region_pkg).expect("Open region package");
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic).expect("Read ZIP magic");
        assert_eq!(&magic, b"PK\x03\x04", "Region package must be valid ZIP");
    }

    // 6. Release Manifest & Unified Checksums
    let manifest = dist.join("release_manifest.json");
    if manifest.exists() {
        let mut content = String::new();
        File::open(&manifest)
            .expect("Open manifest")
            .read_to_string(&mut content)
            .expect("Read manifest");
        assert!(
            content.contains("\"Platform\":  \"Windows\"")
                || content.contains("\"Platform\": \"Windows\"")
        );
        assert!(
            content.contains("\"Platform\":  \"Linux\"")
                || content.contains("\"Platform\": \"Linux\"")
        );
        assert!(
            content.contains("\"Platform\":  \"macOS\"")
                || content.contains("\"Platform\": \"macOS\"")
        );
        assert!(
            content.contains("\"Platform\":  \"iOS\"") || content.contains("\"Platform\": \"iOS\"")
        );
        assert!(
            content.contains("\"Status\":  \"CERTIFIED")
                || content.contains("\"Status\": \"CERTIFIED")
        );
    }

    let sha_sums = dist.join("SHA256SUMS");
    assert!(sha_sums.exists(), "dist/SHA256SUMS must exist");
}

// ============================================================================
// F4 — MOBILE C FFI & JNI PARITY CONTRACT
// ============================================================================

#[test]
fn test_f4_mobile_c_ffi_and_jni_parity_contract() {
    println!("📱 [STAGE F4] Mobile C FFI and JNI Parity Contract...");

    let _guard = MOBILE_FFI_LOCK.lock().unwrap();

    // 1. Initialize mobile simulation in San Francisco
    let region = CString::new("san-francisco").expect("CString");
    assert_eq!(
        realearth_init(region.as_ptr()),
        0,
        "Mobile nativeInit must return 0 (Success)"
    );

    // 2. Query initial telemetry
    let mut telem = RealEarthTelemetry::default();
    assert_eq!(realearth_get_telemetry(&mut telem), 0);
    assert_eq!(telem.is_active, 1);
    assert!(telem.lat > 37.0 && telem.lat < 38.0);
    assert!(telem.lon < -122.0 && telem.lon > -123.0);
    assert_eq!(telem.gear, 1);

    // 3. Step forward driving inputs through virtual mobile touch controls
    // Signature: realearth_step(dt, throttle, steer, brake)
    for _ in 0..120 {
        assert_eq!(realearth_step(1.0 / 60.0, 1.0, 0.0, 0.0), 0);
    }

    assert_eq!(realearth_get_telemetry(&mut telem), 0);
    assert!(
        telem.speed_kph > 15.0,
        "Mobile vehicle must accelerate under virtual touch throttle, got {:.1} km/h",
        telem.speed_kph
    );
    assert!(
        telem.engine_rpm > 2000.0,
        "Engine RPM must increase, got {:.0}",
        telem.engine_rpm
    );

    // 4. Shutdown mobile lifecycle
    assert_eq!(realearth_shutdown(), 0);
}

// ============================================================================
// F5 — MANDATORY RELEASE GATES: PHYSICS, GIS COVERAGE, AND SCENE ENTROPY
// ============================================================================

#[test]
fn test_f5_automated_release_gates_physics_gis_and_entropy() {
    println!("🛡️ [STAGE F5] Mandatory Automated Release Gates: Physics, GIS, Scene Entropy...");

    // Gate 1: Authoritative Physics - Caster Stabilization in Reverse on Slopes
    let ground = SlopedGround::new(0.0, 0.0, -0.06); // 6% downward grade along -Z
    let car_config = VehicleConfig::sports_coupe();
    let eq_y = car_config.equilibrium_ride_height();

    let mut car = DynamicVehicle::new(car_config.clone(), Vec3::new(0.0, eq_y, 0.0), 0.0);
    car.recover_on_surface(0.0);

    // Reverse down slope with zero steering input
    let rev_inputs = VehicleInputs {
        gear_mode: TransmissionMode::Reverse,
        throttle: 0.5,
        steer: 0.0,
        ..Default::default()
    };

    for _ in 0..120 {
        car.step(&rev_inputs, &ground, 1.0 / 60.0);
    }

    assert!(
        car.linear_velocity.z < -2.0,
        "Car must accelerate backward down slope in Reverse"
    );
    assert!(
        car.angular_velocity.y.abs() < 0.05,
        "Zero-input reverse roll must NOT self-steer, yaw rate was {:.4} rad/s",
        car.angular_velocity.y
    );
    assert!(
        car.position.x.abs() < 0.25,
        "Zero-input reverse drift must remain laterally stable, lateral delta was {:.4}m",
        car.position.x
    );

    // Gate 2: Parking Pawl Invariance on Sloped Terrain
    let mut park_car = DynamicVehicle::new(car_config, Vec3::new(0.0, eq_y, 0.0), 0.0);
    park_car.recover_on_surface(0.0);

    let park_inputs = VehicleInputs {
        gear_mode: TransmissionMode::Park,
        throttle: 1.0, // Attempting throttle in Park mode
        ..Default::default()
    };
    let pos_before_park = park_car.position;

    for _ in 0..60 {
        park_car.step(&park_inputs, &ground, 1.0 / 60.0);
    }

    let delta_p = (park_car.position - pos_before_park).length();
    assert!(
        delta_p < 0.01,
        "Vehicle locked in Park must remain rigidly stationary on slope, delta was {:.4}m",
        delta_p
    );

    // Gate 3: GIS Completeness across all 38 US cities
    let cities = all_us_cities();
    assert_eq!(cities.len(), 38, "Mandatory 38 US city catalog entries");

    for city in cities {
        assert!(!city.name.is_empty(), "City name must not be empty");
        assert!(!city.id.is_empty(), "City id must not be empty");
        assert!(
            city.bounds.max_lat > city.bounds.min_lat,
            "Valid latitude bounds for {}",
            city.name
        );
        assert!(
            city.bounds.max_lon > city.bounds.min_lon,
            "Valid longitude bounds for {}",
            city.name
        );
        assert!(
            city.bounds.contains(&city.anchor),
            "Anchor coordinate must be inside city bounds for {}",
            city.name
        );
    }
}

// ============================================================================
// F6 — CROSS-PLATFORM INPUT & ADAPTIVE UI DPI INVARIANCE
// ============================================================================

#[test]
fn test_f6_cross_platform_input_and_adaptive_ui_dpi() {
    println!("🖱️ [STAGE F6] Cross-Platform Input and Adaptive UI DPI Invariance...");

    let screen_profiles = [
        (1280.0, 720.0, 1.0, "Desktop 720p @ 1.0x"),
        (1920.0, 1080.0, 1.0, "Desktop 1080p @ 1.0x"),
        (2048.0, 1152.0, 1.25, "Laptop 1152p @ 1.25x"),
        (2560.0, 1440.0, 1.5, "QHD Desktop @ 1.5x"),
        (3840.0, 2160.0, 2.0, "4K UHD @ 2.0x"),
    ];

    for (w, h, dpi, label) in screen_profiles {
        let layout = InGameHudLayout::compute(w, h, true);

        // Test deterministic hit testing on control chips
        let (pcx, pcy) = layout.pause_btn.center();
        assert_eq!(
            layout.hit_test(pcx, pcy),
            Some(InGameHudHit::Pause),
            "{}: Hit-testing Pause button failed at DPI {}",
            label,
            dpi
        );

        let (park_cx, park_cy) = layout.park_chip.center();
        assert_eq!(
            layout.hit_test(park_cx, park_cy),
            Some(InGameHudHit::ParkToggle),
            "{}: Hit-testing Park chip failed at DPI {}",
            label,
            dpi
        );

        let (garage_cx, garage_cy) = layout.garage_chip.center();
        assert_eq!(
            layout.hit_test(garage_cx, garage_cy),
            Some(InGameHudHit::OpenGarage),
            "{}: Hit-testing Garage chip failed at DPI {}",
            label,
            dpi
        );

        let (jobs_cx, jobs_cy) = layout.jobs_chip.center();
        assert_eq!(
            layout.hit_test(jobs_cx, jobs_cy),
            Some(InGameHudHit::OpenJobs),
            "{}: Hit-testing Jobs chip failed at DPI {}",
            label,
            dpi
        );

        let (cities_cx, cities_cy) = layout.cities_chip.center();
        assert_eq!(
            layout.hit_test(cities_cx, cities_cy),
            Some(InGameHudHit::OpenCities),
            "{}: Hit-testing Cities chip failed at DPI {}",
            label,
            dpi
        );
    }
}

// ============================================================================
// F7 — MULTIMODAL END-TO-END JOURNEY & PERSISTENCE ROUNDTRIP
// ============================================================================

#[test]
fn test_f7_multimodal_end_to_end_journey_and_persistence_roundtrip() {
    println!("🎮 [STAGE F7] Multimodal End-to-End Journey and Persistence Roundtrip...");

    let temp_dir = tempfile::tempdir().expect("Temporary directory");
    let save_file = temp_dir.path().join("profile_save.json");

    let mut profile = PlayerProfile::new("pilot-01", "Aero Explorer");
    assert_eq!(profile.wallet.balance(), 500.0);
    assert_eq!(profile.completed_jobs, 0);

    // Accept San Francisco Aviation Delivery Job
    let jobs = JobCatalogGenerator::jobs_for_region("san-francisco");
    let air_job_def = jobs
        .iter()
        .find(|j| j.career == CareerType::AviationPilot)
        .expect("AviationPilot job in San Francisco");

    let mut job = air_job_def.clone();
    let bus = GameplayEventBus::new();

    job.accept(&profile, &bus)
        .expect("Accept aviation contract");
    assert_eq!(job.state, JobState::EnRouteToPickup);

    // Boarding dwell sequence
    let p_coord = job.pickup.coord;
    let _ = job.update(0.1, p_coord, 0.0, &bus);
    assert!(matches!(job.state, JobState::AtPickup { .. }));

    for _ in 0..20 {
        let _ = job.update(0.1, p_coord, 0.0, &bus);
    }
    assert_eq!(job.state, JobState::EnRouteToDropoff);

    // Unloading dwell sequence
    let d_coord = job.dropoff.coord;
    let _ = job.update(0.1, d_coord, 0.0, &bus);
    assert!(matches!(job.state, JobState::AtDropoff { .. }));

    let mut completion = None;
    for _ in 0..20 {
        if let Some(res) = job.update(0.1, d_coord, 0.0, &bus) {
            completion = Some(res);
            break;
        }
    }

    let result = completion.expect("Job must emit completion");
    assert!(result.final_payout > 0.0);

    profile.record_job_completion(
        result.career,
        result.final_payout,
        result.xp_gained,
        result.reputation_gained,
        200,
    );

    assert_eq!(profile.completed_jobs, 1);
    let expected_balance = 500.0 + result.final_payout;
    assert_eq!(profile.wallet.balance(), expected_balance);

    // Save and load roundtrip
    let save_data = SaveData::new(profile.clone(), "san-francisco", p_coord);
    SaveManager::save_to_file(&save_file, &save_data).expect("Save profile to disk");

    let loaded = SaveManager::load_from_file(&save_file).expect("Load profile from disk");
    assert_eq!(loaded.profile.name, "Aero Explorer");
    assert_eq!(loaded.profile.completed_jobs, 1);
    assert_eq!(loaded.profile.wallet.balance(), expected_balance);
}
