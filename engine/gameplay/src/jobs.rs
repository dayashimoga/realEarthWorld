//! Mission and job state machines, authentic transport contracts, and interactions.

use crate::career::{CareerType, ReputationTier};
use crate::events::{GameplayEvent, GameplayEventBus};
use crate::profile::PlayerProfile;
use realearth_geospatial::wgs84_to_ecef;
use realearth_shared::coordinates::Wgs84Coord;
use serde::{Deserialize, Serialize};

/// Geographic waypoint or interaction perimeter for a job.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JobLocation {
    pub name: String,
    pub coord: Wgs84Coord,
    pub region_id: String,
    pub radius_meters: f32,
}

impl JobLocation {
    pub fn new(
        name: impl Into<String>,
        coord: Wgs84Coord,
        region_id: impl Into<String>,
        radius_meters: f32,
    ) -> Self {
        Self {
            name: name.into(),
            coord,
            region_id: region_id.into(),
            radius_meters: radius_meters.max(5.0),
        }
    }

    /// Evaluates metric distance in 3D ECEF from a given vehicle location.
    pub fn distance_to(&self, vehicle_coord: Wgs84Coord) -> f64 {
        let ecef_loc = wgs84_to_ecef(self.coord);
        let ecef_veh = wgs84_to_ecef(vehicle_coord);
        let dx = ecef_loc.x - ecef_veh.x;
        let dy = ecef_loc.y - ecef_veh.y;
        let dz = ecef_loc.z - ecef_veh.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Checks if a vehicle coordinate falls inside this interaction zone.
    pub fn contains(&self, vehicle_coord: Wgs84Coord) -> bool {
        self.distance_to(vehicle_coord) <= (self.radius_meters as f64)
    }
}

/// Nature of payload carried during a transport job.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CargoOrPassenger {
    /// Passenger transport with comfort sensitivity.
    Passenger {
        name: String,
        party_size: u8,
        destination_note: String,
    },
    /// Parcel or courier package with fragility rating.
    Package {
        item_name: String,
        weight_kg: f32,
        fragile: bool,
    },
    /// Commercial heavy freight with tonnage rating.
    Freight {
        cargo_type: String,
        tonnage: f32,
        hazardous: bool,
    },
    /// Open exploration without payload constraints.
    Exploration,
}

/// Operational difficulty rating for transport contracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobDifficulty {
    Relaxed,
    Standard,
    Challenging,
    Extreme,
}

impl JobDifficulty {
    pub fn payout_bonus(&self) -> f64 {
        match self {
            Self::Relaxed => 1.0,
            Self::Standard => 1.15,
            Self::Challenging => 1.40,
            Self::Extreme => 1.80,
        }
    }
}

/// Comprehensive state transitions for active jobs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JobState {
    /// Available in regional dispatch catalog.
    Available,
    /// Accepted by player, ready to commence.
    Accepted,
    /// Player driving towards authentic pickup location.
    EnRouteToPickup,
    /// Vehicle stopped within pickup perimeter; boarding/loading in progress.
    AtPickup {
        dwell_timer: f32,
        required_dwell: f32,
    },
    /// Passenger safely on board or cargo secured; ready to navigate to destination.
    Loaded,
    /// Player driving towards destination dropoff location.
    EnRouteToDropoff,
    /// Vehicle stopped within dropoff perimeter; passenger disembarking or cargo unloading.
    AtDropoff {
        dwell_timer: f32,
        required_dwell: f32,
    },
    /// Job successfully completed.
    Completed {
        final_payout: f64,
        xp_gained: u32,
        duration_secs: f64,
        rating_stars: f32,
    },
    /// Job failed due to timeout, total cargo destruction, or abandonment.
    Failed { reason: String },
    /// Job deliberately cancelled by player.
    Abandoned,
}

/// Completion settlement record emitted when a job completes.
#[derive(Debug, Clone, PartialEq)]
pub struct JobCompletionResult {
    pub job_id: String,
    pub career: CareerType,
    pub base_payout: f64,
    pub tip_bonus: f64,
    pub speed_bonus: f64,
    pub final_payout: f64,
    pub xp_gained: u32,
    pub reputation_gained: u32,
    pub rating_stars: f32,
    pub duration_secs: f64,
}

