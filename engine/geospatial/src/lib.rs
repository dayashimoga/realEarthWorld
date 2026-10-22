//! High-precision Geodetic, ECEF, ENU, and UTM transformations for Real Earth.

pub mod crs;
pub mod datum;
pub mod ellipsoid;
pub mod geodesic;
pub mod manifest;
pub mod planetary_grid;
pub mod tiles;
pub mod transforms;
pub mod utm;

pub use crs::{CrsDefinition, CrsKind, CrsRegistry, StandardVerticalDatum};
pub use datum::{VerticalDatum, VerticalDatumMetadata};
pub use ellipsoid::Wgs84Ellipsoid;
pub use geodesic::{
    antimeridian_distance_lon, geodesic_distance_vincenty, haversine_distance,
    interpolate_across_antimeridian, is_antimeridian_crossing, normalize_longitude,
    should_reanchor, EARTH_MEAN_RADIUS_METERS,
};
pub use manifest::{DataSourceProvider, GlobalDataSourceManifest, LicenseTerms, QualityProfile};
pub use planetary_grid::{GlobalCellKey, PlanetarySpatialIndex, MAX_PLANETARY_LEVEL};
pub use tiles::TileCoord;
pub use transforms::{ecef_to_wgs84, wgs84_to_ecef, EnuConverter};
pub use utm::{utm_central_meridian, utm_zone, wgs84_to_utm, UtmCoord};

#[cfg(test)]
mod tests {
    use super::*;
    use realearth_shared::coordinates::{EnuCoord, Wgs84Coord};

    const EPSILON_METERS: f64 = 1e-6; // 1 micrometer

    #[test]
    fn test_wgs84_to_ecef_roundtrip_bengaluru() {
        // Vidhana Soudha, Bengaluru
        let original = Wgs84Coord::new(12.9797, 77.5907, 920.0);
        let ecef = wgs84_to_ecef(original);
        let back = ecef_to_wgs84(ecef);

        assert!(
            (original.lat - back.lat).abs() < 1e-9,
            "Lat error too large"
        );
        assert!(
            (original.lon - back.lon).abs() < 1e-9,
            "Lon error too large"
        );
        assert!(
            (original.alt - back.alt).abs() < EPSILON_METERS,
            "Alt error too large"
        );

        // Verify distance in ECEF is sub-micrometer
        let back_ecef = wgs84_to_ecef(back);
        let diff = ((ecef.x - back_ecef.x).powi(2)
            + (ecef.y - back_ecef.y).powi(2)
            + (ecef.z - back_ecef.z).powi(2))
        .sqrt();
        assert!(diff < EPSILON_METERS, "ECEF distance error was {} m", diff);
    }

    #[test]
    fn test_global_edge_cases_roundtrip() {
        let test_coords = [
            Wgs84Coord::new(0.0, 0.0, 0.0),           // Null Island
            Wgs84Coord::new(90.0, 0.0, 100.0),        // North Pole
            Wgs84Coord::new(-90.0, 0.0, -50.0),       // South Pole
            Wgs84Coord::new(0.0, 180.0, 500.0),       // Antimeridian Equator
            Wgs84Coord::new(0.0, -180.0, 500.0),      // Antimeridian Equator (West)
            Wgs84Coord::new(45.0, 45.0, 1500.0),      // Mid-latitude
            Wgs84Coord::new(13.1986, 77.7066, 915.0), // Kempegowda Intl Airport
        ];

        for coord in test_coords {
            let ecef = wgs84_to_ecef(coord);
            let back = ecef_to_wgs84(ecef);

            let back_ecef = wgs84_to_ecef(back);
            let diff = ((ecef.x - back_ecef.x).powi(2)
                + (ecef.y - back_ecef.y).powi(2)
                + (ecef.z - back_ecef.z).powi(2))
            .sqrt();
            assert!(
                diff < 1e-5,
                "Failed roundtrip for {:?}, diff was {} m",
                coord,
                diff
            );
        }
    }

    #[test]
    fn test_enu_converter_bengaluru() {
        let anchor = Wgs84Coord::new(12.9797, 77.5907, 920.0); // Vidhana Soudha
        let converter = EnuConverter::new(anchor);

        // Anchor itself must convert to (0, 0, 0)
        let anchor_enu = converter.wgs84_to_enu(anchor);
        assert!(anchor_enu.east.abs() < 1e-6);
        assert!(anchor_enu.north.abs() < 1e-6);
        assert!(anchor_enu.up.abs() < 1e-6);

        // Point 100m East, 200m North, 50m Up
        let target_enu = EnuCoord::new(100.0, 200.0, 50.0);
        let target_wgs84 = converter.enu_to_wgs84(target_enu);
        let back_enu = converter.wgs84_to_enu(target_wgs84);

        assert!((target_enu.east - back_enu.east).abs() < 1e-6);
        assert!((target_enu.north - back_enu.north).abs() < 1e-6);
        assert!((target_enu.up - back_enu.up).abs() < 1e-6);
    }

    #[test]
    fn test_utm_bengaluru_zone() {
        // Bengaluru is at 77.59°E -> UTM Zone 43N
        let coord = Wgs84Coord::new(12.9797, 77.5907, 920.0);
        let utm = wgs84_to_utm(coord);

        assert_eq!(utm.zone, 43);
        assert!(utm.north);
        // Easting should be roughly 780,000 m (central meridian 75°E is 500,000 m)
        assert!(utm.easting > 700_000.0 && utm.easting < 850_000.0);
        // Northing is approx 1,435,000 m
        assert!(utm.northing > 1_400_000.0 && utm.northing < 1_500_000.0);
    }

    #[test]
    fn test_web_mercator_tile_bounds() {
        let coord = Wgs84Coord::new(12.9797, 77.5907, 0.0);
        let tile = TileCoord::from_wgs84(coord, 14);
        let bounds = tile.bounds();

        assert!(
            bounds.contains(&coord),
            "Tile bounds must contain original coord"
        );
    }

    #[test]
    fn test_vertical_datum_conversions() {
        let meta = VerticalDatumMetadata::bengaluru_egm96();
        let orthometric = 920.0; // 920m above MSL
        let ellipsoidal = meta.orthometric_to_ellipsoidal(orthometric);
        assert_eq!(ellipsoidal, 920.0 - 90.5);

        let back_ortho = meta.ellipsoidal_to_orthometric(ellipsoidal);
        assert_eq!(back_ortho, 920.0);
    }

    #[test]
    fn test_san_francisco_utm_zone_and_navd88() {
        // Transamerica Pyramid, San Francisco: 37.7952° N, 122.4028° W, 10m MSL
        let sf_coord = Wgs84Coord::new(37.7952, -122.4028, 10.0);
        let utm = wgs84_to_utm(sf_coord);
        assert_eq!(utm.zone, 10, "San Francisco must be in UTM Zone 10");
        assert!(utm.north, "San Francisco is in Northern hemisphere");
        assert!(utm.easting > 500_000.0 && utm.easting < 600_000.0);
        assert!(utm.northing > 4_100_000.0 && utm.northing < 4_200_000.0);

        let navd88 = VerticalDatumMetadata::san_francisco_navd88();
        assert_eq!(navd88.datum, VerticalDatum::Navd88);
        let ellip = navd88.orthometric_to_ellipsoidal(10.0);
        assert!((ellip - (10.0 - 32.4)).abs() < 1e-6);
    }
}
