//! STAGE 2 CERTIFICATION SUITE: Authentic GIS Pipeline & Cache Invalidation
//!
//! Validates:
//! 1. High-density authentic San Francisco GIS dataset (>= 50 surveyed buildings, >= 25 roads, waterfront, parks).
//! 2. Source-asset SHA-256 cache invalidation in ChunkDatabase and world package resolution.
//! 3. Honest US Catalog tier separation (Tier 1 Benchmark vs Tier 2 Metropolitan Definition) with limitation disclosure.
//! 4. Topological validation cleanliness and authoritative chunk compilation integrity.

use realearth_compiler::database::ChunkDatabase;
use realearth_ingestion::authentic::load_authentic_city;
use realearth_shared::us_catalog::{
    all_us_cities, cities_by_data_tier, tier1_benchmark_cities, tier2_definition_cities,
    CatalogDataTier,
};
use realearth_validation::validate_dataset;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Workspace root must exist")
        .to_path_buf()
}

// ============================================================================
// 2.1: HIGH-DENSITY AUTHENTIC SAN FRANCISCO GIS INGESTION & TOPOLOGY
// ============================================================================

#[test]
fn test_stage2_san_francisco_high_density_gis_ingestion() {
    let (dataset, dem, meta) =
        load_authentic_city("san-francisco").expect("Authentic San Francisco must load cleanly");

    // Check building density
    assert!(
        dataset.buildings.len() >= 50,
        "Authentic SF dataset must contain >= 50 surveyed buildings, found {}",
        dataset.buildings.len()
    );
    assert!(
        dataset.buildings.len() >= 80,
        "Authentic SF dataset must contain >= 80 surveyed buildings, found {}",
        dataset.buildings.len()
    );

    // Verify key landmark skyscrapers with exact surveyed heights
    let salesforce = dataset
        .buildings
        .iter()
        .find(|b| b.name.as_deref() == Some("Salesforce Tower"))
        .expect("Salesforce Tower must be present");
    assert_eq!(salesforce.explicit_height, Some(326.0));
    assert_eq!(salesforce.levels, Some(61));

    let transamerica = dataset
        .buildings
        .iter()
        .find(|b| b.name.as_deref() == Some("Transamerica Pyramid"))
        .expect("Transamerica Pyramid must be present");
    assert_eq!(transamerica.explicit_height, Some(260.0));
    assert_eq!(transamerica.levels, Some(48));

    let fremont181 = dataset
        .buildings
        .iter()
        .find(|b| b.name.as_deref() == Some("181 Fremont"))
        .expect("181 Fremont must be present");
    assert_eq!(fremont181.explicit_height, Some(245.0));

    let california555 = dataset
        .buildings
        .iter()
        .find(|b| b.name.as_deref() == Some("555 California Street"))
        .expect("555 California Street must be present");
    assert_eq!(california555.explicit_height, Some(237.0));

    let ferry_building = dataset
        .buildings
        .iter()
        .find(|b| b.name.as_deref() == Some("San Francisco Ferry Building"))
        .expect("Ferry Building must be present");
    assert_eq!(ferry_building.explicit_height, Some(75.0));

    let coit_tower = dataset
        .buildings
        .iter()
        .find(|b| b.name.as_deref() == Some("Coit Tower"))
        .expect("Coit Tower must be present");
    assert_eq!(coit_tower.explicit_height, Some(64.0));

    // Verify road network density
    assert!(
        dataset.roads.len() >= 25,
        "Authentic SF dataset must contain >= 25 roads, found {}",
        dataset.roads.len()
    );
    assert!(
        dataset.roads.len() >= 33,
        "Authentic SF dataset must contain >= 33 interconnected roads, found {}",
        dataset.roads.len()
    );

    // Verify grade-separated infrastructure
    let bay_bridge = dataset
        .roads
        .iter()
        .find(|r| r.name.as_deref() == Some("Interstate 80 Bay Bridge Approach"))
        .expect("Bay Bridge approach must be present");
    assert!(bay_bridge.is_bridge);
    assert_eq!(bay_bridge.layer, 1);
    assert_eq!(bay_bridge.lanes, 5);

    let broadway_tunnel = dataset
        .roads
        .iter()
        .find(|r| r.name.as_deref() == Some("Broadway Tunnel"))
        .expect("Broadway Tunnel must be present");
    assert!(broadway_tunnel.is_tunnel);
    assert_eq!(broadway_tunnel.layer, -1);
    assert_eq!(broadway_tunnel.lanes, 4);

    // Verify urban land cover & parks
    assert!(
        dataset.land_cover.len() >= 5,
        "Authentic SF dataset must contain >= 5 urban parks, found {}",
        dataset.land_cover.len()
    );

    // Verify coastal hydrography
    assert_eq!(dataset.water_bodies.len(), 1);
    assert!(dataset.water_bodies[0].is_coastline);

    // Verify USGS 3DEP DEM
    assert_eq!(dem.width, 64);
    assert_eq!(dem.height, 64);
    assert_eq!(dem.elevations.len(), 4096);

    // Verify SHA-256 digests in metadata
    assert_eq!(meta.source_osm_sha256.len(), 64);
    assert_eq!(meta.source_dem_sha256.len(), 64);
}

