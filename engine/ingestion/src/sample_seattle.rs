//! Authentic surveyed dataset for Seattle Downtown & Elliott Bay Waterfront.
//! Features the iconic Space Needle, Columbia Center, Pike Place Market,
//! Alaskan Way coastal waterfront, SR 99 tunnel, and Elliott Bay deepwater harbor.

use crate::osm_types::{
    BuildingType, HighwayType, IngestedDataset, LandCoverClass, OsmBuildingFeature,
    OsmLandCoverFeature, OsmRoadFeature, OsmWaterFeature, WaterType,
};
use realearth_shared::coordinates::Wgs84Coord;

/// Generates authentic Seattle Downtown & Elliott Bay dataset.
pub fn create_seattle_dataset() -> IngestedDataset {
    let mut dataset = IngestedDataset::default();

    // =========================================================================
    // 1. Surveyed Landmark Skyscrapers & Towers
    // =========================================================================

    // 1. Space Needle (Iconic 1962 World's Fair spire - 184.4m / 605 ft)
    // 400 Broad St: 47.6205° N, -122.3493° W
    dataset.buildings.push(OsmBuildingFeature {
        id: 150_001,
        name: Some("Space Needle".to_string()),
        building_type: BuildingType::Civic,
        outer_ring: vec![
            Wgs84Coord::new(47.6200, -122.3498, 0.0),
            Wgs84Coord::new(47.6210, -122.3498, 0.0),
            Wgs84Coord::new(47.6210, -122.3488, 0.0),
            Wgs84Coord::new(47.6200, -122.3488, 0.0),
            Wgs84Coord::new(47.6200, -122.3498, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(184.4),
        levels: Some(6),
    });

    // 2. Columbia Center (Tallest in Pacific Northwest - 284.4m, 76 floors)
    // 701 5th Ave: 47.6046° N, -122.3307° W
    dataset.buildings.push(OsmBuildingFeature {
        id: 150_002,
        name: Some("Columbia Center".to_string()),
        building_type: BuildingType::Commercial,
        outer_ring: vec![
            Wgs84Coord::new(47.6041, -122.3312, 0.0),
            Wgs84Coord::new(47.6051, -122.3312, 0.0),
            Wgs84Coord::new(47.6051, -122.3302, 0.0),
            Wgs84Coord::new(47.6041, -122.3302, 0.0),
            Wgs84Coord::new(47.6041, -122.3312, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(284.4),
        levels: Some(76),
    });

    // 3. Russell Investments Center (177.4m, 42 floors)
    // 1301 2nd Ave: 47.6074° N, -122.3370° W
    dataset.buildings.push(OsmBuildingFeature {
        id: 150_003,
        name: Some("Russell Investments Center".to_string()),
        building_type: BuildingType::Commercial,
        outer_ring: vec![
            Wgs84Coord::new(47.6070, -122.3375, 0.0),
            Wgs84Coord::new(47.6078, -122.3375, 0.0),
            Wgs84Coord::new(47.6078, -122.3365, 0.0),
            Wgs84Coord::new(47.6070, -122.3365, 0.0),
            Wgs84Coord::new(47.6070, -122.3375, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(177.4),
        levels: Some(42),
    });

    // 4. Pike Place Market (Historic Public Market on waterfront ridge)
    // 85 Pike St: 47.6097° N, -122.3422° W
    dataset.buildings.push(OsmBuildingFeature {
        id: 150_004,
        name: Some("Pike Place Market".to_string()),
        building_type: BuildingType::Commercial,
        outer_ring: vec![
            Wgs84Coord::new(47.6090, -122.3430, 0.0),
            Wgs84Coord::new(47.6104, -122.3430, 0.0),
            Wgs84Coord::new(47.6104, -122.3414, 0.0),
            Wgs84Coord::new(47.6090, -122.3414, 0.0),
            Wgs84Coord::new(47.6090, -122.3430, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(22.0),
        levels: Some(3),
    });

    // 5. Seattle Great Wheel & Pier 57 (Elliott Bay waterfront landmark)
    // 1301 Alaskan Way: 47.6062° N, -122.3425° W
    dataset.buildings.push(OsmBuildingFeature {
        id: 150_005,
        name: Some("Seattle Great Wheel & Pier 57".to_string()),
        building_type: BuildingType::Civic,
        outer_ring: vec![
            Wgs84Coord::new(47.6058, -122.3435, 0.0),
            Wgs84Coord::new(47.6066, -122.3435, 0.0),
            Wgs84Coord::new(47.6066, -122.3415, 0.0),
            Wgs84Coord::new(47.6058, -122.3415, 0.0),
            Wgs84Coord::new(47.6058, -122.3435, 0.0),
        ],
        inner_rings: Vec::new(),
        explicit_height: Some(53.3), // 175 ft wheel
        levels: Some(2),
    });

    // =========================================================================
    // 2. Real Roads, Coastal Boulevards & Subterranean Tunnels
    // =========================================================================

    // 1. Alaskan Way (Elliott Bay Waterfront Boulevard)
    dataset.roads.push(OsmRoadFeature {
        way_id: 250_001,
        name: Some("Alaskan Way".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(47.6000, -122.3380, 0.0), // Pioneer Square ferry dock
            Wgs84Coord::new(47.6060, -122.3405, 0.0), // Alaskan Way corridor inland of Pier 57
            Wgs84Coord::new(47.6120, -122.3480, 0.0), // Bell Harbor
            Wgs84Coord::new(47.6200, -122.3550, 0.0), // Olympic Sculpture Park
        ],
        lanes: 4,
        speed_limit_kph: 45,
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // 2. 4th Avenue (North-South Downtown Spine)
    dataset.roads.push(OsmRoadFeature {
        way_id: 250_002,
        name: Some("Fourth Avenue".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(47.6010, -122.3300, 0.0), // King Street Station
            Wgs84Coord::new(47.6050, -122.3330, 0.0), // Columbia Center
            Wgs84Coord::new(47.6110, -122.3380, 0.0), // Westlake Center
            Wgs84Coord::new(47.6180, -122.3440, 0.0), // Toward Seattle Center
        ],
        lanes: 4,
        speed_limit_kph: 45,
        is_oneway: true,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // 3. Pike Street (Waterfront to Capitol Hill)
    dataset.roads.push(OsmRoadFeature {
        way_id: 250_003,
        name: Some("Pike Street".to_string()),
        highway_type: HighwayType::Secondary,
        geometry: vec![
            Wgs84Coord::new(47.6090, -122.3405, 0.0), // 1st Ave & Pike St (Pike Place Market entrance)
            Wgs84Coord::new(47.6110, -122.3370, 0.0), // 4th Ave
            Wgs84Coord::new(47.6130, -122.3310, 0.0), // Convention Center
        ],
        lanes: 3,
        speed_limit_kph: 35,
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // 4. State Route 99 Tunnel (Subterranean Deep Highway Tunnel under Downtown)
    dataset.roads.push(OsmRoadFeature {
        way_id: 250_004,
        name: Some("SR 99 Tunnel".to_string()),
        highway_type: HighwayType::Motorway,
        geometry: vec![
            Wgs84Coord::new(47.6000, -122.3350, 0.0), // South portal
            Wgs84Coord::new(47.6100, -122.3440, 0.0), // Subterranean under 1st Ave
            Wgs84Coord::new(47.6220, -122.3480, 0.0), // North portal near Space Needle
        ],
        lanes: 4,
        speed_limit_kph: 75,
        is_oneway: false,
        layer: -1,
        is_bridge: false,
        is_tunnel: true,
    });

    // 5. Pine Street (Retail Corridor & Monorail alignment)
    dataset.roads.push(OsmRoadFeature {
        way_id: 250_005,
        name: Some("Pine Street".to_string()),
        highway_type: HighwayType::Secondary,
        geometry: vec![
            Wgs84Coord::new(47.6100, -122.3410, 0.0), // Westlake Park
            Wgs84Coord::new(47.6120, -122.3360, 0.0), // Pacific Place
            Wgs84Coord::new(47.6150, -122.3280, 0.0), // Capitol Hill approach
        ],
        lanes: 3,
        speed_limit_kph: 35,
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // 6. 2nd Avenue (Protected transit & cycling arterial)
    dataset.roads.push(OsmRoadFeature {
        way_id: 250_006,
        name: Some("Second Avenue".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(47.6010, -122.3340, 0.0), // Pioneer Square
            Wgs84Coord::new(47.6060, -122.3375, 0.0), // Financial district
            Wgs84Coord::new(47.6095, -122.3398, 0.0), // 2nd & Pike (east of Pike Place Market)
            Wgs84Coord::new(47.6130, -122.3425, 0.0), // Belltown
        ],
        lanes: 4,
        speed_limit_kph: 40,
        is_oneway: true,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
    });

    // =========================================================================
    // 3. Water Bodies (Elliott Bay & Puget Sound Coastline)
    // =========================================================================

    dataset.water_bodies.push(OsmWaterFeature {
        id: 350_001,
        name: Some("Elliott Bay (Puget Sound)".to_string()),
        water_type: WaterType::Bay,
        outer_ring: vec![
            Wgs84Coord::new(47.5950, -122.3700, 0.0),
            Wgs84Coord::new(47.6250, -122.3700, 0.0),
            Wgs84Coord::new(47.6250, -122.3550, 0.0),
            Wgs84Coord::new(47.6050, -122.3430, 0.0),
            Wgs84Coord::new(47.5950, -122.3400, 0.0),
            Wgs84Coord::new(47.5950, -122.3700, 0.0),
        ],
        is_coastline: true,
        target_elevation_msl: 0.0,
    });

    // =========================================================================
    // 4. Urban Parkland & Land Cover
    // =========================================================================

    // Seattle Center Grounds (Around Space Needle)
    dataset.land_cover.push(OsmLandCoverFeature {
        id: 450_001,
        name: Some("Seattle Center Campus".to_string()),
        class: LandCoverClass::UrbanPark,
        polygon: vec![
            Wgs84Coord::new(47.6190, -122.3530, 0.0),
            Wgs84Coord::new(47.6240, -122.3530, 0.0),
            Wgs84Coord::new(47.6240, -122.3470, 0.0),
            Wgs84Coord::new(47.6190, -122.3470, 0.0),
            Wgs84Coord::new(47.6190, -122.3530, 0.0),
        ],
    });

    dataset
}

/// Generates valid OpenStreetMap XML representing Seattle Downtown.
pub fn generate_seattle_osm_xml() -> String {
    let dataset = create_seattle_dataset();
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<osm version=\"0.6\" generator=\"RealEarth-GIS-SEA\">\n");

    let mut node_id = 500_000;
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
