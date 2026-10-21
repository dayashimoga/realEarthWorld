//! World Compiler and 3D Binary Chunk Packaging for Real Earth.

pub mod building_mesher;
pub mod chunk;
pub mod compiler_pipeline;
pub mod database;
pub mod mesh;
pub mod packager;
pub mod road_mesher;
pub mod terrain_mesher;

pub use building_mesher::mesh_building;
pub use chunk::{ChunkBuildingMeta, ChunkHeader, WorldChunk, CHUNK_MAGIC, CHUNK_VERSION};
pub use compiler_pipeline::{compile_world_package, CompilationStats, CompilerOptions};
pub use database::{ChunkDatabase, ChunkIndexRecord};
pub use mesh::{ChunkMesh, Vertex};
pub use packager::{
    pack_chunk_to_bytes, read_chunk_file, unpack_chunk_from_bytes, write_chunk_file,
};
pub use road_mesher::mesh_road;
pub use terrain_mesher::mesh_terrain;

#[cfg(test)]
mod tests {
    use super::*;
    use realearth_ingestion::dem_grid::DemGrid;
    use realearth_ingestion::sample_bengaluru::create_bengaluru_central_dataset;
    use realearth_shared::bounds::GeoBounds;
    use realearth_shared::coordinates::Wgs84Coord;
    use realearth_validation::pipeline::validate_dataset;
    use tempfile::tempdir;

    #[test]
    fn test_pack_unpack_chunk_roundtrip() {
        let header = ChunkHeader {
            magic: *CHUNK_MAGIC,
            version: CHUNK_VERSION,
            chunk_x: 42,
            chunk_y: 84,
            lod_level: 0,
            bounds: GeoBounds::new(12.97, 12.98, 77.59, 77.60),
            anchor: Wgs84Coord::new(12.975, 77.595, 920.0),
            min_elevation: 900.0,
            max_elevation: 950.0,
        };

        let mut terrain = ChunkMesh::default();
        terrain.vertices.push(Vertex::new(
            [0.0, 920.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0],
            [0.3, 0.5, 0.3, 1.0],
        ));
        terrain.vertices.push(Vertex::new(
            [10.0, 920.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 0.0],
            [0.3, 0.5, 0.3, 1.0],
        ));
        terrain.vertices.push(Vertex::new(
            [10.0, 920.0, 10.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0],
            [0.3, 0.5, 0.3, 1.0],
        ));
        terrain.indices.extend_from_slice(&[0, 1, 2]);

        let chunk = WorldChunk {
            header: header.clone(),
            terrain,
            roads: ChunkMesh::default(),
            buildings: ChunkMesh::default(),
            building_metadata: Vec::new(),
        };

        let packed_bytes = pack_chunk_to_bytes(&chunk).expect("Packing failed");
        assert!(!packed_bytes.is_empty());

        let unpacked = unpack_chunk_from_bytes(&packed_bytes).expect("Unpacking failed");
        assert_eq!(unpacked.header, header);
        assert_eq!(unpacked.terrain.vertices.len(), 3);
        assert_eq!(unpacked.terrain.indices.len(), 3);
    }

