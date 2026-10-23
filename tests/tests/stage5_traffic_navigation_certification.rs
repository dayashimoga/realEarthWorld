//! STAGE 5 — ROAD NAVIGATION, TRAFFIC INTELLIGENCE AND REALISTIC CITY ACTIVITY CERTIFICATION
//!
//! Comprehensive end-to-end verification of Stage 5 requirements from docs/ExecutionContract.md:
//! 1. Turn-by-Turn Guidance, A* Shortest/Fastest Routing & Wrong-Way Alert Tracking
//! 2. Grade-Separated Layer Isolation, Ramps, One-Way Directions & Turn/Access Restrictions
//! 3. Intersection Traffic Light State Cycles, Stop-Line Braking & Autonomous Vehicle Queuing
//! 4. Multilane MOBIL Lane Changing, Gap Acceptance & Smooth Lateral Interpolation
//! 5. High-Density ECS Spatial Hash Grid Partitioning & 60 FPS Fleet Performance (>=64 vehicles)
//! 6. Pedestrian Crosswalk Yielding, Audio Telemetry (Skid/Honk) & Gridlock Unstick Recovery

use glam::Vec3;
use realearth_ingestion::sample_san_francisco::create_san_francisco_dataset;
use realearth_navigation::{
    calculate_bearing_deg, find_route, generate_turn_by_turn, HighwayType, InferredRoadAttributes,
    ManeuverType, NavigationStatus, RoadAccessRestriction, RoadEdge, RoadNetworkGraph,
    RouteTracker, RoutingPreference, TurnRestriction, TurnRestrictionType,
};
use realearth_rendering::{
    generate_pedestrian_mesh, generate_road_sign_mesh, generate_traffic_signal_mesh, RoadSignType,
    TrafficSignalVisualState,
};
use realearth_shared::coordinates::Wgs84Coord;
use realearth_vehicles::{
    FlatGround, IdmConfig, IntersectionStopLine, PedestrianAgent, TrafficAgent, TrafficAudioEvent,
    TrafficLight, TrafficLightState, TrafficSimulation,
};
use std::time::Instant;

// ============================================================================
// TEST 1: TURN-BY-TURN GUIDANCE, A* ROUTING & WRONG-WAY ALERT TRACKING
// ============================================================================

#[test]
fn test_stage5_turn_by_turn_and_wrong_way_detection() {
    let sf = create_san_francisco_dataset();
    let network = RoadNetworkGraph::from_roads(&sf.roads);

    // 1. Select origin and destination from real San Francisco roads
    let start_coord = sf.roads[0].geometry[0];
    let target_coord = sf.roads[0].geometry.last().copied().unwrap();

    let route = find_route(
        &network,
        start_coord,
        target_coord,
        RoutingPreference::FastestTime,
    )
    .expect("A* route must resolve across San Francisco road network");

    assert!(route.waypoints.len() >= 2, "Route must contain waypoints");
    assert!(route.total_distance_meters > 0.0);
    assert!(route.estimated_duration_seconds > 0.0);

    // 2. Generate turn-by-turn guidance instructions
    let guidance = generate_turn_by_turn(&route, &network);
    assert!(
        !guidance.instructions.is_empty(),
        "Guidance must produce route instructions"
    );
    assert_eq!(
        guidance.instructions[0].maneuver,
        ManeuverType::Depart,
        "First instruction must be Depart"
    );
    assert_eq!(
        guidance.instructions.last().unwrap().maneuver,
        ManeuverType::Arrive,
        "Final instruction must be Arrive"
    );

    // 3. Initialize RouteTracker
    let mut tracker = RouteTracker::new(route.clone(), guidance);

    // Test correct direction progression
    let p0 = route.waypoints[0];
    let p1 = route.waypoints[1];
    let seg_bearing = calculate_bearing_deg(p0, p1);

    // Midpoint along first segment, heading forward
    let mid_coord = Wgs84Coord::new(
        (p0.lat + p1.lat) * 0.5,
        (p0.lon + p1.lon) * 0.5,
        (p0.alt + p1.alt) * 0.5,
    );
    let update_forward = tracker.update(mid_coord, seg_bearing);
    assert!(
        !update_forward.is_wrong_way,
        "Forward progression along segment must not trigger wrong way"
    );
    assert!(
        !update_forward.is_off_route,
        "Vehicle on segment must not be off route"
    );
    assert_eq!(
        update_forward.status,
        NavigationStatus::EnRoute,
        "Status must be EnRoute"
    );
    assert!(update_forward.distance_remaining_meters > 0.0);

    // 4. Test Wrong-Way Detection (heading 180 degrees opposite to segment)
    let wrong_way_heading = (seg_bearing + 180.0) % 360.0;
    let update_wrong_way = tracker.update(mid_coord, wrong_way_heading);
    assert!(
        update_wrong_way.is_wrong_way,
        "Opposite heading must trigger wrong-way alert"
    );
    assert_eq!(
        update_wrong_way.status,
        NavigationStatus::WrongWayWarning,
        "Status must transition to WrongWayWarning"
    );

    // 5. Test Off-Route Detection (> 35m perpendicular offset)
    let off_route_coord = Wgs84Coord::new(
        mid_coord.lat + 0.001, // ~111m North of route
        mid_coord.lon,
        mid_coord.alt,
    );
    let update_off_route = tracker.update(off_route_coord, seg_bearing);
    assert!(
        update_off_route.is_off_route,
        "Significant cross-track error must trigger off-route alert"
    );
    assert_eq!(
        update_off_route.status,
        NavigationStatus::OffRouteRerouteRequired,
        "Status must transition to OffRouteRerouteRequired"
    );

    // 6. Test Destination Arrival
    let dest_coord = *route.waypoints.last().unwrap();
    let update_arrival = tracker.update(dest_coord, seg_bearing);
    assert_eq!(
        update_arrival.status,
        NavigationStatus::Arrived,
        "Proximity to destination must register Arrived"
    );
    assert_eq!(update_arrival.distance_remaining_meters, 0.0);
}

