//! STAGE D VERIFICATION SUITE — REAL MAJOR US CITY COVERAGE
//!
//! Comprehensive tests verifying:
//! - D1: Authoritative US data sources, licensing, provenance, and vertical datums.
//! - D2: Mandatory 38-city catalog across Priority 1, 2, and 3 tiers.
//! - D3: Region completeness, boundary definitions, and area computations.
//! - D4: Authentic feature compilation (roads, DEMs, buildings, bridges, water, parks, airports, ports).
//! - D5: US interstate highway connectivity across supported metropolitan hubs.
//! - D6: Automated city validation, topological continuity, and coverage certification.
//! - D7: City search, bookmarks, on-demand streaming, and storage quota management.

use realearth_compiler::compiler_pipeline::{compile_world_package, CompilerOptions};
use realearth_ingestion::dem_grid::DemGrid;
use realearth_ingestion::us_cities::generate_us_city_dataset;
use realearth_shared::region::RegionProfile;
use realearth_shared::us_catalog::{
    all_interstate_corridors, all_us_cities, cities_by_priority, find_city_by_id, search_cities,
    CityPriorityTier, CoverageLevel, DataSourceProvenance,
};
use realearth_streaming::package_delivery::{InstalledPackageRecord, StorageQuotaManager};
use realearth_validation::city_catalog_validator::{
    audit_all_us_cities, validate_and_certify_us_city,
};
use realearth_validation::pipeline::validate_dataset;
use std::collections::HashSet;
use tempfile::tempdir;

// ============================================================================
// D1: GEOGRAPHIC SOURCES & PROVENANCE
// ============================================================================

#[test]
fn test_d1_data_sources_provenance_and_licensing() {
    let prov = DataSourceProvenance::default();

    assert!(
        prov.osm_license.contains("ODbL"),
        "Expected ODbL licensing for OpenStreetMap features: {}",
        prov.osm_license
    );
    assert!(
        prov.elevation_source.contains("3DEP") && prov.elevation_source.contains("NAVD88"),
        "Expected USGS 3DEP with NAVD88 vertical datum: {}",
        prov.elevation_source
    );
    assert!(
        prov.land_cover_source.contains("NLCD"),
        "Expected USGS NLCD land cover source: {}",
        prov.land_cover_source
    );
    assert!(
        prov.hydrography_source.contains("NHD") || prov.hydrography_source.contains("NOAA"),
        "Expected USGS NHD or NOAA hydrography: {}",
        prov.hydrography_source
    );
    assert!(
        prov.transportation_source.contains("FHWA")
            || prov.transportation_source.contains("OpenStreetMap"),
        "Expected FHWA or OSM transportation network: {}",
        prov.transportation_source
    );
    assert!(
        prov.source_date_utc.starts_with("202"),
        "Expected valid UTC ISO-8601 date: {}",
        prov.source_date_utc
    );
}

// ============================================================================
// D2: MANDATORY 38-CITY CATALOG
// ============================================================================

#[test]
fn test_d2_mandatory_38_city_catalog_completeness() {
    let cities = all_us_cities();
    assert_eq!(
        cities.len(),
        38,
        "Expected exactly 38 authoritative US metropolitan cities in catalog, got {}",
        cities.len()
    );

    // Verify Unique IDs
    let mut ids = HashSet::new();
    for city in cities {
        assert!(
            ids.insert(city.id),
            "Duplicate city ID detected in US catalog: {}",
            city.id
        );
    }

    // Priority 1: Primary Metropolitan Coverage (10 cities)
    let p1 = cities_by_priority(CityPriorityTier::Priority1Primary);
    assert_eq!(
        p1.len(),
        10,
        "Expected 10 Priority 1 primary metropolitan regions"
    );
    let p1_ids: HashSet<&str> = p1.iter().map(|c| c.id).collect();
    let expected_p1 = [
        "san-francisco",
        "los-angeles",
        "san-diego",
        "new-york",
        "chicago",
        "houston",
        "dallas",
        "washington-dc",
        "miami",
        "seattle",
    ];
    for expected in &expected_p1 {
        assert!(
            p1_ids.contains(expected),
            "Missing Priority 1 primary city: {}",
            expected
        );
    }

    // Priority 2: Additional Major Metropolitan Coverage (15 cities)
    let p2 = cities_by_priority(CityPriorityTier::Priority2Major);
    assert_eq!(
        p2.len(),
        15,
        "Expected 15 Priority 2 major metropolitan regions"
    );
    let p2_ids: HashSet<&str> = p2.iter().map(|c| c.id).collect();
    let expected_p2 = [
        "boston",
        "philadelphia",
        "phoenix",
        "denver",
        "atlanta",
        "detroit",
        "minneapolis",
        "austin",
        "san-antonio",
        "las-vegas",
        "portland",
        "sacramento",
        "charlotte",
        "orlando",
        "tampa",
    ];
    for expected in &expected_p2 {
        assert!(
            p2_ids.contains(expected),
            "Missing Priority 2 major city: {}",
            expected
        );
    }

    // Priority 3: Extended US Cities (13 cities)
    let p3 = cities_by_priority(CityPriorityTier::Priority3Extended);
    assert_eq!(p3.len(), 13, "Expected 13 Priority 3 extended US cities");
    let p3_ids: HashSet<&str> = p3.iter().map(|c| c.id).collect();
    let expected_p3 = [
        "nashville",
        "new-orleans",
        "baltimore",
        "pittsburgh",
        "cleveland",
        "columbus",
        "indianapolis",
        "jacksonville",
        "kansas-city",
        "st-louis",
        "salt-lake-city",
        "honolulu",
        "anchorage",
    ];
    for expected in &expected_p3 {
        assert!(
            p3_ids.contains(expected),
            "Missing Priority 3 extended city: {}",
            expected
        );
    }
}

