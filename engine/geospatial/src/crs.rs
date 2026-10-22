//! Extensible Coordinate Reference System (CRS) Registry,
//! Cross-Country Datum Handling, and Universal Projection Mapping.

use crate::utm::utm_zone;
use realearth_shared::coordinates::Wgs84Coord;
use serde::{Deserialize, Serialize};

/// Type classification for Coordinate Reference Systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrsKind {
    Geographic2D,
    Geographic3D,
    ProjectedUtm,
    ProjectedNationalGrid,
    GeocentricEcef,
}

/// Standardized vertical datum definitions across countries and jurisdictions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StandardVerticalDatum {
    Wgs84Ellipsoidal,
    Egm96Geoid,
    Egm2008Geoid,
    Navd88,              // North American Vertical Datum 1988 (US)
    Evrf2019,            // European Vertical Reference Frame 2019 (EU)
    OrdnanceDatumNewlyn, // UK ODN
    TokyoPeil,           // Japan T.P.
    MeanSeaLevel,
}

impl StandardVerticalDatum {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Wgs84Ellipsoidal => "WGS 84 Ellipsoidal Height",
            Self::Egm96Geoid => "Earth Gravitational Model 1996 (EGM96)",
            Self::Egm2008Geoid => "Earth Gravitational Model 2008 (EGM2008)",
            Self::Navd88 => "North American Vertical Datum 1988 (NAVD88)",
            Self::Evrf2019 => "European Vertical Reference Frame 2019 (EVRF2019)",
            Self::OrdnanceDatumNewlyn => "Ordnance Datum Newlyn (ODN - UK)",
            Self::TokyoPeil => "Tokyo Peil (T.P. - Japan)",
            Self::MeanSeaLevel => "Global Mean Sea Level (MSL)",
        }
    }
}

/// Metadata and mathematical properties for a registered CRS.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrsDefinition {
    pub epsg_code: u32,
    pub name: &'static str,
    pub datum: &'static str,
    pub kind: CrsKind,
    pub utm_zone: Option<u8>,
    pub is_north: Option<bool>,
    pub default_vertical_datum: StandardVerticalDatum,
}

/// Universal CRS Registry managing worldwide projections and datums.
pub struct CrsRegistry;

impl CrsRegistry {
    /// Looks up a CRS definition by standard EPSG code.
    pub fn lookup(epsg: u32) -> Option<CrsDefinition> {
        // Universal UTM North: 32601..=32660
        if (32601..=32660).contains(&epsg) {
            let zone = (epsg - 32600) as u8;
            return Some(CrsDefinition {
                epsg_code: epsg,
                name: "WGS 84 / UTM North",
                datum: "WGS 84",
                kind: CrsKind::ProjectedUtm,
                utm_zone: Some(zone),
                is_north: Some(true),
                default_vertical_datum: StandardVerticalDatum::Egm96Geoid,
            });
        }

        // Universal UTM South: 32701..=32760
        if (32701..=32760).contains(&epsg) {
            let zone = (epsg - 32700) as u8;
            return Some(CrsDefinition {
                epsg_code: epsg,
                name: "WGS 84 / UTM South",
                datum: "WGS 84",
                kind: CrsKind::ProjectedUtm,
                utm_zone: Some(zone),
                is_north: Some(false),
                default_vertical_datum: StandardVerticalDatum::Egm96Geoid,
            });
        }

        match epsg {
            4326 => Some(CrsDefinition {
                epsg_code: 4326,
                name: "WGS 84 - World Geodetic System 1984",
                datum: "WGS 84",
                kind: CrsKind::Geographic2D,
                utm_zone: None,
                is_north: None,
                default_vertical_datum: StandardVerticalDatum::Wgs84Ellipsoidal,
            }),
            3857 => Some(CrsDefinition {
                epsg_code: 3857,
                name: "WGS 84 / Pseudo-Mercator (Web Mercator)",
                datum: "WGS 84",
                kind: CrsKind::ProjectedNationalGrid,
                utm_zone: None,
                is_north: None,
                default_vertical_datum: StandardVerticalDatum::MeanSeaLevel,
            }),
            27700 => Some(CrsDefinition {
                epsg_code: 27700,
                name: "OSGB36 / British National Grid (United Kingdom)",
                datum: "OSGB 1936",
                kind: CrsKind::ProjectedNationalGrid,
                utm_zone: Some(30),
                is_north: Some(true),
                default_vertical_datum: StandardVerticalDatum::OrdnanceDatumNewlyn,
            }),
            2154 => Some(CrsDefinition {
                epsg_code: 2154,
                name: "RGF93 / Lambert-93 (France)",
                datum: "Réseau Géodésique Français 1993",
                kind: CrsKind::ProjectedNationalGrid,
                utm_zone: Some(31),
                is_north: Some(true),
                default_vertical_datum: StandardVerticalDatum::Evrf2019,
            }),
            25832 => Some(CrsDefinition {
                epsg_code: 25832,
                name: "ETRS89 / UTM zone 32N (Germany/Central Europe)",
                datum: "European Terrestrial Reference System 1989",
                kind: CrsKind::ProjectedNationalGrid,
                utm_zone: Some(32),
                is_north: Some(true),
                default_vertical_datum: StandardVerticalDatum::Evrf2019,
            }),
            6677 => Some(CrsDefinition {
                epsg_code: 6677,
                name: "JGD2011 / Japan Plane Rectangular CS IX (Tokyo/Kanto)",
                datum: "Japanese Geodetic Datum 2011",
                kind: CrsKind::ProjectedNationalGrid,
                utm_zone: Some(54),
                is_north: Some(true),
                default_vertical_datum: StandardVerticalDatum::TokyoPeil,
            }),
            7755 => Some(CrsDefinition {
                epsg_code: 7755,
                name: "WGS 84 / India National Grid",
                datum: "WGS 84",
                kind: CrsKind::ProjectedNationalGrid,
                utm_zone: Some(43),
                is_north: Some(true),
                default_vertical_datum: StandardVerticalDatum::Egm96Geoid,
            }),
            _ => None,
        }
    }