    #[test]
    fn test_sqlite_chunk_database_query() {
        let dir = tempdir().expect("Failed to create tempdir");
        let db_path = dir.path().join("test_chunks.db");
        let db = ChunkDatabase::open(&db_path).expect("Failed to open DB");

        let rec = ChunkIndexRecord {
            chunk_x: 1,
            chunk_y: 2,
            lod_level: 0,
            bounds: GeoBounds::new(12.970, 12.975, 77.590, 77.595),
            min_alt: 900.0,
            max_alt: 940.0,
            compressed_bytes: 4096,
            uncompressed_bytes: 16384,
            feature_count: 5,
            storage_path: "chunks/chunk_1_2.rec".to_string(),
            file_offset: 0,
        };

        db.insert_chunk(&rec).expect("Failed to insert chunk");
        assert_eq!(db.count_chunks().unwrap(), 1);

        // Query overlapping bounding box
        let query_box = GeoBounds::new(12.965, 12.980, 77.585, 77.600);
        let results = db.query_bbox(&query_box).expect("Query failed");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].chunk_x, 1);
        assert_eq!(results[0].chunk_y, 2);

        // Query disjoint bounding box
        let disjoint_box = GeoBounds::new(13.000, 13.010, 78.000, 78.010);
        let disjoint_results = db.query_bbox(&disjoint_box).expect("Disjoint query failed");
        assert_eq!(disjoint_results.len(), 0);
    }

    #[test]
    fn test_compile_bengaluru_world_package() {
        let dir = tempdir().expect("Failed to create tempdir");
        let output_dir = dir.path().join("bengaluru_world");

        let raw_dataset = create_bengaluru_central_dataset();
        let (validated, report) = validate_dataset(&raw_dataset);
        assert!(report.is_clean());

        let dem = DemGrid::create_bengaluru_plateau_dem(32, 32);

        let options = CompilerOptions {
            chunk_size_meters: 1000.0,
            terrain_grid_resolution: 8,
        };

        let stats = compile_world_package(&validated, &dem, &output_dir, &options)
            .expect("World compilation failed");

        assert!(stats.total_chunks_generated >= 4, "Expected >= 4 chunks");
        assert!(stats.total_vertices > 0, "Expected > 0 vertices");
        assert!(stats.total_triangles > 0, "Expected > 0 triangles");
        assert!(
            stats.total_compressed_bytes > 0,
            "Expected compressed bytes > 0"
        );

        // Verify SQLite index was created and contains records
        let db_path = output_dir.join("chunks.db");
        assert!(db_path.exists());
        let db = ChunkDatabase::open(&db_path).expect("Failed to read chunks.db");
        assert_eq!(db.count_chunks().unwrap(), stats.total_chunks_generated);

        // Verify first chunk file can be read and unpacked from disk
        let first_chunk_file = output_dir.join("chunks/chunk_0_0.rec");
        assert!(first_chunk_file.exists());
        let chunk = read_chunk_file(&first_chunk_file).expect("Failed to read chunk from disk");
        assert_eq!(&chunk.header.magic, CHUNK_MAGIC);
    }

    #[test]
    fn test_compile_san_francisco_world_package() {
        use realearth_ingestion::sample_san_francisco::create_san_francisco_dataset;

        let dir = tempdir().expect("Failed to create tempdir");
        let output_dir = dir.path().join("sf_world");

        let raw_dataset = create_san_francisco_dataset();
        let (validated, report) = validate_dataset(&raw_dataset);
        assert_eq!(report.quarantined.len(), 0);

        let dem = DemGrid::create_san_francisco_coastal_dem(32, 32);

        let options = CompilerOptions {
            chunk_size_meters: 500.0,
            terrain_grid_resolution: 8,
        };

        let stats = compile_world_package(&validated, &dem, &output_dir, &options)
            .expect("San Francisco world compilation failed");

        assert!(stats.total_chunks_generated >= 4, "Expected >= 4 chunks");
        assert!(stats.total_vertices > 0);
        assert!(stats.total_triangles > 0);

        // Verify SQLite index
        let db_path = output_dir.join("chunks.db");
        assert!(db_path.exists());
        let db = ChunkDatabase::open(&db_path).expect("Failed to read chunks.db");
        assert_eq!(db.count_chunks().unwrap(), stats.total_chunks_generated);

        // Verify min/max elevation in chunks captures coastal to hill range
        let all_chunks = db.query_bbox(&dem.bounds).expect("Query failed");
        let min_recorded = all_chunks
            .iter()
            .map(|c| c.min_alt)
            .fold(f32::INFINITY, f32::min);
        let max_recorded = all_chunks
            .iter()
            .map(|c| c.max_alt)
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(
            min_recorded <= 5.0,
            "Coastal chunk min elevation must be <= 5m MSL"
        );
        assert!(
            max_recorded >= 50.0,
            "Telegraph/Nob hill chunk max elevation must be >= 50m MSL"
        );
    }
}
