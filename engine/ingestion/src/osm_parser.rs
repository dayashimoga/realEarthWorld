//! Streaming OpenStreetMap XML parser.

use crate::osm_types::{
    BuildingType, HighwayType, IngestedDataset, LandCoverClass, OsmBuildingFeature,
    OsmLandCoverFeature, OsmRoadFeature, OsmWaterFeature, WaterType,
};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use realearth_shared::coordinates::Wgs84Coord;
use realearth_shared::errors::{RealEarthError, Result};
use std::collections::HashMap;

/// Parses OSM XML string or buffer into an IngestedDataset.
pub fn parse_osm_xml(xml_content: &str) -> Result<IngestedDataset> {
    let mut reader = Reader::from_str(xml_content);
    reader.config_mut().trim_text(true);

    let mut nodes: HashMap<u64, Wgs84Coord> = HashMap::new();
    let mut dataset = IngestedDataset::default();

    let mut current_way_id: Option<u64> = None;
    let mut current_way_nodes: Vec<u64> = Vec::new();
    let mut current_way_tags: HashMap<String, String> = HashMap::new();

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let name = e.name();
                match name.as_ref() {
                    b"node" => {
                        let mut id = None;
                        let mut lat = None;
                        let mut lon = None;

                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"id" => {
                                    if let Ok(v) = std::str::from_utf8(&attr.value) {
                                        id = v.parse::<u64>().ok();
                                    }
                                }
                                b"lat" => {
                                    if let Ok(v) = std::str::from_utf8(&attr.value) {
                                        lat = v.parse::<f64>().ok();
                                    }
                                }
                                b"lon" => {
                                    if let Ok(v) = std::str::from_utf8(&attr.value) {
                                        lon = v.parse::<f64>().ok();
                                    }
                                }
                                _ => {}
                            }
                        }

                        if let (Some(id), Some(lat), Some(lon)) = (id, lat, lon) {
                            nodes.insert(id, Wgs84Coord::new(lat, lon, 0.0));
                        }
                    }
                    b"way" => {
                        current_way_nodes.clear();
                        current_way_tags.clear();
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"id" {
                                if let Ok(v) = std::str::from_utf8(&attr.value) {
                                    current_way_id = v.parse::<u64>().ok();
                                }
                            }
                        }
                    }
                    b"nd" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"ref" {
                                if let Ok(v) = std::str::from_utf8(&attr.value) {
                                    if let Ok(node_ref) = v.parse::<u64>() {
                                        current_way_nodes.push(node_ref);
                                    }
                                }
                            }
                        }
                    }
                    b"tag" => {
                        let mut k = None;
                        let mut v = None;
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"k" => {
                                    if let Ok(s) = std::str::from_utf8(&attr.value) {
                                        k = Some(s.to_string());
                                    }
                                }
                                b"v" => {
                                    if let Ok(s) = std::str::from_utf8(&attr.value) {
                                        v = Some(s.to_string());
                                    }
                                }
                                _ => {}
                            }
                        }
                        if let (Some(k), Some(v)) = (k, v) {
                            current_way_tags.insert(k, v);
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"way" {
                    if let Some(way_id) = current_way_id.take() {
                        let coords: Vec<Wgs84Coord> = current_way_nodes
                            .iter()
                            .filter_map(|id| nodes.get(id).copied())
                            .collect();

                        // Process building
                        if current_way_tags.contains_key("building") {
                            if coords.len() >= 3 {
                                let b_type = current_way_tags
                                    .get("building")
                                    .map(|s| BuildingType::from_tag(s))
                                    .unwrap_or(BuildingType::Other);

                                let explicit_height =
                                    current_way_tags.get("height").and_then(|h| {
                                        h.trim_end_matches('m').trim().parse::<f64>().ok()
                                    });

                                let levels = current_way_tags
                                    .get("building:levels")
                                    .and_then(|l| l.parse::<u32>().ok());

                                let name = current_way_tags.get("name").cloned();

                                dataset.buildings.push(OsmBuildingFeature {
                                    id: way_id,
                                    name,
                                    building_type: b_type,
                                    outer_ring: coords,
                                    inner_rings: Vec::new(),
                                    explicit_height,
                                    levels,
                                });
                            }
                        } else if let Some(hw_tag) = current_way_tags.get("highway") {
                            // Process highway
                            if coords.len() >= 2 {
                                let hw_type = HighwayType::from_tag(hw_tag);
                                let name = current_way_tags.get("name").cloned();

                                let lanes = current_way_tags
                                    .get("lanes")
                                    .and_then(|l| l.parse::<u32>().ok())
                                    .unwrap_or(2);

                                let speed_limit_kph = current_way_tags
                                    .get("maxspeed")
                                    .and_then(|s| {
                                        if let Some(mph) = s.strip_suffix(" mph") {
                                            mph.trim()
                                                .parse::<f32>()
                                                .map(|v| (v * 1.60934).round() as u32)
                                                .ok()
                                        } else {
                                            s.trim_end_matches(" km/h").parse::<u32>().ok()
                                        }
                                    })
                                    .unwrap_or_else(|| hw_type.default_speed_limit());

                                let is_oneway = current_way_tags
                                    .get("oneway")
                                    .map(|v| v == "yes" || v == "1")
                                    .unwrap_or(false);

                                let layer = current_way_tags
                                    .get("layer")
                                    .and_then(|l| l.parse::<i8>().ok())
                                    .unwrap_or(0);

                                let is_bridge = current_way_tags
                                    .get("bridge")
                                    .map(|b| b == "yes" || b == "viaduct")
                                    .unwrap_or(false);

                                let is_tunnel = current_way_tags
                                    .get("tunnel")
                                    .map(|t| t == "yes")
                                    .unwrap_or(false);

                                dataset.roads.push(OsmRoadFeature {
                                    way_id,
                                    name,
                                    highway_type: hw_type,
                                    geometry: coords,
                                    lanes,
                                    speed_limit_kph,
                                    is_oneway,
                                    layer,
                                    is_bridge,
                                    is_tunnel,
                                });
                            }
                        } else if (current_way_tags.get("natural").map(|s| s.as_str())
                            == Some("water")
                            || current_way_tags.get("natural").map(|s| s.as_str())
                                == Some("coastline")
                            || current_way_tags.contains_key("water"))
                            && coords.len() >= 3
                        {
                            let name = current_way_tags.get("name").cloned();
                            let is_coastline = current_way_tags.get("natural").map(|s| s.as_str())
                                == Some("coastline")
                                || current_way_tags.get("water").map(|s| s.as_str()) == Some("bay");
                            let w_tag = current_way_tags
                                .get("water")
                                .map(|s| s.as_str())
                                .unwrap_or("bay");
                            let water_type = WaterType::from_tag(w_tag);

                            dataset.water_bodies.push(OsmWaterFeature {
                                id: way_id,
                                name,
                                water_type,
                                outer_ring: coords,
                                is_coastline,
                                target_elevation_msl: 0.0,
                            });
                        } else if coords.len() >= 3
                            && (current_way_tags.contains_key("leisure")
                                || current_way_tags.contains_key("landuse"))
                        {
                            let leisure = current_way_tags.get("leisure").map(|s| s.as_str());
                            let landuse = current_way_tags.get("landuse").map(|s| s.as_str());
                            let natural = current_way_tags.get("natural").map(|s| s.as_str());
                            let class = LandCoverClass::from_tags(landuse, leisure, natural);

                            if class != LandCoverClass::Other {
                                dataset.land_cover.push(OsmLandCoverFeature {
                                    id: way_id,
                                    name: current_way_tags.get("name").cloned(),
                                    class,
                                    polygon: coords,
                                });
                            }
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(RealEarthError::Ingestion(format!("XML parse error: {}", e))),
            _ => {}
        }
        buf.clear();
    }

    Ok(dataset)
}