    /// Automatically determines the appropriate projected UTM EPSG code for any WGS84 point.
    pub fn auto_detect_utm_epsg(coord: Wgs84Coord) -> u32 {
        let zone = utm_zone(coord.lon);
        if coord.lat >= 0.0 {
            32600 + (zone as u32)
        } else {
            32700 + (zone as u32)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utm_auto_zoning_cross_continents() {
        // San Francisco (USA): 37.7952°N, -122.4028°W -> Zone 10 North -> EPSG:32610
        let sf = Wgs84Coord::new(37.7952, -122.4028, 0.0);
        assert_eq!(CrsRegistry::auto_detect_utm_epsg(sf), 32610);

        // Bengaluru (India): 12.9797°N, 77.5907°E -> Zone 43 North -> EPSG:32643
        let blr = Wgs84Coord::new(12.9797, 77.5907, 0.0);
        assert_eq!(CrsRegistry::auto_detect_utm_epsg(blr), 32643);

        // Chennai (India): 13.0827°N, 80.2755°E -> Zone 44 North -> EPSG:32644
        let chn = Wgs84Coord::new(13.0827, 80.2755, 0.0);
        assert_eq!(CrsRegistry::auto_detect_utm_epsg(chn), 32644);

        // Sydney (Australia - Southern Hemisphere): 33.8688°S, 151.2093°E -> Zone 56 South -> EPSG:32756
        let syd = Wgs84Coord::new(-33.8688, 151.2093, 0.0);
        assert_eq!(CrsRegistry::auto_detect_utm_epsg(syd), 32756);

        // Paris (France): 48.8566°N, 2.3522°E -> Zone 31 North -> EPSG:32631
        let par = Wgs84Coord::new(48.8566, 2.3522, 0.0);
        assert_eq!(CrsRegistry::auto_detect_utm_epsg(par), 32631);
    }

    #[test]
    fn test_lookup_national_and_global_projections() {
        let bng = CrsRegistry::lookup(27700).expect("Must find British National Grid");
        assert_eq!(bng.kind, CrsKind::ProjectedNationalGrid);
        assert_eq!(
            bng.default_vertical_datum,
            StandardVerticalDatum::OrdnanceDatumNewlyn
        );

        let lambert = CrsRegistry::lookup(2154).expect("Must find French Lambert-93");
        assert_eq!(
            lambert.default_vertical_datum,
            StandardVerticalDatum::Evrf2019
        );

        let tokyo = CrsRegistry::lookup(6677).expect("Must find JGD2011 Tokyo");
        assert_eq!(
            tokyo.default_vertical_datum,
            StandardVerticalDatum::TokyoPeil
        );

        let wgs84 = CrsRegistry::lookup(4326).expect("Must find WGS 84");
        assert_eq!(wgs84.kind, CrsKind::Geographic2D);
    }
}
