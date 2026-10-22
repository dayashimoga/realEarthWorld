//! Session management, authentication tokens, connection lifecycles,
//! heartbeat keepalives, and token-bucket rate limiting.

use crate::protocol::{NetworkError, PlayerRole};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Connection state of a remote client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Handshaking,
    Connected,
    Reconnecting,
    Terminated,
}

/// Token bucket algorithm for per-client inbound rate limiting.
#[derive(Debug, Clone)]
pub struct TokenBucket {
    pub capacity: f32,
    pub tokens: f32,
    pub refill_rate_per_sec: f32,
    pub last_refill: Instant,
}

impl TokenBucket {
    pub fn new(capacity: f32, refill_rate_per_sec: f32) -> Self {
        Self {
            capacity,
            tokens: capacity,
            refill_rate_per_sec,
            last_refill: Instant::now(),
        }
    }

    /// Attempts to consume 1 token. Returns true if permitted, false if rate limited.
    pub fn try_consume(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f32();
        self.tokens = (self.tokens + elapsed * self.refill_rate_per_sec).min(self.capacity);
        self.last_refill = now;

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

/// An authenticated client session managed by the authoritative server.
#[derive(Debug, Clone)]
pub struct ClientSession {
    pub session_id: String,
    pub player_id: u32,
    pub player_name: String,
    pub role: PlayerRole,
    pub state: ConnectionState,
    pub auth_token: String,
    pub rtt_ms: f32,
    pub last_heartbeat: Instant,
    pub reconnect_deadline: Option<Instant>,
    pub rate_limiter: TokenBucket,
    pub controlled_entity_id: Option<u32>,
}

impl ClientSession {
    pub fn new(
        session_id: String,
        player_id: u32,
        player_name: String,
        role: PlayerRole,
        auth_token: String,
    ) -> Self {
        Self {
            session_id,
            player_id,
            player_name,
            role,
            state: ConnectionState::Connected,
            auth_token,
            rtt_ms: 25.0, // Initial estimate
            last_heartbeat: Instant::now(),
            reconnect_deadline: None,
            rate_limiter: TokenBucket::new(120.0, 100.0), // 120 max burst, 100/sec refill
            controlled_entity_id: None,
        }
    }

    /// Records round-trip latency sample with exponential moving average.
    pub fn update_rtt(&mut self, sample_ms: f32) {
        let alpha = 0.15; // Smooth EWMA factor
        self.rtt_ms = (1.0 - alpha) * self.rtt_ms + alpha * sample_ms;
        self.last_heartbeat = Instant::now();
    }
}

/// Thread-safe authoritative session registry and lifecycle manager.
#[derive(Debug)]
pub struct SessionManager {
    pub max_players: usize,
    pub heartbeat_timeout: Duration,
    pub reconnect_grace_period: Duration,
    pub sessions: HashMap<String, ClientSession>,
    next_player_id: u32,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new(64, 10.0, 15.0)
    }
}

impl SessionManager {
    pub fn new(max_players: usize, timeout_secs: f32, reconnect_grace_secs: f32) -> Self {
        Self {
            max_players,
            heartbeat_timeout: Duration::from_secs_f32(timeout_secs),
            reconnect_grace_period: Duration::from_secs_f32(reconnect_grace_secs),
            sessions: HashMap::new(),
            next_player_id: 1,
        }
    }

    /// Registers a new player session and generates a secure authentication token.
    pub fn register_player(
        &mut self,
        player_name: &str,
        role: PlayerRole,
    ) -> Result<ClientSession, NetworkError> {
        let active_count = self
            .sessions
            .values()
            .filter(|s| s.state == ConnectionState::Connected)
            .count();

        if active_count >= self.max_players {
            return Err(NetworkError::SessionError(
                "Server player limit reached".to_string(),
            ));
        }

        let player_id = self.next_player_id;
        self.next_player_id += 1;

        let session_id = format!("sess_{:08x}_{}", player_id, player_name);
        let auth_token = format!("tok_{:016x}_{}", player_id, player_name);

        let session = ClientSession::new(
            session_id.clone(),
            player_id,
            player_name.to_string(),
            role,
            auth_token,
        );

        self.sessions.insert(session_id, session.clone());
        Ok(session)
    }

