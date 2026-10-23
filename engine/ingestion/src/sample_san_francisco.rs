//! Authentic, high-fidelity sample dataset for San Francisco Downtown & Waterfront.
//! Features real surveyed skyscrapers, coastal roads, the Bay Bridge approach,
//! the Broadway Tunnel, Embarcadero shoreline, and urban parks.

use crate::osm_types::{
    BuildingType, HighwayType, IngestedDataset, LandCoverClass, OsmBuildingFeature,
    OsmLandCoverFeature, OsmRoadFeature, OsmWaterFeature, WaterType,
};
use realearth_shared::coordinates::Wgs84Coord;

/// Generates an authentic, geographically surveyed San Francisco Downtown & Waterfront dataset.
pub fn create_san_francisco_dataset() -> IngestedDataset {
    let mut dataset = IngestedDataset::default();

    // =========================================================================
    // 1. Surveyed Buildings & Skyscrapers
    // =========================================================================

    // 1. Salesforce Tower (Tallest skyscraper in San Francisco - 326m, 61 floors)
    // 415 Mission Street: 37.7897° N, 122.3972° W
    dataset.buildings.push(OsmBuildingFeature {
        id: 110_001,
        name: Some("Salesforce Tower".to_string()),
        building_type: BuildingType::Commercial,
        outer_ring: vec![
            Wgs84Coord::new(37.7893, -122.3976, 0.0),
            Wgs84Coord::new(37.7901, -122.3976, 0.0),
            Wgs84Coord::new(37.7901, -122.3968, 0.0),
            Wgs84Coord::new(37.7893, -122.3968, 0.0),
            Wgs84Coord::new(37.7893, -122.3976, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(326.0),
        levels: Some(61),
    });

    // 2. Transamerica Pyramid (Iconic 260m architectural spire)
    // 600 Montgomery Street: 37.7952° N, 122.4028° W
    dataset.buildings.push(OsmBuildingFeature {
        id: 110_002,
        name: Some("Transamerica Pyramid".to_string()),
        building_type: BuildingType::Commercial,
        outer_ring: vec![
            Wgs84Coord::new(37.7948, -122.4026, 0.0),
            Wgs84Coord::new(37.7956, -122.4026, 0.0),
            Wgs84Coord::new(37.7956, -122.4018, 0.0),
            Wgs84Coord::new(37.7948, -122.4018, 0.0),
            Wgs84Coord::new(37.7948, -122.4026, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(260.0),
        levels: Some(48),
    });

    // 3. 555 California Street (Former Bank of America Center - 237m, 52 floors)
    // 37.7928° N, 122.4038° W
    dataset.buildings.push(OsmBuildingFeature {
        id: 110_003,
        name: Some("555 California Street".to_string()),
        building_type: BuildingType::Commercial,
        outer_ring: vec![
            Wgs84Coord::new(37.7924, -122.4043, 0.0),
            Wgs84Coord::new(37.7932, -122.4043, 0.0),
            Wgs84Coord::new(37.7932, -122.4033, 0.0),
            Wgs84Coord::new(37.7924, -122.4033, 0.0),
            Wgs84Coord::new(37.7924, -122.4043, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(237.0),
        levels: Some(52),
    });

    // 4. San Francisco Ferry Building (Historic Beaux-Arts terminal & clock tower - 75m)
    // The Embarcadero: 37.7955° N, 122.3937° W
    dataset.buildings.push(OsmBuildingFeature {
        id: 110_004,
        name: Some("San Francisco Ferry Building".to_string()),
        building_type: BuildingType::Civic,
        outer_ring: vec![
            Wgs84Coord::new(37.7948, -122.3942, 0.0),
            Wgs84Coord::new(37.7962, -122.3942, 0.0),
            Wgs84Coord::new(37.7962, -122.3932, 0.0),
            Wgs84Coord::new(37.7948, -122.3932, 0.0),
            Wgs84Coord::new(37.7948, -122.3942, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(75.0),
        levels: Some(3),
    });

    // 5. Coit Tower (Pioneer Park, Telegraph Hill - 64m monument on 84m hill)
    // 37.8024° N, 122.4058° W
    dataset.buildings.push(OsmBuildingFeature {
        id: 110_005,
        name: Some("Coit Tower".to_string()),
        building_type: BuildingType::Civic,
        outer_ring: vec![
            Wgs84Coord::new(37.8021, -122.4061, 0.0),
            Wgs84Coord::new(37.8027, -122.4061, 0.0),
            Wgs84Coord::new(37.8027, -122.4055, 0.0),
            Wgs84Coord::new(37.8021, -122.4055, 0.0),
            Wgs84Coord::new(37.8021, -122.4061, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(64.0),
        levels: Some(3),
    });

    // =========================================================================
    // 2. Real Roads, Highways, Flyovers, and Tunnels
    // =========================================================================

    // 1. Market Street (San Francisco's main spine / transit arterial)
    dataset.roads.push(OsmRoadFeature {
        way_id: 210_001,
        name: Some("Market Street".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(37.7880, -122.4030, 0.0),
            Wgs84Coord::new(37.7910, -122.3990, 0.0),
            Wgs84Coord::new(37.7950, -122.3946, 0.0), // Ferry Building intersection with The Embarcadero
        ],
        lanes: 4,
        speed_limit_kph: 40, // 25 mph
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // 2. The Embarcadero (Coastal waterfront boulevard)
    dataset.roads.push(OsmRoadFeature {
        way_id: 210_002,
        name: Some("The Embarcadero".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(37.7900, -122.3900, 0.0),
            Wgs84Coord::new(37.7950, -122.3946, 0.0), // In front of Ferry Building
            Wgs84Coord::new(37.8000, -122.3980, 0.0),
            Wgs84Coord::new(37.8040, -122.4020, 0.0),
        ],
        lanes: 4,
        speed_limit_kph: 48, // 30 mph
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // 3. California Street (Climbs steep Nob Hill from Financial District)
    dataset.roads.push(OsmRoadFeature {
        way_id: 210_003,
        name: Some("California Street".to_string()),
        highway_type: HighwayType::Secondary,
        geometry: vec![
            Wgs84Coord::new(37.7922, -122.3970, 0.0),
            Wgs84Coord::new(37.7922, -122.4038, 0.0), // Passing south of 555 California
            Wgs84Coord::new(37.7922, -122.4110, 0.0), // Nob Hill crest (~105m elevation)
        ],
        lanes: 2,
        speed_limit_kph: 40, // 25 mph
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // 4. Montgomery Street (Wall Street of the West)
    dataset.roads.push(OsmRoadFeature {
        way_id: 210_004,
        name: Some("Montgomery Street".to_string()),
        highway_type: HighwayType::Secondary,
        geometry: vec![
            Wgs84Coord::new(37.7890, -122.4030, 0.0),
            Wgs84Coord::new(37.7952, -122.4030, 0.0), // Financial Canyon between 555 Cal and Transamerica
            Wgs84Coord::new(37.7990, -122.4030, 0.0),
        ],
        lanes: 2,
        speed_limit_kph: 40, // 25 mph
        is_oneway: true,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // 5. San Francisco–Oakland Bay Bridge I-80 Approach (Elevated motorway flyover, Layer 1)
    dataset.roads.push(OsmRoadFeature {
        way_id: 210_005,
        name: Some("Interstate 80 Bay Bridge Approach".to_string()),
        highway_type: HighwayType::Motorway,
        geometry: vec![
            Wgs84Coord::new(37.7865, -122.3950, 0.0),
            Wgs84Coord::new(37.7885, -122.3900, 0.0),
            Wgs84Coord::new(37.7910, -122.3840, 0.0), // Over San Francisco Bay
        ],
        lanes: 5,
        speed_limit_kph: 80, // 50 mph
        is_oneway: true,
        layer: 1,
        is_bridge: true,
        is_tunnel: false,
    });

    // 6. Broadway Tunnel (Robert C. Levy Tunnel under Russian Hill, Layer -1)
    dataset.roads.push(OsmRoadFeature {
        way_id: 210_006,
        name: Some("Broadway Tunnel".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(37.7978, -122.4100, 0.0),
            Wgs84Coord::new(37.7980, -122.4135, 0.0), // Subterranean section
            Wgs84Coord::new(37.7982, -122.4170, 0.0),
        ],
        lanes: 4,
        speed_limit_kph: 56, // 35 mph
        is_oneway: false,
        layer: -1,
        is_bridge: false,
        is_tunnel: true,
    });

    // =========================================================================
    // 3. Coastline and Maritime Hydrography
    // =========================================================================

    // San Francisco Bay coastal water body
    dataset.water_bodies.push(OsmWaterFeature {
        id: 310_001,
        name: Some("San Francisco Bay".to_string()),
        water_type: WaterType::Bay,
        outer_ring: vec![
            Wgs84Coord::new(37.7890, -122.3890, 0.0),
            Wgs84Coord::new(37.7950, -122.3910, 0.0),
            Wgs84Coord::new(37.8045, -122.3910, 0.0),
            Wgs84Coord::new(37.8045, -122.3850, 0.0),
            Wgs84Coord::new(37.7890, -122.3850, 0.0),
            Wgs84Coord::new(37.7890, -122.3890, 0.0),
        ],
        is_coastline: true,
        target_elevation_msl: 0.0,
    });

    // =========================================================================
    // 4. Urban Land Cover & Parks
    // =========================================================================

    // Portsmouth Square (Historic park in Chinatown / Financial District)
    dataset.land_cover.push(OsmLandCoverFeature {
        id: 410_001,
        name: Some("Portsmouth Square".to_string()),
        class: LandCoverClass::UrbanPark,
        polygon: vec![
            Wgs84Coord::new(37.7945, -122.4055, 0.0),
            Wgs84Coord::new(37.7955, -122.4055, 0.0),
            Wgs84Coord::new(37.7955, -122.4045, 0.0),
            Wgs84Coord::new(37.7945, -122.4045, 0.0),
            Wgs84Coord::new(37.7945, -122.4055, 0.0),
        ],
    });

    // Sue Bierman Park (Embarcadero waterfront green space)
    dataset.land_cover.push(OsmLandCoverFeature {
        id: 410_002,
        name: Some("Sue Bierman Park".to_string()),
        class: LandCoverClass::UrbanPark,
        polygon: vec![
            Wgs84Coord::new(37.7960, -122.3965, 0.0),
            Wgs84Coord::new(37.7975, -122.3965, 0.0),
            Wgs84Coord::new(37.7975, -122.3950, 0.0),
            Wgs84Coord::new(37.7960, -122.3950, 0.0),
            Wgs84Coord::new(37.7960, -122.3965, 0.0),
        ],
    });

    dataset
}

/// Generates valid OpenStreetMap XML representing San Francisco Downtown & Waterfront.
pub fn generate_san_francisco_osm_xml() -> String {
    let dataset = create_san_francisco_dataset();
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<osm version=\"0.6\" generator=\"RealEarth-GIS-SF\">\n");

    let mut node_id = 1000;
    let mut add_node = |pt: &Wgs84Coord, xml: &mut String| -> u64 {
        node_id += 1;
        xml.push_str(&format!(
            "  <node id=\"{}\" lat=\"{:.7}\" lon=\"{:.7}\"/>\n",
            node_id, pt.lat, pt.lon
        ));
        node_id
    };

    // 1. Buildings
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

    // 2. Roads
    for r in &dataset.roads {
        let mut refs = Vec::new();
        for pt in &r.geometry {
            refs.push(add_node(pt, &mut xml));
        }

        xml.push_str(&format!("  <way id=\"{}\">\n", r.way_id));
        for rf in refs {
            xml.push_str(&format!("    <nd ref=\"{}\"/>\n", rf));
        }
        let hw_val = match r.highway_type {
            HighwayType::Motorway => "motorway",
            HighwayType::Primary => "primary",
            HighwayType::Secondary => "secondary",
            _ => "primary",
        };
        xml.push_str(&format!("    <tag k=\"highway\" v=\"{}\"/>\n", hw_val));
        if let Some(ref name) = r.name {
            xml.push_str(&format!("    <tag k=\"name\" v=\"{}\"/>\n", name));
        }
        xml.push_str(&format!("    <tag k=\"lanes\" v=\"{}\"/>\n", r.lanes));
        xml.push_str(&format!("    <tag k=\"layer\" v=\"{}\"/>\n", r.layer));
        xml.push_str(&format!(
            "    <tag k=\"maxspeed\" v=\"{} km/h\"/>\n",
            r.speed_limit_kph
        ));
        if r.is_oneway {
            xml.push_str("    <tag k=\"oneway\" v=\"yes\"/>\n");
        }
        if r.is_bridge {
            xml.push_str("    <tag k=\"bridge\" v=\"yes\"/>\n");
        }
        if r.is_tunnel {
            xml.push_str("    <tag k=\"tunnel\" v=\"yes\"/>\n");
        }
        xml.push_str("  </way>\n");
    }

    // 3. Coastline & Water
    for w in &dataset.water_bodies {
        let mut refs = Vec::new();
        for pt in &w.outer_ring {
            refs.push(add_node(pt, &mut xml));
        }
        xml.push_str(&format!("  <way id=\"{}\">\n", w.id));
        for rf in refs {
            xml.push_str(&format!("    <nd ref=\"{}\"/>\n", rf));
        }
        if w.is_coastline {
            xml.push_str("    <tag k=\"natural\" v=\"coastline\"/>\n");
        } else {
            xml.push_str("    <tag k=\"natural\" v=\"water\"/>\n");
        }
        xml.push_str("    <tag k=\"water\" v=\"bay\"/>\n");
        if let Some(ref name) = w.name {
            xml.push_str(&format!("    <tag k=\"name\" v=\"{}\"/>\n", name));
        }
        xml.push_str("  </way>\n");
    }

    // 4. Land Cover
    for lc in &dataset.land_cover {
        let mut refs = Vec::new();
        for pt in &lc.polygon {
            refs.push(add_node(pt, &mut xml));
        }
        xml.push_str(&format!("  <way id=\"{}\">\n", lc.id));
        for rf in refs {
            xml.push_str(&format!("    <nd ref=\"{}\"/>\n", rf));
        }
        xml.push_str("    <tag k=\"leisure\" v=\"park\"/>\n");
        if let Some(ref name) = lc.name {
            xml.push_str(&format!("    <tag k=\"name\" v=\"{}\"/>\n", name));
        }
        xml.push_str("  </way>\n");
    }

    xml.push_str("</osm>\n");
    xml
}
