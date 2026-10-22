//! High-Precision Ellipsoidal Geodesics, Antimeridian Continuity,
//! and Planetary Floating-Origin Coordinate Management.

use crate::ellipsoid::Wgs84Ellipsoid;
use realearth_shared::coordinates::Wgs84Coord;
use realearth_shared::errors::Result;

/// Earth mean radius in meters (IUGG standard).
pub const EARTH_MEAN_RADIUS_METERS: f64 = 6_371_008.8;

/// Fast spherical Haversine great-circle distance between two WGS84 points in meters.
pub fn haversine_distance(p1: Wgs84Coord, p2: Wgs84Coord) -> f64 {
    let lat1_rad = p1.lat.to_radians();
    let lat2_rad = p2.lat.to_radians();
    let delta_lat = (p2.lat - p1.lat).to_radians();
    let delta_lon = antimeridian_distance_lon(p1.lon, p2.lon).to_radians();

    let a = (delta_lat * 0.5).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lon * 0.5).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_MEAN_RADIUS_METERS * c
}

/// Computes the shortest angular distance between two longitudes in degrees,
/// correctly accounting for antimeridian wraparound ([-180°, +180°]).
pub fn antimeridian_distance_lon(lon1: f64, lon2: f64) -> f64 {
    let mut diff = ((lon2 - lon1) + 180.0) % 360.0;
    if diff < 0.0 {
        diff += 360.0;
    }
    diff - 180.0
}

/// Normalizes any longitude in degrees into the canonical [-180.0, 180.0] range.
pub fn normalize_longitude(lon: f64) -> f64 {
    let mut normalized = (lon + 180.0) % 360.0;
    if normalized < 0.0 {
        normalized += 360.0;
    }
    normalized - 180.0
}

/// Checks whether the direct shortest path between two longitudes crosses the antimeridian.
pub fn is_antimeridian_crossing(lon1: f64, lon2: f64) -> bool {
    let norm1 = normalize_longitude(lon1);
    let norm2 = normalize_longitude(lon2);
    (norm1 - norm2).abs() > 180.0
}

/// High-precision inverse Vincenty geodesic formula on the WGS84 ellipsoid.
/// Calculates ellipsoidal surface distance with sub-millimeter precision (< 0.5mm).
pub fn geodesic_distance_vincenty(p1: Wgs84Coord, p2: Wgs84Coord) -> Result<f64> {
    let a = Wgs84Ellipsoid::A;
    let b = Wgs84Ellipsoid::B;
    let f = Wgs84Ellipsoid::F;

    let phi1 = p1.lat.to_radians();
    let phi2 = p2.lat.to_radians();

    let l = antimeridian_distance_lon(p1.lon, p2.lon).to_radians();

    // Check for coincident or nearly coincident points
    if (phi1 - phi2).abs() < 1e-12 && l.abs() < 1e-12 {
        return Ok(0.0);
    }

    let u1 = ((1.0 - f) * phi1.tan()).atan();
    let u2 = ((1.0 - f) * phi2.tan()).atan();

    let sin_u1 = u1.sin();
    let cos_u1 = u1.cos();
    let sin_u2 = u2.sin();
    let cos_u2 = u2.cos();

    let mut lambda = l;
    let mut lambda_prev: f64;
    let mut sin_sigma: f64;
    let mut cos_sigma: f64;
    let mut sigma: f64;
    let mut sin_alpha: f64;
    let mut cos_sq_alpha: f64;
    let mut cos2_sigma_m: f64;

    let max_iterations = 200;
    let mut iter = 0;

    loop {
        let sin_lambda = lambda.sin();
        let cos_lambda = lambda.cos();

        sin_sigma = ((cos_u2 * sin_lambda).powi(2)
            + (cos_u1 * sin_u2 - sin_u1 * cos_u2 * cos_lambda).powi(2))
        .sqrt();

        if sin_sigma.abs() < 1e-12 {
            return Ok(0.0); // Co-incident points
        }

        cos_sigma = sin_u1 * sin_u2 + cos_u1 * cos_u2 * cos_lambda;
        sigma = sin_sigma.atan2(cos_sigma);

        sin_alpha = cos_u1 * cos_u2 * sin_lambda / sin_sigma;
        cos_sq_alpha = 1.0 - sin_alpha * sin_alpha;

        cos2_sigma_m = if cos_sq_alpha.abs() < 1e-12 {
            0.0 // Equatorial line
        } else {
            cos_sigma - 2.0 * sin_u1 * sin_u2 / cos_sq_alpha
        };

        let c = f / 16.0 * cos_sq_alpha * (4.0 + f * (4.0 - 3.0 * cos_sq_alpha));
        lambda_prev = lambda;
        lambda = l
            + (1.0 - c)
                * f
                * sin_alpha
                * (sigma
                    + c * sin_sigma
                        * (cos2_sigma_m
                            + c * cos_sigma * (-1.0 + 2.0 * cos2_sigma_m * cos2_sigma_m)));

        iter += 1;
        if (lambda - lambda_prev).abs() < 1e-12 || iter >= max_iterations {
            break;
        }
    }

    if iter >= max_iterations {
        // Fallback to Haversine for antipodal edge cases where Vincenty fails to converge
        return Ok(haversine_distance(p1, p2));
    }

    let u_sq = cos_sq_alpha * (a * a - b * b) / (b * b);
    let cap_a = 1.0 + u_sq / 16384.0 * (4096.0 + u_sq * (-768.0 + u_sq * (320.0 - 175.0 * u_sq)));
    let cap_b = u_sq / 1024.0 * (256.0 + u_sq * (-128.0 + u_sq * (74.0 - 47.0 * u_sq)));

    let delta_sigma = cap_b
        * sin_sigma
        * (cos2_sigma_m
            + cap_b / 4.0
                * (cos_sigma * (-1.0 + 2.0 * cos2_sigma_m * cos2_sigma_m)
                    - cap_b / 6.0
                        * cos2_sigma_m
                        * (-3.0 + 4.0 * sin_sigma * sin_sigma)
                        * (-3.0 + 4.0 * cos2_sigma_m * cos2_sigma_m)));

    let s = b * cap_a * (sigma - delta_sigma);
    Ok(s)
}

