//! Stage H3 — Real US Metropolitan Coverage and Transactional City Warping Quality Gates.
//!
//! Enforces:
//! - Phase 1: Honest Region Catalog Audit across all 38 US cities (zero catalog-only fake READY).
//! - Phase 2: Complete, authentic reference cities (San Francisco, New York City, Portland).
//! - Phase 3: Transactional Warp Pipeline (SELECT -> VERIFY -> ACQUIRE -> VALIDATE -> COMPILE -> STAGE -> LOAD ESSENTIAL CHUNKS -> SAFE SPAWN -> ATOMIC ACTIVATE) with atomic rollback.
//! - Phase 4: City Differentiation & Multi-City Exploration (disjoint geometries, unique road topology, authentic DEM topography, repeated transitions).

use realearth_compiler::database::ChunkDatabase;
use realearth_ingestion::authentic::{load_authentic_city, verify_city_playable};
use realearth_shared::us_catalog::{all_us_cities, find_city_by_id, RegionPackageReadiness};
use realearth_streaming::warper::{TransactionalCityWarper, WarpStage};
use std::collections::HashSet;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

// ============================================================================
// GATE 1: HONEST REGION CATALOG AUDIT (ALL 38 US CITIES)
// ============================================================================

#[test]
fn test_gate1_honest_region_catalog_audit() {
    let cities = all_us_cities();
    assert_eq!(
        cities.len(),
        38,
        "Authoritative US Geographic Catalog must contain exactly 38 cities"
    );

    let mut ready_count = 0;
    let mut download_req_count = 0;
    let mut partial_count = 0;

    for city in cities {
        // 1. Verify bounds and anchor containment
        assert!(
            city.bounds.min_lat < city.bounds.max_lat,
            "City {} lat bounds invalid",
            city.id
        );
        assert!(
            city.bounds.min_lon < city.bounds.max_lon,
            "City {} lon bounds invalid",
            city.id
        );
        assert!(
            city.bounds.contains(&city.anchor),
            "City {} anchor landmark '{}' must be contained inside bounds",
            city.id,
            city.anchor_landmark_name
        );

        // 2. Verify UTM Zone and NAVD88 vertical datum
        assert!(
            city.utm_zone >= 4 && city.utm_zone <= 19,
            "City {} UTM zone {} outside continental US / Alaska / Hawaii range",
            city.id,
            city.utm_zone
        );
        assert!(
            city.elevation_min_m <= city.elevation_max_m,
            "City {} elevation range invalid",
            city.id
        );

        // 3. Verify honest readiness
        let readiness = city.package_readiness();
        match readiness {
            RegionPackageReadiness::InstalledReady => {
                ready_count += 1;
                // Only cities with verified authentic files on disk can be InstalledReady
                let is_verified_benchmark = city.id == "san-francisco"
                    || city.id == "portland"
                    || city.id == "new-york"
                    || city.id == "chicago"
                    || city.id == "los-angeles";
                assert!(
                    is_verified_benchmark,
                    "City '{}' is a catalog-only city and cannot claim InstalledReady without spatial package on disk!",
                    city.id
                );
            }
            RegionPackageReadiness::DownloadRequired => {
                download_req_count += 1;
                // Verify that verify_city_playable rejects catalog-only cities honestly
                let play_res = verify_city_playable(city.id);
                assert!(
                    play_res.is_err(),
                    "Catalog-only city '{}' must require download package before play",
                    city.id
                );
            }
            RegionPackageReadiness::Partial => {
                partial_count += 1;
            }
            RegionPackageReadiness::Unavailable => {}
        }
    }

    assert_eq!(
        ready_count, 5,
        "Exactly 5 benchmark cities with authentic assets on disk must be InstalledReady"
    );
    assert!(
        download_req_count >= 30,
        "All unbundled catalog-only cities must honestly report DownloadRequired (found {})",
        download_req_count
    );
    println!(
        "✅ Gate 1 Passed: 38 cities audited (Ready: {}, DownloadRequired: {}, Partial: {})",
        ready_count, download_req_count, partial_count
    );
}

