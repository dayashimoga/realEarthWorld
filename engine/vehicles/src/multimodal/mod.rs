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
