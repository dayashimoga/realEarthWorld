//! Autonomous AI traffic simulation using the Intelligent Driver Model (IDM) and Pure Pursuit.

use crate::ground::GroundQuery;
use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Intelligent Driver Model (IDM) configuration for realistic car-following behavior.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct IdmConfig {
    /// Desired free-flow cruising speed in m/s.
    pub desired_speed_ms: f32,
    /// Maximum comfortable acceleration in m/s^2.
    pub max_accel_ms2: f32,
    /// Comfortable braking deceleration in m/s^2.
    pub comfortable_decel_ms2: f32,
    /// Minimum bumper-to-bumper standstill jam distance in meters.
    pub min_gap_meters: f32,
    /// Safe time headway to leading vehicle in seconds.
    pub time_headway_seconds: f32,
    /// Free acceleration exponent (typically 4.0).
    pub accel_exponent: f32,
}

impl Default for IdmConfig {
    fn default() -> Self {
        Self {
            desired_speed_ms: 13.89, // 50 km/h
            max_accel_ms2: 1.8,
            comfortable_decel_ms2: 2.0,
            min_gap_meters: 3.5,
            time_headway_seconds: 1.5,
            accel_exponent: 4.0,
        }
    }
}

impl IdmConfig {
    /// Computes longitudinal acceleration based on velocity, distance to leader, and leader velocity.
    pub fn calculate_acceleration(
        &self,
        current_speed: f32,
        lead_gap: Option<f32>,
        lead_speed: Option<f32>,
    ) -> f32 {
        let v = current_speed.max(0.0);
        let free_road_term = 1.0 - (v / self.desired_speed_ms.max(0.1)).powf(self.accel_exponent);

        let interaction_term = if let (Some(s), Some(v_lead)) = (lead_gap, lead_speed) {
            let delta_v = v - v_lead.max(0.0);
            let s_star = self.min_gap_meters
                + (v * self.time_headway_seconds
                    + (v * delta_v)
                        / (2.0 * (self.max_accel_ms2 * self.comfortable_decel_ms2).sqrt()))
                .max(0.0);
            (s_star / s.max(0.5)).powi(2)
        } else {
            0.0
        };

        let accel = self.max_accel_ms2 * (free_road_term - interaction_term);
        // Clamp to emergency braking limit (-8 m/s^2)
        accel.clamp(-8.0, self.max_accel_ms2)
    }
}

/// Autonomous vehicle agent participating in road traffic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficAgent {
    pub id: u64,
    pub position: Vec3,
    pub yaw: f32,
    pub speed_ms: f32,
    pub idm: IdmConfig,
    pub waypoints: Vec<Vec3>,
    pub current_waypoint_idx: usize,
    pub lookahead_dist: f32,
    pub is_active: bool,
    pub length_meters: f32,
}

impl TrafficAgent {
    pub fn new(id: u64, initial_pos: Vec3, waypoints: Vec<Vec3>, idm: IdmConfig) -> Self {
        let yaw = if waypoints.len() > 1 {
            let diff = waypoints[1] - waypoints[0];
            diff.x.atan2(diff.z)
        } else {
            0.0
        };

        Self {
            id,
            position: initial_pos,
            yaw,
            speed_ms: 0.0,
            idm,
            waypoints,
            current_waypoint_idx: 0,
            lookahead_dist: 6.0,
            is_active: true,
            length_meters: 4.5,
        }
    }

