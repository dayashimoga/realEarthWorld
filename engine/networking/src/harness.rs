//! Network simulation harness: latency, jitter, packet loss, and reordering injection
//! for rigorous continuous integration stress testing.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// Configuration parameters for simulated network transport impairments.
#[derive(Debug, Clone, PartialEq)]
pub struct NetworkImpairmentConfig {
    /// Base one-way transit delay in milliseconds (e.g. 50ms).
    pub base_latency_ms: u64,
    /// Random delay jitter variation in milliseconds (e.g. +/- 15ms).
    pub jitter_ms: u64,
    /// Packet loss probability [0.0, 1.0] (e.g. 0.05 for 5% drop).
    pub packet_loss_rate: f32,
    /// Packet duplication probability [0.0, 1.0].
    pub duplication_rate: f32,
    /// Reordering probability.
    pub reorder_rate: f32,
}

impl Default for NetworkImpairmentConfig {
    fn default() -> Self {
        Self {
            base_latency_ms: 0,
            jitter_ms: 0,
            packet_loss_rate: 0.0,
            duplication_rate: 0.0,
            reorder_rate: 0.0,
        }
    }
}

/// A scheduled in-flight packet waiting for simulated transit delay.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ScheduledPacket {
    deliver_at_ms: u64,
    sequence: u64,
    payload: Vec<u8>,
}

impl Ord for ScheduledPacket {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap (earliest delivery time at top)
        other
            .deliver_at_ms
            .cmp(&self.deliver_at_ms)
            .then_with(|| other.sequence.cmp(&self.sequence))
    }
}

impl PartialOrd for ScheduledPacket {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Simulated network transport link with deterministic pseudo-random impairment injection.
#[derive(Debug)]
pub struct SimulatedNetworkLink {
    pub config: NetworkImpairmentConfig,
    queue: BinaryHeap<ScheduledPacket>,
    next_sequence: u64,
    rng_seed: u64,
    // Statistics
    pub packets_sent: u64,
    pub packets_delivered: u64,
    pub packets_dropped: u64,
}

impl SimulatedNetworkLink {
    pub fn new(config: NetworkImpairmentConfig) -> Self {
        Self {
            config,
            queue: BinaryHeap::new(),
            next_sequence: 1,
            rng_seed: 0xDEAD_BEEF_CAFE_BABE,
            packets_sent: 0,
            packets_delivered: 0,
            packets_dropped: 0,
        }
    }

    /// Linear congruential pseudo-random number generator for deterministic test reproducibility.
    fn next_float(&mut self) -> f32 {
        self.rng_seed = self
            .rng_seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let val = (self.rng_seed >> 32) as u32;
        (val as f32) / (u32::MAX as f32)
    }

    /// Sends a packet across the simulated link.
    pub fn send(&mut self, payload: Vec<u8>, current_time_ms: u64) {
        self.packets_sent += 1;

        // 1. Packet Loss Check
        if self.config.packet_loss_rate > 0.0 && self.next_float() < self.config.packet_loss_rate {
            self.packets_dropped += 1;
            return;
        }

        // 2. Latency & Jitter Calculation
        let jitter_offset = if self.config.jitter_ms > 0 {
            let rnd = self.next_float();
            let jitter_range = (self.config.jitter_ms * 2) as f32;
            ((rnd * jitter_range) as i64) - (self.config.jitter_ms as i64)
        } else {
            0
        };

        let delay_ms = (self.config.base_latency_ms as i64 + jitter_offset).max(0) as u64;
        let deliver_at_ms = current_time_ms + delay_ms;

        let seq = self.next_sequence;
        self.next_sequence += 1;

        self.queue.push(ScheduledPacket {
            deliver_at_ms,
            sequence: seq,
            payload: payload.clone(),
        });

        // 3. Optional Duplication
        if self.config.duplication_rate > 0.0 && self.next_float() < self.config.duplication_rate {
            self.queue.push(ScheduledPacket {
                deliver_at_ms: deliver_at_ms + 10,
                sequence: seq,
                payload,
            });
        }
    }

    /// Drains all packets whose simulated arrival time has arrived.
    pub fn receive(&mut self, current_time_ms: u64) -> Vec<Vec<u8>> {
        let mut ready = Vec::new();

        while let Some(top) = self.queue.peek() {
            if top.deliver_at_ms <= current_time_ms {
                let packet = self.queue.pop().unwrap();
                self.packets_delivered += 1;
                ready.push(packet.payload);
            } else {
                break;
            }
        }

        ready
    }

    /// Total packets currently buffered in transit.
    pub fn in_flight_count(&self) -> usize {
        self.queue.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lossless_latency_delay() {
        let config = NetworkImpairmentConfig {
            base_latency_ms: 50,
            jitter_ms: 0,
            packet_loss_rate: 0.0,
            duplication_rate: 0.0,
            reorder_rate: 0.0,
        };
        let mut link = SimulatedNetworkLink::new(config);

        let data = vec![1, 2, 3, 4];
        link.send(data.clone(), 100);

        // At t = 120ms (only 20ms elapsed), packet should not arrive yet
        let received_early = link.receive(120);
        assert!(received_early.is_empty());
        assert_eq!(link.in_flight_count(), 1);

        // At t = 150ms (50ms elapsed), packet must be delivered
        let received_on_time = link.receive(150);
        assert_eq!(received_on_time.len(), 1);
        assert_eq!(received_on_time[0], data);
        assert_eq!(link.in_flight_count(), 0);
    }

    #[test]
    fn test_packet_loss_drop_rate() {
        let config = NetworkImpairmentConfig {
            base_latency_ms: 10,
            jitter_ms: 0,
            packet_loss_rate: 0.30, // 30% drop rate
            duplication_rate: 0.0,
            reorder_rate: 0.0,
        };
        let mut link = SimulatedNetworkLink::new(config);

        let total = 200;
        for i in 0..total {
            link.send(vec![i as u8], 0);
        }

        assert_eq!(link.packets_sent, total);
        assert!(
            link.packets_dropped > 30,
            "Should have dropped significant packets: {}",
            link.packets_dropped
        );
        assert!(link.packets_dropped < 90);

        let received = link.receive(50);
        assert_eq!(received.len(), (total - link.packets_dropped) as usize);
    }
}
