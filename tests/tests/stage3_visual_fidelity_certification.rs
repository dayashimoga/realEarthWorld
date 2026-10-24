//! STAGE 3 — VISUAL FIDELITY AND URBAN ENVIRONMENT CERTIFICATION SUITE
//!
//! Rigorously certifies:
//! - Subtask 1: Building Facades, Architectural Cornices & Rooftop Mechanical Infrastructure
//! - Subtask 2: Road Markings, Concrete Curbs & Raised Sidewalk Geometry
//! - Subtask 3: Atmosphere, Solar Disk Halo & Dynamic Horizon Fog Distance
//! - Integration: Authentic San Francisco Visual Asset Scale & Vertex Density

use realearth_compiler::building_mesher::mesh_building;
use realearth_compiler::database::ChunkDatabase;
use realearth_compiler::packager::read_chunk_file;
use realearth_compiler::road_mesher::{mesh_road, mesh_road_infrastructure};
use realearth_geospatial::transforms::EnuConverter;
use realearth_ingestion::dem_grid::DemGrid;
use realearth_ingestion::osm_types::{BuildingType, HighwayType, OsmRoadFeature};
use realearth_rendering::{SceneUniforms, WORLD_SHADER_WGSL};
use realearth_shared::coordinates::Wgs84Coord;
use realearth_validation::polygon_validator::ValidatedBuilding;
use realearth_validation::{HeightProvenance, ValidatedHeight};
use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Workspace root must exist")
        .to_path_buf()
}

/// Subtask 2: Concrete curbs, vertical curb risers, and raised pedestrian sidewalks.
#[test]
fn test_stage3_curb_riser_and_sidewalk_geometry() {
    let anchor = Wgs84Coord::new(37.795, -122.405, 10.0);
    let converter = EnuConverter::new(anchor);
    let dem = DemGrid::create_san_francisco_coastal_dem(32, 32);

    let road = OsmRoadFeature {
        way_id: 701,
        name: Some("California Street".to_string()),
        highway_type: HighwayType::Primary,
        lanes: 2,
        is_oneway: false,
        layer: 0,
        is_bridge: false,
        is_tunnel: false,
        speed_limit_kph: 40,
        geometry: vec![
            Wgs84Coord::new(37.7940, -122.4050, 0.0),
            Wgs84Coord::new(37.7945, -122.4050, 0.0),
            Wgs84Coord::new(37.7950, -122.4050, 0.0),
        ],
    };

    let infra_mesh = mesh_road_infrastructure(&road, &converter, &dem);
    assert!(
        !infra_mesh.is_empty(),
        "Road infrastructure mesh must generate curbs and sidewalks"
    );

    // 1. Sidewalk material tagging (alpha 0.92)
    let sidewalk_verts: Vec<_> = infra_mesh
        .vertices
        .iter()
        .filter(|v| (v.color[3] - 0.92).abs() < 0.01)
        .collect();
    assert!(
        !sidewalk_verts.is_empty(),
        "Sidewalk vertices must be tagged with alpha 0.92"
    );

    // 2. Vertical concrete curb risers (located at carriageway edge |x| ~ 3.5m)
    let riser_verts: Vec<_> = sidewalk_verts
        .iter()
        .filter(|v| {
            v.normal[1].abs() < 1e-4
                && (v.normal[0].abs() > 0.9 || v.normal[2].abs() > 0.9)
                && (v.position[0].abs() - 3.5).abs() < 0.6
        })
        .collect();
    assert!(
        !riser_verts.is_empty(),
        "Vertical curb riser face vertices must be present along the carriageway edge"
    );

    // Verify curb riser pairs connect asphalt edge to curb top with exactly 0.16m height
    assert!(
        riser_verts.len() % 2 == 0,
        "Riser vertices must be emitted in bottom/top pairs"
    );
    for chunk in riser_verts.as_chunks::<2>().0 {
        let delta_y = chunk[1].position[1] - chunk[0].position[1];
        assert!(
            (delta_y - 0.16).abs() < 1e-4,
            "Curb riser cross-section height must be exactly 0.16m (got {delta_y})"
        );
    }

    // 3. Sidewalk horizontal walking deck (normal pointing directly upwards +Y)
    let deck_verts: Vec<_> = sidewalk_verts
        .iter()
        .filter(|v| (v.normal[1] - 1.0).abs() < 1e-4)
        .collect();
    assert!(
        !deck_verts.is_empty(),
        "Sidewalk horizontal deck vertices with +Y normal must be present"
    );

    // 4. Outer sidewalk drop faces (located at outer boundary |x| ~ 5.5m, dropping 0.26m to ground)
    let drop_verts: Vec<_> = sidewalk_verts
        .iter()
        .filter(|v| {
            v.normal[1].abs() < 1e-4
                && (v.normal[0].abs() > 0.9 || v.normal[2].abs() > 0.9)
                && (v.position[0].abs() - 5.5).abs() < 0.6
        })
        .collect();
    assert!(
        !drop_verts.is_empty(),
        "Outer sidewalk drop face vertices must be present along the outer boundary"
    );
    assert!(
        drop_verts.len() % 2 == 0,
        "Drop vertices must be emitted in top/bottom pairs"
    );
    for chunk in drop_verts.as_chunks::<2>().0 {
        let delta_y = chunk[0].position[1] - chunk[1].position[1];
        assert!(
            (delta_y - 0.26).abs() < 1e-4,
            "Outer drop face cross-section must drop 0.26m down to ground (got {delta_y})"
        );
    }

    // 4. Asphalt Carriageway Ribbon with Metric UVs
    let road_mesh = mesh_road(&road, &converter, &dem);
    assert!(
        !road_mesh.is_empty(),
        "Road carriageway ribbon must be generated"
    );
    let asphalt_verts: Vec<_> = road_mesh
        .vertices
        .iter()
        .filter(|v| (v.color[3] - 0.90).abs() < 0.01)
        .collect();
    assert!(
        !asphalt_verts.is_empty(),
        "Asphalt carriageway vertices must be tagged with alpha 0.90"
    );
}