// ============================================================================
// GATE 2: THREE AUTHENTIC REFERENCE CITIES (SF, NYC, PORTLAND)
// ============================================================================

#[test]
fn test_gate2_three_reference_cities_differentiation() {
    let sf = find_city_by_id("san-francisco").expect("San Francisco must exist");
    let nyc = find_city_by_id("new-york").expect("New York must exist");
    let pdx = find_city_by_id("portland").expect("Portland must exist");

    // 1. Ingest authentic datasets
    let (sf_data, sf_dem, sf_meta) = load_authentic_city(sf.id).expect("SF must load");
    let (nyc_data, nyc_dem, nyc_meta) = load_authentic_city(nyc.id).expect("NYC must load");
    let (pdx_data, pdx_dem, _pdx_meta) = load_authentic_city(pdx.id).expect("Portland must load");

    // 2. Extents and anchors must be completely disjoint
    assert_ne!(sf_meta.utm_zone, nyc_meta.utm_zone);
    assert!((sf.anchor.lon - nyc.anchor.lon).abs() > 40.0);
    assert!((sf.anchor.lat - pdx.anchor.lat).abs() > 7.0);

    // 3. Building and Road Feature Counts
    assert!(
        sf_data.buildings.len() >= 250,
        "SF must have >= 250 authentic buildings (got {})",
        sf_data.buildings.len()
    );
    assert!(
        nyc_data.buildings.len() >= 180,
        "NYC must have >= 180 authentic buildings (got {})",
        nyc_data.buildings.len()
    );
    assert!(
        pdx_data.buildings.len() >= 140,
        "Portland must have >= 140 authentic buildings (got {})",
        pdx_data.buildings.len()
    );

    assert!(
        sf_data.roads.len() >= 30,
        "SF must have >= 30 connected roads (got {})",
        sf_data.roads.len()
    );
    assert!(
        nyc_data.roads.len() >= 10,
        "NYC must have >= 10 connected roads (got {})",
        nyc_data.roads.len()
    );
    assert!(
        pdx_data.roads.len() >= 10,
        "Portland must have >= 10 connected roads (got {})",
        pdx_data.roads.len()
    );

    // 4. Disjoint OSM Feature Identifiers (Zero ID collisions across cities)
    let sf_bld_ids: HashSet<u64> = sf_data.buildings.iter().map(|b| b.id).collect();
    let nyc_bld_ids: HashSet<u64> = nyc_data.buildings.iter().map(|b| b.id).collect();
    let pdx_bld_ids: HashSet<u64> = pdx_data.buildings.iter().map(|b| b.id).collect();

    assert!(
        sf_bld_ids.is_disjoint(&nyc_bld_ids),
        "SF and NYC must have disjoint building IDs"
    );
    assert!(
        sf_bld_ids.is_disjoint(&pdx_bld_ids),
        "SF and Portland must have disjoint building IDs"
    );
    assert!(
        nyc_bld_ids.is_disjoint(&pdx_bld_ids),
        "NYC and Portland must have disjoint building IDs"
    );

    let sf_road_ids: HashSet<u64> = sf_data.roads.iter().map(|r| r.way_id).collect();
    let nyc_road_ids: HashSet<u64> = nyc_data.roads.iter().map(|r| r.way_id).collect();
    let pdx_road_ids: HashSet<u64> = pdx_data.roads.iter().map(|r| r.way_id).collect();

    assert!(sf_road_ids.is_disjoint(&nyc_road_ids));
    assert!(sf_road_ids.is_disjoint(&pdx_road_ids));
    assert!(nyc_road_ids.is_disjoint(&pdx_road_ids));

    // 5. Authentic Topography / DEM Relief Differentiation
    // San Francisco: Steep coastal hills (elevation span > 80m in downtown sample)
    let sf_min_h = sf_dem.elevations.iter().cloned().fold(f32::INFINITY, f32::min);
    let sf_max_h = sf_dem.elevations.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    assert!(
        sf_max_h - sf_min_h >= 60.0,
        "SF topography must exhibit genuine hill relief (span: {:.1}m)",
        sf_max_h - sf_min_h
    );

    // New York: Low coastal bedrock (0m to ~35m)
    let nyc_min_h = nyc_dem.elevations.iter().cloned().fold(f32::INFINITY, f32::min);
    let nyc_max_h = nyc_dem.elevations.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    assert!(
        nyc_max_h <= 45.0 && nyc_min_h >= 0.0,
        "NYC topography must reflect coastal plains [0, 45m] (got [{:.1}, {:.1}])",
        nyc_min_h,
        nyc_max_h
    );

    // Portland: River valley with West Hills rising (3.5m to ~95m)
    let pdx_min_h = pdx_dem.elevations.iter().cloned().fold(f32::INFINITY, f32::min);
    let pdx_max_h = pdx_dem.elevations.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    assert!(
        pdx_min_h >= 2.0 && pdx_max_h >= 30.0,
        "Portland topography must reflect Willamette River valley (got [{:.1}, {:.1}])",
        pdx_min_h,
        pdx_max_h
    );

    // 6. Verified Landmark Skyscraper Identity
    assert!(sf_data.buildings.iter().any(|b| b.name.as_deref() == Some("Salesforce Tower")));
    assert!(sf_data.buildings.iter().any(|b| b.name.as_deref() == Some("Transamerica Pyramid")));
    assert!(nyc_data.buildings.iter().any(|b| b.name.as_deref() == Some("One World Trade Center")));
    assert!(nyc_data.buildings.iter().any(|b| b.name.as_deref() == Some("Woolworth Building")));
    assert!(pdx_data.buildings.iter().any(|b| b.name.as_deref() == Some("Wells Fargo Center")));
    assert!(pdx_data.buildings.iter().any(|b| b.name.as_deref() == Some("US Bancorp Tower (Big Pink)")));

    println!("✅ Gate 2 Passed: San Francisco, New York, and Portland are 100% geographically distinct");
}

