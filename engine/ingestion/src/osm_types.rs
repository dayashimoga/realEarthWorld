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

/// Specific classification for water bodies and maritime coastlines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WaterType {
    #[default]
    Ocean,
    Bay,
    River,
    Canal,
    Lake,
    Wetland,
    Other,
}

impl WaterType {
    pub fn from_tag(tag: &str) -> Self {
        match tag {
            "ocean" | "sea" => Self::Ocean,
            "bay" | "harbour" => Self::Bay,
            "river" | "stream" | "riverbank" => Self::River,
            "canal" => Self::Canal,
            "lake" | "reservoir" | "pond" | "tank" => Self::Lake,
            "wetland" | "marsh" => Self::Wetland,
            _ => Self::Other,
        }
    }
}

/// Water body feature (lake, bay, river, coastline).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OsmWaterFeature {
    pub id: u64,
    pub name: Option<String>,
    pub water_type: WaterType,
    pub outer_ring: Vec<Wgs84Coord>,
    pub is_coastline: bool,
    pub target_elevation_msl: f32,
}

/// Authoritative Land Cover Classification (aligned with USGS NLCD).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LandCoverClass {
    OpenWater,
    DevelopedOpenSpace,
    DevelopedLowIntensity,
    DevelopedMediumIntensity,
    #[default]
    DevelopedHighIntensity,
    DeciduousForest,
    EvergreenForest,
    MixedForest,
    ShrubScrub,
    GrasslandHerbaceous,
    PastureHay,
    CultivatedCrops,
    WoodyWetlands,
    EmergentHerbaceousWetlands,
    UrbanPark,
    Other,
}

impl LandCoverClass {
    pub fn from_tags(landuse: Option<&str>, leisure: Option<&str>, natural: Option<&str>) -> Self {
        if let Some(l) = leisure {
            if l == "park" || l == "garden" || l == "recreation_ground" {
                return Self::UrbanPark;
            }
        }
        if let Some(n) = natural {
            match n {
                "wood" | "tree_row" => return Self::DeciduousForest,
                "scrub" | "heath" => return Self::ShrubScrub,
                "grassland" => return Self::GrasslandHerbaceous,
                "wetland" => return Self::WoodyWetlands,
                "water" | "bay" => return Self::OpenWater,
                _ => {}
            }
        }
        if let Some(lu) = landuse {
            match lu {
                "forest" => return Self::DeciduousForest,
                "meadow" | "grass" => return Self::GrasslandHerbaceous,
                "farmland" | "farm" => return Self::CultivatedCrops,
                "commercial" | "retail" => return Self::DevelopedHighIntensity,
                "residential" => return Self::DevelopedMediumIntensity,
                "industrial" => return Self::DevelopedHighIntensity,
                _ => {}
            }
        }
        Self::Other
    }
}

/// Normalized land cover / vegetation parcel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OsmLandCoverFeature {
    pub id: u64,
    pub name: Option<String>,
    pub class: LandCoverClass,
    pub polygon: Vec<Wgs84Coord>,
}

/// Railway track classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RailwayType {
    Mainline,
    #[default]
    LightRail,
    Subway,
    Tram,
    HighSpeed,
    Spur,
    Other,
}

impl RailwayType {
    pub fn from_tag(tag: &str) -> Self {
        match tag {
            "rail" => Self::Mainline,
            "light_rail" => Self::LightRail,
            "subway" => Self::Subway,
            "tram" => Self::Tram,
            "highspeed" => Self::HighSpeed,
            "spur" | "siding" | "yard" => Self::Spur,
            _ => Self::Other,
        }
    }
}

/// Normalized railway track segment extracted from OSM.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OsmRailwayFeature {
    pub way_id: u64,
    pub name: Option<String>,
    pub railway_type: RailwayType,
    pub geometry: Vec<Wgs84Coord>,
    pub layer: i8,
    pub is_bridge: bool,
    pub is_tunnel: bool,
    pub is_electrified: bool,
    pub gauge_mm: Option<u32>,
}

/// Aeroway facility classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AerowayType {
    #[default]
    Runway,
    Taxiway,
    Apron,
    Helipad,
    Gate,
    Terminal,
    Other,
}

impl AerowayType {
    pub fn from_tag(tag: &str) -> Self {
        match tag {
            "runway" => Self::Runway,
            "taxiway" => Self::Taxiway,
            "apron" => Self::Apron,
            "helipad" => Self::Helipad,
            "gate" => Self::Gate,
            "terminal" => Self::Terminal,
            _ => Self::Other,
        }
    }
}

/// Normalized airport / aeroway feature.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OsmAerowayFeature {
    pub id: u64,
    pub name: Option<String>,
    pub aeroway_type: AerowayType,
    pub geometry: Vec<Wgs84Coord>,
    pub width_m: Option<f32>,
    pub ref_code: Option<String>,
}

/// Maritime port facility classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PortType {
    #[default]
    Pier,
    Dock,
    Berth,
    Slipway,
    FerryTerminal,
    Other,
}

impl PortType {
    pub fn from_tag(tag: &str) -> Self {
        match tag {
            "pier" => Self::Pier,
            "dock" => Self::Dock,
            "berth" => Self::Berth,
            "slipway" => Self::Slipway,
            "ferry_terminal" => Self::FerryTerminal,
            _ => Self::Other,
        }
    }
}

/// Normalized port / dock / maritime feature.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OsmPortFeature {
    pub id: u64,
    pub name: Option<String>,
    pub port_type: PortType,
    pub polygon: Vec<Wgs84Coord>,
    pub min_depth_m: Option<f32>,
}

/// Collection of raw ingested features for a geographic area.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct IngestedDataset {
    pub roads: Vec<OsmRoadFeature>,
    pub buildings: Vec<OsmBuildingFeature>,
    pub water_bodies: Vec<OsmWaterFeature>,
    pub land_cover: Vec<OsmLandCoverFeature>,
    #[serde(default)]
    pub railways: Vec<OsmRailwayFeature>,
    #[serde(default)]
    pub aeroways: Vec<OsmAerowayFeature>,
    #[serde(default)]
    pub ports: Vec<OsmPortFeature>,
}
