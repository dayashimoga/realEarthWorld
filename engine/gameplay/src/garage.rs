//! Vehicle ownership, garage management, and vehicle progression upgrades.

use serde::{Deserialize, Serialize};

/// Vehicle functional classifications for specific gameplay roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VehicleCategory {
    /// Versatile entry-level car suitable for free roam and basic passenger jobs.
    EconomySedan,
    /// Premium sedan with soundproofing and plush suspension for VIP taxi transport.
    LuxuryCab,
    /// Enclosed delivery van designed for fast urban parcel logistics.
    CargoVan,
    /// High-torque commercial prime mover hauling heavy intercity freight.
    HeavyTruck,
}

impl VehicleCategory {
    /// Human-readable title.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::EconomySedan => "Standard City Sedan",
            Self::LuxuryCab => "Executive Luxury Chauffeur Cab",
            Self::CargoVan => "Urban Express Delivery Van",
            Self::HeavyTruck => "Heavy Commercial Freight Hauler",
        }
    }

    /// Base purchase price in economy credits.
    pub fn purchase_price(&self) -> f64 {
        match self {
            Self::EconomySedan => 1200.0,
            Self::LuxuryCab => 3500.0,
            Self::CargoVan => 2800.0,
            Self::HeavyTruck => 8500.0,
        }
    }

    /// Maximum cargo payload weight in kilograms.
    pub fn max_payload_kg(&self) -> f32 {
        match self {
            Self::EconomySedan => 250.0,
            Self::LuxuryCab => 200.0,
            Self::CargoVan => 1500.0,
            Self::HeavyTruck => 28000.0,
        }
    }
}

/// An individual vehicle owned by the player.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OwnedVehicle {
    pub id: String,
    pub model_name: String,
    pub category: VehicleCategory,
    pub engine_stage: u8,
    pub handling_stage: u8,
    pub comfort_stage: u8,
    pub condition_pct: f32,
    pub odometer_km: f64,
}

impl OwnedVehicle {
    /// Creates a brand new vehicle of the given category with factory defaults.
    pub fn new(
        id: impl Into<String>,
        model_name: impl Into<String>,
        category: VehicleCategory,
    ) -> Self {
        Self {
            id: id.into(),
            model_name: model_name.into(),
            category,
            engine_stage: 0,
            handling_stage: 0,
            comfort_stage: 0,
            condition_pct: 100.0,
            odometer_km: 0.0,
        }
    }

    /// Power multiplier derived from engine upgrade stage (0 to 3).
    pub fn power_multiplier(&self) -> f32 {
        1.0 + (self.engine_stage as f32 * 0.15)
    }

    /// Handling grip multiplier derived from handling upgrade stage (0 to 3).
    pub fn grip_multiplier(&self) -> f32 {
        1.0 + (self.handling_stage as f32 * 0.12)
    }

    /// Comfort multiplier dampening lateral jerk for passenger rating (0 to 3).
    pub fn comfort_damping_multiplier(&self) -> f32 {
        1.0 + (self.comfort_stage as f32 * 0.20)
    }
}

/// Player vehicle garage holding owned fleet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VehicleGarage {
    vehicles: Vec<OwnedVehicle>,
    active_index: usize,
}

impl Default for VehicleGarage {
    fn default() -> Self {
        // Starter vehicle provided free on registration
        let starter = OwnedVehicle::new(
            "starter-sedan-01",
            "Terra Classic Sedan",
            VehicleCategory::EconomySedan,
        );
        Self {
            vehicles: vec![starter],
            active_index: 0,
        }
    }
}

impl VehicleGarage {
    /// Creates a garage initialized with an existing vehicle list.
    pub fn new(vehicles: Vec<OwnedVehicle>, active_index: usize) -> Self {
        let active = if vehicles.is_empty() {
            0
        } else {
            active_index.min(vehicles.len() - 1)
        };
        Self {
            vehicles,
            active_index: active,
        }
    }

    /// Reference to currently active vehicle.
    pub fn active_vehicle(&self) -> Option<&OwnedVehicle> {
        self.vehicles.get(self.active_index)
    }

    /// Mutable reference to currently active vehicle.
    pub fn active_vehicle_mut(&mut self) -> Option<&mut OwnedVehicle> {
        self.vehicles.get_mut(self.active_index)
    }

    /// Selects active vehicle by index.
    pub fn select_vehicle(&mut self, index: usize) -> bool {
        if index < self.vehicles.len() {
            self.active_index = index;
            true
        } else {
            false
        }
    }

    /// Adds a newly purchased vehicle to garage.
    pub fn add_vehicle(&mut self, vehicle: OwnedVehicle) -> usize {
        self.vehicles.push(vehicle);
        self.vehicles.len() - 1
    }

    /// Returns all vehicles in garage.
    pub fn vehicles(&self) -> &[OwnedVehicle] {
        &self.vehicles
    }

    /// Active index within garage fleet.
    pub fn active_index(&self) -> usize {
        self.active_index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_garage_vehicle_selection_and_upgrades() {
        let mut garage = VehicleGarage::default();
        assert_eq!(garage.vehicles().len(), 1);
        assert_eq!(
            garage.active_vehicle().unwrap().category,
            VehicleCategory::EconomySedan
        );

        let van = OwnedVehicle::new("van-02", "Express Cargo Van", VehicleCategory::CargoVan);
        let van_idx = garage.add_vehicle(van);
        assert_eq!(van_idx, 1);
        assert!(garage.select_vehicle(van_idx));
        assert_eq!(
            garage.active_vehicle().unwrap().category,
            VehicleCategory::CargoVan
        );

        let active = garage.active_vehicle_mut().unwrap();
        assert_eq!(active.engine_stage, 0);
        assert_eq!(active.power_multiplier(), 1.0);
        active.engine_stage = 2;
        assert_eq!(active.power_multiplier(), 1.30);
    }
}
