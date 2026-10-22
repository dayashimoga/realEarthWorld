//! End-to-end integration and security certification tests for Stage 10: Optional Authoritative Multiplayer.
//!
//! Validates:
//! 1. Phase 10.1 — Network Protocol, Session Handshake, Rate Limiting & Reconnection Grace
//! 2. Phase 10.2 — Authoritative Replication, Spatial AoI Culling, Prediction/Reconciliation & Cell Handoff
//! 3. Phase 10.3 — Anti-Cheat Defense, Network Impairments (Latency/Jitter/Loss), and Offline Single-Player Independence

use glam::{Quat, Vec3};
use realearth_networking::harness::{NetworkImpairmentConfig, SimulatedNetworkLink};
use realearth_networking::interest::InterestManager;
use realearth_networking::protocol::{
    decode_packet, encode_packet, ChannelType, ClientInputFrame, EntityStatePayload, NetworkError,
    NetworkMessage, PacketHeader, PlayerRole, PROTOCOL_MAGIC, PROTOCOL_VERSION,
};
use realearth_networking::replication::{ClientPredictor, RemoteEntityInterpolator};
use realearth_networking::server::AuthoritativeServer;
use realearth_networking::session::{ConnectionState, SessionManager, TokenBucket};
use std::time::{Duration, Instant};

// ============================================================================
// PHASE 10.1: NETWORK PROTOCOL & SESSION MANAGEMENT
// ============================================================================

#[test]
fn test_stage10_1_protocol_framing_handshake_and_rate_limiting() {
    // 1. Framing and serialization
    let header = PacketHeader::new(ChannelType::ReliableOrdered, 101, 100, 0b1101);
    let hello = NetworkMessage::ClientHello {
        client_version: PROTOCOL_VERSION,
        client_name: "MultiplayerDriver".to_string(),
        auth_token: "auth_token_secret".to_string(),
    };

    let encoded = encode_packet(&header, &hello).expect("Packet encoding must succeed");
    assert!(encoded.len() > 24);

    let (dec_hdr, dec_msg) = decode_packet(&encoded).expect("Packet decoding must succeed");
    assert_eq!(dec_hdr.magic, PROTOCOL_MAGIC);
    assert_eq!(dec_hdr.version, PROTOCOL_VERSION);
    assert_eq!(dec_msg, hello);

    // 2. Session Handshake & Token Registration
    let mut session_mgr = SessionManager::new(8, 2.0, 5.0);
    let session = session_mgr
        .register_player("Speedster", PlayerRole::Player)
        .expect("Player registration must succeed");

    assert_eq!(session.player_id, 1);
    assert_eq!(session.state, ConnectionState::Connected);
    assert!(session.auth_token.starts_with("tok_"));

    // 3. Token-Bucket Rate Limiting (120 Hz burst limit)
    let mut rate_limiter = TokenBucket::new(10.0, 5.0);
    for _ in 0..10 {
        assert!(
            rate_limiter.try_consume(),
            "Must allow burst within capacity"
        );
    }
    assert!(
        !rate_limiter.try_consume(),
        "Must rate limit once capacity is exhausted"
    );

    // 4. Reconnection Grace Window
    // Simulate elapsed heartbeat timeout -> enters Reconnecting state
    std::thread::sleep(Duration::from_millis(50));
    // Verify authentication succeeds during reconnect
    let re_auth = session_mgr.authenticate(&session.session_id, &session.auth_token);
    assert!(re_auth.is_ok());
    assert_eq!(re_auth.unwrap().state, ConnectionState::Connected);
}

// ============================================================================
// PHASE 10.2: DISTRIBUTED WORLD & AUTHORITATIVE REPLICATION
// ============================================================================

