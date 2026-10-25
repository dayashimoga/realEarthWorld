//! Vehicle ownership, garage management, and vehicle progression upgrades.

use serde::{Deserialize, Serialize};

/// Vehicle functional classifications for specific gameplay roles across multimodal transport.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VehicleCategory {
    /// High-performance GT sports coupe with tuned suspension and aerodynamic downforce.
    SportsCoupe,
    /// Versatile entry-level car suitable for free roam and basic passenger jobs.
    EconomySedan,
    /// Premium sedan or medallion cab with plush suspension for VIP taxi transport.
    LuxuryCab,
    /// All-terrain AWD utility vehicle with rugged ground clearance.
    Suv,
    /// Agile supersport motorcycle with dynamic cornering roll lean.
    Motorcycle,
    /// Enclosed delivery van designed for fast urban parcel logistics.
    CargoVan,
    /// High-torque commercial prime mover hauling heavy intercity freight with 53ft semitrailer.
    HeavyTruck,
    /// 40ft low-floor city transit bus with pneumatic doors and passenger seating rows.
    TransitBus,
    /// Heavy diesel-electric locomotive with flanged steel wheels and tractive physics.
    Locomotive,
    /// Bi-level commuter rail coach with dual passenger decks and gangways.
    PassengerCoach,
    /// High-speed catamaran passenger ferry with twin sponson hulls and waterjets.
    FerryVessel,
    /// Commercial narrow-body twin-turbofan airliner with swept wings.
    Airliner,
    /// General aviation single-engine high-wing monoplane with propeller propulsion.
    LightAircraft,
}

impl VehicleCategory {
    /// Human-readable title.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::SportsCoupe => "Apex GT Sports Coupe",
            Self::EconomySedan => "Executive City Sedan",
            Self::LuxuryCab => "Metro Medallion Taxi / Chauffeur",
            Self::Suv => "Overland Explorer All-Terrain SUV",
            Self::Motorcycle => "Hypersport RR1000 Superbike",
            Self::CargoVan => "Urban Express Delivery Van",
            Self::HeavyTruck => "Transcontinental Class 8 Semi & 53ft Trailer",
            Self::TransitBus => "Cityline 40ft Low-Floor Transit Bus",
            Self::Locomotive => "Pacific Diesel-Electric Rail Locomotive",
            Self::PassengerCoach => "Bombardier BiLevel Commuter Rail Coach",
            Self::FerryVessel => "Golden Gate Catamaran High-Speed Ferry",
            Self::Airliner => "SkyCourier B737 Commercial Jetliner",
            Self::LightAircraft => "Skyhawk C172 General Aviation Aircraft",
        }
    }

    /// Base purchase price in economy credits.
    pub fn purchase_price(&self) -> f64 {
        match self {
            Self::SportsCoupe => 4500.0,
            Self::EconomySedan => 1200.0,
            Self::LuxuryCab => 3500.0,
            Self::Suv => 3200.0,
            Self::Motorcycle => 2200.0,
            Self::CargoVan => 2800.0,
            Self::HeavyTruck => 8500.0,
            Self::TransitBus => 9200.0,
            Self::Locomotive => 24000.0,
            Self::PassengerCoach => 16000.0,
            Self::FerryVessel => 32000.0,
            Self::Airliner => 48000.0,
            Self::LightAircraft => 12500.0,
        }
    }

    /// Maximum cargo payload weight in kilograms.
    pub fn max_payload_kg(&self) -> f32 {
        match self {
            Self::SportsCoupe => 150.0,
            Self::EconomySedan => 250.0,
            Self::LuxuryCab => 200.0,
            Self::Suv => 600.0,
            Self::Motorcycle => 40.0,
            Self::CargoVan => 1500.0,
            Self::HeavyTruck => 28000.0,
            Self::TransitBus => 4500.0,
            Self::Locomotive => 60000.0,
            Self::PassengerCoach => 12000.0,
            Self::FerryVessel => 35000.0,
            Self::Airliner => 20000.0,
            Self::LightAircraft => 400.0,
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
    #[serde(default)]
    pub brakes_stage: u8,
    #[serde(default)]
    pub custom_paint: Option<[f32; 4]>,
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
            brakes_stage: 0,
            custom_paint: None,
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

    /// Braking efficiency multiplier derived from brakes upgrade stage (0 to 3).
    pub fn brake_multiplier(&self) -> f32 {
        1.0 + (self.brakes_stage as f32 * 0.15)
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
