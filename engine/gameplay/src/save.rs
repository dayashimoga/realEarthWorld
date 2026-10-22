//! Persistent save/load system with schema versioning and forward migrations.

use crate::jobs::Job;
use crate::profile::PlayerProfile;
use realearth_shared::coordinates::Wgs84Coord;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use thiserror::Error;

/// Current active schema version.
pub const CURRENT_SAVE_SCHEMA_VERSION: u32 = 2;

/// Errors during game save or load operations.
#[derive(Debug, Error)]
pub enum SaveError {
    #[error("I/O error during save/load operation: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Unsupported future save schema version {found}, maximum supported is {supported}")]
    FutureVersion { found: u32, supported: u32 },
    #[error("Migration error: {0}")]
    Migration(String),
}

/// Gameplay and UI preferences persisted in save files.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameSettings {
    pub difficulty: String,
    pub metric_units: bool,
    pub currency_symbol: String,
    pub subtitle_notifications: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            difficulty: "Standard".to_string(),
            metric_units: true,
            currency_symbol: "₹".to_string(),
            subtitle_notifications: true,
        }
    }
}

/// Root persistent save state container.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaveData {
    pub schema_version: u32,
    pub timestamp_unix: u64,
    pub profile: PlayerProfile,
    pub active_job: Option<Job>,
    pub active_region_id: String,
    pub vehicle_position: Wgs84Coord,
    pub settings: GameSettings,
}

impl SaveData {
    /// Creates a new default save state.
    pub fn new(
        profile: PlayerProfile,
        active_region: impl Into<String>,
        position: Wgs84Coord,
    ) -> Self {
        Self {
            schema_version: CURRENT_SAVE_SCHEMA_VERSION,
            timestamp_unix: 0,
            profile,
            active_job: None,
            active_region_id: active_region.into(),
            vehicle_position: position,
            settings: GameSettings::default(),
        }
    }
}

/// Engine executing declarative schema migrations across save file versions.
pub struct SaveMigrationEngine;

impl SaveMigrationEngine {
    /// Loads a save file from raw JSON, executing any necessary migrations to bring it to CURRENT_SAVE_SCHEMA_VERSION.
    pub fn load_and_migrate(json_str: &str) -> Result<SaveData, SaveError> {
        // Inspect version tag first via untyped JSON value
        let raw_val: serde_json::Value = serde_json::from_str(json_str)?;
        let version = raw_val
            .get("schema_version")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as u32;

        if version > CURRENT_SAVE_SCHEMA_VERSION {
            return Err(SaveError::FutureVersion {
                found: version,
                supported: CURRENT_SAVE_SCHEMA_VERSION,
            });
        }

        let mut current_json = raw_val;

        // Migration step: Schema V1 -> Schema V2
        if version == 1 {
            current_json = Self::migrate_v1_to_v2(current_json)?;
        }

        let migrated: SaveData = serde_json::from_value(current_json)?;
        Ok(migrated)
    }

    /// Migrates Schema V1 (legacy structure without currency_symbol and settings) into Schema V2.
    fn migrate_v1_to_v2(mut v1: serde_json::Value) -> Result<serde_json::Value, SaveError> {
        let obj = v1
            .as_object_mut()
            .ok_or_else(|| SaveError::Migration("Root JSON is not an object".to_string()))?;

        // Update schema version tag
        obj.insert("schema_version".to_string(), serde_json::json!(2));

        // Inject default settings if missing
        if !obj.contains_key("settings") {
            let default_settings = serde_json::to_value(GameSettings::default())?;
            obj.insert("settings".to_string(), default_settings);
        }

        Ok(v1)
    }
}

/// Save manager handling file I/O and persistence.
pub struct SaveManager;

impl SaveManager {
    /// Writes game state to a JSON file.
    pub fn save_to_file(path: &Path, data: &SaveData) -> Result<(), SaveError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json_str = serde_json::to_string_pretty(data)?;
        let mut file = File::create(path)?;
        file.write_all(json_str.as_bytes())?;
        Ok(())
    }

    /// Reads and migrates game state from a JSON file.
    pub fn load_from_file(path: &Path) -> Result<SaveData, SaveError> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        SaveMigrationEngine::load_and_migrate(&contents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_save_and_load_roundtrip() {
        let dir = tempdir().unwrap();
        let save_path = dir.path().join("profile_save.json");

        let profile = PlayerProfile::new("pilot-test", "Test Pilot");
        let initial = SaveData::new(
            profile,
            "bengaluru-central",
            Wgs84Coord::new(12.9797, 77.5907, 920.0),
        );

        SaveManager::save_to_file(&save_path, &initial).unwrap();
        let loaded = SaveManager::load_from_file(&save_path).unwrap();

        assert_eq!(loaded.schema_version, CURRENT_SAVE_SCHEMA_VERSION);
        assert_eq!(loaded.profile.name, "Test Pilot");
        assert_eq!(loaded.active_region_id, "bengaluru-central");
        assert_eq!(loaded.vehicle_position.lat, 12.9797);
    }

    #[test]
    fn test_schema_migration_v1_to_v2() {
        // Synthetic V1 payload lacking 'settings' and schema_version: 1
        let v1_json = r#"{
            "schema_version": 1,
            "timestamp_unix": 1000,
            "profile": {
                "player_id": "legacy-01",
                "name": "Legacy Driver",
                "level": 3,
                "xp": 450,
                "wallet": {
                    "balance": 1250.0,
                    "transactions": []
                },
                "reputation": {
                    "TaxiPassenger": 150
                },
                "total_distance_km": 42.5,
                "clean_streak_km": 12.0,
                "completed_jobs": 4,
                "failed_jobs": 0,
                "garage": {
                    "vehicles": [
                        {
                            "id": "car-01",
                            "model_name": "Old Sedan",
                            "category": "EconomySedan",
                            "engine_stage": 1,
                            "handling_stage": 0,
                            "comfort_stage": 0,
                            "condition_pct": 95.0,
                            "odometer_km": 42.5
                        }
                    ],
                    "active_index": 0
                },
                "achievements": {
                    "achievements": {}
                },
                "discovered_landmarks": []
            },
            "active_job": null,
            "active_region_id": "bengaluru-central",
            "vehicle_position": {
                "lat": 12.9797,
                "lon": 77.5907,
                "alt": 920.0
            }
        }"#;

        let migrated = SaveMigrationEngine::load_and_migrate(v1_json).unwrap();
        assert_eq!(migrated.schema_version, 2);
        assert_eq!(migrated.profile.name, "Legacy Driver");
        assert_eq!(migrated.settings.currency_symbol, "₹");
        assert!(migrated.settings.metric_units);
    }
}
