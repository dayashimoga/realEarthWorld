//! Modern responsive HUD data structures for minimal-click in-game UI.

use crate::career::CareerType;
use crate::jobs::{Job, JobDifficulty, JobState};
use crate::profile::PlayerProfile;
use realearth_shared::coordinates::Wgs84Coord;
use serde::{Deserialize, Serialize};

/// Real-time active mission and navigation objective overlay.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActiveObjectiveHud {
    pub active_job_id: Option<String>,
    pub career_title: String,
    pub instruction_text: String,
    pub target_location_name: String,
    pub distance_to_target_meters: f64,
    pub eta_seconds: Option<f64>,
    pub time_limit_remaining_secs: Option<f64>,
    pub dwell_progress_ratio: Option<f32>,
    pub comfort_rating_stars: f32,
    pub cargo_integrity_pct: f32,
    pub is_in_interaction_zone: bool,
}

/// Real-time player financial wallet and earnings status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EarningsHud {
    pub balance: f64,
    pub currency_symbol: String,
    pub potential_payout: f64,
    pub clean_streak_km: f64,
}

/// Compact summary card for an available job in the dispatch browser.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JobOfferSummary {
    pub job_id: String,
    pub title: String,
    pub career: CareerType,
    pub pickup_name: String,
    pub dropoff_name: String,
    pub route_distance_km: f64,
    pub payout: f64,
    pub xp: u32,
    pub difficulty: JobDifficulty,
    pub can_accept: bool,
}

/// Dispatch browser listing available regional jobs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JobBrowserHud {
    pub offers: Vec<JobOfferSummary>,
    pub selected_index: usize,
}

/// Debrief summary dialog presented upon contract completion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DebriefModal {
    pub job_id: String,
    pub career_title: String,
    pub base_payout: f64,
    pub tip_bonus: f64,
    pub speed_bonus: f64,
    pub net_payout: f64,
    pub xp_gained: u32,
    pub reputation_gained: u32,
    pub comfort_rating_stars: f32,
    pub unlocked_achievements: Vec<String>,
}

/// High-level synthesizer extracting presentation-ready HUD state.
pub struct GameplayHudModel;

impl GameplayHudModel {
    /// Generates the active objective HUD from active job and vehicle metrics.
    pub fn build_objective_hud(
        active_job: Option<&Job>,
        vehicle_coord: Wgs84Coord,
        vehicle_speed_kph: f32,
    ) -> ActiveObjectiveHud {
        if let Some(job) = active_job {
            let (target_name, dist_m, is_in_zone, dwell_ratio, instruction) = match &job.state {
                JobState::EnRouteToPickup => {
                    let d = job.pickup.distance_to(vehicle_coord);
                    (
                        job.pickup.name.clone(),
                        d,
                        job.pickup.contains(vehicle_coord),
                        None,
                        format!("Drive to Pickup: {}", job.pickup.name),
                    )
                }
                JobState::AtPickup {
                    dwell_timer,
                    required_dwell,
                } => {
                    let ratio = (*dwell_timer / *required_dwell).min(1.0);
                    let instr = if vehicle_speed_kph > 2.0 {
                        "STOP VEHICLE TO LOAD PASSENGER / CARGO".to_string()
                    } else {
                        format!("Boarding / Loading: {:.0}%", ratio * 100.0)
                    };
                    (job.pickup.name.clone(), 0.0, true, Some(ratio), instr)
                }
                JobState::Loaded | JobState::EnRouteToDropoff => {
                    let d = job.dropoff.distance_to(vehicle_coord);
                    (
                        job.dropoff.name.clone(),
                        d,
                        job.dropoff.contains(vehicle_coord),
                        None,
                        format!("Deliver to Destination: {}", job.dropoff.name),
                    )
                }
                JobState::AtDropoff {
                    dwell_timer,
                    required_dwell,
                } => {
                    let ratio = (*dwell_timer / *required_dwell).min(1.0);
                    let instr = if vehicle_speed_kph > 2.0 {
                        "STOP VEHICLE TO UNLOAD PASSENGER / CARGO".to_string()
                    } else {
                        format!("Disembarking / Unloading: {:.0}%", ratio * 100.0)
                    };
                    (job.dropoff.name.clone(), 0.0, true, Some(ratio), instr)
                }
                JobState::Completed { .. } => (
                    "Destination Reached".to_string(),
                    0.0,
                    false,
                    None,
                    "Job Completed Successfully!".to_string(),
                ),
                JobState::Failed { reason } => (
                    "Mission Failed".to_string(),
                    0.0,
                    false,
                    None,
                    format!("Failed: {}", reason),
                ),
                JobState::Abandoned => (
                    "Mission Abandoned".to_string(),
                    0.0,
                    false,
                    None,
                    "Contract Abandoned".to_string(),
                ),
                _ => (
                    "Standby".to_string(),
                    0.0,
                    false,
                    None,
                    "Awaiting Dispatch".to_string(),
                ),
            };

            let eta = if vehicle_speed_kph > 5.0 {
                let speed_mps = (vehicle_speed_kph as f64) * (1000.0 / 3600.0);
                Some(dist_m / speed_mps)
            } else {
                None
            };

            let remaining_limit = job
                .time_limit_secs
                .map(|lim| (lim - job.elapsed_secs).max(0.0));

            ActiveObjectiveHud {
                active_job_id: Some(job.id.clone()),
                career_title: job.career.display_name().to_string(),
                instruction_text: instruction,
                target_location_name: target_name,
                distance_to_target_meters: dist_m,
                eta_seconds: eta,
                time_limit_remaining_secs: remaining_limit,
                dwell_progress_ratio: dwell_ratio,
                comfort_rating_stars: job.comfort_score,
                cargo_integrity_pct: job.cargo_integrity * 100.0,
                is_in_interaction_zone: is_in_zone,
            }
        } else {
            ActiveObjectiveHud {
                active_job_id: None,
                career_title: "Free Roam & Exploration".to_string(),
                instruction_text: "Free Roam Mode — Press [J] to Browse Available Regional Jobs"
                    .to_string(),
                target_location_name: "World City".to_string(),
                distance_to_target_meters: 0.0,
                eta_seconds: None,
                time_limit_remaining_secs: None,
                dwell_progress_ratio: None,
                comfort_rating_stars: 5.0,
                cargo_integrity_pct: 100.0,
                is_in_interaction_zone: false,
            }
        }
    }

