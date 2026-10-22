//! Production UDP Socket Transport for Real Earth authoritative multiplayer.
//!
//! Provides non-blocking packet send/receive operations with MTU sizing,
//! packet framing, and peer address management over real OS UDP sockets.

use crate::protocol::{
    decode_packet, encode_packet, ChannelType, NetworkError, NetworkMessage, PacketHeader,
};
use std::io::ErrorKind;
use std::net::{SocketAddr, UdpSocket};

/// Standard safe Ethernet MTU payload limit for gaming packets.
pub const MAX_UDP_PACKET_SIZE: usize = 1400;

/// Production UDP Socket Transport with non-blocking I/O.
#[derive(Debug)]
pub struct UdpSocketTransport {
    socket: UdpSocket,
    local_addr: SocketAddr,
    recv_buf: Vec<u8>,
    next_sequence: u64,
}

impl UdpSocketTransport {
    /// Binds a new UDP transport to the specified local address.
    pub fn bind(addr: &str) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;
        let local_addr = socket.local_addr()?;
        Ok(Self {
            socket,
            local_addr,
            recv_buf: vec![0u8; MAX_UDP_PACKET_SIZE],
            next_sequence: 1,
        })
    }

    /// Returns the local bound socket address.
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Sends an encoded NetworkMessage to the destination address.
    pub fn send_message(
        &mut self,
        dest: SocketAddr,
        channel: ChannelType,
        msg: &NetworkMessage,
    ) -> Result<usize, NetworkError> {
        let seq = self.next_sequence;
        self.next_sequence += 1;

        let header = PacketHeader::new(channel, seq, 0, 0);
        let packet = encode_packet(&header, msg)?;
        if packet.len() > MAX_UDP_PACKET_SIZE {
            return Err(NetworkError::MalformedPacket(format!(
                "Packet length {} exceeds safe UDP MTU {}",
                packet.len(),
                MAX_UDP_PACKET_SIZE
            )));
        }
        self.socket
            .send_to(&packet, dest)
            .map_err(|e| NetworkError::Transport(e.to_string()))
    }

    /// Attempts to receive and decode a single packet from the non-blocking UDP socket.
    /// Returns Ok(Some((header, msg, source_addr))) if a packet was available,
    /// Ok(None) if WouldBlock (no pending packets), or Err if malformed.
    pub fn recv_message(
        &mut self,
    ) -> Result<Option<(PacketHeader, NetworkMessage, SocketAddr)>, NetworkError> {
        match self.socket.recv_from(&mut self.recv_buf) {
            Ok((bytes_read, src)) => {
                let (header, msg) = decode_packet(&self.recv_buf[..bytes_read])?;
                Ok(Some((header, msg, src)))
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(NetworkError::Transport(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::ClientInputFrame;

    #[test]
    fn test_udp_socket_transport_real_loopback_roundtrip() {
        // 1. Bind server transport on ephemeral loopback port
        let mut server_transport =
            UdpSocketTransport::bind("127.0.0.1:0").expect("Failed to bind server UDP socket");
        let server_addr = server_transport.local_addr();

        // 2. Bind client transport on ephemeral loopback port
        let mut client_transport =
            UdpSocketTransport::bind("127.0.0.1:0").expect("Failed to bind client UDP socket");
        let client_addr = client_transport.local_addr();

        assert_ne!(server_addr, client_addr);

        // 3. Client sends ClientHello message to server
        let hello_msg = NetworkMessage::ClientHello {
            client_version: 1,
            client_name: "TestPilot".to_string(),
            auth_token: "auth-test-secret-123".to_string(),
        };

        let bytes_sent = client_transport
            .send_message(server_addr, ChannelType::ReliableOrdered, &hello_msg)
            .expect("Client send must succeed");
        assert!(bytes_sent > 24);

        // Allow OS network loopback buffer delivery
        std::thread::sleep(std::time::Duration::from_millis(10));

        // 4. Server receives and decodes from real UDP socket
        let received = server_transport
            .recv_message()
            .expect("Server recv must succeed");

        assert!(
            received.is_some(),
            "Server should receive pending UDP packet"
        );
        let (header, msg, from_addr) = received.unwrap();
        assert_eq!(from_addr, client_addr);
        assert_eq!(header.channel, ChannelType::ReliableOrdered);

        match msg {
            NetworkMessage::ClientHello {
                client_name,
                auth_token,
                ..
            } => {
                assert_eq!(client_name, "TestPilot");
                assert_eq!(auth_token, "auth-test-secret-123");
            }
            _ => panic!("Expected ClientHello variant"),
        }

        // 5. Server sends response input acknowledgment back to client
        let ack_msg = NetworkMessage::Input(ClientInputFrame {
            tick: 100,
            delta_time: 1.0 / 60.0,
            throttle: 0.85,
            steer: -0.1,
            brake: 0.0,
            handbrake: false,
            client_timestamp_ms: 1600,
        });

        server_transport
            .send_message(client_addr, ChannelType::UnreliableSequenced, &ack_msg)
            .expect("Server send must succeed");

        std::thread::sleep(std::time::Duration::from_millis(10));

        // 6. Client receives response from real UDP socket
        let client_received = client_transport
            .recv_message()
            .expect("Client recv must succeed");
        assert!(client_received.is_some());
        let (cli_header, cli_msg, cli_from) = client_received.unwrap();
        assert_eq!(cli_from, server_addr);
        assert_eq!(cli_header.channel, ChannelType::UnreliableSequenced);

        match cli_msg {
            NetworkMessage::Input(frame) => {
                assert_eq!(frame.tick, 100);
                assert_eq!(frame.throttle, 0.85);
            }
            _ => panic!("Expected Input variant"),
        }
    }
}
