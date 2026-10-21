//! High-fidelity sample dataset for Bengaluru Central (Vidhana Soudha, Cubbon Park, MG Road).

use crate::osm_types::{
    BuildingType, HighwayType, IngestedDataset, OsmBuildingFeature, OsmRoadFeature,
    OsmWaterFeature, WaterType,
};
use realearth_shared::coordinates::Wgs84Coord;

/// Generates a realistic Bengaluru Central dataset with real landmarks and multi-level grade separations.
pub fn create_bengaluru_central_dataset() -> IngestedDataset {
    let mut dataset = IngestedDataset::default();

    // 1. Vidhana Soudha (Karnataka State Legislature) - Massive Neo-Dravidian building
    dataset.buildings.push(OsmBuildingFeature {
        id: 100_001,
        name: Some("Vidhana Soudha".to_string()),
        building_type: BuildingType::Civic,
        outer_ring: vec![
            Wgs84Coord::new(12.9791, 77.5901, 0.0),
            Wgs84Coord::new(12.9803, 77.5901, 0.0),
            Wgs84Coord::new(12.9803, 77.5913, 0.0),
            Wgs84Coord::new(12.9791, 77.5913, 0.0),
            Wgs84Coord::new(12.9791, 77.5901, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(46.0), // Real height ~46 meters with central dome
        levels: Some(4),
    });

    // 2. High Court of Karnataka (Attara Kacheri) - Red brick Greco-Roman structure
    dataset.buildings.push(OsmBuildingFeature {
        id: 100_002,
        name: Some("High Court of Karnataka".to_string()),
        building_type: BuildingType::Civic,
        outer_ring: vec![
            Wgs84Coord::new(12.9780, 77.5915, 0.0),
            Wgs84Coord::new(12.9788, 77.5915, 0.0),
            Wgs84Coord::new(12.9788, 77.5925, 0.0),
            Wgs84Coord::new(12.9780, 77.5925, 0.0),
            Wgs84Coord::new(12.9780, 77.5915, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(22.0),
        levels: Some(2),
    });

    // 3. M. Chinnaswamy Stadium
    dataset.buildings.push(OsmBuildingFeature {
        id: 100_003,
        name: Some("M. Chinnaswamy Stadium".to_string()),
        building_type: BuildingType::Stadium,
        outer_ring: vec![
            Wgs84Coord::new(12.9782, 77.5985, 0.0),
            Wgs84Coord::new(12.9802, 77.5985, 0.0),
            Wgs84Coord::new(12.9802, 77.6005, 0.0),
            Wgs84Coord::new(12.9782, 77.6005, 0.0),
            Wgs84Coord::new(12.9782, 77.5985, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(35.0),
        levels: Some(5),
    });

    // 4. Commercial Towers along MG Road
    dataset.buildings.push(OsmBuildingFeature {
        id: 100_004,
        name: Some("Utility Building (MG Road)".to_string()),
        building_type: BuildingType::Commercial,
        outer_ring: vec![
            Wgs84Coord::new(12.9745, 77.6070, 0.0),
            Wgs84Coord::new(12.9752, 77.6070, 0.0),
            Wgs84Coord::new(12.9752, 77.6078, 0.0),
            Wgs84Coord::new(12.9745, 77.6078, 0.0),
            Wgs84Coord::new(12.9745, 77.6070, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(85.0), // 24-storey high-rise
        levels: Some(24),
    });

    // --- Road Networks ---
    // 5. Mahatma Gandhi Road (MG Road) - Primary 4-lane arterial road
    dataset.roads.push(OsmRoadFeature {
        way_id: 200_001,
        name: Some("Mahatma Gandhi Road".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(12.9740, 77.6000, 0.0),
            Wgs84Coord::new(12.9748, 77.6060, 0.0),
            Wgs84Coord::new(12.9755, 77.6120, 0.0),
            Wgs84Coord::new(12.9760, 77.6150, 0.0),
        ],
        lanes: 4,
        speed_limit_kph: 50,
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // 6. Brigade Road - Busy commercial street
    dataset.roads.push(OsmRoadFeature {
        way_id: 200_002,
        name: Some("Brigade Road".to_string()),
        highway_type: HighwayType::Secondary,
        geometry: vec![
            Wgs84Coord::new(12.9748, 77.6060, 0.0),
            Wgs84Coord::new(12.9700, 77.6062, 0.0),
            Wgs84Coord::new(12.9650, 77.6065, 0.0),
        ],
        lanes: 2,
        speed_limit_kph: 40,
        is_oneway: true,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // 7. Kasturba Road Ground Highway (Layer 0)
    dataset.roads.push(OsmRoadFeature {
        way_id: 200_003,
        name: Some("Kasturba Road".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(12.9700, 77.5950, 0.0),
            Wgs84Coord::new(12.9750, 77.5950, 0.0),
            Wgs84Coord::new(12.9800, 77.5950, 0.0),
        ],
        lanes: 4,
        speed_limit_kph: 50,
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // 8. Grade-Separated Flyover Crossing Kasturba Road (Layer 1, bridge=yes)
    // Crosses at (12.9750, 77.5950) overhead WITHOUT intersecting at-grade
    dataset.roads.push(OsmRoadFeature {
        way_id: 200_004,
        name: Some("Cubbon Park Elevated Flyover".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(12.9750, 77.5920, 0.0),
            Wgs84Coord::new(12.9750, 77.5950, 0.0), // Overhead crossing point
            Wgs84Coord::new(12.9750, 77.5980, 0.0),
        ],
        lanes: 2,
        speed_limit_kph: 60,
        is_oneway: false,
        layer: 1,
        is_bridge: true,
        is_tunnel: false,
    });

    // 9. Ulsoor Lake / Cubbon Lotus Pond water body
    dataset.water_bodies.push(OsmWaterFeature {
        id: 300_001,
        name: Some("Cubbon Park Lotus Pond".to_string()),
        water_type: WaterType::Lake,
        outer_ring: vec![
            Wgs84Coord::new(12.9730, 77.5910, 0.0),
            Wgs84Coord::new(12.9740, 77.5910, 0.0),
            Wgs84Coord::new(12.9740, 77.5925, 0.0),
            Wgs84Coord::new(12.9730, 77.5925, 0.0),
            Wgs84Coord::new(12.9730, 77.5910, 0.0),
        ],
        is_coastline: false,
        target_elevation_msl: 915.0,
    });

    dataset
}

/// Generates valid OSM XML representation of the Bengaluru Central sample dataset.
pub fn generate_bengaluru_osm_xml() -> String {
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<osm version=\"0.6\" generator=\"RealEarth\">\n");

    let mut node_id: u64 = 10_000;
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

    let dataset = create_bengaluru_central_dataset();

    // Nodes for buildings
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

    // Nodes for roads
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
