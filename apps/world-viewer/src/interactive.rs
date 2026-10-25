//! Interactive desktop windowing, live wgpu surface presentation, and navigation loop.
#![allow(clippy::needless_return, clippy::type_complexity)]

use std::collections::HashSet;
use std::path::{Path, PathBuf};
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
    save::CURRENT_SAVE_SCHEMA_VERSION, CareerType, GameSettings, GameplayEventBus, Job,
    JobCatalogGenerator, PlayerProfile, SaveData, SaveManager,
};
use realearth_geospatial::transforms::EnuConverter;
use realearth_ingestion::dem_grid::DemGrid;
use realearth_ingestion::load_authentic_city;
use realearth_ingestion::osm_types::HighwayType;
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
use realearth_navigation::{
    find_route, generate_turn_by_turn, ManeuverType, NavigationUpdate, RoadNetworkGraph,
    RouteTracker, RoutingPreference, RuleOfTheRoad,
};
use realearth_rendering::{
    generate_pedestrian_mesh, generate_road_sign_mesh, generate_traffic_signal_mesh,
    generate_vehicle_mesh, CatalogHit, CatalogLayout, FloatingOriginCamera, GarageHit,
    GarageLayout, GraphicsPresetLevel, GraphicsSettings, HudMesh, JobBrowserHit, JobBrowserLayout,
    PauseMenuHit, PauseMenuLayout, RealEarthRenderer, RoadSignType, SettingsHit, SettingsLayout,
    StartScreenHit, StartScreenLayout, TrafficSignalVisualState, VehicleLodLevel,
    VehicleMeshParams, VehicleVisualType,
};
use realearth_shared::coordinates::Wgs84Coord;
use realearth_shared::metrics::get_current_process_memory;
use realearth_shared::us_catalog::{all_us_cities, find_city_by_id, CityPriorityTier};
use realearth_simulation::RealEarthSimWorld;
use realearth_streaming::{StreamingConfig, WorldStreamer};
use realearth_validation::pipeline::validate_dataset;
use realearth_vehicles::{
    AircraftFlight, DynamicVehicle, FlightInputs, GroundQuery, HeavyTruckModel,
    IntersectionStopLine, LocomotiveType, MarineInputs, MarineVessel, MotorcycleModel,
    PedestrianAgent, TrafficLight, TrafficLightState, TrafficSimulation, TrainCar, TrainConsist,
    TransitBusModel, TransmissionMode, VehicleCameraController, VehicleCameraPerspective,
    VehicleConfig, VehicleInputs,
};

/// Verified authentic roadway spawn location candidate.
#[derive(Debug, Clone)]
pub struct RoadSpawnCandidate {
    pub position: glam::Vec3,
    pub yaw: f32,
    pub surface_elevation: f32,
    pub surface_normal: glam::Vec3,
    pub road_name: String,
    pub highway_type: HighwayType,
    pub lane_width: f32,
}

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

/// Playable multimodal transportation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ActiveTransportMode {
    Car,
    Motorcycle,
    TransitBus,
    HeavyTruck,
    Train,
    Marine,
    Aviation,
}

impl ActiveTransportMode {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Car => "Automobile (Ackermann/Pacejka)",
            Self::Motorcycle => "Motorcycle (2-Wheel Lean Dynamics)",
            Self::TransitBus => "Transit Bus (Pneumatic Air Brakes)",
            Self::HeavyTruck => "Heavy Haul Truck (Trailer Articulation)",
            Self::Train => "Railway Train Consist (Davis / Westinghouse)",
            Self::Marine => "Marine Vessel (Archimedes / Metacentric)",
            Self::Aviation => "Aviation Aircraft (6-DOF Aerodynamic Polar)",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Car => "🏎️",
            Self::Motorcycle => "🏍️",
            Self::TransitBus => "🚌",
            Self::HeavyTruck => "🚛",
            Self::Train => "🚆",
            Self::Marine => "🚢",
            Self::Aviation => "✈️",
        }
    }
}

/// Configuration options passed when launching the interactive viewer.
#[derive(Debug, Clone)]
pub struct ViewerLaunchConfig {
    pub region_id: String,
    pub initial_mode: CameraMode,
    pub is_drive: bool,
    pub active_mode: ActiveTransportMode,
    pub preset_level: GraphicsPresetLevel,
    pub traffic_count: usize,
    pub vehicle_cam_perspective: VehicleCameraPerspective,
    pub career_arg: Option<String>,
    pub job_id_arg: Option<String>,
    pub target_location: Option<String>,
    pub multiplayer_server: Option<String>,
    pub host_server_port: Option<u16>,
    pub player_name: String,
}

impl Default for ViewerLaunchConfig {
    fn default() -> Self {
        Self {
            region_id: "san-francisco".to_string(),
            initial_mode: CameraMode::Fly,
            is_drive: false,
            active_mode: ActiveTransportMode::Car,
            preset_level: GraphicsPresetLevel::High,
            traffic_count: 8,
            vehicle_cam_perspective: VehicleCameraPerspective::Chase,
            career_arg: None,
            job_id_arg: None,
            target_location: None,
            multiplayer_server: None,
            host_server_port: None,
            player_name: "PilotAlpha".to_string(),
        }
    }
}

/// Loaded regional dataset and simulation assets.
struct RegionResources {
    _world_dir: PathBuf,
    region_name: &'static str,
    landmark: Landmark,
    package_anchor: Wgs84Coord,
    dem: DemGrid,
    raw_roads: Vec<realearth_ingestion::osm_types::OsmRoadFeature>,
    raw_buildings: Vec<realearth_ingestion::osm_types::OsmBuildingFeature>,
    streamer: WorldStreamer,
}

/// Explicit asynchronous region transition state machine phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionTransitionPhase {
    Idle,
    Selected,
    CheckingAssets,
    Downloading,
    Verifying,
    Compiling,
    Loading,
    Ready,
    Activating,
    Complete,
    Failed,
    Cancelled,
}

/// Stage 6 high-level user interface and gameplay navigation states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppUiState {
    StartScreen,
    InGame,
    PauseMenu,
    JobBrowser,
    GarageSelect,
    SettingsMenu,
    CatalogModal,
    DebriefModal,
    MultiplayerModal,
}

/// Settlement debrief details emitted when a mission contract completes.
#[derive(Debug, Clone)]
pub struct ActiveDebrief {
    pub title: String,
    pub career: String,
    pub base_payout: f64,
    pub tip_bonus: f64,
    pub speed_bonus: f64,
    pub net_payout: f64,
    pub xp_reward: u32,
    pub comfort_stars: f32,
}

/// Centralized UI domain actions dispatched by Mouse, Keyboard, or Controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiDomainAction {
    ContinueCareer,
    FreeRoam,
    OpenContracts,
    OpenGeography,
    OpenGarage,
    OpenSettings,
    ExitApplication,
    CancelOrBack,
    ResumeSimulation,
    RoadsideRecovery,
    QuicksaveProgress,
    QuickloadProgress,
    ReturnToStartScreen,
    AcceptSelectedContract,
    SelectAndSpawnVehicle(usize),
    ToggleSelectedSetting(usize),
    WarpToSelectedCity(usize),
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
    physics_accumulator: f32,
    previous_vehicle_pos: glam::Vec3,
    previous_vehicle_orient: glam::Quat,
    smoothed_steer_input: f32,
    pub park_engaged: bool,
    pub wheel_spin_angle: f32,
    traffic_sim: TrafficSimulation,
    vehicle_visual_type: VehicleVisualType,
    gameplay_bus: GameplayEventBus,
    player_profile: PlayerProfile,
    active_job: Option<Job>,
    route_tracker: Option<RouteTracker>,
    active_nav_update: Option<NavigationUpdate>,

    // Multimodal transportation state
    active_transport_mode: ActiveTransportMode,
    motorcycle: MotorcycleModel,
    transit_bus: TransitBusModel,
    heavy_truck: HeavyTruckModel,
    train_consist: TrainConsist,
    train_position: glam::Vec3,
    train_yaw: f32,
    marine_vessel: MarineVessel,
    marine_elapsed_time: f32,
    aircraft: AircraftFlight,
    aircraft_throttle: f32,

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
    catalog_selected_idx: usize,

    // Stage 6 & Stage 1 Modern UI/UX state machine
    pub ui_state: AppUiState,
    pub previous_ui_state: AppUiState,
    pub cursor_pos: Option<(f32, f32)>,
    pub mouse_left_down: bool,
    pub start_menu_selected_idx: usize,
    pub pause_menu_selected_idx: usize,
    pub job_browser_selected_idx: usize,
    pub garage_selected_idx: usize,
    pub garage_selected_color_idx: usize,
    pub vehicle_custom_color: Option<[f32; 4]>,
    pub audio_engine: realearth_gameplay::audio::VehicleAudioEngine,
    pub settings_selected_idx: usize,
    pub show_debug_overlay: bool,
    pub use_metric_units: bool,
    pub active_debrief: Option<ActiveDebrief>,

    // Stage 9 Authoritative Multiplayer state
    pub multiplayer_client: Option<realearth_networking::MultiplayerClient>,
    pub multiplayer_server_handle: Option<(
        std::sync::Arc<std::sync::atomic::AtomicBool>,
        std::thread::JoinHandle<()>,
    )>,
    pub multiplayer_selected_action: usize,
    pub multiplayer_server_addr_input: String,
    pub transition_phase: RegionTransitionPhase,
}

/// Resolves the storage location for compiled region chunks, supporting bundled packages,
/// standalone deployment directories, developer builds, and persistent user caches.
pub fn resolve_world_package_dir(slug: &str) -> PathBuf {
    // 1. Check if bundled in data/world_<slug> (distribution package in current working directory)
    let bundled = PathBuf::from(format!("data/world_{}", slug));
    if bundled.join("chunks.db").exists() {
        return bundled;
    }
    // 2. Check next to executable: <exe_dir>/data/world_<slug> and bundle layouts
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            // Windows & portable layout: <exe_dir>/data/world_<slug>
            let exe_bundled = exe_dir.join(format!("data/world_{}", slug));
            if exe_bundled.join("chunks.db").exists() {
                return exe_bundled;
            }
            // macOS App Bundle: <exe_dir>/../Resources/data/world_<slug>
            let mac_res = exe_dir.join(format!("../Resources/data/world_{}", slug));
            if mac_res.join("chunks.db").exists() {
                return mac_res;
            }
            // Linux AppDir / FHS: <exe_dir>/../share/realearth/data/world_<slug>
            let linux_share = exe_dir.join(format!("../share/realearth/data/world_{}", slug));
            if linux_share.join("chunks.db").exists() {
                return linux_share;
            }
        }
    }
    // 3. Default runtime cache directory in data/cache/world_<slug>
    let local_cache = PathBuf::from(format!("data/cache/world_{}", slug));
    if local_cache.join("chunks.db").exists() {
        return local_cache;
    }
    // 4. If developer repository target/ exists, use target/world_<slug>
    if std::path::Path::new("target").exists() {
        return PathBuf::from(format!("target/world_{}", slug));
    }
    // 5. Default runtime target directory in data/cache/world_<slug>
    local_cache
}

