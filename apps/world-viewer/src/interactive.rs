//! Interactive desktop windowing, live wgpu surface presentation, and navigation loop.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes, WindowId};

use crate::camera_controller::{CameraController, CameraMode, Landmark};
use crate::screenshot::capture_texture_to_ppm;
use realearth_compiler::compiler_pipeline::{compile_world_package, CompilerOptions};
use realearth_gameplay::{
    save::CURRENT_SAVE_SCHEMA_VERSION, CareerType, GameSettings, GameplayEventBus,
    GameplayHudModel, Job, JobCatalogGenerator, PlayerProfile, SaveData, SaveManager,
};
use realearth_ingestion::dem_grid::DemGrid;
use realearth_ingestion::load_authentic_city;
use realearth_ingestion::sample_bengaluru::create_bengaluru_central_dataset;
use realearth_ingestion::sample_chennai::create_chennai_dataset;
use realearth_ingestion::sample_chicago::create_chicago_dataset;
use realearth_ingestion::sample_corridors::create_intercity_corridors_dataset;
use realearth_ingestion::sample_hyderabad::create_hyderabad_dataset;
use realearth_ingestion::sample_los_angeles::create_los_angeles_dataset;
use realearth_ingestion::sample_miami::create_miami_dataset;
use realearth_ingestion::sample_mysuru::create_mysuru_dataset;
use realearth_ingestion::sample_new_york::create_new_york_dataset;
use realearth_ingestion::sample_san_francisco::create_san_francisco_dataset;
use realearth_ingestion::sample_seattle::create_seattle_dataset;
use realearth_navigation::{RoadNetworkGraph, RuleOfTheRoad};
use realearth_rendering::{
    generate_vehicle_mesh, FloatingOriginCamera, GraphicsPresetLevel, GraphicsSettings, HudMesh,
    RealEarthRenderer, VehicleMeshParams, VehicleVisualType,
};
use realearth_shared::coordinates::Wgs84Coord;
use realearth_shared::metrics::get_current_process_memory;
use realearth_shared::us_catalog::find_city_by_id;
use realearth_simulation::RealEarthSimWorld;
use realearth_streaming::{StreamingConfig, WorldStreamer};
use realearth_validation::pipeline::validate_dataset;
use realearth_vehicles::{
    DynamicVehicle, GroundQuery, TrafficSimulation, VehicleCameraController,
    VehicleCameraPerspective, VehicleConfig, VehicleInputs,
};

/// Ground elevation query backed by geographic DEM bilinear interpolation.
#[derive(Clone)]
pub struct DemGroundQuery {
    pub dem: DemGrid,
    pub anchor: Wgs84Coord,
    pub meters_per_deg_lat: f64,
    pub meters_per_deg_lon: f64,
}

impl DemGroundQuery {
    pub fn new(dem: DemGrid, anchor: Wgs84Coord) -> Self {
        let meters_per_deg_lat = 111_320.0;
        let meters_per_deg_lon = 111_320.0 * anchor.lat.to_radians().cos().abs().max(0.01);
        Self {
            dem,
            anchor,
            meters_per_deg_lat,
            meters_per_deg_lon,
        }
    }

    pub fn elevation(&self, x: f32, z: f32) -> f32 {
        let lat = self.anchor.lat + (z as f64 / self.meters_per_deg_lat);
        let lon = self.anchor.lon + (x as f64 / self.meters_per_deg_lon);
        let coord = Wgs84Coord::new(lat, lon, 0.0);
        self.dem.sample_bilinear(&coord)
    }

    /// Evaluates the true terrain surface normal using analytical gradient differences.
    pub fn normal(&self, x: f32, z: f32) -> glam::Vec3 {
        let eps = 0.5;
        let h_r = self.elevation(x + eps, z);
        let h_l = self.elevation(x - eps, z);
        let h_u = self.elevation(x, z + eps);
        let h_d = self.elevation(x, z - eps);

        let slope_x = (h_r - h_l) / (2.0 * eps);
        let slope_z = (h_u - h_d) / (2.0 * eps);

        glam::Vec3::new(-slope_x, 1.0, -slope_z).normalize()
    }
}

impl GroundQuery for DemGroundQuery {
    fn query_ground_contact(&self, origin: glam::Vec3, max_dist: f32) -> Option<(f32, glam::Vec3)> {
        let ground_y = self.elevation(origin.x, origin.z);
        let dist = origin.y - ground_y;
        let normal = self.normal(origin.x, origin.z);
        if (0.0..=max_dist).contains(&dist) {
            Some((dist, normal))
        } else if dist < 0.0 && dist > -10.0 {
            // Wheels penetrating surface: provide full contact to arrest sinking
            Some((0.0, normal))
        } else {
            None
        }
    }
}

/// Configuration options passed when launching the interactive viewer.
#[derive(Debug, Clone)]
pub struct ViewerLaunchConfig {
    pub region_id: String,
    pub initial_mode: CameraMode,
    pub is_drive: bool,
    pub preset_level: GraphicsPresetLevel,
    pub traffic_count: usize,
    pub vehicle_cam_perspective: VehicleCameraPerspective,
    pub career_arg: Option<String>,
    pub job_id_arg: Option<String>,
    pub target_location: Option<String>,
}

impl Default for ViewerLaunchConfig {
    fn default() -> Self {
        Self {
            region_id: "san-francisco".to_string(),
            initial_mode: CameraMode::Fly,
            is_drive: false,
            preset_level: GraphicsPresetLevel::High,
            traffic_count: 8,
            vehicle_cam_perspective: VehicleCameraPerspective::Chase,
            career_arg: None,
            job_id_arg: None,
            target_location: None,
        }
    }
}

/// Loaded regional dataset and simulation assets.
struct RegionResources {
    _world_dir: PathBuf,
    region_name: &'static str,
    landmark: Landmark,
    dem: DemGrid,
    _raw_roads: Vec<realearth_ingestion::osm_types::OsmRoadFeature>,
    streamer: WorldStreamer,
}

/// Persistent winit + wgpu interactive surface viewer application.
pub struct InteractiveViewerApp {
    config: ViewerLaunchConfig,
    window: Option<Arc<Window>>,
    surface: Option<wgpu::Surface<'static>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    renderer: Option<RealEarthRenderer>,
    depth_texture: Option<wgpu::Texture>,
    depth_view: Option<wgpu::TextureView>,
    is_minimized: bool,

    // Regional world and streaming state
    region_resources: Option<RegionResources>,
    sim_world: Option<RealEarthSimWorld>,
    ground_query: Option<DemGroundQuery>,
    camera_ctrl: CameraController,
    camera: FloatingOriginCamera,

    // Driving, traffic, and gameplay state
    vehicle: DynamicVehicle,
    vehicle_cam: VehicleCameraController,
    traffic_sim: TrafficSimulation,
    vehicle_visual_type: VehicleVisualType,
    gameplay_bus: GameplayEventBus,
    player_profile: PlayerProfile,
    active_job: Option<Job>,

    // Interactive inputs & metrics
    pressed_keys: HashSet<KeyCode>,
    mouse_drag_active: bool,
    last_cursor_pos: Option<(f64, f64)>,
    last_frame: Instant,
    frame_durations_ms: Vec<f64>,
    total_rendered_frames: u64,
    last_culled_chunks: usize,
    last_culled_triangles: usize,
    show_hud: bool,
    status_notification: Option<(String, Instant)>,
    region_cycle: Vec<&'static str>,
    region_cycle_idx: usize,
}

/// Resolves the storage location for compiled region chunks, supporting bundled packages,
/// standalone deployment directories, developer builds, and persistent user caches.
pub fn resolve_world_package_dir(slug: &str) -> PathBuf {
    // 1. Check if bundled in data/world_<slug> (distribution package)
    let bundled = PathBuf::from(format!("data/world_{}", slug));
    if bundled.join("chunks.db").exists() {
        return bundled;
    }
    // 2. Check next to executable: <exe_dir>/data/world_<slug>
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let exe_bundled = exe_dir.join(format!("data/world_{}", slug));
            if exe_bundled.join("chunks.db").exists() {
                return exe_bundled;
            }
        }
    }
    // 3. If developer repository target/ exists, use target/world_<slug>
    if std::path::Path::new("target").exists() {
        return PathBuf::from(format!("target/world_{}", slug));
    }
    // 4. Default runtime cache directory in data/cache/world_<slug>
    PathBuf::from(format!("data/cache/world_{}", slug))
}

