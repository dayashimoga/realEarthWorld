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