/// Validates whether an on-disk world package cache directory contains an authentic chunks.db
/// whose recorded `source_asset_sha256` matches the expected digest.
/// If `expected_source_hash` is None, checks only file existence.
pub fn is_world_package_cache_valid(
    dir: &Path,
    package_name: &str,
    expected_source_hash: Option<&str>,
) -> bool {
    let db_path = dir.join("chunks.db");
    if !db_path.exists() {
        return false;
    }
    if let Some(expected) = expected_source_hash {
        if let Ok(db) = realearth_compiler::ChunkDatabase::open(&db_path) {
            return db.is_package_source_hash_matching(package_name, expected);
        }
        return false;
    }
    true
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

        let motorcycle =
            MotorcycleModel::new(glam::Vec3::new(3.0, 10.0, 0.0), initial_landmark.yaw);
        let transit_bus =
            TransitBusModel::new(glam::Vec3::new(-5.0, 10.0, 0.0), initial_landmark.yaw);
        let heavy_truck =
            HeavyTruckModel::new(glam::Vec3::new(8.0, 10.0, 0.0), initial_landmark.yaw);

        let mut train_consist = TrainConsist::new_locomotive(
            "WAP-7 / Pacific Electric",
            LocomotiveType::Electric,
            4_500_000.0,
            450_000.0,
            123_000.0,
            20.5,
        );
        train_consist.add_wagon(TrainCar::new_wagon("Coach-A", 42_000.0, 3_000.0, 24.0));
        train_consist.add_wagon(TrainCar::new_wagon("Coach-B", 42_000.0, 3_000.0, 24.0));
        let train_position = glam::Vec3::new(0.0, 10.0, 20.0);
        let train_yaw = initial_landmark.yaw;

        let marine_vessel = MarineVessel::new_ferry(
            "Golden Gate Passenger Ferry",
            glam::Vec3::new(250.0, 0.0, -100.0),
            initial_landmark.yaw,
        );
        let marine_elapsed_time = 0.0;

        let mut aircraft = AircraftFlight::new_airliner(
            "EarthAir B737-800",
            glam::Vec3::new(0.0, 350.0, -500.0),
            initial_landmark.yaw,
        );
        aircraft.linear_velocity =
            glam::Quat::from_rotation_y(initial_landmark.yaw) * glam::Vec3::new(0.0, 0.0, 75.0);
        let aircraft_throttle = 0.75;
        let active_transport_mode = config.active_mode;

        let mut region_cycle: Vec<&'static str> = all_us_cities().iter().map(|c| c.id).collect();
        region_cycle.extend_from_slice(&[
            "bengaluru",
            "mysuru",
            "chennai",
            "hyderabad",
            "corridors",
        ]);

        let region_cycle_idx = region_cycle
            .iter()
            .position(|r| *r == config.region_id)
            .unwrap_or(0);

        let is_drive = config.is_drive;

        let mut app = Self {
            config: config.clone(),
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

            previous_vehicle_pos: vehicle.position,
            previous_vehicle_orient: vehicle.orientation,
            vehicle,
            vehicle_cam,
            physics_accumulator: 0.0,
            smoothed_steer_input: 0.0,
            park_engaged: false,
            wheel_spin_angle: 0.0,
            traffic_sim: TrafficSimulation::new(),
            vehicle_visual_type: VehicleVisualType::SportsCoupe,
            gameplay_bus,
            player_profile,
            active_job: None,
            route_tracker: None,
            active_nav_update: None,

            active_transport_mode,
            motorcycle,
            transit_bus,
            heavy_truck,
            train_consist,
            train_position,
            train_yaw,
            marine_vessel,
            marine_elapsed_time,
            aircraft,
            aircraft_throttle,

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
                "Welcome to Real Earth World! Press [H] for HUD, [Esc] for Pause Menu, [F3] for Debug".to_string(),
                Instant::now(),
            )),
            region_cycle,
            region_cycle_idx,
            catalog_selected_idx: 0,

            ui_state: if is_drive {
                AppUiState::InGame
            } else {
                AppUiState::StartScreen
            },
            previous_ui_state: AppUiState::StartScreen,
            cursor_pos: None,
            mouse_left_down: false,
            start_menu_selected_idx: 0,
            pause_menu_selected_idx: 0,
            job_browser_selected_idx: 0,
            garage_selected_idx: 0,
            garage_selected_color_idx: 0,
            vehicle_custom_color: None,
            audio_engine: realearth_gameplay::audio::VehicleAudioEngine::new(
                realearth_gameplay::audio::VehicleAudioPreset::SportsCoupe,
            ),
            settings_selected_idx: 0,
            show_debug_overlay: false,
            use_metric_units: true,
            active_debrief: None,

            multiplayer_client: None,
            multiplayer_server_handle: None,
            multiplayer_selected_action: 0,
            multiplayer_server_addr_input: config.multiplayer_server.clone().unwrap_or_else(|| "127.0.0.1:7777".to_string()),
            transition_phase: RegionTransitionPhase::Idle,
        };

        if let Some(port) = app.config.host_server_port {
            app.host_and_join_local_server(port);
        } else if let Some(ref addr_str) = app.config.multiplayer_server.clone() {
            app.join_multiplayer_server(addr_str);
        }

        app
    }

    /// Hosts a dedicated headless authoritative server on a background thread and connects local client.
    pub fn host_and_join_local_server(&mut self, port: u16) {
        self.disconnect_multiplayer();
        let bind_addr = format!("127.0.0.1:{}", port);
        match realearth_networking::HeadlessAuthoritativeServer::spawn_in_background(
            &bind_addr, 60, 16,
        ) {
            Ok((bound_addr, running, handle)) => {
                println!(
                    "🌐 Dedicated authoritative server spawned on {}",
                    bound_addr
                );
                self.multiplayer_server_handle = Some((running, handle));
                self.multiplayer_server_addr_input = bound_addr.to_string();

                match realearth_networking::MultiplayerClient::new(
                    &self.config.player_name,
                    bound_addr,
                ) {
                    Ok(mut client) => {
                        let _ = client.connect();
                        self.multiplayer_client = Some(client);
                        self.status_notification = Some((
                            format!("Hosting and joined local server on {}", bound_addr),
                            Instant::now(),
                        ));
                    }
                    Err(e) => {
                        eprintln!("Failed to connect local client: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to host local server on {}: {}", bind_addr, e);
                self.status_notification =
                    Some((format!("Failed to host server: {}", e), Instant::now()));
            }
        }
    }

    /// Connects to a remote or local authoritative multiplayer server.
    pub fn join_multiplayer_server(&mut self, addr_str: &str) {
        self.disconnect_multiplayer();
        let target_addr: std::net::SocketAddr = match addr_str.parse() {
            Ok(addr) => addr,
            Err(_) => {
                let fallback = "127.0.0.1:7777".parse().unwrap();
                println!(
                    "Invalid address '{}', falling back to {}",
                    addr_str, fallback
                );
                fallback
            }
        };

        match realearth_networking::MultiplayerClient::new(&self.config.player_name, target_addr) {
            Ok(mut client) => {
                let _ = client.connect();
                self.multiplayer_client = Some(client);
                self.multiplayer_server_addr_input = target_addr.to_string();
                self.status_notification = Some((
                    format!("Connecting to server at {}...", target_addr),
                    Instant::now(),
                ));
            }
            Err(e) => {
                eprintln!("Failed to connect to server at {}: {}", target_addr, e);
                self.status_notification =
                    Some((format!("Connection failed: {}", e), Instant::now()));
            }
        }
    }

    /// Cleanly disconnects multiplayer client and shuts down local server if hosting.
    pub fn disconnect_multiplayer(&mut self) {
        if let Some(mut client) = self.multiplayer_client.take() {
            client.disconnect("User disconnected");
            println!("Disconnected multiplayer client.");
        }
        if let Some((running, handle)) = self.multiplayer_server_handle.take() {
            running.store(false, std::sync::atomic::Ordering::Relaxed);
            let _ = handle.join();
            println!("Stopped local authoritative server thread.");
        }
        self.status_notification = Some((
            "Disconnected from multiplayer. Single-player mode active.".to_string(),
            Instant::now(),
        ));
    }

    /// Performs an atomic quicksave of current profile, active job, and vehicle coordinates.
    pub fn perform_quicksave(&mut self) {
        let save_dir = if std::path::Path::new("target").exists() {
            PathBuf::from("target/saves")
        } else {
            PathBuf::from("saves")
        };
        let _ = std::fs::create_dir_all(&save_dir);
        let save_path = save_dir.join("quicksave.json");

        let veh_coord = if let Some(ref res) = self.region_resources {
            let meters_per_deg_lat = 111_320.0;
            let meters_per_deg_lon =
                111_320.0 * res.package_anchor.lat.to_radians().cos().abs().max(0.01);
            Wgs84Coord::new(
                res.package_anchor.lat + (self.vehicle.position.z as f64 / meters_per_deg_lat),
                res.package_anchor.lon + (self.vehicle.position.x as f64 / meters_per_deg_lon),
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
            settings: GameSettings {
                metric_units: self.use_metric_units,
                abs_enabled: self.vehicle.config.abs_enabled,
                tcs_enabled: self.vehicle.config.tcs_enabled,
                esc_enabled: self.vehicle.config.esc_enabled,
                custom_vehicle_color: self.vehicle_custom_color,
                ..GameSettings::default()
            },
        };

        match SaveManager::save_to_file(&save_path, &save_data) {
            Ok(_) => {
                let msg = format!(
                    "💾 Quicksave saved (Wallet: ${:.2}, Completed: {})",
                    self.player_profile.wallet.balance(),
                    self.player_profile.completed_jobs
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

    /// Performs quickload restoring player profile, career progress, and active jobs.
    pub fn perform_quickload(&mut self) -> bool {
        let save_path = if PathBuf::from("target/saves/quicksave.json").exists() {
            PathBuf::from("target/saves/quicksave.json")
        } else {
            PathBuf::from("saves/quicksave.json")
        };
        if save_path.exists() {
            match SaveManager::load_from_file(&save_path) {
                Ok(loaded) => {
                    self.player_profile = loaded.profile;
                    self.active_job = loaded.active_job;
                    self.use_metric_units = loaded.settings.metric_units;
                    self.vehicle.config.abs_enabled = loaded.settings.abs_enabled;
                    self.vehicle.config.tcs_enabled = loaded.settings.tcs_enabled;
                    self.vehicle.config.esc_enabled = loaded.settings.esc_enabled;
                    if let Some(col) = loaded.settings.custom_vehicle_color {
                        self.vehicle_custom_color = Some(col);
                    }
                    if loaded.active_region_id != self.config.region_id {
                        let _ = self.load_region(&loaded.active_region_id);
                    }
                    let msg = format!(
                        "📂 Quickload restored (Wallet: ${:.2}, Jobs: {})",
                        self.player_profile.wallet.balance(),
                        self.player_profile.completed_jobs
                    );
                    println!("{}", msg);
                    self.status_notification = Some((msg, Instant::now()));
                    true
                }
                Err(e) => {
                    let msg = format!("⚠️ Quickload failed: {}", e);
                    eprintln!("{}", msg);
                    self.status_notification = Some((msg, Instant::now()));
                    false
                }
            }
        } else {
            let msg = "⚠️ No quicksave file found (checked target/saves and saves)".to_string();
            println!("{}", msg);
            self.status_notification = Some((msg, Instant::now()));
            false
        }
    }

    /// Checks whether a valid quicksave file exists on disk.
    pub fn check_saved_game_exists() -> bool {
        PathBuf::from("target/saves/quicksave.json").exists()
            || PathBuf::from("saves/quicksave.json").exists()
    }

    /// Checks whether a valid quicksave file exists.
    pub fn has_saved_game(&self) -> bool {
        Self::check_saved_game_exists()
    }

    /// Centralized UI domain action router.
    /// Dispatched identically by mouse clicks, keyboard keys, or future controller inputs.
    pub fn dispatch_ui_action(
        &mut self,
        action: UiDomainAction,
        event_loop: Option<&ActiveEventLoop>,
    ) {
        self.pressed_keys.clear();
        self.smoothed_steer_input = 0.0;
        match action {
            UiDomainAction::ContinueCareer => {
                if self.has_saved_game() {
                    let _ = self.perform_quickload();
                    self.config.is_drive = true;
                    self.ui_state = AppUiState::InGame;
                } else {
                    let msg = "⚠️ No saved career found. Select Free Roam or Start New Career."
                        .to_string();
                    println!("{}", msg);
                    self.status_notification = Some((msg, Instant::now()));
                }
            }
            UiDomainAction::FreeRoam => {
                self.config.is_drive = true;
                self.ui_state = AppUiState::InGame;
            }
            UiDomainAction::OpenContracts => {
                self.previous_ui_state = self.ui_state;
                self.ui_state = AppUiState::JobBrowser;
            }
            UiDomainAction::OpenGeography => {
                self.previous_ui_state = self.ui_state;
                self.ui_state = AppUiState::CatalogModal;
            }
            UiDomainAction::OpenGarage => {
                self.previous_ui_state = self.ui_state;
                self.ui_state = AppUiState::GarageSelect;
            }
            UiDomainAction::OpenSettings => {
                self.previous_ui_state = self.ui_state;
                self.ui_state = AppUiState::SettingsMenu;
            }
            UiDomainAction::ExitApplication => {
                println!("Exit selected from UI domain action router.");
                if let Some(el) = event_loop {
                    el.exit();
                }
            }
            UiDomainAction::CancelOrBack => {
                let target = match self.ui_state {
                    AppUiState::StartScreen => {
                        if let Some(el) = event_loop {
                            println!("Escape pressed on Start Screen: exiting Real Earth.");
                            el.exit();
                        }
                        AppUiState::StartScreen
                    }
                    _ => self.previous_ui_state,
                };
                self.ui_state = target;
            }
            UiDomainAction::ResumeSimulation => {
                self.ui_state = AppUiState::InGame;
            }
            UiDomainAction::RoadsideRecovery => {
                self.perform_roadside_recovery();
                self.ui_state = AppUiState::InGame;
            }
            UiDomainAction::QuicksaveProgress => {
                self.perform_quicksave();
            }
            UiDomainAction::QuickloadProgress => {
                let _ = self.perform_quickload();
            }
            UiDomainAction::ReturnToStartScreen => {
                self.previous_ui_state = AppUiState::InGame;
                self.ui_state = AppUiState::StartScreen;
            }
            UiDomainAction::AcceptSelectedContract => {
                let available = JobCatalogGenerator::jobs_for_region(&self.config.region_id);
                if self.job_browser_selected_idx < available.len() {
                    let job = available[self.job_browser_selected_idx].clone();
                    let can_accept =
                        self.player_profile.career_tier(job.career) >= job.required_tier;
                    if can_accept {
                        let mut accepted = job;
                        let _ = accepted.accept(&self.player_profile, &self.gameplay_bus);
                        let label = format!(
                            "📋 Accepted Contract: {} (Payout: ${:.2})",
                            accepted.title, accepted.base_payout
                        );
                        println!("{}", label);
                        self.active_job = Some(accepted);
                        self.status_notification = Some((label, Instant::now()));
                        self.config.is_drive = true;
                        self.ui_state = AppUiState::InGame;
                    } else {
                        let msg =
                            format!("🔒 Career Tier Locked: Requires {:?}", job.required_tier);
                        println!("{}", msg);
                        self.status_notification = Some((msg, Instant::now()));
                    }
                }
            }
            UiDomainAction::SelectAndSpawnVehicle(idx) => match self.execute_spawn_vehicle(idx) {
                Ok(()) => {
                    self.ui_state = AppUiState::InGame;
                }
                Err(e) => {
                    let msg = format!("⚠️ Spawning Failed: {}", e);
                    eprintln!("{}", msg);
                    self.status_notification = Some((msg, Instant::now()));
                }
            },
            UiDomainAction::ToggleSelectedSetting(idx) => {
                self.settings_selected_idx = idx;
                match idx {
                    0 => {
                        self.use_metric_units = !self.use_metric_units;
                        let msg = format!(
                            "Speed Units: {}",
                            if self.use_metric_units {
                                "KM/H (Metric)"
                            } else {
                                "MPH (Imperial)"
                            }
                        );
                        self.status_notification = Some((msg, Instant::now()));
                    }
                    1 => {
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
                        let msg = format!("Graphics Preset: {:?}", next_preset);
                        self.status_notification = Some((msg, Instant::now()));
                    }
                    2 => {
                        self.vehicle_cam.cycle_perspective();
                        let msg = format!("Camera Perspective: {:?}", self.vehicle_cam.perspective);
                        self.status_notification = Some((msg, Instant::now()));
                    }
                    3 => {
                        self.show_debug_overlay = !self.show_debug_overlay;
                        let msg = format!(
                            "Developer Overlay: {}",
                            if self.show_debug_overlay { "ON" } else { "OFF" }
                        );
                        self.status_notification = Some((msg, Instant::now()));
                    }
                    4 => {
                        self.show_hud = !self.show_hud;
                        let msg =
                            format!("HUD Display: {}", if self.show_hud { "ON" } else { "OFF" });
                        self.status_notification = Some((msg, Instant::now()));
                    }
                    _ => {}
                }
            }
            UiDomainAction::WarpToSelectedCity(idx) => {
                let cities = all_us_cities();
                if idx < cities.len() {
                    self.catalog_selected_idx = idx;
                    let city = &cities[idx];

                    if let Err(e) = realearth_ingestion::authentic::verify_city_playable(city.id) {
                        self.transition_phase = RegionTransitionPhase::Failed;
                        let msg = format!("⚠️ Download Required: {}", e);
                        eprintln!("{}", msg);
                        self.status_notification = Some((msg, Instant::now()));
                        return;
                    }

                    self.transition_phase = RegionTransitionPhase::Selected;
                    println!(
                        "🌐 Initiating region transition to {} ({})...",
                        city.name, city.state
                    );
                    self.transition_phase = RegionTransitionPhase::CheckingAssets;
                    self.transition_phase = RegionTransitionPhase::Verifying;
                    self.transition_phase = RegionTransitionPhase::Loading;

                    match self.load_region(city.id) {
                        Ok(()) => {
                            self.config.region_id = city.id.to_string();
                            self.config.is_drive = true;
                            self.transition_phase = RegionTransitionPhase::Complete;
                            self.ui_state = AppUiState::InGame;
                            let msg = format!(
                                "🌍 Arrived in {} ({}) — Anchor: {}",
                                city.name, city.state, city.anchor_landmark_name
                            );
                            println!("{}", msg);
                            self.status_notification = Some((msg, Instant::now()));
                        }
                        Err(e) => {
                            self.transition_phase = RegionTransitionPhase::Failed;
                            let msg = format!(
                                "⚠️ Region Transition to {} ({}) Failed: {}",
                                city.name, city.state, e
                            );
                            eprintln!("{}", msg);
                            self.status_notification = Some((msg, Instant::now()));
                        }
                    }
                }
            }
        }

        if let Some(ref window) = self.window {
            window.set_cursor_visible(self.ui_state != AppUiState::InGame);
        }
    }

    /// Stabilizes and recovers vehicle onto the ground surface with upright orientation.
    pub fn perform_roadside_recovery(&mut self) {
        let ground_y = if let Some(ref gq) = self.ground_query {
            match self.active_transport_mode {
                ActiveTransportMode::Car => {
                    gq.elevation(self.vehicle.position.x, self.vehicle.position.z)
                }
                ActiveTransportMode::Motorcycle => {
                    gq.elevation(self.motorcycle.position.x, self.motorcycle.position.z)
                }
                ActiveTransportMode::TransitBus => {
                    gq.elevation(self.transit_bus.position.x, self.transit_bus.position.z)
                }
                ActiveTransportMode::HeavyTruck => gq.elevation(
                    self.heavy_truck.tractor_position.x,
                    self.heavy_truck.tractor_position.z,
                ),
                ActiveTransportMode::Train => {
                    gq.elevation(self.train_position.x, self.train_position.z)
                }
                ActiveTransportMode::Marine => 0.0,
                ActiveTransportMode::Aviation => {
                    gq.elevation(self.aircraft.position.x, self.aircraft.position.z)
                }
            }
        } else {
            0.0
        };
        match self.active_transport_mode {
            ActiveTransportMode::Car => self.vehicle.recover_on_surface(ground_y),
            ActiveTransportMode::Motorcycle => self.motorcycle.recover_upright(),
            ActiveTransportMode::TransitBus => {
                self.transit_bus.position.y = ground_y + 0.95;
                self.transit_bus.linear_velocity = glam::Vec3::ZERO;
                self.transit_bus.speed_kph = 0.0;
            }
            ActiveTransportMode::HeavyTruck => {
                self.heavy_truck.is_rolled_over = false;
                self.heavy_truck.tractor_position.y = ground_y + 1.2;
                self.heavy_truck.linear_velocity = glam::Vec3::ZERO;
                self.heavy_truck.speed_kph = 0.0;
                self.heavy_truck.trailer_articulation_angle = 0.0;
            }
            ActiveTransportMode::Train => {
                self.train_consist.reset_emergency_brake();
                self.train_consist.velocity_mps = 0.0;
                self.train_consist.throttle = 0.0;
            }
            ActiveTransportMode::Marine => {
                self.marine_vessel.position.y = 0.0;
                self.marine_vessel.linear_velocity = glam::Vec3::ZERO;
                self.marine_vessel.roll_angle = 0.0;
                self.marine_vessel.pitch_angle = 0.0;
            }
            ActiveTransportMode::Aviation => {
                self.aircraft.position.y = ground_y + 350.0;
                let yaw = self.aircraft.orientation.to_euler(glam::EulerRot::YXZ).0;
                self.aircraft.linear_velocity =
                    glam::Quat::from_rotation_y(yaw) * glam::Vec3::new(0.0, 0.0, 75.0);
                self.aircraft.is_stalled = false;
            }
        }
        self.previous_vehicle_pos = self.vehicle.position;
        self.previous_vehicle_orient = self.vehicle.orientation;
        self.smoothed_steer_input = 0.0;
        self.vehicle_cam.reset();
        let label = format!(
            "🛠️ {} Safely Recovered to Surface",
            self.active_transport_mode.display_name()
        );
        println!("{}", label);
        self.status_notification = Some((label, Instant::now()));
    }

    /// Resolves an authentic, verified road-lane spawn candidate close to the target coordinate.
    pub fn find_road_spawn_candidate(
        &self,
        target_coord: Wgs84Coord,
        idx: usize,
    ) -> Result<RoadSpawnCandidate, String> {
        let res = self
            .region_resources
            .as_ref()
            .ok_or_else(|| "Region resources not loaded".to_string())?;
        let gq = self
            .ground_query
            .as_ref()
            .ok_or_else(|| "DEM ground query not available".to_string())?;
        let package_anchor = res.package_anchor;
        let conv = EnuConverter::new(package_anchor);

        let is_heavy = idx == 4 || idx == 5; // Bus or Heavy Truck
        let mut best_candidate: Option<(f64, RoadSpawnCandidate)> = None;

        for road in &res.raw_roads {
            if road.layer != 0 || road.is_tunnel {
                continue;
            }
            if road.highway_type == HighwayType::Footway || road.highway_type == HighwayType::Path {
                continue;
            }
            if is_heavy && road.lanes < 2 {
                continue;
            }
            if road.geometry.len() < 2 {
                continue;
            }

            for window in road.geometry.windows(2) {
                let p1 = window[0];
                let p2 = window[1];

                let enu1 = conv.wgs84_to_enu(p1);
                let enu2 = conv.wgs84_to_enu(p2);
                let dx = (enu2.east - enu1.east) as f32;
                let dz = (enu2.north - enu1.north) as f32;
                let seg_len = (dx * dx + dz * dz).sqrt();
                if seg_len < 3.0 {
                    continue;
                }

                let tangent = (dx / seg_len, dz / seg_len);
                let heading_yaw = tangent.0.atan2(tangent.1);

                // Lane lateral offset perpendicular to tangent
                let lane_width = road.highway_type.default_lane_width().max(2.8);
                let right_normal = (tangent.1, -tangent.0);
                let lateral_offset = lane_width * 0.45;

                let mid_enu_x =
                    ((enu1.east + enu2.east) * 0.5) as f32 + right_normal.0 * lateral_offset;
                let mid_enu_z =
                    ((enu1.north + enu2.north) * 0.5) as f32 + right_normal.1 * lateral_offset;

                let elev = gq.elevation(mid_enu_x, mid_enu_z);
                let normal = gq.normal(mid_enu_x, mid_enu_z);
                if normal.y < 0.65 {
                    // Excessive slope / cliff: skip segment
                    continue;
                }

                // Verify collision chunk is loaded and ready
                let mid_wgs84 = conv.enu_to_wgs84(realearth_shared::coordinates::EnuCoord::new(
                    mid_enu_x as f64,
                    mid_enu_z as f64,
                    elev as f64,
                ));
                if !res.streamer.is_collision_ready(mid_wgs84) {
                    continue;
                }

                // Building footprint collision and clearance check (>= lane_width * 0.5 + 1.5m)
                let mut building_conflict = false;
                let req_clearance = (lane_width * 0.5 + 1.5) as f64;
                let lat_margin = 35.0 / 111_000.0;
                let cos_lat = mid_wgs84.lat.to_radians().cos().abs().max(0.1);
                let lon_margin = 35.0 / (111_000.0 * cos_lat);

                for b in &res.raw_buildings {
                    if b.outer_ring.len() < 3 {
                        continue;
                    }
                    let mut b_min_lat = f64::MAX;
                    let mut b_max_lat = f64::MIN;
                    let mut b_min_lon = f64::MAX;
                    let mut b_max_lon = f64::MIN;
                    for pt in &b.outer_ring {
                        b_min_lat = b_min_lat.min(pt.lat);
                        b_max_lat = b_max_lat.max(pt.lat);
                        b_min_lon = b_min_lon.min(pt.lon);
                        b_max_lon = b_max_lon.max(pt.lon);
                    }

                    if mid_wgs84.lat < b_min_lat - lat_margin
                        || mid_wgs84.lat > b_max_lat + lat_margin
                        || mid_wgs84.lon < b_min_lon - lon_margin
                        || mid_wgs84.lon > b_max_lon + lon_margin
                    {
                        continue;
                    }

                    // Convert building ring to local ENU
                    let enu_ring: Vec<(f64, f64)> = b
                        .outer_ring
                        .iter()
                        .map(|pt| {
                            let e = conv.wgs84_to_enu(*pt);
                            (e.east, e.north)
                        })
                        .collect();

                    let cand_px = mid_enu_x as f64;
                    let cand_pz = mid_enu_z as f64;

                    // Point-in-polygon ray casting
                    let mut inside = false;
                    let n = enu_ring.len();
                    for i in 0..n {
                        let j = (i + n - 1) % n;
                        let (xi, zi) = enu_ring[i];
                        let (xj, zj) = enu_ring[j];
                        if ((zi > cand_pz) != (zj > cand_pz))
                            && (cand_px < (xj - xi) * (cand_pz - zi) / (zj - zi) + xi)
                        {
                            inside = !inside;
                        }
                    }

                    if inside {
                        building_conflict = true;
                        break;
                    }

                    // Edge clearance check
                    for i in 0..n {
                        let j = (i + 1) % n;
                        let (x1, z1) = enu_ring[i];
                        let (x2, z2) = enu_ring[j];
                        let dx = x2 - x1;
                        let dz = z2 - z1;
                        let len_sq = dx * dx + dz * dz;
                        let t = if len_sq > 1e-6 {
                            (((cand_px - x1) * dx + (cand_pz - z1) * dz) / len_sq).clamp(0.0, 1.0)
                        } else {
                            0.0
                        };
                        let proj_x = x1 + t * dx;
                        let proj_z = z1 + t * dz;
                        let edge_dist_sq = (cand_px - proj_x).powi(2) + (cand_pz - proj_z).powi(2);
                        if edge_dist_sq < req_clearance * req_clearance {
                            building_conflict = true;
                            break;
                        }
                    }

                    if building_conflict {
                        break;
                    }
                }

                if building_conflict {
                    continue;
                }

                // Distance squared to target coordinate
                let target_enu = conv.wgs84_to_enu(target_coord);
                let dist_sq = (mid_enu_x as f64 - target_enu.east).powi(2)
                    + (mid_enu_z as f64 - target_enu.north).powi(2);

                if best_candidate
                    .as_ref()
                    .is_none_or(|(best_dist, _)| dist_sq < *best_dist)
                {
                    let candidate = RoadSpawnCandidate {
                        position: glam::Vec3::new(mid_enu_x, elev, mid_enu_z),
                        yaw: heading_yaw,
                        surface_elevation: elev,
                        surface_normal: normal,
                        road_name: road
                            .name
                            .clone()
                            .unwrap_or_else(|| "City Street".to_string()),
                        highway_type: road.highway_type,
                        lane_width,
                    };
                    best_candidate = Some((dist_sq, candidate));
                }
            }
        }

        if let Some((_, candidate)) = best_candidate {
            Ok(candidate)
        } else {
            Err(format!(
                "No verified roadway geometry found with safe building clearance (>= {:.1}m) and ready collision chunks for vehicle type {}",
                2.8 * 0.5 + 1.5,
                idx
            ))
        }
    }

    /// Prepares vehicle models at the authentic ground surface without activating drive mode or moving camera.
    pub fn prepare_initial_vehicle(&mut self, idx: usize) {
        let (landmark, package_anchor) = if let Some(ref r) = self.region_resources {
            (r.landmark, r.package_anchor)
        } else {
            (
                Landmark::SAN_FRANCISCO_TRANSAMERICA,
                Landmark::SAN_FRANCISCO_TRANSAMERICA.coord,
            )
        };

        let ground_y = if let Some(ref gq) = self.ground_query {
            gq.elevation(0.0, 0.0)
        } else {
            0.0
        };

        let (spawn_x, spawn_z, spawn_yaw, elev) = if idx <= 6 {
            if let Ok(candidate) = self.find_road_spawn_candidate(landmark.coord, idx) {
                (
                    candidate.position.x,
                    candidate.position.z,
                    candidate.yaw,
                    candidate.surface_elevation,
                )
            } else {
                (0.0, 0.0, landmark.yaw, ground_y)
            }
        } else if idx == 7 {
            let conv = EnuConverter::new(package_anchor);
            let lm_enu = conv.wgs84_to_enu(landmark.coord);
            let e = if let Some(ref gq) = self.ground_query {
                gq.elevation(lm_enu.east as f32, lm_enu.north as f32)
            } else {
                ground_y
            };
            (lm_enu.east as f32, lm_enu.north as f32, landmark.yaw, e)
        } else if idx == 8 {
            (250.0, -100.0, landmark.yaw, 0.5)
        } else if idx == 9 {
            (0.0, -200.0, landmark.yaw, ground_y + 350.0)
        } else {
            (0.0, 0.0, landmark.yaw, ground_y)
        };

        match idx {
            0 => {
                self.active_transport_mode = ActiveTransportMode::Car;
                self.vehicle_visual_type = VehicleVisualType::SportsCoupe;
                self.vehicle = DynamicVehicle::new(
                    VehicleConfig::sports_coupe(),
                    glam::Vec3::new(spawn_x, elev, spawn_z),
                    spawn_yaw,
                );
                self.vehicle.recover_on_surface(elev);
            }
            1 => {
                self.active_transport_mode = ActiveTransportMode::Car;
                self.vehicle_visual_type = VehicleVisualType::Sedan;
                self.vehicle = DynamicVehicle::new(
                    VehicleConfig::sedan(),
                    glam::Vec3::new(spawn_x, elev, spawn_z),
                    spawn_yaw,
                );
                self.vehicle.recover_on_surface(elev);
            }
            2 => {
                self.active_transport_mode = ActiveTransportMode::Car;
                self.vehicle_visual_type = VehicleVisualType::Suv;
                self.vehicle = DynamicVehicle::new(
                    VehicleConfig::suv(),
                    glam::Vec3::new(spawn_x, elev, spawn_z),
                    spawn_yaw,
                );
                self.vehicle.recover_on_surface(elev);
            }
            3 => {
                self.active_transport_mode = ActiveTransportMode::Car;
                self.vehicle_visual_type = VehicleVisualType::Taxi;
                self.vehicle = DynamicVehicle::new(
                    VehicleConfig::taxi(),
                    glam::Vec3::new(spawn_x, elev, spawn_z),
                    spawn_yaw,
                );
                self.vehicle.recover_on_surface(elev);
            }
            4 => {
                self.active_transport_mode = ActiveTransportMode::TransitBus;
                self.vehicle_visual_type = VehicleVisualType::TransitBus;
                self.transit_bus =
                    TransitBusModel::new(glam::Vec3::new(spawn_x, elev, spawn_z), spawn_yaw);
            }
            5 => {
                self.active_transport_mode = ActiveTransportMode::HeavyTruck;
                self.vehicle_visual_type = VehicleVisualType::HeavyTruck;
                self.heavy_truck =
                    HeavyTruckModel::new(glam::Vec3::new(spawn_x, elev, spawn_z), spawn_yaw);
            }
            6 => {
                self.active_transport_mode = ActiveTransportMode::Motorcycle;
                self.vehicle_visual_type = VehicleVisualType::Motorcycle;
                self.motorcycle =
                    MotorcycleModel::new(glam::Vec3::new(spawn_x, elev, spawn_z), spawn_yaw);
            }
            7 => {
                self.active_transport_mode = ActiveTransportMode::Train;
                self.vehicle_visual_type = VehicleVisualType::TrainLocomotive;
                self.train_position = glam::Vec3::new(spawn_x, elev + 0.6, spawn_z);
                self.train_yaw = spawn_yaw;
            }
            8 => {
                self.active_transport_mode = ActiveTransportMode::Marine;
                self.vehicle_visual_type = VehicleVisualType::MarineVessel;
                self.marine_vessel = MarineVessel::new_ferry(
                    "Harbor Passenger Ferry",
                    glam::Vec3::new(spawn_x, elev, spawn_z),
                    spawn_yaw,
                );
            }
            9 => {
                self.active_transport_mode = ActiveTransportMode::Aviation;
                self.vehicle_visual_type = VehicleVisualType::CommercialAirliner;
                self.aircraft = AircraftFlight::new_airliner(
                    "Commercial Airliner",
                    glam::Vec3::new(spawn_x, elev, spawn_z),
                    spawn_yaw,
                );
            }
            _ => {}
        }
        self.previous_vehicle_pos = self.vehicle.position;
        self.previous_vehicle_orient = self.vehicle.orientation;
        self.smoothed_steer_input = 0.0;
        self.vehicle_cam.reset();
        self.vehicle_custom_color = Some(
            realearth_rendering::GARAGE_PAINT_PALETTE[self
                .garage_selected_color_idx
                .min(realearth_rendering::GARAGE_PAINT_PALETTE.len() - 1)]
            .1,
        );
    }

    /// Executes validated multimodal vehicle spawning on appropriate infrastructure with camera synchronization.
    pub fn execute_spawn_vehicle(&mut self, idx: usize) -> Result<(), String> {
        let (_landmark, package_anchor) = self
            .region_resources
            .as_ref()
            .map(|r| (r.landmark, r.package_anchor))
            .ok_or_else(|| "Region resources not loaded".to_string())?;

        let gq = self
            .ground_query
            .as_ref()
            .ok_or_else(|| "Ground query not available".to_string())?;

        let ground_y = gq.elevation(0.0, 0.0);

        let (spawn_x, spawn_z, spawn_yaw, elev) = if idx <= 6 {
            let landmark_coord = self
                .region_resources
                .as_ref()
                .map(|r| r.landmark.coord)
                .unwrap_or(package_anchor);
            let candidate = self.find_road_spawn_candidate(landmark_coord, idx)?;
            (
                candidate.position.x,
                candidate.position.z,
                candidate.yaw,
                candidate.surface_elevation,
            )
        } else if idx == 7 {
            // Rail
            let landmark_coord = self
                .region_resources
                .as_ref()
                .map(|r| r.landmark.coord)
                .unwrap_or(package_anchor);
            let conv = EnuConverter::new(package_anchor);
            let lm_enu = conv.wgs84_to_enu(landmark_coord);
            let e = gq.elevation(lm_enu.east as f32, lm_enu.north as f32);
            (lm_enu.east as f32, lm_enu.north as f32, 0.0, e)
        } else if idx == 8 {
            // Marine
            (250.0, -100.0, 0.0, 0.5)
        } else if idx == 9 {
            // Aviation
            (0.0, -200.0, 0.0, ground_y + 350.0)
        } else {
            return Err(format!("Invalid vehicle fleet index: {}", idx));
        };

        match idx {
            0 => {
                self.active_transport_mode = ActiveTransportMode::Car;
                self.vehicle_visual_type = VehicleVisualType::SportsCoupe;
                self.vehicle = DynamicVehicle::new(
                    VehicleConfig::sports_coupe(),
                    glam::Vec3::new(spawn_x, elev, spawn_z),
                    spawn_yaw,
                );
                self.vehicle.recover_on_surface(elev);
            }
            1 => {
                self.active_transport_mode = ActiveTransportMode::Car;
                self.vehicle_visual_type = VehicleVisualType::Sedan;
                self.vehicle = DynamicVehicle::new(
                    VehicleConfig::sedan(),
                    glam::Vec3::new(spawn_x, elev, spawn_z),
                    spawn_yaw,
                );
                self.vehicle.recover_on_surface(elev);
            }
            2 => {
                self.active_transport_mode = ActiveTransportMode::Car;
                self.vehicle_visual_type = VehicleVisualType::Suv;
                self.vehicle = DynamicVehicle::new(
                    VehicleConfig::suv(),
                    glam::Vec3::new(spawn_x, elev, spawn_z),
                    spawn_yaw,
                );
                self.vehicle.recover_on_surface(elev);
            }
            3 => {
                self.active_transport_mode = ActiveTransportMode::Car;
                self.vehicle_visual_type = VehicleVisualType::Taxi;
                self.vehicle = DynamicVehicle::new(
                    VehicleConfig::taxi(),
                    glam::Vec3::new(spawn_x, elev, spawn_z),
                    spawn_yaw,
                );
                self.vehicle.recover_on_surface(elev);
            }
            4 => {
                self.active_transport_mode = ActiveTransportMode::TransitBus;
                self.vehicle_visual_type = VehicleVisualType::TransitBus;
                self.transit_bus =
                    TransitBusModel::new(glam::Vec3::new(spawn_x, elev, spawn_z), spawn_yaw);
            }
            5 => {
                self.active_transport_mode = ActiveTransportMode::HeavyTruck;
                self.vehicle_visual_type = VehicleVisualType::HeavyTruck;
                self.heavy_truck =
                    HeavyTruckModel::new(glam::Vec3::new(spawn_x, elev, spawn_z), spawn_yaw);
            }
            6 => {
                self.active_transport_mode = ActiveTransportMode::Motorcycle;
                self.vehicle_visual_type = VehicleVisualType::Motorcycle;
                self.motorcycle =
                    MotorcycleModel::new(glam::Vec3::new(spawn_x, elev, spawn_z), spawn_yaw);
            }
            7 => {
                self.active_transport_mode = ActiveTransportMode::Train;
                self.vehicle_visual_type = VehicleVisualType::TrainLocomotive;
                self.train_position = glam::Vec3::new(spawn_x, elev + 0.6, spawn_z);
                self.train_yaw = spawn_yaw;
                self.train_consist.velocity_mps = 0.0;
                self.train_consist.speed_kph = 0.0;
                self.train_consist.throttle = 0.0;
                self.train_consist.reverser = 1;
                self.train_consist.brake_pipe_pressure_bar = 5.0;
                self.train_consist.brake_cylinder_pressure_bar = 0.0;
                self.train_consist.acceleration_mps2 = 0.0;
            }
            8 => {
                self.active_transport_mode = ActiveTransportMode::Marine;
                self.vehicle_visual_type = VehicleVisualType::MarineVessel;
                self.marine_vessel = MarineVessel::new_ferry(
                    "Harbor Passenger Ferry",
                    glam::Vec3::new(spawn_x, elev, spawn_z),
                    spawn_yaw,
                );
            }
            9 => {
                self.active_transport_mode = ActiveTransportMode::Aviation;
                self.vehicle_visual_type = VehicleVisualType::CommercialAirliner;
                self.aircraft = AircraftFlight::new_airliner(
                    "Commercial Airliner",
                    glam::Vec3::new(spawn_x, elev, spawn_z),
                    spawn_yaw,
                );
                self.aircraft.linear_velocity =
                    glam::Quat::from_rotation_y(spawn_yaw) * glam::Vec3::new(0.0, 0.0, 75.0);
                self.aircraft.altitude_msl_meters = elev;
                self.aircraft.indicated_airspeed_knots = 145.0;
                self.aircraft.is_airborne = true;
                self.aircraft.is_stalled = false;
            }
            _ => return Err(format!("Invalid vehicle fleet index: {}", idx)),
        }

        self.previous_vehicle_pos = self.vehicle.position;
        self.previous_vehicle_orient = self.vehicle.orientation;
        self.smoothed_steer_input = 0.0;
        self.vehicle_cam.reset();

        self.vehicle_custom_color = Some(
            realearth_rendering::GARAGE_PAINT_PALETTE[self
                .garage_selected_color_idx
                .min(realearth_rendering::GARAGE_PAINT_PALETTE.len() - 1)]
            .1,
        );

        let audio_preset = match idx {
            0 => realearth_gameplay::audio::VehicleAudioPreset::SportsCoupe,
            1 => realearth_gameplay::audio::VehicleAudioPreset::LuxurySedan,
            2 => realearth_gameplay::audio::VehicleAudioPreset::OverlandSuv,
            3 => realearth_gameplay::audio::VehicleAudioPreset::CityTaxi,
            4 => realearth_gameplay::audio::VehicleAudioPreset::TransitBus,
            5 => realearth_gameplay::audio::VehicleAudioPreset::HeavyTruck,
            6 => realearth_gameplay::audio::VehicleAudioPreset::Superbike,
            7 => realearth_gameplay::audio::VehicleAudioPreset::Electric,
            _ => realearth_gameplay::audio::VehicleAudioPreset::SportsCoupe,
        };
        self.audio_engine = realearth_gameplay::audio::VehicleAudioEngine::new(audio_preset);
        self.audio_engine
            .play_ui_cue(realearth_gameplay::audio::UiSoundCue::MenuSelect);

        // Camera perspective & positioning update
        self.vehicle_cam.perspective = VehicleCameraPerspective::Chase;
        let (cam_pos, cam_yaw, cam_pitch) = match self.active_transport_mode {
            ActiveTransportMode::Car => self.vehicle_cam.update_pose_with_ground(
                self.vehicle.position,
                self.vehicle.orientation,
                5.5,
                2.2,
                self.vehicle.speed_kph,
                0.016,
                Some(elev),
            ),
            ActiveTransportMode::Motorcycle => self.vehicle_cam.update_pose_with_ground(
                self.motorcycle.position,
                self.motorcycle.orientation,
                4.2,
                1.6,
                self.motorcycle.speed_kph,
                0.016,
                Some(elev),
            ),
            ActiveTransportMode::TransitBus => self.vehicle_cam.update_pose_with_ground(
                self.transit_bus.position,
                self.transit_bus.orientation,
                16.0,
                4.5,
                self.transit_bus.speed_kph,
                0.016,
                Some(elev),
            ),
            ActiveTransportMode::HeavyTruck => self.vehicle_cam.update_pose_with_ground(
                self.heavy_truck.tractor_position,
                self.heavy_truck.tractor_orientation,
                18.5,
                4.8,
                self.heavy_truck.speed_kph,
                0.016,
                Some(elev),
            ),
            ActiveTransportMode::Train => self.vehicle_cam.update_pose_with_ground(
                self.train_position,
                glam::Quat::from_rotation_y(self.train_yaw),
                28.0,
                6.2,
                self.train_consist.speed_kph,
                0.016,
                Some(elev),
            ),
            ActiveTransportMode::Marine => self.vehicle_cam.update_pose_with_ground(
                self.marine_vessel.position,
                self.marine_vessel.orientation,
                90.0,
                24.0,
                self.marine_vessel.speed_knots * 1.852,
                0.016,
                Some(0.0),
            ),
            ActiveTransportMode::Aviation => self.vehicle_cam.update_pose_with_ground(
                self.aircraft.position,
                self.aircraft.orientation,
                48.0,
                12.5,
                self.aircraft.true_airspeed_mps * 3.6,
                0.016,
                Some(elev),
            ),
        };

        let meters_per_deg_lat = 111_320.0;
        let meters_per_deg_lon = 111_320.0 * package_anchor.lat.to_radians().cos().abs().max(0.01);
        let cam_lat = package_anchor.lat + (cam_pos.z as f64 / meters_per_deg_lat);
        let cam_lon = package_anchor.lon + (cam_pos.x as f64 / meters_per_deg_lon);
        self.camera_ctrl.coord = Wgs84Coord::new(cam_lat, cam_lon, cam_pos.y as f64);
        self.camera_ctrl.yaw = cam_yaw;
        self.camera_ctrl.pitch = cam_pitch;
        self.camera.fov_y = self.vehicle_cam.current_fov_deg.to_radians();
        self.camera_ctrl.update_camera(&mut self.camera);

        // Clear incompatible career mission contract
        if self.active_transport_mode != ActiveTransportMode::Car {
            if let Some(ref j) = self.active_job {
                if j.career == CareerType::TaxiPassenger || j.career == CareerType::PackageDelivery
                {
                    self.active_job = None;
                }
            }
        }

        self.config.is_drive = true;
        self.garage_selected_idx = idx;
        let label = format!("🏎️ Spawned: {}", self.active_transport_mode.display_name());
        println!("{}", label);
        self.status_notification = Some((label, Instant::now()));
        Ok(())
    }

    /// Selects and inspects a vehicle from the fleet garage.
    pub fn apply_garage_selection(&mut self, idx: usize) {
        self.garage_selected_idx = idx;
        let names = [
            "Apex GT Sports Coupe",
            "Executive Sedan",
            "Overland Explorer SUV",
            "Metro Yellow Cab",
            "CityLine Transit Bus",
            "TransContinental Semi",
            "HyperSport RR-1000",
            "Pacific Intercity Consist",
            "Golden Gate Passenger Ferry",
            "SkyCourier B737-800",
        ];
        let name = names.get(idx).copied().unwrap_or("Vehicle");
        let label = format!("🔍 Inspected: {} (Click 'SPAWN VEHICLE' to deploy)", name);
        self.status_notification = Some((label, Instant::now()));
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

        let (world_dir, region_name, landmark, dem, raw_roads, raw_buildings) = if let Some(city) =
            find_city_by_id(region_id)
        {
            let (raw, dem, meta) = load_authentic_city(city.id)?;
            let dir = resolve_world_package_dir(&meta.region_id.replace('-', "_"));
            let landmark = Landmark::from_us_city(city);
            let package_name = format!("{}_package", meta.region_id.replace('-', "_"));
            if !is_world_package_cache_valid(&dir, &package_name, Some(&meta.source_osm_sha256)) {
                println!(
                    "📦 Compiling authoritative {} ({}) package [DEM accuracy: ±{:.1}m, Source SHA256: {}]...",
                    meta.name, meta.state, meta.dem_accuracy_m, &meta.source_osm_sha256[..8]
                );
                let (val, _) = validate_dataset(&raw);
                let options = CompilerOptions {
                    chunk_size_meters: 500.0,
                    terrain_grid_resolution: 16,
                    package_name: package_name.clone(),
                    source_asset_sha256: Some(meta.source_osm_sha256.clone()),
                    ..Default::default()
                };
                compile_world_package(&val, &dem, &dir, &options)?;
            }
            (dir, city.name, landmark, dem, raw.roads, raw.buildings)
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
                raw.buildings,
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
                raw.buildings,
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
                raw.buildings,
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
                raw.buildings,
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
                raw.buildings,
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
                raw.buildings,
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
                raw.buildings,
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
                raw.buildings,
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
                raw.buildings,
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
                raw.buildings,
            )
        } else {
            let dir = resolve_world_package_dir("san_francisco");
            let (raw, dem, sf_source_hash) = match load_authentic_city("san-francisco") {
                Ok((dataset, dem, meta)) => (dataset, dem, Some(meta.source_osm_sha256)),
                Err(e) => {
                    eprintln!("⚠️ Warning: Failed to load authentic San Francisco data ({:?}), using fallback", e);
                    (
                        create_san_francisco_dataset(),
                        DemGrid::create_san_francisco_coastal_dem(64, 64),
                        None,
                    )
                }
            };
            if !is_world_package_cache_valid(
                &dir,
                "san_francisco_package",
                sf_source_hash.as_deref(),
            ) {
                println!("📦 Compiling authentic San Francisco Downtown & Waterfront package...");
                let (val, _) = validate_dataset(&raw);
                let options = CompilerOptions {
                    chunk_size_meters: 500.0,
                    terrain_grid_resolution: 16,
                    package_name: "san_francisco_package".to_string(),
                    source_asset_sha256: sf_source_hash,
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
                raw.buildings,
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

        // Purge renderer persistent GPU chunk cache to avoid local coordinate key collision
        if let Some(ref r) = self.renderer {
            let cleared = r.clear_gpu_cache();
            if cleared > 0 {
                println!(
                    "🧹 Cleared {} stale GPU chunk buffers from previous region",
                    cleared
                );
            }
        }

        let package_anchor = active_chunks
            .first()
            .map(|c| c.header.anchor)
            .unwrap_or(landmark.coord);

        let ground_query = DemGroundQuery::new(dem.clone(), package_anchor);
        let conv = EnuConverter::new(package_anchor);
        let lm_enu = conv.wgs84_to_enu(landmark.coord);
        let initial_elevation = ground_query.elevation(lm_enu.east as f32, lm_enu.north as f32);

        // Initialize Bevy ECS simulation world
        let mut sim_world = RealEarthSimWorld::new(package_anchor);
        for chunk in &active_chunks {
            let enu_c = conv.wgs84_to_enu(chunk.header.bounds.center());
            let center = glam::Vec3::new(
                enu_c.east as f32,
                (chunk.header.min_elevation + chunk.header.max_elevation) * 0.5,
                enu_c.north as f32,
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

        // Initialize vehicle and multimodal fleet at ground surface
        self.vehicle = DynamicVehicle::new(
            VehicleConfig::default(),
            glam::Vec3::new(lm_enu.east as f32, initial_elevation, lm_enu.north as f32),
            landmark.yaw,
        );
        self.vehicle.recover_on_surface(initial_elevation);

        self.motorcycle = MotorcycleModel::new(
            glam::Vec3::new(
                lm_enu.east as f32 + 3.0,
                initial_elevation,
                lm_enu.north as f32,
            ),
            landmark.yaw,
        );
        self.transit_bus = TransitBusModel::new(
            glam::Vec3::new(
                lm_enu.east as f32 - 5.0,
                initial_elevation,
                lm_enu.north as f32,
            ),
            landmark.yaw,
        );
        self.heavy_truck = HeavyTruckModel::new(
            glam::Vec3::new(
                lm_enu.east as f32 + 8.0,
                initial_elevation,
                lm_enu.north as f32,
            ),
            landmark.yaw,
        );
        self.train_position = glam::Vec3::new(
            lm_enu.east as f32,
            initial_elevation + 0.6,
            lm_enu.north as f32 + 20.0,
        );
        self.train_yaw = landmark.yaw;
        self.marine_vessel = MarineVessel::new_ferry(
            "Harbor Passenger Ferry",
            glam::Vec3::new(250.0, 0.0, -100.0),
            landmark.yaw,
        );
        self.aircraft = AircraftFlight::new_airliner(
            "Commercial Airliner",
            glam::Vec3::new(0.0, initial_elevation + 350.0, -500.0),
            landmark.yaw,
        );
        self.aircraft.linear_velocity =
            glam::Quat::from_rotation_y(landmark.yaw) * glam::Vec3::new(0.0, 0.0, 75.0);

        self.previous_vehicle_pos = self.vehicle.position;
        self.previous_vehicle_orient = self.vehicle.orientation;
        self.smoothed_steer_input = 0.0;
        self.vehicle_cam.reset();

        // Populate traffic fleet, signals, and pedestrians
        self.traffic_sim = TrafficSimulation::new();
        let is_left_hand = matches!(
            region_id,
            "bengaluru" | "mysuru" | "chennai" | "hyderabad" | "corridors" | "in"
        );
        let rule = if is_left_hand {
            RuleOfTheRoad::DriveOnLeft
        } else {
            RuleOfTheRoad::DriveOnRight
        };

        if !raw_roads.is_empty() {
            let road_graph = RoadNetworkGraph::from_roads(&raw_roads);
            if self.config.traffic_count > 0 {
                self.traffic_sim.populate_from_road_network(
                    &road_graph,
                    self.config.traffic_count,
                    rule,
                );
            }

            // Populate Navigation Route from Landmark if genuine route exists
            let start_coord = landmark.coord;
            let dest_coord = Wgs84Coord::new(
                landmark.coord.lat + 0.005,
                landmark.coord.lon + 0.004,
                landmark.coord.alt,
            );
            match find_route(
                &road_graph,
                start_coord,
                dest_coord,
                RoutingPreference::FastestTime,
            ) {
                Ok(route) => {
                    let guidance = generate_turn_by_turn(&route, &road_graph);
                    self.route_tracker = Some(RouteTracker::new(route, guidance));
                }
                Err(e) => {
                    println!(
                        "ℹ️ Road navigation unavailable between {:?} and {:?}: {:?}",
                        start_coord, dest_coord, e
                    );
                    self.route_tracker = None;
                }
            }
        } else {
            self.route_tracker = None;
        }

        // Add intersection traffic lights, stop lines, and procedural pedestrians
        let tl_pos = glam::Vec3::new(
            lm_enu.east as f32 + 6.0,
            initial_elevation,
            lm_enu.north as f32 + 60.0,
        );
        let tl = TrafficLight::new(101, tl_pos);
        self.traffic_sim.add_traffic_light(tl);
        self.traffic_sim.add_stop_line(IntersectionStopLine {
            id: 201,
            traffic_light_id: 101,
            position: glam::Vec3::new(
                lm_enu.east as f32,
                initial_elevation,
                lm_enu.north as f32 + 45.0,
            ),
            is_ns_direction: true,
        });

        // Procedural pedestrians walking across crosswalks and along sidewalks
        let ped1 = PedestrianAgent::new(
            501,
            vec![
                glam::Vec3::new(
                    lm_enu.east as f32 - 7.0,
                    initial_elevation,
                    lm_enu.north as f32 + 55.0,
                ),
                glam::Vec3::new(
                    lm_enu.east as f32 + 7.0,
                    initial_elevation,
                    lm_enu.north as f32 + 55.0,
                ),
            ],
            1.2,
        );
        let mut ped2 = PedestrianAgent::new(
            502,
            vec![
                glam::Vec3::new(
                    lm_enu.east as f32 + 7.5,
                    initial_elevation,
                    lm_enu.north as f32 + 25.0,
                ),
                glam::Vec3::new(
                    lm_enu.east as f32 + 7.5,
                    initial_elevation,
                    lm_enu.north as f32 + 85.0,
                ),
            ],
            1.35,
        );
        ped2.walk_cycle_phase = 1.5;
        self.traffic_sim.add_pedestrian(ped1);
        self.traffic_sim.add_pedestrian(ped2);

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

        // Reposition camera floating origin anchor to package anchor coordinates
        self.camera.set_anchor(package_anchor);
        self.camera_ctrl.coord = landmark.coord;
        self.camera_ctrl.yaw = landmark.yaw;
        self.camera_ctrl.pitch = landmark.pitch;
        self.camera_ctrl.update_camera(&mut self.camera);

        self.ground_query = Some(ground_query);
        self.sim_world = Some(sim_world);
        self.region_resources = Some(RegionResources {
            _world_dir: world_dir,
            region_name,
            landmark,
            package_anchor,
            dem,
            raw_roads,
            raw_buildings,
            streamer,
        });

        // Initialize vehicle: spawn if in drive mode, otherwise prepare initial vehicle preview without hijacking camera
        if self.config.is_drive {
            if let Err(e) = self.execute_spawn_vehicle(self.garage_selected_idx) {
                eprintln!("⚠️ Failed to spawn vehicle on region load: {}", e);
            }
        } else {
            self.prepare_initial_vehicle(self.garage_selected_idx);
        }

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
            window.set_cursor_visible(self.ui_state != AppUiState::InGame);

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
                if self.ui_state == AppUiState::InGame {
                    if button == MouseButton::Right || button == MouseButton::Left {
                        self.mouse_drag_active = state == ElementState::Pressed;
                        if !self.mouse_drag_active {
                            self.last_cursor_pos = None;
                        }
                    }
                } else if button == MouseButton::Left {
                    match state {
                        ElementState::Pressed => {
                            self.mouse_left_down = true;
                        }
                        ElementState::Released => {
                            if self.mouse_left_down {
                                self.mouse_left_down = false;
                                let (screen_w, screen_h) =
                                    if let Some(ref config) = self.surface_config {
                                        (config.width as f32, config.height as f32)
                                    } else {
                                        (1920.0, 1080.0)
                                    };
                                if let Some((px, py)) = self.cursor_pos {
                                    match self.ui_state {
                                        AppUiState::StartScreen => {
                                            let layout =
                                                StartScreenLayout::compute(screen_w, screen_h, 7);
                                            match layout.hit_test(px, py) {
                                                Some(StartScreenHit::Option(idx)) => {
                                                    self.start_menu_selected_idx = idx;
                                                    let action = match idx {
                                                        0 => UiDomainAction::ContinueCareer,
                                                        1 => UiDomainAction::FreeRoam,
                                                        2 => UiDomainAction::OpenContracts,
                                                        3 => UiDomainAction::OpenGeography,
                                                        4 => UiDomainAction::OpenGarage,
                                                        5 => UiDomainAction::OpenSettings,
                                                        6 => UiDomainAction::ExitApplication,
                                                        _ => UiDomainAction::FreeRoam,
                                                    };
                                                    self.dispatch_ui_action(
                                                        action,
                                                        Some(event_loop),
                                                    );
                                                }
                                                Some(StartScreenHit::QuickRoam) => {
                                                    self.dispatch_ui_action(
                                                        UiDomainAction::FreeRoam,
                                                        Some(event_loop),
                                                    );
                                                }
                                                Some(StartScreenHit::Close) => {
                                                    self.dispatch_ui_action(
                                                        UiDomainAction::ExitApplication,
                                                        Some(event_loop),
                                                    );
                                                }
                                                None => {}
                                            }
                                        }
                                        AppUiState::PauseMenu => {
                                            let layout =
                                                PauseMenuLayout::compute(screen_w, screen_h, 10);
                                            match layout.hit_test(px, py) {
                                                Some(PauseMenuHit::Option(idx)) => {
                                                    self.pause_menu_selected_idx = idx;
                                                    let action = match idx {
                                                        0 => UiDomainAction::ResumeSimulation,
                                                        1 => UiDomainAction::OpenContracts,
                                                        2 => UiDomainAction::OpenGarage,
                                                        3 => UiDomainAction::OpenGeography,
                                                        4 => UiDomainAction::RoadsideRecovery,
                                                        5 => UiDomainAction::QuicksaveProgress,
                                                        6 => UiDomainAction::QuickloadProgress,
                                                        7 => UiDomainAction::OpenSettings,
                                                        8 => UiDomainAction::ReturnToStartScreen,
                                                        9 => UiDomainAction::ExitApplication,
                                                        _ => UiDomainAction::ResumeSimulation,
                                                    };
                                                    self.dispatch_ui_action(
                                                        action,
                                                        Some(event_loop),
                                                    );
                                                }
                                                Some(PauseMenuHit::Resume)
                                                | Some(PauseMenuHit::Close) => {
                                                    self.dispatch_ui_action(
                                                        UiDomainAction::ResumeSimulation,
                                                        Some(event_loop),
                                                    );
                                                }
                                                None => {}
                                            }
                                        }
                                        AppUiState::JobBrowser => {
                                            let available = JobCatalogGenerator::jobs_for_region(
                                                &self.config.region_id,
                                            );
                                            let layout = JobBrowserLayout::compute(
                                                screen_w,
                                                screen_h,
                                                self.job_browser_selected_idx,
                                                available.len(),
                                            );
                                            match layout.hit_test(px, py) {
                                                Some(JobBrowserHit::JobItem(idx)) => {
                                                    self.job_browser_selected_idx = idx;
                                                }
                                                Some(JobBrowserHit::AcceptContract) => {
                                                    self.dispatch_ui_action(
                                                        UiDomainAction::AcceptSelectedContract,
                                                        Some(event_loop),
                                                    );
                                                }
                                                Some(JobBrowserHit::Back) => {
                                                    self.dispatch_ui_action(
                                                        UiDomainAction::CancelOrBack,
                                                        Some(event_loop),
                                                    );
                                                }
                                                None => {}
                                            }
                                        }
                                        AppUiState::GarageSelect => {
                                            let layout =
                                                GarageLayout::compute(screen_w, screen_h, 10);
                                            match layout.hit_test(px, py) {
                                                Some(GarageHit::VehicleItem(idx)) => {
                                                    self.garage_selected_idx = idx;
                                                }
                                                Some(GarageHit::ColorOption(c_idx)) => {
                                                    self.garage_selected_color_idx = c_idx;
                                                    self.vehicle_custom_color = Some(
                                                        realearth_rendering::GARAGE_PAINT_PALETTE
                                                            [c_idx]
                                                            .1,
                                                    );
                                                    self.audio_engine.play_ui_cue(
                                                        realearth_gameplay::audio::UiSoundCue::MenuNavigate,
                                                    );
                                                }
                                                Some(GarageHit::SelectAndSpawn) => {
                                                    self.dispatch_ui_action(
                                                        UiDomainAction::SelectAndSpawnVehicle(
                                                            self.garage_selected_idx,
                                                        ),
                                                        Some(event_loop),
                                                    );
                                                }
                                                Some(GarageHit::Back) => {
                                                    self.dispatch_ui_action(
                                                        UiDomainAction::CancelOrBack,
                                                        Some(event_loop),
                                                    );
                                                }
                                                None => {}
                                            }
                                        }
                                        AppUiState::SettingsMenu => {
                                            let layout =
                                                SettingsLayout::compute(screen_w, screen_h, 5);
                                            match layout.hit_test(px, py) {
                                                Some(SettingsHit::SettingItem(idx)) => {
                                                    self.settings_selected_idx = idx;
                                                    self.dispatch_ui_action(
                                                        UiDomainAction::ToggleSelectedSetting(idx),
                                                        Some(event_loop),
                                                    );
                                                }
                                                Some(SettingsHit::Back) => {
                                                    self.dispatch_ui_action(
                                                        UiDomainAction::CancelOrBack,
                                                        Some(event_loop),
                                                    );
                                                }
                                                None => {}
                                            }
                                        }
                                        AppUiState::CatalogModal => {
                                            let total = all_us_cities().len();
                                            let layout = CatalogLayout::compute(
                                                screen_w,
                                                screen_h,
                                                self.catalog_selected_idx,
                                                total,
                                            );
                                            match layout.hit_test(px, py) {
                                                Some(CatalogHit::CityItem(idx)) => {
                                                    self.catalog_selected_idx = idx;
                                                }
                                                Some(CatalogHit::WarpCity) => {
                                                    self.dispatch_ui_action(
                                                        UiDomainAction::WarpToSelectedCity(
                                                            self.catalog_selected_idx,
                                                        ),
                                                        Some(event_loop),
                                                    );
                                                }
                                                Some(CatalogHit::Back) => {
                                                    self.dispatch_ui_action(
                                                        UiDomainAction::CancelOrBack,
                                                        Some(event_loop),
                                                    );
                                                }
                                                None => {}
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                let px = position.x as f32;
                let py = position.y as f32;
                self.cursor_pos = Some((px, py));

                // 100% Camera Isolation: do NOT rotate camera when interacting with menus!
                if self.ui_state == AppUiState::InGame && self.mouse_drag_active {
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
                if self.ui_state == AppUiState::InGame {
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
                } else if scroll_y < -1.0 {
                    // Scroll down through menu items
                    match self.ui_state {
                        AppUiState::StartScreen => {
                            self.start_menu_selected_idx =
                                (self.start_menu_selected_idx + 1).min(6);
                        }
                        AppUiState::PauseMenu => {
                            self.pause_menu_selected_idx =
                                (self.pause_menu_selected_idx + 1).min(9);
                        }
                        AppUiState::JobBrowser => {
                            let total =
                                JobCatalogGenerator::jobs_for_region(&self.config.region_id)
                                    .len()
                                    .max(1);
                            self.job_browser_selected_idx =
                                (self.job_browser_selected_idx + 1).min(total - 1);
                        }
                        AppUiState::GarageSelect => {
                            self.garage_selected_idx = (self.garage_selected_idx + 1).min(9);
                        }
                        AppUiState::SettingsMenu => {
                            self.settings_selected_idx = (self.settings_selected_idx + 1).min(4);
                        }
                        AppUiState::CatalogModal => {
                            let total = all_us_cities().len();
                            self.catalog_selected_idx =
                                (self.catalog_selected_idx + 1).min(total.saturating_sub(1));
                        }
                        _ => {}
                    }
                } else if scroll_y > 1.0 {
                    // Scroll up through menu items
                    match self.ui_state {
                        AppUiState::StartScreen => {
                            self.start_menu_selected_idx =
                                self.start_menu_selected_idx.saturating_sub(1);
                        }
                        AppUiState::PauseMenu => {
                            self.pause_menu_selected_idx =
                                self.pause_menu_selected_idx.saturating_sub(1);
                        }
                        AppUiState::JobBrowser => {
                            self.job_browser_selected_idx =
                                self.job_browser_selected_idx.saturating_sub(1);
                        }
                        AppUiState::GarageSelect => {
                            self.garage_selected_idx = self.garage_selected_idx.saturating_sub(1);
                        }
                        AppUiState::SettingsMenu => {
                            self.settings_selected_idx =
                                self.settings_selected_idx.saturating_sub(1);
                        }
                        AppUiState::CatalogModal => {
                            self.catalog_selected_idx = self.catalog_selected_idx.saturating_sub(1);
                        }
                        _ => {}
                    }
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

                    match self.ui_state {
                        AppUiState::StartScreen => {
                            let total_options = 7;
                            match key {
                                KeyCode::ArrowUp | KeyCode::KeyW => {
                                    self.start_menu_selected_idx =
                                        self.start_menu_selected_idx.saturating_sub(1);
                                    return;
                                }
                                KeyCode::ArrowDown | KeyCode::KeyS => {
                                    self.start_menu_selected_idx =
                                        (self.start_menu_selected_idx + 1).min(total_options - 1);
                                    return;
                                }
                                KeyCode::Enter | KeyCode::Space => {
                                    let action = match self.start_menu_selected_idx {
                                        0 => UiDomainAction::ContinueCareer,
                                        1 => UiDomainAction::FreeRoam,
                                        2 => UiDomainAction::OpenContracts,
                                        3 => UiDomainAction::OpenGeography,
                                        4 => UiDomainAction::OpenGarage,
                                        5 => UiDomainAction::OpenSettings,
                                        6 => UiDomainAction::ExitApplication,
                                        _ => UiDomainAction::FreeRoam,
                                    };
                                    self.dispatch_ui_action(action, Some(event_loop));
                                    return;
                                }
                                KeyCode::Escape => {
                                    self.dispatch_ui_action(
                                        UiDomainAction::CancelOrBack,
                                        Some(event_loop),
                                    );
                                    return;
                                }
                                _ => {}
                            }
                        }

                        AppUiState::PauseMenu => {
                            let total_options = 10;
                            match key {
                                KeyCode::ArrowUp | KeyCode::KeyW => {
                                    self.pause_menu_selected_idx =
                                        self.pause_menu_selected_idx.saturating_sub(1);
                                    return;
                                }
                                KeyCode::ArrowDown | KeyCode::KeyS => {
                                    self.pause_menu_selected_idx =
                                        (self.pause_menu_selected_idx + 1).min(total_options - 1);
                                    return;
                                }
                                KeyCode::Escape => {
                                    self.dispatch_ui_action(
                                        UiDomainAction::ResumeSimulation,
                                        Some(event_loop),
                                    );
                                    return;
                                }
                                KeyCode::Enter | KeyCode::Space => {
                                    let action = match self.pause_menu_selected_idx {
                                        0 => UiDomainAction::ResumeSimulation,
                                        1 => UiDomainAction::OpenContracts,
                                        2 => UiDomainAction::OpenGarage,
                                        3 => UiDomainAction::OpenGeography,
                                        4 => UiDomainAction::RoadsideRecovery,
                                        5 => UiDomainAction::QuicksaveProgress,
                                        6 => UiDomainAction::QuickloadProgress,
                                        7 => UiDomainAction::OpenSettings,
                                        8 => UiDomainAction::ReturnToStartScreen,
                                        9 => UiDomainAction::ExitApplication,
                                        _ => UiDomainAction::ResumeSimulation,
                                    };
                                    self.dispatch_ui_action(action, Some(event_loop));
                                    return;
                                }
                                _ => {}
                            }
                        }

                        AppUiState::JobBrowser => {
                            let available =
                                JobCatalogGenerator::jobs_for_region(&self.config.region_id);
                            let total = available.len().max(1);
                            match key {
                                KeyCode::ArrowUp | KeyCode::KeyW => {
                                    self.job_browser_selected_idx =
                                        self.job_browser_selected_idx.saturating_sub(1);
                                    return;
                                }
                                KeyCode::ArrowDown | KeyCode::KeyS => {
                                    self.job_browser_selected_idx = (self.job_browser_selected_idx
                                        + 1)
                                    .min(total.saturating_sub(1));
                                    return;
                                }
                                KeyCode::Enter => {
                                    self.dispatch_ui_action(
                                        UiDomainAction::AcceptSelectedContract,
                                        Some(event_loop),
                                    );
                                    return;
                                }
                                KeyCode::Escape | KeyCode::KeyJ | KeyCode::Tab => {
                                    self.dispatch_ui_action(
                                        UiDomainAction::CancelOrBack,
                                        Some(event_loop),
                                    );
                                    return;
                                }
                                _ => {}
                            }
                        }

                        AppUiState::GarageSelect => {
                            let total_vehicles = 10;
                            match key {
                                KeyCode::ArrowUp | KeyCode::KeyW => {
                                    self.garage_selected_idx =
                                        self.garage_selected_idx.saturating_sub(1);
                                    return;
                                }
                                KeyCode::ArrowDown | KeyCode::KeyS => {
                                    self.garage_selected_idx =
                                        (self.garage_selected_idx + 1).min(total_vehicles - 1);
                                    return;
                                }
                                KeyCode::ArrowLeft | KeyCode::KeyA => {
                                    self.garage_selected_color_idx =
                                        if self.garage_selected_color_idx == 0 {
                                            realearth_rendering::GARAGE_PAINT_PALETTE.len() - 1
                                        } else {
                                            self.garage_selected_color_idx - 1
                                        };
                                    self.vehicle_custom_color = Some(
                                        realearth_rendering::GARAGE_PAINT_PALETTE
                                            [self.garage_selected_color_idx]
                                            .1,
                                    );
                                    self.audio_engine.play_ui_cue(
                                        realearth_gameplay::audio::UiSoundCue::MenuNavigate,
                                    );
                                    return;
                                }
                                KeyCode::ArrowRight | KeyCode::KeyD => {
                                    self.garage_selected_color_idx =
                                        (self.garage_selected_color_idx + 1)
                                            % realearth_rendering::GARAGE_PAINT_PALETTE.len();
                                    self.vehicle_custom_color = Some(
                                        realearth_rendering::GARAGE_PAINT_PALETTE
                                            [self.garage_selected_color_idx]
                                            .1,
                                    );
                                    self.audio_engine.play_ui_cue(
                                        realearth_gameplay::audio::UiSoundCue::MenuNavigate,
                                    );
                                    return;
                                }
                                KeyCode::Enter => {
                                    self.dispatch_ui_action(
                                        UiDomainAction::SelectAndSpawnVehicle(
                                            self.garage_selected_idx,
                                        ),
                                        Some(event_loop),
                                    );
                                    return;
                                }
                                KeyCode::Escape | KeyCode::KeyG => {
                                    self.dispatch_ui_action(
                                        UiDomainAction::CancelOrBack,
                                        Some(event_loop),
                                    );
                                    return;
                                }
                                _ => {}
                            }
                        }

                        AppUiState::SettingsMenu => {
                            let total_settings = 5;
                            match key {
                                KeyCode::ArrowUp | KeyCode::KeyW => {
                                    self.settings_selected_idx =
                                        self.settings_selected_idx.saturating_sub(1);
                                    return;
                                }
                                KeyCode::ArrowDown | KeyCode::KeyS => {
                                    self.settings_selected_idx =
                                        (self.settings_selected_idx + 1).min(total_settings - 1);
                                    return;
                                }
                                KeyCode::Enter => {
                                    self.dispatch_ui_action(
                                        UiDomainAction::ToggleSelectedSetting(
                                            self.settings_selected_idx,
                                        ),
                                        Some(event_loop),
                                    );
                                    return;
                                }
                                KeyCode::Escape | KeyCode::KeyO => {
                                    self.dispatch_ui_action(
                                        UiDomainAction::CancelOrBack,
                                        Some(event_loop),
                                    );
                                    return;
                                }
                                _ => {}
                            }
                        }

                        AppUiState::CatalogModal => {
                            let total = all_us_cities().len();
                            match key {
                                KeyCode::ArrowUp | KeyCode::KeyW => {
                                    self.catalog_selected_idx =
                                        self.catalog_selected_idx.saturating_sub(1);
                                    return;
                                }
                                KeyCode::ArrowDown | KeyCode::KeyS => {
                                    self.catalog_selected_idx = (self.catalog_selected_idx + 1)
                                        .min(total.saturating_sub(1));
                                    return;
                                }
                                KeyCode::Enter => {
                                    self.dispatch_ui_action(
                                        UiDomainAction::WarpToSelectedCity(
                                            self.catalog_selected_idx,
                                        ),
                                        Some(event_loop),
                                    );
                                    return;
                                }
                                KeyCode::Escape | KeyCode::KeyU | KeyCode::F2 => {
                                    self.dispatch_ui_action(
                                        UiDomainAction::CancelOrBack,
                                        Some(event_loop),
                                    );
                                    return;
                                }
                                _ => {}
                            }
                        }

                        AppUiState::DebriefModal => match key {
                            KeyCode::Enter | KeyCode::Escape | KeyCode::Space => {
                                self.active_debrief = None;
                                self.ui_state = AppUiState::InGame;
                                return;
                            }
                            _ => {}
                        },

                        AppUiState::MultiplayerModal => match key {
                            KeyCode::ArrowUp | KeyCode::KeyW => {
                                self.multiplayer_selected_action =
                                    self.multiplayer_selected_action.saturating_sub(1);
                                return;
                            }
                            KeyCode::ArrowDown | KeyCode::KeyS => {
                                self.multiplayer_selected_action =
                                    (self.multiplayer_selected_action + 1).min(3);
                                return;
                            }
                            KeyCode::Escape => {
                                self.ui_state = AppUiState::InGame;
                                return;
                            }
                            KeyCode::Enter | KeyCode::Space => {
                                match self.multiplayer_selected_action {
                                    0 => {
                                        self.host_and_join_local_server(7777);
                                    }
                                    1 => {
                                        let addr = self.multiplayer_server_addr_input.clone();
                                        self.join_multiplayer_server(&addr);
                                    }
                                    2 => {
                                        self.disconnect_multiplayer();
                                    }
                                    3 => {
                                        self.ui_state = AppUiState::InGame;
                                    }
                                    _ => {}
                                }
                                return;
                            }
                            _ => {}
                        },

                        AppUiState::InGame => match key {
                            KeyCode::Escape => {
                                self.pressed_keys.clear();
                                self.smoothed_steer_input = 0.0;
                                self.ui_state = AppUiState::PauseMenu;
                                return;
                            }

                            KeyCode::KeyN | KeyCode::F4 => {
                                self.pressed_keys.clear();
                                self.smoothed_steer_input = 0.0;
                                self.ui_state = AppUiState::MultiplayerModal;
                                return;
                            }

                            KeyCode::F3 => {
                                self.show_debug_overlay = !self.show_debug_overlay;
                                let label = format!(
                                    "Developer Debug Overlay: {}",
                                    if self.show_debug_overlay { "ON" } else { "OFF" }
                                );
                                println!("🔧 {}", label);
                                self.status_notification = Some((label, Instant::now()));
                                return;
                            }

                            KeyCode::KeyJ | KeyCode::Tab => {
                                self.pressed_keys.clear();
                                self.smoothed_steer_input = 0.0;
                                self.ui_state = AppUiState::JobBrowser;
                                return;
                            }

                            KeyCode::KeyG => {
                                self.pressed_keys.clear();
                                self.smoothed_steer_input = 0.0;
                                self.ui_state = AppUiState::GarageSelect;
                                return;
                            }

                            KeyCode::KeyO => {
                                self.pressed_keys.clear();
                                self.smoothed_steer_input = 0.0;
                                self.ui_state = AppUiState::SettingsMenu;
                                return;
                            }

                            KeyCode::KeyU | KeyCode::F2 => {
                                self.pressed_keys.clear();
                                self.smoothed_steer_input = 0.0;
                                self.ui_state = AppUiState::CatalogModal;
                                return;
                            }

                            KeyCode::KeyH | KeyCode::F1 => {
                                self.show_hud = !self.show_hud;
                                let label = format!(
                                    "HUD Display: {}",
                                    if self.show_hud { "ON" } else { "OFF" }
                                );
                                println!("📊 {}", label);
                                self.status_notification = Some((label, Instant::now()));
                                return;
                            }

                            KeyCode::F5 => {
                                self.perform_quicksave();
                                return;
                            }

                            KeyCode::F9 => {
                                self.perform_quickload();
                                return;
                            }

                            KeyCode::F12 => {
                                self.capture_screenshot();
                                return;
                            }

                            KeyCode::KeyR => {
                                if self.config.is_drive {
                                    self.perform_roadside_recovery();
                                } else {
                                    self.region_cycle_idx =
                                        (self.region_cycle_idx + 1) % self.region_cycle.len();
                                    let next_reg = self.region_cycle[self.region_cycle_idx];
                                    self.config.region_id = next_reg.to_string();
                                    let _ = self.load_region(next_reg);
                                }
                                return;
                            }

                            KeyCode::Home => {
                                self.camera_ctrl.reset_to_default();
                                let label = "Camera Reset to Anchor Landmark".to_string();
                                println!("🎯 {}", label);
                                self.status_notification = Some((label, Instant::now()));
                                return;
                            }

                            KeyCode::BracketRight | KeyCode::Equal | KeyCode::PageUp => {
                                let spd = self.camera_ctrl.increase_speed();
                                let label = format!(
                                    "Camera Speed: {:.1} m/s ({:.1}x)",
                                    spd, self.camera_ctrl.speed_multiplier
                                );
                                println!("⏩ {}", label);
                                self.status_notification = Some((label, Instant::now()));
                                return;
                            }

                            KeyCode::BracketLeft | KeyCode::Minus | KeyCode::PageDown => {
                                let spd = self.camera_ctrl.decrease_speed();
                                let label = format!(
                                    "Camera Speed: {:.1} m/s ({:.1}x)",
                                    spd, self.camera_ctrl.speed_multiplier
                                );
                                println!("⏪ {}", label);
                                self.status_notification = Some((label, Instant::now()));
                                return;
                            }

                            KeyCode::KeyV => {
                                self.config.is_drive = !self.config.is_drive;
                                let label = if self.config.is_drive {
                                    "Simulation Mode: Complete Drivable Vehicle Dynamics"
                                } else {
                                    "Simulation Mode: Free Camera Exploration"
                                };
                                println!("🏎️ {}", label);
                                self.status_notification =
                                    Some((label.to_string(), Instant::now()));
                                return;
                            }

                            KeyCode::KeyM => {
                                self.active_transport_mode = match self.active_transport_mode {
                                    ActiveTransportMode::Car => ActiveTransportMode::Motorcycle,
                                    ActiveTransportMode::Motorcycle => {
                                        ActiveTransportMode::TransitBus
                                    }
                                    ActiveTransportMode::TransitBus => {
                                        ActiveTransportMode::HeavyTruck
                                    }
                                    ActiveTransportMode::HeavyTruck => ActiveTransportMode::Train,
                                    ActiveTransportMode::Train => ActiveTransportMode::Marine,
                                    ActiveTransportMode::Marine => ActiveTransportMode::Aviation,
                                    ActiveTransportMode::Aviation => ActiveTransportMode::Car,
                                };
                                self.config.is_drive = true;
                                let label = format!(
                                    "{} Transport Mode: {}",
                                    self.active_transport_mode.icon(),
                                    self.active_transport_mode.display_name()
                                );
                                println!("{}", label);
                                self.status_notification = Some((label, Instant::now()));
                                return;
                            }

                            KeyCode::KeyC => {
                                if self.config.is_drive {
                                    self.vehicle_cam.cycle_perspective();
                                    let label = format!(
                                        "Vehicle Camera: {:?}",
                                        self.vehicle_cam.perspective
                                    );
                                    println!("📷 {}", label);
                                    self.status_notification = Some((label, Instant::now()));
                                }
                                return;
                            }

                            KeyCode::KeyP => {
                                if self.config.is_drive {
                                    self.park_engaged = !self.park_engaged;
                                    let label = if self.park_engaged {
                                        "🅿️ Transmission: [P] PARK (Parking Brake Locked)"
                                    } else {
                                        "🏎️ Transmission: [D] DRIVE"
                                    };
                                    println!("{}", label);
                                    self.status_notification =
                                        Some((label.to_string(), Instant::now()));
                                    return;
                                }
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
                                return;
                            }

                            KeyCode::Digit1 | KeyCode::Numpad1 => {
                                if self.config.is_drive {
                                    self.active_transport_mode = ActiveTransportMode::Car;
                                    self.vehicle_visual_type = VehicleVisualType::SportsCoupe;
                                    self.vehicle.config = VehicleConfig::sports_coupe();
                                    let label =
                                        "🏎️ Mode: Automobile (Sports Coupe / Ackermann-Pacejka)"
                                            .to_string();
                                    println!("{}", label);
                                    self.status_notification = Some((label, Instant::now()));
                                } else {
                                    self.camera_ctrl
                                        .jump_to(&Landmark::SAN_FRANCISCO_TRANSAMERICA);
                                    self.status_notification = Some((
                                        format!(
                                            "📍 Jump to {}",
                                            Landmark::SAN_FRANCISCO_TRANSAMERICA.name
                                        ),
                                        Instant::now(),
                                    ));
                                }
                                return;
                            }
                            KeyCode::Digit2 | KeyCode::Numpad2 => {
                                if self.config.is_drive {
                                    self.active_transport_mode = ActiveTransportMode::Motorcycle;
                                    let label =
                                        "🏍️ Mode: Motorcycle (2-Wheel Lean Dynamics)".to_string();
                                    println!("{}", label);
                                    self.status_notification = Some((label, Instant::now()));
                                } else {
                                    self.camera_ctrl
                                        .jump_to(&Landmark::SAN_FRANCISCO_SALESFORCE);
                                    self.status_notification = Some((
                                        format!(
                                            "📍 Jump to {}",
                                            Landmark::SAN_FRANCISCO_SALESFORCE.name
                                        ),
                                        Instant::now(),
                                    ));
                                }
                                return;
                            }
                            KeyCode::Digit3 | KeyCode::Numpad3 => {
                                if self.config.is_drive {
                                    self.active_transport_mode = ActiveTransportMode::TransitBus;
                                    let label =
                                        "🚌 Mode: Transit Bus (Pneumatic Air Brakes)".to_string();
                                    println!("{}", label);
                                    self.status_notification = Some((label, Instant::now()));
                                } else {
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
                                return;
                            }
                            KeyCode::Digit4 | KeyCode::Numpad4 => {
                                if self.config.is_drive {
                                    self.active_transport_mode = ActiveTransportMode::HeavyTruck;
                                    let label =
                                        "🚛 Mode: Heavy Truck (Trailer Articulation)".to_string();
                                    println!("{}", label);
                                    self.status_notification = Some((label, Instant::now()));
                                } else {
                                    self.camera_ctrl
                                        .jump_to(&Landmark::SAN_FRANCISCO_COIT_TOWER);
                                    self.status_notification = Some((
                                        format!(
                                            "📍 Jump to {}",
                                            Landmark::SAN_FRANCISCO_COIT_TOWER.name
                                        ),
                                        Instant::now(),
                                    ));
                                }
                                return;
                            }
                            KeyCode::Digit5 | KeyCode::Numpad5 => {
                                if self.config.is_drive {
                                    self.active_transport_mode = ActiveTransportMode::Train;
                                    let label = "🚆 Mode: Train Consist (Davis Drag & Air Brakes)"
                                        .to_string();
                                    println!("{}", label);
                                    self.status_notification = Some((label, Instant::now()));
                                } else {
                                    self.camera_ctrl
                                        .jump_to(&Landmark::SAN_FRANCISCO_BAY_BRIDGE);
                                    self.status_notification = Some((
                                        format!(
                                            "📍 Jump to {}",
                                            Landmark::SAN_FRANCISCO_BAY_BRIDGE.name
                                        ),
                                        Instant::now(),
                                    ));
                                }
                                return;
                            }
                            KeyCode::Digit6 | KeyCode::Numpad6 => {
                                if self.config.is_drive {
                                    self.active_transport_mode = ActiveTransportMode::Marine;
                                    let label =
                                        "🚢 Mode: Marine Vessel (Archimedes Buoyancy)".to_string();
                                    println!("{}", label);
                                    self.status_notification = Some((label, Instant::now()));
                                } else {
                                    self.camera_ctrl
                                        .jump_to(&Landmark::BENGALURU_VIDHANA_SOUDHA);
                                    self.status_notification = Some((
                                        format!(
                                            "📍 Jump to {}",
                                            Landmark::BENGALURU_VIDHANA_SOUDHA.name
                                        ),
                                        Instant::now(),
                                    ));
                                }
                                return;
                            }
                            KeyCode::Digit7 | KeyCode::Numpad7 => {
                                if self.config.is_drive {
                                    self.active_transport_mode = ActiveTransportMode::Aviation;
                                    let label = "✈️ Mode: Aviation Aircraft (6-DOF Flight Polar)"
                                        .to_string();
                                    println!("{}", label);
                                    self.status_notification = Some((label, Instant::now()));
                                } else {
                                    self.camera_ctrl.jump_to(&Landmark::MYSURU_PALACE);
                                    self.status_notification = Some((
                                        format!("📍 Jump to {}", Landmark::MYSURU_PALACE.name),
                                        Instant::now(),
                                    ));
                                }
                                return;
                            }
                            KeyCode::Digit8 => {
                                self.camera_ctrl.jump_to(&Landmark::CHENNAI_CENTRAL);
                                self.status_notification = Some((
                                    format!("📍 Jump to {}", Landmark::CHENNAI_CENTRAL.name),
                                    Instant::now(),
                                ));
                                return;
                            }
                            KeyCode::Digit9 => {
                                self.camera_ctrl.jump_to(&Landmark::HYDERABAD_CHARMINAR);
                                self.status_notification = Some((
                                    format!("📍 Jump to {}", Landmark::HYDERABAD_CHARMINAR.name),
                                    Instant::now(),
                                ));
                                return;
                            }
                            _ => {}
                        },
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

                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;

                if let Some(ref mut client) = self.multiplayer_client {
                    let _ = client.poll_network(now_ms);
                }

                // 3. Fixed-timestep simulation & Camera / Vehicle updates
                let anchor = res.package_anchor;
                let meters_per_deg_lat = 111_320.0;
                let meters_per_deg_lon = 111_320.0 * anchor.lat.to_radians().cos().abs().max(0.01);

                sim_world.tick(dt);
                let solar = sim_world.current_solar_position();

                let is_gameplay = self.ui_state == AppUiState::InGame;
                let is_braking = if is_gameplay {
                    self.pressed_keys.contains(&KeyCode::KeyS)
                        || self.pressed_keys.contains(&KeyCode::ArrowDown)
                } else {
                    true
                };

                const PHYSICS_DT: f32 = 1.0 / 60.0;
                self.physics_accumulator = (self.physics_accumulator + dt as f32).min(0.20);

                let mut last_road_inputs = VehicleInputs::default();

                if self.config.is_drive {
                    let is_forward_key = is_gameplay
                        && (self.pressed_keys.contains(&KeyCode::KeyW)
                            || self.pressed_keys.contains(&KeyCode::ArrowUp));
                    let is_reverse_or_brake_key = is_gameplay
                        && (self.pressed_keys.contains(&KeyCode::KeyS)
                            || self.pressed_keys.contains(&KeyCode::ArrowDown));
                    let steer_left = is_gameplay
                        && (self.pressed_keys.contains(&KeyCode::KeyA)
                            || self.pressed_keys.contains(&KeyCode::ArrowLeft));
                    let steer_right = is_gameplay
                        && (self.pressed_keys.contains(&KeyCode::KeyD)
                            || self.pressed_keys.contains(&KeyCode::ArrowRight));
                    let is_handbrake = is_gameplay && self.pressed_keys.contains(&KeyCode::Space);

                    while self.physics_accumulator >= PHYSICS_DT {
                        self.previous_vehicle_pos = self.vehicle.position;
                        self.previous_vehicle_orient = self.vehicle.orientation;

                        let speed_kph = match self.active_transport_mode {
                            ActiveTransportMode::Car => self.vehicle.speed_kph.abs(),
                            ActiveTransportMode::Motorcycle => self.motorcycle.speed_kph.abs(),
                            ActiveTransportMode::TransitBus => self.transit_bus.speed_kph.abs(),
                            ActiveTransportMode::HeavyTruck => self.heavy_truck.speed_kph.abs(),
                            ActiveTransportMode::Train => self.train_consist.speed_kph.abs(),
                            ActiveTransportMode::Marine => self.marine_vessel.speed_knots * 1.852,
                            ActiveTransportMode::Aviation => self.aircraft.true_airspeed_mps * 3.6,
                        };
                        let speed_factor = (1.0 - (speed_kph / 120.0).min(0.60)).clamp(0.40, 1.0);

                        let target_steer = if !is_gameplay {
                            0.0
                        } else if steer_left {
                            -0.65 * speed_factor
                        } else if steer_right {
                            0.65 * speed_factor
                        } else {
                            0.0
                        };

                        let steer_rate = if target_steer.abs() < self.smoothed_steer_input.abs()
                            || target_steer == 0.0
                        {
                            5.0
                        } else {
                            3.5
                        };
                        let steer_delta = target_steer - self.smoothed_steer_input;
                        let max_step = steer_rate * PHYSICS_DT;
                        if steer_delta.abs() <= max_step {
                            self.smoothed_steer_input = target_steer;
                        } else {
                            self.smoothed_steer_input += steer_delta.signum() * max_step;
                        }

                        let mut road_inputs = VehicleInputs {
                            steer: self.smoothed_steer_input,
                            ..Default::default()
                        };

                        if self.park_engaged || !is_gameplay {
                            road_inputs.gear_mode = TransmissionMode::Park;
                            road_inputs.brake = 1.0;
                            road_inputs.handbrake = true;
                            road_inputs.throttle = 0.0;
                        } else {
                            let rot = glam::Mat3::from_quat(self.vehicle.orientation);
                            let forward = rot.col(2);
                            let forward_speed = self.vehicle.linear_velocity.dot(forward);

                            if is_forward_key {
                                if forward_speed < -0.3 {
                                    road_inputs.brake = 0.95;
                                    road_inputs.gear_mode = TransmissionMode::Reverse;
                                } else {
                                    road_inputs.throttle = 1.0;
                                    road_inputs.gear_mode = TransmissionMode::Drive;
                                }
                            } else if is_reverse_or_brake_key {
                                if forward_speed > 0.35 {
                                    road_inputs.brake = 0.95;
                                    road_inputs.gear_mode = TransmissionMode::Drive;
                                } else {
                                    road_inputs.throttle = 0.85;
                                    road_inputs.gear_mode = TransmissionMode::Reverse;
                                }
                            } else {
                                road_inputs.throttle = 0.0;
                                road_inputs.brake = 0.0;
                                road_inputs.gear_mode = TransmissionMode::Drive;
                            }

                            if is_handbrake {
                                road_inputs.handbrake = true;
                            }
                        }

                        // Step physics for active multimodal mode
                        match self.active_transport_mode {
                            ActiveTransportMode::Car => {
                                self.vehicle.step(&road_inputs, gq, PHYSICS_DT);

                                let ground_y =
                                    gq.elevation(self.vehicle.position.x, self.vehicle.position.z);
                                let min_allowed_y = ground_y + self.vehicle.config.wheel_radius;
                                if self.vehicle.position.y < min_allowed_y {
                                    self.vehicle.position.y = min_allowed_y;
                                    if self.vehicle.linear_velocity.y < 0.0 {
                                        self.vehicle.linear_velocity.y = 0.0;
                                    }
                                }

                                let rot = glam::Mat3::from_quat(self.vehicle.orientation);
                                let forward = rot.col(2);
                                let forward_speed = self.vehicle.linear_velocity.dot(forward);
                                let wheel_r = self.vehicle.config.wheel_radius.max(0.1);
                                self.wheel_spin_angle = (self.wheel_spin_angle
                                    + (forward_speed * PHYSICS_DT) / wheel_r)
                                    % (2.0 * std::f32::consts::PI);
                            }
                            ActiveTransportMode::Motorcycle => {
                                self.motorcycle.step(&road_inputs, gq, PHYSICS_DT);
                                let forward_speed = self.motorcycle.speed_kph / 3.6;
                                let wheel_r = 0.31f32;
                                self.wheel_spin_angle = (self.wheel_spin_angle
                                    + (forward_speed * PHYSICS_DT) / wheel_r)
                                    % (2.0 * std::f32::consts::PI);
                            }
                            ActiveTransportMode::TransitBus => {
                                self.transit_bus.step(&road_inputs, gq, PHYSICS_DT);
                                let forward_speed = self.transit_bus.speed_kph / 3.6;
                                let wheel_r = 0.52f32;
                                self.wheel_spin_angle = (self.wheel_spin_angle
                                    + (forward_speed * PHYSICS_DT) / wheel_r)
                                    % (2.0 * std::f32::consts::PI);
                            }
                            ActiveTransportMode::HeavyTruck => {
                                self.heavy_truck.step(&road_inputs, gq, PHYSICS_DT);
                                let forward_speed = self.heavy_truck.speed_kph / 3.6;
                                let wheel_r = 0.52f32;
                                self.wheel_spin_angle = (self.wheel_spin_angle
                                    + (forward_speed * PHYSICS_DT) / wheel_r)
                                    % (2.0 * std::f32::consts::PI);
                            }
                            ActiveTransportMode::Train => {
                                if is_forward_key {
                                    self.train_consist.throttle =
                                        (self.train_consist.throttle + 0.35 * PHYSICS_DT).min(1.0);
                                    self.train_consist.apply_service_brake(0.0);
                                } else if is_reverse_or_brake_key {
                                    if self.train_consist.velocity_mps.abs() < 0.15
                                        && self.train_consist.throttle < 0.05
                                    {
                                        self.train_consist.reverser = -self.train_consist.reverser;
                                    } else {
                                        self.train_consist.apply_service_brake(0.85);
                                        self.train_consist.throttle = (self.train_consist.throttle
                                            - 0.6 * PHYSICS_DT)
                                            .max(0.0);
                                    }
                                } else {
                                    self.train_consist.apply_service_brake(0.0);
                                }

                                if is_handbrake {
                                    self.train_consist.trigger_emergency_brake();
                                }

                                self.train_consist.step(PHYSICS_DT);

                                let forward =
                                    glam::Quat::from_rotation_y(self.train_yaw) * glam::Vec3::Z;
                                self.train_position +=
                                    forward * (self.train_consist.velocity_mps * PHYSICS_DT);
                                let ground_y =
                                    gq.elevation(self.train_position.x, self.train_position.z);
                                self.train_position.y = ground_y + 0.6;
                            }
                            ActiveTransportMode::Marine => {
                                let mut marine_inputs = MarineInputs::default();
                                if is_forward_key {
                                    marine_inputs.throttle = 1.0;
                                } else if is_reverse_or_brake_key {
                                    marine_inputs.throttle = -0.6;
                                }
                                if steer_left {
                                    marine_inputs.rudder = -1.0;
                                } else if steer_right {
                                    marine_inputs.rudder = 1.0;
                                }

                                self.marine_elapsed_time += PHYSICS_DT;
                                self.marine_vessel.step(
                                    &marine_inputs,
                                    self.marine_elapsed_time,
                                    PHYSICS_DT,
                                );
                            }
                            ActiveTransportMode::Aviation => {
                                let mut flight_inputs = FlightInputs {
                                    gear_down: self.aircraft.altitude_msl_meters < 50.0
                                        || !self.aircraft.is_airborne,
                                    flaps: 0.25,
                                    ..Default::default()
                                };

                                if is_forward_key {
                                    flight_inputs.elevator = -0.7; // Pitch down
                                } else if is_reverse_or_brake_key {
                                    flight_inputs.elevator = 0.7; // Pitch up
                                }

                                if steer_left {
                                    flight_inputs.aileron = -0.7; // Roll left
                                } else if steer_right {
                                    flight_inputs.aileron = 0.7; // Roll right
                                }

                                if self.pressed_keys.contains(&KeyCode::KeyQ) {
                                    flight_inputs.rudder = -0.5;
                                } else if self.pressed_keys.contains(&KeyCode::KeyE) {
                                    flight_inputs.rudder = 0.5;
                                }

                                if self.pressed_keys.contains(&KeyCode::ShiftLeft)
                                    || self.pressed_keys.contains(&KeyCode::ShiftRight)
                                {
                                    self.aircraft_throttle =
                                        (self.aircraft_throttle + 0.3 * PHYSICS_DT).min(1.0);
                                }
                                if self.pressed_keys.contains(&KeyCode::ControlLeft)
                                    || self.pressed_keys.contains(&KeyCode::ControlRight)
                                {
                                    self.aircraft_throttle =
                                        (self.aircraft_throttle - 0.3 * PHYSICS_DT).max(0.0);
                                }

                                if is_handbrake {
                                    flight_inputs.wheel_brakes = 1.0;
                                }

                                flight_inputs.throttle = self.aircraft_throttle;
                                let ground_y = gq
                                    .elevation(self.aircraft.position.x, self.aircraft.position.z);
                                self.aircraft.step(&flight_inputs, ground_y, PHYSICS_DT);
                            }
                        }

                        self.traffic_sim.step(PHYSICS_DT, gq);
                        last_road_inputs = road_inputs;
                        self.physics_accumulator -= PHYSICS_DT;
                    }
                }

                let alpha = (self.physics_accumulator / PHYSICS_DT).clamp(0.0, 1.0);
                let render_pos = self.previous_vehicle_pos.lerp(self.vehicle.position, alpha);
                let render_orient = self
                    .previous_vehicle_orient
                    .slerp(self.vehicle.orientation, alpha);

                let velocity = if self.config.is_drive {
                    // Authoritative Multiplayer client input dispatch & prediction sync
                    if let Some(ref mut client) = self.multiplayer_client {
                        if client.state == realearth_networking::ConnectionState::Connected {
                            let input_frame = realearth_networking::ClientInputFrame {
                                tick: client.server_tick + 1,
                                delta_time: dt as f32,
                                throttle: last_road_inputs.throttle,
                                steer: last_road_inputs.steer,
                                brake: last_road_inputs.brake,
                                handbrake: last_road_inputs.handbrake,
                                client_timestamp_ms: now_ms,
                            };
                            let _ = client.send_input(&input_frame);

                            if client.controlled_entity_id.is_some() {
                                self.vehicle.position = client.predictor.position;
                                self.vehicle.orientation = client.predictor.orientation;
                                self.vehicle.linear_velocity = client.predictor.linear_velocity;
                            }
                        }
                    }

                    let (active_pos, active_orient, active_speed_kph) =
                        match self.active_transport_mode {
                            ActiveTransportMode::Car => {
                                (render_pos, render_orient, self.vehicle.speed_kph)
                            }
                            ActiveTransportMode::Motorcycle => (
                                self.motorcycle.position,
                                self.motorcycle.orientation,
                                self.motorcycle.speed_kph,
                            ),
                            ActiveTransportMode::TransitBus => (
                                self.transit_bus.position,
                                self.transit_bus.orientation,
                                self.transit_bus.speed_kph,
                            ),
                            ActiveTransportMode::HeavyTruck => (
                                self.heavy_truck.tractor_position,
                                self.heavy_truck.tractor_orientation,
                                self.heavy_truck.speed_kph,
                            ),
                            ActiveTransportMode::Train => (
                                self.train_position,
                                glam::Quat::from_rotation_y(self.train_yaw),
                                self.train_consist.speed_kph,
                            ),
                            ActiveTransportMode::Marine => (
                                self.marine_vessel.position,
                                self.marine_vessel.orientation,
                                self.marine_vessel.speed_knots * 1.852,
                            ),
                            ActiveTransportMode::Aviation => (
                                self.aircraft.position,
                                self.aircraft.orientation,
                                self.aircraft.true_airspeed_mps * 3.6,
                            ),
                        };

                    let veh_coord = Wgs84Coord::new(
                        anchor.lat + (active_pos.z as f64 / meters_per_deg_lat),
                        anchor.lon + (active_pos.x as f64 / meters_per_deg_lon),
                        active_pos.y as f64,
                    );

                    let rot_mat = glam::Mat3::from_quat(active_orient);
                    let forward_vec = rot_mat.col(2);
                    let heading_deg = (forward_vec.x.atan2(forward_vec.z) as f64)
                        .to_degrees()
                        .rem_euclid(360.0);

                    if let Some(ref mut tracker) = self.route_tracker {
                        self.active_nav_update = Some(tracker.update(veh_coord, heading_deg));
                    }

                    if let Some(ref mut job) = self.active_job {
                        if let Some(completion) =
                            job.update(dt as f32, veh_coord, active_speed_kph, &self.gameplay_bus)
                        {
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

                            self.active_debrief = Some(ActiveDebrief {
                                title: job.title.clone(),
                                career: job.career.display_name().to_string(),
                                base_payout: completion.base_payout,
                                tip_bonus: completion.tip_bonus,
                                speed_bonus: completion.speed_bonus,
                                net_payout: completion.final_payout,
                                xp_reward: completion.xp_gained,
                                comfort_stars: completion.rating_stars,
                            });
                            self.ui_state = AppUiState::DebriefModal;

                            let msg = format!(
                                "🎉 CONTRACT COMPLETE! +${:.2} | Rating: {:.1} Stars | Wallet: ${:.2}",
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
                            } else {
                                self.active_job = None;
                            }
                        }
                    }
                    let speed_mps = active_speed_kph as f64 * (1000.0 / 3600.0);
                    self.player_profile
                        .record_distance((speed_mps * dt) / 1000.0);

                    // Step audio synthesis telemetry
                    let (slip_ratio, slip_angle) = if self.active_transport_mode
                        == ActiveTransportMode::Car
                    {
                        let w0 = &self.vehicle.wheels[0];
                        let forward_speed = (self.vehicle.speed_kph / 3.6).abs();
                        let wheel_speed =
                            (w0.angular_velocity * self.vehicle.config.wheel_radius).abs();
                        let ratio = if forward_speed > 0.5 {
                            ((wheel_speed - forward_speed).abs() / forward_speed).clamp(0.0, 1.0)
                        } else {
                            0.0
                        };
                        (ratio, w0.slip_angle.to_degrees().abs())
                    } else {
                        (0.0, 0.0)
                    };
                    let telemetry = realearth_gameplay::audio::AudioTelemetry {
                        engine_rpm: match self.active_transport_mode {
                            ActiveTransportMode::Car => self.vehicle.engine_rpm,
                            ActiveTransportMode::Motorcycle => {
                                (self.motorcycle.speed_kph * 95.0 + 1200.0).min(11500.0)
                            }
                            _ => 2200.0,
                        },
                        throttle: if is_braking { 0.0 } else { 0.8 },
                        speed_kph: active_speed_kph,
                        brake_input: if is_braking || self.park_engaged {
                            1.0
                        } else {
                            0.0
                        },
                        tire_slip_ratio: slip_ratio,
                        tire_slip_angle_deg: slip_angle,
                        collision_impulse: 0.0,
                        horn_pressed: self.pressed_keys.contains(&KeyCode::KeyH),
                    };
                    let _ = self.audio_engine.update_telemetry(telemetry, dt as f32);

                    // Update camera based on active transport mode with ground elevation clamping and speed FOV
                    match self.active_transport_mode {
                        ActiveTransportMode::Car => {
                            let g_elev = gq.elevation(render_pos.x, render_pos.z);
                            let (cam_pos, cam_yaw, cam_pitch) =
                                self.vehicle_cam.update_pose_with_ground(
                                    render_pos,
                                    render_orient,
                                    5.5,
                                    2.2,
                                    self.vehicle.speed_kph,
                                    dt as f32,
                                    Some(g_elev),
                                );
                            let cam_lat = anchor.lat + (cam_pos.z as f64 / meters_per_deg_lat);
                            let cam_lon = anchor.lon + (cam_pos.x as f64 / meters_per_deg_lon);
                            self.camera_ctrl.coord =
                                Wgs84Coord::new(cam_lat, cam_lon, cam_pos.y as f64);
                            self.camera_ctrl.yaw = cam_yaw;
                            self.camera_ctrl.pitch = cam_pitch;
                        }
                        ActiveTransportMode::Motorcycle => {
                            let g_elev = gq
                                .elevation(self.motorcycle.position.x, self.motorcycle.position.z);
                            let (cam_pos, cam_yaw, cam_pitch) =
                                self.vehicle_cam.update_pose_with_ground(
                                    self.motorcycle.position,
                                    self.motorcycle.orientation,
                                    4.2,
                                    1.6,
                                    self.motorcycle.speed_kph,
                                    dt as f32,
                                    Some(g_elev),
                                );
                            let cam_lat = anchor.lat + (cam_pos.z as f64 / meters_per_deg_lat);
                            let cam_lon = anchor.lon + (cam_pos.x as f64 / meters_per_deg_lon);
                            self.camera_ctrl.coord =
                                Wgs84Coord::new(cam_lat, cam_lon, cam_pos.y as f64);
                            self.camera_ctrl.yaw = cam_yaw;
                            self.camera_ctrl.pitch = cam_pitch;
                        }
                        ActiveTransportMode::TransitBus => {
                            let g_elev = gq.elevation(
                                self.transit_bus.position.x,
                                self.transit_bus.position.z,
                            );
                            let (cam_pos, cam_yaw, cam_pitch) =
                                self.vehicle_cam.update_pose_with_ground(
                                    self.transit_bus.position,
                                    self.transit_bus.orientation,
                                    16.0,
                                    4.5,
                                    self.transit_bus.speed_kph,
                                    dt as f32,
                                    Some(g_elev),
                                );
                            let cam_lat = anchor.lat + (cam_pos.z as f64 / meters_per_deg_lat);
                            let cam_lon = anchor.lon + (cam_pos.x as f64 / meters_per_deg_lon);
                            self.camera_ctrl.coord =
                                Wgs84Coord::new(cam_lat, cam_lon, cam_pos.y as f64);
                            self.camera_ctrl.yaw = cam_yaw;
                            self.camera_ctrl.pitch = cam_pitch;
                        }
                        ActiveTransportMode::HeavyTruck => {
                            let g_elev = gq.elevation(
                                self.heavy_truck.tractor_position.x,
                                self.heavy_truck.tractor_position.z,
                            );
                            let (cam_pos, cam_yaw, cam_pitch) =
                                self.vehicle_cam.update_pose_with_ground(
                                    self.heavy_truck.tractor_position,
                                    self.heavy_truck.tractor_orientation,
                                    18.5,
                                    4.8,
                                    self.heavy_truck.speed_kph,
                                    dt as f32,
                                    Some(g_elev),
                                );
                            let cam_lat = anchor.lat + (cam_pos.z as f64 / meters_per_deg_lat);
                            let cam_lon = anchor.lon + (cam_pos.x as f64 / meters_per_deg_lon);
                            self.camera_ctrl.coord =
                                Wgs84Coord::new(cam_lat, cam_lon, cam_pos.y as f64);
                            self.camera_ctrl.yaw = cam_yaw;
                            self.camera_ctrl.pitch = cam_pitch;
                        }
                        ActiveTransportMode::Train => {
                            let g_elev = gq.elevation(self.train_position.x, self.train_position.z);
                            let (cam_pos, cam_yaw, cam_pitch) =
                                self.vehicle_cam.update_pose_with_ground(
                                    self.train_position,
                                    glam::Quat::from_rotation_y(self.train_yaw),
                                    28.0,
                                    6.2,
                                    self.train_consist.speed_kph,
                                    dt as f32,
                                    Some(g_elev),
                                );
                            let cam_lat = anchor.lat + (cam_pos.z as f64 / meters_per_deg_lat);
                            let cam_lon = anchor.lon + (cam_pos.x as f64 / meters_per_deg_lon);
                            self.camera_ctrl.coord =
                                Wgs84Coord::new(cam_lat, cam_lon, cam_pos.y as f64);
                            self.camera_ctrl.yaw = cam_yaw;
                            self.camera_ctrl.pitch = cam_pitch;
                        }
                        ActiveTransportMode::Marine => {
                            let (cam_pos, cam_yaw, cam_pitch) =
                                self.vehicle_cam.update_pose_with_ground(
                                    self.marine_vessel.position,
                                    self.marine_vessel.orientation,
                                    90.0,
                                    24.0,
                                    self.marine_vessel.speed_knots * 1.852,
                                    dt as f32,
                                    Some(0.0),
                                );
                            let cam_lat = anchor.lat + (cam_pos.z as f64 / meters_per_deg_lat);
                            let cam_lon = anchor.lon + (cam_pos.x as f64 / meters_per_deg_lon);
                            self.camera_ctrl.coord =
                                Wgs84Coord::new(cam_lat, cam_lon, cam_pos.y as f64);
                            self.camera_ctrl.yaw = cam_yaw;
                            self.camera_ctrl.pitch = cam_pitch;
                        }
                        ActiveTransportMode::Aviation => {
                            let g_elev =
                                gq.elevation(self.aircraft.position.x, self.aircraft.position.z);
                            let (cam_pos, cam_yaw, cam_pitch) =
                                self.vehicle_cam.update_pose_with_ground(
                                    self.aircraft.position,
                                    self.aircraft.orientation,
                                    48.0,
                                    12.5,
                                    self.aircraft.true_airspeed_mps * 3.6,
                                    dt as f32,
                                    Some(g_elev),
                                );
                            let cam_lat = anchor.lat + (cam_pos.z as f64 / meters_per_deg_lat);
                            let cam_lon = anchor.lon + (cam_pos.x as f64 / meters_per_deg_lon);
                            self.camera_ctrl.coord =
                                Wgs84Coord::new(cam_lat, cam_lon, cam_pos.y as f64);
                            self.camera_ctrl.yaw = cam_yaw;
                            self.camera_ctrl.pitch = cam_pitch;
                        }
                    }
                    self.camera.fov_y = self.vehicle_cam.current_fov_deg.to_radians();
                    self.camera_ctrl.update_camera(&mut self.camera);

                    forward_vec * (active_speed_kph * (1000.0 / 3600.0))
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

                    if is_gameplay {
                        if self.pressed_keys.contains(&KeyCode::KeyW) {
                            self.camera_ctrl
                                .move_forward(effective_move, Some(&res.dem));
                        }
                        if self.pressed_keys.contains(&KeyCode::KeyS) {
                            self.camera_ctrl
                                .move_forward(-effective_move, Some(&res.dem));
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

                // 7. Render 3D Vehicles (Player Multimodal Fleet and AI Traffic)
                let mut fleet_mesh = realearth_compiler::mesh::ChunkMesh::default();
                if self.config.is_drive {
                    match self.active_transport_mode {
                        ActiveTransportMode::Car => {
                            let rot_mat = glam::Mat3::from_quat(self.vehicle.orientation);
                            let forward_vec = rot_mat.col(2);
                            let is_in_reverse = self.vehicle.current_gear == -1
                                || self.vehicle.linear_velocity.dot(forward_vec) < -0.05;

                            let player_mesh = generate_vehicle_mesh(&VehicleMeshParams {
                                position: render_pos,
                                orientation: render_orient,
                                color: self.vehicle_custom_color.unwrap_or(
                                    match self.vehicle_visual_type {
                                        VehicleVisualType::SportsCoupe => [0.92, 0.12, 0.12, 1.0], // Cherry Red
                                        VehicleVisualType::Sedan => [0.15, 0.35, 0.85, 1.0], // Electric Blue
                                        VehicleVisualType::Suv => [0.20, 0.22, 0.25, 1.0], // Graphite Gray
                                        VehicleVisualType::Taxi => [0.95, 0.80, 0.10, 1.0], // Yellow Taxi
                                        VehicleVisualType::TransitBus => [0.10, 0.65, 0.40, 1.0], // City Green Transit
                                        VehicleVisualType::HeavyTruck => [0.85, 0.45, 0.10, 1.0], // Orange Hauler
                                        _ => [0.92, 0.12, 0.12, 1.0],
                                    },
                                ),
                                vehicle_type: self.vehicle_visual_type,
                                wheel_steer_angle: self.vehicle.wheels[0].steer_angle,
                                trailer_angle: 0.0,
                                headlights_on: true,
                                brake_lights_on: is_braking,
                                reverse_lights_on: is_in_reverse,
                                wheel_spin_angle: self.wheel_spin_angle,
                                wheel_compressions: [
                                    self.vehicle.wheels[0].compression,
                                    self.vehicle.wheels[1].compression,
                                    self.vehicle.wheels[2].compression,
                                    self.vehicle.wheels[3].compression,
                                ],
                                lod: VehicleLodLevel::Lod0High,
                            });
                            fleet_mesh.append(&player_mesh);
                        }
                        ActiveTransportMode::Motorcycle => {
                            let bike_mesh = generate_vehicle_mesh(&VehicleMeshParams {
                                position: self.motorcycle.position,
                                orientation: self.motorcycle.orientation,
                                color: [0.95, 0.42, 0.08, 1.0], // Racing Orange
                                vehicle_type: VehicleVisualType::Motorcycle,
                                wheel_steer_angle: self.motorcycle.steer_angle,
                                trailer_angle: 0.0,
                                headlights_on: true,
                                brake_lights_on: is_braking,
                                reverse_lights_on: false,
                                wheel_spin_angle: self.wheel_spin_angle,
                                wheel_compressions: [0.0; 4],
                                lod: VehicleLodLevel::Lod0High,
                            });
                            fleet_mesh.append(&bike_mesh);
                        }
                        ActiveTransportMode::TransitBus => {
                            let bus_mesh = generate_vehicle_mesh(&VehicleMeshParams {
                                position: self.transit_bus.position,
                                orientation: self.transit_bus.orientation,
                                color: [0.10, 0.65, 0.40, 1.0], // City Green Transit
                                vehicle_type: VehicleVisualType::TransitBus,
                                wheel_steer_angle: self.transit_bus.steer_angle,
                                trailer_angle: 0.0,
                                headlights_on: true,
                                brake_lights_on: is_braking,
                                reverse_lights_on: false,
                                wheel_spin_angle: self.wheel_spin_angle,
                                wheel_compressions: [0.0; 4],
                                lod: VehicleLodLevel::Lod0High,
                            });
                            fleet_mesh.append(&bus_mesh);
                        }
                        ActiveTransportMode::HeavyTruck => {
                            let truck_mesh = generate_vehicle_mesh(&VehicleMeshParams {
                                position: self.heavy_truck.tractor_position,
                                orientation: self.heavy_truck.tractor_orientation,
                                color: [0.85, 0.22, 0.15, 1.0], // Crimson Hauler
                                vehicle_type: VehicleVisualType::HeavyTruck,
                                wheel_steer_angle: self.heavy_truck.steer_angle,
                                trailer_angle: self.heavy_truck.trailer_articulation_angle,
                                headlights_on: true,
                                brake_lights_on: is_braking,
                                reverse_lights_on: false,
                                wheel_spin_angle: self.wheel_spin_angle,
                                wheel_compressions: [0.0; 4],
                                lod: VehicleLodLevel::Lod0High,
                            });
                            fleet_mesh.append(&truck_mesh);
                        }
                        ActiveTransportMode::Train => {
                            let loco_mesh = generate_vehicle_mesh(&VehicleMeshParams {
                                position: self.train_position,
                                orientation: glam::Quat::from_rotation_y(self.train_yaw),
                                color: [0.15, 0.35, 0.85, 1.0], // Electric Locomotive Blue
                                vehicle_type: VehicleVisualType::TrainLocomotive,
                                wheel_steer_angle: 0.0,
                                trailer_angle: 0.0,
                                headlights_on: true,
                                brake_lights_on: self.train_consist.brake_cylinder_pressure_bar
                                    > 0.5,
                                reverse_lights_on: self.train_consist.reverser == -1,
                                ..Default::default()
                            });
                            fleet_mesh.append(&loco_mesh);

                            // Append 2 passenger coach wagons behind locomotive
                            let back = -glam::Quat::from_rotation_y(self.train_yaw) * glam::Vec3::Z;
                            for i in 1..=2 {
                                let wagon_pos = self.train_position
                                    + back * (20.5 + (i as f32 - 1.0) * 24.0 + 12.0);
                                let wagon_mesh = generate_vehicle_mesh(&VehicleMeshParams {
                                    position: wagon_pos,
                                    orientation: glam::Quat::from_rotation_y(self.train_yaw),
                                    color: [0.75, 0.78, 0.82, 1.0], // Stainless Steel Coach
                                    vehicle_type: VehicleVisualType::TrainWagon,
                                    wheel_steer_angle: 0.0,
                                    trailer_angle: 0.0,
                                    headlights_on: false,
                                    brake_lights_on: self.train_consist.brake_cylinder_pressure_bar
                                        > 0.5,
                                    reverse_lights_on: false,
                                    ..Default::default()
                                });
                                fleet_mesh.append(&wagon_mesh);
                            }
                        }
                        ActiveTransportMode::Marine => {
                            let ship_mesh = generate_vehicle_mesh(&VehicleMeshParams {
                                position: self.marine_vessel.position,
                                orientation: self.marine_vessel.orientation,
                                color: [0.12, 0.22, 0.35, 1.0], // Maritime Navy Blue
                                vehicle_type: VehicleVisualType::MarineVessel,
                                wheel_steer_angle: self.marine_vessel.current_rudder_angle,
                                trailer_angle: 0.0,
                                headlights_on: true,
                                brake_lights_on: false,
                                reverse_lights_on: false,
                                ..Default::default()
                            });
                            fleet_mesh.append(&ship_mesh);
                        }
                        ActiveTransportMode::Aviation => {
                            let plane_mesh = generate_vehicle_mesh(&VehicleMeshParams {
                                position: self.aircraft.position,
                                orientation: self.aircraft.orientation,
                                color: [0.95, 0.95, 0.98, 1.0], // Airliner White & Silver
                                vehicle_type: VehicleVisualType::CommercialAirliner,
                                wheel_steer_angle: 0.0,
                                trailer_angle: 0.0,
                                headlights_on: true,
                                brake_lights_on: false,
                                reverse_lights_on: false,
                                ..Default::default()
                            });
                            fleet_mesh.append(&plane_mesh);
                        }
                    }
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
                    let t_type = match idx % 5 {
                        0 => VehicleVisualType::Taxi,
                        1 => VehicleVisualType::Sedan,
                        2 => VehicleVisualType::Suv,
                        3 => VehicleVisualType::SportsCoupe,
                        _ => VehicleVisualType::Sedan,
                    };
                    let t_mesh = generate_vehicle_mesh(&VehicleMeshParams {
                        position: agent.position,
                        orientation: glam::Quat::from_rotation_y(agent.yaw),
                        color: t_color,
                        vehicle_type: t_type,
                        wheel_steer_angle: 0.0,
                        trailer_angle: 0.0,
                        headlights_on: true,
                        brake_lights_on: agent.speed_ms < 1.0,
                        reverse_lights_on: false,
                        wheel_spin_angle: (agent.speed_ms * 2.5) % (2.0 * std::f32::consts::PI),
                        wheel_compressions: [0.0; 4],
                        lod: VehicleLodLevel::Lod1Medium,
                    });
                    fleet_mesh.append(&t_mesh);
                }

                // Append 3D Traffic Signals
                for light in &self.traffic_sim.traffic_lights {
                    let visual_state = match light.state_for_direction(true) {
                        TrafficLightState::Green => TrafficSignalVisualState::Green,
                        TrafficLightState::Amber => TrafficSignalVisualState::Amber,
                        TrafficLightState::Red => TrafficSignalVisualState::Red,
                    };
                    let sig_mesh = generate_traffic_signal_mesh(light.position, 0.0, visual_state);
                    fleet_mesh.append(&sig_mesh);
                }

                // Append 3D Road Signs at Stop Lines
                for stop_line in &self.traffic_sim.stop_lines {
                    let sign_pos = stop_line.position + glam::Vec3::new(3.5, 0.0, 0.0);
                    let sign_mesh = generate_road_sign_mesh(sign_pos, 0.0, RoadSignType::Stop);
                    fleet_mesh.append(&sign_mesh);
                }

                // Append 3D Animated Pedestrians
                let ped_colors = [
                    [0.2, 0.5, 0.8, 1.0],
                    [0.8, 0.2, 0.2, 1.0],
                    [0.3, 0.7, 0.4, 1.0],
                    [0.9, 0.6, 0.1, 1.0],
                ];
                for (idx, ped) in self.traffic_sim.pedestrians.iter().enumerate() {
                    let ped_mesh = generate_pedestrian_mesh(
                        ped.position,
                        ped.yaw,
                        ped.walk_cycle_phase,
                        ped_colors[idx % ped_colors.len()],
                    );
                    fleet_mesh.append(&ped_mesh);
                }

                // Append remote multiplayer vehicles
                if let Some(ref client) = self.multiplayer_client {
                    let now_ms = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    let remotes = client.get_remote_vehicles(now_ms);
                    for remote in remotes {
                        let v_type = match remote.vehicle_type.as_str() {
                            "Motorcycle" => VehicleVisualType::Motorcycle,
                            "TransitBus" => VehicleVisualType::TransitBus,
                            "HeavyTruck" => VehicleVisualType::HeavyTruck,
                            "TrainLocomotive" => VehicleVisualType::TrainLocomotive,
                            "MarineVessel" => VehicleVisualType::MarineVessel,
                            "CommercialAirliner" => VehicleVisualType::CommercialAirliner,
                            _ => VehicleVisualType::SportsCoupe,
                        };
                        let r_mesh = generate_vehicle_mesh(&VehicleMeshParams {
                            position: remote.position,
                            orientation: remote.orientation,
                            color: remote.color,
                            vehicle_type: v_type,
                            wheel_steer_angle: 0.0,
                            trailer_angle: 0.0,
                            headlights_on: true,
                            brake_lights_on: remote.speed_kph < 2.0,
                            reverse_lights_on: false,
                            ..Default::default()
                        });
                        fleet_mesh.append(&r_mesh);
                    }
                }

                if !fleet_mesh.is_empty() {
                    renderer.render_mesh(&fleet_mesh, &color_view, depth_view, false);
                }

                // 8. Render Modern Stage 6 Glassmorphic UI & In-Game Telemetry
                {
                    let mut hud = HudMesh::new();
                    let width_f = surface_config.width as f32;
                    let height_f = surface_config.height as f32;

                    match self.ui_state {
                        AppUiState::StartScreen => {
                            let options = [
                                (
                                    "CONTINUE CAREER",
                                    "Resume last saved session or active profile",
                                ),
                                (
                                    "FREE ROAM SIMULATION",
                                    "Explore open surveyed terrain & landmarks",
                                ),
                                (
                                    "CAREER CONTRACTS",
                                    "Select taxi, cargo, and freight dispatch missions",
                                ),
                                (
                                    "REGIONAL GEOGRAPHY",
                                    "Browse 38 high-precision US surveyed cities",
                                ),
                                (
                                    "VEHICLE FLEET GARAGE",
                                    "Inspect and choose from 10 multimodal vehicles",
                                ),
                                (
                                    "SETTINGS & PREFERENCES",
                                    "Configure units, graphics, and camera settings",
                                ),
                                ("EXIT TO DESKTOP", "Quit the planetary simulation"),
                            ];
                            let layout =
                                StartScreenLayout::compute(width_f, height_f, options.len());
                            let hit = self.cursor_pos.and_then(|(px, py)| layout.hit_test(px, py));
                            let hovered_opt = hit.and_then(|h| {
                                if let StartScreenHit::Option(i) = h {
                                    Some(i)
                                } else {
                                    None
                                }
                            });
                            let hovered_btn =
                                hit.filter(|h| matches!(h, StartScreenHit::QuickRoam));
                            let pressed_opt = if self.mouse_left_down {
                                hovered_opt
                            } else {
                                None
                            };
                            let pressed_btn = if self.mouse_left_down {
                                hovered_btn
                            } else {
                                None
                            };

                            hud.add_start_screen_interactive(
                                width_f,
                                height_f,
                                self.start_menu_selected_idx,
                                hovered_opt,
                                pressed_opt,
                                hovered_btn,
                                pressed_btn,
                                &options,
                                &self.player_profile.name,
                                self.player_profile.wallet.balance(),
                                self.player_profile.completed_jobs as u32,
                                Self::check_saved_game_exists(),
                            );
                        }

                        AppUiState::PauseMenu => {
                            let options = [
                                "RESUME SIMULATION",
                                "REGIONAL DISPATCH & JOBS [J]",
                                "VEHICLE FLEET GARAGE [G]",
                                "WORLD REGION CATALOG (38 CITIES) [U]",
                                "ROADSIDE RECOVERY & RESET [R]",
                                "QUICKSAVE PROGRESS [F5]",
                                "QUICKLOAD PROGRESS [F9]",
                                "SETTINGS & PREFERENCES [O]",
                                "RETURN TO TITLE SCREEN",
                                "EXIT TO DESKTOP",
                            ];
                            let layout = PauseMenuLayout::compute(width_f, height_f, options.len());
                            let hit = self.cursor_pos.and_then(|(px, py)| layout.hit_test(px, py));
                            let hovered_opt = hit.and_then(|h| {
                                if let PauseMenuHit::Option(i) = h {
                                    Some(i)
                                } else {
                                    None
                                }
                            });
                            let hovered_btn = hit.filter(|h| {
                                matches!(h, PauseMenuHit::Resume | PauseMenuHit::Close)
                            });
                            let pressed_opt = if self.mouse_left_down {
                                hovered_opt
                            } else {
                                None
                            };
                            let pressed_btn = if self.mouse_left_down {
                                hovered_btn
                            } else {
                                None
                            };

                            hud.add_pause_menu_interactive(
                                width_f,
                                height_f,
                                self.pause_menu_selected_idx,
                                hovered_opt,
                                pressed_opt,
                                hovered_btn,
                                pressed_btn,
                                &options,
                                res.region_name,
                            );
                        }

                        AppUiState::JobBrowser => {
                            let available =
                                JobCatalogGenerator::jobs_for_region(&self.config.region_id);
                            let jobs_tuples: Vec<(
                                &str,
                                &str,
                                &str,
                                &str,
                                f64,
                                f64,
                                u32,
                                &str,
                                bool,
                            )> = available
                                .iter()
                                .map(|j| {
                                    let career_str = j.career.display_name();
                                    let diff_str = match j.difficulty {
                                        realearth_gameplay::JobDifficulty::Relaxed => "Relaxed",
                                        realearth_gameplay::JobDifficulty::Standard => "Standard",
                                        realearth_gameplay::JobDifficulty::Challenging => {
                                            "Challenging"
                                        }
                                        realearth_gameplay::JobDifficulty::Extreme => "Extreme",
                                    };
                                    let can_accept = self.player_profile.career_tier(j.career)
                                        >= j.required_tier;
                                    (
                                        j.id.as_str(),
                                        career_str,
                                        j.title.as_str(),
                                        j.description.as_str(),
                                        j.route_distance_m / 1000.0,
                                        j.base_payout,
                                        j.xp_reward,
                                        diff_str,
                                        can_accept,
                                    )
                                })
                                .collect();

                            let layout = JobBrowserLayout::compute(
                                width_f,
                                height_f,
                                self.job_browser_selected_idx,
                                jobs_tuples.len(),
                            );
                            let hit = self.cursor_pos.and_then(|(px, py)| layout.hit_test(px, py));
                            let hovered_idx = hit.and_then(|h| {
                                if let JobBrowserHit::JobItem(i) = h {
                                    Some(i)
                                } else {
                                    None
                                }
                            });
                            let hovered_btn = hit.filter(|h| {
                                matches!(h, JobBrowserHit::AcceptContract | JobBrowserHit::Back)
                            });
                            let pressed_idx = if self.mouse_left_down {
                                hovered_idx
                            } else {
                                None
                            };
                            let pressed_btn = if self.mouse_left_down {
                                hovered_btn
                            } else {
                                None
                            };

                            hud.add_job_browser_modal_interactive(
                                width_f,
                                height_f,
                                self.job_browser_selected_idx,
                                hovered_idx,
                                pressed_idx,
                                hovered_btn,
                                pressed_btn,
                                &jobs_tuples,
                                self.active_job.as_ref().map(|j| j.id.as_str()),
                            );
                        }

                        AppUiState::GarageSelect => {
                            let vehicles = [
                                (
                                    "Apex GT Sports Coupe",
                                    "Sports Coupe",
                                    "RWD 6-Speed",
                                    1450.0,
                                    380.0,
                                    260.0,
                                    self.active_transport_mode == ActiveTransportMode::Car
                                        && self.vehicle_visual_type
                                            == VehicleVisualType::SportsCoupe,
                                ),
                                (
                                    "Executive Sedan",
                                    "Sedan",
                                    "AWD 8-Speed",
                                    1750.0,
                                    280.0,
                                    220.0,
                                    self.active_transport_mode == ActiveTransportMode::Car
                                        && self.vehicle_visual_type == VehicleVisualType::Sedan,
                                ),
                                (
                                    "Overland Explorer SUV",
                                    "Full-Size SUV",
                                    "4WD Low-Range",
                                    2250.0,
                                    320.0,
                                    195.0,
                                    self.active_transport_mode == ActiveTransportMode::Car
                                        && self.vehicle_visual_type == VehicleVisualType::Suv,
                                ),
                                (
                                    "Metro Yellow Cab",
                                    "City Taxi",
                                    "FWD Hybrid",
                                    1600.0,
                                    180.0,
                                    175.0,
                                    self.active_transport_mode == ActiveTransportMode::Car
                                        && self.vehicle_visual_type == VehicleVisualType::Taxi,
                                ),
                                (
                                    "CityLine Transit Bus",
                                    "Transit Bus",
                                    "Rear-Engine Air Brake",
                                    11500.0,
                                    350.0,
                                    105.0,
                                    self.active_transport_mode == ActiveTransportMode::TransitBus,
                                ),
                                (
                                    "TransContinental Semi",
                                    "Heavy Truck",
                                    "6x4 Twin-Axle",
                                    18500.0,
                                    560.0,
                                    120.0,
                                    self.active_transport_mode == ActiveTransportMode::HeavyTruck,
                                ),
                                (
                                    "HyperSport RR-1000",
                                    "Motorcycle",
                                    "Chain-Drive 6-Speed",
                                    195.0,
                                    205.0,
                                    295.0,
                                    self.active_transport_mode == ActiveTransportMode::Motorcycle,
                                ),
                                (
                                    "Pacific Intercity Consist",
                                    "Locomotive Consist",
                                    "Dual-Cab Electric 25kV",
                                    165000.0,
                                    6000.0,
                                    160.0,
                                    self.active_transport_mode == ActiveTransportMode::Train,
                                ),
                                (
                                    "Golden Gate Passenger Ferry",
                                    "Marine Ferry",
                                    "Twin Screw Diesel",
                                    420000.0,
                                    4800.0,
                                    42.0,
                                    self.active_transport_mode == ActiveTransportMode::Marine,
                                ),
                                (
                                    "SkyCourier B737-800",
                                    "Commercial Airliner",
                                    "Twin Turbofan High-Bypass",
                                    79000.0,
                                    54000.0,
                                    850.0,
                                    self.active_transport_mode == ActiveTransportMode::Aviation,
                                ),
                            ];
                            let layout = GarageLayout::compute(width_f, height_f, vehicles.len());
                            let hit = self.cursor_pos.and_then(|(px, py)| layout.hit_test(px, py));
                            let hovered_idx = hit.and_then(|h| {
                                if let GarageHit::VehicleItem(i) = h {
                                    Some(i)
                                } else {
                                    None
                                }
                            });
                            let hovered_btn = hit.filter(|h| {
                                matches!(h, GarageHit::SelectAndSpawn | GarageHit::Back)
                            });
                            let pressed_idx = if self.mouse_left_down {
                                hovered_idx
                            } else {
                                None
                            };
                            let pressed_btn = if self.mouse_left_down {
                                hovered_btn
                            } else {
                                None
                            };

                            hud.add_garage_modal_interactive_with_color(
                                width_f,
                                height_f,
                                self.garage_selected_idx,
                                self.garage_selected_color_idx,
                                hovered_idx,
                                pressed_idx,
                                hovered_btn,
                                pressed_btn,
                                &vehicles,
                            );
                        }

                        AppUiState::SettingsMenu => {
                            let settings = [
                                (
                                    "SPEED UNITS",
                                    if self.use_metric_units {
                                        "KM/H (METRIC)"
                                    } else {
                                        "MPH (IMPERIAL)"
                                    },
                                    "Toggle vehicle speedometer readout units",
                                ),
                                (
                                    "GRAPHICS QUALITY",
                                    match self.config.preset_level {
                                        GraphicsPresetLevel::Low => "LOW (1.5KM)",
                                        GraphicsPresetLevel::Medium => "MEDIUM (2.5KM)",
                                        GraphicsPresetLevel::High => "HIGH (5.0KM)",
                                        GraphicsPresetLevel::Adaptive => "ADAPTIVE DYNAMIC",
                                    },
                                    "Visual fidelity and LOD chunk streaming distance",
                                ),
                                (
                                    "CAMERA PERSPECTIVE",
                                    match self.vehicle_cam.perspective {
                                        VehicleCameraPerspective::Chase => "CHASE (THIRD PERSON)",
                                        VehicleCameraPerspective::Hood => "HOOD MOUNT",
                                        VehicleCameraPerspective::Cockpit => {
                                            "COCKPIT (FIRST PERSON)"
                                        }
                                        VehicleCameraPerspective::Bumper => "BUMPER MOUNT",
                                    },
                                    "Active vehicle driving camera viewpoint",
                                ),
                                (
                                    "DEVELOPER OVERLAY [F3]",
                                    if self.show_debug_overlay {
                                        "ENABLED (VISIBLE)"
                                    } else {
                                        "DISABLED (HIDDEN)"
                                    },
                                    "Toggle F3 diagnostic metrics HUD",
                                ),
                                (
                                    "HUD DISPLAY [H]",
                                    if self.show_hud {
                                        "ENABLED (ON)"
                                    } else {
                                        "DISABLED (OFF)"
                                    },
                                    "Master heads-up display overlay",
                                ),
                            ];
                            let layout = SettingsLayout::compute(width_f, height_f, settings.len());
                            let hit = self.cursor_pos.and_then(|(px, py)| layout.hit_test(px, py));
                            let hovered_idx = hit.and_then(|h| {
                                if let SettingsHit::SettingItem(i) = h {
                                    Some(i)
                                } else {
                                    None
                                }
                            });
                            let hovered_btn = hit.filter(|h| matches!(h, SettingsHit::Back));
                            let pressed_idx = if self.mouse_left_down {
                                hovered_idx
                            } else {
                                None
                            };
                            let pressed_btn = if self.mouse_left_down {
                                hovered_btn
                            } else {
                                None
                            };

                            hud.add_settings_modal_interactive(
                                width_f,
                                height_f,
                                self.settings_selected_idx,
                                hovered_idx,
                                pressed_idx,
                                hovered_btn,
                                pressed_btn,
                                &settings,
                            );
                        }

                        AppUiState::CatalogModal => {
                            let cities_data: Vec<(&str, &str, &str, &str, f32, bool, &str)> =
                                all_us_cities()
                                    .iter()
                                    .map(|c| {
                                        let tier_str = match c.priority {
                                            CityPriorityTier::Priority1Primary => "P1-Primary",
                                            CityPriorityTier::Priority2Major => "P2-Major",
                                            CityPriorityTier::Priority3Extended => "P3-Extend",
                                        };
                                        (
                                            c.id,
                                            c.name,
                                            c.state,
                                            c.anchor_landmark_name,
                                            c.elevation_max_m,
                                            c.has_maritime_port,
                                            tier_str,
                                        )
                                    })
                                    .collect();
                            let layout = CatalogLayout::compute(
                                width_f,
                                height_f,
                                self.catalog_selected_idx,
                                cities_data.len(),
                            );
                            let hit = self.cursor_pos.and_then(|(px, py)| layout.hit_test(px, py));
                            let hovered_idx = hit.and_then(|h| {
                                if let CatalogHit::CityItem(i) = h {
                                    Some(i)
                                } else {
                                    None
                                }
                            });
                            let hovered_btn = hit
                                .filter(|h| matches!(h, CatalogHit::WarpCity | CatalogHit::Back));
                            let pressed_idx = if self.mouse_left_down {
                                hovered_idx
                            } else {
                                None
                            };
                            let pressed_btn = if self.mouse_left_down {
                                hovered_btn
                            } else {
                                None
                            };

                            hud.add_city_catalog_modal_interactive(
                                width_f,
                                height_f,
                                self.catalog_selected_idx,
                                hovered_idx,
                                pressed_idx,
                                hovered_btn,
                                pressed_btn,
                                &cities_data,
                            );
                        }

                        AppUiState::DebriefModal => {
                            if let Some(ref debrief) = self.active_debrief {
                                hud.add_debrief_modal(
                                    width_f,
                                    height_f,
                                    &debrief.title,
                                    &debrief.career,
                                    debrief.base_payout,
                                    debrief.tip_bonus,
                                    debrief.speed_bonus,
                                    debrief.net_payout,
                                    debrief.xp_reward,
                                    debrief.comfort_stars,
                                );
                            }
                        }

                        AppUiState::MultiplayerModal => {
                            let (is_connected, is_hosting, ping_ms) =
                                if let Some(ref client) = self.multiplayer_client {
                                    (
                                        client.state
                                            == realearth_networking::ConnectionState::Connected,
                                        self.multiplayer_server_handle.is_some(),
                                        client.ping_ms,
                                    )
                                } else {
                                    (false, false, 0.0)
                                };
                            let roster =
                                [(self.config.player_name.as_str(), "Local Driver", ping_ms)];
                            hud.add_multiplayer_modal(
                                width_f,
                                height_f,
                                is_connected,
                                is_hosting,
                                &self.multiplayer_server_addr_input,
                                ping_ms,
                                &self.config.player_name,
                                self.multiplayer_selected_action,
                                &roster,
                            );
                        }

                        AppUiState::InGame => {
                            if self.show_hud {
                                let veh_coord = Wgs84Coord::new(
                                    anchor.lat
                                        + (self.vehicle.position.z as f64 / meters_per_deg_lat),
                                    anchor.lon
                                        + (self.vehicle.position.x as f64 / meters_per_deg_lon),
                                    self.vehicle.position.y as f64,
                                );
                                let current_speed_kph = match self.active_transport_mode {
                                    ActiveTransportMode::Car => self.vehicle.speed_kph.abs(),
                                    ActiveTransportMode::Motorcycle => {
                                        self.motorcycle.speed_kph.abs()
                                    }
                                    ActiveTransportMode::TransitBus => {
                                        self.transit_bus.speed_kph.abs()
                                    }
                                    ActiveTransportMode::HeavyTruck => {
                                        self.heavy_truck.speed_kph.abs()
                                    }
                                    ActiveTransportMode::Train => {
                                        self.train_consist.speed_kph.abs()
                                    }
                                    ActiveTransportMode::Marine => {
                                        self.marine_vessel.speed_knots * 1.852
                                    }
                                    ActiveTransportMode::Aviation => {
                                        self.aircraft.true_airspeed_mps * 3.6
                                    }
                                };

                                // Location & Compass Pill (Top Left, unless debug overlay active)
                                if !self.show_debug_overlay {
                                    let rot_mat = glam::Mat3::from_quat(self.vehicle.orientation);
                                    let forward_vec = rot_mat.col(2);
                                    let heading_deg = (forward_vec.x.atan2(forward_vec.z) as f64)
                                        .to_degrees()
                                        .rem_euclid(360.0);
                                    let heading_cardinal = if (337.5..=360.0).contains(&heading_deg)
                                        || (0.0..22.5).contains(&heading_deg)
                                    {
                                        "N"
                                    } else if (22.5..67.5).contains(&heading_deg) {
                                        "NE"
                                    } else if (67.5..112.5).contains(&heading_deg) {
                                        "E"
                                    } else if (112.5..157.5).contains(&heading_deg) {
                                        "SE"
                                    } else if (157.5..202.5).contains(&heading_deg) {
                                        "S"
                                    } else if (202.5..247.5).contains(&heading_deg) {
                                        "SW"
                                    } else if (247.5..292.5).contains(&heading_deg) {
                                        "W"
                                    } else {
                                        "NW"
                                    };
                                    let current_street = self
                                        .active_nav_update
                                        .as_ref()
                                        .and_then(|u| u.current_instruction.as_ref())
                                        .and_then(|i| i.street_name.as_deref())
                                        .unwrap_or(res.landmark.name);
                                    hud.add_location_pill(
                                        16.0,
                                        16.0,
                                        res.region_name,
                                        current_street,
                                        heading_cardinal,
                                    );
                                }

                                // 1. Turn-by-Turn Navigation Guidance Overlay Banner
                                if let Some(ref update) = self.active_nav_update {
                                    let (icon, instruction, street_name) =
                                        if let Some(ref inst) = update.current_instruction {
                                            let icon_str = match inst.maneuver {
                                                ManeuverType::Depart => "->",
                                                ManeuverType::Continue => "^^",
                                                ManeuverType::TurnLeft
                                                | ManeuverType::SharpLeft
                                                | ManeuverType::SlightLeft => "<-",
                                                ManeuverType::TurnRight
                                                | ManeuverType::SharpRight
                                                | ManeuverType::SlightRight => "->",
                                                ManeuverType::Uturn => "<>",
                                                ManeuverType::EnterRoundabout
                                                | ManeuverType::ExitRoundabout => "()",
                                                ManeuverType::TakeRamp => "/>",
                                                ManeuverType::Arrive => "**",
                                            };
                                            (
                                                icon_str,
                                                inst.instruction.as_str(),
                                                inst.street_name.as_deref(),
                                            )
                                        } else {
                                            ("^^", "Proceed along route", None)
                                        };

                                    hud.add_navigation_guidance(
                                        width_f,
                                        height_f,
                                        icon,
                                        update.distance_to_next_maneuver_meters,
                                        instruction,
                                        street_name,
                                        45,
                                        update.is_wrong_way,
                                    );
                                }

                                // 2. Instrument Cluster (Speedometer or Multimodal Specialized Clusters)
                                if self.config.is_drive {
                                    match self.active_transport_mode {
                                        ActiveTransportMode::Car => {
                                            let gear_str = if self.park_engaged {
                                                "P"
                                            } else {
                                                match self.vehicle.current_gear {
                                                    -1 => "R",
                                                    0 => "N",
                                                    1 => "D1",
                                                    2 => "D2",
                                                    3 => "D3",
                                                    4 => "D4",
                                                    5 => "D5",
                                                    _ => "D",
                                                }
                                            };
                                            let steer_deg =
                                                self.vehicle.wheels[0].steer_angle.to_degrees();
                                            hud.add_speedometer_cluster_assisted(
                                                16.0,
                                                height_f - 145.0,
                                                self.vehicle.speed_kph,
                                                gear_str,
                                                self.vehicle.engine_rpm,
                                                7500.0,
                                                steer_deg,
                                                self.use_metric_units,
                                                self.vehicle_visual_type.display_name(),
                                                self.vehicle.config.abs_enabled,
                                                self.vehicle.config.tcs_enabled,
                                                self.vehicle.config.esc_enabled,
                                                self.park_engaged,
                                            );
                                        }
                                        ActiveTransportMode::Motorcycle => {
                                            hud.add_motorcycle_cluster(
                                                16.0,
                                                height_f - 145.0,
                                                self.motorcycle.speed_kph,
                                                self.motorcycle.lean_angle,
                                                self.motorcycle.steer_angle,
                                                (self.motorcycle.speed_kph * 95.0 + 1200.0)
                                                    .min(11500.0),
                                                self.motorcycle.is_crashed,
                                            );
                                        }
                                        ActiveTransportMode::TransitBus => {
                                            hud.add_transit_bus_cluster(
                                                16.0,
                                                height_f - 145.0,
                                                self.transit_bus.speed_kph,
                                                self.transit_bus.passenger_count,
                                                self.transit_bus.total_mass_kg,
                                                self.transit_bus.passenger_comfort_rating,
                                                self.transit_bus.air_brake_pressure_bar,
                                            );
                                        }
                                        ActiveTransportMode::HeavyTruck => {
                                            let speed_mps =
                                                self.heavy_truck.speed_kph * (1000.0 / 3600.0);
                                            let yaw_rate = speed_mps
                                                * (self.heavy_truck.steer_angle
                                                    / self.heavy_truck.tractor_wheelbase);
                                            let lat_accel = speed_mps * yaw_rate;
                                            let gear_str = if self.heavy_truck.linear_velocity.dot(
                                                self.heavy_truck.tractor_orientation
                                                    * glam::Vec3::Z,
                                            ) < -0.1
                                            {
                                                "R"
                                            } else {
                                                "D4"
                                            };
                                            hud.add_heavy_truck_cluster(
                                                16.0,
                                                height_f - 145.0,
                                                self.heavy_truck.speed_kph,
                                                gear_str,
                                                self.heavy_truck
                                                    .trailer_articulation_angle
                                                    .to_degrees(),
                                                (lat_accel / 9.81).abs(),
                                                8.2,
                                                self.heavy_truck.is_rolled_over,
                                            );
                                        }
                                        ActiveTransportMode::Train => {
                                            hud.add_train_cluster(
                                                16.0,
                                                height_f - 145.0,
                                                self.train_consist.speed_kph,
                                                self.train_consist.throttle,
                                                self.train_consist.brake_pipe_pressure_bar,
                                                self.train_consist.brake_cylinder_pressure_bar,
                                                self.train_consist.rated_power_watts / 1000.0,
                                                self.train_consist.reverser,
                                                match self.train_consist.current_signal {
                                                    realearth_navigation::rail_graph::SignalAspect::Clear => "Clear",
                                                    realearth_navigation::rail_graph::SignalAspect::Approach => "Approach",
                                                    realearth_navigation::rail_graph::SignalAspect::Stop => "Stop",
                                                },
                                                self.train_consist.wheel_slip,
                                            );
                                        }
                                        ActiveTransportMode::Marine => {
                                            let marine_throttle = if self
                                                .pressed_keys
                                                .contains(&KeyCode::KeyW)
                                                || self.pressed_keys.contains(&KeyCode::ArrowUp)
                                            {
                                                1.0
                                            } else if self.pressed_keys.contains(&KeyCode::KeyS)
                                                || self.pressed_keys.contains(&KeyCode::ArrowDown)
                                            {
                                                -0.6
                                            } else {
                                                0.0
                                            };
                                            hud.add_marine_cluster(
                                                16.0,
                                                height_f - 145.0,
                                                self.marine_vessel.speed_knots,
                                                marine_throttle,
                                                self.marine_vessel
                                                    .current_rudder_angle
                                                    .to_degrees(),
                                                self.marine_vessel
                                                    .heading_radians
                                                    .to_degrees()
                                                    .rem_euclid(360.0),
                                                self.marine_vessel.design_draft_meters,
                                                self.marine_vessel.metacentric_height_gm,
                                            );
                                        }
                                        ActiveTransportMode::Aviation => {
                                            hud.add_aviation_pfd(
                                                width_f - 390.0,
                                                height_f - 240.0,
                                                self.aircraft.indicated_airspeed_knots,
                                                self.aircraft.altitude_msl_meters,
                                                self.aircraft.climb_rate_mps,
                                                self.aircraft
                                                    .orientation
                                                    .to_euler(glam::EulerRot::YXZ)
                                                    .1,
                                                self.aircraft
                                                    .orientation
                                                    .to_euler(glam::EulerRot::YXZ)
                                                    .2,
                                                0.25,
                                                self.aircraft.is_stalled,
                                                self.aircraft.altitude_msl_meters < 50.0
                                                    || !self.aircraft.is_airborne,
                                            );
                                        }
                                    }
                                }

                                // 3. Tactical GPS Minimap (Bottom Right)
                                let (rel_target, is_pickup) = if let Some(ref job) = self.active_job
                                {
                                    let (target_coord, is_pick) = match job.state {
                                        realearth_gameplay::JobState::EnRouteToPickup
                                        | realearth_gameplay::JobState::AtPickup { .. } => {
                                            (job.pickup.coord, true)
                                        }
                                        realearth_gameplay::JobState::EnRouteToDropoff
                                        | realearth_gameplay::JobState::AtDropoff { .. } => {
                                            (job.dropoff.coord, false)
                                        }
                                        _ => (job.pickup.coord, true),
                                    };
                                    let d_lat = (target_coord.lat - self.camera_ctrl.coord.lat)
                                        * meters_per_deg_lat;
                                    let d_lon = (target_coord.lon - self.camera_ctrl.coord.lon)
                                        * meters_per_deg_lon;
                                    let cos_yaw = (-self.camera_ctrl.yaw as f64).cos();
                                    let sin_yaw = (-self.camera_ctrl.yaw as f64).sin();
                                    let rx = (d_lon * cos_yaw - d_lat * sin_yaw) as f32;
                                    let rz = (d_lon * sin_yaw + d_lat * cos_yaw) as f32;
                                    (Some((rx, rz)), is_pick)
                                } else {
                                    (None, true)
                                };
                                hud.add_gps_minimap(
                                    width_f - 85.0,
                                    height_f - 85.0,
                                    65.0,
                                    self.camera_ctrl.yaw,
                                    rel_target,
                                    is_pickup,
                                );

                                // 4. Active Mission HUD Card (Top Right)
                                if let Some(ref job) = self.active_job {
                                    let (step_desc, dist_to_target) = match job.state {
                                        realearth_gameplay::JobState::EnRouteToPickup => (
                                            "Drive to pickup zone",
                                            job.pickup.distance_to(veh_coord),
                                        ),
                                        realearth_gameplay::JobState::AtPickup { .. } => {
                                            ("Stop vehicle to board / load", 0.0)
                                        }
                                        realearth_gameplay::JobState::EnRouteToDropoff => (
                                            "Drive to delivery destination",
                                            job.dropoff.distance_to(veh_coord),
                                        ),
                                        realearth_gameplay::JobState::AtDropoff { .. } => {
                                            ("Stop vehicle to complete mission", 0.0)
                                        }
                                        realearth_gameplay::JobState::Completed { .. } => {
                                            ("Contract completed!", 0.0)
                                        }
                                        realearth_gameplay::JobState::Failed { .. } => {
                                            ("Contract failed", 0.0)
                                        }
                                        _ => ("En route", 0.0),
                                    };
                                    hud.add_mission_hud_card(
                                        width_f - 420.0,
                                        16.0,
                                        404.0,
                                        100.0,
                                        job.career.display_name(),
                                        &job.title,
                                        step_desc,
                                        dist_to_target,
                                        job.comfort_score,
                                        job.cargo_integrity,
                                        job.base_payout,
                                    );
                                }

                                // 5. Boarding / Loading Dwell Progress
                                if let Some(ref job) = self.active_job {
                                    match job.state {
                                        realearth_gameplay::JobState::AtPickup {
                                            dwell_timer,
                                            required_dwell,
                                        } => {
                                            let progress =
                                                (dwell_timer / required_dwell).clamp(0.0, 1.0);
                                            hud.add_dwell_progress(
                                                width_f,
                                                height_f,
                                                progress,
                                                "PASSENGER BOARDING / CARGO LOADING",
                                                current_speed_kph < 2.0,
                                            );
                                        }
                                        realearth_gameplay::JobState::AtDropoff {
                                            dwell_timer,
                                            required_dwell,
                                        } => {
                                            let progress =
                                                (dwell_timer / required_dwell).clamp(0.0, 1.0);
                                            hud.add_dwell_progress(
                                                width_f,
                                                height_f,
                                                progress,
                                                "DISEMBARKING / UNLOADING CARGO",
                                                current_speed_kph < 2.0,
                                            );
                                        }
                                        _ => {}
                                    }
                                }

                                // 6. Developer Telemetry Overlay (Top Left, [F3])
                                if self.show_debug_overlay {
                                    let avg_ms = if self.frame_durations_ms.is_empty() {
                                        16.6
                                    } else {
                                        self.frame_durations_ms.iter().sum::<f64>()
                                            / self.frame_durations_ms.len() as f64
                                    };
                                    let fps = (1000.0 / avg_ms).max(1.0);
                                    let one_pct_low_fps = if self.frame_durations_ms.len() >= 10 {
                                        let mut sorted = self.frame_durations_ms.clone();
                                        sorted.sort_by(|a, b| {
                                            a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
                                        });
                                        let idx = ((sorted.len() as f64 * 0.99).floor() as usize)
                                            .min(sorted.len() - 1);
                                        1000.0 / sorted[idx].max(0.1)
                                    } else {
                                        fps
                                    };
                                    let proc_mem = get_current_process_memory();
                                    let vram_mb =
                                        renderer.gpu_memory_bytes() as f64 / (1024.0 * 1024.0);
                                    let coord_str = format!(
                                        "{:.4}N, {:.4}W, {:.1}m | Anchor: {}",
                                        self.camera_ctrl.coord.lat,
                                        self.camera_ctrl.coord.lon,
                                        self.camera_ctrl.coord.alt,
                                        res.landmark.name
                                    );
                                    hud.add_debug_overlay(
                                        16.0,
                                        16.0,
                                        fps,
                                        one_pct_low_fps,
                                        avg_ms,
                                        &format!("{:?}", renderer.settings.level),
                                        &coord_str,
                                        active.len(),
                                        self.last_culled_chunks,
                                        self.last_culled_triangles,
                                        proc_mem.working_set_size_mb(),
                                        vram_mb,
                                    );

                                    if self.config.is_drive
                                        && self.active_transport_mode == ActiveTransportMode::Car
                                    {
                                        let rot_mat =
                                            glam::Mat3::from_quat(self.vehicle.orientation);
                                        let fwd = rot_mat.col(2);
                                        let contacts = self
                                            .vehicle
                                            .wheels
                                            .iter()
                                            .filter(|w| w.in_contact)
                                            .count();
                                        let steer_deg =
                                            self.vehicle.wheels[0].steer_angle.to_degrees();
                                        let (g_elev, g_normal) = (
                                            gq.elevation(
                                                self.vehicle.position.x,
                                                self.vehicle.position.z,
                                            ),
                                            gq.normal(
                                                self.vehicle.position.x,
                                                self.vehicle.position.z,
                                            ),
                                        );
                                        let road_grade_pct =
                                            (1.0 - g_normal.y.clamp(0.0, 1.0)).sqrt() * 100.0;
                                        let gear_label = if self.park_engaged {
                                            "PARK [P]"
                                        } else {
                                            match self.vehicle.current_gear {
                                                -1 => "REVERSE",
                                                0 => "NEUTRAL",
                                                _ => "DRIVE",
                                            }
                                        };

                                        let py = 16.0 + 95.0 + 8.0;
                                        hud.add_rect(
                                            16.0,
                                            py,
                                            480.0,
                                            76.0,
                                            [0.03, 0.04, 0.06, 0.85],
                                        );
                                        hud.add_rect_outline(
                                            16.0,
                                            py,
                                            480.0,
                                            76.0,
                                            1.0,
                                            [0.25, 0.50, 0.80, 0.5],
                                        );
                                        hud.add_text(
                                            26.0,
                                            py + 8.0,
                                            1.0,
                                            &format!(
                                                "VEHICLE DYNAMICS TELEMETRY | Mode: {}",
                                                gear_label
                                            ),
                                            [0.3, 0.85, 1.0, 1.0],
                                        );
                                        hud.add_text(26.0, py + 24.0, 0.9, &format!("Forward: [{:.3}, {:.3}, {:.3}] | Steer: {:>+5.1}° | Contacts: {}/4", fwd.x, fwd.y, fwd.z, steer_deg, contacts), [0.85, 0.90, 0.95, 1.0]);
                                        hud.add_text(26.0, py + 40.0, 0.9, &format!("Speed: {:>5.1} km/h | Vel: [{:.2}, {:.2}, {:.2}] m/s", self.vehicle.speed_kph, self.vehicle.linear_velocity.x, self.vehicle.linear_velocity.y, self.vehicle.linear_velocity.z), [0.85, 0.90, 0.95, 1.0]);
                                        hud.add_text(26.0, py + 56.0, 0.9, &format!("Ground Elev: {:.1}m | Road Grade: {:.1}% | Normal: [{:.2}, {:.2}, {:.2}]", g_elev, road_grade_pct, g_normal.x, g_normal.y, g_normal.z), [0.85, 0.85, 0.40, 1.0]);
                                    }
                                }

                                // 7. Multiplayer Status Chip (Top Right)
                                let (is_connected, is_reconnecting, ping_ms, remote_count) =
                                    if let Some(ref client) = self.multiplayer_client {
                                        (
                                        client.state == realearth_networking::ConnectionState::Connected,
                                        client.state == realearth_networking::ConnectionState::Reconnecting,
                                        client.ping_ms,
                                        client.get_remote_vehicles(0).len(),
                                    )
                                    } else {
                                        (false, false, 0.0, 0)
                                    };
                                hud.add_multiplayer_chip(
                                    width_f,
                                    is_connected,
                                    is_reconnecting,
                                    ping_ms,
                                    remote_count,
                                );

                                // Contextual Action Prompt
                                if let Some(ref job) = self.active_job {
                                    match job.state {
                                        realearth_gameplay::JobState::AtPickup { .. } => {
                                            hud.add_contextual_action_prompt(
                                                width_f * 0.5,
                                                height_f - 60.0,
                                                "SPACE",
                                                "HOLD TO BOARD PASSENGERS",
                                            );
                                        }
                                        realearth_gameplay::JobState::AtDropoff { .. } => {
                                            hud.add_contextual_action_prompt(
                                                width_f * 0.5,
                                                height_f - 60.0,
                                                "SPACE",
                                                "HOLD TO COMPLETE CONTRACT",
                                            );
                                        }
                                        _ => {
                                            hud.add_contextual_action_prompt(
                                                width_f * 0.5,
                                                height_f - 60.0,
                                                "TAB / J",
                                                "DISPATCH CONTRACTS",
                                            );
                                        }
                                    }
                                } else {
                                    hud.add_contextual_action_prompt(
                                        width_f * 0.5,
                                        height_f - 60.0,
                                        "J",
                                        "BROWSE DISPATCH CONTRACTS",
                                    );
                                }

                                // 8. Clean Keyboard Hint Footer
                                let hint_str = "[W/S/A/D] Drive  |  [P] Park  |  [Esc] Pause  |  [N] Multiplayer  |  [J] Jobs  |  [G] Garage  |  [U] World Cities  |  [F3] Debug";
                                let hint_w = hint_str.len() as f32 * 6.0 * 1.0;
                                let hint_x = (width_f - hint_w) * 0.5;
                                hud.add_rect(
                                    hint_x - 10.0,
                                    height_f - 24.0,
                                    hint_w + 20.0,
                                    20.0,
                                    [0.05, 0.07, 0.10, 0.75],
                                );
                                hud.add_text(
                                    hint_x,
                                    height_f - 18.0,
                                    1.0,
                                    hint_str,
                                    [0.7, 0.8, 0.9, 0.8],
                                );
                            }
                        }
                    }

                    // Temporary Status / Notification Banner (Relocated to top-center to eliminate footer collision)
                    if let Some((ref text, stamp)) = self.status_notification {
                        if stamp.elapsed().as_secs_f32() < 4.0 {
                            let banner_w = 560.0;
                            let banner_x = (width_f - banner_w) * 0.5;
                            let banner_y = if self.active_nav_update.is_some() {
                                78.0
                            } else {
                                18.0
                            };
                            hud.add_panel(banner_x, banner_y, banner_w, 36.0, [0.1, 0.9, 0.6, 1.0]);
                            hud.add_text_proportional(
                                banner_x + 14.0,
                                banner_y + 10.0,
                                text,
                                realearth_rendering::FontSize::Body.pixels(),
                                [1.0, 1.0, 1.0, 1.0],
                                realearth_rendering::FontWeight::Bold,
                                true,
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
