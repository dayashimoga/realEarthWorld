//! Aviation navigation, airport runways, taxiway networks, and ILS glideslope guidance.
//!
//! Provides dedicated aeronautical navigation geometry strictly decoupled from road graphs.

use realearth_shared::coordinates::Wgs84Coord;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Standard Instrument Landing System (ILS) approach guidance signals.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct IlsGuidance {
    /// Horizontal deviation from runway centerline in degrees (negative = left, positive = right).
    pub localizer_deviation_deg: f32,
    /// Vertical deviation from nominal 3.0° glideslope (negative = below slope, positive = above slope).
    pub glideslope_deviation_deg: f32,
    /// Slant distance to runway threshold in meters.
    pub distance_to_threshold_meters: f32,
    /// Is the aircraft within the valid reception cone of the ILS localizer (+/- 35 degrees).
    pub is_localizer_captured: bool,
    /// Is the aircraft within the valid glideslope beam (+/- 1.4 degrees of 3.0° slope).
    pub is_glideslope_captured: bool,
}

/// An airport runway threshold and orientation specification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AirportRunway {
    /// Runway designation (e.g. "09L", "27R").
    pub identifier: String,
    /// Primary threshold location.
    pub threshold: Wgs84Coord,
    /// Opposite reciprocal threshold location.
    pub reciprocal_threshold: Wgs84Coord,
    /// True heading of runway centerline in degrees [0, 360).
    pub true_heading_deg: f32,
    /// Physical runway length in meters.
    pub length_meters: f32,
    /// Physical runway width in meters.
    pub width_meters: f32,
    /// Nominal glideslope descent angle (standard: 3.0 degrees).
    pub glideslope_angle_deg: f32,
    /// Localizer transmission frequency in MHz (e.g. 110.3 MHz).
    pub ils_frequency_mhz: Option<f32>,
}

impl AirportRunway {
    /// Computes instantaneous ILS localizer and glideslope deviations for an approaching aircraft.
    pub fn compute_ils_guidance(
        &self,
        aircraft_coord: &Wgs84Coord,
        aircraft_alt_msl: f32,
    ) -> IlsGuidance {
        let (lat1, lon1) = (
            self.threshold.lat.to_radians(),
            self.threshold.lon.to_radians(),
        );
        let (lat2, lon2) = (
            aircraft_coord.lat.to_radians(),
            aircraft_coord.lon.to_radians(),
        );

        let d_lat = lat2 - lat1;
        let d_lon = lon2 - lon1;

        // Flat-earth tangent approximation suitable for terminal approach (< 40 km)
        let r_earth = 6371000.0;
        let delta_north = d_lat * r_earth;
        let delta_east = d_lon * r_earth * lat1.cos();
        let ground_distance = (delta_north * delta_north + delta_east * delta_east).sqrt() as f32;

        let runway_rad = self.true_heading_deg.to_radians();
        let bearing_to_ac = delta_east.atan2(delta_north) as f32;

        // Angle between approach path and aircraft bearing
        // Inbound heading is reciprocal of runway heading (runway_rad - PI)
        let inbound_bearing =
            (runway_rad + std::f32::consts::PI).rem_euclid(2.0 * std::f32::consts::PI);
        let mut loc_dev = (bearing_to_ac - inbound_bearing).to_degrees();
        if loc_dev > 180.0 {
            loc_dev -= 360.0;
        } else if loc_dev < -180.0 {
            loc_dev += 360.0;
        }

        let runway_alt = self.threshold.alt as f32;
        let height_above_threshold = (aircraft_alt_msl - runway_alt).max(0.0);
        let actual_glide_angle = (height_above_threshold / ground_distance.max(10.0))
            .atan()
            .to_degrees();
        let gs_dev = actual_glide_angle - self.glideslope_angle_deg;

        let slant_dist = (ground_distance * ground_distance
            + height_above_threshold * height_above_threshold)
            .sqrt();

        IlsGuidance {
            localizer_deviation_deg: loc_dev,
            glideslope_deviation_deg: gs_dev,
            distance_to_threshold_meters: slant_dist,
            is_localizer_captured: loc_dev.abs() < 10.0,
            is_glideslope_captured: gs_dev.abs() < 1.0 && ground_distance > 500.0,
        }
    }
}

/// An airport terminal facility with runways and taxiway holding points.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Airport {
    pub icao_code: String,
    pub name: String,
    pub reference_coord: Wgs84Coord,
    pub elevation_msl_meters: f32,
    pub runways: Vec<AirportRunway>,
}

/// Airway navigation corridor for high-altitude en-route flight.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AirCorridorSegment {
    pub airway_id: String,
    pub start_waypoint: String,
    pub end_waypoint: String,
    pub start_coord: Wgs84Coord,
    pub end_coord: Wgs84Coord,
    pub minimum_enroute_altitude_meters: f32,
}

/// Global aviation aeronautical navigation chart database.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AirNavigationDatabase {
    pub airports: HashMap<String, Airport>,
    pub airways: Vec<AirCorridorSegment>,
}

impl AirNavigationDatabase {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_airport(&mut self, airport: Airport) {
        self.airports.insert(airport.icao_code.clone(), airport);
    }

    pub fn add_airway(&mut self, airway: AirCorridorSegment) {
        self.airways.push(airway);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kempegowda_bengaluru_vobl_ils_guidance() {
        let mut db = AirNavigationDatabase::new();

        // Kempegowda International Airport (VOBL) Runway 09R
        let rwy_09r = AirportRunway {
            identifier: "09R".to_string(),
            threshold: Wgs84Coord::new(13.1979, 77.6800, 915.0),
            reciprocal_threshold: Wgs84Coord::new(13.1979, 77.7200, 915.0),
            true_heading_deg: 90.0,
            length_meters: 4000.0,
            width_meters: 45.0,
            glideslope_angle_deg: 3.0,
            ils_frequency_mhz: Some(110.3),
        };

        let airport = Airport {
            icao_code: "VOBL".to_string(),
            name: "Kempegowda International Airport".to_string(),
            reference_coord: Wgs84Coord::new(13.1979, 77.7063, 915.0),
            elevation_msl_meters: 915.0,
            runways: vec![rwy_09r.clone()],
        };

        db.add_airport(airport);

        // Aircraft on 10 km final approach along 090° inbound heading
        // Runway heading 90° means inbound approach comes from West (long < 77.6800)
        // 10 km West: delta_lon ~ -10000 / (6371000 * cos(13.2 deg)) ~ -0.091 deg
        let aircraft_pos = Wgs84Coord::new(13.1979, 77.6800 - 0.091, 915.0 + 524.0); // 3° slope height ~ 524m
        let guidance = rwy_09r.compute_ils_guidance(&aircraft_pos, (915.0 + 524.0) as f32);

        assert!(
            guidance.localizer_deviation_deg.abs() < 1.0,
            "Aircraft on extended centerline should have ~0 localizer deviation, got {}",
            guidance.localizer_deviation_deg
        );
        assert!(
            guidance.glideslope_deviation_deg.abs() < 0.5,
            "Aircraft on 3-degree glideslope should have ~0 glideslope deviation, got {}",
            guidance.glideslope_deviation_deg
        );
        assert!(guidance.is_localizer_captured);
    }
}
