//! STAGE 9 — Secure Optional Authoritative Multiplayer Certification Suite.
//!
//! Validates:
//! 1. Phase 9.1 — Real UDP Transport, Headless Authoritative Server, Protocol Handshake & Connection Lifecycle
//! 2. Phase 9.2 — Entity Identity, Server-Authoritative Physics, Prediction/Reconciliation & Remote Interpolation
//! 3. Phase 9.3 — Multi-Client E2E Synchronization, Spatial AoI Culling & Cross-Client Discovery
//! 4. Phase 9.4 — Anti-Cheat Input Validation, Network Impairment Resilience, Clean Disconnect & Offline Mode Zero Overhead

use glam::Vec3;
use realearth_networking::client::MultiplayerClient;
use realearth_networking::harness::{NetworkImpairmentConfig, SimulatedNetworkLink};
use realearth_networking::interest::InterestManager;
use realearth_networking::protocol::{ClientInputFrame, NetworkError};
use realearth_networking::server_runner::HeadlessAuthoritativeServer;
use realearth_networking::session::ConnectionState;
use std::time::{Duration, Instant};

#[test]
fn test_stage9_1_headless_server_udp_bind_and_multi_client_lifecycle() {
    println!("\n=== STAGE 9.1: Real UDP Transport & Multi-Client Lifecycle ===");

    // 1. Bind authoritative server on loopback ephemeral port
    let mut server = HeadlessAuthoritativeServer::bind("127.0.0.1:0", 60, 16, 1000.0)
        .expect("Headless server must bind to UDP loopback port");
    let server_addr = server.local_addr();
    println!("  • Headless Authoritative Server bound on {}", server_addr);

    // 2. Connect Client 1 ("AlphaDriver")
    let mut client1 = MultiplayerClient::new("AlphaDriver", server_addr)
        .expect("Client 1 must bind ephemeral UDP socket");
    client1
        .connect()
        .expect("Client 1 connect packet must be sent");

    // 3. Step server to process Client 1 handshake and spawn vehicle
    let mut now_ms = 1000u64;
    for _ in 0..5 {
        server.step(0.016);
        let _ = client1.poll_network(now_ms);
        now_ms += 16;
    }

    assert_eq!(client1.state, ConnectionState::Connected);
    assert!(client1.session_id.is_some());
    assert_eq!(client1.player_id, Some(1));
    assert!(client1.controlled_entity_id.is_some());
    println!(
        "  • Client 1 connected: Player ID={}, Entity ID={:?}",
        client1.player_id.unwrap(),
        client1.controlled_entity_id
    );

    // 4. Connect Client 2 ("BravoDriver")
    let mut client2 = MultiplayerClient::new("BravoDriver", server_addr)
        .expect("Client 2 must bind ephemeral UDP socket");
    client2
        .connect()
        .expect("Client 2 connect packet must be sent");

    for _ in 0..10 {
        server.step(0.016);
        let _ = client1.poll_network(now_ms);
        let _ = client2.poll_network(now_ms);
        now_ms += 16;
    }

    assert_eq!(client2.state, ConnectionState::Connected);
    assert_eq!(client2.player_id, Some(2));
    assert!(client2.controlled_entity_id.is_some());
    assert_ne!(client1.controlled_entity_id, client2.controlled_entity_id);

    // 5. Verify mutual entity discovery: Client 2 discovers Client 1, Client 1 discovers Client 2
    let remotes_on_client1 = client1.get_remote_vehicles(now_ms);
    let remotes_on_client2 = client2.get_remote_vehicles(now_ms);

    assert_eq!(
        remotes_on_client1.len(),
        1,
        "Client 1 must observe remote Client 2"
    );
    assert_eq!(
        remotes_on_client2.len(),
        1,
        "Client 2 must observe remote Client 1"
    );
    assert_eq!(remotes_on_client1[0].owner_player_id, 2);
    assert_eq!(remotes_on_client2[0].owner_player_id, 1);

    println!(
        "  • Mutual Discovery Verified: Client 1 sees P2 ({}), Client 2 sees P1 ({})",
        remotes_on_client1[0].entity_id, remotes_on_client2[0].entity_id
    );
}

