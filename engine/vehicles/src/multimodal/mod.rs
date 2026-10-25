//! Multimodal vehicle simulation modules: Road transport, Railway, Marine, and Aviation.

pub mod aviation;
pub mod marine;
pub mod road_transport;
pub mod train;

pub use aviation::{AircraftFlight, AviationCategory, FlightInputs};
pub use marine::{MarineInputs, MarineVessel, MarineVesselType};
pub use road_transport::{
    HeavyTruckModel, MotorcycleModel, RoadAccessRestriction, RoadVehicleCategory, TransitBusModel,
};
pub use train::{DavisCoefficients, LocomotiveType, TrainBrakeState, TrainCar, TrainConsist};

/// Domain-specific audio events emitted by multimodal transportation vehicles.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum MultimodalAudioEvent {
    /// High-revving motorcycle 4-cylinder engine acoustic signature.
    MotorcycleRev { rpm: f32 },
    /// Heavy transit bus compressed air brake exhaust hiss.
    BusAirBrakeRelease { pressure_bar: f32 },
    /// Passenger boarding or next-stop request chime.
    BusPassengerChime,
    /// Heavy semi-truck engine compression jake brake retarding sound.
    TruckJakeBrake { rpm: f32 },
    /// Fifth-wheel kingpin trailer hitch coupling.
    TruckTrailerHitch,
    /// High-pressure locomotive pneumatic horn or whistle.
    TrainHorn,
    /// Steel wheel-on-rail acoustic flange squeal on curved track.
    TrainFlangeSqueal { speed_kph: f32 },
    /// Low-frequency deep maritime vessel foghorn blast.
    ShipFoghorn,
    /// Wave impact displacement splash against bow hull.
    ShipWaveSplash { speed_knots: f32 },
    /// High-bypass turbofan jet engine spool-up or thrust reverse roar.
    AircraftJetSpool { throttle: f32 },
    /// Audible stall warning horn / stick shaker alarm.
    AircraftStallWarning,
    /// Oleo-pneumatic landing gear tire touchdown chirp.
    AircraftGearTouchdown { descent_mps: f32 },
}

/// Primary active transportation mode across the multimodal fleet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ActiveTransportMode {
    Car,
    Motorcycle,
    TransitBus,
    HeavyTruck,
    Train,
    Marine,
    Aviation,
}

impl ActiveTransportMode {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Car => "Automobile (Ackermann/Pacejka)",
            Self::Motorcycle => "Motorcycle (2-Wheel Lean Dynamics)",
            Self::TransitBus => "Transit Bus (Pneumatic Air Brakes)",
            Self::HeavyTruck => "Heavy Haul Truck (Trailer Articulation)",
            Self::Train => "Railway Train Consist (Davis / Westinghouse)",
            Self::Marine => "Marine Vessel (Archimedes / Metacentric)",
            Self::Aviation => "Aviation Aircraft (6-DOF Aerodynamic Polar)",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Car => "🏎️",
            Self::Motorcycle => "🏍️",
            Self::TransitBus => "🚌",
            Self::HeavyTruck => "🚛",
            Self::Train => "🚆",
            Self::Marine => "🚢",
            Self::Aviation => "✈️",
        }
    }
}

/// Authoritative operational motion state across all transportation modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AuthoritativeVehicleState {
    Parked,
    Idle,
    Driving,
    Braking,
    Reversing,
    Rail,
    Marine,
    Flight,
    Paused,
}