impl InteractiveViewerApp {
    pub fn new(config: ViewerLaunchConfig) -> Self {
        let initial_landmark = Landmark::SAN_FRANCISCO_TRANSAMERICA;
        let mut camera_ctrl = CameraController::new(initial_landmark.coord);
        camera_ctrl.mode = config.initial_mode;
        camera_ctrl.yaw = initial_landmark.yaw;
        camera_ctrl.pitch = initial_landmark.pitch;

        let camera = FloatingOriginCamera::new(initial_landmark.coord, 16.0 / 9.0);
        let vehicle = DynamicVehicle::new(
            VehicleConfig::default(),
            glam::Vec3::new(0.0, 10.0, 0.0),
            initial_landmark.yaw,
        );
        let vehicle_cam = VehicleCameraController::new(config.vehicle_cam_perspective);

        let gameplay_bus = GameplayEventBus::new();
        let mut player_profile = PlayerProfile::new("driver-01", "Planetary Transporter");
        player_profile.add_reputation(CareerType::HeavyFreight, 500);

        let region_cycle = vec![
            "san-francisco",
            "new-york",
            "los-angeles",
            "chicago",
            "seattle",
            "miami",
            "bengaluru",
            "mysuru",
            "chennai",
            "hyderabad",
        ];

        let region_cycle_idx = region_cycle
            .iter()
            .position(|r| *r == config.region_id)
            .unwrap_or(0);

        Self {
            config,
            window: None,
            surface: None,
            surface_config: None,
            renderer: None,
            depth_texture: None,
            depth_view: None,
            is_minimized: false,

            region_resources: None,
            sim_world: None,
            ground_query: None,
            camera_ctrl,
            camera,

            vehicle,
            vehicle_cam,
            traffic_sim: TrafficSimulation::new(),
            vehicle_visual_type: VehicleVisualType::SportsCoupe,
            gameplay_bus,
            player_profile,
            active_job: None,

            pressed_keys: HashSet::new(),
            mouse_drag_active: false,
            last_cursor_pos: None,
            last_frame: Instant::now(),
            frame_durations_ms: Vec::with_capacity(120),
            total_rendered_frames: 0,
            last_culled_chunks: 0,
            last_culled_triangles: 0,
            show_hud: true,
            status_notification: Some((
                "Welcome to Real Earth World! Press [H] for HUD, [V] for Drivable Car".to_string(),
                Instant::now(),
            )),
            region_cycle,
            region_cycle_idx,
        }
    }

