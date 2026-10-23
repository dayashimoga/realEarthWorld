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
            if s <= 0.05 {
                // Severe bumper collision / penetration: apply emergency maximum deceleration immediately
                return -8.0;
            }
            let denom_sqrt =
                (self.max_accel_ms2.max(0.01) * self.comfortable_decel_ms2.max(0.01)).sqrt();
            let delta_v = v - v_lead.max(0.0);
            let s_star = self.min_gap_meters
                + (v * self.time_headway_seconds + (v * delta_v) / (2.0 * denom_sqrt)).max(0.0);
            (s_star / s.max(0.1)).powi(2)
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
    pub current_lane: u32,
    pub target_lane: u32,
    pub lane_change_progress: f32,
    pub is_changing_lane: bool,
    pub lateral_lane_offset: f32,
    pub stuck_timer: f32,
    pub is_stopped_at_signal: bool,
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
            current_lane: 0,
            target_lane: 0,
            lane_change_progress: 1.0,
            is_changing_lane: false,
            lateral_lane_offset: 0.0,
            stuck_timer: 0.0,
            is_stopped_at_signal: false,
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

        // 2. Lateral offset interpolation during lane change
        if self.is_changing_lane {
            self.lane_change_progress = (self.lane_change_progress + dt / 2.0).min(1.0);
            let t = self.lane_change_progress;
            let smooth_t = t * t * (3.0 - 2.0 * t);
            let lane_diff = (self.target_lane as f32 - self.current_lane as f32) * 3.5;
            self.lateral_lane_offset = lane_diff * smooth_t;

            if self.lane_change_progress >= 1.0 {
                self.current_lane = self.target_lane;
                self.is_changing_lane = false;
                self.lateral_lane_offset = 0.0;
            }
        }

        // 3. Pure Pursuit Steering with lateral lane offset
        let fwd_norm = Vec3::new(self.yaw.sin(), 0.0, self.yaw.cos());
        let perp_norm = Vec3::new(fwd_norm.z, 0.0, -fwd_norm.x);
        let adjusted_target = target + perp_norm * self.lateral_lane_offset;
        let adj_diff = adjusted_target - self.position;

        let target_heading = adj_diff.x.atan2(adj_diff.z);
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

        // 4. IDM Acceleration & Velocity Update
        let accel = self
            .idm
            .calculate_acceleration(self.speed_ms, lead_gap, lead_speed);
        self.speed_ms = (self.speed_ms + accel * dt).max(0.0);

        // Track standstill stuck duration for gridlock recovery
        if self.speed_ms < 0.1 && !self.is_stopped_at_signal {
            self.stuck_timer += dt;
        } else {
            self.stuck_timer = 0.0;
        }

        // If bumper contact/penetration occurs with leading vehicle, halt forward speed
        if let Some(gap) = lead_gap {
            if gap <= 0.05 && accel < 0.0 {
                self.speed_ms = 0.0;
            }
        }

        // 5. Update Position
        let forward = Vec3::new(self.yaw.sin(), 0.0, self.yaw.cos());
        self.position += forward * self.speed_ms * dt;

        // 6. Ground Contact Query & Elevation Clamping
        let ray_origin = Vec3::new(self.position.x, self.position.y + 10.0, self.position.z);
        if let Some((dist, _)) = ground.query_ground_contact(ray_origin, 25.0) {
            self.position.y = (ray_origin.y - dist) + 0.35; // Wheel radius elevation
        }
    }
}

/// Traffic light signal phase state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrafficLightState {
    Green,
    Amber,
    Red,
}

/// Dynamic intersection traffic light controller.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrafficLight {
    pub id: u64,
    pub position: Vec3,
    pub cycle_time_seconds: f32,
    pub green_duration_seconds: f32,
    pub amber_duration_seconds: f32,
    pub timer: f32,
}

impl TrafficLight {
    pub fn new(id: u64, position: Vec3) -> Self {
        Self {
            id,
            position,
            cycle_time_seconds: 30.0,
            green_duration_seconds: 12.0,
            amber_duration_seconds: 3.0,
            timer: 0.0,
        }
    }