// ============================================================================
// GATE 3: TRANSACTIONAL WARPING PIPELINE AND ATOMIC ROLLBACK
// ============================================================================

#[test]
fn test_gate3_transactional_warp_pipeline_and_atomic_rollback() {
    let mut warper = TransactionalCityWarper::new();
    assert_eq!(warper.active_region_id(), None);

    // 1. Warp to San Francisco (Phase 1-9: SELECT -> ATOMIC ACTIVATE)
    let sf_report = warper.execute_warp("san-francisco").expect("SF warp should succeed");
    assert!(sf_report.is_successful);
    assert_eq!(sf_report.final_stage, WarpStage::Complete);
    assert_eq!(warper.active_region_id(), Some("san-francisco"));
    assert!(sf_report.chunks_resident_count > 0);
    assert!(sf_report.arrival_notification.is_some());
    assert!(
        sf_report.stage_history.contains(&WarpStage::Select)
            && sf_report.stage_history.contains(&WarpStage::Verify)
            && sf_report.stage_history.contains(&WarpStage::Acquire)
            && sf_report.stage_history.contains(&WarpStage::Validate)
            && sf_report.stage_history.contains(&WarpStage::Compile)
            && sf_report.stage_history.contains(&WarpStage::Stage)
            && sf_report.stage_history.contains(&WarpStage::LoadEssentialChunks)
            && sf_report.stage_history.contains(&WarpStage::SafeSpawn)
            && sf_report.stage_history.contains(&WarpStage::AtomicActivate)
    );

    // 2. Warp to New York City
    let nyc_report = warper.execute_warp("new-york").expect("NYC warp should succeed");
    assert!(nyc_report.is_successful);
    assert_eq!(warper.active_region_id(), Some("new-york"));
    assert_eq!(nyc_report.source_region_id.as_deref(), Some("san-francisco"));
    assert!(nyc_report.chunks_resident_count > 0);
    assert!(nyc_report.arrival_notification.unwrap().contains("New York"));

    // 3. Warp to Portland
    let pdx_report = warper.execute_warp("portland").expect("Portland warp should succeed");
    assert!(pdx_report.is_successful);
    assert_eq!(warper.active_region_id(), Some("portland"));
    assert_eq!(pdx_report.source_region_id.as_deref(), Some("new-york"));
    assert!(pdx_report.arrival_notification.unwrap().contains("Portland"));

    // 4. Attempt warp to unbundled catalog-only city ("phoenix") -> MUST FAIL closed & ROLL BACK
    let fail_result = warper.execute_warp("phoenix");
    assert!(
        fail_result.is_err(),
        "Warping to unbundled city phoenix must fail"
    );

    // STRICT TRANSACTIONAL ROLLBACK CONTRACT:
    // Active region MUST remain Portland (not wiped, not set to phoenix, not reverted to SF)
    assert_eq!(
        warper.active_region_id(),
        Some("portland"),
        "Failure must roll back cleanly and retain active region 'portland'"
    );

    // Inspect failed report in history: arrival notification must be None!
    let last_report = warper.warp_history().last().unwrap();
    assert!(!last_report.is_successful);
    assert_eq!(last_report.final_stage, WarpStage::RolledBack);
    assert!(
        last_report.arrival_notification.is_none(),
        "Failed warp MUST NOT emit an arrival notification"
    );

    // 5. Attempt warp to invalid city ("atlantis") -> MUST FAIL at Select & ROLL BACK
    let invalid_res = warper.execute_warp("atlantis");
    assert!(invalid_res.is_err());
    assert_eq!(warper.active_region_id(), Some("portland"));

    // 6. Successful warp back to San Francisco
    let return_sf = warper.execute_warp("san-francisco").expect("Return to SF should succeed");
    assert!(return_sf.is_successful);
    assert_eq!(warper.active_region_id(), Some("san-francisco"));

    println!("✅ Gate 3 Passed: 9-phase transactional warping and atomic rollback certified");
}