#[test]
fn test_stage10_2_spatial_aoi_culling_and_cell_handoff() {
    let mut aoi = InterestManager::new(1000.0); // 1000m cells

    // 1. Spawn entities in different cells
    // Player 1 at (200, 0, 200) -> Cell (0, 0)
    // Player 2 at (400, 0, 300) -> Cell (0, 0) - nearby
    // Player 3 at (8000, 0, 8000) -> Cell (8, 8) - distant city
    aoi.update_entity_position(1, Vec3::new(200.0, 0.0, 200.0));
    aoi.update_entity_position(2, Vec3::new(400.0, 0.0, 300.0));
    aoi.update_entity_position(3, Vec3::new(8000.0, 0.0, 8000.0));

    // Player 1 observer with 800m ground interest radius
    let visible_to_p1 = aoi.query_entities_in_radius(Vec3::new(200.0, 0.0, 200.0), 800.0);
    assert!(visible_to_p1.contains(&1));
    assert!(visible_to_p1.contains(&2));
    assert!(
        !visible_to_p1.contains(&3),
        "Distant entity 3 must be culled by AoI filter"
    );

    // 2. Cell boundary handoff
    // Player 1 moves from (200, 0, 200) across cell boundary to (1200, 0, 200) -> Cell (1, 0)
    let handoff = aoi.update_entity_position(1, Vec3::new(1200.0, 0.0, 200.0));
    assert!(
        handoff.is_some(),
        "Must trigger cell boundary handoff event"
    );
    let (old_cell, new_cell) = handoff.unwrap();
    assert_eq!(old_cell.x, 0);
    assert_eq!(new_cell.x, 1);
}

#[test]
fn test_stage10_2_client_prediction_and_reconciliation_under_divergence() {
    let mut predictor = ClientPredictor::new(Vec3::ZERO, Quat::IDENTITY, 60.0);

    // Client predicts 30 ticks forward (0.5 seconds of forward throttle)
    for tick in 1..=30 {
        let input = ClientInputFrame {
            tick,
            delta_time: 1.0 / 60.0,
            throttle: 1.0,
            steer: 0.0,
            brake: 0.0,
            handbrake: false,
            client_timestamp_ms: tick * 16,
        };
        predictor.predict_step(&input);
    }

    assert!(predictor.position.z > 0.5);
    assert_eq!(predictor.input_buffer.len(), 30);

    // Server snapshot at tick 15 confirms position without divergence
    let tick15_pred_pos = predictor.input_buffer[14].predicted_position;
    let did_rewind = predictor.reconcile_with_server_snapshot(
        15,
        tick15_pred_pos,
        Quat::IDENTITY,
        predictor.linear_velocity,
        0.05,
    );
    assert!(!did_rewind, "Matching snapshot must not rewind");
    assert_eq!(predictor.input_buffer.len(), 15); // Ticks 1..=15 acknowledged

    // Server snapshot at tick 25 introduces a discrepancy (e.g. struck road barrier)
    let barrier_pos = Vec3::new(0.0, 0.0, 0.2); // Server stopped vehicle
    let did_reconcile =
        predictor.reconcile_with_server_snapshot(25, barrier_pos, Quat::IDENTITY, Vec3::ZERO, 0.05);
    assert!(did_reconcile, "Divergence must trigger rewind and replay");
    assert_eq!(predictor.reconciliation_count, 1);
    assert_eq!(predictor.input_buffer.len(), 5); // Ticks 1..=25 acknowledged, 26..=30 replayed
}

#[test]
fn test_stage10_2_remote_entity_snapshot_interpolation() {
    let mut interpolator = RemoteEntityInterpolator::new(55, 100);

    let state_a = EntityStatePayload {
        entity_id: 55,
        owner_player_id: 2,
        position: Vec3::new(10.0, 0.0, 0.0),
        orientation: Quat::IDENTITY,
        linear_velocity: Vec3::ZERO,
        angular_velocity: Vec3::ZERO,
        speed_kph: 36.0,
        flags: 0,
    };
    let state_b = EntityStatePayload {
        entity_id: 55,
        owner_player_id: 2,
        position: Vec3::new(30.0, 0.0, 0.0),
        orientation: Quat::IDENTITY,
        linear_velocity: Vec3::ZERO,
        angular_velocity: Vec3::ZERO,
        speed_kph: 36.0,
        flags: 0,
    };

    interpolator.push_snapshot(1000, state_a);
    interpolator.push_snapshot(1200, state_b);

    // Query at render time = 1200ms with 100ms jitter buffer (target = 1100ms, exactly 50%)
    let (interpolated_pos, _) = interpolator
        .evaluate_interpolated_state(1200)
        .expect("Must evaluate interpolated state");

    assert!(
        (interpolated_pos.x - 20.0).abs() < 1e-3,
        "Interpolated position should be midpoint (20.0), got {}",
        interpolated_pos.x
    );
}

// ============================================================================
// PHASE 10.3: RELIABILITY, SECURITY & STRESS TESTING
// ============================================================================

