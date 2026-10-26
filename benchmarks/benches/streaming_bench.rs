//! Criterion microbenchmarks for spatial streaming, prefetch bounds, and LOD hysteresis.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use realearth_compiler::chunk::{ChunkHeader, WorldChunk, CHUNK_MAGIC, CHUNK_VERSION};
use realearth_compiler::mesh::ChunkMesh;
use realearth_shared::bounds::GeoBounds;
use realearth_shared::coordinates::Wgs84Coord;
use realearth_streaming::{
    calculate_prefetch_bounds, select_lod_with_hysteresis, ChunkKey, LruChunkCache, StreamingConfig,
};

fn make_benchmark_chunk(cx: i32, cy: i32) -> WorldChunk {
    WorldChunk {
        header: ChunkHeader {
            magic: *CHUNK_MAGIC,
            version: CHUNK_VERSION,
            chunk_x: cx,
            chunk_y: cy,
            lod_level: 0,
            bounds: GeoBounds::new(37.79, 37.80, -122.41, -122.40),
            anchor: Wgs84Coord::new(37.795, -122.405, 10.0),
            min_elevation: 0.0,
            max_elevation: 50.0,
        },
        terrain: ChunkMesh::default(),
        roads: ChunkMesh::default(),
        buildings: ChunkMesh::default(),
        water: ChunkMesh::default(),
        vegetation: ChunkMesh::default(),
        building_metadata: Vec::new(),
        road_metadata: Vec::new(),
    }
}

fn bench_prefetch_bounds_calculation(c: &mut Criterion) {
    let config = StreamingConfig::default();
    let pos = Wgs84Coord::new(37.7952, -122.3937, 20.0);
    let velocity = glam::Vec3::new(30.0, 0.0, 30.0);

    c.bench_function("prefetch_bounds_calculation", |b| {
        b.iter(|| {
            black_box(calculate_prefetch_bounds(
                black_box(pos),
                black_box(velocity),
                black_box(&config),
            ))
        });
    });
}

fn bench_lod_hysteresis_selection(c: &mut Criterion) {
    let config = StreamingConfig::default();

    c.bench_function("lod_hysteresis_selection", |b| {
        b.iter(|| {
            let _ = black_box(select_lod_with_hysteresis(
                black_box(1520.0),
                black_box(Some(0)),
                black_box(&config),
            ));
        });
    });
}

fn bench_lru_cache_operations(c: &mut Criterion) {
    let mut cache = LruChunkCache::new();
    for i in 0..64 {
        let key = ChunkKey::new(i, 0, 0);
        let chunk = make_benchmark_chunk(i, 0);
        cache.insert(key, chunk, i as u64);
    }

    c.bench_function("lru_cache_lookup_and_budget_evict", |b| {
        b.iter(|| {
            let key = ChunkKey::new(10, 0, 0);
            let _ = black_box(cache.get(&key));
            let _ = black_box(cache.evict_to_budget(black_box(500_000), 10, 0));
        });
    });
}

criterion_group!(
    benches,
    bench_prefetch_bounds_calculation,
    bench_lod_hysteresis_selection,
    bench_lru_cache_operations
);
criterion_main!(benches);