/// Seamlessly interpolates between two geographic coordinates across the shortest path,
/// correctly wrapping across the antimeridian without wrapping anomalies.
pub fn interpolate_across_antimeridian(p1: Wgs84Coord, p2: Wgs84Coord, t: f64) -> Wgs84Coord {
    let t_clamped = t.clamp(0.0, 1.0);
    let lat = p1.lat + (p2.lat - p1.lat) * t_clamped;
    let delta_lon = antimeridian_distance_lon(p1.lon, p2.lon);
    let lon = normalize_longitude(p1.lon + delta_lon * t_clamped);
    let alt = p1.alt + (p2.alt - p1.alt) * t_clamped;

    Wgs84Coord::new(lat, lon, alt)
}

/// Evaluates whether camera motion warrants re-anchoring the local floating-origin frame.
/// Re-anchoring occurs when the camera moves beyond `threshold_meters` (default 50 km)
/// to maintain 32-bit GPU floating-point precision with sub-millimeter precision.
pub fn should_reanchor(camera: Wgs84Coord, anchor: Wgs84Coord, threshold_meters: f64) -> bool {
    haversine_distance(camera, anchor) > threshold_meters
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haversine_and_vincenty_bengaluru_to_chennai() {
        // Bengaluru Vidhana Soudha: 12.9797°N, 77.5907°E
        let blr = Wgs84Coord::new(12.9797, 77.5907, 0.0);
        // Chennai Central Station: 13.0827°N, 80.2755°E
        let chn = Wgs84Coord::new(13.0827, 80.2755, 0.0);

        let dist_vincenty = geodesic_distance_vincenty(blr, chn).unwrap();
        let dist_haversine = haversine_distance(blr, chn);

        // Approximate direct distance is ~290 km
        assert!(
            (dist_vincenty - 291_000.0).abs() < 5000.0,
            "Vincenty distance {} should be ~291km",
            dist_vincenty
        );

        // Haversine and Vincenty should agree within 0.5% on spherical vs ellipsoid
        let diff_ratio = (dist_vincenty - dist_haversine).abs() / dist_vincenty;
        assert!(
            diff_ratio < 0.005,
            "Haversine error ratio was {} (budget < 0.5%)",
            diff_ratio
        );
    }

    #[test]
    fn test_antimeridian_crossing_continuity() {
        // Point A: 179.0°E, Point B: -179.0°W (2° apart across 180° meridian)
        let p_a = Wgs84Coord::new(0.0, 179.0, 0.0);
        let p_b = Wgs84Coord::new(0.0, -179.0, 0.0);

        assert!(is_antimeridian_crossing(p_a.lon, p_b.lon));
        assert_eq!(antimeridian_distance_lon(p_a.lon, p_b.lon), 2.0);

        // Midpoint must be exactly at the 180.0° antimeridian
        let mid = interpolate_across_antimeridian(p_a, p_b, 0.5);
        assert!(
            (mid.lon.abs() - 180.0).abs() < 1e-6,
            "Midpoint longitude {} should be +/- 180.0",
            mid.lon
        );

        // Geodesic distance must be ~222 km (2 degrees at equator)
        let dist = geodesic_distance_vincenty(p_a, p_b).unwrap();
        assert!((dist - 222_600.0).abs() < 2000.0);
    }

    #[test]
    fn test_floating_origin_reanchor_trigger() {
        let anchor = Wgs84Coord::new(37.7952, -122.4028, 0.0); // San Francisco
        let nearby = Wgs84Coord::new(37.8000, -122.4000, 0.0); // ~600m away
        let far = Wgs84Coord::new(37.3382, -121.8863, 0.0); // San Jose (~70km away)

        assert!(!should_reanchor(nearby, anchor, 50_000.0));
        assert!(should_reanchor(far, anchor, 50_000.0));
    }
}
