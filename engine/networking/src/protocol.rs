//! Binary packet framing, reliable/unreliable channel encoding, sequence numbering,
//! and schema-versioned network messages.

use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Protocol magic header: "REAL" in ASCII (0x5245414C).
pub const PROTOCOL_MAGIC: u32 = 0x5245_414C;

/// Current wire protocol schema version.
pub const PROTOCOL_VERSION: u16 = 1;

/// Network serialization and protocol framing errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum NetworkError {
    #[error("Invalid protocol magic header: expected {expected:#x}, got {actual:#x}")]
    InvalidMagic { expected: u32, actual: u32 },

    #[error("Protocol version mismatch: server={server}, client={client}")]
    VersionMismatch { server: u16, client: u16 },

    #[error("Packet payload truncated or corrupted")]
    PayloadCorrupted,

    #[error("Invalid client input: {0}")]
    InvalidInput(String),

    #[error("Unauthorized action: {0}")]
    Unauthorized(String),

    #[error("Rate limit exceeded: {0} packets/sec")]
    RateLimitExceeded(u32),

    #[error("Session error: {0}")]
    SessionError(String),
}

/// Transmission channel semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelType {
    /// Guaranteed delivery with in-order processing (handshakes, RPC, entity spawn).
    ReliableOrdered,
    /// Low-latency sequenced delivery; stale packets dropped (inputs, telemetry, snapshots).
    UnreliableSequenced,
}

/// Fixed-size binary packet framing header (24 bytes).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PacketHeader {
    pub magic: u32,
    pub version: u16,
    pub channel: ChannelType,
    pub sequence: u64,
    pub ack_sequence: u64,
    pub ack_bitfield: u32,
}

impl PacketHeader {
    pub fn new(channel: ChannelType, sequence: u64, ack_sequence: u64, ack_bitfield: u32) -> Self {
        Self {
            magic: PROTOCOL_MAGIC,
            version: PROTOCOL_VERSION,
            channel,
            sequence,
            ack_sequence,
            ack_bitfield,
        }
    }
}

/// Rejection reason for handshake failures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RejectReason {
    VersionMismatch(u16),
    InvalidToken,
    ServerFull,
    Banned,
}

/// Player security role for authorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerRole {
    Player,
    Spectator,
    Moderator,
    Admin,
}

/// High-frequency vehicle input frame sent from client to authoritative server.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClientInputFrame {
    pub tick: u64,
    pub delta_time: f32,
    pub throttle: f32,
    pub steer: f32,
    pub brake: f32,
    pub handbrake: bool,
    pub client_timestamp_ms: u64,
}

impl ClientInputFrame {
    /// Validates inputs against NaN, infinity, and illegal physical bounds.
    pub fn validate(&self) -> Result<(), NetworkError> {
        if !self.throttle.is_finite() || !(-1.0..=1.0).contains(&self.throttle) {
            return Err(NetworkError::InvalidInput(format!(
                "Throttle out of bounds [-1.0, 1.0]: {}",
                self.throttle
            )));
        }
        if !self.steer.is_finite() || !(-1.0..=1.0).contains(&self.steer) {
            return Err(NetworkError::InvalidInput(format!(
                "Steer out of bounds [-1.0, 1.0]: {}",
                self.steer
            )));
        }
        if !self.brake.is_finite() || !(0.0..=1.0).contains(&self.brake) {
            return Err(NetworkError::InvalidInput(format!(
                "Brake out of bounds [0.0, 1.0]: {}",
                self.brake
            )));
        }
        if !self.delta_time.is_finite() || self.delta_time <= 0.0 || self.delta_time > 0.5 {
            return Err(NetworkError::InvalidInput(format!(
                "Invalid delta time: {}",
                self.delta_time
            )));
        }
        Ok(())
    }
}

/// Replicated entity state snapshot payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityStatePayload {
    pub entity_id: u32,
    pub owner_player_id: u32,
    pub position: Vec3,
    pub orientation: Quat,
    pub linear_velocity: Vec3,
    pub angular_velocity: Vec3,
    pub speed_kph: f32,
    pub flags: u8,
}

