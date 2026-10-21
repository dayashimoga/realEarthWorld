//! Vertical datum definitions and height conversions.

use serde::{Deserialize, Serialize};

/// Vertical datum classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerticalDatum {
    /// WGS84 Ellipsoidal Height (native GPS height).
    Wgs84Ellipsoidal,
    /// Orthometric Height (Height above Mean Sea Level / Geoid, e.g. EGM96 or EGM2008).
    Egm96Geoid,
    Egm2008Geoid,
}

/// Metadata indicating vertical datum and geoid separation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct VerticalDatumMetadata {
    pub datum: VerticalDatum,
    /// Geoid undulation N in meters: h = H + N
    pub geoid_undulation_meters: f64,
}

impl VerticalDatumMetadata {
    pub fn wgs84_default() -> Self {
        Self {
            datum: VerticalDatum::Wgs84Ellipsoidal,
            geoid_undulation_meters: 0.0,
        }
    }

    /// Approximate geoid undulation for Bengaluru region (EGM96 is approx -88.0m to -92.0m in South India).
    pub fn bengaluru_egm96() -> Self {
        Self {
            datum: VerticalDatum::Egm96Geoid,
            geoid_undulation_meters: -90.5,
        }
    }

    /// Converts ellipsoidal height to orthometric height: H = h - N
    pub fn ellipsoidal_to_orthometric(&self, ellipsoidal_h: f64) -> f64 {
        ellipsoidal_h - self.geoid_undulation_meters
    }

    /// Converts orthometric height to ellipsoidal height: h = H + N
    pub fn orthometric_to_ellipsoidal(&self, orthometric_h: f64) -> f64 {
        orthometric_h + self.geoid_undulation_meters
    }
}
