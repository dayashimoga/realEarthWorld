//! Real Earth Modular Gameplay Platform (`realearth-gameplay`).
//!
//! Provides an independent, event-driven gameplay framework, playable careers,
//! mission state machines, player profile progression, garage upgrades,
//! save/load migrations, roadside recovery, and minimal-click HUD systems.
//! Completely decoupled from rendering, display servers, or physical geometry mutations.

pub mod achievements;
pub mod career;
pub mod destinations;
pub mod economy;
pub mod events;
pub mod garage;
pub mod hud;
pub mod jobs;
pub mod profile;
pub mod recovery;
pub mod save;

// Primary re-exports
pub use achievements::{Achievement, AchievementRegistry};
pub use career::{CareerType, ReputationTier};
pub use destinations::{JobCatalogGenerator, SurveyedDestinations};
pub use economy::{EconomyConfig, EconomyError, Transaction, TransactionCategory, Wallet};
pub use events::{EventCallback, GameplayEvent, GameplayEventBus};
pub use garage::{OwnedVehicle, VehicleCategory, VehicleGarage};
pub use hud::{
    ActiveObjectiveHud, DebriefModal, EarningsHud, GameplayHudModel, JobBrowserHud, JobOfferSummary,
};
pub use jobs::{CargoOrPassenger, Job, JobCompletionResult, JobDifficulty, JobLocation, JobState};
pub use profile::PlayerProfile;
pub use recovery::{RecoveryDispatcher, RecoveryError, RecoveryOutcome};
pub use save::{
    GameSettings, SaveData, SaveError, SaveManager, SaveMigrationEngine,
    CURRENT_SAVE_SCHEMA_VERSION,
};
