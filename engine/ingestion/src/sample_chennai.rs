//! High-fidelity sample dataset for Chennai Central (Ripon Building, Central Station, Marina Beach, Napier Bridge).

use crate::osm_types::{
    BuildingType, HighwayType, IngestedDataset, OsmBuildingFeature, OsmRoadFeature,
    OsmWaterFeature, WaterType,
};
use realearth_shared::coordinates::Wgs84Coord;

/// Generates a realistic Chennai Central dataset with coastal waterfront, historic termini, and river crossings.
pub fn create_chennai_dataset() -> IngestedDataset {
    let mut dataset = IngestedDataset::default();

    // 1. Chennai Central Railway Station (Puratchi Thalaivar Dr. M.G. Ramachandran Central)
    dataset.buildings.push(OsmBuildingFeature {
        id: 500_001,
        name: Some("Chennai Central Railway Station".to_string()),
        building_type: BuildingType::Civic,
        outer_ring: vec![
            Wgs84Coord::new(13.0820, 80.2745, 0.0),
            Wgs84Coord::new(13.0835, 80.2745, 0.0),
            Wgs84Coord::new(13.0835, 80.2765, 0.0),
            Wgs84Coord::new(13.0820, 80.2765, 0.0),
            Wgs84Coord::new(13.0820, 80.2745, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(42.0), // Real height ~42 meters central clock tower
        levels: Some(4),
    });

    // 2. Ripon Building (Headquarters of Greater Chennai Corporation) - Neoclassical White Palace
    dataset.buildings.push(OsmBuildingFeature {
        id: 500_002,
        name: Some("Ripon Building".to_string()),
        building_type: BuildingType::Civic,
        outer_ring: vec![
            Wgs84Coord::new(13.0815, 80.2715, 0.0),
            Wgs84Coord::new(13.0828, 80.2715, 0.0),
            Wgs84Coord::new(13.0828, 80.2730, 0.0),
            Wgs84Coord::new(13.0815, 80.2730, 0.0),
            Wgs84Coord::new(13.0815, 80.2715, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(40.0),
        levels: Some(3),
    });

    // 3. Fort St. George (Tamil Nadu Legislative Assembly & Secretariat)
    dataset.buildings.push(OsmBuildingFeature {
        id: 500_003,
        name: Some("Fort St. George (Secretariat)".to_string()),
        building_type: BuildingType::Civic,
        outer_ring: vec![
            Wgs84Coord::new(13.0780, 80.2860, 0.0),
            Wgs84Coord::new(13.0800, 80.2860, 0.0),
            Wgs84Coord::new(13.0800, 80.2885, 0.0),
            Wgs84Coord::new(13.0780, 80.2885, 0.0),
            Wgs84Coord::new(13.0780, 80.2860, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(24.0),
        levels: Some(3),
    });

    // 4. LIC Building (Anna Salai 15-storey landmark modernist skyscraper)
    dataset.buildings.push(OsmBuildingFeature {
        id: 500_004,
        name: Some("LIC Building (Anna Salai)".to_string()),
        building_type: BuildingType::Commercial,
        outer_ring: vec![
            Wgs84Coord::new(13.0635, 80.2635, 0.0),
            Wgs84Coord::new(13.0645, 80.2635, 0.0),
            Wgs84Coord::new(13.0645, 80.2645, 0.0),
            Wgs84Coord::new(13.0635, 80.2645, 0.0),
            Wgs84Coord::new(13.0635, 80.2635, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(54.0),
        levels: Some(15),
    });

    // --- Road Networks ---
    // 5. Poonamallee High Road (EVR Periyar Salai / NH 48 approach) - Primary 6-lane artery
    dataset.roads.push(OsmRoadFeature {
        way_id: 510_001,
        name: Some("Poonamallee High Road (EVR Periyar Salai)".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(13.0805, 80.2600, 0.0),
            Wgs84Coord::new(13.0805, 80.2700, 0.0),
            Wgs84Coord::new(13.0805, 80.2740, 0.0), // In front of Central Station
            Wgs84Coord::new(13.0805, 80.2800, 0.0),
        ],
        lanes: 6,
        speed_limit_kph: 50,
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // 6. Anna Salai (Mount Road / NH 45) - 6-lane commercial spine
    dataset.roads.push(OsmRoadFeature {
        way_id: 510_002,
        name: Some("Anna Salai (Mount Road)".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(13.0805, 80.2740, 0.0), // Connects at Central junction
            Wgs84Coord::new(13.0750, 80.2710, 0.0),
            Wgs84Coord::new(13.0640, 80.2655, 0.0), // Past LIC Building corridor
            Wgs84Coord::new(13.0500, 80.2550, 0.0),
        ],
        lanes: 6,
        speed_limit_kph: 60,
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // 7. Kamarajar Salai (Marina Beach Road) - Wide coastal boulevard
    dataset.roads.push(OsmRoadFeature {
        way_id: 510_003,
        name: Some("Kamarajar Salai (Marina Beach Road)".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(13.0690, 80.2850, 0.0), // South end of Napier Bridge
            Wgs84Coord::new(13.0600, 80.2840, 0.0),
            Wgs84Coord::new(13.0450, 80.2820, 0.0),
        ],
        lanes: 6,
        speed_limit_kph: 60,
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // 8. Napier Bridge (Historic 6-arch elevated bridge crossing Cooum River mouth)
    dataset.roads.push(OsmRoadFeature {
        way_id: 510_004,
        name: Some("Napier Bridge (Cooum River Crossing)".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(13.0720, 80.2860, 0.0), // North abutment
            Wgs84Coord::new(13.0705, 80.2855, 0.0), // Mid-span over river
            Wgs84Coord::new(13.0690, 80.2850, 0.0), // South abutment (joins Kamarajar Salai)
        ],
        lanes: 4,
        speed_limit_kph: 45,
        is_oneway: false,
        layer: 1, // Elevated Grade Separation
        is_bridge: true,
        is_tunnel: false,
    });

    // 9. Rajaji Salai (Harbor approach along Fort St. George)
    dataset.roads.push(OsmRoadFeature {
        way_id: 510_005,
        name: Some("Rajaji Salai (Chennai Port Road)".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(13.0805, 80.2800, 0.0), // Connects at Poonamallee High Rd junction
            Wgs84Coord::new(13.0780, 80.2848, 0.0), // Corridor west of Fort St. George
            Wgs84Coord::new(13.0720, 80.2855, 0.0), // Joins Napier Bridge north abutment
        ],
        lanes: 4,
        speed_limit_kph: 50,
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // --- Water Bodies & Coastline ---
    // 10. Bay of Bengal Marina Coastline
    dataset.water_bodies.push(OsmWaterFeature {
        id: 520_001,
        name: Some("Bay of Bengal (Marina Coastline)".to_string()),
        water_type: WaterType::Ocean,
        outer_ring: vec![
            Wgs84Coord::new(13.0900, 80.2950, 0.0),
            Wgs84Coord::new(13.0900, 80.3200, 0.0),
            Wgs84Coord::new(13.0400, 80.3200, 0.0),
            Wgs84Coord::new(13.0400, 80.2850, 0.0),
            Wgs84Coord::new(13.0700, 80.2880, 0.0),
            Wgs84Coord::new(13.0900, 80.2950, 0.0),
        ],
        is_coastline: true,
        target_elevation_msl: 0.0,
    });

    // 11. Cooum River Estuary (Passed underneath Napier Bridge at Layer 0)
    dataset.water_bodies.push(OsmWaterFeature {
        id: 520_002,
        name: Some("Cooum River Estuary".to_string()),
        water_type: WaterType::River,
        outer_ring: vec![
            Wgs84Coord::new(13.0700, 80.2700, 0.0),
            Wgs84Coord::new(13.0715, 80.2700, 0.0),
            Wgs84Coord::new(13.0715, 80.2890, 0.0),
            Wgs84Coord::new(13.0695, 80.2890, 0.0),
            Wgs84Coord::new(13.0700, 80.2700, 0.0),
        ],
        is_coastline: false,
        target_elevation_msl: 0.5,
    });

    dataset
}

/// Generates valid OSM XML representation of the Chennai Central dataset.
pub fn generate_chennai_osm_xml() -> String {
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<osm version=\"0.6\" generator=\"RealEarth\">\n");

    let mut node_id: u64 = 50_000;
    let mut coord_to_node: std::collections::HashMap<(u64, u64), u64> =
        std::collections::HashMap::new();

    let mut add_node = |coord: &Wgs84Coord, xml: &mut String| -> u64 {
        let key = ((coord.lat * 1e7) as u64, (coord.lon * 1e7) as u64);
        if let Some(&existing) = coord_to_node.get(&key) {
            existing
        } else {
            node_id += 1;
            xml.push_str(&format!(
                "  <node id=\"{}\" lat=\"{:.7}\" lon=\"{:.7}\"/>\n",
                node_id, coord.lat, coord.lon
            ));
            coord_to_node.insert(key, node_id);
            node_id
        }
    };

    let dataset = create_chennai_dataset();

    for b in &dataset.buildings {
        let mut refs = Vec::new();
        for pt in &b.outer_ring {
            refs.push(add_node(pt, &mut xml));
        }

        xml.push_str(&format!("  <way id=\"{}\">\n", b.id));
        for r in refs {
            xml.push_str(&format!("    <nd ref=\"{}\"/>\n", r));
        }
        xml.push_str("    <tag k=\"building\" v=\"civic\"/>\n");
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
        for rf in refs {
            xml.push_str(&format!("    <nd ref=\"{}\"/>\n", rf));
        }
        xml.push_str("    <tag k=\"highway\" v=\"primary\"/>\n");
        if let Some(ref name) = r.name {
            xml.push_str(&format!("    <tag k=\"name\" v=\"{}\"/>\n", name));
        }
        xml.push_str(&format!("    <tag k=\"lanes\" v=\"{}\"/>\n", r.lanes));
        xml.push_str(&format!("    <tag k=\"layer\" v=\"{}\"/>\n", r.layer));
        if r.is_bridge {
            xml.push_str("    <tag k=\"bridge\" v=\"yes\"/>\n");
        }
        xml.push_str("  </way>\n");
    }

    xml.push_str("</osm>\n");
    xml
}
