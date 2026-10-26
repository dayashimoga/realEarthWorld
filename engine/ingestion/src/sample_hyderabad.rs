//! High-fidelity sample dataset for Hyderabad Central (Charminar, Hussain Sagar, HITEC City, PVNR Expressway).

use crate::osm_types::{
    BuildingType, HighwayType, IngestedDataset, OsmBuildingFeature, OsmRoadFeature,
    OsmWaterFeature, WaterType,
};
use realearth_shared::coordinates::Wgs84Coord;

/// Generates a realistic Hyderabad Central dataset with historic monuments, modern IT towers, and elevated highways.
pub fn create_hyderabad_dataset() -> IngestedDataset {
    let mut dataset = IngestedDataset::default();

    // 1. Charminar (Iconic 1591 CE Indo-Islamic monument with four 56m high minarets)
    dataset.buildings.push(OsmBuildingFeature {
        id: 600_001,
        name: Some("Charminar".to_string()),
        building_type: BuildingType::Civic,
        outer_ring: vec![
            Wgs84Coord::new(17.3611, 78.4742, 0.0),
            Wgs84Coord::new(17.3621, 78.4742, 0.0),
            Wgs84Coord::new(17.3621, 78.4752, 0.0),
            Wgs84Coord::new(17.3611, 78.4752, 0.0),
            Wgs84Coord::new(17.3611, 78.4742, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(56.0), // 56 meters minarets
        levels: Some(4),
    });

    // 2. Mecca Masjid (One of the largest mosques in India, historic granite architecture)
    dataset.buildings.push(OsmBuildingFeature {
        id: 600_002,
        name: Some("Mecca Masjid".to_string()),
        building_type: BuildingType::Civic,
        outer_ring: vec![
            Wgs84Coord::new(17.3598, 78.4725, 0.0),
            Wgs84Coord::new(17.3610, 78.4725, 0.0),
            Wgs84Coord::new(17.3610, 78.4740, 0.0),
            Wgs84Coord::new(17.3598, 78.4740, 0.0),
            Wgs84Coord::new(17.3598, 78.4725, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(25.0),
        levels: Some(2),
    });

    // 3. HITEC City Cyber Towers (Symbol of Hyderabad tech boom, 10-storey landmark drum tower)
    dataset.buildings.push(OsmBuildingFeature {
        id: 600_003,
        name: Some("HITEC City Cyber Towers".to_string()),
        building_type: BuildingType::Commercial,
        outer_ring: vec![
            Wgs84Coord::new(17.4495, 78.3800, 0.0),
            Wgs84Coord::new(17.4515, 78.3800, 0.0),
            Wgs84Coord::new(17.4515, 78.3820, 0.0),
            Wgs84Coord::new(17.4495, 78.3820, 0.0),
            Wgs84Coord::new(17.4495, 78.3800, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(95.0),
        levels: Some(10),
    });

    // 4. Golconda Fort (Massive granite citadel fortress on western rocky hill)
    dataset.buildings.push(OsmBuildingFeature {
        id: 600_004,
        name: Some("Golconda Fort (Bala Hissar)".to_string()),
        building_type: BuildingType::Civic,
        outer_ring: vec![
            Wgs84Coord::new(17.3820, 78.4000, 0.0),
            Wgs84Coord::new(17.3845, 78.4000, 0.0),
            Wgs84Coord::new(17.3845, 78.4025, 0.0),
            Wgs84Coord::new(17.3820, 78.4025, 0.0),
            Wgs84Coord::new(17.3820, 78.4000, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(45.0),
        levels: Some(3),
    });

    // --- Road Networks ---
    // 5. PV Narasimha Rao (PVNR) Elevated Expressway (11.6km long 4-lane viaduct)
    dataset.roads.push(OsmRoadFeature {
        way_id: 610_001,
        name: Some("PVNR Elevated Expressway".to_string()),
        highway_type: HighwayType::Motorway,
        geometry: vec![
            Wgs84Coord::new(17.3400, 78.4400, 0.0), // South approach from NH 44 (Aramghar)
            Wgs84Coord::new(17.3600, 78.4450, 0.0), // Elevated span over Mehdipatnam
            Wgs84Coord::new(17.3800, 78.4480, 0.0), // North terminus at Sarojini Devi Road
        ],
        lanes: 4,
        speed_limit_kph: 80,
        is_oneway: false,
        layer: 1, // Elevated Grade Separation
        is_bridge: true,
        is_tunnel: false,
    });

    // 6. Inner Ring Road (Hyderabad 6-lane urban orbital connecting Charminar corridor)
    dataset.roads.push(OsmRoadFeature {
        way_id: 610_002,
        name: Some("Inner Ring Road (Charminar Approach)".to_string()),
        highway_type: HighwayType::Trunk,
        geometry: vec![
            Wgs84Coord::new(17.3500, 78.4600, 0.0),
            Wgs84Coord::new(17.3600, 78.4700, 0.0),
            Wgs84Coord::new(17.3635, 78.4735, 0.0), // Charminar corridor
            Wgs84Coord::new(17.3700, 78.4800, 0.0),
        ],
        lanes: 6,
        speed_limit_kph: 60,
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // 7. Tank Bund Road (Scenic causeway spanning the eastern shore of Hussain Sagar lake)
    dataset.roads.push(OsmRoadFeature {
        way_id: 610_003,
        name: Some("Tank Bund Road (Hussain Sagar Promenade)".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(17.4150, 78.4800, 0.0),
            Wgs84Coord::new(17.4250, 78.4820, 0.0),
            Wgs84Coord::new(17.4350, 78.4830, 0.0),
        ],
        lanes: 6,
        speed_limit_kph: 50,
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // 8. Nayapul / Musi River Bridge Road (Elevated bridge crossing Musi River at Layer 1)
    dataset.roads.push(OsmRoadFeature {
        way_id: 610_004,
        name: Some("Nayapul (Musi River Crossing)".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(17.3670, 78.4740, 0.0), // South bank (from Charminar)
            Wgs84Coord::new(17.3685, 78.4740, 0.0), // Over Musi River
            Wgs84Coord::new(17.3700, 78.4740, 0.0), // North bank (towards Afzal Gunj)
        ],
        lanes: 4,
        speed_limit_kph: 45,
        is_oneway: false,
        layer: 1, // Elevated Grade Separation
        is_bridge: true,
        is_tunnel: false,
    });

    // 9. HITEC City Main Road (Primary corridor connecting IT Hub)
    dataset.roads.push(OsmRoadFeature {
        way_id: 610_005,
        name: Some("HITEC City Main Road".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(17.4450, 78.3750, 0.0),
            Wgs84Coord::new(17.4480, 78.3810, 0.0), // Cyber Towers south concourse
            Wgs84Coord::new(17.4550, 78.3880, 0.0),
        ],
        lanes: 6,
        speed_limit_kph: 60,
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // --- Water Bodies ---
    // 10. Hussain Sagar Lake (Historic heart-shaped lake)
    dataset.water_bodies.push(OsmWaterFeature {
        id: 620_001,
        name: Some("Hussain Sagar Lake".to_string()),
        water_type: WaterType::Lake,
        outer_ring: vec![
            Wgs84Coord::new(17.4180, 78.4650, 0.0),
            Wgs84Coord::new(17.4320, 78.4650, 0.0),
            Wgs84Coord::new(17.4320, 78.4810, 0.0),
            Wgs84Coord::new(17.4180, 78.4810, 0.0),
            Wgs84Coord::new(17.4180, 78.4650, 0.0),
        ],
        is_coastline: false,
        target_elevation_msl: 512.0,
    });

    // 11. Musi River (Passed underneath Nayapul at Layer 0)
    dataset.water_bodies.push(OsmWaterFeature {
        id: 620_002,
        name: Some("Musi River".to_string()),
        water_type: WaterType::River,
        outer_ring: vec![
            Wgs84Coord::new(17.3680, 78.4600, 0.0),
            Wgs84Coord::new(17.3690, 78.4600, 0.0),
            Wgs84Coord::new(17.3690, 78.4900, 0.0),
            Wgs84Coord::new(17.3680, 78.4900, 0.0),
            Wgs84Coord::new(17.3680, 78.4600, 0.0),
        ],
        is_coastline: false,
        target_elevation_msl: 504.0,
    });

    dataset
}

/// Generates valid OSM XML representation of the Hyderabad Central dataset.
pub fn generate_hyderabad_osm_xml() -> String {
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<osm version=\"0.6\" generator=\"RealEarth\">\n");

    let mut node_id: u64 = 60_000;
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

    let dataset = create_hyderabad_dataset();

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