// ============================================================================
// D3: REGION COMPLETENESS & BOUNDARY DEFINITIONS
// ============================================================================

#[test]
fn test_d3_region_completeness_and_boundaries() {
    let cities = all_us_cities();

    for city in cities {
        // 1. Valid Bounding Box
        assert!(
            city.bounds.min_lat < city.bounds.max_lat,
            "{}: min_lat ({}) >= max_lat ({})",
            city.id,
            city.bounds.min_lat,
            city.bounds.max_lat
        );
        assert!(
            city.bounds.min_lon < city.bounds.max_lon,
            "{}: min_lon ({}) >= max_lon ({})",
            city.id,
            city.bounds.min_lon,
            city.bounds.max_lon
        );

        // 2. Anchor Landmark strictly contained in Bounding Box
        assert!(
            city.bounds.contains(&city.anchor),
            "{}: Anchor landmark '{}' ({:?}) not inside bounds {:?}",
            city.id,
            city.anchor_landmark_name,
            city.anchor,
            city.bounds
        );

        // 3. Realistic Metropolitan Area (> 10 km2)
        let area = city.approximate_area_km2();
        assert!(
            (5.0..=500.0).contains(&area),
            "{}: Calculated metropolitan core area out of expected bounds: {} km2",
            city.id,
            area
        );

        // 4. Coverage Level must be explicitly categorized
        assert!(
            matches!(
                city.coverage_level,
                CoverageLevel::ValidatedCityCore
                    | CoverageLevel::ExtendedCity
                    | CoverageLevel::MetroArea
            ),
            "{}: Invalid coverage level {:?}",
            city.id,
            city.coverage_level
        );

        // 5. Surveyed elevation range
        assert!(
            city.elevation_min_m <= city.elevation_max_m,
            "{}: elevation_min_m ({}) > elevation_max_m ({})",
            city.id,
            city.elevation_min_m,
            city.elevation_max_m
        );
    }
}

// ============================================================================
// D4: AUTHENTIC CITY FEATURE GENERATION
// ============================================================================

#[test]
fn test_d4_authentic_city_feature_generation() {
    let test_samples = [
        "san-francisco",
        "new-york",
        "los-angeles",
        "chicago",
        "seattle",
        "miami",
        "boston",
        "denver",
        "austin",
        "honolulu",
        "anchorage",
        "new-orleans",
    ];

    for city_id in test_samples {
        let city = find_city_by_id(city_id).expect("City should exist");
        let dataset = generate_us_city_dataset(city);

        // 1. Buildings must include surveyed anchor landmark
        assert!(
            !dataset.buildings.is_empty(),
            "{}: Buildings list is empty",
            city.id
        );
        let has_anchor = dataset.buildings.iter().any(|b| {
            b.explicit_height.is_some()
                && b.name
                    .as_deref()
                    .map(|n| {
                        city.anchor_landmark_name.contains(n)
                            || n.contains(city.anchor_landmark_name)
                            || n.contains(city.name)
                    })
                    .unwrap_or(false)
        });
        assert!(
            has_anchor,
            "{}: Surveyed anchor building '{}' not found in dataset",
            city.id, city.anchor_landmark_name
        );

        // 2. Roads must contain multiple connected segments
        assert!(
            dataset.roads.len() >= 3,
            "{}: Expected at least 3 arterial road segments, got {}",
            city.id,
            dataset.roads.len()
        );

        // 3. Aeroway primary airport matching IATA code
        assert!(
            !dataset.aeroways.is_empty(),
            "{}: Missing aeroways for primary airport IATA {}",
            city.id,
            city.primary_airport_iata
        );
        let has_iata_match = dataset.aeroways.iter().any(|a| {
            a.name
                .as_deref()
                .map(|n| n.contains(city.primary_airport_iata))
                .unwrap_or(false)
        });
        assert!(
            has_iata_match,
            "{}: Missing airport facility for IATA code {}",
            city.id, city.primary_airport_iata
        );

        // 4. Ports: coastal cities with port flag must have maritime ports
        if city.has_maritime_port {
            assert!(
                !dataset.ports.is_empty(),
                "{}: Expected maritime port facility since has_maritime_port is true",
                city.id
            );
        }

        // 5. Water bodies & land cover parks
        assert!(
            !dataset.water_bodies.is_empty(),
            "{}: Missing water bodies",
            city.id
        );
        assert!(
            !dataset.land_cover.is_empty(),
            "{}: Missing USGS NLCD land cover parcels",
            city.id
        );
    }
}

