//! High-performance offline world compilation pipeline.

use crate::building_mesher::mesh_building;
use crate::chunk::{ChunkBuildingMeta, ChunkHeader, WorldChunk, CHUNK_MAGIC, CHUNK_VERSION};
use crate::database::{ChunkDatabase, ChunkIndexRecord};
use crate::mesh::ChunkMesh;
use crate::packager::write_chunk_file;
use crate::road_mesher::mesh_road;
use crate::terrain_mesher::mesh_terrain;
use realearth_geospatial::transforms::EnuConverter;
use realearth_ingestion::dem_grid::DemGrid;
use realearth_shared::bounds::GeoBounds;
use realearth_shared::coordinates::Wgs84Coord;
use realearth_shared::errors::Result;
use realearth_validation::pipeline::ValidatedDataset;
use std::path::Path;

/// Summary statistics of a compilation run.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CompilationStats {
    pub total_chunks_generated: usize,
    pub total_vertices: usize,
    pub total_triangles: usize,
    pub total_compressed_bytes: usize,
    pub total_uncompressed_bytes: usize,
}

/// Options for the world compiler.
#[derive(Debug, Clone)]
pub struct CompilerOptions {
    pub chunk_size_meters: f64,
    pub terrain_grid_resolution: usize,
}

impl Default for CompilerOptions {
    fn default() -> Self {
        Self {
            chunk_size_meters: 500.0,
            terrain_grid_resolution: 16,
        }
    }
}

/// Compiles a validated dataset and elevation model into a spatial world package.
pub fn compile_world_package(
    dataset: &ValidatedDataset,
    dem: &DemGrid,
    output_dir: &Path,
    options: &CompilerOptions,
) -> Result<CompilationStats> {
    std::fs::create_dir_all(output_dir)?;
    let chunks_dir = output_dir.join("chunks");
    std::fs::create_dir_all(&chunks_dir)?;

    let db_path = output_dir.join("chunks.db");
    let db = ChunkDatabase::open(&db_path)?;

    let bounds = dem.bounds;

    // Compute chunk grid dimensions based on metric size
    // 1 deg lat ~ 111,000m, 1 deg lon scales with cos(lat)
    let center_lat = bounds.center().lat;
    let m_per_deg_lat = 111_000.0;
    let m_per_deg_lon = (111_320.0 * center_lat.to_radians().cos())
        .abs()
        .max(10_000.0);

    let chunk_d_lat = options.chunk_size_meters / m_per_deg_lat;
    let chunk_d_lon = options.chunk_size_meters / m_per_deg_lon;

    let lat_span = bounds.max_lat - bounds.min_lat;
    let lon_span = bounds.max_lon - bounds.min_lon;

    let num_chunks_y = ((lat_span / chunk_d_lat).ceil() as usize).max(1);
    let num_chunks_x = ((lon_span / chunk_d_lon).ceil() as usize).max(1);

    let mut stats = CompilationStats::default();

    for cy in 0..num_chunks_y {
        let chunk_max_lat = bounds.max_lat - (cy as f64) * chunk_d_lat;
        let chunk_min_lat = (chunk_max_lat - chunk_d_lat).max(bounds.min_lat);

        for cx in 0..num_chunks_x {
            let chunk_min_lon = bounds.min_lon + (cx as f64) * chunk_d_lon;
            let chunk_max_lon = (chunk_min_lon + chunk_d_lon).min(bounds.max_lon);

            let chunk_bounds =
                GeoBounds::new(chunk_min_lat, chunk_max_lat, chunk_min_lon, chunk_max_lon);
            let chunk_anchor = chunk_bounds.center();
            let chunk_converter = EnuConverter::new(chunk_anchor);

            // 1. Terrain Mesh
            let terrain_mesh = mesh_terrain(
                &chunk_bounds,
                &chunk_converter,
                dem,
                options.terrain_grid_resolution,
            );

            // 2. Buildings Mesh & Metadata
            let mut building_mesh = ChunkMesh::default();
            let mut building_meta = Vec::new();

            for b in &dataset.buildings {
                let mean_lat =
                    b.outer_ring.iter().map(|p| p.lat).sum::<f64>() / b.outer_ring.len() as f64;
                let mean_lon =
                    b.outer_ring.iter().map(|p| p.lon).sum::<f64>() / b.outer_ring.len() as f64;
                let centroid = Wgs84Coord::new(mean_lat, mean_lon, 0.0);

                if chunk_bounds.contains(&centroid) {
                    let m = mesh_building(b, &chunk_converter, dem);
                    building_mesh.append(&m);

                    building_meta.push(ChunkBuildingMeta {
                        feature_id: b.id,
                        name: b.name.clone(),
                        height_meters: b.height.height_meters as f32,
                        provenance: b.height.provenance,
                        confidence: b.height.confidence,
                    });
                }
            }

            // 3. Roads Mesh
            let mut road_mesh = ChunkMesh::default();
            for r in &dataset.roads {
                // Check if road geometry intersects chunk bounds
                let mut intersects = false;
                for pt in &r.geometry {
                    if chunk_bounds.contains(pt) {
                        intersects = true;
                        break;
                    }
                }

                if intersects {
                    let m = mesh_road(r, &chunk_converter, dem);
                    road_mesh.append(&m);
                }
            }

            let feature_count = building_meta.len();

            let (min_elev, max_elev) = if terrain_mesh.vertices.is_empty() {
                (0.0, 100.0)
            } else {
                let mut min_e = f32::MAX;
                let mut max_e = f32::MIN;
                for v in &terrain_mesh.vertices {
                    if v.position[1] < min_e {
                        min_e = v.position[1];
                    }
                    if v.position[1] > max_e {
                        max_e = v.position[1];
                    }
                }
                (min_e, max_e)
            };

            let header = ChunkHeader {
                magic: *CHUNK_MAGIC,
                version: CHUNK_VERSION,
                chunk_x: cx as i32,
                chunk_y: cy as i32,
                lod_level: 0,
                bounds: chunk_bounds,
                anchor: chunk_anchor,
                min_elevation: min_elev,
                max_elevation: max_elev,
            };

            let chunk = WorldChunk {
                header,
                terrain: terrain_mesh,
                roads: road_mesh,
                buildings: building_mesh,
                building_metadata: building_meta,
            };

            let total_verts = chunk.total_vertices();
            let total_tris = chunk.total_triangles();

            let file_rel_path = format!("chunks/chunk_{}_{}.rec", cx, cy);
            let file_full_path = output_dir.join(&file_rel_path);

            let uncompressed_bytes = bincode::serialized_size(&chunk).unwrap_or(0) as usize;
            let compressed_bytes = write_chunk_file(&chunk, &file_full_path)?;

            // Register in SQLite
            db.insert_chunk(&ChunkIndexRecord {
                chunk_x: cx as i32,
                chunk_y: cy as i32,
                lod_level: 0,
                bounds: chunk_bounds,
                min_alt: min_elev,
                max_alt: max_elev,
                compressed_bytes,
                uncompressed_bytes,
                feature_count,
                storage_path: file_rel_path,
                file_offset: 0,
            })?;

            stats.total_chunks_generated += 1;
            stats.total_vertices += total_verts;
            stats.total_triangles += total_tris;
            stats.total_compressed_bytes += compressed_bytes;
            stats.total_uncompressed_bytes += uncompressed_bytes;
        }
    }

    Ok(stats)
}
