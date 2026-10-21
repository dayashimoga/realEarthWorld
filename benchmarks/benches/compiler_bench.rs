use criterion::{black_box, criterion_group, criterion_main, Criterion};
use realearth_compiler::building_mesher::mesh_building;
use realearth_compiler::chunk::{ChunkHeader, WorldChunk, CHUNK_MAGIC, CHUNK_VERSION};
use realearth_compiler::mesh::ChunkMesh;
use realearth_compiler::packager::{pack_chunk_to_bytes, unpack_chunk_from_bytes};
use realearth_compiler::road_mesher::mesh_road;
use realearth_geospatial::transforms::EnuConverter;
use realearth_ingestion::dem_grid::DemGrid;
use realearth_ingestion::osm_types::{BuildingType, HighwayType, OsmRoadFeature};
use realearth_shared::bounds::GeoBounds;
use realearth_shared::coordinates::Wgs84Coord;
use realearth_validation::polygon_validator::ValidatedBuilding;
use realearth_validation::provenance::{HeightProvenance, ValidatedHeight};

fn bench_compiler_operations(c: &mut Criterion) {
    let anchor = Wgs84Coord::new(12.9797, 77.5907, 920.0);
    let enu_conv = EnuConverter::new(anchor);
    let dem = DemGrid::create_bengaluru_plateau_dem(32, 32);

    // Sample building: Vidhana Soudha outline
    let building = ValidatedBuilding {
        id: 100001,
        name: Some("Vidhana Soudha".to_string()),
        building_type: BuildingType::Civic,
        outer_ring: vec![
            Wgs84Coord::new(12.9790, 77.5900, 920.0),
            Wgs84Coord::new(12.9805, 77.5900, 920.0),
            Wgs84Coord::new(12.9805, 77.5920, 920.0),
            Wgs84Coord::new(12.9790, 77.5920, 920.0),
            Wgs84Coord::new(12.9790, 77.5900, 920.0),
        ],
        inner_rings: Vec::new(),
        height: ValidatedHeight {
            height_meters: 22.0,
            provenance: HeightProvenance::Surveyed,
            confidence: 1.0,
        },
        area_m2: 36000.0,
    };

    // Sample road: Cubbon Park Flyover
    let road = OsmRoadFeature {
        way_id: 200001,
        name: Some("Cubbon Park Flyover".to_string()),
        highway_type: HighwayType::Primary,
        geometry: vec![
            Wgs84Coord::new(12.9750, 77.5910, 920.0),
            Wgs84Coord::new(12.9770, 77.5912, 920.0),
            Wgs84Coord::new(12.9790, 77.5914, 920.0),
            Wgs84Coord::new(12.9810, 77.5916, 920.0),
        ],
        is_oneway: false,
        lanes: 4,
        speed_limit_kph: 60,
        is_bridge: true,
        is_tunnel: false,
        layer: 1,
    };

    let mut group = c.benchmark_group("compiler_meshing_and_compression");

    group.bench_function("generate_building_mesh_3d", |b| {
        b.iter(|| {
            let _ = mesh_building(black_box(&building), black_box(&enu_conv), black_box(&dem));
        });
    });

    group.bench_function("generate_road_mesh_ribbon", |b| {
        b.iter(|| {
            let _ = mesh_road(black_box(&road), black_box(&enu_conv), black_box(&dem));
        });
    });

    // Chunk compression benchmark
    let chunk = WorldChunk {
        header: ChunkHeader {
            magic: *CHUNK_MAGIC,
            version: CHUNK_VERSION,
            chunk_x: 0,
            chunk_y: 0,
            lod_level: 0,
            bounds: GeoBounds::new(12.9700, 12.9800, 77.5800, 77.5900),
            anchor,
            min_elevation: 900.0,
            max_elevation: 935.0,
        },
        terrain: ChunkMesh::default(),
        roads: ChunkMesh::default(),
        buildings: ChunkMesh::default(),
        building_metadata: Vec::new(),
    };
    let packed = pack_chunk_to_bytes(&chunk).expect("pack failed");

    group.bench_function("pack_chunk_zstd", |b| {
        b.iter(|| {
            let _ = pack_chunk_to_bytes(black_box(&chunk));
        });
    });

    group.bench_function("unpack_chunk_zstd", |b| {
        b.iter(|| {
            let _ = unpack_chunk_from_bytes(black_box(&packed));
        });
    });

    group.finish();
}

criterion_group!(benches, bench_compiler_operations);
criterion_main!(benches);