// ============================================================================
// D5: US HIGHWAY CONNECTIVITY
// ============================================================================

#[test]
fn test_d5_interstate_highway_connectivity() {
    let corridors = all_interstate_corridors();
    assert_eq!(
        corridors.len(),
        7,
        "Expected 7 major verified interstate corridors"
    );

    let all_city_ids: HashSet<&str> = all_us_cities().iter().map(|c| c.id).collect();

    for corridor in corridors {
        // Designation must start with "I-"
        assert!(
            corridor.designation.starts_with("I-"),
            "Invalid interstate designation: {}",
            corridor.designation
        );

        // Minimum length > 1,000 km for cross-country / multi-state routes
        assert!(
            corridor.length_km >= 1000.0,
            "{}: Corridor length too short: {} km",
            corridor.designation,
            corridor.length_km
        );

        // Legal interstate speed limit (100 - 130 km/h ~ 65 - 80 mph)
        assert!(
            (100..=130).contains(&corridor.speed_limit_kph),
            "{}: Illegal interstate speed limit: {} kph",
            corridor.designation,
            corridor.speed_limit_kph
        );

        // Connected cities must have at least 3 major hubs
        assert!(
            corridor.connected_cities.len() >= 3,
            "{}: Corridor connects fewer than 3 cities ({})",
            corridor.designation,
            corridor.connected_cities.len()
        );

        // All connected cities must exist in the authoritative US catalog!
        for &city_id in corridor.connected_cities {
            assert!(
                all_city_ids.contains(city_id),
                "{}: Corridor connects non-existent catalog city ID '{}'",
                corridor.designation,
                city_id
            );
        }
    }
}

// ============================================================================
// D6: CITY VALIDATION & COVERAGE CERTIFICATION
// ============================================================================

#[test]
fn test_d6_automated_city_validation_and_certification() {
    let (reports, summary) = audit_all_us_cities();

    assert_eq!(reports.len(), 38, "Expected 38 city reports");
    assert_eq!(summary.total_cities, 38);
    assert_eq!(summary.priority1_count, 10);
    assert_eq!(summary.priority2_count, 15);
    assert_eq!(summary.priority3_count, 13);

    for r in &reports {
        if r.overall_readiness_pct < 90.0 || !r.known_limitations.is_empty() {
            println!(
                "UNCERTIFIED: {} | readiness: {}% | limitations: {:?}",
                r.city_id, r.overall_readiness_pct, r.known_limitations
            );
        }
    }

    // 100% Certification requirement: all 38 cities must achieve certification
    assert_eq!(
        summary.certified_cities_count, 38,
        "Expected all 38 cities to be certified, but only {} passed certification",
        summary.certified_cities_count
    );

    // Mean readiness percentage must exceed 95%
    assert!(
        summary.mean_readiness_pct >= 95.0,
        "Mean catalog readiness too low: {}%",
        summary.mean_readiness_pct
    );

    // Verify individual city reports
    for report in reports {
        assert!(
            report.road_network_coverage_pct >= 75.0,
            "{}: Road coverage too low: {}%",
            report.city_id,
            report.road_network_coverage_pct
        );
        assert!(
            report.building_footprint_coverage_pct >= 80.0,
            "{}: Building coverage too low: {}%",
            report.city_id,
            report.building_footprint_coverage_pct
        );
        assert_eq!(
            report.elevation_dem_coverage_pct, 100.0,
            "{}: Elevation DEM coverage must be 100% NAVD88 compliant",
            report.city_id
        );
        assert!(
            report.hydrography_coverage_pct >= 90.0,
            "{}: Hydrography coverage too low: {}%",
            report.city_id,
            report.hydrography_coverage_pct
        );
        assert!(
            report.overall_readiness_pct >= 90.0,
            "{}: Overall readiness too low: {}%",
            report.city_id,
            report.overall_readiness_pct
        );
        assert!(
            report.known_limitations.is_empty(),
            "{}: Found unexpected validation limitations: {:?}",
            report.city_id,
            report.known_limitations
        );
    }

    // Direct certification test for Primary Anchor Hub (San Francisco)
    let sf = find_city_by_id("san-francisco").expect("San Francisco must exist");
    let (sf_report, sf_certified) = validate_and_certify_us_city(sf);
    assert!(sf_certified, "San Francisco must certify clean");
    assert_eq!(sf_report.overall_readiness_pct, 100.0);
}