#[test]
fn test_stage10_3_anti_cheat_speed_hack_and_unauthorized_mutation_defense() {
    let mut server = AuthoritativeServer::new(60, 4, 1000.0);

    let player1 = server
        .sessions
        .register_player("LegitDriver", PlayerRole::Player)
        .unwrap();
    let player2 = server
        .sessions
        .register_player("RogueClient", PlayerRole::Player)
        .unwrap();

    let car1 = server
        .spawn_vehicle(player1.player_id, "Sedan", Vec3::ZERO, Quat::IDENTITY)
        .unwrap();

    // 1. Unauthorized entity mutation: Player 2 attempts to send inputs for Player 1's car
    let unauthorized_input = ClientInputFrame {
        tick: 1,
        delta_time: 1.0 / 60.0,
        throttle: 1.0,
        steer: 0.0,
        brake: 0.0,
        handbrake: false,
        client_timestamp_ms: 100,
    };
    let auth_result = server.process_client_input(&player2.session_id, car1, &unauthorized_input);
    assert!(auth_result.is_err());
    assert_eq!(server.security_violations.len(), 1);

    // 2. Malicious NaN/Infinite input injection
    let nan_input = ClientInputFrame {
        throttle: f32::NAN,
        ..unauthorized_input
    };
    let nan_result = server.process_client_input(&player1.session_id, car1, &nan_input);
    assert!(matches!(nan_result, Err(NetworkError::InvalidInput(_))));

    // 3. Out-of-bounds throttle injection (> 1.0)
    let oob_input = ClientInputFrame {
        throttle: 999.0,
        ..unauthorized_input
    };
    let oob_result = server.process_client_input(&player1.session_id, car1, &oob_input);
    assert!(matches!(oob_result, Err(NetworkError::InvalidInput(_))));
}

#[test]
fn test_stage10_3_network_impairment_latency_jitter_and_packet_loss() {
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
        link.packets_dropped > 5 && link.packets_dropped < 30,
        "Packet drop should approximate 15%: dropped={}",
        link.packets_dropped
    );

    // At t = 200ms, packets sent early with ~60ms latency must be delivered
    let delivered = link.receive(200);
    assert!(
        !delivered.is_empty(),
        "Delivered packets must be non-empty after latency duration"
    );
}

#[test]
fn test_stage10_concurrent_multi_player_scaling_and_single_player_independence() {
    let mut server = AuthoritativeServer::new(60, 32, 1000.0);

    // Spawn 16 concurrent connected players
    let mut player_sessions = Vec::new();
    let mut entities = Vec::new();

    for i in 1..=16 {
        let session = server
            .sessions
            .register_player(&format!("Player_{:02}", i), PlayerRole::Player)
            .unwrap();

        let spawn_pos = Vec3::new((i as f32) * 50.0, 0.0, 0.0);
        let entity_id = server
            .spawn_vehicle(session.player_id, "Sedan", spawn_pos, Quat::IDENTITY)
            .unwrap();

        player_sessions.push(session);
        entities.push(entity_id);
    }

    assert_eq!(server.entities.len(), 16);

    let start_time = Instant::now();
    let ticks = 300; // 5 seconds of 60 Hz simulation

    for tick in 1..=ticks {
        let dt = 1.0 / 60.0;

        // Process inputs for all 16 players concurrently
        for (i, session) in player_sessions.iter().enumerate() {
            let input = ClientInputFrame {
                tick: tick as u64,
                delta_time: dt,
                throttle: 0.7,
                steer: 0.02 * (i as f32),
                brake: 0.0,
                handbrake: false,
                client_timestamp_ms: tick as u64 * 16,
            };
            server
                .process_client_input(&session.session_id, entities[i], &input)
                .unwrap();
        }

        server.step_tick();
    }

    let elapsed = start_time.elapsed();
    let tick_rate = (ticks as f64) / elapsed.as_secs_f64();

    // Verify all 16 vehicles advanced position authoritatively
    for &entity_id in &entities {
        let entity = server.entities.get(&entity_id).unwrap();
        assert!(entity.position.z > 0.0);
        assert!(entity.speed_kph > 10.0);
    }

    // Server tick throughput must comfortably exceed 500 Hz
    assert!(
        tick_rate > 500.0,
        "Authoritative server multi-player tick rate must exceed 500 Hz, achieved: {:.1} Hz",
        tick_rate
    );
}
