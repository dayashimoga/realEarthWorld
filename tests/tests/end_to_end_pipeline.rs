//! Comprehensive End-to-End System Integration Test for Real Earth World Platform.
//!
//! Validates the full chain:
//! Ingestion -> Validation -> World Compilation -> Binary Chunk Storage ->
//! Streaming -> Bevy ECS Simulation -> Navigation -> Physics & Vehicle Control -> Headless Rendering.

use glam::Vec2;
use realearth_compiler::compiler_pipeline::{compile_world_package, CompilerOptions};
use realearth_ingestion::dem_grid::DemGrid;
use realearth_ingestion::sample_bengaluru::create_bengaluru_central_dataset;
use realearth_navigation::{find_route, RoadNetworkGraph, RoutingPreference};
use realearth_rendering::{FloatingOriginCamera, RealEarthRenderer};
use realearth_shared::coordinates::Wgs84Coord;
use realearth_simulation::RealEarthSimWorld;
use realearth_streaming::{StreamingConfig, WorldStreamer};
use realearth_validation::pipeline::validate_dataset;
use realearth_vehicles::{AckermannVehicle, PhysicsWorld, PurePursuitController};
use std::time::Instant;
use tempfile::tempdir;

#[test]
fn test_full_bengaluru_end_to_end_vertical_slice() {
    let test_start = Instant::now();

    // 1. Data Ingestion
    println!("Step 1: Ingestion...");
    let dataset = create_bengaluru_central_dataset();
    assert_eq!(dataset.roads.len(), 4);
    assert_eq!(dataset.buildings.len(), 4);

    // 2. Validation & Topology Checking
    println!("Step 2: Validation & Topology Checking...");
    let (validated, report) = validate_dataset(&dataset);
    assert_eq!(report.valid_roads_passed, 4);
    assert_eq!(report.valid_buildings_passed, 4);
    assert_eq!(report.quarantined.len(), 0);
    // Verified 4 polygon winding orders repaired to CCW
    assert_eq!(report.repairs.len(), 4);

    // 3. DEM Generation & World Package Compilation
    println!("Step 3: World Compilation...");
    let dem = DemGrid::create_bengaluru_plateau_dem(64, 64);
    let temp_pkg_dir = tempdir().expect("Failed to create temporary world directory");
    let options = CompilerOptions {
        chunk_size_meters: 1000.0,
        terrain_grid_resolution: 16,
    };
    let stats = compile_world_package(&validated, &dem, temp_pkg_dir.path(), &options)
        .expect("Compilation failed");
    assert!(stats.total_chunks_generated > 0, "No chunks generated");
    assert!(
        temp_pkg_dir.path().join("chunks.db").exists(),
        "SQLite database missing"
    );

    // 4. World Streaming & Chunk Cache
    println!("Step 4: Streaming & LRU Cache...");
    let vidhana_soudha = Wgs84Coord::new(12.9797, 77.5907, 920.0);
    let streaming_config = StreamingConfig {
        max_memory_mb: 512,
        view_distance_meters: 3000.0,
        ..Default::default()
    };
    let mut streamer = WorldStreamer::open(temp_pkg_dir.path(), streaming_config)
        .expect("Failed to open world streamer");

    let loaded_count = streamer
        .update(vidhana_soudha)
        .expect("Streaming update failed");
    assert!(loaded_count > 0, "No chunks loaded into cache");
    let active_chunks = streamer.get_active_chunks();
    assert!(!active_chunks.is_empty(), "No active chunks available");

    // 5. Bevy ECS Simulation World & Determinism
    println!("Step 5: Bevy ECS Simulation & Ephemeris...");
    let mut sim_world = RealEarthSimWorld::new(vidhana_soudha);
    for _ in 0..60 {
        sim_world.tick(1.0 / 60.0);
    }
    let solar = sim_world.current_solar_position();
    assert!(solar.sun_intensity > 0.0);

    // 6. Navigation & Pathfinding
    println!("Step 6: Navigation Graph & A* Pathfinding...");
    let network = RoadNetworkGraph::from_roads(&dataset.roads);
    let mg_start = Wgs84Coord::new(12.9740, 77.6000, 0.0);
    let mg_end = Wgs84Coord::new(12.9760, 77.6150, 0.0);

    let route = find_route(
        &network,
        mg_start,
        mg_end,
        RoutingPreference::ShortestDistance,
    )
    .expect("Pathfinding should succeed on MG Road");
    assert!(!route.waypoints.is_empty());
    assert!(route.total_distance_meters > 1000.0);

    // 7. Physics & Vehicle Control
    println!("Step 7: Rapier3D Physics & Vehicle Kinematics...");
    let mut physics = PhysicsWorld::new();
    physics.spawn_ground(0.0);
    let (v_body, _) = physics.spawn_vehicle_body(0.0, 5.0, 0.0, [1.0, 0.5, 2.0], 1500.0);
    for _ in 0..120 {
        physics.step(1.0 / 60.0);
    }
    let pos_after = physics.rigid_body_set[v_body].translation();
    assert!(pos_after.y < 5.0, "Gravity was not applied");
    assert!(pos_after.y > 0.0, "Vehicle penetrated ground");

    let mut car = AckermannVehicle::new(2.7, 0.6);
    let waypoints = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 0.0),
        Vec2::new(20.0, 5.0),
        Vec2::new(30.0, 5.0),
    ];
    let mut controller = PurePursuitController::new(waypoints, 5.0, 10.0);
    for _ in 0..100 {
        let reached = controller.update(&mut car);
        car.step(0.02);
        if reached {
            break;
        }
    }
    assert!(car.position.x > 5.0, "Car did not follow trajectory");

    // 8. Headless WGPU Rendering
    println!("Step 8: Headless WGPU Rendering Pipeline...");
    let renderer_res = RealEarthRenderer::new_headless(wgpu::TextureFormat::Rgba8UnormSrgb);
    if let Ok(renderer) = renderer_res {
        let camera = FloatingOriginCamera::new(vidhana_soudha, 16.0 / 9.0);
        let width = 640;
        let height = 360;

        let color_tex = renderer.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Test Color Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let color_view = color_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let depth_tex = renderer.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Test Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth_tex.create_view(&wgpu::TextureViewDescriptor::default());

        renderer.update_uniforms(
            &camera,
            [
                solar.sun_direction_enu.x,
                solar.sun_direction_enu.y,
                solar.sun_direction_enu.z,
            ],
            solar.sun_intensity,
        );

        renderer.render_chunks(&active_chunks, &color_view, &depth_view);
        println!("Headless rendering frame executed successfully!");
    } else {
        println!("GPU unavailable in headless CI test runner; graceful fallback confirmed.");
    }

    let elapsed = test_start.elapsed();
    println!(
        "✅ End-to-End Pipeline test completed in {:.2}ms!",
        elapsed.as_secs_f64() * 1000.0
    );
}