    /// Advances the agent along its route using IDM for speed and Pure Pursuit for steering.
    pub fn step(
        &mut self,
        dt: f32,
        lead_gap: Option<f32>,
        lead_speed: Option<f32>,
        ground: &impl GroundQuery,
    ) {
        if !self.is_active || self.waypoints.is_empty() {
            return;
        }

        // 1. Advance waypoint target if reached or behind vehicle
        while self.current_waypoint_idx < self.waypoints.len() - 1 {
            let target = self.waypoints[self.current_waypoint_idx];
            let to_target = target - self.position;
            let dist_xz = (to_target.x * to_target.x + to_target.z * to_target.z).sqrt();
            let forward = Vec3::new(self.yaw.sin(), 0.0, self.yaw.cos());
            let is_behind = forward.dot(to_target) < 0.0;

            if is_behind || dist_xz < self.lookahead_dist {
                self.current_waypoint_idx += 1;
            } else {
                break;
            }
        }

        let target = self.waypoints[self.current_waypoint_idx];
        let diff = target - self.position;
        let dist_to_target = (diff.x * diff.x + diff.z * diff.z).sqrt();

        // Check if finished entire path
        if dist_to_target < 2.0 && self.current_waypoint_idx == self.waypoints.len() - 1 {
            self.speed_ms = 0.0;
            self.is_active = false;
            return;
        }

        // 2. Pure Pursuit Steering
        let target_heading = diff.x.atan2(diff.z);
        let mut angle_error = target_heading - self.yaw;
        while angle_error > std::f32::consts::PI {
            angle_error -= std::f32::consts::TAU;
        }
        while angle_error < -std::f32::consts::PI {
            angle_error += std::f32::consts::TAU;
        }

        let max_steer_rate = 2.5; // rad/s
        let steer_delta = (angle_error * 3.0).clamp(-max_steer_rate, max_steer_rate);
        self.yaw += steer_delta * dt;

        // 3. IDM Acceleration & Velocity Update
        let accel = self
            .idm
            .calculate_acceleration(self.speed_ms, lead_gap, lead_speed);
        self.speed_ms = (self.speed_ms + accel * dt).max(0.0);

        // 4. Update Position
        let forward = Vec3::new(self.yaw.sin(), 0.0, self.yaw.cos());
        self.position += forward * self.speed_ms * dt;

        // 5. Ground Contact Query & Elevation Clamping
        let ray_origin = Vec3::new(self.position.x, self.position.y + 10.0, self.position.z);
        if let Some((dist, _)) = ground.query_ground_contact(ray_origin, 25.0) {
            self.position.y = (ray_origin.y - dist) + 0.35; // Wheel radius elevation
        }
    }
}

/// Traffic simulation managing autonomous AI vehicles.
#[derive(Debug, Default)]
pub struct TrafficSimulation {
    pub agents: Vec<TrafficAgent>,
}

impl TrafficSimulation {
    pub fn new() -> Self {
        Self { agents: Vec::new() }
    }

    /// Spawns a new traffic agent with the given path.
    pub fn spawn_agent(&mut self, agent: TrafficAgent) {
        self.agents.push(agent);
    }

    /// Advances the entire traffic fleet by `dt` seconds with inter-vehicle leader detection.
    pub fn step(&mut self, dt: f32, ground: &impl GroundQuery) {
        let count = self.agents.len();
        if count == 0 {
            return;
        }

        // Precompute leader info for each vehicle
        let mut leader_info = vec![(None, None); count];

        for (i, info) in leader_info.iter_mut().enumerate() {
            if !self.agents[i].is_active {
                continue;
            }

            let p_i = self.agents[i].position;
            let yaw_i = self.agents[i].yaw;
            let forward_i = Vec3::new(yaw_i.sin(), 0.0, yaw_i.cos());

            let mut min_gap = f32::MAX;
            let mut lead_speed = None;

            for (j, other) in self.agents.iter().enumerate() {
                if i == j || !other.is_active {
                    continue;
                }

                let to_j = other.position - p_i;
                let forward_dist = forward_i.dot(to_j);

                // Is vehicle j ahead within 100 meters?
                if forward_dist > 0.0 && forward_dist < 100.0 {
                    let lateral_dist = (to_j - forward_i * forward_dist).length();
                    // In same lane if lateral distance < 2.5 meters
                    if lateral_dist < 2.5 {
                        let bumper_gap = forward_dist - other.length_meters;
                        if bumper_gap < min_gap {
                            min_gap = bumper_gap;
                            lead_speed = Some(other.speed_ms);
                        }
                    }
                }
            }

            if min_gap < f32::MAX {
                *info = (Some(min_gap.max(0.1)), lead_speed);
            }
        }

        // Advance each agent
        for (i, agent) in self.agents.iter_mut().enumerate() {
            let (gap, speed) = leader_info[i];
            agent.step(dt, gap, speed, ground);
        }
    }