    /// Generates earnings HUD from player profile and active job.
    pub fn build_earnings_hud(profile: &PlayerProfile, active_job: Option<&Job>) -> EarningsHud {
        let potential = active_job.map(|j| j.base_payout).unwrap_or(0.0);
        EarningsHud {
            balance: profile.wallet.balance(),
            currency_symbol: "₹".to_string(),
            potential_payout: potential,
            clean_streak_km: profile.clean_streak_km,
        }
    }

    /// Generates job browser HUD listing all available regional offers.
    pub fn build_browser_hud(
        available_jobs: &[Job],
        profile: &PlayerProfile,
        selected_index: usize,
    ) -> JobBrowserHud {
        let offers = available_jobs
            .iter()
            .map(|j| JobOfferSummary {
                job_id: j.id.clone(),
                title: j.title.clone(),
                career: j.career,
                pickup_name: j.pickup.name.clone(),
                dropoff_name: j.dropoff.name.clone(),
                route_distance_km: j.route_distance_m / 1000.0,
                payout: j.base_payout,
                xp: j.xp_reward,
                difficulty: j.difficulty,
                can_accept: j.can_accept(profile),
            })
            .collect();

        JobBrowserHud {
            offers,
            selected_index,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::destinations::SurveyedDestinations;
    use crate::jobs::CargoOrPassenger;

    #[test]
    fn test_objective_hud_generation() {
        let profile = PlayerProfile::default();
        let pickup = SurveyedDestinations::bengaluru_vidhana_soudha();
        let dropoff = SurveyedDestinations::bengaluru_mg_road();

        let job = Job::new(
            "test-hud-01",
            CareerType::TaxiPassenger,
            "VIP Ride",
            "Commute",
            pickup.clone(),
            dropoff,
            2000.0,
            Some(300.0),
            150.0,
            100,
            crate::career::ReputationTier::Novice,
            CargoOrPassenger::Exploration,
            JobDifficulty::Standard,
        );

        let vehicle_coord = Wgs84Coord::new(12.9790, 77.5900, 920.0);
        let hud = GameplayHudModel::build_objective_hud(Some(&job), vehicle_coord, 40.0);

        assert_eq!(hud.active_job_id, Some("test-hud-01".to_string()));
        assert!(
            hud.instruction_text.contains("Standby") || hud.instruction_text.contains("Awaiting")
        );

        let earnings = GameplayHudModel::build_earnings_hud(&profile, Some(&job));
        assert_eq!(earnings.balance, profile.wallet.balance());
        assert_eq!(earnings.potential_payout, 150.0);
    }
}