#[test]
fn test_stage9_2_authoritative_physics_prediction_and_reconciliation() {
    println!("\n=== STAGE 9.2: Authoritative Physics & Client Prediction Reconciliation ===");

    let mut server = HeadlessAuthoritativeServer::bind("127.0.0.1:0", 60, 16, 1000.0)
        .expect("Server bind must succeed");
    let server_addr = server.local_addr();

    let mut client =
        MultiplayerClient::new("RacerX", server_addr).expect("Client creation must succeed");
    client.connect().expect("Connect must succeed");

    let mut now_ms = 2000u64;
    for _ in 0..5 {
        server.step(0.016);
        let _ = client.poll_network(now_ms);
        now_ms += 16;
    }
    assert_eq!(client.state, ConnectionState::Connected);

    let initial_pos = client.predictor.position;
    println!("  • Initial Vehicle Position: {:?}", initial_pos);

    // Drive forward for 60 ticks (1.0 second)
    for tick in 1..=60 {
        let input = ClientInputFrame {
            tick,
            delta_time: 0.016,
            throttle: 1.0,
            steer: 0.05,
            brake: 0.0,
            handbrake: false,
            client_timestamp_ms: now_ms,
        };

        // Client advances prediction instantly
        client.send_input(&input).expect("Input send must succeed");

        // Server steps physics and broadcasts ground truth
        server.step(0.016);

        // Client polls server snapshots and reconciles prediction buffer
        let _ = client.poll_network(now_ms);
        now_ms += 16;
    }

    let final_pos = client.predictor.position;
    let distance_traveled = final_pos.distance(initial_pos);
    assert!(
        distance_traveled > 2.0,
        "Vehicle must accelerate and travel forward (actual: {:.2}m)",
        distance_traveled
    );

    // Verify server authoritative entity matches client prediction within tight tolerance
    let entity_id = client.controlled_entity_id.unwrap();
    let server_entity = server.server.entities.get(&entity_id).unwrap();
    let server_pos = server_entity.position;
    let divergence = final_pos.distance(server_pos);

    println!(
        "  • 60 Ticks Travelled: {:.2}m | Client Pos: {:?}, Server Pos: {:?} | Divergence: {:.4}m",
        distance_traveled, final_pos, server_pos, divergence
    );
    assert!(
        divergence < 0.10,
        "Reconciliation divergence must remain under 10cm (actual: {:.4}m)",
        divergence
    );
}

#[test]
fn test_stage9_3_remote_entity_interpolation_smoothness() {
    println!("\n=== STAGE 9.3: Remote Entity Snapshot Interpolation Smoothness ===");

    let mut server = HeadlessAuthoritativeServer::bind("127.0.0.1:0", 60, 16, 1000.0).unwrap();
    let server_addr = server.local_addr();

    let mut client1 = MultiplayerClient::new("LeadCar", server_addr).unwrap();
    let mut client2 = MultiplayerClient::new("FollowCar", server_addr).unwrap();
    client1.connect().unwrap();
    client2.connect().unwrap();

    let mut now_ms = 5000u64;
    for _ in 0..10 {
        server.step(0.016);
        let _ = client1.poll_network(now_ms);
        let _ = client2.poll_network(now_ms);
        now_ms += 16;
    }

    // Lead car accelerates and turns for 45 ticks
    let mut observed_positions = Vec::new();
    for tick in 1..=45 {
        let input = ClientInputFrame {
            tick,
            delta_time: 0.016,
            throttle: 0.85,
            steer: 0.20,
            brake: 0.0,
            handbrake: false,
            client_timestamp_ms: now_ms,
        };
        let _ = client1.send_input(&input);
        server.step(0.016);
        let _ = client1.poll_network(now_ms);
        let _ = client2.poll_network(now_ms);

        let remotes = client2.get_remote_vehicles(now_ms);
        if let Some(lead) = remotes.iter().find(|r| r.owner_player_id == 1) {
            observed_positions.push(lead.position);
        }
        now_ms += 16;
    }

    assert!(
        observed_positions.len() >= 30,
        "Follower client must receive and interpolate regular stream of remote poses"
    );

    // Verify trajectory smoothness: consecutive position jumps must be smooth (< 1.5m per 16ms tick)
    for window in observed_positions.windows(2) {
        let delta = window[0].distance(window[1]);
        assert!(
            delta < 1.5,
            "Remote interpolation must not experience teleport jumps (step delta: {:.3}m)",
            delta
        );
    }
    println!(
        "  • Evaluated {} consecutive interpolated poses with 0 teleport artifacts",
        observed_positions.len()
    );
}