    /// Loads or hot-reloads region geographic assets, DEM, and streaming database.
    fn load_region(&mut self, region_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let preset = self.config.preset_level;
        let streaming_config = StreamingConfig {
            max_memory_mb: 1024,
            max_gpu_memory_mb: 512,
            view_distance_meters: match preset {
                GraphicsPresetLevel::Low => 1500.0,
                GraphicsPresetLevel::Medium => 2500.0,
                GraphicsPresetLevel::High | GraphicsPresetLevel::Adaptive => 4000.0,
            },
            lod1_distance_meters: match preset {
                GraphicsPresetLevel::Low => 800.0,
                GraphicsPresetLevel::Medium => 1500.0,
                GraphicsPresetLevel::High | GraphicsPresetLevel::Adaptive => 2200.0,
            },
            lod_hysteresis_meters: 100.0,
            lookahead_seconds: 2.0,
            collision_critical_radius_meters: 250.0,
            ..Default::default()
        };

        let (world_dir, region_name, landmark, dem, raw_roads) = if let Some(city) =
            find_city_by_id(region_id)
        {
            let (raw, dem, meta) = load_authentic_city(city.id)?;
            let dir = resolve_world_package_dir(&meta.region_id.replace('-', "_"));
            let landmark = Landmark::from_us_city(city);
            if !dir.join("chunks.db").exists() {
                println!(
                    "📦 Compiling authoritative {} ({}) package [DEM accuracy: ±{:.1}m, Source SHA256: {}]...",
                    meta.name, meta.state, meta.dem_accuracy_m, &meta.source_osm_sha256[..8]
                );
                let (val, _) = validate_dataset(&raw);
                let options = CompilerOptions {
                    chunk_size_meters: 500.0,
                    terrain_grid_resolution: 16,
                    package_name: format!("{}_package", meta.region_id),
                    ..Default::default()
                };
                compile_world_package(&val, &dem, &dir, &options)?;
            }
            (dir, city.name, landmark, dem, raw.roads)
        } else if region_id.contains("mysuru") {
            let dir = resolve_world_package_dir("mysuru");
            let dem = DemGrid::create_mysuru_dem(64, 64);
            let raw = create_mysuru_dataset();
            if !dir.join("chunks.db").exists() {
                println!("📦 Compiling cached Mysuru Urban package...");
                let (val, _) = validate_dataset(&raw);
                let options = CompilerOptions {
                    chunk_size_meters: 1000.0,
                    terrain_grid_resolution: 16,
                    ..Default::default()
                };
                compile_world_package(&val, &dem, &dir, &options)?;
            }
            (
                dir,
                "Mysuru Urban (IN)",
                Landmark::MYSURU_PALACE,
                dem,
                raw.roads,
            )
        } else if region_id.contains("chennai") {
            let dir = resolve_world_package_dir("chennai");
            let dem = DemGrid::create_chennai_coastal_dem(64, 64);
            let raw = create_chennai_dataset();
            if !dir.join("chunks.db").exists() {
                println!("📦 Compiling cached Chennai Central package...");
                let (val, _) = validate_dataset(&raw);
                let options = CompilerOptions {
                    chunk_size_meters: 1000.0,
                    terrain_grid_resolution: 16,
                    ..Default::default()
                };
                compile_world_package(&val, &dem, &dir, &options)?;
            }
            (
                dir,
                "Chennai Central (IN)",
                Landmark::CHENNAI_CENTRAL,
                dem,
                raw.roads,
            )
        } else if region_id.contains("hyderabad") {
            let dir = resolve_world_package_dir("hyderabad");
            let dem = DemGrid::create_hyderabad_plateau_dem(64, 64);
            let raw = create_hyderabad_dataset();
            if !dir.join("chunks.db").exists() {
                println!("📦 Compiling cached Hyderabad Central package...");
                let (val, _) = validate_dataset(&raw);
                let options = CompilerOptions {
                    chunk_size_meters: 1000.0,
                    terrain_grid_resolution: 16,
                    ..Default::default()
                };
                compile_world_package(&val, &dem, &dir, &options)?;
            }
            (
                dir,
                "Hyderabad Central (IN)",
                Landmark::HYDERABAD_CHARMINAR,
                dem,
                raw.roads,
            )
        } else if region_id.contains("corridor") {
            let dir = resolve_world_package_dir("corridors");
            let dem = DemGrid::create_intercity_corridors_dem(64, 64);
            let raw = create_intercity_corridors_dataset();
            if !dir.join("chunks.db").exists() {
                println!("📦 Compiling cached Intercity Corridors package...");
                let (val, _) = validate_dataset(&raw);
                let options = CompilerOptions {
                    chunk_size_meters: 2000.0,
                    terrain_grid_resolution: 16,
                    ..Default::default()
                };
                compile_world_package(&val, &dem, &dir, &options)?;
            }
            (
                dir,
                "South India Intercity Corridors",
                Landmark::CORRIDOR_KAVERI_BRIDGE,
                dem,
                raw.roads,
            )
        } else if region_id.contains("bengaluru") || region_id == "in" {
            let dir = resolve_world_package_dir("bengaluru");
            let dem = DemGrid::create_bengaluru_plateau_dem(64, 64);
            let raw = create_bengaluru_central_dataset();
            if !dir.join("chunks.db").exists() {
                println!("📦 Compiling cached Bengaluru Central package...");
                let (val, _) = validate_dataset(&raw);
                let options = CompilerOptions {
                    chunk_size_meters: 1000.0,
                    terrain_grid_resolution: 16,
                    ..Default::default()
                };
                compile_world_package(&val, &dem, &dir, &options)?;
            }
            (
                dir,
                "Bengaluru Central (IN)",
                Landmark::BENGALURU_VIDHANA_SOUDHA,
                dem,
                raw.roads,
            )
        } else if region_id.contains("nyc") || region_id.contains("new-york") {
            let dir = resolve_world_package_dir("new_york");
            let dem = DemGrid::create_new_york_coastal_dem(64, 64);
            let raw = create_new_york_dataset();
            if !dir.join("chunks.db").exists() {
                println!("📦 Compiling cached New York Lower Manhattan package...");
                let (val, _) = validate_dataset(&raw);
                let options = CompilerOptions {
                    chunk_size_meters: 500.0,
                    terrain_grid_resolution: 16,
                    ..Default::default()
                };
                compile_world_package(&val, &dem, &dir, &options)?;
            }
            (
                dir,
                "New York Lower Manhattan (US)",
                Landmark::NEW_YORK_ONE_WTC,
                dem,
                raw.roads,
            )
        } else if region_id.contains("la") || region_id.contains("los-angeles") {
            let dir = resolve_world_package_dir("los_angeles");
            let dem = DemGrid::create_los_angeles_basin_dem(64, 64);
            let raw = create_los_angeles_dataset();
            if !dir.join("chunks.db").exists() {
                println!("📦 Compiling cached Los Angeles Downtown package...");
                let (val, _) = validate_dataset(&raw);
                let options = CompilerOptions {
                    chunk_size_meters: 500.0,
                    terrain_grid_resolution: 16,
                    ..Default::default()
                };
                compile_world_package(&val, &dem, &dir, &options)?;
            }
            (
                dir,
                "Los Angeles Downtown (US)",
                Landmark::LOS_ANGELES_WILSHIRE_GRAND,
                dem,
                raw.roads,
            )
        } else if region_id.contains("chicago") {
            let dir = resolve_world_package_dir("chicago");
            let dem = DemGrid::create_chicago_loop_dem(64, 64);
            let raw = create_chicago_dataset();
            if !dir.join("chunks.db").exists() {
                println!("📦 Compiling cached Chicago The Loop package...");
                let (val, _) = validate_dataset(&raw);
                let options = CompilerOptions {
                    chunk_size_meters: 500.0,
                    terrain_grid_resolution: 16,
                    ..Default::default()
                };
                compile_world_package(&val, &dem, &dir, &options)?;
            }
            (
                dir,
                "Chicago The Loop (US)",
                Landmark::CHICAGO_WILLIS_TOWER,
                dem,
                raw.roads,
            )
        } else if region_id.contains("seattle") {
            let dir = resolve_world_package_dir("seattle");
            let dem = DemGrid::create_seattle_coastal_dem(64, 64);
            let raw = create_seattle_dataset();
            if !dir.join("chunks.db").exists() {
                println!("📦 Compiling cached Seattle Waterfront package...");
                let (val, _) = validate_dataset(&raw);
                let options = CompilerOptions {
                    chunk_size_meters: 500.0,
                    terrain_grid_resolution: 16,
                    ..Default::default()
                };
                compile_world_package(&val, &dem, &dir, &options)?;
            }
            (
                dir,
                "Seattle Waterfront (US)",
                Landmark::SEATTLE_SPACE_NEEDLE,
                dem,
                raw.roads,
            )
        } else if region_id.contains("miami") {
            let dir = resolve_world_package_dir("miami");
            let dem = DemGrid::create_miami_coastal_dem(64, 64);
            let raw = create_miami_dataset();
            if !dir.join("chunks.db").exists() {
                println!("📦 Compiling cached Miami Brickell package...");
                let (val, _) = validate_dataset(&raw);
                let options = CompilerOptions {
                    chunk_size_meters: 500.0,
                    terrain_grid_resolution: 16,
                    ..Default::default()
                };
                compile_world_package(&val, &dem, &dir, &options)?;
            }
            (
                dir,
                "Miami Brickell & Biscayne (US)",
                Landmark::MIAMI_BRICKELL_PANORAMA,
                dem,
                raw.roads,
            )
        } else {
            let dir = resolve_world_package_dir("san_francisco");
            let dem = DemGrid::create_san_francisco_coastal_dem(64, 64);
            let raw = create_san_francisco_dataset();
            if !dir.join("chunks.db").exists() {
                println!("📦 Compiling cached San Francisco Downtown package...");
                let (val, _) = validate_dataset(&raw);
                let options = CompilerOptions {
                    chunk_size_meters: 500.0,
                    terrain_grid_resolution: 16,
                    ..Default::default()
                };
                compile_world_package(&val, &dem, &dir, &options)?;
            }
            (
                dir,
                "San Francisco Downtown & Waterfront (US)",
                Landmark::SAN_FRANCISCO_TRANSAMERICA,
                dem,
                raw.roads,
            )
        };

        let mut streamer = WorldStreamer::open(&world_dir, streaming_config)?;
        let loaded = streamer.update(landmark.coord)?;
        let active_chunks = streamer.get_active_chunks();
        println!(
            "🌍 Region Loaded: {} | Initial Chunks: {} ({} active in RAM)",
            region_name,
            loaded,
            active_chunks.len()
        );

        let ground_query = DemGroundQuery::new(dem.clone(), landmark.coord);

        // Initialize Bevy ECS simulation world
        let mut sim_world = RealEarthSimWorld::new(landmark.coord);
        for chunk in &active_chunks {
            let center = glam::Vec3::new(
                0.0,
                (chunk.header.min_elevation + chunk.header.max_elevation) * 0.5,
                0.0,
            );
            sim_world.spawn_chunk_entity(
                chunk.header.chunk_x,
                chunk.header.chunk_y,
                chunk.header.lod_level,
                center,
                750.0,
                !chunk.terrain.is_empty(),
                !chunk.roads.is_empty(),
                !chunk.buildings.is_empty(),
                !chunk.water.is_empty(),
                !chunk.vegetation.is_empty(),
            );
        }

        // Initialize vehicle at ground surface
        let initial_elevation = ground_query.elevation(0.0, 0.0);
        self.vehicle = DynamicVehicle::new(
            VehicleConfig::default(),
            glam::Vec3::new(0.0, initial_elevation, 0.0),
            landmark.yaw,
        );
        self.vehicle.recover_on_surface(initial_elevation);

        // Populate traffic fleet
        self.traffic_sim = TrafficSimulation::new();
        if self.config.traffic_count > 0 && !raw_roads.is_empty() {
            let is_left_hand = matches!(
                region_id,
                "bengaluru" | "mysuru" | "chennai" | "hyderabad" | "corridors" | "in"
            );
            let rule = if is_left_hand {
                RuleOfTheRoad::DriveOnLeft
            } else {
                RuleOfTheRoad::DriveOnRight
            };
            let road_graph = RoadNetworkGraph::from_roads(&raw_roads);
            self.traffic_sim.populate_from_road_network(
                &road_graph,
                self.config.traffic_count,
                rule,
            );
        }

        // Accept career job if configured
        let regional_jobs = JobCatalogGenerator::jobs_for_region(region_id);
        self.active_job = if let Some(ref jid) = self.config.job_id_arg {
            regional_jobs.iter().find(|j| j.id == *jid).cloned()
        } else if let Some(ref car) = self.config.career_arg {
            let career_type = match car.as_str() {
                "taxi" => CareerType::TaxiPassenger,
                "delivery" => CareerType::PackageDelivery,
                "freight" => CareerType::HeavyFreight,
                _ => CareerType::FreeRoam,
            };
            regional_jobs
                .iter()
                .find(|j| j.career == career_type)
                .cloned()
        } else if self.config.is_drive {
            regional_jobs
                .iter()
                .find(|j| j.career == CareerType::TaxiPassenger)
                .cloned()
        } else {
            None
        };

        if let Some(ref mut job) = self.active_job {
            let _ = job.accept(&self.player_profile, &self.gameplay_bus);
        }

        // Reposition camera to landmark
        self.camera_ctrl.coord = landmark.coord;
        self.camera_ctrl.yaw = landmark.yaw;
        self.camera_ctrl.pitch = landmark.pitch;
        if let Some(first_chunk) = active_chunks.first() {
            self.camera.set_anchor(first_chunk.header.anchor);
        }
        self.camera_ctrl.update_camera(&mut self.camera);

        self.ground_query = Some(ground_query);
        self.sim_world = Some(sim_world);
        self.region_resources = Some(RegionResources {
            _world_dir: world_dir,
            region_name,
            landmark,
            dem,
            _raw_roads: raw_roads,
            streamer,
        });

        self.status_notification = Some((
            format!("📍 Region: {} loaded at {}", region_name, landmark.name),
            Instant::now(),
        ));

        Ok(())
    }

