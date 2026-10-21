//! Diagnostic models, severity classifications, and visual error overlays.

use realearth_shared::coordinates::Wgs84Coord;
use serde::{Deserialize, Serialize};

/// Classification of diagnostic severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    /// Non-geometric metadata inference or suggestion.
    Info,
    /// Non-fatal geometric defect safely repaired with high confidence.
    Warning,
    /// Feature-level defect that cannot be safely resolved; feature is quarantined.
    Error,
    /// Critical defect that halts compilation (e.g. corrupted CRS, invalid coordinate, zero vertices).
    Blocker,
}

impl std::fmt::Display for DiagnosticSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "INFO"),
            Self::Warning => write!(f, "WARNING"),
            Self::Error => write!(f, "ERROR"),
            Self::Blocker => write!(f, "BLOCKER"),
        }
    }
}

/// Structured diagnostic entry with geolocation and traceability.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticEntry {
    pub code: String,
    pub severity: DiagnosticSeverity,
    pub feature_id: Option<u64>,
    pub feature_type: Option<String>,
    pub location: Option<Wgs84Coord>,
    pub message: String,
    pub rule_id: Option<String>,
    pub source_dataset: Option<String>,
}

/// Source-resolution-aware validation tolerances.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ValidationTolerance {
    /// Linear snapping tolerance for duplicate or near-identical coordinates in meters.
    pub linear_snap_m: f64,
    /// Minimum polygon footprint area in square meters.
    pub min_polygon_area_m2: f64,
    /// Maximum allowable terrain slope angle in degrees before flagging foundation risk.
    pub max_terrain_slope_deg: f64,
    /// Minimum runway length in meters.
    pub min_runway_length_m: f64,
    /// Maximum elevation difference between adjacent DEM cells (m).
    pub max_dem_step_m: f32,
    /// Minimum confidence required to accept an automated repair (0.0 to 1.0).
    pub min_repair_confidence: f32,
}

impl Default for ValidationTolerance {
    fn default() -> Self {
        Self::surveyed()
    }
}

impl ValidationTolerance {
    /// High-precision surveyed / LiDAR / authoritative ground data.
    pub fn surveyed() -> Self {
        Self {
            linear_snap_m: 0.05,
            min_polygon_area_m2: 4.0,
            max_terrain_slope_deg: 45.0,
            min_runway_length_m: 300.0,
            max_dem_step_m: 15.0,
            min_repair_confidence: 0.80,
        }
    }

    /// Satellite imagery / USGS 3DEP 1/3 arc-second DEM.
    pub fn satellite() -> Self {
        Self {
            linear_snap_m: 0.50,
            min_polygon_area_m2: 10.0,
            max_terrain_slope_deg: 50.0,
            min_runway_length_m: 300.0,
            max_dem_step_m: 25.0,
            min_repair_confidence: 0.75,
        }
    }

    /// Coarse / Global datasets (e.g. SRTM 30m, global OSM).
    pub fn coarse() -> Self {
        Self {
            linear_snap_m: 2.0,
            min_polygon_area_m2: 25.0,
            max_terrain_slope_deg: 60.0,
            min_runway_length_m: 300.0,
            max_dem_step_m: 50.0,
            min_repair_confidence: 0.70,
        }
    }
}

/// Geolocated visual error overlay for 3D viewer rendering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticOverlay {
    pub id: u64,
    pub severity: DiagnosticSeverity,
    pub location: Wgs84Coord,
    pub title: String,
    pub description: String,
    pub color_rgb: [f32; 3],
}

impl DiagnosticOverlay {
    pub fn from_diagnostic(id: u64, entry: &DiagnosticEntry) -> Option<Self> {
        let location = entry.location?;
        let color_rgb = match entry.severity {
            DiagnosticSeverity::Blocker => [1.0, 0.0, 0.0], // Red
            DiagnosticSeverity::Error => [1.0, 0.35, 0.0],  // Orange
            DiagnosticSeverity::Warning => [1.0, 0.85, 0.0], // Yellow
            DiagnosticSeverity::Info => [0.2, 0.6, 1.0],    // Light Blue
        };

        Some(Self {
            id,
            severity: entry.severity,
            location,
            title: entry.code.clone(),
            description: entry.message.clone(),
            color_rgb,
        })
    }
}
