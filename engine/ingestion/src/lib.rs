//! Geospatial data ingestion for OpenStreetMap vector and Digital Elevation Model rasters.

pub mod acquisition;
pub mod dem_grid;
pub mod osm_parser;
pub mod osm_types;
pub mod sample_bengaluru;
pub mod sample_chennai;
pub mod sample_chicago;
pub mod sample_corridors;
pub mod sample_hyderabad;
pub mod sample_los_angeles;
pub mod sample_miami;
pub mod sample_mysuru;
pub mod sample_new_york;
pub mod sample_san_francisco;
pub mod sample_seattle;
pub mod us_cities;

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
pub use sample_chennai::{create_chennai_dataset, generate_chennai_osm_xml};
pub use sample_chicago::{create_chicago_dataset, generate_chicago_osm_xml};
pub use sample_corridors::{
    create_intercity_corridors_dataset, generate_intercity_corridors_osm_xml,
};
pub use sample_hyderabad::{create_hyderabad_dataset, generate_hyderabad_osm_xml};
pub use sample_los_angeles::{create_los_angeles_dataset, generate_los_angeles_osm_xml};
pub use sample_miami::{create_miami_dataset, generate_miami_osm_xml};
pub use sample_mysuru::{create_mysuru_dataset, generate_mysuru_osm_xml};
pub use sample_new_york::{create_new_york_dataset, generate_new_york_osm_xml};
pub use sample_san_francisco::{create_san_francisco_dataset, generate_san_francisco_osm_xml};
pub use sample_seattle::{create_seattle_dataset, generate_seattle_osm_xml};
pub use us_cities::generate_us_city_dataset;

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
        assert_eq!(dataset.roads.len(), 7, "Expected 7 road segments");

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

    #[test]
    fn test_mysuru_osm_xml_and_dem() {
        let xml = generate_mysuru_osm_xml();
        let dataset = parse_osm_xml(&xml).expect("Failed to parse Mysuru OSM XML");
        assert_eq!(dataset.buildings.len(), 4);
        assert_eq!(dataset.roads.len(), 5);

        let palace = dataset
            .buildings
            .iter()
            .find(|b| b.name.as_deref() == Some("Mysore Palace (Amba Vilas)"))
            .expect("Palace missing");
        assert_eq!(palace.explicit_height, Some(45.0));

        let dem = DemGrid::create_mysuru_dem(32, 32);
        let chamundi_coord = Wgs84Coord::new(12.2745, 76.6710, 0.0);
        let elev = dem.sample_bilinear(&chamundi_coord);
        assert!(
            elev >= 1000.0,
            "Chamundi Hill elevation should be >= 1000m (was {:.1}m)",
            elev
        );
    }

    #[test]
    fn test_chennai_osm_xml_and_coastal_dem() {
        let xml = generate_chennai_osm_xml();
        let dataset = parse_osm_xml(&xml).expect("Failed to parse Chennai OSM XML");
        assert_eq!(dataset.buildings.len(), 4);
        assert_eq!(dataset.roads.len(), 5);

        let napier = dataset
            .roads
            .iter()
            .find(|r| r.name.as_deref() == Some("Napier Bridge (Cooum River Crossing)"))
            .expect("Napier Bridge missing");
        assert_eq!(napier.layer, 1);
        assert!(napier.is_bridge);

        let dem = DemGrid::create_chennai_coastal_dem(32, 32);
        let beach_coord = Wgs84Coord::new(13.0600, 80.2900, 0.0);
        let elev = dem.sample_bilinear(&beach_coord);
        assert!(
            elev <= 5.0,
            "Marina beach elevation should be near sea level (was {:.1}m)",
            elev
        );
    }

    #[test]
    fn test_hyderabad_osm_xml_and_dem() {
        let xml = generate_hyderabad_osm_xml();
        let dataset = parse_osm_xml(&xml).expect("Failed to parse Hyderabad OSM XML");
        assert_eq!(dataset.buildings.len(), 4);
        assert_eq!(dataset.roads.len(), 5);

        let pvnr = dataset
            .roads
            .iter()
            .find(|r| r.name.as_deref() == Some("PVNR Elevated Expressway"))
            .expect("PVNR missing");
        assert_eq!(pvnr.layer, 1);
        assert!(pvnr.is_bridge);

        let dem = DemGrid::create_hyderabad_plateau_dem(32, 32);
        let charminar_coord = Wgs84Coord::new(17.3616, 78.4747, 0.0);
        let elev = dem.sample_bilinear(&charminar_coord);
        assert!(
            (510.0..=560.0).contains(&elev),
            "Charminar elevation should be ~536m (was {:.1}m)",
            elev
        );
    }

    #[test]
    fn test_intercity_corridors_osm_xml_and_rivers() {
        let xml = generate_intercity_corridors_osm_xml();
        let dataset = parse_osm_xml(&xml).expect("Failed to parse Corridors OSM XML");
        assert_eq!(dataset.roads.len(), 13); // NH 275 (3), NH 48 (3), NH 44 (3) + 4 feeder links

        // Check Kaveri, Palar, and Tungabhadra bridges are elevated
        let bridges: Vec<_> = dataset.roads.iter().filter(|r| r.is_bridge).collect();
        assert_eq!(bridges.len(), 3);
        for b in bridges {
            assert_eq!(b.layer, 1);
        }
    }

    #[test]
    fn test_new_york_osm_xml_and_dem() {
        let xml = generate_new_york_osm_xml();
        let dataset = parse_osm_xml(&xml).expect("Failed to parse NYC OSM XML");
        assert!(dataset.buildings.len() >= 5);
        assert!(dataset.roads.len() >= 6);

        let wtc = dataset
            .buildings
            .iter()
            .find(|b| b.name.as_deref() == Some("One World Trade Center"))
            .expect("One WTC missing");
        assert_eq!(wtc.explicit_height, Some(541.3));

        let dem = DemGrid::create_new_york_coastal_dem(32, 32);
        let center = dem.bounds.center();
        let elev = dem.sample_bilinear(&center);
        assert!(
            (0.0..=60.0).contains(&elev),
            "NYC elevation should be coastal lowland/bedrock (was {:.1}m)",
            elev
        );
    }

    #[test]
    fn test_los_angeles_osm_xml_and_dem() {
        let xml = generate_los_angeles_osm_xml();
        let dataset = parse_osm_xml(&xml).expect("Failed to parse LA OSM XML");
        assert!(dataset.buildings.len() >= 5);
        assert!(dataset.roads.len() >= 6);

        let wilshire = dataset
            .buildings
            .iter()
            .find(|b| b.name.as_deref() == Some("Wilshire Grand Center"))
            .expect("Wilshire Grand missing");
        assert_eq!(wilshire.explicit_height, Some(335.3));

        let dem = DemGrid::create_los_angeles_basin_dem(32, 32);
        let center = dem.bounds.center();
        let elev = dem.sample_bilinear(&center);
        assert!(
            (50.0..=200.0).contains(&elev),
            "LA Basin elevation should be around 80-150m (was {:.1}m)",
            elev
        );
    }

    #[test]
    fn test_chicago_osm_xml_and_dem() {
        let xml = generate_chicago_osm_xml();
        let dataset = parse_osm_xml(&xml).expect("Failed to parse Chicago OSM XML");
        assert!(dataset.buildings.len() >= 5);
        assert!(dataset.roads.len() >= 6);

        let willis = dataset
            .buildings
            .iter()
            .find(|b| b.name.as_deref() == Some("Willis Tower"))
            .expect("Willis Tower missing");
        assert_eq!(willis.explicit_height, Some(442.1));

        let dem = DemGrid::create_chicago_loop_dem(32, 32);
        let center = dem.bounds.center();
        let elev = dem.sample_bilinear(&center);
        assert!(
            (170.0..=195.0).contains(&elev),
            "Chicago elevation should be Lake Michigan plain ~176-182m (was {:.1}m)",
            elev
        );
    }

    #[test]
    fn test_seattle_osm_xml_and_dem() {
        let xml = generate_seattle_osm_xml();
        let dataset = parse_osm_xml(&xml).expect("Failed to parse Seattle OSM XML");
        assert!(dataset.buildings.len() >= 5);
        assert!(dataset.roads.len() >= 6);

        let needle = dataset
            .buildings
            .iter()
            .find(|b| b.name.as_deref() == Some("Space Needle"))
            .expect("Space Needle missing");
        assert_eq!(needle.explicit_height, Some(184.4));

        let dem = DemGrid::create_seattle_coastal_dem(32, 32);
        let center = dem.bounds.center();
        let elev = dem.sample_bilinear(&center);
        assert!(
            (0.0..=120.0).contains(&elev),
            "Seattle coastal elevation should be 0-100m (was {:.1}m)",
            elev
        );
    }

    #[test]
    fn test_miami_osm_xml_and_dem() {
        let xml = generate_miami_osm_xml();
        let dataset = parse_osm_xml(&xml).expect("Failed to parse Miami OSM XML");
        assert!(dataset.buildings.len() >= 5);
        assert!(dataset.roads.len() >= 6);

        let panorama = dataset
            .buildings
            .iter()
            .find(|b| b.name.as_deref() == Some("Panorama Tower"))
            .expect("Panorama Tower missing");
        assert_eq!(panorama.explicit_height, Some(264.6));

        let dem = DemGrid::create_miami_coastal_dem(32, 32);
        let center = dem.bounds.center();
        let elev = dem.sample_bilinear(&center);
        assert!(
            (0.0..=10.0).contains(&elev),
            "Miami elevation should be low-lying coastal 0-8m (was {:.1}m)",
            elev
        );
    }
}