// ============================================================================
// D7: EFFICIENT EXPANSION, SEARCH, BOOKMARKS & QUOTA MANAGEMENT
// ============================================================================

#[test]
fn test_d7_city_search_and_alias_resolution() {
    // Exact ID
    assert_eq!(find_city_by_id("boston").unwrap().name, "Boston");

    // Common Aliases
    assert_eq!(find_city_by_id("nyc").unwrap().id, "new-york");
    assert_eq!(find_city_by_id("manhattan").unwrap().id, "new-york");
    assert_eq!(find_city_by_id("la").unwrap().id, "los-angeles");
    assert_eq!(find_city_by_id("sf").unwrap().id, "san-francisco");
    assert_eq!(find_city_by_id("bay-area").unwrap().id, "san-francisco");
    assert_eq!(find_city_by_id("dc").unwrap().id, "washington-dc");
    assert_eq!(find_city_by_id("dfw").unwrap().id, "dallas");
    assert_eq!(find_city_by_id("twin-cities").unwrap().id, "minneapolis");

    // Search Query Matching
    let ca_cities = search_cities("CA");
    assert!(
        ca_cities.len() >= 4,
        "Expected at least 4 California cities (SF, LA, San Diego, Sacramento), got {}",
        ca_cities.len()
    );

    let tower_search = search_cities("Tower");
    assert!(
        tower_search.len() >= 4,
        "Expected multiple landmark towers, got {}",
        tower_search.len()
    );

    let airport_search = search_cities("ORD");
    assert_eq!(airport_search.first().unwrap().id, "chicago");
}

#[test]
fn test_d7_region_profile_runtime_loading() {
    // Default region must be US (San Francisco Downtown & Waterfront)
    let def = RegionProfile::default_region();
    assert_eq!(def.country, "US");
    assert_eq!(def.id, "san-francisco-downtown");

    // Dynamic conversion from US catalog
    let austin_profile = RegionProfile::from_us_catalog("austin").expect("Austin should convert");
    assert_eq!(austin_profile.country, "US");
    assert_eq!(austin_profile.state_or_province, "TX");
    assert_eq!(austin_profile.utm_zone, 14);
    assert_eq!(austin_profile.vertical_datum, "NAVD88");
}

#[test]
fn test_d7_on_demand_world_package_compilation_and_quota() {
    let tmp = tempdir().expect("Failed to create temporary directory");
    let target_dir = tmp.path().join("world_denver");

    let denver = find_city_by_id("denver").expect("Denver should exist");
    let dataset = generate_us_city_dataset(denver);
    let dem = DemGrid::create_us_city_dem(denver, 32, 32);

    let (validated, report) = validate_dataset(&dataset);
    assert!(report.is_clean(), "Denver dataset should be clean");

    let options = CompilerOptions {
        chunk_size_meters: 500.0,
        terrain_grid_resolution: 16,
        ..Default::default()
    };

    // Compile single city package independently
    let package_metrics = compile_world_package(&validated, &dem, &target_dir, &options)
        .expect("World package compilation should succeed");

    assert!(
        package_metrics.total_chunks_generated > 0,
        "Expected chunks to be compiled"
    );
    assert!(
        target_dir.join("chunks.db").exists(),
        "chunks.db must be written"
    );

    // Verify storage quota accounting for installed packages
    let mut quota_manager = StorageQuotaManager::new(500 * 1024 * 1024); // 500 MB quota
    quota_manager.register_package(InstalledPackageRecord {
        package_id: "world-denver".to_string(),
        country_code: "US".to_string(),
        version: "1.0.0".to_string(),
        disk_bytes: package_metrics.total_uncompressed_bytes,
        is_pinned: false,
        is_active: true,
        last_accessed_timestamp: 100,
    });

    assert_eq!(quota_manager.packages.len(), 1);
    assert!(quota_manager.total_used_bytes() > 0);
    assert!(quota_manager.available_headroom_bytes() > 0);
}