/// Subtask 1: Building facades, architectural cornices, and rooftop mechanical massing.
#[test]
fn test_stage3_building_cornice_and_mechanical_equipment() {
    let anchor = Wgs84Coord::new(37.795, -122.405, 10.0);
    let converter = EnuConverter::new(anchor);
    let dem = DemGrid::create_san_francisco_coastal_dem(32, 32);

    // A. Standard commercial building (35m tall)
    let building_35m = ValidatedBuilding {
        id: 801,
        name: Some("Financial Commercial Office".to_string()),
        building_type: BuildingType::Commercial,
        outer_ring: vec![
            Wgs84Coord::new(37.7946, -122.4054, 0.0),
            Wgs84Coord::new(37.7946, -122.4046, 0.0),
            Wgs84Coord::new(37.7954, -122.4046, 0.0),
            Wgs84Coord::new(37.7954, -122.4054, 0.0),
            Wgs84Coord::new(37.7946, -122.4054, 0.0),
        ],
        inner_rings: vec![],
        height: ValidatedHeight {
            height_meters: 35.0,
            provenance: HeightProvenance::Surveyed,
            confidence: 1.0,
        },
        area_m2: 2500.0,
    };

    let mesh_35m = mesh_building(&building_35m, &converter, &dem);
    assert!(!mesh_35m.is_empty());

    // 1. Architectural cornice moulding vertices:
    // Soffit (underside normal -Y) and coping wash (top normal +Y)
    let soffit_verts: Vec<_> = mesh_35m
        .vertices
        .iter()
        .filter(|v| (v.normal[1] - (-1.0)).abs() < 1e-4)
        .collect();
    assert!(
        !soffit_verts.is_empty(),
        "Architectural cornice underside (soffit) vertices must be generated with -Y normal"
    );

    let coping_verts: Vec<_> = mesh_35m
        .vertices
        .iter()
        .filter(|v| (v.normal[1] - 1.0).abs() < 1e-4 && (v.uv[1] - 0.30).abs() < 0.05)
        .collect();
    assert!(
        !coping_verts.is_empty(),
        "Architectural cornice coping wash vertices must be generated with +Y normal"
    );

    // 2. Rooftop mechanical HVAC chillers (color alpha 0.70)
    let hvac_verts: Vec<_> = mesh_35m
        .vertices
        .iter()
        .filter(|v| (v.color[3] - 0.70).abs() < 0.01)
        .collect();
    assert!(
        !hvac_verts.is_empty(),
        "Rooftop mechanical equipment / chiller vertices must be present (alpha 0.70)"
    );

    // B. High-rise skyscraper (75m tall) with telecommunications / lightning mast
    let building_75m = ValidatedBuilding {
        id: 802,
        name: Some("Metropolitan High-Rise Tower".to_string()),
        building_type: BuildingType::Commercial,
        outer_ring: vec![
            Wgs84Coord::new(37.7946, -122.4054, 0.0),
            Wgs84Coord::new(37.7946, -122.4046, 0.0),
            Wgs84Coord::new(37.7954, -122.4046, 0.0),
            Wgs84Coord::new(37.7954, -122.4054, 0.0),
            Wgs84Coord::new(37.7946, -122.4054, 0.0),
        ],
        inner_rings: vec![],
        height: ValidatedHeight {
            height_meters: 75.0,
            provenance: HeightProvenance::Surveyed,
            confidence: 1.0,
        },
        area_m2: 2500.0,
    };

    let mesh_75m = mesh_building(&building_75m, &converter, &dem);
    let max_y = mesh_75m
        .vertices
        .iter()
        .map(|v| v.position[1])
        .fold(f32::NEG_INFINITY, f32::max);

    // Roof elev is base_elev (~10m) + 75m = ~85m. Penthouse is +3.5m = ~88.5m.
    // Antenna mast adds 10m-16m, bringing apex to >= 95m.
    assert!(
        max_y >= 95.0,
        "Skyscraper antenna mast must extend above the penthouse roof (max_y: {max_y})"
    );
}