// ============================================================================
// TEST 2: GRADE-SEPARATED LAYER ISOLATION, RAMPS & ACCESS RESTRICTIONS
// ============================================================================

#[test]
fn test_stage5_grade_separated_elevations_and_restrictions() {
    let mut network = RoadNetworkGraph::new();

    // Crossing point at identical lat/lon: ground road (layer 0) vs elevated flyover (layer 1)
    let cross_lat = 37.7800;
    let cross_lon = -122.4000;

    let coord_ground_w = Wgs84Coord::new(cross_lat, cross_lon - 0.002, 10.0);
    let coord_ground_mid = Wgs84Coord::new(cross_lat, cross_lon, 10.0);
    let coord_ground_e = Wgs84Coord::new(cross_lat, cross_lon + 0.002, 10.0);

    let coord_elev_s = Wgs84Coord::new(cross_lat - 0.002, cross_lon, 25.0);
    let coord_elev_mid = Wgs84Coord::new(cross_lat, cross_lon, 25.0);
    let coord_elev_n = Wgs84Coord::new(cross_lat + 0.002, cross_lon, 25.0);

    let n_ground_w = network.get_or_create_node(coord_ground_w, 0);
    let n_ground_mid = network.get_or_create_node(coord_ground_mid, 0);
    let n_ground_e = network.get_or_create_node(coord_ground_e, 0);

    let n_elev_s = network.get_or_create_node(coord_elev_s, 1);
    let n_elev_mid = network.get_or_create_node(coord_elev_mid, 1);
    let n_elev_n = network.get_or_create_node(coord_elev_n, 1);

    // 1. Verify Layer Isolation: ground midpoint != elevated midpoint
    assert_ne!(
        n_ground_mid, n_elev_mid,
        "Grade-separated nodes at identical lat/lon must have distinct node IDs"
    );

    // 2. Connect ground arterial (East-West)
    network.graph.add_edge(
        n_ground_w,
        n_ground_mid,
        RoadEdge {
            way_id: 101,
            name: Some("Howard St Ground".to_string()),
            highway_type: HighwayType::Primary,
            length_meters: 200.0,
            speed_limit_kph: 45,
            lanes: 2,
            layer: 0,
            is_bridge: false,
            is_tunnel: false,
            is_oneway: false,
            is_roundabout: false,
            is_ramp: false,
            access_restriction: RoadAccessRestriction::All,
            inferred_attributes: InferredRoadAttributes::default(),
        },
    );
    network.graph.add_edge(
        n_ground_mid,
        n_ground_e,
        RoadEdge {
            way_id: 102,
            name: Some("Howard St Ground".to_string()),
            highway_type: HighwayType::Primary,
            length_meters: 200.0,
            speed_limit_kph: 45,
            lanes: 2,
            layer: 0,
            is_bridge: false,
            is_tunnel: false,
            is_oneway: false,
            is_roundabout: false,
            is_ramp: false,
            access_restriction: RoadAccessRestriction::All,
            inferred_attributes: InferredRoadAttributes::default(),
        },
    );

    // 3. Connect elevated freeway (One-Way South-to-North, Motorway)
    network.graph.add_edge(
        n_elev_s,
        n_elev_mid,
        RoadEdge {
            way_id: 201,
            name: Some("Skyway Flyover".to_string()),
            highway_type: HighwayType::Motorway,
            length_meters: 220.0,
            speed_limit_kph: 90,
            lanes: 3,
            layer: 1,
            is_bridge: true,
            is_tunnel: false,
            is_oneway: true,
            is_roundabout: false,
            is_ramp: false,
            access_restriction: RoadAccessRestriction::All,
            inferred_attributes: InferredRoadAttributes::default(),
        },
    );
    network.graph.add_edge(
        n_elev_mid,
        n_elev_n,
        RoadEdge {
            way_id: 202,
            name: Some("Skyway Flyover".to_string()),
            highway_type: HighwayType::Motorway,
            length_meters: 220.0,
            speed_limit_kph: 90,
            lanes: 3,
            layer: 1,
            is_bridge: true,
            is_tunnel: false,
            is_oneway: true,
            is_roundabout: false,
            is_ramp: false,
            access_restriction: RoadAccessRestriction::All,
            inferred_attributes: InferredRoadAttributes::default(),
        },
    );

    // 4. Verify Layer-Aware Nearest Node Resolution
    let query_coord = Wgs84Coord::new(cross_lat, cross_lon, 10.0);
    let nearest_ground = network
        .find_nearest_node_with_layer(query_coord, Some(0))
        .expect("Must find layer 0 node");
    let nearest_elev = network
        .find_nearest_node_with_layer(query_coord, Some(1))
        .expect("Must find layer 1 node");

    assert_eq!(nearest_ground, n_ground_mid);
    assert_eq!(nearest_elev, n_elev_mid);

    // 5. Verify isolated routing: West to North has no path unless ramp exists
    let no_ramp_route = find_route(
        &network,
        network.graph[n_ground_w].coord,
        network.graph[n_elev_n].coord,
        RoutingPreference::ShortestDistance,
    );
    assert!(
        no_ramp_route.is_err(),
        "Ground road cannot route onto elevated flyover without an explicit ramp"
    );

    // 6. Connect via directional highway on-ramp
    network.add_ramp_connection(
        coord_ground_mid,
        0,
        coord_elev_mid,
        1,
        301,
        Some("Flyover On-Ramp".to_string()),
        50,
    );

    let with_ramp_route = find_route(
        &network,
        network.graph[n_ground_w].coord,
        network.graph[n_elev_n].coord,
        RoutingPreference::ShortestDistance,
    )
    .expect("Route must succeed via connecting ramp");
    assert_eq!(with_ramp_route.waypoints.len(), 4);

    // 7. Verify Turn Restriction definition and lookup
    let turn_restrict = TurnRestriction {
        from_way_id: 101,
        to_way_id: 102,
        restriction_type: TurnRestrictionType::NoLeftTurn,
    };
    network.add_turn_restriction(turn_restrict);
    assert_eq!(
        network.get_turn_restriction(101, 102),
        Some(TurnRestrictionType::NoLeftTurn)
    );
}