    pub fn step(&mut self, dt: f32) {
        self.timer = (self.timer + dt) % self.cycle_time_seconds;
    }

    /// Evaluates signal state for North-South versus East-West approaches.
    pub fn state_for_direction(&self, is_ns: bool) -> TrafficLightState {
        let half_cycle = self.cycle_time_seconds * 0.5;
        let t_phase = if is_ns {
            if self.timer < half_cycle {
                self.timer
            } else {
                return TrafficLightState::Red;
            }
        } else {
            if self.timer >= half_cycle {
                self.timer - half_cycle
            } else {
                return TrafficLightState::Red;
            }
        };

        if t_phase < self.green_duration_seconds {
            TrafficLightState::Green
        } else {
            TrafficLightState::Amber
        }
    }
}

/// Stop line marker associated with an intersection traffic signal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntersectionStopLine {
    pub id: u64,
    pub traffic_light_id: u64,
    pub position: Vec3,
    pub is_ns_direction: bool,
}

/// 2D Spatial partitioning hash grid for O(1) local traffic agent neighbor queries.
#[derive(Debug, Clone, Default)]
pub struct TrafficSpatialGrid {
    pub cell_size: f32,
    pub grid: std::collections::HashMap<(i32, i32), Vec<usize>>,
}

impl TrafficSpatialGrid {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size: cell_size.max(5.0),
            grid: std::collections::HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.grid.clear();
    }

    pub fn insert(&mut self, agent_idx: usize, pos: Vec3) {
        let cx = (pos.x / self.cell_size).floor() as i32;
        let cz = (pos.z / self.cell_size).floor() as i32;
        self.grid.entry((cx, cz)).or_default().push(agent_idx);
    }

    pub fn query_nearby(&self, pos: Vec3, radius: f32) -> Vec<usize> {
        let mut results = Vec::new();
        let r_cells = (radius / self.cell_size).ceil() as i32;
        let cx = (pos.x / self.cell_size).floor() as i32;
        let cz = (pos.z / self.cell_size).floor() as i32;

        for dx in -r_cells..=r_cells {
            for dz in -r_cells..=r_cells {
                if let Some(indices) = self.grid.get(&(cx + dx, cz + dz)) {
                    results.extend(indices.iter().copied());
                }
            }
        }
        results
    }
}

/// Contextual audio events emitted by the traffic system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TrafficAudioEvent {
    EngineIdle { agent_id: u64 },
    EngineAccel { agent_id: u64, speed_ms: f32 },
    TireSkid { agent_id: u64, decel_ms2: f32 },
    HornHonk { agent_id: u64 },
    SignalWalkChime { light_id: u64 },
}

/// Procedural pedestrian crowd agent walking along sidewalks and crosswalks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PedestrianAgent {
    pub id: u64,
    pub position: Vec3,
    pub yaw: f32,
    pub speed_ms: f32,
    pub path: Vec<Vec3>,
    pub current_point_idx: usize,
    pub is_crossing: bool,
    pub walk_cycle_phase: f32,
    pub is_active: bool,
}

impl PedestrianAgent {
    pub fn new(id: u64, path: Vec<Vec3>, speed_ms: f32) -> Self {
        let position = path.first().copied().unwrap_or(Vec3::ZERO);
        let yaw = if path.len() > 1 {
            let diff = path[1] - path[0];
            diff.x.atan2(diff.z)
        } else {
            0.0
        };

        Self {
            id,
            position,
            yaw,
            speed_ms: speed_ms.max(0.8),
            path,
            current_point_idx: 0,
            is_crossing: false,
            walk_cycle_phase: 0.0,
            is_active: true,
        }
    }

