//! Achievement system tracking milestones across all careers and driving mastery.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An individual achievement entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Achievement {
    pub id: String,
    pub title: String,
    pub description: String,
    pub unlocked: bool,
    pub progress: f32,
    pub reward_xp: u32,
    pub reward_credits: f64,
}

/// Registry of all defined game achievements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AchievementRegistry {
    achievements: HashMap<String, Achievement>,
}

impl Default for AchievementRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AchievementRegistry {
    /// Creates the standard set of authentic achievements for the Real Earth Platform.
    pub fn new() -> Self {
        let mut reg = HashMap::new();

        let list = vec![
            Achievement {
                id: "FIRST_FARE".to_string(),
                title: "First Fare".to_string(),
                description: "Complete your first passenger taxi trip safely.".to_string(),
                unlocked: false,
                progress: 0.0,
                reward_xp: 150,
                reward_credits: 200.0,
            },
            Achievement {
                id: "SMOOTH_CHAUFFEUR".to_string(),
                title: "Smooth Chauffeur".to_string(),
                description: "Deliver a passenger with a 5-star comfort rating (zero high-G jerks).".to_string(),
                unlocked: false,
                progress: 0.0,
                reward_xp: 250,
                reward_credits: 350.0,
            },
            Achievement {
                id: "EXPRESS_COURIER".to_string(),
                title: "Express Courier".to_string(),
                description: "Deliver an urgent parcel with at least 50% time margin remaining.".to_string(),
                unlocked: false,
                progress: 0.0,
                reward_xp: 200,
                reward_credits: 300.0,
            },
            Achievement {
                id: "DECCAN_EXPRESS".to_string(),
                title: "Deccan Express".to_string(),
                description: "Complete a heavy freight haul across the Bengaluru-Mysuru Expressway (NH 275).".to_string(),
                unlocked: false,
                progress: 0.0,
                reward_xp: 500,
                reward_credits: 1000.0,
            },
            Achievement {
                id: "CHENNAI_PORT_HAUL".to_string(),
                title: "Maritime Connection".to_string(),
                description: "Deliver industrial cargo to Chennai Port across the NH 48 corridor.".to_string(),
                unlocked: false,
                progress: 0.0,
                reward_xp: 600,
                reward_credits: 1200.0,
            },
            Achievement {
                id: "HYDERABAD_EXPRESS".to_string(),
                title: "Deccan Hauler".to_string(),
                description: "Deliver heavy logistics to HITEC City across the NH 44 corridor and PVNR Expressway.".to_string(),
                unlocked: false,
                progress: 0.0,
                reward_xp: 750,
                reward_credits: 1500.0,
            },
            Achievement {
                id: "CENTURION_KM".to_string(),
                title: "Century Club".to_string(),
                description: "Drive 100 cumulative kilometers on real surveyed roads.".to_string(),
                unlocked: false,
                progress: 0.0,
                reward_xp: 400,
                reward_credits: 500.0,
            },
            Achievement {
                id: "CLEAN_STREAK".to_string(),
                title: "Flawless Pilot".to_string(),
                description: "Drive 15 kilometers continuously without any collision impact.".to_string(),
                unlocked: false,
                progress: 0.0,
                reward_xp: 300,
                reward_credits: 400.0,
            },
            Achievement {
                id: "LANDMARK_EXPLORER".to_string(),
                title: "World Voyager".to_string(),
                description: "Visit at least 5 authentic surveyed landmarks across the world.".to_string(),
                unlocked: false,
                progress: 0.0,
                reward_xp: 350,
                reward_credits: 450.0,
            },
        ];

        for ach in list {
            reg.insert(ach.id.clone(), ach);
        }

        Self { achievements: reg }
    }

    /// Gets an achievement by ID.
    pub fn get(&self, id: &str) -> Option<&Achievement> {
        self.achievements.get(id)
    }

    /// Updates progress towards an achievement, unlocking it if progress reaches 1.0.
    /// Returns Some((xp, credits)) if newly unlocked, or None if already unlocked or still in progress.
    pub fn update_progress(&mut self, id: &str, progress: f32) -> Option<(u32, f64)> {
        if let Some(ach) = self.achievements.get_mut(id) {
            if ach.unlocked {
                return None;
            }
            ach.progress = progress.clamp(0.0, 1.0);
            if ach.progress >= 1.0 {
                ach.unlocked = true;
                return Some((ach.reward_xp, ach.reward_credits));
            }
        }
        None
    }

    /// Unlocks an achievement directly by ID.
    pub fn unlock(&mut self, id: &str) -> Option<(u32, f64)> {
        self.update_progress(id, 1.0)
    }

    /// Total number of achievements.
    pub fn total_count(&self) -> usize {
        self.achievements.len()
    }

    /// Total unlocked achievements.
    pub fn unlocked_count(&self) -> usize {
        self.achievements.values().filter(|a| a.unlocked).count()
    }

    /// All achievements.
    pub fn all(&self) -> &HashMap<String, Achievement> {
        &self.achievements
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_achievements_unlock_flow() {
        let mut reg = AchievementRegistry::new();
        assert_eq!(reg.unlocked_count(), 0);

        let res1 = reg.update_progress("FIRST_FARE", 0.5);
        assert_eq!(res1, None);
        assert!(!reg.get("FIRST_FARE").unwrap().unlocked);

        let res2 = reg.update_progress("FIRST_FARE", 1.0);
        assert_eq!(res2, Some((150, 200.0)));
        assert!(reg.get("FIRST_FARE").unwrap().unlocked);
        assert_eq!(reg.unlocked_count(), 1);

        // Subsequent updates do not re-reward
        let res3 = reg.update_progress("FIRST_FARE", 1.0);
        assert_eq!(res3, None);
    }
}