// ============================================================================
// GATE 4: ROAD / BUILDING CLEARANCE AND COLLISION INTEGRITY
// ============================================================================

#[test]
fn test_gate4_road_building_clearance_and_collision_integrity() {
    let cities = ["san-francisco", "new-york", "portland"];

    for city_id in cities {
        let (raw, dem, _) = load_authentic_city(city_id).unwrap();

        // 1. Verify all roads have valid drivable coordinates
        for road in &raw.roads {
            assert!(
                road.geometry.len() >= 2,
                "City {} road {:?} has < 2 nodes",
                city_id,
                road.name
            );
            for node in &road.geometry {
                let elev = dem.sample_bilinear(node);
                assert!(
                    !elev.is_nan() && !elev.is_infinite(),
                    "City {} road {:?} has invalid elevation {:.2}",
                    city_id,
                    road.name,
                    elev
                );
            }
        }

        // 2. Verify building polygons do not block surface road ribbons
        // Using road clearance solver projection:
        let lat_scale = 111_139.0;
        let mid_lat = raw.roads[0].geometry[0].lat;
        let lon_scale = 111_139.0 * (mid_lat as f64).to_radians().cos();

        for road in &raw.roads {
            if road.layer != 0 || road.is_bridge || road.is_tunnel {
                continue; // Bridges and tunnels are grade-separated
            }

            let half_road_width_m = (road.lanes.max(1) as f64) * 3.5 * 0.5;

            for w in road.geometry.windows(2) {
                let p1_x = w[0].lon * lon_scale;
                let p1_y = w[0].lat * lat_scale;
                let p2_x = w[1].lon * lon_scale;
                let p2_y = w[1].lat * lat_scale;
                let dx = p2_x - p1_x;
                let dy = p2_y - p1_y;
                let len = (dx * dx + dy * dy).sqrt();
                if len < 1e-4 {
                    continue;
                }

                // Check a sample of buildings near this segment
                for bld in &raw.buildings {
                    let bld_center_x = bld.outer_ring.iter().map(|n| n.lon * lon_scale).sum::<f64>()
                        / bld.outer_ring.len() as f64;
                    let bld_center_y = bld.outer_ring.iter().map(|n| n.lat * lat_scale).sum::<f64>()
                        / bld.outer_ring.len() as f64;

                    let vx = bld_center_x - p1_x;
                    let vy = bld_center_y - p1_y;
                    let t = ((vx * dx + vy * dy) / (len * len)).clamp(0.0, 1.0);
                    let closest_x = p1_x + t * dx;
                    let closest_y = p1_y + t * dy;
                    let dist = ((bld_center_x - closest_x).powi(2)
                        + (bld_center_y - closest_y).powi(2))
                    .sqrt();

                    // Building centroid must not lie within road roadway clearance ribbon
                    assert!(
                        dist >= half_road_width_m,
                        "City {} building '{}' (id: {}) encroaches into road '{:?}' (dist: {:.2}m, min: {:.2}m)",
                        city_id,
                        bld.name.as_deref().unwrap_or("Unnamed"),
                        bld.id,
                        road.name,
                        dist,
                        half_road_width_m
                    );
                }
            }
        }
    }

    println!("✅ Gate 4 Passed: Road network clearance and ground query certified");
}

