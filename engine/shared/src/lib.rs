//! Real Earth Shared Foundations
//! Core mathematical, units, floating-origin and error primitives.

pub mod bounds;
pub mod coordinates;
pub mod errors;
pub mod gameplay_recording;
pub mod metrics;
pub mod region;
pub mod us_catalog;

pub use gameplay_recording::{
    GameplayInputFrame, GameplayRecordingSession, RecordedCameraState, RecordedCollisionState,
    RecordedGeographicContext, RecordedTransform, RecordedVehicleState, ReplayVerificationSummary,
};
pub use metrics::{
    compute_percentile, get_current_process_memory, FramePacingMetrics,
    GeometryAndStreamingMetrics, ManagedObjectCounts, MetricStatus, PerformanceCertificationReport,
    ProcessMemoryMetrics, SystemResourceMetrics,
};
pub use region::RegionProfile;
pub use us_catalog::{
    all_interstate_corridors, all_us_cities, cities_by_data_tier, cities_by_priority,
    find_city_by_id, search_cities, tier1_benchmark_cities, tier2_definition_cities,
    CatalogDataTier, CityCoverageReport, CityPriorityTier, CoverageLevel, DataSourceProvenance,
    InterstateCorridor, UsCityDefinition, US_CITY_CATALOG, US_INTERSTATE_CORRIDORS,
};