/// Authentic mission contract with verifiable real-road destinations and state machine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub career: CareerType,
    pub title: String,
    pub description: String,
    pub pickup: JobLocation,
    pub dropoff: JobLocation,
    pub route_distance_m: f64,
    pub time_limit_secs: Option<f64>,
    pub elapsed_secs: f64,
    pub base_payout: f64,
    pub xp_reward: u32,
    pub required_tier: ReputationTier,
    pub cargo: CargoOrPassenger,
    pub difficulty: JobDifficulty,
    pub state: JobState,
    pub comfort_score: f32,
    pub cargo_integrity: f32,
}

impl Job {
    /// Creates a new job offer with specified parameters.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        career: CareerType,
        title: impl Into<String>,
        description: impl Into<String>,
        pickup: JobLocation,
        dropoff: JobLocation,
        route_distance_m: f64,
        time_limit_secs: Option<f64>,
        base_payout: f64,
        xp_reward: u32,
        required_tier: ReputationTier,
        cargo: CargoOrPassenger,
        difficulty: JobDifficulty,
    ) -> Self {
        Self {
            id: id.into(),
            career,
            title: title.into(),
            description: description.into(),
            pickup,
            dropoff,
            route_distance_m,
            time_limit_secs,
            elapsed_secs: 0.0,
            base_payout,
            xp_reward,
            required_tier,
            cargo,
            difficulty,
            state: JobState::Available,
            comfort_score: 5.0,   // 5 stars default
            cargo_integrity: 1.0, // 100% integrity default
        }
    }

    /// Evaluates whether the player profile meets rank requirements to accept this contract.
    pub fn can_accept(&self, profile: &PlayerProfile) -> bool {
        self.state == JobState::Available && profile.career_tier(self.career) >= self.required_tier
    }

    /// Accepts this job offer and transitions into the EnRouteToPickup state.
    pub fn accept(
        &mut self,
        profile: &PlayerProfile,
        event_bus: &GameplayEventBus,
    ) -> Result<(), &'static str> {
        if !self.can_accept(profile) {
            return Err("Cannot accept job: rank requirements not met or job unavailable");
        }
        self.state = JobState::Accepted;
        event_bus.emit(GameplayEvent::JobAccepted {
            job_id: self.id.clone(),
            career: self.career,
        });

        // Begin navigation towards pickup
        self.state = JobState::EnRouteToPickup;
        event_bus.emit(GameplayEvent::JobStarted {
            job_id: self.id.clone(),
        });
        Ok(())
    }

    /// Advances the job state machine by one simulation step.
    /// Handles proximity detection, vehicle stopping verification (< 2.0 km/h),
    /// dwell loading timers, deadline countdowns, and completion calculation.
    pub fn update(
        &mut self,
        dt: f32,
        vehicle_coord: Wgs84Coord,
        vehicle_speed_kph: f32,
        event_bus: &GameplayEventBus,
    ) -> Option<JobCompletionResult> {
        // Only active states consume time
        match self.state {
            JobState::EnRouteToPickup
            | JobState::AtPickup { .. }
            | JobState::Loaded
            | JobState::EnRouteToDropoff
            | JobState::AtDropoff { .. } => {
                self.elapsed_secs += dt as f64;

                // Check deadline expiration
                if let Some(limit) = self.time_limit_secs {
                    if self.elapsed_secs > limit {
                        self.state = JobState::Failed {
                            reason: "Contract time limit expired".to_string(),
                        };
                        event_bus.emit(GameplayEvent::JobFailed {
                            job_id: self.id.clone(),
                            career: self.career,
                            reason: "Contract time limit expired".to_string(),
                        });
                        return None;
                    }
                }
            }
            _ => return None,
        }

        // State Machine Step
        match self.state {
            JobState::EnRouteToPickup => {
                if self.pickup.contains(vehicle_coord) {
                    event_bus.emit(GameplayEvent::ZoneEntered {
                        job_id: self.id.clone(),
                        is_pickup: true,
                        zone_name: self.pickup.name.clone(),
                    });
                    self.state = JobState::AtPickup {
                        dwell_timer: 0.0,
                        required_dwell: 2.0, // 2.0 seconds stop required
                    };
                }
            }
            JobState::AtPickup {
                ref mut dwell_timer,
                required_dwell,
            } => {
                if !self.pickup.contains(vehicle_coord) {
                    // Left perimeter before boarding
                    self.state = JobState::EnRouteToPickup;
                } else if vehicle_speed_kph < 2.0 {
                    // Vehicle stopped: advance boarding timer
                    *dwell_timer += dt;
                    let progress = (*dwell_timer / required_dwell).min(1.0);
                    event_bus.emit(GameplayEvent::InteractionProgress {
                        job_id: self.id.clone(),
                        progress_ratio: progress,
                    });

                    if *dwell_timer >= required_dwell {
                        match &self.cargo {
                            CargoOrPassenger::Passenger { name, .. } => {
                                event_bus.emit(GameplayEvent::PassengerBoarded {
                                    job_id: self.id.clone(),
                                    passenger_name: name.clone(),
                                });
                            }
                            CargoOrPassenger::Package {
                                item_name,
                                weight_kg,
                                ..
                            } => {
                                event_bus.emit(GameplayEvent::CargoLoaded {
                                    job_id: self.id.clone(),
                                    cargo_name: item_name.clone(),
                                    weight_kg: *weight_kg,
                                });
                            }
                            CargoOrPassenger::Freight {
                                cargo_type,
                                tonnage,
                                ..
                            } => {
                                event_bus.emit(GameplayEvent::CargoLoaded {
                                    job_id: self.id.clone(),
                                    cargo_name: cargo_type.clone(),
                                    weight_kg: *tonnage * 1000.0,
                                });
                            }
                            CargoOrPassenger::Exploration => {}
                        }
                        self.state = JobState::EnRouteToDropoff;
                    }
                }
            }
            JobState::Loaded => {
                self.state = JobState::EnRouteToDropoff;
            }
            JobState::EnRouteToDropoff => {
                if self.dropoff.contains(vehicle_coord) {
                    event_bus.emit(GameplayEvent::ZoneEntered {
                        job_id: self.id.clone(),
                        is_pickup: false,
                        zone_name: self.dropoff.name.clone(),
                    });
                    self.state = JobState::AtDropoff {
                        dwell_timer: 0.0,
                        required_dwell: 2.0,
                    };
                }
            }
            JobState::AtDropoff {
                ref mut dwell_timer,
                required_dwell,
            } => {
                if !self.dropoff.contains(vehicle_coord) {
                    // Left perimeter prematurely
                    self.state = JobState::EnRouteToDropoff;
                } else if vehicle_speed_kph < 2.0 {
                    // Vehicle stopped: advance unloading timer
                    *dwell_timer += dt;
                    let progress = (*dwell_timer / required_dwell).min(1.0);
                    event_bus.emit(GameplayEvent::InteractionProgress {
                        job_id: self.id.clone(),
                        progress_ratio: progress,
                    });

                    if *dwell_timer >= required_dwell {
                        // Successful completion calculation
                        return Some(self.finalize_completion(event_bus));
                    }
                }
            }
            _ => {}
        }

        None
    }

    /// Evaluates financial payouts, tips, and unlocks on completion.
    fn finalize_completion(&mut self, event_bus: &GameplayEventBus) -> JobCompletionResult {
        // Comfort tip for taxi passengers
        let tip_bonus = if self.career == CareerType::TaxiPassenger {
            let comfort_ratio = (self.comfort_score - 1.0).max(0.0) / 4.0;
            self.base_payout * 0.35 * (comfort_ratio as f64)
        } else {
            0.0
        };

        // Speed bonus if completed well before deadline
        let speed_bonus = if let Some(limit) = self.time_limit_secs {
            if self.elapsed_secs < limit * 0.70 {
                self.base_payout * 0.20 // 20% early delivery bonus
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Cargo integrity deduction for freight
        let final_payout = ((self.base_payout + tip_bonus + speed_bonus)
            * self.difficulty.payout_bonus()
            * (self.cargo_integrity as f64))
            .round();

        let xp_gained = (self.xp_reward as f32 * self.career.xp_multiplier()).round() as u32;
        let rep_gain = match self.difficulty {
            JobDifficulty::Relaxed => 25,
            JobDifficulty::Standard => 50,
            JobDifficulty::Challenging => 100,
            JobDifficulty::Extreme => 250,
        };

        self.state = JobState::Completed {
            final_payout,
            xp_gained,
            duration_secs: self.elapsed_secs,
            rating_stars: self.comfort_score,
        };

        event_bus.emit(GameplayEvent::JobCompleted {
            job_id: self.id.clone(),
            career: self.career,
            payout: final_payout,
            xp: xp_gained,
            rating_stars: self.comfort_score,
        });

        JobCompletionResult {
            job_id: self.id.clone(),
            career: self.career,
            base_payout: self.base_payout,
            tip_bonus,
            speed_bonus,
            final_payout,
            xp_gained,
            reputation_gained: rep_gain,
            rating_stars: self.comfort_score,
            duration_secs: self.elapsed_secs,
        }
    }

    /// Records collision impact penalty on passenger comfort and cargo integrity.
    pub fn record_collision(&mut self, impulse: f32, event_bus: &GameplayEventBus) {
        if impulse > 5.0 {
            // Deduct comfort score (down to minimum 1.0 star)
            self.comfort_score = (self.comfort_score - (impulse * 0.05)).max(1.0);

            // Deduct cargo integrity
            self.cargo_integrity = (self.cargo_integrity - (impulse * 0.02)).max(0.0);

            let repair_cost = (impulse as f64) * 15.0;
            event_bus.emit(GameplayEvent::CollisionImpact {
                impulse,
                repair_cost,
            });

            if self.cargo_integrity <= 0.0 {
                self.state = JobState::Failed {
                    reason: "Cargo destroyed due to excessive collision impact".to_string(),
                };
                event_bus.emit(GameplayEvent::JobFailed {
                    job_id: self.id.clone(),
                    career: self.career,
                    reason: "Cargo destroyed due to excessive collision impact".to_string(),
                });
            }
        }
    }

    /// Records high lateral acceleration / jerk affecting passenger comfort.
    pub fn record_lateral_accel(&mut self, lateral_accel_mps2: f32) {
        if lateral_accel_mps2 > 4.5 {
            // Rough turning / aggressive drift degrades comfort
            self.comfort_score = (self.comfort_score - 0.05).max(1.0);
        }
    }

    /// Abandons the active job.
    pub fn abandon(&mut self, event_bus: &GameplayEventBus) {
        self.state = JobState::Abandoned;
        event_bus.emit(GameplayEvent::JobAbandoned {
            job_id: self.id.clone(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_lifecycle_state_machine() {
        let bus = GameplayEventBus::new();
        let profile = PlayerProfile::default();

        let pickup = JobLocation::new(
            "Vidhana Soudha",
            Wgs84Coord::new(12.9797, 77.5907, 920.0),
            "bengaluru-central",
            25.0,
        );
        let dropoff = JobLocation::new(
            "MG Road",
            Wgs84Coord::new(12.9750, 77.6080, 912.0),
            "bengaluru-central",
            25.0,
        );

        let mut job = Job::new(
            "job-001",
            CareerType::TaxiPassenger,
            "Cubbon Park Commute",
            "Drive VIP passenger to MG Road",
            pickup.clone(),
            dropoff.clone(),
            2000.0,
            Some(300.0),
            120.0,
            100,
            ReputationTier::Novice,
            CargoOrPassenger::Passenger {
                name: "Dr. Rao".to_string(),
                party_size: 1,
                destination_note: "MG Road Metro Gate".to_string(),
            },
            JobDifficulty::Standard,
        );

        assert_eq!(job.state, JobState::Available);
        assert!(job.can_accept(&profile));

        // 1. Accept job
        job.accept(&profile, &bus).unwrap();
        assert_eq!(job.state, JobState::EnRouteToPickup);

        // 2. Drive to pickup and enter zone
        job.update(0.1, pickup.coord, 25.0, &bus);
        assert!(matches!(job.state, JobState::AtPickup { .. }));

        // 3. Stop inside zone (< 2.0 km/h) for 2.0s dwell
        job.update(1.0, pickup.coord, 0.0, &bus);
        assert!(matches!(job.state, JobState::AtPickup { .. }));
        job.update(1.1, pickup.coord, 0.0, &bus);
        assert_eq!(job.state, JobState::EnRouteToDropoff);

        // 4. Drive to dropoff and enter zone
        job.update(0.1, dropoff.coord, 30.0, &bus);
        assert!(matches!(job.state, JobState::AtDropoff { .. }));

        // 5. Stop inside dropoff zone for 2.0s dwell -> Complete!
        let comp1 = job.update(1.0, dropoff.coord, 0.0, &bus);
        assert_eq!(comp1, None);
        let comp2 = job.update(1.1, dropoff.coord, 0.0, &bus);
        assert!(comp2.is_some());
        let result = comp2.unwrap();
        assert_eq!(result.job_id, "job-001");
        assert!(result.final_payout > 100.0);
        assert!(matches!(job.state, JobState::Completed { .. }));
    }
}
