//! Geospatial data ingestion for OpenStreetMap vector and Digital Elevation Model rasters.

pub mod acquisition;
pub mod dem_grid;
pub mod osm_parser;
pub mod osm_types;
pub mod sample_bengaluru;
pub mod sample_san_francisco;

pub use acquisition::{
    compute_sha256, verify_checksum, AcquisitionEngine, RegionAcquisitionManifest, SourceProvenance,
};
pub use dem_grid::DemGrid;
pub use osm_parser::parse_osm_xml;
pub use osm_types::{
    BuildingType, HighwayType, IngestedDataset, LandCoverClass, OsmBuildingFeature,
    OsmLandCoverFeature, OsmRoadFeature, OsmWaterFeature, WaterType,
};
pub use sample_bengaluru::{create_bengaluru_central_dataset, generate_bengaluru_osm_xml};
pub use sample_san_francisco::{create_san_francisco_dataset, generate_san_francisco_osm_xml};

#[cfg(test)]
mod tests {
    use super::*;
    use realearth_shared::coordinates::Wgs84Coord;
    use realearth_shared::region::RegionProfile;
    use tempfile::tempdir;

    #[test]
    fn test_bengaluru_osm_xml_parsing() {
        let xml = generate_bengaluru_osm_xml();
        let dataset = parse_osm_xml(&xml).expect("Failed to parse generated Bengaluru OSM XML");

        assert_eq!(dataset.buildings.len(), 4, "Expected 4 landmark buildings");
        assert_eq!(dataset.roads.len(), 4, "Expected 4 road segments");

        // Verify Vidhana Soudha
        let vidhana = dataset
            .buildings
            .iter()
            .find(|b| b.name.as_deref() == Some("Vidhana Soudha"))
            .expect("Vidhana Soudha not found");
        assert_eq!(vidhana.explicit_height, Some(46.0));
        assert_eq!(vidhana.levels, Some(4));
        assert_eq!(vidhana.outer_ring.len(), 5);

        // Verify Flyover Grade Separation
        let flyover = dataset
            .roads
            .iter()
            .find(|r| r.name.as_deref() == Some("Cubbon Park Elevated Flyover"))
            .expect("Flyover not found");
        assert_eq!(flyover.layer, 1);
        assert!(flyover.is_bridge);

        let ground_road = dataset
            .roads
            .iter()
            .find(|r| r.name.as_deref() == Some("Kasturba Road"))
            .expect("Kasturba Road not found");
        assert_eq!(ground_road.layer, 0);
        assert!(!ground_road.is_bridge);
    }

    #[test]
    fn test_san_francisco_osm_xml_parsing_and_landmarks() {
        let xml = generate_san_francisco_osm_xml();
        let dataset = parse_osm_xml(&xml).expect("Failed to parse San Francisco OSM XML");

        assert_eq!(
            dataset.buildings.len(),
            5,
            "Expected 5 San Francisco skyscrapers/landmarks"
        );
        assert_eq!(dataset.roads.len(), 6, "Expected 6 road segments");
        assert_eq!(
            dataset.water_bodies.len(),
            1,
            "Expected San Francisco Bay water body"
        );
        assert_eq!(dataset.land_cover.len(), 2, "Expected 2 urban parks");

        // 1. Verify Salesforce Tower (326m, 61 floors)
        let salesforce = dataset
            .buildings
            .iter()
            .find(|b| b.name.as_deref() == Some("Salesforce Tower"))
            .expect("Salesforce Tower not found");
        assert_eq!(salesforce.explicit_height, Some(326.0));
        assert_eq!(salesforce.levels, Some(61));

        // 2. Verify Bay Bridge I-80 Approach (Motorway, Bridge layer 1)
        let bay_bridge = dataset
            .roads
            .iter()
            .find(|r| r.name.as_deref() == Some("Interstate 80 Bay Bridge Approach"))
            .expect("Bay Bridge approach not found");
        assert_eq!(bay_bridge.layer, 1);
        assert!(bay_bridge.is_bridge);
        assert_eq!(bay_bridge.lanes, 5);
        assert_eq!(bay_bridge.speed_limit_kph, 80);

        // 3. Verify Broadway Tunnel (Subterranean layer -1)
        let tunnel = dataset
            .roads
            .iter()
            .find(|r| r.name.as_deref() == Some("Broadway Tunnel"))
            .expect("Broadway Tunnel not found");
        assert_eq!(tunnel.layer, -1);
        assert!(tunnel.is_tunnel);

        // 4. Verify San Francisco Bay coastline
        let sf_bay = &dataset.water_bodies[0];
        assert_eq!(sf_bay.name.as_deref(), Some("San Francisco Bay"));
        assert!(sf_bay.is_coastline);
        assert_eq!(sf_bay.target_elevation_msl, 0.0);
    }