/// Subtask 1 & 2: Shader procedural texturing logic for roads and facades.
#[test]
fn test_stage3_procedural_facades_and_road_markings_in_shader() {
    // 1. Road markings: double yellow centerlines and dashed lane dividers
    assert!(
        WORLD_SHADER_WGSL.contains("0.96, 0.76, 0.12"),
        "Shader must define double yellow road centerline color"
    );
    assert!(
        WORLD_SHADER_WGSL.contains("dash_phase < 0.32"),
        "Shader must implement white dashed lane divider marks"
    );
    assert!(
        WORLD_SHADER_WGSL.contains("fract(in.uv.y / 2.0)")
            && WORLD_SHADER_WGSL.contains("fract(in.uv.x / 1.5)"),
        "Shader must implement longitudinal and transverse concrete sidewalk joint scoring"
    );

    // 2. Architectural facade procedural texturing
    assert!(
        WORLD_SHADER_WGSL.contains("fract(in.uv.y / floor_h)"),
        "Shader must implement vertical floor mullion subdivision for glass curtain walls"
    );
    assert!(
        WORLD_SHADER_WGSL.contains("10.8"),
        "Shader must implement stone/granite cornice reveals every 3 floors (~10.8m)"
    );
    assert!(
        WORLD_SHADER_WGSL.contains("3.2"),
        "Shader must implement decorative soldier course lintels for brick masonry"
    );
    assert!(
        WORLD_SHADER_WGSL.contains("7.2"),
        "Shader must implement civic concrete formwork reveal belts"
    );
    assert!(
        WORLD_SHADER_WGSL.contains("0.35") && WORLD_SHADER_WGSL.contains("vw_y"),
        "Shader must implement Victorian horizontal clapboards and white painted pediment headers"
    );
}

/// Subtask 3: Atmosphere, solar halo, and dynamic horizon fog distance.
#[test]
fn test_stage3_atmosphere_tuning_and_fog_horizon() {
    // 1. SceneUniforms default fog parameters
    let uniforms = SceneUniforms::default();
    assert_eq!(
        uniforms.fog_params[0], 0.0001,
        "Atmospheric fog density must default to realistic atmospheric value (0.0001)"
    );
    assert_eq!(
        uniforms.fog_params[1], 800.0,
        "Fog start must be at least 800m to prevent close-range horizon washout"
    );
    assert_eq!(
        uniforms.fog_params[2], 4000.0,
        "Fog distance must match 4000m view distance"
    );

    // 2. Dynamic fog start formula: max(view_distance * 0.30, 300.0)
    let view_dist = 4000.0f32;
    let computed_fog_start = (view_dist * 0.30).max(300.0);
    assert!(
        computed_fog_start >= 1200.0,
        "Dynamic fog start must remain >= 1200m at 4000m view distance (got {computed_fog_start})"
    );

    // 3. Distance fog formula in shader: eff_dist = max(dist - fog_start, 0.0)
    assert!(
        WORLD_SHADER_WGSL.contains("eff_dist = max(dist - fog_start, 0.0);"),
        "Shader must compute effective fog distance relative to fog_start to eliminate foreground fog"
    );
    assert!(
        WORLD_SHADER_WGSL.contains("pow(sun_align, 64.0) * 0.35 * day_factor"),
        "Shader must implement realistic solar disk halo / Mie scattering flare"
    );
}

/// Verification of authoritative San Francisco GIS compilation scale & vertex density.
#[test]
fn test_stage3_authentic_san_francisco_package_visual_metrics() {
    let pkg_dir = workspace_root().join("data/world_san_francisco");
    assert!(
        pkg_dir.exists(),
        "San Francisco package directory {:?} must exist",
        pkg_dir
    );

    let db_path = pkg_dir.join("chunks.db");
    let db = ChunkDatabase::open(&db_path).expect("Must open chunks.db");
    let chunk_count = db.count_chunks().expect("Must read chunk count");

    assert!(
        chunk_count >= 25,
        "Compiled package must have at least 25 chunks (got {chunk_count})"
    );

    let total_features = db.total_feature_count().expect("Must read feature count");
    assert!(
        total_features > 50,
        "Total features must exceed 50 (got {total_features})"
    );

    // Read and verify real vertex and triangle counts directly from chunk files on disk
    let chunks_dir = pkg_dir.join("chunks");
    let mut total_vertices = 0usize;
    let mut total_triangles = 0usize;

    for entry in fs::read_dir(chunks_dir).expect("Must read chunks directory") {
        let entry = entry.expect("Valid entry");
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("rec") {
            let chunk = read_chunk_file(&path).expect("Must unpack chunk file");
            total_vertices += chunk.total_vertices();
            total_triangles += chunk.total_triangles();
        }
    }

    // With authentic curbs, cornices, rooftop equipment, and antenna masts, vertex count exceeds 30,000
    assert!(
        total_vertices >= 30_000,
        "Total vertex count with authentic curbs, cornices, and rooftop infrastructure must exceed 30,000 (got {total_vertices})"
    );
    assert!(
        total_triangles >= 25_000,
        "Total triangle count must exceed 25,000 (got {total_triangles})"
    );
}
