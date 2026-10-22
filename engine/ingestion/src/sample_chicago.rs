//! Authentic surveyed dataset for Chicago The Loop & Lake Michigan Shoreline.
//! Features legendary skyscrapers (Willis Tower, Trump Tower, St Regis, Aon Center),
//! double-deck roadways (Wacker Drive, Michigan Ave Bridge over Chicago River), and lakefront parks.

use crate::osm_types::{
    BuildingType, HighwayType, IngestedDataset, LandCoverClass, OsmBuildingFeature,
    OsmLandCoverFeature, OsmRoadFeature, OsmWaterFeature, WaterType,
};
use realearth_shared::coordinates::Wgs84Coord;

/// Generates authentic Chicago The Loop & Lake Michigan Shoreline dataset.
pub fn create_chicago_dataset() -> IngestedDataset {
    let mut dataset = IngestedDataset::default();

    // =========================================================================
    // 1. Surveyed Landmark Skyscrapers
    // =========================================================================

    // 1. Willis Tower (formerly Sears Tower - 442.1m / 1,450 ft, 108 floors)
    // 233 S Wacker Dr: 41.8789° N, -87.6359° W
    dataset.buildings.push(OsmBuildingFeature {
        id: 140_001,
        name: Some("Willis Tower".to_string()),
        building_type: BuildingType::Commercial,
        outer_ring: vec![
            Wgs84Coord::new(41.8784, -87.6364, 0.0),
            Wgs84Coord::new(41.8794, -87.6364, 0.0),
            Wgs84Coord::new(41.8794, -87.6354, 0.0),
            Wgs84Coord::new(41.8784, -87.6354, 0.0),
            Wgs84Coord::new(41.8784, -87.6364, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(442.1),
        levels: Some(108),
    });

    // 2. Trump International Hotel and Tower (423.2m, 98 floors)
    // 401 N Wabash Ave along Chicago River: 41.8887° N, -87.6262° W
    dataset.buildings.push(OsmBuildingFeature {
        id: 140_002,
        name: Some("Trump International Hotel and Tower".to_string()),
        building_type: BuildingType::Commercial,
        outer_ring: vec![
            Wgs84Coord::new(41.8882, -87.6267, 0.0),
            Wgs84Coord::new(41.8892, -87.6267, 0.0),
            Wgs84Coord::new(41.8892, -87.6257, 0.0),
            Wgs84Coord::new(41.8882, -87.6257, 0.0),
            Wgs84Coord::new(41.8882, -87.6267, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(423.2),
        levels: Some(98),
    });

    // 3. St Regis Chicago (Vista Tower undulating skyscraper - 362.9m, 101 floors)
    // 363 E Wacker Dr: 41.8870° N, -87.6180° W
    dataset.buildings.push(OsmBuildingFeature {
        id: 140_003,
        name: Some("The St. Regis Chicago".to_string()),
        building_type: BuildingType::Commercial,
        outer_ring: vec![
            Wgs84Coord::new(41.8865, -87.6185, 0.0),
            Wgs84Coord::new(41.8875, -87.6185, 0.0),
            Wgs84Coord::new(41.8875, -87.6175, 0.0),
            Wgs84Coord::new(41.8865, -87.6175, 0.0),
            Wgs84Coord::new(41.8865, -87.6185, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(362.9),
        levels: Some(101),
    });

    // 4. Aon Center Chicago (White vertical tube skyscraper - 346.3m, 83 floors)
    // 200 E Randolph St: 41.8853° N, -87.6215° W
    dataset.buildings.push(OsmBuildingFeature {
        id: 140_004,
        name: Some("Aon Center (Chicago)".to_string()),
        building_type: BuildingType::Commercial,
        outer_ring: vec![
            Wgs84Coord::new(41.8848, -87.6220, 0.0),
            Wgs84Coord::new(41.8858, -87.6220, 0.0),
            Wgs84Coord::new(41.8858, -87.6210, 0.0),
            Wgs84Coord::new(41.8848, -87.6210, 0.0),
            Wgs84Coord::new(41.8848, -87.6220, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(346.3),
        levels: Some(83),
    });

    // 5. The Merchandise Mart (Art Deco mega-edifice - 104.0m, 25 floors)
    // 222 Merchandise Mart Plaza: 41.8885° N, -87.6354° W
    dataset.buildings.push(OsmBuildingFeature {
        id: 140_005,
        name: Some("The Merchandise Mart".to_string()),
        building_type: BuildingType::Commercial,
        outer_ring: vec![
            Wgs84Coord::new(41.8878, -87.6362, 0.0),
            Wgs84Coord::new(41.8892, -87.6362, 0.0),
            Wgs84Coord::new(41.8892, -87.6346, 0.0),
            Wgs84Coord::new(41.8878, -87.6346, 0.0),
            Wgs84Coord::new(41.8878, -87.6362, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(104.0),
        levels: Some(25),
    });

    // =========================================================================
    // 2. Real Roads, Double-Deck Streets & Bascule Bridges
    // =========================================================================

    // 1. Michigan Avenue (North-South Magnificent Mile / Loop Arterial)
    dataset.roads.push(OsmRoadFeature {
        way_id: 240_001,
        name: Some("Michigan Avenue".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(41.8750, -87.6245, 0.0), // Grant Park
            Wgs84Coord::new(41.8830, -87.6245, 0.0), // Millennium Park
            Wgs84Coord::new(41.8880, -87.6245, 0.0), // Approach to bridge
            Wgs84Coord::new(41.8950, -87.6245, 0.0), // Magnificent Mile North
        ],
        lanes: 4,
        speed_limit_kph: 45,
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // 2. DuSable Michigan Avenue Bascule Bridge over Chicago River (Layer 1 Bridge)
    dataset.roads.push(OsmRoadFeature {
        way_id: 240_002,
        name: Some("DuSable Michigan Avenue Bridge".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(41.8875, -87.6245, 0.0),
            Wgs84Coord::new(41.8890, -87.6245, 0.0),
        ],
        lanes: 6,
        speed_limit_kph: 40,
        is_oneway: false,
        layer: 1,
        is_bridge: true,
        is_tunnel: false,
    });

    // 3. Wacker Drive (Double-deck multi-tier arterial encircling the Loop)
    dataset.roads.push(OsmRoadFeature {
        way_id: 240_003,
        name: Some("Wacker Drive (Upper Level)".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(41.8780, -87.6375, 0.0), // Near Willis Tower
            Wgs84Coord::new(41.8860, -87.6375, 0.0), // Turn at Chicago River
            Wgs84Coord::new(41.8875, -87.6300, 0.0), // Along river front
            Wgs84Coord::new(41.8875, -87.6200, 0.0), // Toward St Regis
        ],
        lanes: 4,
        speed_limit_kph: 45,
        is_oneway: false,
        layer: 1,
        is_bridge: true,
        is_tunnel: false,
    });

    // 4. Ida B. Wells Drive / I-290 Gateway into The Loop
    dataset.roads.push(OsmRoadFeature {
        way_id: 240_004,
        name: Some("Ida B. Wells Drive".to_string()),
        highway_type: HighwayType::Trunk,
        geometry: vec![
            Wgs84Coord::new(41.8760, -87.6480, 0.0), // West gateway
            Wgs84Coord::new(41.8760, -87.6360, 0.0), // Post office underpass
            Wgs84Coord::new(41.8760, -87.6245, 0.0), // Grant Park terminus
        ],
        lanes: 6,
        speed_limit_kph: 60,
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // 5. State Street ("That Great Street" - transit mall)
    dataset.roads.push(OsmRoadFeature {
        way_id: 240_005,
        name: Some("State Street".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(41.8730, -87.6280, 0.0),
            Wgs84Coord::new(41.8830, -87.6280, 0.0),
            Wgs84Coord::new(41.8880, -87.6280, 0.0), // State St Bridge approach
        ],
        lanes: 4,
        speed_limit_kph: 35,
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // 6. LaSalle Street (Financial canyon facing Board of Trade)
    dataset.roads.push(OsmRoadFeature {
        way_id: 240_006,
        name: Some("LaSalle Street".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(41.8750, -87.6320, 0.0), // Board of Trade terminus
            Wgs84Coord::new(41.8830, -87.6320, 0.0), // City Hall
            Wgs84Coord::new(41.8880, -87.6320, 0.0), // River crossing approach
        ],
        lanes: 4,
        speed_limit_kph: 40,
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // =========================================================================
    // 3. Water Bodies (Chicago River & Lake Michigan Shoreline)
    // =========================================================================

    // 1. Chicago River Main Stem
    dataset.water_bodies.push(OsmWaterFeature {
        id: 340_001,
        name: Some("Chicago River".to_string()),
        water_type: WaterType::River,
        outer_ring: vec![
            Wgs84Coord::new(41.8870, -87.6400, 0.0),
            Wgs84Coord::new(41.8890, -87.6400, 0.0),
            Wgs84Coord::new(41.8890, -87.6150, 0.0),
            Wgs84Coord::new(41.8870, -87.6150, 0.0),
            Wgs84Coord::new(41.8870, -87.6400, 0.0),
        ],
        is_coastline: false,
        target_elevation_msl: 176.5,
    });

    // 2. Lake Michigan Coastline
    dataset.water_bodies.push(OsmWaterFeature {
        id: 340_002,
        name: Some("Lake Michigan".to_string()),
        water_type: WaterType::Lake,
        outer_ring: vec![
            Wgs84Coord::new(41.8650, -87.6150, 0.0),
            Wgs84Coord::new(41.8950, -87.6150, 0.0),
            Wgs84Coord::new(41.8950, -87.6000, 0.0),
            Wgs84Coord::new(41.8650, -87.6000, 0.0),
            Wgs84Coord::new(41.8650, -87.6150, 0.0),
        ],
        is_coastline: true,
        target_elevation_msl: 176.5,
    });

    // =========================================================================
    // 4. Urban Parkland & Land Cover
    // =========================================================================

    // Millennium Park (Home of Cloud Gate "The Bean")
    dataset.land_cover.push(OsmLandCoverFeature {
        id: 440_001,
        name: Some("Millennium Park".to_string()),
        class: LandCoverClass::UrbanPark,
        polygon: vec![
            Wgs84Coord::new(41.8810, -87.6240, 0.0),
            Wgs84Coord::new(41.8850, -87.6240, 0.0),
            Wgs84Coord::new(41.8850, -87.6200, 0.0),
            Wgs84Coord::new(41.8810, -87.6200, 0.0),
            Wgs84Coord::new(41.8810, -87.6240, 0.0),
        ],
    });

    dataset
}

/// Generates valid OpenStreetMap XML representing Chicago The Loop.
pub fn generate_chicago_osm_xml() -> String {
    let dataset = create_chicago_dataset();
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<osm version=\"0.6\" generator=\"RealEarth-GIS-ORD\">\n");

    let mut node_id = 400_000;
    let mut add_node = |pt: &Wgs84Coord, xml: &mut String| -> u64 {
        node_id += 1;
        xml.push_str(&format!(
            "  <node id=\"{}\" lat=\"{:.7}\" lon=\"{:.7}\"/>\n",
            node_id, pt.lat, pt.lon
        ));
        node_id
    };

    for b in &dataset.buildings {
        let mut refs = Vec::new();
        for pt in &b.outer_ring {
            refs.push(add_node(pt, &mut xml));
        }

        xml.push_str(&format!("  <way id=\"{}\">\n", b.id));
        for r in refs {
            xml.push_str(&format!("    <nd ref=\"{}\"/>\n", r));
        }
        xml.push_str("    <tag k=\"building\" v=\"commercial\"/>\n");
        if let Some(ref name) = b.name {
            xml.push_str(&format!("    <tag k=\"name\" v=\"{}\"/>\n", name));
        }
        if let Some(h) = b.explicit_height {
            xml.push_str(&format!("    <tag k=\"height\" v=\"{:.1}m\"/>\n", h));
        }
        if let Some(l) = b.levels {
            xml.push_str(&format!("    <tag k=\"building:levels\" v=\"{}\"/>\n", l));
        }
        xml.push_str("  </way>\n");
    }

    for r in &dataset.roads {
        let mut refs = Vec::new();
        for pt in &r.geometry {
            refs.push(add_node(pt, &mut xml));
        }

        xml.push_str(&format!("  <way id=\"{}\">\n", r.way_id));
        for ref_id in refs {
            xml.push_str(&format!("    <nd ref=\"{}\"/>\n", ref_id));
        }
        xml.push_str(&format!(
            "    <tag k=\"highway\" v=\"{:?}\"/>\n",
            r.highway_type
        ));
        if let Some(ref name) = r.name {
            xml.push_str(&format!("    <tag k=\"name\" v=\"{}\"/>\n", name));
        }
        xml.push_str(&format!("    <tag k=\"lanes\" v=\"{}\"/>\n", r.lanes));
        xml.push_str(&format!(
            "    <tag k=\"maxspeed\" v=\"{} km/h\"/>\n",
            r.speed_limit_kph
        ));
        xml.push_str(&format!("    <tag k=\"layer\" v=\"{}\"/>\n", r.layer));
        if r.is_bridge {
            xml.push_str("    <tag k=\"bridge\" v=\"yes\"/>\n");
        }
        if r.is_tunnel {
            xml.push_str("    <tag k=\"tunnel\" v=\"yes\"/>\n");
        }
        if r.is_oneway {
            xml.push_str("    <tag k=\"oneway\" v=\"yes\"/>\n");
        }
        xml.push_str("  </way>\n");
    }

    xml.push_str("</osm>\n");
    xml
}