    #[test]
    fn test_san_francisco_coastal_dem_sampling() {
        let dem = DemGrid::create_san_francisco_coastal_dem(64, 64);

        // 1. Embarcadero shoreline (Waterfront) should be close to sea level (0m - 5m)
        let embarcadero = Wgs84Coord::new(37.7950, -122.3935, 0.0);
        let elev_shore = dem.sample_bilinear(&embarcadero);
        assert!(
            elev_shore <= 15.0,
            "Embarcadero waterfront elevation too high: {} m",
            elev_shore
        );

        // 2. Nob Hill crest (California St) should rise steeply (> 70m)
        let nob_hill = Wgs84Coord::new(37.7925, -122.4110, 0.0);
        let elev_nob = dem.sample_bilinear(&nob_hill);
        assert!(
            elev_nob >= 60.0,
            "Nob Hill elevation should be >= 60m, was: {} m",
            elev_nob
        );

        // 3. Telegraph Hill (Coit Tower) should have pronounced local peak
        let coit_tower = Wgs84Coord::new(37.8024, -122.4058, 0.0);
        let elev_coit = dem.sample_bilinear(&coit_tower);
        assert!(
            elev_coit >= 40.0,
            "Telegraph Hill elevation should be elevated, was: {} m",
            elev_coit
        );
    }

    #[test]
    fn test_acquisition_engine_sha256_and_manifest() {
        let temp = tempdir().expect("tempdir failed");
        let engine = AcquisitionEngine::new(temp.path());

        let sf_profile = RegionProfile::san_francisco_downtown();
        let (dataset, dem, manifest) = engine
            .acquire_region(&sf_profile)
            .expect("Failed to acquire San Francisco region");

        assert_eq!(manifest.region_id, "san-francisco-downtown");
        assert_eq!(manifest.sources.len(), 4);
        assert_eq!(dataset.buildings.len(), 5);
        assert_eq!(dem.bounds.min_lat, 37.7800);

        // Verify SHA-256 integrity
        let test_str = b"RealEarth World Platform";
        let hash = compute_sha256(test_str);
        assert!(verify_checksum(test_str, &hash));
    }

    #[test]
    fn test_dem_grid_elevation_sampling() {
        let dem = DemGrid::create_bengaluru_plateau_dem(64, 64);
        assert_eq!(dem.width, 64);
        assert_eq!(dem.height, 64);

        // Test center coordinate of Bengaluru Central
        let center = dem.bounds.center();
        let elev = dem.sample_bilinear(&center);

        // Bengaluru plateau elevation is in range 900m - 940m
        assert!(
            (900.0..=940.0).contains(&elev),
            "Elevation at Bengaluru center was unexpected: {} m",
            elev
        );
    }

    #[test]
    fn test_dem_bounds_clamping() {
        let dem = DemGrid::create_bengaluru_plateau_dem(32, 32);
        // Coordinate slightly outside bounds should clamp smoothly without panicking
        let outside = Wgs84Coord::new(12.9500, 77.5700, 0.0);
        let elev = dem.sample_bilinear(&outside);
        assert!(elev > 0.0);
    }
}
