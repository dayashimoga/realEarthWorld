//! Worldwide Data Source Manifests, Provenance Attribution,
//! Licensing Policy Enforcement, and Region-Specific Quality Profiles.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Standard global open geospatial data providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataSourceProvider {
    OpenStreetMap,
    CopernicusGlo30,
    CopernicusGlo90,
    Usgs3Dep,
    NasaSrtm,
    EsaWorldCover,
    UkOrdnanceSurvey,
    IgnFrance,
    GsiJapan,
}

impl DataSourceProvider {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::OpenStreetMap => "OpenStreetMap Foundation (OSMF)",
            Self::CopernicusGlo30 => "European Space Agency / Copernicus DEM GLO-30",
            Self::CopernicusGlo90 => "European Space Agency / Copernicus DEM GLO-90",
            Self::Usgs3Dep => "US Geological Survey 3D Elevation Program (3DEP)",
            Self::NasaSrtm => "NASA Shuttle Radar Topography Mission (SRTM)",
            Self::EsaWorldCover => "ESA WorldCover Global Land Cover 10m",
            Self::UkOrdnanceSurvey => "Ordnance Survey OpenData (United Kingdom)",
            Self::IgnFrance => {
                "Institut National de l'Information Géographique et Forestière (IGN)"
            }
            Self::GsiJapan => "Geospatial Information Authority of Japan (GSI)",
        }
    }
}

/// Standardized open licensing terms and compliance conditions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LicenseTerms {
    pub license_id: String,
    pub name: String,
    pub url: String,
    pub attribution_required: bool,
    pub share_alike_copyleft: bool,
    pub commercial_permitted: bool,
    pub redistribution_permitted: bool,
    pub attribution_notice: String,
}

impl LicenseTerms {
    pub fn odbl_1_0() -> Self {
        Self {
            license_id: "ODbL-1.0".to_string(),
            name: "Open Database License (ODbL) v1.0".to_string(),
            url: "https://opendatacommons.org/licenses/odbl/1-0/".to_string(),
            attribution_required: true,
            share_alike_copyleft: true,
            commercial_permitted: true,
            redistribution_permitted: true,
            attribution_notice: "© OpenStreetMap contributors (ODbL)".to_string(),
        }
    }

    pub fn copernicus_open_data() -> Self {
        Self {
            license_id: "Copernicus-OpenData".to_string(),
            name: "European Union Copernicus Legal Notice on Data Access".to_string(),
            url: "https://spacedata.copernicus.eu/terms-and-conditions".to_string(),
            attribution_required: true,
            share_alike_copyleft: false,
            commercial_permitted: true,
            redistribution_permitted: true,
            attribution_notice: "© European Space Agency (Copernicus DEM)".to_string(),
        }
    }

    pub fn us_public_domain() -> Self {
        Self {
            license_id: "US-PublicDomain".to_string(),
            name: "US Government Work / Public Domain".to_string(),
            url: "https://www.usa.gov/publicdomain/label/1.0/".to_string(),
            attribution_required: false,
            share_alike_copyleft: false,
            commercial_permitted: true,
            redistribution_permitted: true,
            attribution_notice: "Public Domain (USGS/NASA)".to_string(),
        }
    }

    pub fn cc_by_4_0() -> Self {
        Self {
            license_id: "CC-BY-4.0".to_string(),
            name: "Creative Commons Attribution 4.0 International".to_string(),
            url: "https://creativecommons.org/licenses/by/4.0/".to_string(),
            attribution_required: true,
            share_alike_copyleft: false,
            commercial_permitted: true,
            redistribution_permitted: true,
            attribution_notice: "© ESA WorldCover project / Contains modified Copernicus data"
                .to_string(),
        }
    }
}

/// Region-specific GIS quality thresholds and validation criteria.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QualityProfile {
    pub profile_name: String,
    pub max_dem_resolution_meters: f32,
    pub dem_vertical_accuracy_meters: f32,
    pub min_building_height_confidence: f32,
    pub max_terrain_slope_tolerance_deg: f32,
    pub requires_antimeridian_continuity: bool,
    pub grade_separated_clearance_min_meters: f32,
}

impl QualityProfile {
    pub fn urban_high_density() -> Self {
        Self {
            profile_name: "UrbanHighDensity".to_string(),
            max_dem_resolution_meters: 5.0,
            dem_vertical_accuracy_meters: 1.0,
            min_building_height_confidence: 0.80,
            max_terrain_slope_tolerance_deg: 45.0,
            requires_antimeridian_continuity: false,
            grade_separated_clearance_min_meters: 4.5,
        }
    }

    pub fn regional_intercity() -> Self {
        Self {
            profile_name: "RegionalIntercity".to_string(),
            max_dem_resolution_meters: 30.0,
            dem_vertical_accuracy_meters: 3.0,
            min_building_height_confidence: 0.60,
            max_terrain_slope_tolerance_deg: 60.0,
            requires_antimeridian_continuity: false,
            grade_separated_clearance_min_meters: 5.0,
        }
    }

    pub fn global_coarse_basemap() -> Self {
        Self {
            profile_name: "GlobalCoarseBasemap".to_string(),
            max_dem_resolution_meters: 90.0,
            dem_vertical_accuracy_meters: 10.0,
            min_building_height_confidence: 0.40,
            max_terrain_slope_tolerance_deg: 85.0,
            requires_antimeridian_continuity: true,
            grade_separated_clearance_min_meters: 4.0,
        }
    }
}

/// Global Data Source Manifest tracking worldwide datasets, licenses,
/// and region assignments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalDataSourceManifest {
    pub providers: HashMap<String, LicenseTerms>,
    pub region_profiles: HashMap<String, QualityProfile>,
}

impl Default for GlobalDataSourceManifest {
    fn default() -> Self {
        let mut providers = HashMap::new();
        providers.insert("osm".to_string(), LicenseTerms::odbl_1_0());
        providers.insert(
            "copernicus".to_string(),
            LicenseTerms::copernicus_open_data(),
        );
        providers.insert("usgs_3dep".to_string(), LicenseTerms::us_public_domain());
        providers.insert("nasa_srtm".to_string(), LicenseTerms::us_public_domain());
        providers.insert("esa_worldcover".to_string(), LicenseTerms::cc_by_4_0());

        let mut region_profiles = HashMap::new();
        region_profiles.insert("urban".to_string(), QualityProfile::urban_high_density());
        region_profiles.insert("regional".to_string(), QualityProfile::regional_intercity());
        region_profiles.insert(
            "global".to_string(),
            QualityProfile::global_coarse_basemap(),
        );

        Self {
            providers,
            region_profiles,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_data_source_manifest_licenses() {
        let manifest = GlobalDataSourceManifest::default();
        let osm = manifest.providers.get("osm").unwrap();
        assert!(osm.attribution_required);
        assert!(osm.share_alike_copyleft);
        assert_eq!(osm.license_id, "ODbL-1.0");

        let usgs = manifest.providers.get("usgs_3dep").unwrap();
        assert!(!usgs.attribution_required);
        assert!(!usgs.share_alike_copyleft);
    }

    #[test]
    fn test_quality_profiles_thresholds() {
        let urban = QualityProfile::urban_high_density();
        assert!(urban.max_dem_resolution_meters <= 5.0);
        assert!(urban.min_building_height_confidence >= 0.80);

        let global = QualityProfile::global_coarse_basemap();
        assert!(global.requires_antimeridian_continuity);
    }
}