    /// Captures the currently rendered visible frame buffer and saves it to a binary PPM file.
    pub fn capture_screenshot(&mut self) {
        if let (Some(renderer), Some(window)) = (&self.renderer, &self.window) {
            let size = window.inner_size();
            if size.width == 0 || size.height == 0 {
                return;
            }

            let copy_tex = renderer.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Interactive Screenshot Copy Texture"),
                size: wgpu::Extent3d {
                    width: size.width,
                    height: size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: renderer.surface_format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let copy_view = copy_tex.create_view(&wgpu::TextureViewDescriptor::default());

            if let Some(ref mut res) = self.region_resources {
                let active = res.streamer.get_active_chunks();
                if let Some(ref depth_view) = self.depth_view {
                    renderer.render_chunks_culled_cached(
                        &active,
                        &self.camera,
                        &copy_view,
                        depth_view,
                    );
                }
            }

            let path = PathBuf::from("target/screenshots").join(format!(
                "{}_interactive_{}.ppm",
                self.config.region_id,
                Instant::now().elapsed().as_millis()
            ));

            if let Err(e) = capture_texture_to_ppm(
                &renderer.device,
                &renderer.queue,
                &copy_tex,
                size.width,
                size.height,
                &path,
            ) {
                eprintln!("⚠️ Screenshot capture error: {}", e);
            } else {
                println!("📸 Visible surface screenshot saved: {:?}", path);
                self.status_notification =
                    Some((format!("📸 Screenshot saved to {:?}", path), Instant::now()));
            }
        }
    }
}

impl ApplicationHandler for InteractiveViewerApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let win_attrs = WindowAttributes::default()
                .with_title("Real Earth World — Navigable Planetary Simulation")
                .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0))
                .with_min_inner_size(winit::dpi::LogicalSize::new(640.0, 360.0));

            let window = match event_loop.create_window(win_attrs) {
                Ok(w) => Arc::new(w),
                Err(e) => {
                    eprintln!("⚠️ Failed to create native display window: {}", e);
                    event_loop.exit();
                    return;
                }
            };
            self.window = Some(window.clone());

            // 1. Initialize wgpu Surface & Adapter
            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends: wgpu::Backends::all(),
                ..Default::default()
            });

            let surface = match instance.create_surface(window.clone()) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("⚠️ Failed to create wgpu window surface: {}", e);
                    event_loop.exit();
                    return;
                }
            };

            let adapter =
                pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                }))
                .unwrap_or_else(|| {
                    pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::LowPower,
                        compatible_surface: Some(&surface),
                        force_fallback_adapter: true,
                    }))
                    .expect("Failed to acquire any suitable graphics adapter for window surface")
                });

            let (device, queue) = pollster::block_on(adapter.request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("RealEarth Interactive Surface Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            ))
            .expect("Failed to create wgpu device");

            let caps = surface.get_capabilities(&adapter);
            let surface_format = caps
                .formats
                .iter()
                .copied()
                .find(|f| f.is_srgb())
                .unwrap_or(caps.formats[0]);

            let physical_size = window.inner_size();
            let width = physical_size.width.max(1);
            let height = physical_size.height.max(1);

            let surface_config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width,
                height,
                present_mode: wgpu::PresentMode::AutoVsync,
                alpha_mode: caps.alpha_modes[0],
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };
            surface.configure(&device, &surface_config);

            // 2. Create Depth Texture
            let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Surface Depth Texture"),
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
            let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

            // 3. Initialize RealEarthRenderer
            let graphics_settings = GraphicsSettings::from_preset(self.config.preset_level);
            let renderer =
                RealEarthRenderer::from_device(device, queue, surface_format, graphics_settings);

            self.surface = Some(surface);
            self.surface_config = Some(surface_config);
            self.renderer = Some(renderer);
            self.depth_texture = Some(depth_texture);
            self.depth_view = Some(depth_view);

            // 4. Load initial region
            let region_to_load = self.config.region_id.clone();
            if let Err(e) = self.load_region(&region_to_load) {
                eprintln!("⚠️ Failed to initialize region {}: {}", region_to_load, e);
            }

            self.last_frame = Instant::now();
            println!(
                "🖥️ Native wgpu surface initialized successfully: {}x{} ({:?})",
                width, height, surface_format
            );
            println!("🎮 Controls: [W/S/A/D] Move/Drive, [Space/C] Up/Down, [Mouse/Arrows] Look, [V] Drive Mode, [Tab] Walk/Fly, [P] Preset, [R] Region, [1-9] Landmarks, [F12] Screenshot, [Esc] Exit");

            window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("👋 Window close requested. Exiting cleanly.");
                event_loop.exit();
            }

            WindowEvent::Resized(physical_size) => {
                if physical_size.width == 0 || physical_size.height == 0 {
                    self.is_minimized = true;
                    return;
                }
                self.is_minimized = false;

                if let (Some(ref mut config), Some(ref surface), Some(ref renderer)) =
                    (&mut self.surface_config, &self.surface, &self.renderer)
                {
                    config.width = physical_size.width;
                    config.height = physical_size.height;
                    surface.configure(&renderer.device, config);

                    let depth_tex = renderer.device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("Surface Depth Texture"),
                        size: wgpu::Extent3d {
                            width: physical_size.width,
                            height: physical_size.height,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Depth32Float,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        view_formats: &[],
                    });
                    self.depth_view =
                        Some(depth_tex.create_view(&wgpu::TextureViewDescriptor::default()));
                    self.depth_texture = Some(depth_tex);
                    self.camera.aspect = physical_size.width as f32 / physical_size.height as f32;

                    if let Some(ref window) = self.window {
                        window.request_redraw();
                    }
                }
            }

            WindowEvent::ScaleFactorChanged { .. } => {
                if let Some(ref window) = self.window {
                    let size = window.inner_size();
                    if size.width == 0 || size.height == 0 {
                        self.is_minimized = true;
                        return;
                    }
                    self.is_minimized = false;

                    if let (Some(ref mut config), Some(ref surface), Some(ref renderer)) =
                        (&mut self.surface_config, &self.surface, &self.renderer)
                    {
                        config.width = size.width;
                        config.height = size.height;
                        surface.configure(&renderer.device, config);

                        let depth_tex = renderer.device.create_texture(&wgpu::TextureDescriptor {
                            label: Some("Surface Depth Texture"),
                            size: wgpu::Extent3d {
                                width: size.width,
                                height: size.height,
                                depth_or_array_layers: 1,
                            },
                            mip_level_count: 1,
                            sample_count: 1,
                            dimension: wgpu::TextureDimension::D2,
                            format: wgpu::TextureFormat::Depth32Float,
                            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                            view_formats: &[],
                        });
                        self.depth_view =
                            Some(depth_tex.create_view(&wgpu::TextureViewDescriptor::default()));
                        self.depth_texture = Some(depth_tex);
                        self.camera.aspect = size.width as f32 / size.height as f32;
                        window.request_redraw();
                    }
                }
            }

            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Right || button == MouseButton::Left {
                    self.mouse_drag_active = state == ElementState::Pressed;
                    if !self.mouse_drag_active {
                        self.last_cursor_pos = None;
                    }
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                if self.mouse_drag_active {
                    if let Some((prev_x, prev_y)) = self.last_cursor_pos {
                        let dx = (position.x - prev_x) as f32;
                        let dy = (position.y - prev_y) as f32;
                        let sensitivity = 0.0035;
                        self.camera_ctrl
                            .rotate(-dx * sensitivity, -dy * sensitivity);
                    }
                    self.last_cursor_pos = Some((position.x, position.y));
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let scroll_y = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y as f64 * 8.0,
                    MouseScrollDelta::PixelDelta(pos) => pos.y * 0.1,
                };
                if self.camera_ctrl.mode == CameraMode::Orbit {
                    self.camera_ctrl.orbit_zoom(
                        -scroll_y * 4.0,
                        self.region_resources.as_ref().map(|r| &r.dem),
                    );
                } else if self.camera_ctrl.mode == CameraMode::Fly
                    || self.camera_ctrl.mode == CameraMode::Free
                {
                    self.camera_ctrl.move_vertical(scroll_y);
                }
            }

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state,
                        ..
                    },
                ..
            } => match state {
                ElementState::Pressed => {
                    self.pressed_keys.insert(key);
                    match key {
                        KeyCode::Escape => {
                            println!("Escape key pressed: exiting Real Earth viewer.");
                            event_loop.exit();
                        }

                        // Cycle Camera Mode (Fly -> Walk -> Orbit -> Free -> Chase -> Fly)
                        KeyCode::Tab => {
                            let new_mode = self.camera_ctrl.cycle_mode();
                            let label = format!("Camera Navigation Mode: {:?}", new_mode);
                            println!("🎥 {}", label);
                            self.status_notification = Some((label, Instant::now()));
                        }

                        // Instant Camera Reset to Anchor Landmark (Home key)
                        KeyCode::Home => {
                            self.camera_ctrl.reset_to_default();
                            let label = "Camera Reset to Anchor Landmark".to_string();
                            println!("🎯 {}", label);
                            self.status_notification = Some((label, Instant::now()));
                        }

                        // Increase Camera Movement Speed ([ / PageUp / Equal)
                        KeyCode::BracketRight | KeyCode::Equal | KeyCode::PageUp => {
                            let spd = self.camera_ctrl.increase_speed();
                            let label = format!(
                                "Camera Speed: {:.1} m/s ({:.1}x)",
                                spd, self.camera_ctrl.speed_multiplier
                            );
                            println!("⏩ {}", label);
                            self.status_notification = Some((label, Instant::now()));
                        }

                        // Decrease Camera Movement Speed (] / PageDown / Minus)
                        KeyCode::BracketLeft | KeyCode::Minus | KeyCode::PageDown => {
                            let spd = self.camera_ctrl.decrease_speed();
                            let label = format!(
                                "Camera Speed: {:.1} m/s ({:.1}x)",
                                spd, self.camera_ctrl.speed_multiplier
                            );
                            println!("⏪ {}", label);
                            self.status_notification = Some((label, Instant::now()));
                        }

                        // Toggle Drivable Car Simulation
                        KeyCode::KeyV => {
                            self.config.is_drive = !self.config.is_drive;
                            let label = if self.config.is_drive {
                                "Simulation Mode: Complete Drivable Vehicle Dynamics"
                            } else {
                                "Simulation Mode: Free Camera Exploration"
                            };
                            println!("🏎️ {}", label);
                            self.status_notification = Some((label.to_string(), Instant::now()));
                        }

                        // Cycle Vehicle Camera Perspective (Chase -> Hood -> Bumper)
                        KeyCode::KeyC => {
                            if self.config.is_drive {
                                self.vehicle_cam.perspective = match self.vehicle_cam.perspective {
                                    VehicleCameraPerspective::Chase => {
                                        VehicleCameraPerspective::Hood
                                    }
                                    VehicleCameraPerspective::Hood => {
                                        VehicleCameraPerspective::Bumper
                                    }
                                    VehicleCameraPerspective::Bumper => {
                                        VehicleCameraPerspective::Chase
                                    }
                                };
                                let label =
                                    format!("Vehicle Camera: {:?}", self.vehicle_cam.perspective);
                                println!("📷 {}", label);
                                self.status_notification = Some((label, Instant::now()));
                            }
                        }

                        // Cycle Graphics Preset (Low -> Medium -> High -> Adaptive)
                        KeyCode::KeyP => {
                            let next_preset = match self.config.preset_level {
                                GraphicsPresetLevel::Low => GraphicsPresetLevel::Medium,
                                GraphicsPresetLevel::Medium => GraphicsPresetLevel::High,
                                GraphicsPresetLevel::High => GraphicsPresetLevel::Adaptive,
                                GraphicsPresetLevel::Adaptive => GraphicsPresetLevel::Low,
                            };
                            self.config.preset_level = next_preset;
                            if let Some(ref mut renderer) = self.renderer {
                                renderer.settings = GraphicsSettings::from_preset(next_preset);
                            }
                            let label = format!("Graphics Quality Preset: {:?}", next_preset);
                            println!("⚙️ {}", label);
                            self.status_notification = Some((label, Instant::now()));
                        }

                        // Cycle Geographic Regions
                        KeyCode::KeyR => {
                            self.region_cycle_idx =
                                (self.region_cycle_idx + 1) % self.region_cycle.len();
                            let next_reg = self.region_cycle[self.region_cycle_idx];
                            self.config.region_id = next_reg.to_string();
                            let _ = self.load_region(next_reg);
                        }

                        // Cycle Garage / Vehicle Mode with authentic physics presets
                        KeyCode::KeyT | KeyCode::KeyG => {
                            self.vehicle_visual_type = match self.vehicle_visual_type {
                                VehicleVisualType::SportsCoupe => VehicleVisualType::Sedan,
                                VehicleVisualType::Sedan => VehicleVisualType::Suv,
                                VehicleVisualType::Suv => VehicleVisualType::Taxi,
                                VehicleVisualType::Taxi => VehicleVisualType::TransitBus,
                                VehicleVisualType::TransitBus => VehicleVisualType::HeavyTruck,
                                VehicleVisualType::HeavyTruck => VehicleVisualType::SportsCoupe,
                            };
                            self.vehicle.config = match self.vehicle_visual_type {
                                VehicleVisualType::SportsCoupe => VehicleConfig::sports_coupe(),
                                VehicleVisualType::Sedan => VehicleConfig::sedan(),
                                VehicleVisualType::Suv => VehicleConfig::suv(),
                                VehicleVisualType::Taxi => VehicleConfig::taxi(),
                                VehicleVisualType::TransitBus => VehicleConfig::transit_bus(),
                                VehicleVisualType::HeavyTruck => VehicleConfig::heavy_truck(),
                            };
                            let label = format!(
                                "Garage Selected: {:?} (Mass: {:.0} kg, Max Torque: {:.0} N)",
                                self.vehicle_visual_type,
                                self.vehicle.config.mass_kg,
                                self.vehicle.config.peak_engine_force
                            );
                            println!("🚗 {}", label);
                            self.status_notification = Some((label, Instant::now()));
                        }

                        // Cycle / Accept Regional Contracts
                        KeyCode::KeyJ => {
                            let available =
                                JobCatalogGenerator::jobs_for_region(&self.config.region_id);
                            if !available.is_empty() {
                                let next_job = if let Some(ref current) = self.active_job {
                                    let idx = available
                                        .iter()
                                        .position(|j| j.id == current.id)
                                        .unwrap_or(0);
                                    available[(idx + 1) % available.len()].clone()
                                } else {
                                    available[0].clone()
                                };
                                let mut j = next_job;
                                let _ = j.accept(&self.player_profile, &self.gameplay_bus);
                                let label = format!(
                                    "📋 Accepted Contract: {} (Payout: ₹{:.2})",
                                    j.title, j.base_payout
                                );
                                println!("{}", label);
                                self.active_job = Some(j);
                                self.status_notification = Some((label, Instant::now()));
                            }
                        }

                        // F5 Quicksave
                        KeyCode::F5 => {
                            let save_dir = if std::path::Path::new("target").exists() {
                                PathBuf::from("target/saves")
                            } else {
                                PathBuf::from("saves")
                            };
                            let _ = std::fs::create_dir_all(&save_dir);
                            let save_path = save_dir.join("quicksave.json");

                            let veh_coord = if let Some(ref res) = self.region_resources {
                                let meters_per_deg_lat = 111_320.0;
                                let meters_per_deg_lon = 111_320.0
                                    * res.landmark.coord.lat.to_radians().cos().abs().max(0.01);
                                Wgs84Coord::new(
                                    res.landmark.coord.lat
                                        + (self.vehicle.position.z as f64 / meters_per_deg_lat),
                                    res.landmark.coord.lon
                                        + (self.vehicle.position.x as f64 / meters_per_deg_lon),
                                    self.vehicle.position.y as f64,
                                )
                            } else {
                                self.camera_ctrl.coord
                            };

                            let save_data = SaveData {
                                schema_version: CURRENT_SAVE_SCHEMA_VERSION,
                                timestamp_unix: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .map(|d| d.as_secs())
                                    .unwrap_or(0),
                                profile: self.player_profile.clone(),
                                active_job: self.active_job.clone(),
                                active_region_id: self.config.region_id.clone(),
                                vehicle_position: veh_coord,
                                settings: GameSettings::default(),
                            };

                            match SaveManager::save_to_file(&save_path, &save_data) {
                                Ok(_) => {
                                    let msg = format!(
                                        "💾 Quicksave saved (Wallet: ₹{:.2})",
                                        self.player_profile.wallet.balance()
                                    );
                                    println!("{}", msg);
                                    self.status_notification = Some((msg, Instant::now()));
                                }
                                Err(e) => {
                                    let msg = format!("⚠️ Quicksave failed: {}", e);
                                    eprintln!("{}", msg);
                                    self.status_notification = Some((msg, Instant::now()));
                                }
                            }
                        }

                        // F9 Quickload
                        KeyCode::F9 => {
                            let save_path = if PathBuf::from("target/saves/quicksave.json").exists()
                            {
                                PathBuf::from("target/saves/quicksave.json")
                            } else {
                                PathBuf::from("saves/quicksave.json")
                            };
                            if save_path.exists() {
                                match SaveManager::load_from_file(&save_path) {
                                    Ok(loaded) => {
                                        self.player_profile = loaded.profile;
                                        self.active_job = loaded.active_job;
                                        let msg = format!(
                                            "📂 Quickload restored (Wallet: ₹{:.2}, Jobs: {})",
                                            self.player_profile.wallet.balance(),
                                            self.player_profile.completed_jobs
                                        );
                                        println!("{}", msg);
                                        self.status_notification = Some((msg, Instant::now()));
                                    }
                                    Err(e) => {
                                        let msg = format!("⚠️ Quickload failed: {}", e);
                                        eprintln!("{}", msg);
                                        self.status_notification = Some((msg, Instant::now()));
                                    }
                                }
                            } else {
                                let msg =
                                    "⚠️ No quicksave file found (checked target/saves and saves)"
                                        .to_string();
                                println!("{}", msg);
                                self.status_notification = Some((msg, Instant::now()));
                            }
                        }

                        // Toggle HUD Overlay
                        KeyCode::KeyH | KeyCode::F1 => {
                            self.show_hud = !self.show_hud;
                            let label = format!(
                                "HUD Display: {}",
                                if self.show_hud { "ON" } else { "OFF" }
                            );
                            println!("📊 {}", label);
                            self.status_notification = Some((label, Instant::now()));
                        }

                        // High-Resolution Screenshot Capture
                        KeyCode::F12 => {
                            self.capture_screenshot();
                        }

                        // Instant Landmark Bookmarks (1-9)
                        KeyCode::Digit1 => {
                            self.camera_ctrl
                                .jump_to(&Landmark::SAN_FRANCISCO_TRANSAMERICA);
                            self.status_notification = Some((
                                format!("📍 Jump to {}", Landmark::SAN_FRANCISCO_TRANSAMERICA.name),
                                Instant::now(),
                            ));
                        }
                        KeyCode::Digit2 => {
                            self.camera_ctrl
                                .jump_to(&Landmark::SAN_FRANCISCO_SALESFORCE);
                            self.status_notification = Some((
                                format!("📍 Jump to {}", Landmark::SAN_FRANCISCO_SALESFORCE.name),
                                Instant::now(),
                            ));
                        }
                        KeyCode::Digit3 => {
                            self.camera_ctrl
                                .jump_to(&Landmark::SAN_FRANCISCO_FERRY_BUILDING);
                            self.status_notification = Some((
                                format!(
                                    "📍 Jump to {}",
                                    Landmark::SAN_FRANCISCO_FERRY_BUILDING.name
                                ),
                                Instant::now(),
                            ));
                        }
                        KeyCode::Digit4 => {
                            self.camera_ctrl
                                .jump_to(&Landmark::SAN_FRANCISCO_COIT_TOWER);
                            self.status_notification = Some((
                                format!("📍 Jump to {}", Landmark::SAN_FRANCISCO_COIT_TOWER.name),
                                Instant::now(),
                            ));
                        }
                        KeyCode::Digit5 => {
                            self.camera_ctrl
                                .jump_to(&Landmark::SAN_FRANCISCO_BAY_BRIDGE);
                            self.status_notification = Some((
                                format!("📍 Jump to {}", Landmark::SAN_FRANCISCO_BAY_BRIDGE.name),
                                Instant::now(),
                            ));
                        }
                        KeyCode::Digit6 => {
                            self.camera_ctrl
                                .jump_to(&Landmark::BENGALURU_VIDHANA_SOUDHA);
                            self.status_notification = Some((
                                format!("📍 Jump to {}", Landmark::BENGALURU_VIDHANA_SOUDHA.name),
                                Instant::now(),
                            ));
                        }
                        KeyCode::Digit7 => {
                            self.camera_ctrl.jump_to(&Landmark::MYSURU_PALACE);
                            self.status_notification = Some((
                                format!("📍 Jump to {}", Landmark::MYSURU_PALACE.name),
                                Instant::now(),
                            ));
                        }
                        KeyCode::Digit8 => {
                            self.camera_ctrl.jump_to(&Landmark::CHENNAI_CENTRAL);
                            self.status_notification = Some((
                                format!("📍 Jump to {}", Landmark::CHENNAI_CENTRAL.name),
                                Instant::now(),
                            ));
                        }
                        KeyCode::Digit9 => {
                            self.camera_ctrl.jump_to(&Landmark::HYDERABAD_CHARMINAR);
                            self.status_notification = Some((
                                format!("📍 Jump to {}", Landmark::HYDERABAD_CHARMINAR.name),
                                Instant::now(),
                            ));
                        }
                        _ => {}
                    }
                }
                ElementState::Released => {
                    self.pressed_keys.remove(&key);
                }
            },

            WindowEvent::RedrawRequested => {
                if self.is_minimized {
                    return;
                }

                let (surface, renderer, surface_config, depth_view, res, sim_world, gq) = match (
                    &self.surface,
                    &mut self.renderer,
                    &self.surface_config,
                    &self.depth_view,
                    &mut self.region_resources,
                    &mut self.sim_world,
                    &self.ground_query,
                ) {
                    (Some(s), Some(r), Some(sc), Some(dv), Some(res), Some(sw), Some(gq)) => {
                        (s, r, sc, dv, res, sw, gq)
                    }
                    _ => return,
                };

                // 1. Acquire current SurfaceTexture
                let surface_texture = match surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(wgpu::SurfaceError::Timeout) => {
                        if let Some(ref window) = self.window {
                            window.request_redraw();
                        }
                        return;
                    }
                    Err(wgpu::SurfaceError::Outdated) | Err(wgpu::SurfaceError::Lost) => {
                        surface.configure(&renderer.device, surface_config);
                        if let Some(ref window) = self.window {
                            window.request_redraw();
                        }
                        return;
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        eprintln!("⚠️ wgpu SurfaceError::OutOfMemory: exiting application");
                        event_loop.exit();
                        return;
                    }
                };

                // 2. Compute delta time (clamped between 1ms and 100ms)
                let now = Instant::now();
                let dt = (now - self.last_frame).as_secs_f64().clamp(0.001, 0.1);
                self.last_frame = now;

                // 3. Fixed-timestep simulation & Camera / Vehicle updates
                let anchor = res.landmark.coord;
                let meters_per_deg_lat = 111_320.0;
                let meters_per_deg_lon = 111_320.0 * anchor.lat.to_radians().cos().abs().max(0.01);

                sim_world.tick(dt);
                let solar = sim_world.current_solar_position();

                let is_braking = self.pressed_keys.contains(&KeyCode::KeyS)
                    || self.pressed_keys.contains(&KeyCode::ArrowDown);

                let velocity = if self.config.is_drive {
                    // Update vehicle dynamics from inputs
                    let mut inputs = VehicleInputs::default();
                    if self.pressed_keys.contains(&KeyCode::KeyW)
                        || self.pressed_keys.contains(&KeyCode::ArrowUp)
                    {
                        inputs.throttle = 1.0;
                    }
                    if is_braking {
                        inputs.brake = 0.8;
                    }
                    if self.pressed_keys.contains(&KeyCode::KeyA)
                        || self.pressed_keys.contains(&KeyCode::ArrowLeft)
                    {
                        inputs.steer = -0.55;
                    }
                    if self.pressed_keys.contains(&KeyCode::KeyD)
                        || self.pressed_keys.contains(&KeyCode::ArrowRight)
                    {
                        inputs.steer = 0.55;
                    }
                    if self.pressed_keys.contains(&KeyCode::Space) {
                        inputs.handbrake = true;
                    }

                    self.vehicle.step(&inputs, gq, dt as f32);

                    // Prevent sinking or tunneling through terrain
                    let ground_y = gq.elevation(self.vehicle.position.x, self.vehicle.position.z);
                    let min_allowed_y = ground_y + self.vehicle.config.wheel_radius;
                    if self.vehicle.position.y < min_allowed_y {
                        self.vehicle.position.y = min_allowed_y;
                        if self.vehicle.linear_velocity.y < 0.0 {
                            self.vehicle.linear_velocity.y = 0.0;
                        }
                    }

                    self.traffic_sim.step(dt as f32, gq);

                    let veh_coord = Wgs84Coord::new(
                        anchor.lat + (self.vehicle.position.z as f64 / meters_per_deg_lat),
                        anchor.lon + (self.vehicle.position.x as f64 / meters_per_deg_lon),
                        self.vehicle.position.y as f64,
                    );

                    if let Some(ref mut job) = self.active_job {
                        if let Some(completion) = job.update(
                            dt as f32,
                            veh_coord,
                            self.vehicle.speed_kph,
                            &self.gameplay_bus,
                        ) {
                            let now_secs = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .map(|d| d.as_secs())
                                .unwrap_or(0);
                            self.player_profile.record_job_completion(
                                completion.career,
                                completion.final_payout,
                                completion.xp_gained,
                                (completion.rating_stars * 20.0) as u32,
                                now_secs,
                            );

                            let msg = format!(
                                "🎉 CONTRACT COMPLETE! +₹{:.2} | Rating: {:.1} Stars | Balance: ₹{:.2}",
                                completion.final_payout,
                                completion.rating_stars,
                                self.player_profile.wallet.balance()
                            );
                            println!("{}", msg);
                            self.status_notification = Some((msg, Instant::now()));

                            let available =
                                JobCatalogGenerator::jobs_for_region(&self.config.region_id);
                            if let Some(next_job) =
                                available.into_iter().find(|j| j.id != completion.job_id)
                            {
                                let mut j = next_job;
                                let _ = j.accept(&self.player_profile, &self.gameplay_bus);
                                self.active_job = Some(j);
                            }
                        }
                    }
                    let speed_mps = self.vehicle.speed_kph as f64 * (1000.0 / 3600.0);
                    self.player_profile
                        .record_distance((speed_mps * dt) / 1000.0);

                    let (cam_pos, cam_yaw, cam_pitch) =
                        self.vehicle_cam.update(&self.vehicle, dt as f32);
                    let cam_lat = anchor.lat + (cam_pos.z as f64 / meters_per_deg_lat);
                    let cam_lon = anchor.lon + (cam_pos.x as f64 / meters_per_deg_lon);
                    self.camera_ctrl.coord = Wgs84Coord::new(cam_lat, cam_lon, cam_pos.y as f64);
                    self.camera_ctrl.yaw = cam_yaw;
                    self.camera_ctrl.pitch = cam_pitch;
                    self.camera_ctrl.update_camera(&mut self.camera);

                    let rot = glam::Mat3::from_quat(self.vehicle.orientation);
                    let forward = rot.col(2);
                    forward * (self.vehicle.speed_kph * (1000.0 / 3600.0))
                } else {
                    // Camera navigation in Fly, Walk, Orbit, Free, or Chase mode
                    let base_speed = match self.camera_ctrl.mode {
                        CameraMode::Fly => self.camera_ctrl.effective_speed(),
                        CameraMode::Walk => 5.0,
                        CameraMode::Orbit => 25.0,
                        CameraMode::Free => self.camera_ctrl.effective_speed() * 1.5,
                        CameraMode::Chase => self.camera_ctrl.effective_speed(),
                    };
                    let move_speed = base_speed * dt;
                    let rot_speed = 1.6 * dt as f32;

                    // Sprint acceleration with Shift / precision slow with Control
                    let speed_mult = if self.pressed_keys.contains(&KeyCode::ShiftLeft)
                        || self.pressed_keys.contains(&KeyCode::ShiftRight)
                    {
                        2.5
                    } else if self.pressed_keys.contains(&KeyCode::ControlLeft)
                        || self.pressed_keys.contains(&KeyCode::ControlRight)
                    {
                        0.35
                    } else {
                        1.0
                    };
                    let effective_move = move_speed * speed_mult;

                    if self.pressed_keys.contains(&KeyCode::KeyW) {
                        self.camera_ctrl.move_forward(effective_move, Some(&res.dem));
                    }
                    if self.pressed_keys.contains(&KeyCode::KeyS) {
                        self.camera_ctrl.move_forward(-effective_move, Some(&res.dem));
                    }
                    if self.pressed_keys.contains(&KeyCode::KeyA) {
                        self.camera_ctrl.strafe(-effective_move, Some(&res.dem));
                    }
                    if self.pressed_keys.contains(&KeyCode::KeyD) {
                        self.camera_ctrl.strafe(effective_move, Some(&res.dem));
                    }
                    if self.pressed_keys.contains(&KeyCode::Space)
                        || self.pressed_keys.contains(&KeyCode::KeyE)
                    {
                        self.camera_ctrl.move_vertical(effective_move);
                    }
                    if self.pressed_keys.contains(&KeyCode::KeyQ) {
                        self.camera_ctrl.move_vertical(-effective_move);
                    }
                    if self.pressed_keys.contains(&KeyCode::ArrowLeft) {
                        self.camera_ctrl.rotate(-rot_speed, 0.0);
                    }
                    if self.pressed_keys.contains(&KeyCode::ArrowRight) {
                        self.camera_ctrl.rotate(rot_speed, 0.0);
                    }
                    if self.pressed_keys.contains(&KeyCode::ArrowUp) {
                        self.camera_ctrl.rotate(0.0, rot_speed);
                    }
                    if self.pressed_keys.contains(&KeyCode::ArrowDown) {
                        self.camera_ctrl.rotate(0.0, -rot_speed);
                    }

                    // Collision-aware ground clamping: prevents clipping below DEM terrain
                    self.camera_ctrl.clamp_terrain(&res.dem);
                    self.camera_ctrl.update_camera(&mut self.camera);
                    self.camera_ctrl.forward_velocity_vector(effective_move)
                };

                // 4. Update nearby real chunks using WorldStreamer
                let _ = res
                    .streamer
                    .update_with_velocity(self.camera_ctrl.coord, velocity);
                let active = res.streamer.get_active_chunks();

                // 5. Update lighting and camera uniforms
                renderer.update_uniforms_with_settings(
                    &self.camera,
                    [
                        solar.sun_direction_enu.x,
                        solar.sun_direction_enu.y,
                        solar.sun_direction_enu.z,
                    ],
                    solar.sun_intensity,
                    &renderer.settings,
                );

                // 6. Render visible world chunks (terrain, roads, buildings, water, vegetation)
                let color_view = surface_texture
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                self.last_culled_chunks = renderer.render_chunks_culled_cached(
                    &active,
                    &self.camera,
                    &color_view,
                    depth_view,
                );
                self.last_culled_triangles = active.iter().map(|c| c.total_triangles()).sum();

                // 7. Render 3D Vehicles (Player Car and AI Traffic)
                let mut fleet_mesh = realearth_compiler::mesh::ChunkMesh::default();
                if self.config.is_drive {
                    let player_mesh = generate_vehicle_mesh(&VehicleMeshParams {
                        position: self.vehicle.position,
                        orientation: self.vehicle.orientation,
                        color: match self.vehicle_visual_type {
                            VehicleVisualType::SportsCoupe => [0.92, 0.12, 0.12, 1.0], // Cherry Red
                            VehicleVisualType::Sedan => [0.15, 0.35, 0.85, 1.0], // Electric Blue
                            VehicleVisualType::Suv => [0.20, 0.22, 0.25, 1.0],   // Graphite Gray
                            VehicleVisualType::Taxi => [0.95, 0.80, 0.10, 1.0],  // Yellow Taxi
                            VehicleVisualType::TransitBus => [0.10, 0.65, 0.40, 1.0], // City Green Transit
                            VehicleVisualType::HeavyTruck => [0.85, 0.45, 0.10, 1.0], // Orange Hauler
                        },
                        vehicle_type: self.vehicle_visual_type,
                        wheel_steer_angle: self.vehicle.wheels[0].steer_angle,
                        headlights_on: true,
                        brake_lights_on: is_braking,
                    });
                    fleet_mesh.append(&player_mesh);
                }

                // Append traffic vehicles
                let traffic_colors = [
                    [0.85, 0.85, 0.88, 1.0], // Silver
                    [0.18, 0.20, 0.24, 1.0], // Black
                    [0.90, 0.20, 0.20, 1.0], // Red
                    [0.20, 0.45, 0.80, 1.0], // Blue
                    [0.95, 0.80, 0.10, 1.0], // Yellow
                ];
                for (idx, agent) in self.traffic_sim.agents.iter().enumerate() {
                    let t_color = traffic_colors[idx % traffic_colors.len()];
                    let t_mesh = generate_vehicle_mesh(&VehicleMeshParams {
                        position: agent.position,
                        orientation: glam::Quat::from_rotation_y(agent.yaw),
                        color: t_color,
                        vehicle_type: if idx % 4 == 0 {
                            VehicleVisualType::Taxi
                        } else {
                            VehicleVisualType::Sedan
                        },
                        wheel_steer_angle: 0.0,
                        headlights_on: true,
                        brake_lights_on: false,
                    });
                    fleet_mesh.append(&t_mesh);
                }

                if !fleet_mesh.is_empty() {
                    renderer.render_mesh(&fleet_mesh, &color_view, depth_view, false);
                }

                // 8. Render Integrated 2D HUD and Telemetry
                if self.show_hud {
                    let mut hud = HudMesh::new();
                    let width_f = surface_config.width as f32;
                    let height_f = surface_config.height as f32;

                    // Top Status Panel
                    hud.add_panel(16.0, 16.0, 520.0, 118.0, [0.0, 0.75, 1.0, 1.0]);
                    hud.add_text(
                        26.0,
                        26.0,
                        1.4,
                        &format!("REAL EARTH WORLD — {}", res.region_name),
                        [1.0, 1.0, 1.0, 1.0],
                    );

                    let avg_ms = if self.frame_durations_ms.is_empty() {
                        16.6
                    } else {
                        self.frame_durations_ms.iter().sum::<f64>()
                            / self.frame_durations_ms.len() as f64
                    };
                    let fps = (1000.0 / avg_ms).max(1.0);

                    // Compute 1% low FPS
                    let one_pct_low_fps = if self.frame_durations_ms.len() >= 10 {
                        let mut sorted = self.frame_durations_ms.clone();
                        sorted
                            .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                        let idx =
                            ((sorted.len() as f64 * 0.99).floor() as usize).min(sorted.len() - 1);
                        1000.0 / sorted[idx].max(0.1)
                    } else {
                        fps
                    };

                    hud.add_text(
                        26.0,
                        44.0,
                        1.05,
                        &format!(
                            "FPS: {:>4.1} (1% Low: {:>4.1}) | {:.2}ms | Preset: {:?}",
                            fps, one_pct_low_fps, avg_ms, renderer.settings.level
                        ),
                        [0.2, 0.95, 0.45, 1.0],
                    );
                    hud.add_text(
                        26.0,
                        62.0,
                        0.95,
                        &format!(
                            "Coord: {:.4}N, {:.4}W, {:.1}m | Anchor: {}",
                            self.camera_ctrl.coord.lat,
                            self.camera_ctrl.coord.lon,
                            self.camera_ctrl.coord.alt,
                            res.landmark.name
                        ),
                        [0.75, 0.85, 0.95, 1.0],
                    );
                    hud.add_text(
                        26.0,
                        80.0,
                        0.95,
                        &format!(
                            "LOD Chunks: {} active ({} culled) | Scene Tris: {}",
                            active.len(),
                            self.last_culled_chunks,
                            self.last_culled_triangles
                        ),
                        [0.85, 0.75, 0.40, 1.0],
                    );
                    let proc_mem = get_current_process_memory();
                    let vram_mb = renderer.gpu_memory_bytes() as f64 / (1024.0 * 1024.0);
                    hud.add_text(
                        26.0,
                        98.0,
                        0.95,
                        &format!(
                            "Memory: RSS {:.1}MB (Peak {:.1}MB) | VRAM: {:.1}MB",
                            proc_mem.working_set_size_mb(),
                            proc_mem.peak_working_set_size_mb(),
                            vram_mb
                        ),
                        [0.95, 0.65, 0.30, 1.0],
                    );

                    // Vehicle Cockpit & Speedometer Panel (when in Drive mode)
                    if self.config.is_drive {
                        let panel_y = height_f - 150.0;
                        hud.add_panel(16.0, panel_y, 400.0, 130.0, [1.0, 0.4, 0.1, 1.0]);
                        hud.add_text(
                            26.0,
                            panel_y + 12.0,
                            1.4,
                            &format!(
                                "SPEED: {:>5.1} KM/H  |  GEAR: {}",
                                self.vehicle.speed_kph, self.vehicle.current_gear
                            ),
                            [1.0, 0.9, 0.2, 1.0],
                        );

                        // Speedometer Bar
                        hud.add_bar(
                            26.0,
                            panel_y + 36.0,
                            360.0,
                            12.0,
                            self.vehicle.speed_kph / 180.0,
                            [0.95, 0.35, 0.10, 1.0],
                            [0.15, 0.18, 0.22, 1.0],
                        );

                        // Tachometer RPM Bar
                        hud.add_text(
                            26.0,
                            panel_y + 58.0,
                            1.1,
                            &format!(
                                "ENGINE: {:.0} RPM  |  Cam: {:?}",
                                self.vehicle.engine_rpm, self.vehicle_cam.perspective
                            ),
                            [0.7, 0.85, 1.0, 1.0],
                        );
                        hud.add_bar(
                            26.0,
                            panel_y + 74.0,
                            360.0,
                            8.0,
                            self.vehicle.engine_rpm / 7500.0,
                            [0.10, 0.85, 0.70, 1.0],
                            [0.15, 0.18, 0.22, 1.0],
                        );

                        // Suspension Compression Telemetry
                        hud.add_text(
                            26.0,
                            panel_y + 92.0,
                            0.95,
                            &format!(
                                "Wheels (m): FL={:.2} FR={:.2} RL={:.2} RR={:.2} | Traffic: {}",
                                self.vehicle.wheels[0].compression,
                                self.vehicle.wheels[1].compression,
                                self.vehicle.wheels[2].compression,
                                self.vehicle.wheels[3].compression,
                                self.traffic_sim.agents.len()
                            ),
                            [0.8, 0.8, 0.8, 0.9],
                        );
                    }

                    // Gameplay Career & Active Job Objective Panel (Top Right)
                    if let Some(ref job) = self.active_job {
                        let hud_model = GameplayHudModel::build_objective_hud(
                            Some(job),
                            self.camera_ctrl.coord,
                            self.vehicle.speed_kph,
                        );
                        let earn_model =
                            GameplayHudModel::build_earnings_hud(&self.player_profile, Some(job));

                        let card_w = 400.0;
                        let card_x = width_f - card_w - 16.0;
                        hud.add_panel(card_x, 16.0, card_w, 110.0, [0.3, 0.9, 0.4, 1.0]);
                        hud.add_text(
                            card_x + 10.0,
                            26.0,
                            1.3,
                            &format!("CAREER: {}", hud_model.career_title),
                            [1.0, 1.0, 1.0, 1.0],
                        );
                        hud.add_text(
                            card_x + 10.0,
                            46.0,
                            1.0,
                            &format!("Target: {}", hud_model.target_location_name),
                            [0.4, 0.95, 0.7, 1.0],
                        );
                        hud.add_text(
                            card_x + 10.0,
                            64.0,
                            1.0,
                            &format!(
                                "Distance: {:.1}m | Comfort: {:.1} Stars | Wallet: ₹{:.2}",
                                hud_model.distance_to_target_meters,
                                hud_model.comfort_rating_stars,
                                earn_model.balance
                            ),
                            [1.0, 0.9, 0.3, 1.0],
                        );
                        hud.add_text(
                            card_x + 10.0,
                            82.0,
                            0.95,
                            &format!("Objective: {}", hud_model.instruction_text),
                            [0.8, 0.85, 0.9, 1.0],
                        );
                    }

                    // Bottom Right Controls Guide
                    let guide_w = 460.0;
                    let guide_x = width_f - guide_w - 16.0;
                    let guide_y = height_f - 95.0;
                    hud.add_panel(guide_x, guide_y, guide_w, 82.0, [0.6, 0.6, 0.6, 0.8]);
                    hud.add_text(
                        guide_x + 10.0,
                        guide_y + 10.0,
                        0.92,
                        "[W/S/A/D] Move/Drive | [Space/C] Handbrake/Down | [Mouse] Look",
                        [0.9, 0.9, 0.9, 1.0],
                    );
                    hud.add_text(
                        guide_x + 10.0,
                        guide_y + 26.0,
                        0.92,
                        "[V] Drivable Car | [G] Garage Fleet | [J] Contracts | [C] Camera",
                        [0.9, 0.9, 0.9, 1.0],
                    );
                    hud.add_text(
                        guide_x + 10.0,
                        guide_y + 42.0,
                        0.92,
                        "[R] Region Cycle | [P] Preset | [1-9] Landmarks | [H] HUD Toggle",
                        [0.9, 0.9, 0.9, 1.0],
                    );
                    hud.add_text(
                        guide_x + 10.0,
                        guide_y + 58.0,
                        0.92,
                        "[F5] Quicksave | [F9] Quickload | [F12] Screenshot | [Esc] Exit",
                        [0.9, 0.9, 0.9, 1.0],
                    );

                    // Temporary Status / Notification Banner
                    if let Some((ref text, stamp)) = self.status_notification {
                        if stamp.elapsed().as_secs_f32() < 4.0 {
                            let banner_w = 560.0;
                            let banner_x = (width_f - banner_w) * 0.5;
                            hud.add_panel(
                                banner_x,
                                height_f - 140.0,
                                banner_w,
                                40.0,
                                [0.1, 0.9, 0.6, 1.0],
                            );
                            hud.add_text(
                                banner_x + 14.0,
                                height_f - 128.0,
                                1.15,
                                text,
                                [1.0, 1.0, 1.0, 1.0],
                            );
                        }
                    }

                    renderer.render_hud(
                        &hud,
                        &color_view,
                        surface_config.width,
                        surface_config.height,
                    );
                }

                // 9. Present SurfaceTexture
                surface_texture.present();

                let frame_elapsed = now.elapsed().as_secs_f64() * 1000.0;
                if self.frame_durations_ms.len() >= 60 {
                    self.frame_durations_ms.remove(0);
                }
                self.frame_durations_ms.push(frame_elapsed);
                self.total_rendered_frames += 1;

                // Adapt graphics if adaptive preset is enabled
                if renderer.settings.level == GraphicsPresetLevel::Adaptive {
                    renderer.settings.adapt_to_frame_time(frame_elapsed as f32);
                }

                // Request continuous presentation for smooth 60 FPS V-synced rendering
                if let Some(ref window) = self.window {
                    window.request_redraw();
                }
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Redraws are scheduled synchronously on frame present and window events,
        // avoiding busy-looping while maintaining responsive vsync framerate.
    }
}

/// Runs the interactive visible world viewer session using native winit + wgpu surface.
pub fn run_interactive_event_loop(
    config: ViewerLaunchConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎮 Initializing persistent winit + wgpu interactive surface viewer...");
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = InteractiveViewerApp::new(config);
    event_loop.run_app(&mut app)?;
    Ok(())
}
