//! Data models for OpenStreetMap ingested features.

use realearth_shared::coordinates::Wgs84Coord;
use serde::{Deserialize, Serialize};

/// Highway classification derived from OSM `highway=*` tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HighwayType {
    Motorway,
    Trunk,
    Primary,
    Secondary,
    Tertiary,
    Residential,
    Service,
    Footway,
    Path,
    Other,
}

impl HighwayType {
    pub fn from_tag(tag: &str) -> Self {
        match tag {
            "motorway" | "motorway_link" => Self::Motorway,
            "trunk" | "trunk_link" => Self::Trunk,
            "primary" | "primary_link" => Self::Primary,
            "secondary" | "secondary_link" => Self::Secondary,
            "tertiary" | "tertiary_link" => Self::Tertiary,
            "residential" | "living_street" => Self::Residential,
            "service" => Self::Service,
            "footway" | "pedestrian" => Self::Footway,
            "path" | "track" => Self::Path,
            _ => Self::Other,
        }
    }

    /// Default width in meters per lane.
    pub fn default_lane_width(&self) -> f32 {
        match self {
            Self::Motorway | Self::Trunk => 3.75,
            Self::Primary | Self::Secondary => 3.5,
            Self::Tertiary | Self::Residential => 3.0,
            Self::Service => 2.5,
            Self::Footway | Self::Path => 1.5,
            Self::Other => 3.0,
        }
    }

    /// Default speed limit in km/h.
    pub fn default_speed_limit(&self) -> u32 {
        match self {
            Self::Motorway => 100,
            Self::Trunk => 80,
            Self::Primary => 60,
            Self::Secondary => 50,
            Self::Tertiary => 40,
            Self::Residential => 30,
            Self::Service => 20,
            Self::Footway | Self::Path => 5,
            Self::Other => 40,
        }
    }
}

/// Normalized road segment extracted from OSM.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OsmRoadFeature {
    pub way_id: u64,
    pub name: Option<String>,
    pub highway_type: HighwayType,
    pub geometry: Vec<Wgs84Coord>,
    pub lanes: u32,
    pub speed_limit_kph: u32,
    pub is_oneway: bool,
    /// Vertical layer index (-5 to +5, 0 = ground level).
    pub layer: i8,
    pub is_bridge: bool,
    pub is_tunnel: bool,
}

/// Building type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildingType {
    Civic,
    Commercial,
    Residential,
    Industrial,
    Stadium,
    Other,
}

impl BuildingType {
    pub fn from_tag(tag: &str) -> Self {
        match tag {
            "civic" | "government" | "public" => Self::Civic,
            "commercial" | "office" | "retail" => Self::Commercial,
            "residential" | "apartments" | "house" => Self::Residential,
            "industrial" | "warehouse" => Self::Industrial,
            "stadium" | "sports_centre" => Self::Stadium,
            _ => Self::Other,
        }
    }

    pub fn default_height(&self) -> f64 {
        match self {
            Self::Civic => 18.0,
            Self::Commercial => 25.0,
            Self::Residential => 12.0,
            Self::Industrial => 10.0,
            Self::Stadium => 30.0,
            Self::Other => 8.0,
        }
    }
}

/// Normalized building footprint extracted from OSM.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OsmBuildingFeature {
    pub id: u64,
    pub name: Option<String>,
    pub building_type: BuildingType,
    pub outer_ring: Vec<Wgs84Coord>,
    pub inner_rings: Vec<Vec<Wgs84Coord>>,
    pub explicit_height: Option<f64>,
    pub levels: Option<u32>,
}

/// Water body feature (lake, river, tank).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OsmWaterFeature {
    pub id: u64,
    pub name: Option<String>,
    pub outer_ring: Vec<Wgs84Coord>,
}

/// Collection of raw ingested features for a geographic area.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct IngestedDataset {
    pub roads: Vec<OsmRoadFeature>,
    pub buildings: Vec<OsmBuildingFeature>,
    pub water_bodies: Vec<OsmWaterFeature>,
}