// ============================================================================
// 2.2: SOURCE-ASSET SHA-256 CACHE INVALIDATION
// ============================================================================

#[test]
fn test_stage2_source_hash_cache_invalidation() {
    let pkg_dir = workspace_root().join("data/world_san_francisco");
    let db_path = pkg_dir.join("chunks.db");
    assert!(
        db_path.exists(),
        "chunks.db must exist for San Francisco world package"
    );

    let db = ChunkDatabase::open(&db_path).expect("Failed to open chunks.db");
    let manifest = db
        .get_package_manifest("san_francisco_package")
        .expect("Query failed")
        .expect("Manifest must exist for san_francisco_package");

    // If on-disk package schema is older than current compiler CHUNK_VERSION, recompile package
    if manifest.schema_version != realearth_compiler::CHUNK_VERSION {
        println!(
            "📦 Schema upgrade detected (v{} -> v{}): Recompiling San Francisco world package...",
            manifest.schema_version,
            realearth_compiler::CHUNK_VERSION
        );
        let (raw, dem, meta) = load_authentic_city("san-francisco").expect("Failed to load SF");
        let (val, _) = validate_dataset(&raw);
        let options = realearth_compiler::CompilerOptions {
            chunk_size_meters: 500.0,
            terrain_grid_resolution: 16,
            package_name: "san_francisco_package".to_string(),
            source_asset_sha256: Some(meta.source_osm_sha256),
            ..Default::default()
        };
        realearth_compiler::compile_world_package(&val, &dem, &pkg_dir, &options)
            .expect("Recompilation failed");
    }

    let db = ChunkDatabase::open(&db_path).expect("Failed to open chunks.db");
    let manifest = db
        .get_package_manifest("san_francisco_package")
        .expect("Query failed")
        .expect("Manifest must exist for san_francisco_package");

    let source_hash = manifest
        .source_asset_sha256
        .clone()
        .expect("source_asset_sha256 must be populated in package_manifest table");
    assert_eq!(
        source_hash.len(),
        64,
        "source_asset_sha256 must be a 64-character SHA-256 hex string"
    );
    assert_eq!(
        manifest.schema_version,
        realearth_compiler::CHUNK_VERSION,
        "Manifest schema version must match current CHUNK_VERSION"
    );

    // Verify matching hash validates
    assert!(
        db.is_package_source_hash_matching("san_francisco_package", &source_hash),
        "Identical hash must match package manifest"
    );

    // Verify mismatched/tampered hash triggers invalidation
    let tampered_hash = "0000000000000000000000000000000000000000000000000000000000000000";
    assert!(
        !db.is_package_source_hash_matching("san_francisco_package", tampered_hash),
        "Tampered hash must invalidate cache"
    );

    // Verify non-existent package name returns false
    assert!(
        !db.is_package_source_hash_matching("non_existent_package", &source_hash),
        "Non-existent package must return false"
    );
}

