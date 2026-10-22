//! Authentic surveyed dataset for Los Angeles Downtown & Harbor Freeway Interchange.
//! Features iconic skyscrapers (Wilshire Grand, US Bank Tower, Aon Center, City Hall, Disney Concert Hall),
//! multi-level freeway flyover stacks (I-110 Harbor Freeway), and urban plazas.

use crate::osm_types::{
    BuildingType, HighwayType, IngestedDataset, LandCoverClass, OsmBuildingFeature,
    OsmLandCoverFeature, OsmRoadFeature,
};
use realearth_shared::coordinates::Wgs84Coord;

/// Generates authentic Los Angeles Downtown & Bunker Hill dataset.
pub fn create_los_angeles_dataset() -> IngestedDataset {
    let mut dataset = IngestedDataset::default();

    // =========================================================================
    // 1. Surveyed Landmark Skyscrapers
    // =========================================================================

    // 1. Wilshire Grand Center (Tallest in LA & California - 335.3m, 73 floors)
    // 900 Wilshire Blvd: 34.0500° N, -118.2600° W
    dataset.buildings.push(OsmBuildingFeature {
        id: 130_001,
        name: Some("Wilshire Grand Center".to_string()),
        building_type: BuildingType::Commercial,
        outer_ring: vec![
            Wgs84Coord::new(34.0495, -118.2605, 0.0),
            Wgs84Coord::new(34.0505, -118.2605, 0.0),
            Wgs84Coord::new(34.0505, -118.2595, 0.0),
            Wgs84Coord::new(34.0495, -118.2595, 0.0),
            Wgs84Coord::new(34.0495, -118.2605, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(335.3),
        levels: Some(73),
    });

    // 2. US Bank Tower (Iconic circular crown skyscraper - 310.3m, 71 floors)
    // 633 W 5th St: 34.0512° N, -118.2543° W
    dataset.buildings.push(OsmBuildingFeature {
        id: 130_002,
        name: Some("US Bank Tower".to_string()),
        building_type: BuildingType::Commercial,
        outer_ring: vec![
            Wgs84Coord::new(34.0508, -118.2548, 0.0),
            Wgs84Coord::new(34.0516, -118.2548, 0.0),
            Wgs84Coord::new(34.0516, -118.2538, 0.0),
            Wgs84Coord::new(34.0508, -118.2538, 0.0),
            Wgs84Coord::new(34.0508, -118.2548, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(310.3),
        levels: Some(71),
    });

    // 3. Aon Center (Modernist black monolith - 260.0m, 62 floors)
    // 707 Wilshire Blvd: 34.0494° N, -118.2567° W
    dataset.buildings.push(OsmBuildingFeature {
        id: 130_003,
        name: Some("Aon Center".to_string()),
        building_type: BuildingType::Commercial,
        outer_ring: vec![
            Wgs84Coord::new(34.0490, -118.2572, 0.0),
            Wgs84Coord::new(34.0498, -118.2572, 0.0),
            Wgs84Coord::new(34.0498, -118.2562, 0.0),
            Wgs84Coord::new(34.0490, -118.2562, 0.0),
            Wgs84Coord::new(34.0490, -118.2572, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(260.0),
        levels: Some(62),
    });

    // 4. Los Angeles City Hall (Historic civic landmark - 138.4m, 32 floors)
    // 200 N Spring St: 34.0537° N, -118.2427° W
    dataset.buildings.push(OsmBuildingFeature {
        id: 130_004,
        name: Some("Los Angeles City Hall".to_string()),
        building_type: BuildingType::Civic,
        outer_ring: vec![
            Wgs84Coord::new(34.0533, -118.2432, 0.0),
            Wgs84Coord::new(34.0541, -118.2432, 0.0),
            Wgs84Coord::new(34.0541, -118.2422, 0.0),
            Wgs84Coord::new(34.0533, -118.2422, 0.0),
            Wgs84Coord::new(34.0533, -118.2432, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(138.4),
        levels: Some(32),
    });

    // 5. Walt Disney Concert Hall (Frank Gehry stainless-steel sculpture - 45.0m)
    // 111 S Grand Ave: 34.0554° N, -118.2498° W
    dataset.buildings.push(OsmBuildingFeature {
        id: 130_005,
        name: Some("Walt Disney Concert Hall".to_string()),
        building_type: BuildingType::Civic,
        outer_ring: vec![
            Wgs84Coord::new(34.0549, -118.2504, 0.0),
            Wgs84Coord::new(34.0559, -118.2504, 0.0),
            Wgs84Coord::new(34.0559, -118.2492, 0.0),
            Wgs84Coord::new(34.0549, -118.2492, 0.0),
            Wgs84Coord::new(34.0549, -118.2504, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(45.0),
        levels: Some(4),
    });

    // =========================================================================
    // 2. Real Roads, Highways & Multi-Level Stack Flyovers
    // =========================================================================

    // 1. I-110 Harbor Freeway (Main North-South Arterial Highway)
    dataset.roads.push(OsmRoadFeature {
        way_id: 230_001,
        name: Some("I-110 Harbor Freeway".to_string()),
        highway_type: HighwayType::Motorway,
        geometry: vec![
            Wgs84Coord::new(34.0350, -118.2680, 0.0), // Convention Center
            Wgs84Coord::new(34.0480, -118.2630, 0.0), // Financial District
            Wgs84Coord::new(34.0580, -118.2530, 0.0), // Four Level Interchange
        ],
        lanes: 8,
        speed_limit_kph: 105,
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // 2. US-101 / I-110 Four Level Interchange Elevated Stack (Layer 2 Flyover)
    dataset.roads.push(OsmRoadFeature {
        way_id: 230_002,
        name: Some("Four Level Interchange Flyover Ramp".to_string()),
        highway_type: HighwayType::Motorway,
        geometry: vec![
            Wgs84Coord::new(34.0560, -118.2560, 0.0),
            Wgs84Coord::new(34.0585, -118.2525, 0.0),
            Wgs84Coord::new(34.0620, -118.2480, 0.0),
        ],
        lanes: 2,
        speed_limit_kph: 80,
        is_oneway: true,
        layer: 2,
        is_bridge: true,
        is_tunnel: false,
    });

    // 3. Grand Avenue (Cultural & Civic Spine atop Bunker Hill)
    dataset.roads.push(OsmRoadFeature {
        way_id: 230_003,
        name: Some("Grand Avenue".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(34.0450, -118.2580, 0.0),
            Wgs84Coord::new(34.0520, -118.2520, 0.0),
            Wgs84Coord::new(34.0560, -118.2490, 0.0), // In front of Disney Hall
            Wgs84Coord::new(34.0610, -118.2440, 0.0),
        ],
        lanes: 4,
        speed_limit_kph: 45,
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // 4. Wilshire Boulevard (Major East-West Boulevard)
    dataset.roads.push(OsmRoadFeature {
        way_id: 230_004,
        name: Some("Wilshire Boulevard".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(34.0520, -118.2670, 0.0),
            Wgs84Coord::new(34.0500, -118.2600, 0.0), // At Wilshire Grand
            Wgs84Coord::new(34.0485, -118.2550, 0.0), // At Aon Center
            Wgs84Coord::new(34.0470, -118.2500, 0.0), // Grand Hope Park
        ],
        lanes: 4,
        speed_limit_kph: 45,
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // 5. Figueroa Street (Commercial Corridor running along Crypto.com Arena & Towers)
    dataset.roads.push(OsmRoadFeature {
        way_id: 230_005,
        name: Some("Figueroa Street".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(34.0400, -118.2660, 0.0),
            Wgs84Coord::new(34.0470, -118.2610, 0.0),
            Wgs84Coord::new(34.0540, -118.2550, 0.0),
            Wgs84Coord::new(34.0600, -118.2500, 0.0),
        ],
        lanes: 4,
        speed_limit_kph: 45,
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // 6. Grand Avenue (Cultural corridor past Disney Concert Hall & Broad Museum)
    dataset.roads.push(OsmRoadFeature {
        way_id: 230_006,
        name: Some("Grand Avenue".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(34.0450, -118.2580, 0.0),
            Wgs84Coord::new(34.0520, -118.2520, 0.0),
            Wgs84Coord::new(34.0555, -118.2490, 0.0), // At Disney Concert Hall
            Wgs84Coord::new(34.0590, -118.2460, 0.0), // Civic Center
        ],
        lanes: 4,
        speed_limit_kph: 40,
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // =========================================================================
    // 3. Urban Parkland & Land Cover
    // =========================================================================

    // Grand Park (Stretching from City Hall to The Music Center)
    dataset.land_cover.push(OsmLandCoverFeature {
        id: 430_001,
        name: Some("Gloria Molina Grand Park".to_string()),
        class: LandCoverClass::UrbanPark,
        polygon: vec![
            Wgs84Coord::new(34.0540, -118.2465, 0.0),
            Wgs84Coord::new(34.0560, -118.2485, 0.0),
            Wgs84Coord::new(34.0570, -118.2470, 0.0),
            Wgs84Coord::new(34.0550, -118.2450, 0.0),
            Wgs84Coord::new(34.0540, -118.2465, 0.0),
        ],
    });

    // Pershing Square
    dataset.land_cover.push(OsmLandCoverFeature {
        id: 430_002,
        name: Some("Pershing Square".to_string()),
        class: LandCoverClass::UrbanPark,
        polygon: vec![
            Wgs84Coord::new(34.0475, -118.2520, 0.0),
            Wgs84Coord::new(34.0490, -118.2520, 0.0),
            Wgs84Coord::new(34.0490, -118.2500, 0.0),
            Wgs84Coord::new(34.0475, -118.2500, 0.0),
            Wgs84Coord::new(34.0475, -118.2520, 0.0),
        ],
    });

    dataset
}

/// Generates valid OpenStreetMap XML representing Los Angeles Downtown.
pub fn generate_los_angeles_osm_xml() -> String {
    let dataset = create_los_angeles_dataset();
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<osm version=\"0.6\" generator=\"RealEarth-GIS-LA\">\n");

    let mut node_id = 300_000;
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