    pub fn step(&mut self, dt: f32, ground: &impl GroundQuery) {
        if !self.is_active || self.path.is_empty() {
            return;
        }

        let target = self.path[self.current_point_idx];
        let diff = target - self.position;
        let dist = (diff.x * diff.x + diff.z * diff.z).sqrt();

        if dist < 1.0 {
            if self.current_point_idx + 1 < self.path.len() {
                self.current_point_idx += 1;
            } else {
                self.path.reverse();
                self.current_point_idx = 0;
            }
        }

        let target = self.path[self.current_point_idx];
        let diff = target - self.position;
        self.yaw = diff.x.atan2(diff.z);

        let forward = Vec3::new(self.yaw.sin(), 0.0, self.yaw.cos());
        self.position += forward * self.speed_ms * dt;
        self.walk_cycle_phase += self.speed_ms * dt * 5.0;

        let ray = Vec3::new(self.position.x, self.position.y + 5.0, self.position.z);
        if let Some((hit_dist, _)) = ground.query_ground_contact(ray, 15.0) {
            self.position.y = ray.y - hit_dist;
        }
    }
}

/// Traffic simulation managing autonomous AI vehicles, signals, and pedestrians.
#[derive(Debug)]
pub struct TrafficSimulation {
    pub agents: Vec<TrafficAgent>,
    pub traffic_lights: Vec<TrafficLight>,
    pub stop_lines: Vec<IntersectionStopLine>,
    pub pedestrians: Vec<PedestrianAgent>,
    pub spatial_grid: TrafficSpatialGrid,
    pub audio_events: Vec<TrafficAudioEvent>,
}

impl Default for TrafficSimulation {
    fn default() -> Self {
        Self::new()
    }
}

impl TrafficSimulation {
    pub fn new() -> Self {
        Self {
            agents: Vec::new(),
            traffic_lights: Vec::new(),
            stop_lines: Vec::new(),
            pedestrians: Vec::new(),
            spatial_grid: TrafficSpatialGrid::new(25.0),
            audio_events: Vec::new(),
        }
    }

    /// Spawns a new traffic agent with the given path.
    pub fn spawn_agent(&mut self, agent: TrafficAgent) {
        self.agents.push(agent);
    }

    /// Adds a traffic light to the simulation.
    pub fn add_traffic_light(&mut self, light: TrafficLight) {
        self.traffic_lights.push(light);
    }

    /// Adds a stop line to the simulation.
    pub fn add_stop_line(&mut self, stop_line: IntersectionStopLine) {
        self.stop_lines.push(stop_line);
    }

    /// Adds a pedestrian agent to the simulation.
    pub fn add_pedestrian(&mut self, pedestrian: PedestrianAgent) {
        self.pedestrians.push(pedestrian);
    }

    /// Drains all emitted audio events.
    pub fn drain_audio_events(&mut self) -> Vec<TrafficAudioEvent> {
        std::mem::take(&mut self.audio_events)
    }