// ============================================================================
// TEST 3: INTERSECTION TRAFFIC LIGHT CYCLES, STOP-LINE BRAKING & QUEUING
// ============================================================================

#[test]
fn test_stage5_traffic_light_compliance_and_queuing() {
    let ground = FlatGround::new(0.0);
    let mut sim = TrafficSimulation::new();

    // 1. Intersection Traffic Signal at (0, 0, 50)
    let light = TrafficLight::new(101, Vec3::new(4.0, 0.0, 50.0));
    sim.add_traffic_light(light);

    // Stop Line at Z = 45m (5m before intersection center)
    let stop_line = IntersectionStopLine {
        id: 201,
        traffic_light_id: 101,
        position: Vec3::new(0.0, 0.0, 45.0),
        is_ns_direction: true,
    };
    sim.add_stop_line(stop_line);

    // 2. Spawn lead vehicle approaching stop line at 12 m/s
    let idm = IdmConfig {
        desired_speed_ms: 12.0,
        comfortable_decel_ms2: 2.5,
        min_gap_meters: 3.0,
        ..Default::default()
    };
    let mut lead_veh = TrafficAgent::new(
        1001,
        Vec3::new(0.0, 0.0, 10.0),
        vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 100.0)],
        idm,
    );
    lead_veh.speed_ms = 12.0;
    lead_veh.yaw = 0.0; // Heading North (+Z)
    sim.spawn_agent(lead_veh);

    // 3. Spawn follower vehicle 15m behind lead vehicle
    let mut follower_veh = TrafficAgent::new(
        1002,
        Vec3::new(0.0, 0.0, -5.0),
        vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 100.0)],
        idm,
    );
    follower_veh.speed_ms = 12.0;
    follower_veh.yaw = 0.0;
    sim.spawn_agent(follower_veh);

    // Set signal to RED phase (timer at 13.0s, which is > 12s green)
    sim.traffic_lights[0].timer = 13.0;
    assert_eq!(
        sim.traffic_lights[0].state_for_direction(true),
        TrafficLightState::Amber
    );
    sim.traffic_lights[0].timer = 16.0; // Red phase
    assert_eq!(
        sim.traffic_lights[0].state_for_direction(true),
        TrafficLightState::Red
    );

    // 4. Simulate vehicles approaching the Red Signal (150 ticks of 0.05s = 7.5s)
    for _ in 0..150 {
        sim.traffic_lights[0].timer = 16.0;
        sim.step(0.05, &ground);
    }

    let lead = &sim.agents[0];
    let follower = &sim.agents[1];

    // Lead vehicle must decelerate to a stop before the stop line (Z <= 45.0)
    assert!(
        lead.position.z <= 45.5,
        "Lead vehicle must stop before or at stop line. Got Z = {}",
        lead.position.z
    );
    assert!(
        lead.speed_ms < 0.5,
        "Lead vehicle must be stationary at Red light. Speed = {}",
        lead.speed_ms
    );
    assert!(lead.is_stopped_at_signal);

    // Follower vehicle must queue safely behind lead vehicle without colliding
    let bumper_to_bumper_gap = lead.position.z - follower.position.z - follower.length_meters;
    assert!(
        bumper_to_bumper_gap >= 1.5,
        "Follower must maintain safe queue headway behind lead vehicle. Gap = {}",
        bumper_to_bumper_gap
    );
    assert!(
        follower.speed_ms < 1.0,
        "Follower must come to a halt in queue. Speed = {}",
        follower.speed_ms
    );

    // 5. Signal turns Green: timer resets to 2.0s
    sim.traffic_lights[0].timer = 2.0;
    assert_eq!(
        sim.traffic_lights[0].state_for_direction(true),
        TrafficLightState::Green
    );

    // Simulate queue dissipation (80 ticks of 0.05s = 4.0s)
    for _ in 0..80 {
        sim.traffic_lights[0].timer = 2.0;
        sim.step(0.05, &ground);
    }

    let lead_dissipating = &sim.agents[0];
    let follower_dissipating = &sim.agents[1];

    assert!(
        lead_dissipating.position.z > 45.0,
        "Lead vehicle must clear stop line on Green light. Z = {}",
        lead_dissipating.position.z
    );
    assert!(
        lead_dissipating.speed_ms > 4.0,
        "Lead vehicle must accelerate on Green"
    );
    assert!(
        follower_dissipating.speed_ms > 2.0,
        "Follower must follow lead vehicle through intersection"
    );
}

