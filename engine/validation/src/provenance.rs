//! Height provenance and confidence classification.

use realearth_ingestion::osm_types::OsmBuildingFeature;
use serde::{Deserialize, Serialize};

/// Provenance metadata for a feature's vertical height.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeightProvenance {
    /// Directly surveyed or documented in authoritative data (confidence: 1.0).
    Surveyed,
    /// Inferred from documented building levels count (confidence: 0.85).
    InferredLevels,
    /// Inferred from building category/usage heuristic (confidence: 0.50).
    DefaultCategory,
}

/// A validated height with explicit provenance and confidence score.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ValidatedHeight {
    pub height_meters: f64,
    pub provenance: HeightProvenance,
    pub confidence: f32,
}

impl ValidatedHeight {
    pub fn from_building(b: &OsmBuildingFeature) -> Self {
        if let Some(h) = b.explicit_height {
            Self {
                height_meters: h,
                provenance: HeightProvenance::Surveyed,
                confidence: 1.0,
            }
        } else if let Some(levels) = b.levels {
            let h = levels as f64 * 3.5 + 1.0;
            Self {
                height_meters: h,
                provenance: HeightProvenance::InferredLevels,
                confidence: 0.85,
            }
        } else {
            let h = b.building_type.default_height();
            Self {
                height_meters: h,
                provenance: HeightProvenance::DefaultCategory,
                confidence: 0.50,
            }
        }
    }
}