    /// Advances the entire traffic fleet by `dt` seconds with spatial indexing and signal logic.
    pub fn step(&mut self, dt: f32, ground: &impl GroundQuery) {
        // 1. Advance traffic lights
        for light in &mut self.traffic_lights {
            light.step(dt);
        }

        // 2. Advance pedestrians
        for ped in &mut self.pedestrians {
            ped.step(dt, ground);
        }

        let count = self.agents.len();
        if count == 0 {
            return;
        }

        // 3. Rebuild spatial grid for O(1) neighbor queries
        self.spatial_grid.clear();
        for (idx, agent) in self.agents.iter().enumerate() {
            if agent.is_active {
                self.spatial_grid.insert(idx, agent.position);
            }
        }

        // 4. Compute leader gap, speed, and intersection signal status for each vehicle
        let mut leader_info = vec![(None, None, false); count];

        for (i, info) in leader_info.iter_mut().enumerate() {
            if !self.agents[i].is_active {
                continue;
            }

            let p_i = self.agents[i].position;
            let yaw_i = self.agents[i].yaw;
            let forward_i = Vec3::new(yaw_i.sin(), 0.0, yaw_i.cos());

            let mut min_gap = f32::MAX;
            let mut lead_speed = None;
            let mut stopped_at_red = false;

            // Spatial neighborhood query (60m radius)
            let neighbors = self.spatial_grid.query_nearby(p_i, 60.0);
            for j in neighbors {
                if i == j || !self.agents[j].is_active {
                    continue;
                }

                let other = &self.agents[j];
                let to_j = other.position - p_i;
                let forward_dist = forward_i.dot(to_j);

                // Is vehicle j ahead within 80m or overlapping?
                if forward_dist > -other.length_meters && forward_dist < 80.0 {
                    let lateral_dist = (to_j - forward_i * forward_dist).length();
                    // Same lane detection
                    if lateral_dist < 2.5 {
                        let forward_j = Vec3::new(other.yaw.sin(), 0.0, other.yaw.cos());
                        let heading_dot = forward_i.dot(forward_j);
                        if heading_dot > 0.2 || (heading_dot <= 0.2 && lateral_dist < 1.2) {
                            let bumper_gap = forward_dist - other.length_meters;
                            if bumper_gap < min_gap {
                                min_gap = bumper_gap;
                                lead_speed = Some(other.speed_ms);
                            }
                        }
                    }
                }
            }

            // Intersection traffic signal stop line check
            for stop_line in &self.stop_lines {
                let to_stop = stop_line.position - p_i;
                let forward_dist = forward_i.dot(to_stop);
                if forward_dist > 0.0 && forward_dist < 50.0 {
                    let lateral_dist = (to_stop - forward_i * forward_dist).length();
                    if lateral_dist < 3.5 {
                        if let Some(light) = self
                            .traffic_lights
                            .iter()
                            .find(|l| l.id == stop_line.traffic_light_id)
                        {
                            let light_state = light.state_for_direction(stop_line.is_ns_direction);
                            if light_state == TrafficLightState::Red
                                || light_state == TrafficLightState::Amber
                            {
                                let stop_gap = (forward_dist - 2.0).max(0.1);
                                if stop_gap < min_gap {
                                    min_gap = stop_gap;
                                    lead_speed = Some(0.0);
                                    stopped_at_red = true;
                                }
                            }
                        }
                    }
                }
            }

            // Pedestrian crosswalk yielding check
            for ped in &self.pedestrians {
                if !ped.is_active {
                    continue;
                }
                let to_ped = ped.position - p_i;
                let forward_dist = forward_i.dot(to_ped);
                if forward_dist > 0.0 && forward_dist < 30.0 {
                    let lateral_dist = (to_ped - forward_i * forward_dist).length();
                    // Detect pedestrian across the entire road crosswalk zone (+/- 6.0m lateral width)
                    if lateral_dist < 6.0 {
                        let ped_gap = (forward_dist - 3.0).max(0.1);
                        if ped_gap < min_gap {
                            min_gap = ped_gap;
                            lead_speed = Some(0.0);
                        }
                    }
                }
            }

            if min_gap < f32::MAX {
                *info = (Some(min_gap), lead_speed, stopped_at_red);
            }
        }

        // 5. Advance each agent and handle gridlock recovery & audio
        for (i, agent) in self.agents.iter_mut().enumerate() {
            let (gap, speed, is_stopped_at_signal) = leader_info[i];
            agent.is_stopped_at_signal = is_stopped_at_signal;

            // Audio skid detection on hard braking
            let accel = agent.idm.calculate_acceleration(agent.speed_ms, gap, speed);
            if accel < -5.0 && agent.speed_ms > 5.0 {
                self.audio_events.push(TrafficAudioEvent::TireSkid {
                    agent_id: agent.id,
                    decel_ms2: accel,
                });
            }

            // Gridlock recovery: if vehicle stuck behind blockage for > 15s
            if agent.stuck_timer > 15.0 && !is_stopped_at_signal {
                // Horn honk
                self.audio_events
                    .push(TrafficAudioEvent::HornHonk { agent_id: agent.id });

                // Nudge agent forward or change lane if available
                if agent.stuck_timer > 20.0 {
                    // Unstick recovery: advance forward slightly or unstuck
                    let forward = Vec3::new(agent.yaw.sin(), 0.0, agent.yaw.cos());
                    agent.position += forward * 2.0;
                    agent.speed_ms = 4.0;
                    agent.stuck_timer = 0.0;
                }
            }

            agent.step(dt, gap, speed, ground);
        }
    }