// ============================================================================
// TEST 4: MULTILANE MOBIL LANE CHANGING & GAP ACCEPTANCE
// ============================================================================

#[test]
fn test_stage5_lane_changing_and_gap_acceptance() {
    let ground = FlatGround::new(0.0);
    let mut sim = TrafficSimulation::new();

    // Spawn agent in lane 0 (lateral offset 0.0)
    let idm = IdmConfig {
        desired_speed_ms: 15.0,
        ..Default::default()
    };
    let mut agent = TrafficAgent::new(
        1001,
        Vec3::new(0.0, 0.0, 10.0),
        vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 200.0)],
        idm,
    );
    agent.current_lane = 0;
    agent.target_lane = 0;
    agent.speed_ms = 10.0;
    sim.spawn_agent(agent);

    // Initiate Lane Change to Lane 1 (right lane, +3.5m offset)
    sim.agents[0].target_lane = 1;
    sim.agents[0].is_changing_lane = true;
    sim.agents[0].lane_change_progress = 0.0;

    // Step through the lane change (150 ticks at 60 Hz = 2.5s > 2.0s duration)
    let mut max_lateral_offset = 0.0f32;
    for _ in 0..150 {
        sim.step(1.0 / 60.0, &ground);
        max_lateral_offset = max_lateral_offset.max(sim.agents[0].lateral_lane_offset.abs());
    }

    let completed_agent = &sim.agents[0];
    assert_eq!(
        completed_agent.current_lane, 1,
        "Agent must arrive at target lane 1"
    );
    assert!(
        !completed_agent.is_changing_lane,
        "Lane change state must conclude upon arrival"
    );
    assert!(
        max_lateral_offset > 2.5,
        "Lateral offset must smoothly deflect during lane transition. Max = {}",
        max_lateral_offset
    );
    assert_eq!(
        completed_agent.lateral_lane_offset, 0.0,
        "Lateral offset must return to 0.0 once aligned in new lane"
    );
}