    /// Spawns `count` autonomous traffic agents distributed across drivable edges in the road graph.
    pub fn populate_from_road_network(
        &mut self,
        network: &realearth_navigation::RoadNetworkGraph,
        count: usize,
        rule: realearth_navigation::RuleOfTheRoad,
    ) {
        use petgraph::visit::EdgeRef;
        use realearth_navigation::LaneDescriptor;

        let edges: Vec<_> = network.graph.edge_references().collect();
        if edges.is_empty() {
            return;
        }

        for i in 0..count {
            let edge = edges[i % edges.len()];
            let source = network.graph[edge.source()].coord;
            let target = network.graph[edge.target()].coord;
            let weight = edge.weight();

            let lane = LaneDescriptor::new(
                (i as u32) % (weight.lanes.max(1)),
                weight.lanes,
                false,
                rule,
            );

            let (p0_lane, p1_lane) = network.compute_lane_coordinates(source, target, &lane);

            let d_lat = (p1_lane.lat - p0_lane.lat) * 111_000.0;
            let d_lon = (p1_lane.lon - p0_lane.lon) * 108_000.0;

            let wp0 = Vec3::new(0.0, p0_lane.alt as f32, 0.0);
            let wp1 = Vec3::new(d_lon as f32, p1_lane.alt as f32, d_lat as f32);

            let fraction = ((i * 17) % 100) as f32 / 100.0;
            let spawn_pos = wp0.lerp(wp1, fraction);

            let target_speed_ms =
                ((weight.speed_limit_kph as f32) * (1000.0 / 3600.0)).clamp(8.0, 16.0);
            let idm = IdmConfig {
                desired_speed_ms: target_speed_ms,
                ..Default::default()
            };

            let mut agent = TrafficAgent::new(1000 + i as u64, spawn_pos, vec![wp0, wp1], idm);
            agent.speed_ms = target_speed_ms * 0.7;
            self.spawn_agent(agent);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ground::FlatGround;

    #[test]
    fn test_idm_free_flow_acceleration_towards_desired_speed() {
        let idm = IdmConfig {
            desired_speed_ms: 20.0, // 20 m/s (~72 km/h)
            max_accel_ms2: 2.0,
            ..Default::default()
        };

        // When stationary on free road, accel should equal max_accel
        let a0 = idm.calculate_acceleration(0.0, None, None);
        assert!((a0 - 2.0).abs() < 1e-4);

        // At desired speed, accel should equal 0.0
        let a_target = idm.calculate_acceleration(20.0, None, None);
        assert!(a_target.abs() < 1e-3);

        // Above desired speed, accel should be negative
        let a_fast = idm.calculate_acceleration(25.0, None, None);
        assert!(a_fast < 0.0);
    }

    #[test]
    fn test_idm_car_following_braking_behind_slow_leader() {
        let idm = IdmConfig {
            desired_speed_ms: 20.0,
            comfortable_decel_ms2: 2.0,
            min_gap_meters: 4.0,
            ..Default::default()
        };

        // Approaching stationary leader (v_lead = 0) at 15 m/s with small gap (10m)
        let a_brake = idm.calculate_acceleration(15.0, Some(10.0), Some(0.0));
        assert!(
            a_brake < -1.5,
            "Vehicle must brake strongly when rapidly closing on stopped leader, got {}",
            a_brake
        );
    }

    #[test]
    fn test_traffic_simulation_platoon_collision_avoidance() {
        let ground = FlatGround::new(0.0);
        let mut sim = TrafficSimulation::new();

        let waypoints = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 50.0),
            Vec3::new(0.0, 0.0, 200.0),
        ];

        // Leader vehicle starts at z = 30m, cruising slowly at 5 m/s
        let mut lead = TrafficAgent::new(
            1,
            Vec3::new(0.0, 0.0, 30.0),
            waypoints.clone(),
            IdmConfig {
                desired_speed_ms: 5.0,
                ..Default::default()
            },
        );
        lead.speed_ms = 5.0;

        // Follower vehicle starts at z = 0m, wants to cruise at 15 m/s
        let mut follower = TrafficAgent::new(
            2,
            Vec3::new(0.0, 0.0, 0.0),
            waypoints,
            IdmConfig {
                desired_speed_ms: 15.0,
                ..Default::default()
            },
        );
        follower.speed_ms = 10.0;

        sim.spawn_agent(lead);
        sim.spawn_agent(follower);

        // Step simulation for 100 ticks of 0.05s = 5 seconds
        for _ in 0..100 {
            sim.step(0.05, &ground);
        }

        let lead_z = sim.agents[0].position.z;
        let follower_z = sim.agents[1].position.z;
        let gap = lead_z - follower_z;

        // Follower must not rear-end leader
        assert!(
            follower_z < lead_z,
            "Follower z ({}) must be behind leader z ({})",
            follower_z,
            lead_z
        );
        assert!(
            gap >= 3.0,
            "Safety gap must be maintained (got gap = {}m)",
            gap
        );
    }
}