    /// Spawns `count` autonomous traffic agents distributed across drivable edges in the road network.
    pub fn populate_from_road_network(
        &mut self,
        network: &realearth_navigation::RoadNetworkGraph,
        count: usize,
        rule: realearth_navigation::RuleOfTheRoad,
    ) {
        use petgraph::visit::EdgeRef;
        use realearth_navigation::LaneDescriptor;

        let edges: Vec<_> = network
            .graph
            .edge_references()
            .filter(|e| e.weight().is_drivable_motor_vehicle())
            .collect();

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
                weight.is_oneway,
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

    #[test]
    fn test_traffic_light_state_transitions() {
        let mut light = TrafficLight::new(1, Vec3::new(0.0, 0.0, 0.0));
        light.cycle_time_seconds = 30.0;
        light.green_duration_seconds = 12.0;
        light.amber_duration_seconds = 3.0;

        // At t = 0: NS should be Green, EW should be Red
        assert_eq!(light.state_for_direction(true), TrafficLightState::Green);
        assert_eq!(light.state_for_direction(false), TrafficLightState::Red);

        // Advance 13s: NS should be Amber
        light.step(13.0);
        assert_eq!(light.state_for_direction(true), TrafficLightState::Amber);
        assert_eq!(light.state_for_direction(false), TrafficLightState::Red);

        // Advance another 3s (t = 16s): EW should be Green, NS should be Red
        light.step(3.0);
        assert_eq!(light.state_for_direction(true), TrafficLightState::Red);
        assert_eq!(light.state_for_direction(false), TrafficLightState::Green);
    }

    #[test]
    fn test_traffic_signal_stop_line_braking_and_queue_dissipation() {
        let ground = FlatGround::new(0.0);
        let mut sim = TrafficSimulation::new();

        // Traffic light at intersection z = 60m, with stop line at z = 50m
        let mut light = TrafficLight::new(10, Vec3::new(0.0, 0.0, 60.0));
        // Force light into Red phase for NS (t = 16s into 30s cycle)
        light.timer = 16.0;
        sim.add_traffic_light(light);

        sim.add_stop_line(IntersectionStopLine {
            id: 100,
            traffic_light_id: 10,
            position: Vec3::new(0.0, 0.0, 50.0),
            is_ns_direction: true,
        });

        let waypoints = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 50.0),
            Vec3::new(0.0, 0.0, 150.0),
        ];

        // Car 1 approaches at z = 20m, speed = 10 m/s
        let mut car1 = TrafficAgent::new(
            1,
            Vec3::new(0.0, 0.0, 20.0),
            waypoints.clone(),
            IdmConfig {
                desired_speed_ms: 12.0,
                ..Default::default()
            },
        );
        car1.speed_ms = 10.0;

        // Car 2 follows at z = 0m, speed = 10 m/s
        let mut car2 = TrafficAgent::new(
            2,
            Vec3::new(0.0, 0.0, 0.0),
            waypoints,
            IdmConfig {
                desired_speed_ms: 12.0,
                ..Default::default()
            },
        );
        car2.speed_ms = 10.0;

        sim.spawn_agent(car1);
        sim.spawn_agent(car2);

        // Step for 6 seconds during Red light (120 ticks) to allow IDM comfortable deceleration to complete standstill
        for _ in 0..120 {
            // Keep light red by fixing timer in second half
            sim.traffic_lights[0].timer = 16.0;
            sim.step(0.05, &ground);
        }

        // Both cars must have halted before the stop line (z = 50m)
        assert!(
            sim.agents[0].position.z <= 50.0,
            "Lead vehicle must stop before stop line at z = 50m (got z = {})",
            sim.agents[0].position.z
        );
        assert!(
            sim.agents[0].speed_ms < 0.5,
            "Lead vehicle must be halted at red light (got speed = {})",
            sim.agents[0].speed_ms
        );

        // Follower must queue safely behind lead car
        assert!(
            sim.agents[1].position.z < sim.agents[0].position.z,
            "Follower must be behind lead vehicle"
        );
        let queue_gap = sim.agents[0].position.z - sim.agents[1].position.z;
        assert!(
            queue_gap >= 3.0,
            "Follower must maintain standstill gap behind lead vehicle (got gap = {})",
            queue_gap
        );