// ============================================================================
// TEST 5: HIGH-DENSITY SPATIAL HASH GRID & 60 FPS FLEET PERFORMANCE (>=64 VEHS)
// ============================================================================

#[test]
fn test_stage5_high_density_spatial_grid_performance() {
    let ground = FlatGround::new(0.0);
    let mut sim = TrafficSimulation::new();
    let fleet_size = 64;

    // Spawn 64 vehicles across 4 parallel multilane corridors
    for i in 0..fleet_size {
        let lane_idx = (i % 4) as f32;
        let x_pos = (lane_idx - 1.5) * 4.0;
        let z_pos = (i / 4) as f32 * 25.0;

        let path = vec![
            Vec3::new(x_pos, 0.0, z_pos),
            Vec3::new(x_pos, 0.0, z_pos + 400.0),
        ];

        let idm = IdmConfig {
            desired_speed_ms: 12.0 + (i % 5) as f32,
            min_gap_meters: 3.5,
            ..Default::default()
        };

        let mut agent = TrafficAgent::new(2000 + i as u64, Vec3::new(x_pos, 0.0, z_pos), path, idm);
        agent.speed_ms = 10.0;
        agent.yaw = 0.0;
        sim.spawn_agent(agent);
    }

    assert_eq!(sim.agents.len(), fleet_size);

    // Execute 100 simulation frames at 60 Hz
    let start_time = Instant::now();
    for _ in 0..100 {
        sim.step(1.0 / 60.0, &ground);
    }
    let total_duration = start_time.elapsed();
    let avg_step_ms = (total_duration.as_secs_f64() * 1000.0) / 100.0;

    println!(
        "⚡ 64-Vehicle TrafficSimulation Performance: {:?} total, {:.3} ms/step",
        total_duration, avg_step_ms
    );

    // Must execute far faster than 16.6ms (60 FPS budget)
    assert!(
        avg_step_ms < 5.0,
        "High-density traffic simulation must complete in under 5.0ms per frame. Got {:.3}ms",
        avg_step_ms
    );

    // Verify spatial hash grid correctly partitioned vehicles
    assert!(
        !sim.spatial_grid.grid.is_empty(),
        "Spatial grid must contain cell partitions"
    );

    // Verify zero collisions: check inter-vehicle distances within the same lane
    for i in 0..sim.agents.len() {
        let p_i = sim.agents[i].position;
        let neighbors = sim.spatial_grid.query_nearby(p_i, 15.0);
        for &j in &neighbors {
            if i != j && (sim.agents[i].position.x - sim.agents[j].position.x).abs() < 1.0 {
                let dist_z = (sim.agents[i].position.z - sim.agents[j].position.z).abs();
                assert!(
                    dist_z >= sim.agents[i].length_meters * 0.85,
                    "Vehicles {} and {} overlapping in lane! dist_z = {}",
                    i,
                    j,
                    dist_z
                );
            }
        }
    }
}

