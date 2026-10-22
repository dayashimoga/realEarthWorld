//! Event-driven API for Real Earth Gameplay Platform.
//!
//! Provides decoupled, strongly-typed events and an event bus to allow
//! headless simulations, ECS systems, UI viewports, and diagnostics to
//! observe gameplay occurrences without tight architectural coupling.

use crate::career::{CareerType, ReputationTier};
use realearth_shared::coordinates::Wgs84Coord;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Distinct gameplay events emitted during simulation and career progression.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GameplayEvent {
    /// A new job offer became available in the region catalog.
    JobOffered {
        job_id: String,
        career: CareerType,
        title: String,
    },
    /// The player accepted an available job.
    JobAccepted { job_id: String, career: CareerType },
    /// The active job has officially begun.
    JobStarted { job_id: String },
    /// The player navigated through an intermediate routing waypoint.
    WaypointReached {
        job_id: String,
        waypoint_index: usize,
        total_waypoints: usize,
    },
    /// The vehicle entered a designated interaction perimeter (pickup or dropoff).
    ZoneEntered {
        job_id: String,
        is_pickup: bool,
        zone_name: String,
    },
    /// Dwell interaction progress update (e.g. passenger boarding or cargo loading).
    InteractionProgress { job_id: String, progress_ratio: f32 },
    /// Passenger safely boarded the vehicle.
    PassengerBoarded {
        job_id: String,
        passenger_name: String,
    },
    /// Cargo safely loaded and secured into the vehicle.
    CargoLoaded {
        job_id: String,
        cargo_name: String,
        weight_kg: f32,
    },
    /// Cargo safely unloaded at destination.
    CargoUnloaded { job_id: String },
    /// Job successfully completed with verified compensation and rating.
    JobCompleted {
        job_id: String,
        career: CareerType,
        payout: f64,
        xp: u32,
        rating_stars: f32,
    },
    /// Job failed due to deadline expiration, severe cargo damage, or abandonment.
    JobFailed {
        job_id: String,
        career: CareerType,
        reason: String,
    },
    /// Job abandoned by player.
    JobAbandoned { job_id: String },
    /// Vehicle was recovered by emergency roadside assistance.
    VehicleRecovered { cost: f64, new_location: Wgs84Coord },
    /// Career reputation points updated.
    ReputationUpdated {
        career: CareerType,
        new_reputation: u32,
        tier: ReputationTier,
    },
    /// Overall player progression level increased.
    LevelUp { new_level: u32 },
    /// An achievement was unlocked.
    AchievementUnlocked {
        achievement_id: String,
        title: String,
    },
    /// High-impact collision occurred, impacting ride comfort and cargo integrity.
    CollisionImpact { impulse: f32, repair_cost: f64 },
    /// Odometer milestone driven along verified roads.
    DistanceMilestone { total_km: f64 },
    /// Discovered a surveyed real-world landmark.
    LandmarkDiscovered {
        landmark_name: String,
        region_id: String,
    },
}

/// Listener callback function for gameplay events.
pub type EventCallback = Box<dyn Fn(&GameplayEvent) + Send + Sync + 'static>;

/// Thread-safe event bus for decoupling gameplay actions from external systems.
#[derive(Default)]
pub struct GameplayEventBus {
    event_history: Arc<Mutex<Vec<GameplayEvent>>>,
    listeners: Arc<Mutex<Vec<EventCallback>>>,
}

impl GameplayEventBus {
    /// Creates a new empty event bus.
    pub fn new() -> Self {
        Self {
            event_history: Arc::new(Mutex::new(Vec::new())),
            listeners: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Emits a new event, notifying all registered listeners and logging to history.
    pub fn emit(&self, event: GameplayEvent) {
        if let Ok(mut history) = self.event_history.lock() {
            history.push(event.clone());
        }
        if let Ok(listeners) = self.listeners.lock() {
            for listener in listeners.iter() {
                listener(&event);
            }
        }
    }

    /// Subscribes an observer callback to receive all future emitted events.
    pub fn subscribe<F>(&self, callback: F)
    where
        F: Fn(&GameplayEvent) + Send + Sync + 'static,
    {
        if let Ok(mut listeners) = self.listeners.lock() {
            listeners.push(Box::new(callback));
        }
    }

    /// Retrieves an immutable snapshot of all events emitted since initialization.
    pub fn history(&self) -> Vec<GameplayEvent> {
        self.event_history
            .lock()
            .map(|h| h.clone())
            .unwrap_or_default()
    }

    /// Clears and returns all buffered events.
    pub fn drain_events(&self) -> Vec<GameplayEvent> {
        if let Ok(mut history) = self.event_history.lock() {
            history.drain(..).collect()
        } else {
            Vec::new()
        }
    }

    /// Returns the total count of events emitted.
    pub fn event_count(&self) -> usize {
        self.event_history.lock().map(|h| h.len()).unwrap_or(0)
    }
}