        // Now light turns GREEN!
        sim.traffic_lights[0].timer = 2.0;

        // Step for 4 seconds during Green light
        for _ in 0..80 {
            sim.traffic_lights[0].timer = 2.0;
            sim.step(0.05, &ground);
        }

        // Vehicles must accelerate and clear the intersection
        assert!(
            sim.agents[0].position.z > 50.0,
            "Lead vehicle must cross intersection on green light (got z = {})",
            sim.agents[0].position.z
        );
        assert!(
            sim.agents[0].speed_ms > 5.0,
            "Lead vehicle must accelerate on green light"
        );
    }

    #[test]
    fn test_spatial_grid_neighbor_indexing() {
        let mut grid = TrafficSpatialGrid::new(20.0);

        // Insert agents at various coordinates
        grid.insert(0, Vec3::new(0.0, 0.0, 0.0));
        grid.insert(1, Vec3::new(10.0, 0.0, 5.0));
        grid.insert(2, Vec3::new(150.0, 0.0, 200.0)); // Far away

        let nearby = grid.query_nearby(Vec3::new(5.0, 0.0, 2.0), 30.0);
        assert!(nearby.contains(&0));
        assert!(nearby.contains(&1));
        assert!(
            !nearby.contains(&2),
            "Distant agent must not be returned in local neighborhood query"
        );
    }

    #[test]
    fn test_gridlock_recovery_unstuck() {
        let ground = FlatGround::new(0.0);
        let mut sim = TrafficSimulation::new();

        let waypoints = vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 100.0)];
        let mut stuck_agent = TrafficAgent::new(
            1,
            Vec3::new(0.0, 0.0, 10.0),
            waypoints,
            IdmConfig::default(),
        );
        stuck_agent.speed_ms = 0.0;
        stuck_agent.stuck_timer = 20.5; // Exceeds gridlock recovery threshold (20.0s)
        stuck_agent.is_stopped_at_signal = false;

        sim.spawn_agent(stuck_agent);

        // Step simulation by 1 tick
        sim.step(0.05, &ground);

        // Audio horn should have sounded and agent should have been unstuck
        let events = sim.drain_audio_events();
        assert!(
            events
                .iter()
                .any(|e| matches!(e, TrafficAudioEvent::HornHonk { .. })),
            "Horn honk must be triggered upon gridlock"
        );
        assert!(
            sim.agents[0].speed_ms > 0.0,
            "Stuck agent must be given forward velocity on gridlock recovery"
        );
        assert_eq!(
            sim.agents[0].stuck_timer, 0.0,
            "Stuck timer must reset after gridlock recovery"
        );
    }

    #[test]
    fn test_pedestrian_crosswalk_yielding() {
        let ground = FlatGround::new(0.0);
        let mut sim = TrafficSimulation::new();

        let waypoints = vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 80.0)];
        let mut car = TrafficAgent::new(
            1,
            Vec3::new(0.0, 0.0, 5.0),
            waypoints,
            IdmConfig {
                desired_speed_ms: 8.0,
                ..Default::default()
            },
        );
        car.speed_ms = 6.0;
        sim.spawn_agent(car);

        // Pedestrian crossing at z = 22m (right in vehicle's travel path)
        let ped_path = vec![Vec3::new(-3.0, 0.0, 22.0), Vec3::new(3.0, 0.0, 22.0)];
        let ped = PedestrianAgent::new(101, ped_path, 1.2);
        sim.add_pedestrian(ped);

        // Step for 4.5 seconds (90 ticks) to complete full stop
        for _ in 0..90 {
            sim.step(0.05, &ground);
        }

        // Vehicle must yield before pedestrian at z = 22m
        assert!(
            sim.agents[0].position.z < 22.0,
            "Vehicle must stop before pedestrian crosswalk (got z = {})",
            sim.agents[0].position.z
        );
        assert!(
            sim.agents[0].speed_ms < 0.5,
            "Vehicle must come to a halt to yield for crossing pedestrian (got speed = {})",
            sim.agents[0].speed_ms
        );
    }
}