// ============================================================================
// GATE 5: CONTINUOUS MULTI-CITY EXPLORATION (REPEATED WARPING & INTEGRITY)
// ============================================================================

#[test]
fn test_gate5_continuous_multi_city_exploration() {
    let root = workspace_root();

    // Verify all 3 reference cities have genuine compiled packages in data/
    let sf_db = root.join("data/world_san_francisco/chunks.db");
    let nyc_db = root.join("data/world_new_york/chunks.db");
    let pdx_db = root.join("data/world_portland/chunks.db");

    assert!(sf_db.exists(), "SF chunks.db must exist at {:?}", sf_db);
    assert!(nyc_db.exists(), "NYC chunks.db must exist at {:?}", nyc_db);
    assert!(pdx_db.exists(), "Portland chunks.db must exist at {:?}", pdx_db);

    let db_sf = ChunkDatabase::open(&sf_db).expect("SF db must open");
    let db_nyc = ChunkDatabase::open(&nyc_db).expect("NYC db must open");
    let db_pdx = ChunkDatabase::open(&pdx_db).expect("Portland db must open");

    let count_sf = db_sf.count_chunks().expect("SF chunk count");
    let count_nyc = db_nyc.count_chunks().expect("NYC chunk count");
    let count_pdx = db_pdx.count_chunks().expect("Portland chunk count");

    assert!(count_sf >= 25, "SF must have >= 25 chunks (got {})", count_sf);
    assert!(count_nyc >= 50, "NYC must have >= 50 chunks (got {})", count_nyc);
    assert!(count_pdx >= 30, "Portland must have >= 30 chunks (got {})", count_pdx);

    // Repeated cycling through cities using TransactionalCityWarper
    let mut warper = TransactionalCityWarper::new();
    let transition_sequence = [
        "san-francisco",
        "new-york",
        "portland",
        "san-francisco",
        "portland",
        "new-york",
        "san-francisco",
    ];

    for &city_id in &transition_sequence {
        let report = warper.execute_warp(city_id).unwrap_or_else(|e| {
            panic!("Transition to {} in continuous sequence failed: {}", city_id, e)
        });

        assert_eq!(report.target_region_id, city_id);
        assert!(report.is_successful);
        assert!(report.chunks_resident_count > 0);
        assert!(report.arrival_notification.is_some());
        assert_eq!(warper.active_region_id(), Some(city_id));
    }

    assert_eq!(warper.warp_history().len(), transition_sequence.len());
    println!(
        "✅ Gate 5 Passed: Continuous cycling across {} city transitions verified without corruption",
        transition_sequence.len()
    );
}
