//! Optional authoritative multiplayer networking, Area of Interest replication,
//! client-side prediction, and security validation for the Real Earth platform.
//!
//! Offline single-player mode operates completely independently with zero runtime overhead.

pub mod client;
pub mod harness;
pub mod interest;
pub mod protocol;
pub mod replication;
pub mod server;
pub mod server_runner;
pub mod session;
pub mod transport;

pub use client::{MultiplayerClient, MultiplayerStatus, RemoteVehicleRenderState};
pub use harness::{NetworkImpairmentConfig, SimulatedNetworkLink};
pub use interest::{InterestManager, SpatialCellId};
pub use protocol::{
    decode_packet, encode_packet, ChannelType, ClientInputFrame, EntityStatePayload, NetworkError,
    NetworkMessage, PacketHeader, PlayerRole, RejectReason, PROTOCOL_MAGIC, PROTOCOL_VERSION,
};
pub use replication::{BufferedInput, ClientPredictor, RemoteEntityInterpolator, TimedSnapshot};
pub use server::{AuthoritativeServer, ServerEntity};
pub use server_runner::{HeadlessAuthoritativeServer, ServerStepSummary};
pub use session::{ClientSession, ConnectionState, SessionManager, TokenBucket};
pub use transport::{UdpSocketTransport, MAX_UDP_PACKET_SIZE};

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Quat, Vec3};

    #[test]
    fn test_multiplayer_end_to_end_local_session() {
        let mut server = AuthoritativeServer::new(60, 4, 1000.0);

        // 1. Client connection and authentication
        let session = server
            .sessions
            .register_player("AlphaDriver", PlayerRole::Player)
            .expect("Session creation must succeed");

        let authed = server
            .sessions
            .authenticate(&session.session_id, &session.auth_token);
        assert!(authed.is_ok());

        // 2. Authoritative vehicle spawn
        let entity_id = server
            .spawn_vehicle(session.player_id, "CitySedan", Vec3::ZERO, Quat::IDENTITY)
            .expect("Vehicle spawn must succeed");

        assert_eq!(server.entities.len(), 1);

        // 3. Client-side prediction setup
        let mut predictor = ClientPredictor::new(Vec3::ZERO, Quat::IDENTITY, 50.0);

        // 4. Drive vehicle: client predicts locally while sending inputs to server
        for tick in 1..=60 {
            let dt = 1.0 / 60.0;
            let input = ClientInputFrame {
                tick,
                delta_time: dt,
                throttle: 1.0,
                steer: 0.0,
                brake: 0.0,
                handbrake: false,
                client_timestamp_ms: tick * 16,
            };

            // Client predicts immediately
            predictor.predict_step(&input);

            // Server receives and simulates authoritatively
            server
                .process_client_input(&session.session_id, entity_id, &input)
                .expect("Valid input must be processed");

            server.step_tick();
        }

        // Server and client positions must match closely
        let server_entity = server.entities.get(&entity_id).unwrap();
        assert!(
            (server_entity.position - predictor.position).length() < 0.1,
            "Authoritative server position and client prediction must converge within 0.1m"
        );
        assert!(server_entity.speed_kph > 15.0);

        // 5. Area of Interest tailored snapshot emission
        let snapshot_msg = server
            .build_snapshot_for_player(session.player_id, 800.0)
            .expect("Snapshot must be generated");

        match snapshot_msg {
            NetworkMessage::ServerSnapshot { tick, states, .. } => {
                assert_eq!(tick, 60);
                assert_eq!(states.len(), 1);
                assert_eq!(states[0].entity_id, entity_id);
            }
            _ => panic!("Expected ServerSnapshot variant"),
        }
    }
}
