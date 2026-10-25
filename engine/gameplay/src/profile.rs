//! Player profile, overall career level curve, and persistent progression state.

use crate::achievements::AchievementRegistry;
use crate::career::{CareerType, ReputationTier};
use crate::economy::{TransactionCategory, Wallet};
use crate::garage::VehicleGarage;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Persistent profile containing complete player progression, stats, and assets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerProfile {
    pub player_id: String,
    pub name: String,
    pub level: u32,
    pub xp: u32,
    pub wallet: Wallet,
    pub reputation: HashMap<CareerType, u32>,
    pub total_distance_km: f64,
    pub clean_streak_km: f64,
    pub completed_jobs: usize,
    pub failed_jobs: usize,
    pub garage: VehicleGarage,
    pub achievements: AchievementRegistry,
    pub discovered_landmarks: HashSet<String>,
}

impl Default for PlayerProfile {
    fn default() -> Self {
        Self::new("pilot-001", "Earth Transporter")
    }
}

impl PlayerProfile {
    /// Creates a fresh player profile with default starter assets.
    pub fn new(player_id: impl Into<String>, name: impl Into<String>) -> Self {
        let mut rep = HashMap::new();
        rep.insert(CareerType::FreeRoam, 0);
        rep.insert(CareerType::TaxiPassenger, 0);
        rep.insert(CareerType::TransitBus, 0);
        rep.insert(CareerType::PackageDelivery, 0);
        rep.insert(CareerType::HeavyFreight, 0);
        rep.insert(CareerType::RailEngineer, 0);
        rep.insert(CareerType::FerryCaptain, 0);
        rep.insert(CareerType::AviationPilot, 0);

        Self {
            player_id: player_id.into(),
            name: name.into(),
            level: 1,
            xp: 0,
            wallet: Wallet::default(),
            reputation: rep,
            total_distance_km: 0.0,
            clean_streak_km: 0.0,
            completed_jobs: 0,
            failed_jobs: 0,
            garage: VehicleGarage::default(),
            achievements: AchievementRegistry::default(),
            discovered_landmarks: HashSet::new(),
        }
    }

    /// XP required to advance from the given level to the next level.
    pub fn xp_threshold(level: u32) -> u32 {
        level * 500
    }

    /// Adds XP to the profile, handling level-ups.
    /// Returns Some(new_level) if a level-up occurred.
    pub fn add_xp(&mut self, amount: u32) -> Option<u32> {
        self.xp += amount;
        let mut leveled_up = false;
        while self.xp >= Self::xp_threshold(self.level) {
            self.xp -= Self::xp_threshold(self.level);
            self.level += 1;
            leveled_up = true;
        }
        if leveled_up {
            Some(self.level)
        } else {
            None
        }
    }

    /// Adds career-specific reputation points and returns (new_total, new_tier).
    pub fn add_reputation(&mut self, career: CareerType, points: u32) -> (u32, ReputationTier) {
        let current = self.reputation.entry(career).or_insert(0);
        *current += points;
        let new_total = *current;
        (new_total, ReputationTier::from_points(new_total))
    }

    /// Retrieves current reputation points for a career.
    pub fn career_reputation(&self, career: CareerType) -> u32 {
        self.reputation.get(&career).copied().unwrap_or(0)
    }

    /// Retrieves current reputation tier for a career.
    pub fn career_tier(&self, career: CareerType) -> ReputationTier {
        ReputationTier::from_points(self.career_reputation(career))
    }

    /// Records newly traversed road distance in kilometers.
    pub fn record_distance(&mut self, distance_km: f64) {
        if distance_km > 0.0 {
            self.total_distance_km += distance_km;
            self.clean_streak_km += distance_km;

            if let Some(active) = self.garage.active_vehicle_mut() {
                active.odometer_km += distance_km;
            }

            // Check distance milestone achievements
            if self.total_distance_km >= 100.0 {
                if let Some((xp, credits)) = self.achievements.unlock("CENTURION_KM") {
                    self.add_xp(xp);
                    let _ = self.wallet.credit(
                        credits,
                        TransactionCategory::ExplorationReward,
                        "Achievement: Century Club",
                        0,
                    );
                }
            }
            if self.clean_streak_km >= 15.0 {
                if let Some((xp, credits)) = self.achievements.unlock("CLEAN_STREAK") {
                    self.add_xp(xp);
                    let _ = self.wallet.credit(
                        credits,
                        TransactionCategory::ExplorationReward,
                        "Achievement: Flawless Pilot",
                        0,
                    );
                }
            }
        }
    }

    /// Resets clean streak upon vehicle collision.
    pub fn record_collision(&mut self) {
        self.clean_streak_km = 0.0;
    }

    /// Records landmark discovery. Returns true if landmark was newly discovered.
    pub fn discover_landmark(&mut self, landmark_name: impl Into<String>) -> bool {
        let name = landmark_name.into();
        let is_new = self.discovered_landmarks.insert(name);
        if is_new && self.discovered_landmarks.len() >= 5 {
            if let Some((xp, credits)) = self.achievements.unlock("LANDMARK_EXPLORER") {
                self.add_xp(xp);
                let _ = self.wallet.credit(
                    credits,
                    TransactionCategory::ExplorationReward,
                    "Achievement: World Voyager",
                    0,
                );
            }
        }
        is_new
    }

    /// Records successful job completion.
    pub fn record_job_completion(
        &mut self,
        career: CareerType,
        payout: f64,
        xp: u32,
        rep_gain: u32,
        timestamp_secs: u64,
    ) {
        self.completed_jobs += 1;
        let _ = self.wallet.credit(
            payout,
            TransactionCategory::JobPayout,
            format!("Completed {} job", career.display_name()),
            timestamp_secs,
        );
        self.add_xp(xp);
        self.add_reputation(career, rep_gain);
    }

    /// Records job failure.
    pub fn record_job_failure(&mut self) {
        self.failed_jobs += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_xp_and_level_ups() {
        let mut profile = PlayerProfile::default();
        assert_eq!(profile.level, 1);
        assert_eq!(profile.xp, 0);

        // Level 1 threshold is 500 XP
        let lvl1 = profile.add_xp(300);
        assert_eq!(lvl1, None);
        assert_eq!(profile.level, 1);
        assert_eq!(profile.xp, 300);

        let lvl2 = profile.add_xp(300); // 600 total -> level 2 with 100 XP remaining
        assert_eq!(lvl2, Some(2));
        assert_eq!(profile.level, 2);
        assert_eq!(profile.xp, 100);
    }

    #[test]
    fn test_profile_reputation_and_tiers() {
        let mut profile = PlayerProfile::default();
        assert_eq!(
            profile.career_tier(CareerType::TaxiPassenger),
            ReputationTier::Novice
        );

        let (_, tier) = profile.add_reputation(CareerType::TaxiPassenger, 150);
        assert_eq!(tier, ReputationTier::Professional);
        assert_eq!(
            profile.career_tier(CareerType::TaxiPassenger),
            ReputationTier::Professional
        );
    }
}