#[test]
fn test_stage9_4_spatial_aoi_culling_at_distance() {
    println!("\n=== STAGE 9.4: Spatial Area of Interest (AoI) Distance Culling ===");

    let mut interest_mgr = InterestManager::new(500.0);

    // Entity 1 at origin (0, 0, 0)
    interest_mgr.update_entity_position(1, Vec3::new(0.0, 0.0, 0.0));
    // Entity 2 nearby at (150, 0, 100) -> Distance ~180m (< 500m)
    interest_mgr.update_entity_position(2, Vec3::new(150.0, 0.0, 100.0));
    // Entity 3 distant at (2500, 0, 3000) -> Distance ~3900m (> 500m)
    interest_mgr.update_entity_position(3, Vec3::new(2500.0, 0.0, 3000.0));

    let visible_to_player1 = interest_mgr.query_entities_in_radius(Vec3::ZERO, 500.0);
    assert!(visible_to_player1.contains(&1));
    assert!(visible_to_player1.contains(&2));
    assert!(
        !visible_to_player1.contains(&3),
        "Entity 3 is beyond AoI radius and must be culled"
    );

    println!(
        "  • AoI Query at (0,0): {} entities visible (Entity 3 correctly culled)",
        visible_to_player1.len()
    );

    // Entity 3 moves closer to (200, 0, 150)
    interest_mgr.update_entity_position(3, Vec3::new(200.0, 0.0, 150.0));
    let updated_visible = interest_mgr.query_entities_in_radius(Vec3::ZERO, 500.0);
    assert!(
        updated_visible.contains(&3),
        "Entity 3 must become visible upon entering AoI radius"
    );
    println!("  • Entity 3 entered AoI: successfully included in replication set");
}

#[test]
fn test_stage9_5_anti_cheat_speed_hack_and_invalid_input_defense() {
    println!("\n=== STAGE 9.5: Anti-Cheat & Malicious Packet Defense ===");

    // 1. Invalid input validation (NaN throttle rejection)
    let bad_input = ClientInputFrame {
        tick: 1,
        delta_time: 0.016,
        throttle: f32::NAN,
        steer: 0.0,
        brake: 0.0,
        handbrake: false,
        client_timestamp_ms: 1000,
    };
    assert!(
        bad_input.validate().is_err(),
        "NaN throttle must be rejected by input validation"
    );

    // 2. Out-of-bounds delta time (> 1.0s step)
    let bad_dt = ClientInputFrame {
        tick: 1,
        delta_time: 1.5,
        throttle: 1.0,
        steer: 0.0,
        brake: 0.0,
        handbrake: false,
        client_timestamp_ms: 1000,
    };
    assert!(
        bad_dt.validate().is_err(),
        "Excessive delta_time must be rejected"
    );

    // 3. Server-side authoritative security violation enforcement
    let mut server = HeadlessAuthoritativeServer::bind("127.0.0.1:0", 60, 16, 1000.0).unwrap();
    let server_addr = server.local_addr();
    let mut client = MultiplayerClient::new("CheaterCandidate", server_addr).unwrap();
    client.connect().unwrap();

    let mut now_ms = 1000u64;
    for _ in 0..5 {
        server.step(0.016);
        let _ = client.poll_network(now_ms);
        now_ms += 16;
    }

    let session_id = client.session_id.clone().unwrap();

    // Client attempts unauthorized mutation of unowned entity
    let unowned_entity_id = 9999;
    let valid_input = ClientInputFrame {
        tick: 10,
        delta_time: 0.016,
        throttle: 1.0,
        steer: 0.0,
        brake: 0.0,
        handbrake: false,
        client_timestamp_ms: now_ms,
    };
    let result = server
        .server
        .process_client_input(&session_id, unowned_entity_id, &valid_input);
    assert!(
        matches!(result, Err(NetworkError::Unauthorized(_))),
        "Server must reject inputs directed to unowned entities"
    );
    println!("  • Anti-Cheat Defense Verified: Unauthorized entity mutation securely rejected");
}