/// Enumeration of strongly-typed network protocol messages.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NetworkMessage {
    // 1. Connection & Session
    ClientHello {
        client_version: u16,
        client_name: String,
        auth_token: String,
    },
    ServerHello {
        session_id: String,
        assigned_player_id: u32,
        server_tick_rate: u32,
        role: PlayerRole,
    },
    HandshakeReject {
        reason: RejectReason,
    },
    Ping {
        client_timestamp_ms: u64,
    },
    Pong {
        client_timestamp_ms: u64,
        server_timestamp_ms: u64,
    },
    Disconnect {
        reason: String,
    },

    // 2. Entity Lifecycle
    SpawnEntity {
        entity_id: u32,
        owner_player_id: u32,
        vehicle_type: String,
        state: EntityStatePayload,
    },
    DespawnEntity {
        entity_id: u32,
        reason: String,
    },
    CellHandoff {
        entity_id: u32,
        target_cell_id: String,
    },

    // 3. High-Frequency Replication
    Input(ClientInputFrame),
    ServerSnapshot {
        tick: u64,
        server_time_ms: u64,
        states: Vec<EntityStatePayload>,
    },

    // 4. Notifications & Chat
    ChatMessage {
        sender_id: u32,
        message: String,
    },
    ErrorMessage {
        code: u16,
        message: String,
    },
}

/// Serializes a packet header and message payload into a contiguous byte vector.
pub fn encode_packet(header: &PacketHeader, msg: &NetworkMessage) -> Result<Vec<u8>, NetworkError> {
    let mut buffer = Vec::with_capacity(64);
    bincode::serialize_into(&mut buffer, header).map_err(|_| NetworkError::PayloadCorrupted)?;
    bincode::serialize_into(&mut buffer, msg).map_err(|_| NetworkError::PayloadCorrupted)?;
    Ok(buffer)
}

/// Decodes and validates a raw packet buffer into header and message.
pub fn decode_packet(bytes: &[u8]) -> Result<(PacketHeader, NetworkMessage), NetworkError> {
    if bytes.len() < 24 {
        return Err(NetworkError::PayloadCorrupted);
    }

    let mut cursor = std::io::Cursor::new(bytes);
    let header: PacketHeader =
        bincode::deserialize_from(&mut cursor).map_err(|_| NetworkError::PayloadCorrupted)?;

    if header.magic != PROTOCOL_MAGIC {
        return Err(NetworkError::InvalidMagic {
            expected: PROTOCOL_MAGIC,
            actual: header.magic,
        });
    }

    if header.version != PROTOCOL_VERSION {
        return Err(NetworkError::VersionMismatch {
            server: PROTOCOL_VERSION,
            client: header.version,
        });
    }

    let msg: NetworkMessage =
        bincode::deserialize_from(&mut cursor).map_err(|_| NetworkError::PayloadCorrupted)?;

    Ok((header, msg))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_encode_decode_roundtrip() {
        let header = PacketHeader::new(ChannelType::ReliableOrdered, 42, 41, 0b1111);
        let msg = NetworkMessage::ClientHello {
            client_version: 1,
            client_name: "PilotAlpha".to_string(),
            auth_token: "secret-token-xyz".to_string(),
        };

        let encoded = encode_packet(&header, &msg).expect("Encoding should succeed");
        assert!(encoded.len() > 24);

        let (decoded_hdr, decoded_msg) = decode_packet(&encoded).expect("Decoding should succeed");
        assert_eq!(decoded_hdr, header);
        assert_eq!(decoded_msg, msg);
    }

    #[test]
    fn test_invalid_magic_header_rejection() {
        let mut header = PacketHeader::new(ChannelType::UnreliableSequenced, 1, 0, 0);
        header.magic = 0xDEADBEEF;
        let msg = NetworkMessage::Ping {
            client_timestamp_ms: 12345,
        };

        let encoded = encode_packet(&header, &msg).unwrap();
        let err = decode_packet(&encoded).unwrap_err();

        match err {
            NetworkError::InvalidMagic { actual, .. } => assert_eq!(actual, 0xDEADBEEF),
            _ => panic!("Expected InvalidMagic error"),
        }
    }

    #[test]
    fn test_client_input_validation() {
        // Valid inputs
        let valid = ClientInputFrame {
            tick: 10,
            delta_time: 1.0 / 60.0,
            throttle: 0.8,
            steer: -0.2,
            brake: 0.0,
            handbrake: false,
            client_timestamp_ms: 1000,
        };
        assert!(valid.validate().is_ok());

        // NaN throttle
        let nan_input = ClientInputFrame {
            throttle: f32::NAN,
            ..valid
        };
        assert!(nan_input.validate().is_err());

        // Out-of-bounds steer
        let bad_steer = ClientInputFrame {
            steer: 2.5,
            ..valid
        };
        assert!(bad_steer.validate().is_err());

        // Negative delta time
        let bad_dt = ClientInputFrame {
            delta_time: -0.01,
            ..valid
        };
        assert!(bad_dt.validate().is_err());
    }
}