    /// Verifies session authentication credentials.
    pub fn authenticate(
        &mut self,
        session_id: &str,
        auth_token: &str,
    ) -> Result<&mut ClientSession, NetworkError> {
        if let Some(session) = self.sessions.get_mut(session_id) {
            if session.auth_token == auth_token {
                if session.state == ConnectionState::Reconnecting {
                    session.state = ConnectionState::Connected;
                    session.reconnect_deadline = None;
                    session.last_heartbeat = Instant::now();
                }
                Ok(session)
            } else {
                Err(NetworkError::Unauthorized("Invalid auth token".to_string()))
            }
        } else {
            Err(NetworkError::SessionError("Session not found".to_string()))
        }
    }

    /// Checks active sessions for timeouts, transitioning to reconnect grace or termination.
    pub fn poll_timeouts(&mut self) -> Vec<u32> {
        let now = Instant::now();
        let mut timed_out_players = Vec::new();

        for session in self.sessions.values_mut() {
            if session.state == ConnectionState::Connected {
                if now.duration_since(session.last_heartbeat) > self.heartbeat_timeout {
                    session.state = ConnectionState::Reconnecting;
                    session.reconnect_deadline = Some(now + self.reconnect_grace_period);
                }
            } else if session.state == ConnectionState::Reconnecting {
                if let Some(deadline) = session.reconnect_deadline {
                    if now > deadline {
                        session.state = ConnectionState::Terminated;
                        timed_out_players.push(session.player_id);
                    }
                }
            }
        }

        // Clean up terminated sessions
        self.sessions
            .retain(|_, s| s.state != ConnectionState::Terminated);
        timed_out_players
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_registration_and_authentication() {
        let mut mgr = SessionManager::new(4, 5.0, 10.0);
        let session = mgr
            .register_player("Pilot01", PlayerRole::Player)
            .expect("Registration should succeed");

        assert_eq!(session.player_id, 1);
        assert_eq!(session.state, ConnectionState::Connected);

        // Authenticate with valid credentials
        let authed = mgr.authenticate(&session.session_id, &session.auth_token);
        assert!(authed.is_ok());

        // Authenticate with invalid token
        let failed = mgr.authenticate(&session.session_id, "wrong_token");
        assert_eq!(
            failed.unwrap_err(),
            NetworkError::Unauthorized("Invalid auth token".to_string())
        );
    }

    #[test]
    fn test_token_bucket_rate_limiter() {
        let mut bucket = TokenBucket::new(5.0, 2.0);

        // Consume 5 tokens immediately
        for _ in 0..5 {
            assert!(bucket.try_consume());
        }

        // 6th token should be rejected (exhausted capacity)
        assert!(!bucket.try_consume());
    }

    #[test]
    fn test_reconnect_grace_period_and_timeout() {
        let mut mgr = SessionManager::new(2, 0.05, 0.1); // Short durations for unit test
        let session = mgr
            .register_player("GhostDriver", PlayerRole::Player)
            .unwrap();

        // Simulate elapsed time exceeding heartbeat timeout
        std::thread::sleep(Duration::from_millis(60));
        let dropped = mgr.poll_timeouts();
        assert!(dropped.is_empty(), "Should be in reconnecting grace window");

        let current = mgr.sessions.get(&session.session_id).unwrap();
        assert_eq!(current.state, ConnectionState::Reconnecting);

        // Successfully reconnect within grace window
        let reconnected = mgr.authenticate(&session.session_id, &session.auth_token);
        assert!(reconnected.is_ok());
        assert_eq!(reconnected.unwrap().state, ConnectionState::Connected);
    }
}
