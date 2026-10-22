//! Roadside recovery and emergency towing service.

use crate::economy::TransactionCategory;
use crate::events::{GameplayEvent, GameplayEventBus};
use crate::profile::PlayerProfile;
use realearth_geospatial::wgs84_to_ecef;
use realearth_shared::coordinates::Wgs84Coord;
use thiserror::Error;

/// Errors arising during vehicle roadside recovery.
#[derive(Debug, Error, PartialEq)]
pub enum RecoveryError {
    #[error("No surveyed road nodes available within recovery service radius")]
    NoRoadNearby,
}

/// Resulting status after emergency vehicle recovery.
#[derive(Debug, Clone, PartialEq)]
pub struct RecoveryOutcome {
    pub recovered_location: Wgs84Coord,
    pub towing_fee: f64,
    pub new_wallet_balance: f64,
}

/// Roadside recovery dispatcher.
pub struct RecoveryDispatcher;

impl RecoveryDispatcher {
    /// Dispatches emergency towing service to recover an immobilized, flipped, or lost vehicle.
    /// Safely repositions vehicle on the nearest surveyed road node and bills the player's wallet.
    pub fn recover_vehicle(
        current_pos: Wgs84Coord,
        candidate_roads: &[Wgs84Coord],
        profile: &mut PlayerProfile,
        towing_fee: f64,
        event_bus: &GameplayEventBus,
        timestamp_secs: u64,
    ) -> Result<RecoveryOutcome, RecoveryError> {
        if candidate_roads.is_empty() {
            return Err(RecoveryError::NoRoadNearby);
        }

        // Find nearest surveyed road node in 3D ECEF
        let veh_ecef = wgs84_to_ecef(current_pos);
        let mut nearest_coord = candidate_roads[0];
        let mut min_dist_sq = f64::MAX;

        for &road_coord in candidate_roads {
            let road_ecef = wgs84_to_ecef(road_coord);
            let dx = road_ecef.x - veh_ecef.x;
            let dy = road_ecef.y - veh_ecef.y;
            let dz = road_ecef.z - veh_ecef.z;
            let d2 = dx * dx + dy * dy + dz * dz;
            if d2 < min_dist_sq {
                min_dist_sq = d2;
                nearest_coord = road_coord;
            }
        }

        // Bill towing fee: if wallet cannot afford, credit emergency loan
        let actual_fee = if profile.wallet.can_afford(towing_fee) {
            let _ = profile.wallet.debit(
                towing_fee,
                TransactionCategory::RoadsideRecovery,
                "Emergency Roadside Towing Service",
                timestamp_secs,
            );
            towing_fee
        } else {
            // Emergency municipal subsidy for stranded drivers
            let payable = profile.wallet.balance();
            if payable > 0.0 {
                let _ = profile.wallet.debit(
                    payable,
                    TransactionCategory::RoadsideRecovery,
                    "Emergency Roadside Assistance (Partial)",
                    timestamp_secs,
                );
            }
            payable
        };

        // Reset vehicle condition slightly
        if let Some(active) = profile.garage.active_vehicle_mut() {
            active.condition_pct = active.condition_pct.max(75.0);
        }

        event_bus.emit(GameplayEvent::VehicleRecovered {
            cost: actual_fee,
            new_location: nearest_coord,
        });

        Ok(RecoveryOutcome {
            recovered_location: nearest_coord,
            towing_fee: actual_fee,
            new_wallet_balance: profile.wallet.balance(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_teleports_to_nearest_surveyed_node() {
        let bus = GameplayEventBus::new();
        let mut profile = PlayerProfile::default();
        let initial_balance = profile.wallet.balance();

        let stranded_pos = Wgs84Coord::new(12.9800, 77.5900, 920.0);
        let road1 = Wgs84Coord::new(12.9797, 77.5907, 920.0); // Close
        let road2 = Wgs84Coord::new(12.9750, 77.6080, 912.0); // Distant

        let res = RecoveryDispatcher::recover_vehicle(
            stranded_pos,
            &[road1, road2],
            &mut profile,
            150.0,
            &bus,
            100,
        )
        .unwrap();

        assert_eq!(res.recovered_location, road1);
        assert_eq!(res.towing_fee, 150.0);
        assert_eq!(profile.wallet.balance(), initial_balance - 150.0);
        assert_eq!(bus.event_count(), 1);
    }
}