// ============================================================================
// 2.3: HONEST US CATALOG TIERS & LIMITATION DISCLOSURES
// ============================================================================

#[test]
fn test_stage2_honest_us_catalog_tiers() {
    let all_cities = all_us_cities();
    assert_eq!(all_cities.len(), 38, "Total catalog must have 38 cities");

    let tier1 = tier1_benchmark_cities();
    assert_eq!(
        tier1.len(),
        1,
        "Exactly 1 benchmark surveyed city must exist in Tier 1"
    );
    assert_eq!(tier1[0].id, "san-francisco");
    assert_eq!(tier1[0].data_tier, CatalogDataTier::Tier1BenchmarkSurveyed);
    assert!(tier1[0].is_benchmark_surveyed());
    assert!(!tier1[0].is_metropolitan_definition_only());

    let tier2 = tier2_definition_cities();
    assert_eq!(
        tier2.len(),
        37,
        "Exactly 37 metropolitan definitions must exist in Tier 2"
    );
    for city in &tier2 {
        assert_eq!(
            city.data_tier,
            CatalogDataTier::Tier2MetropolitanDefinition,
            "City {} must be Tier 2 Metropolitan Definition",
            city.id
        );
        assert!(!city.is_benchmark_surveyed());
        assert!(city.is_metropolitan_definition_only());
    }

    // Verify query helper
    let by_tier1 = cities_by_data_tier(CatalogDataTier::Tier1BenchmarkSurveyed);
    assert_eq!(by_tier1.len(), 1);
    let by_tier2 = cities_by_data_tier(CatalogDataTier::Tier2MetropolitanDefinition);
    assert_eq!(by_tier2.len(), 37);

    // Verify honest limitation disclosure in metadata for Tier 2 cities
    let (_dataset, _dem, meta) =
        load_authentic_city("new-york").expect("Tier 2 city loads definition metadata");
    assert!(
        meta.geographic_limitations
            .iter()
            .any(|lim| lim.contains("CATALOG_DEFINITION_ONLY")),
        "Tier 2 city metadata must disclose CATALOG_DEFINITION_ONLY limitation: {:?}",
        meta.geographic_limitations
    );

    // Verify Tier 1 San Francisco does NOT have CATALOG_DEFINITION_ONLY limitation
    let (_sf_dataset, _sf_dem, sf_meta) =
        load_authentic_city("san-francisco").expect("Tier 1 SF must load");
    assert!(
        !sf_meta
            .geographic_limitations
            .iter()
            .any(|lim| lim.contains("CATALOG_DEFINITION_ONLY")),
        "Tier 1 benchmark city must not have CATALOG_DEFINITION_ONLY limitation"
    );
}

// ============================================================================
// 2.4: TOPOLOGICAL VALIDATION CLEANLINESS & PRODUCTION COMPILATION
// ============================================================================

#[test]
fn test_stage2_topological_validation_and_clean_compilation() {
    let (dataset, _dem, _meta) =
        load_authentic_city("san-francisco").expect("San Francisco must load cleanly");

    let (validated, report) = validate_dataset(&dataset);

    // Zero quarantined features
    assert_eq!(
        report.quarantined.len(),
        0,
        "Topological validator must produce 0 quarantined features: {:?}",
        report.quarantined
    );
    assert!(report.is_clean());

    assert!(
        validated.buildings.len() >= 80,
        "Expected >= 80 buildings, found {}",
        validated.buildings.len()
    );
    assert!(
        validated.roads.len() >= 33,
        "Expected >= 33 roads, found {}",
        validated.roads.len()
    );
    assert_eq!(validated.water_bodies.len(), 1);
    assert_eq!(validated.land_cover.len(), 7);

    // Verify compiled package statistics in SQLite index
    let pkg_dir = workspace_root().join("data/world_san_francisco");
    let db = ChunkDatabase::open(&pkg_dir.join("chunks.db")).expect("Failed to open chunks.db");
    let chunk_count = db.count_chunks().expect("Failed to count chunks");
    assert!(
        chunk_count >= 20,
        "Compiled San Francisco package must contain >= 20 chunks, found {}",
        chunk_count
    );
}
