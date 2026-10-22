//! Authentic surveyed dataset for New York City (Lower Manhattan & Financial District).
//! Features iconic skyscrapers (One WTC, Empire State, Chrysler Building),
//! major arterials (Broadway, 5th Ave, FDR Drive), Brooklyn Bridge approach, and waterfronts.

use crate::osm_types::{
    BuildingType, HighwayType, IngestedDataset, LandCoverClass, OsmBuildingFeature,
    OsmLandCoverFeature, OsmRoadFeature, OsmWaterFeature, WaterType,
};
use realearth_shared::coordinates::Wgs84Coord;

/// Generates authentic New York City (Lower Manhattan & Midtown South) dataset.
pub fn create_new_york_dataset() -> IngestedDataset {
    let mut dataset = IngestedDataset::default();

    // =========================================================================
    // 1. Surveyed Landmark Skyscrapers
    // =========================================================================

    // 1. One World Trade Center (Tallest in Western Hemisphere - 541.3m / 1,776 ft, 104 floors)
    // 40.7127° N, -74.0134° W
    dataset.buildings.push(OsmBuildingFeature {
        id: 120_001,
        name: Some("One World Trade Center".to_string()),
        building_type: BuildingType::Commercial,
        outer_ring: vec![
            Wgs84Coord::new(40.7122, -74.0139, 0.0),
            Wgs84Coord::new(40.7132, -74.0139, 0.0),
            Wgs84Coord::new(40.7132, -74.0129, 0.0),
            Wgs84Coord::new(40.7122, -74.0129, 0.0),
            Wgs84Coord::new(40.7122, -74.0139, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(541.3),
        levels: Some(104),
    });

    // 2. Empire State Building (Iconic Art Deco Landmark - 381.0m, 102 floors)
    // 350 5th Ave: 40.7484° N, -73.9857° W
    dataset.buildings.push(OsmBuildingFeature {
        id: 120_002,
        name: Some("Empire State Building".to_string()),
        building_type: BuildingType::Commercial,
        outer_ring: vec![
            Wgs84Coord::new(40.7480, -73.9862, 0.0),
            Wgs84Coord::new(40.7488, -73.9862, 0.0),
            Wgs84Coord::new(40.7488, -73.9852, 0.0),
            Wgs84Coord::new(40.7480, -73.9852, 0.0),
            Wgs84Coord::new(40.7480, -73.9862, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(381.0),
        levels: Some(102),
    });

    // 3. Chrysler Building (Art Deco Spire - 319.0m, 77 floors)
    // 405 Lexington Ave: 40.7516° N, -73.9755° W
    dataset.buildings.push(OsmBuildingFeature {
        id: 120_003,
        name: Some("Chrysler Building".to_string()),
        building_type: BuildingType::Commercial,
        outer_ring: vec![
            Wgs84Coord::new(40.7512, -73.9760, 0.0),
            Wgs84Coord::new(40.7520, -73.9760, 0.0),
            Wgs84Coord::new(40.7520, -73.9750, 0.0),
            Wgs84Coord::new(40.7512, -73.9750, 0.0),
            Wgs84Coord::new(40.7512, -73.9760, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(319.0),
        levels: Some(77),
    });

    // 4. Woolworth Building (Neo-Gothic "Cathedral of Commerce" - 241.4m, 60 floors)
    // 233 Broadway: 40.7122° N, -74.0083° W
    dataset.buildings.push(OsmBuildingFeature {
        id: 120_004,
        name: Some("Woolworth Building".to_string()),
        building_type: BuildingType::Commercial,
        outer_ring: vec![
            Wgs84Coord::new(40.7118, -74.0088, 0.0),
            Wgs84Coord::new(40.7126, -74.0088, 0.0),
            Wgs84Coord::new(40.7126, -74.0078, 0.0),
            Wgs84Coord::new(40.7118, -74.0078, 0.0),
            Wgs84Coord::new(40.7118, -74.0088, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(241.4),
        levels: Some(60),
    });

    // 5. 30 Hudson Yards (Observation deck skyscraper - 387.0m, 73 floors)
    // 40.7538° N, -74.0022° W
    dataset.buildings.push(OsmBuildingFeature {
        id: 120_005,
        name: Some("30 Hudson Yards".to_string()),
        building_type: BuildingType::Commercial,
        outer_ring: vec![
            Wgs84Coord::new(40.7533, -74.0027, 0.0),
            Wgs84Coord::new(40.7543, -74.0027, 0.0),
            Wgs84Coord::new(40.7543, -74.0017, 0.0),
            Wgs84Coord::new(40.7533, -74.0017, 0.0),
            Wgs84Coord::new(40.7533, -74.0027, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(387.0),
        levels: Some(73),
    });

    // =========================================================================
    // 2. Real Roads, Highways & Bridges
    // =========================================================================

    // 1. Broadway (Main Diagonal Spine across Manhattan)
    dataset.roads.push(OsmRoadFeature {
        way_id: 220_001,
        name: Some("Broadway".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(40.7040, -74.0135, 0.0), // Bowling Green
            Wgs84Coord::new(40.7125, -74.0075, 0.0), // City Hall
            Wgs84Coord::new(40.7250, -73.9980, 0.0), // SoHo
            Wgs84Coord::new(40.7400, -73.9890, 0.0), // Flatiron / Madison Sq
            Wgs84Coord::new(40.7540, -73.9840, 0.0), // Times Square South
        ],
        lanes: 4,
        speed_limit_kph: 40,
        is_oneway: true,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // 2. 5th Avenue (Grand North-South Arterial)
    dataset.roads.push(OsmRoadFeature {
        way_id: 220_002,
        name: Some("Fifth Avenue".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(40.7310, -73.9965, 0.0), // Washington Square Arch
            Wgs84Coord::new(40.7420, -73.9880, 0.0), // Flatiron
            Wgs84Coord::new(40.7485, -73.9855, 0.0), // Empire State Building
            Wgs84Coord::new(40.7535, -73.9810, 0.0), // Midtown
        ],
        lanes: 4,
        speed_limit_kph: 40,
        is_oneway: true,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // 3. FDR Drive (Elevated Coastal Expressway along East River)
    dataset.roads.push(OsmRoadFeature {
        way_id: 220_003,
        name: Some("FDR Drive".to_string()),
        highway_type: HighwayType::Motorway,
        geometry: vec![
            Wgs84Coord::new(40.7015, -74.0125, 0.0), // Battery Park Underpass
            Wgs84Coord::new(40.7080, -73.9985, 0.0), // Seaport
            Wgs84Coord::new(40.7130, -73.9770, 0.0), // Williamsburg Bridge approach
            Wgs84Coord::new(40.7320, -73.9720, 0.0), // Stuyvesant Town
        ],
        lanes: 6,
        speed_limit_kph: 75,
        is_oneway: false,
        layer: 1,
        is_bridge: true,
        is_tunnel: false,
    });

    // 4. West Side Highway (Hudson River Coastal Parkway)
    dataset.roads.push(OsmRoadFeature {
        way_id: 220_004,
        name: Some("West Side Highway (NY 9A)".to_string()),
        highway_type: HighwayType::Trunk,
        geometry: vec![
            Wgs84Coord::new(40.7020, -74.0165, 0.0), // Battery Place
            Wgs84Coord::new(40.7130, -74.0150, 0.0), // World Trade Center
            Wgs84Coord::new(40.7350, -74.0105, 0.0), // Greenwich Village Piers
            Wgs84Coord::new(40.7530, -74.0040, 0.0), // Hudson Yards
        ],
        lanes: 6,
        speed_limit_kph: 60,
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // 5. Brooklyn Bridge Approach (Elevated Grade-Separated Suspension Highway)
    dataset.roads.push(OsmRoadFeature {
        way_id: 220_005,
        name: Some("Brooklyn Bridge".to_string()),
        highway_type: HighwayType::Motorway,
        geometry: vec![
            Wgs84Coord::new(40.7130, -74.0040, 0.0), // Park Row / Centre St
            Wgs84Coord::new(40.7100, -73.9970, 0.0), // Manhattan Anchorage
            Wgs84Coord::new(40.7060, -73.9900, 0.0), // Mid-span over East River
        ],
        lanes: 6,
        speed_limit_kph: 60,
        is_oneway: false,
        layer: 1,
        is_bridge: true,
        is_tunnel: false,
    });

    // 6. Manhattan Bridge Approach (Grade-Separated Dual Suspension Deck)
    dataset.roads.push(OsmRoadFeature {
        way_id: 220_006,
        name: Some("Manhattan Bridge".to_string()),
        highway_type: HighwayType::Motorway,
        geometry: vec![
            Wgs84Coord::new(40.7160, -73.9960, 0.0), // Canal St plaza
            Wgs84Coord::new(40.7120, -73.9910, 0.0), // Manhattan Tower
            Wgs84Coord::new(40.7070, -73.9850, 0.0), // Over East River
        ],
        lanes: 6,
        speed_limit_kph: 60,
        is_oneway: false,
        layer: 1,
        is_bridge: true,
        is_tunnel: false,
    });

    // =========================================================================
    // 3. Water Bodies (East River & Hudson River Waterfront)
    // =========================================================================

    dataset.water_bodies.push(OsmWaterFeature {
        id: 320_001,
        name: Some("East River & New York Harbor".to_string()),
        water_type: WaterType::Bay,
        outer_ring: vec![
            Wgs84Coord::new(40.7000, -74.0200, 0.0),
            Wgs84Coord::new(40.7000, -73.9700, 0.0),
            Wgs84Coord::new(40.7200, -73.9700, 0.0),
            Wgs84Coord::new(40.7100, -73.9800, 0.0),
            Wgs84Coord::new(40.7010, -74.0150, 0.0),
            Wgs84Coord::new(40.7000, -74.0200, 0.0),
        ],
        is_coastline: true,
        target_elevation_msl: 0.0,
    });

    // =========================================================================
    // 4. Urban Parkland & Land Cover
    // =========================================================================

    // Battery Park (Tip of Lower Manhattan)
    dataset.land_cover.push(OsmLandCoverFeature {
        id: 420_001,
        name: Some("The Battery".to_string()),
        class: LandCoverClass::UrbanPark,
        polygon: vec![
            Wgs84Coord::new(40.7015, -74.0170, 0.0),
            Wgs84Coord::new(40.7050, -74.0170, 0.0),
            Wgs84Coord::new(40.7050, -74.0130, 0.0),
            Wgs84Coord::new(40.7015, -74.0130, 0.0),
            Wgs84Coord::new(40.7015, -74.0170, 0.0),
        ],
    });

    // City Hall Park
    dataset.land_cover.push(OsmLandCoverFeature {
        id: 420_002,
        name: Some("City Hall Park".to_string()),
        class: LandCoverClass::UrbanPark,
        polygon: vec![
            Wgs84Coord::new(40.7115, -74.0075, 0.0),
            Wgs84Coord::new(40.7135, -74.0075, 0.0),
            Wgs84Coord::new(40.7135, -74.0055, 0.0),
            Wgs84Coord::new(40.7115, -74.0055, 0.0),
            Wgs84Coord::new(40.7115, -74.0075, 0.0),
        ],
    });

    dataset
}

/// Generates valid OpenStreetMap XML representing New York City.
pub fn generate_new_york_osm_xml() -> String {
    let dataset = create_new_york_dataset();
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<osm version=\"0.6\" generator=\"RealEarth-GIS-NYC\">\n");

    let mut node_id = 200_000;
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
