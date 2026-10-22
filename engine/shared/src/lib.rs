//! Real Earth Shared Foundations
//! Core mathematical, units, floating-origin and error primitives.

pub mod bounds;
pub mod coordinates;
pub mod errors;
pub mod region;
pub mod us_catalog;

pub use region::RegionProfile;
pub use us_catalog::{
    all_interstate_corridors, all_us_cities, cities_by_priority, find_city_by_id, search_cities,
    CityCoverageReport, CityPriorityTier, CoverageLevel, DataSourceProvenance, InterstateCorridor,
    UsCityDefinition, US_CITY_CATALOG, US_INTERSTATE_CORRIDORS,
};
