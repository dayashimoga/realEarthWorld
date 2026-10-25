//! Career types, reputation hierarchy, and career-specific attributes.

use serde::{Deserialize, Serialize};

/// Playable careers available in the Real Earth World Platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CareerType {
    /// Free exploration, landmark discovery, and scenic driving.
    FreeRoam,
    /// Point-to-point passenger transport with ride comfort scoring.
    TaxiPassenger,
    /// Scheduled urban low-floor transit bus services with designated passenger stops.
    TransitBus,
    /// Time-sensitive parcel and express deliveries between urban logistics depots.
    PackageDelivery,
    /// Long-distance commercial logistics haulage across intercity transport corridors.
    HeavyFreight,
    /// Intercity and regional commuter train operations adhering to track signals and stations.
    RailEngineer,
    /// High-speed passenger catamaran ferry navigation across bays, straits, and waterways.
    FerryCaptain,
    /// Commercial airline and general aviation flight operations between regional airfields.
    AviationPilot,
}

impl CareerType {
    /// Human-readable display title for this career.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::FreeRoam => "Free Roam & Exploration",
            Self::TaxiPassenger => "Urban Taxi & Chauffeur",
            Self::TransitBus => "Municipal Transit Bus Lines",
            Self::PackageDelivery => "Express Courier & Delivery",
            Self::HeavyFreight => "Intercity Freight Logistics",
            Self::RailEngineer => "Passenger & Freight Railroad Engineer",
            Self::FerryCaptain => "Maritime Passenger Ferry Captain",
            Self::AviationPilot => "Commercial & General Aviation Pilot",
        }
    }

    /// Descriptive summary of the career responsibilities.
    pub fn description(&self) -> &'static str {
        match self {
            Self::FreeRoam => "Freely explore authentic surveyed world cities, discover real architectural landmarks, and earn distance exploration milestones.",
            Self::TaxiPassenger => "Transport authentic passengers across surveyed city streets, maintaining smooth driving and zero-collision comfort for generous customer tips.",
            Self::TransitBus => "Operate 40ft low-floor transit buses along scheduled municipal routes, timing arrivals, boarding commuters at curbside shelters, and executing smooth pneumatic stops.",
            Self::PackageDelivery => "Dispatch fragile parcels and high-priority packages to businesses within tight delivery windows across metropolitan street networks.",
            Self::HeavyFreight => "Haul massive tonnage across regional expressways and transport corridors (NH 275, NH 48, NH 44, I-80, I-5) with 53ft semitrailers while obeying legal highway safety limits.",
            Self::RailEngineer => "Command diesel-electric locomotives and bi-level commuter coaches, managing tractive throttle, Westinghouse air brakes, track switch turnouts, and terminal platform stops.",
            Self::FerryCaptain => "Navigate high-speed catamaran passenger ferries across open bay shipping fairways, combating currents and berthing at passenger terminal slips.",
            Self::AviationPilot => "Fly turbofan airliners and general aviation monoplanes along airway waypoints, executing instrument ILS approaches, runway takeoffs, and crosswind landings.",
        }
    }

    /// Base payout multiplier for completed missions in this career.
    pub fn payout_multiplier(&self) -> f64 {
        match self {
            Self::FreeRoam => 1.0,
            Self::TaxiPassenger => 1.25,
            Self::TransitBus => 1.40,
            Self::PackageDelivery => 1.50,
            Self::HeavyFreight => 2.20,
            Self::RailEngineer => 2.40,
            Self::FerryCaptain => 1.80,
            Self::AviationPilot => 2.80,
        }
    }

    /// Base experience multiplier for completed missions in this career.
    pub fn xp_multiplier(&self) -> f32 {
        match self {
            Self::FreeRoam => 1.0,
            Self::TaxiPassenger => 1.2,
            Self::TransitBus => 1.35,
            Self::PackageDelivery => 1.4,
            Self::HeavyFreight => 2.0,
            Self::RailEngineer => 2.2,
            Self::FerryCaptain => 1.7,
            Self::AviationPilot => 2.5,
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
