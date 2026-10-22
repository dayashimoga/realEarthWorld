//! Career types, reputation hierarchy, and career-specific attributes.

use serde::{Deserialize, Serialize};

/// Playable careers available in the Real Earth World Platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CareerType {
    /// Free exploration, landmark discovery, and scenic driving.
    FreeRoam,
    /// Point-to-point passenger transport with ride comfort scoring.
    TaxiPassenger,
    /// Time-sensitive parcel and express deliveries between urban logistics depots.
    PackageDelivery,
    /// Long-distance commercial logistics haulage across intercity transport corridors.
    HeavyFreight,
}

impl CareerType {
    /// Human-readable display title for this career.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::FreeRoam => "Free Roam & Exploration",
            Self::TaxiPassenger => "Urban Taxi & Chauffeur",
            Self::PackageDelivery => "Express Courier & Delivery",
            Self::HeavyFreight => "Intercity Freight Logistics",
        }
    }

    /// Descriptive summary of the career responsibilities.
    pub fn description(&self) -> &'static str {
        match self {
            Self::FreeRoam => "Freely explore authentic surveyed world cities, discover real architectural landmarks, and earn distance exploration milestones.",
            Self::TaxiPassenger => "Transport authentic passengers across surveyed city streets, maintaining smooth driving and zero-collision comfort for generous customer tips.",
            Self::PackageDelivery => "Dispatch fragile parcels and high-priority packages to businesses within tight delivery windows across metropolitan street networks.",
            Self::HeavyFreight => "Haul massive tonnage across regional expressways and transport corridors (NH 275, NH 48, NH 44) while obeying legal highway safety limits.",
        }
    }

    /// Base payout multiplier for completed missions in this career.
    pub fn payout_multiplier(&self) -> f64 {
        match self {
            Self::FreeRoam => 1.0,
            Self::TaxiPassenger => 1.25,
            Self::PackageDelivery => 1.5,
            Self::HeavyFreight => 2.2,
        }
    }

    /// Base experience multiplier for completed missions in this career.
    pub fn xp_multiplier(&self) -> f32 {
        match self {
            Self::FreeRoam => 1.0,
            Self::TaxiPassenger => 1.2,
            Self::PackageDelivery => 1.4,
            Self::HeavyFreight => 2.0,
        }
    }
}

/// Career progression and reputation tier ranking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ReputationTier {
    /// Entry-level rank with basic job access.
    Novice = 0,
    /// Proven professional with access to express routes and higher fares.
    Professional = 1,
    /// Renowned expert handling sensitive, high-value, and long-range contracts.
    Expert = 2,
    /// Elite master transporter commanding top commercial rates and luxury clients.
    Master = 3,
}

impl ReputationTier {
    /// Returns the minimum reputation points required to attain this tier.
    pub fn min_points(&self) -> u32 {
        match self {
            Self::Novice => 0,
            Self::Professional => 100,
            Self::Expert => 500,
            Self::Master => 1500,
        }
    }

    /// Determines the reputation tier for a given point score.
    pub fn from_points(points: u32) -> Self {
        if points >= Self::Master.min_points() {
            Self::Master
        } else if points >= Self::Expert.min_points() {
            Self::Expert
        } else if points >= Self::Professional.min_points() {
            Self::Professional
        } else {
            Self::Novice
        }
    }

    /// Human-readable title for this tier.
    pub fn title(&self) -> &'static str {
        match self {
            Self::Novice => "Novice",
            Self::Professional => "Professional",
            Self::Expert => "Expert",
            Self::Master => "Master Transporter",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reputation_tier_thresholds() {
        assert_eq!(ReputationTier::from_points(0), ReputationTier::Novice);
        assert_eq!(ReputationTier::from_points(99), ReputationTier::Novice);
        assert_eq!(
            ReputationTier::from_points(100),
            ReputationTier::Professional
        );
        assert_eq!(
            ReputationTier::from_points(499),
            ReputationTier::Professional
        );
        assert_eq!(ReputationTier::from_points(500), ReputationTier::Expert);
        assert_eq!(ReputationTier::from_points(1499), ReputationTier::Expert);
        assert_eq!(ReputationTier::from_points(1500), ReputationTier::Master);
        assert_eq!(ReputationTier::from_points(5000), ReputationTier::Master);
    }
}