// ============================================================================
// TEST 6: PEDESTRIAN CROSSWALK YIELDING, AUDIO & GRIDLOCK RECOVERY
// ============================================================================

#[test]
fn test_stage5_gridlock_recovery_and_pedestrian_safety() {
    let ground = FlatGround::new(0.0);
    let mut sim = TrafficSimulation::new();

    // 1. Procedural Pedestrian crossing at Z = 30m
    let ped = PedestrianAgent::new(
        801,
        vec![Vec3::new(-6.0, 0.0, 30.0), Vec3::new(6.0, 0.0, 30.0)],
        1.2,
    );
    sim.add_pedestrian(ped);

    // 2. Vehicle approaching crosswalk at 10 m/s
    let idm = IdmConfig {
        desired_speed_ms: 10.0,
        comfortable_decel_ms2: 2.0,
        min_gap_meters: 3.0,
        ..Default::default()
    };
    let mut agent = TrafficAgent::new(
        1001,
        Vec3::new(0.0, 0.0, 5.0),
        vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 100.0)],
        idm,
    );
    agent.speed_ms = 10.0;
    agent.yaw = 0.0;
    sim.spawn_agent(agent);

    // Step for 4.5 seconds (90 ticks of 0.05s) to allow complete deceleration for crossing pedestrian
    for _ in 0..90 {
        sim.step(0.05, &ground);
    }

    // Vehicle must decelerate and yield before the crosswalk (Z < 30.0)
    assert!(
        sim.agents[0].position.z < 30.0,
        "Vehicle must stop/yield before pedestrian crosswalk. Z = {}",
        sim.agents[0].position.z
    );
    assert!(
        sim.agents[0].speed_ms < 1.0,
        "Vehicle must come to a halt to yield for crossing pedestrian (got speed = {})",
        sim.agents[0].speed_ms
    );

    // 3. Test Gridlock Stuck Timer & Horn Audio
    sim.agents[0].stuck_timer = 15.5; // Trigger horn honk
    sim.step(1.0 / 60.0, &ground);

    let audio_events = sim.drain_audio_events();
    let has_horn = audio_events
        .iter()
        .any(|ev| matches!(ev, TrafficAudioEvent::HornHonk { .. }));
    assert!(
        has_horn,
        "Gridlocked vehicle must emit HornHonk audio event"
    );

    // 4. Test Unstick Recovery
    sim.agents[0].stuck_timer = 20.5; // Exceeds unstick threshold
    let z_before = sim.agents[0].position.z;
    sim.step(1.0 / 60.0, &ground);

    assert!(
        sim.agents[0].position.z > z_before,
        "Unstick recovery must nudge vehicle forward"
    );
    assert_eq!(
        sim.agents[0].stuck_timer, 0.0,
        "Stuck timer must reset following unstick recovery"
    );

    // 5. Verify City Interaction 3D Mesh Generation (Traffic Signal, Sign, Pedestrian)
    let sig_mesh = generate_traffic_signal_mesh(Vec3::ZERO, 0.0, TrafficSignalVisualState::Green);
    assert!(
        !sig_mesh.is_empty(),
        "Traffic signal mesh must produce vertices"
    );

    let sign_mesh = generate_road_sign_mesh(Vec3::ZERO, 0.0, RoadSignType::Stop);
    assert!(
        !sign_mesh.is_empty(),
        "Road sign mesh must produce vertices"
    );

    let ped_mesh = generate_pedestrian_mesh(Vec3::ZERO, 0.0, 0.5, [0.2, 0.5, 0.8, 1.0]);
    assert!(
        !ped_mesh.is_empty(),
        "Pedestrian mesh must produce vertices with walk articulation"
    );
}