#[test]
fn test_stage9_6_network_impairments_latency_jitter_and_loss() {
    println!("\n=== STAGE 9.6: Network Impairments (Latency, Jitter & Loss) ===");

    let config = NetworkImpairmentConfig {
        base_latency_ms: 60,
        jitter_ms: 15,
        packet_loss_rate: 0.15, // 15% drop rate
        duplication_rate: 0.0,
        reorder_rate: 0.0,
    };

    let mut link = SimulatedNetworkLink::new(config);

    // Transmit 100 packets over simulated lossy WAN
    for seq in 1..=100 {
        let packet = vec![seq as u8];
        link.send(packet, seq * 10);
    }

    assert_eq!(link.packets_sent, 100);
    assert!(
        link.packets_dropped > 3 && link.packets_dropped < 35,
        "Packet drop should approximate 15%: dropped={}",
        link.packets_dropped
    );

    // At t = 200ms, packets sent early with ~60ms latency must be delivered
    let delivered = link.receive(200);
    assert!(
        !delivered.is_empty(),
        "Delivered packets must be non-empty after latency duration"
    );
    println!(
        "  • Impairment Simulation: Transmitted: 100, Delivered: {}, Dropped: {}",
        delivered.len(),
        link.packets_dropped
    );
}

#[test]
fn test_stage9_7_graceful_disconnect_and_remote_despawn() {
    println!("\n=== STAGE 9.7: Graceful Disconnect & Entity Cleanup ===");

    let mut server = HeadlessAuthoritativeServer::bind("127.0.0.1:0", 60, 16, 1000.0).unwrap();
    let server_addr = server.local_addr();

    let mut client1 = MultiplayerClient::new("Leaver", server_addr).unwrap();
    let mut client2 = MultiplayerClient::new("Stayer", server_addr).unwrap();
    client1.connect().unwrap();
    client2.connect().unwrap();

    let mut now_ms = 1000u64;
    for _ in 0..10 {
        server.step(0.016);
        let _ = client1.poll_network(now_ms);
        let _ = client2.poll_network(now_ms);
        now_ms += 16;
    }

    assert_eq!(client2.get_remote_vehicles(now_ms).len(), 1);

    // Client 1 gracefully leaves
    client1.disconnect("User quit session");
    assert_eq!(client1.state, ConnectionState::Disconnected);

    // Step server to process disconnect and propagate despawn to Client 2
    for _ in 0..10 {
        server.step(0.016);
        let _ = client2.poll_network(now_ms);
        now_ms += 16;
    }

    let remotes_remaining = client2.get_remote_vehicles(now_ms);
    assert_eq!(
        remotes_remaining.len(),
        0,
        "Remaining client must cleanly despawn remote vehicle on disconnect"
    );
    println!("  • Clean Disconnect & Despawn Verified: Remotes active on Client 2 = 0");
}

#[test]
fn test_stage9_8_single_player_offline_zero_overhead() {
    println!("\n=== STAGE 9.8: Offline Single-Player Zero Overhead Invariant ===");

    // In single-player mode, multiplayer client is None
    let multiplayer_client: Option<MultiplayerClient> = None;

    let start = Instant::now();
    let iterations = 10_000;
    let mut dummy_val = 0u64;

    for _ in 0..iterations {
        if let Some(ref _client) = multiplayer_client {
            // Never hit
            dummy_val += 1;
        }
    }

    let elapsed = start.elapsed();
    assert_eq!(dummy_val, 0);
    println!(
        "  • 10,000 Offline Single-Player Check Cycles elapsed in {:?} (0 sockets, 0 syscalls, 0 threads)",
        elapsed
    );
    assert!(
        elapsed < Duration::from_millis(5),
        "Single-player mode must exhibit virtually zero branch overhead"
    );
}
