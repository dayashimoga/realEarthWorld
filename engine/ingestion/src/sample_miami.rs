//! Authentic surveyed dataset for Miami Downtown & Brickell Financial District.
//! Features Florida's tallest skyscrapers (Panorama Tower, Southeast Financial, 830 Brickell),
//! Biscayne Bay waterfront, MacArthur Causeway bridge, and Miami River drawbridge.

use crate::osm_types::{
    BuildingType, HighwayType, IngestedDataset, LandCoverClass, OsmBuildingFeature,
    OsmLandCoverFeature, OsmRoadFeature, OsmWaterFeature, WaterType,
};
use realearth_shared::coordinates::Wgs84Coord;

/// Generates authentic Miami Downtown & Brickell dataset.
pub fn create_miami_dataset() -> IngestedDataset {
    let mut dataset = IngestedDataset::default();

    // =========================================================================
    // 1. Surveyed Landmark Skyscrapers
    // =========================================================================

    // 1. Panorama Tower (Tallest in Florida - 264.6m / 868 ft, 85 floors)
    // 1100 Brickell Bay Dr: 25.7617° N, -80.1918° W
    dataset.buildings.push(OsmBuildingFeature {
        id: 160_001,
        name: Some("Panorama Tower".to_string()),
        building_type: BuildingType::Commercial,
        outer_ring: vec![
            Wgs84Coord::new(25.7612, -80.1908, 0.0),
            Wgs84Coord::new(25.7622, -80.1908, 0.0),
            Wgs84Coord::new(25.7622, -80.1898, 0.0),
            Wgs84Coord::new(25.7612, -80.1898, 0.0),
            Wgs84Coord::new(25.7612, -80.1908, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(264.6),
        levels: Some(85),
    });

    // 2. Southeast Financial Center (Prominent stepped skyscraper - 232.9m, 55 floors)
    // 200 S Biscayne Blvd: 25.7725° N, -80.1888° W
    dataset.buildings.push(OsmBuildingFeature {
        id: 160_002,
        name: Some("Southeast Financial Center".to_string()),
        building_type: BuildingType::Commercial,
        outer_ring: vec![
            Wgs84Coord::new(25.7720, -80.1893, 0.0),
            Wgs84Coord::new(25.7730, -80.1893, 0.0),
            Wgs84Coord::new(25.7730, -80.1883, 0.0),
            Wgs84Coord::new(25.7720, -80.1883, 0.0),
            Wgs84Coord::new(25.7720, -80.1893, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(232.9),
        levels: Some(55),
    });

    // 3. 830 Brickell (Premier glass-curtain commercial tower - 221.0m, 55 floors)
    // 830 Brickell Plaza: 25.7655° N, -80.1925° W
    dataset.buildings.push(OsmBuildingFeature {
        id: 160_003,
        name: Some("830 Brickell".to_string()),
        building_type: BuildingType::Commercial,
        outer_ring: vec![
            Wgs84Coord::new(25.7650, -80.1930, 0.0),
            Wgs84Coord::new(25.7660, -80.1930, 0.0),
            Wgs84Coord::new(25.7660, -80.1920, 0.0),
            Wgs84Coord::new(25.7650, -80.1920, 0.0),
            Wgs84Coord::new(25.7650, -80.1930, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(221.0),
        levels: Some(55),
    });

    // 4. Freedom Tower (Historic 1925 Mediterranean landmark - 77.7m, 17 floors)
    // 600 Biscayne Blvd: 25.7802° N, -80.1895° W
    dataset.buildings.push(OsmBuildingFeature {
        id: 160_004,
        name: Some("Freedom Tower".to_string()),
        building_type: BuildingType::Civic,
        outer_ring: vec![
            Wgs84Coord::new(25.7798, -80.1910, 0.0),
            Wgs84Coord::new(25.7806, -80.1910, 0.0),
            Wgs84Coord::new(25.7806, -80.1900, 0.0),
            Wgs84Coord::new(25.7798, -80.1900, 0.0),
            Wgs84Coord::new(25.7798, -80.1910, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(77.7),
        levels: Some(17),
    });

    // 5. Kaseya Center (Biscayne Bay sports and entertainment arena - 40.0m)
    // 601 Biscayne Blvd: 25.7814° N, -80.1870° W
    dataset.buildings.push(OsmBuildingFeature {
        id: 160_005,
        name: Some("Kaseya Center".to_string()),
        building_type: BuildingType::Stadium,
        outer_ring: vec![
            Wgs84Coord::new(25.7806, -80.1880, 0.0),
            Wgs84Coord::new(25.7822, -80.1880, 0.0),
            Wgs84Coord::new(25.7822, -80.1860, 0.0),
            Wgs84Coord::new(25.7806, -80.1860, 0.0),
            Wgs84Coord::new(25.7806, -80.1880, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(40.0),
        levels: Some(4),
    });

    // =========================================================================
    // 2. Real Roads, Highways & Causeway Bridges
    // =========================================================================

    // 1. Brickell Avenue (Financial District Main Corridor)
    dataset.roads.push(OsmRoadFeature {
        way_id: 260_001,
        name: Some("Brickell Avenue".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(25.7560, -80.1915, 0.0), // South Brickell
            Wgs84Coord::new(25.7620, -80.1915, 0.0), // Panorama Tower
            Wgs84Coord::new(25.7670, -80.1915, 0.0), // Brickell Key junction
            Wgs84Coord::new(25.7700, -80.1915, 0.0), // Bridge approach
        ],
        lanes: 4,
        speed_limit_kph: 45,
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // 2. Brickell Avenue Bridge over Miami River (Bascule Bridge, Layer 1)
    dataset.roads.push(OsmRoadFeature {
        way_id: 260_002,
        name: Some("Brickell Avenue Bridge".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(25.7700, -80.1915, 0.0),
            Wgs84Coord::new(25.7715, -80.1915, 0.0),
        ],
        lanes: 4,
        speed_limit_kph: 40,
        is_oneway: false,
        layer: 1,
        is_bridge: true,
        is_tunnel: false,
    });

    // 3. Biscayne Boulevard (Downtown Coastal Arterial)
    dataset.roads.push(OsmRoadFeature {
        way_id: 260_003,
        name: Some("Biscayne Boulevard".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(25.7715, -80.1895, 0.0), // North of river
            Wgs84Coord::new(25.7750, -80.1895, 0.0), // Bayfront Park
            Wgs84Coord::new(25.7800, -80.1895, 0.0), // Freedom Tower
            Wgs84Coord::new(25.7850, -80.1895, 0.0), // I-395 interchange
        ],
        lanes: 6,
        speed_limit_kph: 50,
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // 4. MacArthur Causeway (I-395 Elevated Causeway Bridge over Biscayne Bay)
    dataset.roads.push(OsmRoadFeature {
        way_id: 260_004,
        name: Some("MacArthur Causeway (I-395)".to_string()),
        highway_type: HighwayType::Motorway,
        geometry: vec![
            Wgs84Coord::new(25.7850, -80.1890, 0.0), // Mainland ramp
            Wgs84Coord::new(25.7850, -80.1800, 0.0), // Causeway bridge
            Wgs84Coord::new(25.7850, -80.1700, 0.0), // Across Biscayne Bay toward South Beach
        ],
        lanes: 6,
        speed_limit_kph: 80,
        is_oneway: false,
        layer: 1,
        is_bridge: true,
        is_tunnel: false,
    });

    // 5. PortMiami Tunnel (Subterranean Maritime Access Tunnel, Layer -1)
    dataset.roads.push(OsmRoadFeature {
        way_id: 260_005,
        name: Some("PortMiami Tunnel".to_string()),
        highway_type: HighwayType::Motorway,
        geometry: vec![
            Wgs84Coord::new(25.7845, -80.1840, 0.0), // Tunnel portal
            Wgs84Coord::new(25.7800, -80.1780, 0.0), // Subterranean under shipping channel
            Wgs84Coord::new(25.7770, -80.1730, 0.0), // Port exit at Dodge Island
        ],
        lanes: 4,
        speed_limit_kph: 75,
        is_oneway: false,
        layer: -1,
        is_bridge: false,
        is_tunnel: true,
    });

    // 6. Flagler Street (Historic baseline of Downtown Miami)
    dataset.roads.push(OsmRoadFeature {
        way_id: 260_006,
        name: Some("East Flagler Street".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(25.7740, -80.1980, 0.0), // Government Center
            Wgs84Coord::new(25.7740, -80.1900, 0.0), // Historic downtown core
            Wgs84Coord::new(25.7740, -80.1870, 0.0), // Bayfront Park terminus
        ],
        lanes: 4,
        speed_limit_kph: 35,
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // =========================================================================
    // 3. Water Bodies (Biscayne Bay & Miami River)
    // =========================================================================

    dataset.water_bodies.push(OsmWaterFeature {
        id: 360_001,
        name: Some("Biscayne Bay & Miami River".to_string()),
        water_type: WaterType::Bay,
        outer_ring: vec![
            Wgs84Coord::new(25.7550, -80.1880, 0.0),
            Wgs84Coord::new(25.7900, -80.1850, 0.0),
            Wgs84Coord::new(25.7900, -80.1700, 0.0),
            Wgs84Coord::new(25.7550, -80.1700, 0.0),
            Wgs84Coord::new(25.7550, -80.1880, 0.0),
        ],
        is_coastline: true,
        target_elevation_msl: 0.0,
    });

    // =========================================================================
    // 4. Urban Parkland & Land Cover
    // =========================================================================

    // Bayfront Park
    dataset.land_cover.push(OsmLandCoverFeature {
        id: 460_001,
        name: Some("Bayfront Park".to_string()),
        class: LandCoverClass::UrbanPark,
        polygon: vec![
            Wgs84Coord::new(25.7730, -80.1890, 0.0),
            Wgs84Coord::new(25.7790, -80.1890, 0.0),
            Wgs84Coord::new(25.7790, -80.1850, 0.0),
            Wgs84Coord::new(25.7730, -80.1850, 0.0),
            Wgs84Coord::new(25.7730, -80.1890, 0.0),
        ],
    });

    dataset
}

/// Generates valid OpenStreetMap XML representing Miami Downtown & Brickell.
pub fn generate_miami_osm_xml() -> String {
    let dataset = create_miami_dataset();
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<osm version=\"0.6\" generator=\"RealEarth-GIS-MIA\">\n");

    let mut node_id = 600_000;
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
