//! High-fidelity sample dataset for Mysuru Urban (Mysore Palace, Chamundi Hill, Devaraja Market).

use crate::osm_types::{
    BuildingType, HighwayType, IngestedDataset, OsmBuildingFeature, OsmRoadFeature,
    OsmWaterFeature, WaterType,
};
use realearth_shared::coordinates::Wgs84Coord;

/// Generates a realistic Mysuru Urban dataset with heritage landmarks, hill temples, and arterial roads.
pub fn create_mysuru_dataset() -> IngestedDataset {
    let mut dataset = IngestedDataset::default();

    // 1. Mysore Palace (Amba Vilas) - World-renowned Indo-Saracenic palace with gold-plated domes
    dataset.buildings.push(OsmBuildingFeature {
        id: 400_001,
        name: Some("Mysore Palace (Amba Vilas)".to_string()),
        building_type: BuildingType::Civic,
        outer_ring: vec![
            Wgs84Coord::new(12.3045, 76.6545, 0.0),
            Wgs84Coord::new(12.3060, 76.6545, 0.0),
            Wgs84Coord::new(12.3060, 76.6560, 0.0),
            Wgs84Coord::new(12.3045, 76.6560, 0.0),
            Wgs84Coord::new(12.3045, 76.6545, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(45.0), // Real height ~45 meters central gilded dome
        levels: Some(3),
    });

    // 2. Chamundeshwari Temple atop Chamundi Hill (Elevation ~1060m MSL)
    dataset.buildings.push(OsmBuildingFeature {
        id: 400_002,
        name: Some("Chamundeshwari Temple (Chamundi Hill)".to_string()),
        building_type: BuildingType::Civic,
        outer_ring: vec![
            Wgs84Coord::new(12.2740, 76.6705, 0.0),
            Wgs84Coord::new(12.2750, 76.6705, 0.0),
            Wgs84Coord::new(12.2750, 76.6715, 0.0),
            Wgs84Coord::new(12.2740, 76.6715, 0.0),
            Wgs84Coord::new(12.2740, 76.6705, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(28.0), // 7-tier sculpted gopuram
        levels: Some(7),
    });

    // 3. Mysuru Junction Railway Station (Heritage railway terminus)
    dataset.buildings.push(OsmBuildingFeature {
        id: 400_003,
        name: Some("Mysuru Junction Railway Station".to_string()),
        building_type: BuildingType::Civic,
        outer_ring: vec![
            Wgs84Coord::new(12.3155, 76.6445, 0.0),
            Wgs84Coord::new(12.3168, 76.6445, 0.0),
            Wgs84Coord::new(12.3168, 76.6460, 0.0),
            Wgs84Coord::new(12.3155, 76.6460, 0.0),
            Wgs84Coord::new(12.3155, 76.6445, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(18.0),
        levels: Some(2),
    });

    // 4. Devaraja Market - Historic bustling market quadrangle
    dataset.buildings.push(OsmBuildingFeature {
        id: 400_004,
        name: Some("Devaraja Market".to_string()),
        building_type: BuildingType::Commercial,
        outer_ring: vec![
            Wgs84Coord::new(12.3095, 76.6505, 0.0),
            Wgs84Coord::new(12.3110, 76.6505, 0.0),
            Wgs84Coord::new(12.3110, 76.6520, 0.0),
            Wgs84Coord::new(12.3095, 76.6520, 0.0),
            Wgs84Coord::new(12.3095, 76.6505, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(12.0),
        levels: Some(2),
    });

    // --- Road Networks ---
    // 5. Sayyaji Rao Road (Primary royal boulevard linking Palace to Railway Station)
    dataset.roads.push(OsmRoadFeature {
        way_id: 410_001,
        name: Some("Sayyaji Rao Road".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(12.3060, 76.6570, 0.0), // K.R. Circle / Palace North Gate (Junction with NH 766 & Albert Victor Rd)
            Wgs84Coord::new(12.3100, 76.6510, 0.0), // Near Devaraja Market
            Wgs84Coord::new(12.3160, 76.6460, 0.0), // Near Station
        ],
        lanes: 4,
        speed_limit_kph: 50,
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // 6. Bengaluru-Nilgiri Road (NH 766 / Expressway Entry Corridor)
    dataset.roads.push(OsmRoadFeature {
        way_id: 410_002,
        name: Some("Bengaluru-Nilgiri Road (NH 766)".to_string()),
        highway_type: HighwayType::Trunk,
        geometry: vec![
            Wgs84Coord::new(12.3060, 76.6570, 0.0),
            Wgs84Coord::new(12.3150, 76.6620, 0.0),
            Wgs84Coord::new(12.3300, 76.6650, 0.0),
            Wgs84Coord::new(12.3500, 76.6680, 0.0), // Links to Bengaluru Expressway
        ],
        lanes: 6,
        speed_limit_kph: 80,
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // 7. Mysuru Outer Ring Road (ORR) - Elevated expressway bypass interchange
    dataset.roads.push(OsmRoadFeature {
        way_id: 410_003,
        name: Some("Mysuru Outer Ring Road (ORR)".to_string()),
        highway_type: HighwayType::Motorway,
        geometry: vec![
            Wgs84Coord::new(12.3480, 76.6400, 0.0),
            Wgs84Coord::new(12.3500, 76.6680, 0.0), // Interchange with NH 766
            Wgs84Coord::new(12.3450, 76.6900, 0.0),
        ],
        lanes: 6,
        speed_limit_kph: 90,
        is_oneway: false,
        layer: 1, // Grade-separated elevated bypass
        is_bridge: true,
        is_tunnel: false,
    });

    // 8. Albert Victor Road (Palace Perimeter)
    dataset.roads.push(OsmRoadFeature {
        way_id: 410_004,
        name: Some("Albert Victor Road".to_string()),
        highway_type: HighwayType::Secondary,
        geometry: vec![
            Wgs84Coord::new(12.3040, 76.6530, 0.0),
            Wgs84Coord::new(12.3045, 76.6570, 0.0),
            Wgs84Coord::new(12.3060, 76.6570, 0.0),
        ],
        lanes: 2,
        speed_limit_kph: 40,
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // 9. Chamundi Hill Ascent Road (Winding slope road ascending to 1060m)
    dataset.roads.push(OsmRoadFeature {
        way_id: 410_005,
        name: Some("Chamundi Hill Road".to_string()),
        highway_type: HighwayType::Secondary,
        geometry: vec![
            Wgs84Coord::new(12.3000, 76.6700, 0.0), // Base
            Wgs84Coord::new(12.2880, 76.6710, 0.0), // Mid-ascent
            Wgs84Coord::new(12.2745, 76.6710, 0.0), // Summit temple
        ],
        lanes: 2,
        speed_limit_kph: 40,
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // 10. Water Body: Karanji Lake
    dataset.water_bodies.push(OsmWaterFeature {
        id: 420_001,
        name: Some("Karanji Lake".to_string()),
        water_type: WaterType::Lake,
        outer_ring: vec![
            Wgs84Coord::new(12.3000, 76.6700, 0.0),
            Wgs84Coord::new(12.3050, 76.6700, 0.0),
            Wgs84Coord::new(12.3050, 76.6760, 0.0),
            Wgs84Coord::new(12.3000, 76.6760, 0.0),
            Wgs84Coord::new(12.3000, 76.6700, 0.0),
        ],
        is_coastline: false,
        target_elevation_msl: 750.0,
    });

    dataset
}

/// Generates valid OSM XML representation of the Mysuru Urban dataset.
pub fn generate_mysuru_osm_xml() -> String {
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<osm version=\"0.6\" generator=\"RealEarth\">\n");

    let mut node_id: u64 = 40_000;
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

    let dataset = create_mysuru_dataset();

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
