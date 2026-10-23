//! High-performance offline world compilation pipeline.

use crate::building_mesher::mesh_building;
use crate::chunk::{ChunkBuildingMeta, ChunkHeader, WorldChunk, CHUNK_MAGIC, CHUNK_VERSION};
use crate::database::{ChunkDatabase, ChunkIndexRecord, ChunkMetaRecord, ManifestMetadataRecord};
use crate::mesh::ChunkMesh;
use crate::packager::write_chunk_file;
use crate::road_mesher::mesh_road;
use crate::terrain_mesher::mesh_terrain;
use crate::vegetation_mesher::mesh_vegetation;
use crate::water_mesher::mesh_water;
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
    pub chunks_reused: usize,
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
    pub incremental: bool,
    pub crs_epsg: i32,
    pub package_name: String,
}

impl Default for CompilerOptions {
    fn default() -> Self {
        Self {
            chunk_size_meters: 500.0,
            terrain_grid_resolution: 16,
            incremental: false,
            crs_epsg: 4326,
            package_name: "world_package".to_string(),
        }
    }
}

/// Deterministically computes a SHA-256 hash of the GIS features and elevation within a chunk.
pub fn compute_chunk_source_hash(
    chunk_bounds: &GeoBounds,
    dataset: &ValidatedDataset,
    dem: &DemGrid,
) -> String {
    let mut hasher_input = Vec::new();

    // 1. Buildings intersecting chunk bounds
    for b in &dataset.buildings {
        let mean_lat = b.outer_ring.iter().map(|p| p.lat).sum::<f64>() / b.outer_ring.len() as f64;
        let mean_lon = b.outer_ring.iter().map(|p| p.lon).sum::<f64>() / b.outer_ring.len() as f64;
        let centroid = Wgs84Coord::new(mean_lat, mean_lon, 0.0);
        if chunk_bounds.contains(&centroid) {
            hasher_input.extend_from_slice(&b.id.to_le_bytes());
            hasher_input.extend_from_slice(&(b.height.height_meters as f32).to_le_bytes());
            for pt in &b.outer_ring {
                hasher_input.extend_from_slice(&pt.lat.to_le_bytes());
                hasher_input.extend_from_slice(&pt.lon.to_le_bytes());
            }
        }
    }

    // 2. Roads intersecting chunk bounds
    for r in &dataset.roads {
        if r.geometry.iter().any(|pt| chunk_bounds.contains(pt)) {
            hasher_input.extend_from_slice(&r.way_id.to_le_bytes());
            hasher_input.push(r.layer as u8);
            hasher_input.push(if r.is_bridge { 1 } else { 0 });
            hasher_input.push(if r.is_tunnel { 1 } else { 0 });
            for pt in &r.geometry {
                hasher_input.extend_from_slice(&pt.lat.to_le_bytes());
                hasher_input.extend_from_slice(&pt.lon.to_le_bytes());
            }
        }
    }

    // 3. Water bodies intersecting chunk bounds
    for w in &dataset.water_bodies {
        if w.outer_ring.iter().any(|pt| chunk_bounds.contains(pt)) {
            hasher_input.extend_from_slice(&w.id.to_le_bytes());
            for pt in &w.outer_ring {
                hasher_input.extend_from_slice(&pt.lat.to_le_bytes());
                hasher_input.extend_from_slice(&pt.lon.to_le_bytes());
            }
        }
    }

    // 4. Land cover intersecting chunk bounds
    for lc in &dataset.land_cover {
        if lc.polygon.iter().any(|pt| chunk_bounds.contains(pt)) {
            hasher_input.extend_from_slice(&lc.id.to_le_bytes());
            for pt in &lc.polygon {
                hasher_input.extend_from_slice(&pt.lat.to_le_bytes());
                hasher_input.extend_from_slice(&pt.lon.to_le_bytes());
            }
        }
    }

    // 5. DEM samples
    let sample_points = [
        Wgs84Coord::new(chunk_bounds.min_lat, chunk_bounds.min_lon, 0.0),
        Wgs84Coord::new(chunk_bounds.min_lat, chunk_bounds.max_lon, 0.0),
        Wgs84Coord::new(chunk_bounds.max_lat, chunk_bounds.min_lon, 0.0),
        Wgs84Coord::new(chunk_bounds.max_lat, chunk_bounds.max_lon, 0.0),
        chunk_bounds.center(),
    ];
    for sp in &sample_points {
        let elev = dem.sample_bilinear(sp);
        hasher_input.extend_from_slice(&elev.to_le_bytes());
    }

    realearth_ingestion::acquisition::compute_sha256(&hasher_input)
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

    let package_anchor = bounds.center();
    let package_converter = EnuConverter::new(package_anchor);

    for cy in 0..num_chunks_y {
        let chunk_max_lat = bounds.max_lat - (cy as f64) * chunk_d_lat;
        let chunk_min_lat = (chunk_max_lat - chunk_d_lat).max(bounds.min_lat);

        for cx in 0..num_chunks_x {
            let chunk_min_lon = bounds.min_lon + (cx as f64) * chunk_d_lon;
            let chunk_max_lon = (chunk_min_lon + chunk_d_lon).min(bounds.max_lon);

            let chunk_bounds =
                GeoBounds::new(chunk_min_lat, chunk_max_lat, chunk_min_lon, chunk_max_lon);
            let chunk_anchor = package_anchor;
            let chunk_converter = package_converter;

            let source_hash = compute_chunk_source_hash(&chunk_bounds, dataset, dem);
            let file_rel_path = format!("chunks/chunk_{}_{}.rec", cx, cy);
            let file_full_path = output_dir.join(&file_rel_path);

            if options.incremental {
                if let Ok(Some(existing_meta)) = db.get_chunk_metadata(cx as i32, cy as i32, 0) {
                    if existing_meta.source_hash == source_hash && file_full_path.exists() {
                        if let Ok(file_bytes) = std::fs::read(&file_full_path) {
                            let computed =
                                realearth_ingestion::acquisition::compute_sha256(&file_bytes);
                            if computed == existing_meta.sha256_checksum {
                                stats.chunks_reused += 1;
                                continue;
                            }
                        }
                    }
                }
            }

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

            // 4. Water Bodies Mesh
            let mut water_mesh = ChunkMesh::default();
            for w in &dataset.water_bodies {
                let mut intersects = false;
                for pt in &w.outer_ring {
                    if chunk_bounds.contains(pt) {
                        intersects = true;
                        break;
                    }
                }
                if intersects {
                    let m = mesh_water(w, &chunk_converter, dem);
                    water_mesh.append(&m);
                }
            }

            // 5. Vegetation & Urban Parks Mesh
            let mut vegetation_mesh = ChunkMesh::default();
            for lc in &dataset.land_cover {
                let mut intersects = false;
                for pt in &lc.polygon {
                    if chunk_bounds.contains(pt) {
                        intersects = true;
                        break;
                    }
                }
                if intersects {
                    let m = mesh_vegetation(lc, &chunk_converter, dem);
                    vegetation_mesh.append(&m);
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
                water: water_mesh,
                vegetation: vegetation_mesh,
                building_metadata: building_meta,
            };

            let total_verts = chunk.total_vertices();
            let total_tris = chunk.total_triangles();

            let uncompressed_bytes = bincode::serialized_size(&chunk).unwrap_or(0) as usize;
            let compressed_bytes = write_chunk_file(&chunk, &file_full_path)?;

            let chunk_file_bytes = std::fs::read(&file_full_path)?;
            let file_sha256 = realearth_ingestion::acquisition::compute_sha256(&chunk_file_bytes);

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

            let neighbor_n = if cy > 0 {
                Some(format!("chunk_{}_{}", cx, cy - 1))
            } else {
                None
            };
            let neighbor_s = if cy + 1 < num_chunks_y {
                Some(format!("chunk_{}_{}", cx, cy + 1))
            } else {
                None
            };
            let neighbor_e = if cx + 1 < num_chunks_x {
                Some(format!("chunk_{}_{}", cx + 1, cy))
            } else {
                None
            };
            let neighbor_w = if cx > 0 {
                Some(format!("chunk_{}_{}", cx - 1, cy))
            } else {
                None
            };

            db.insert_chunk_metadata(&ChunkMetaRecord {
                chunk_x: cx as i32,
                chunk_y: cy as i32,
                lod_level: 0,
                sha256_checksum: file_sha256,
                source_hash,
                neighbor_n,
                neighbor_s,
                neighbor_e,
                neighbor_w,
            })?;

            stats.total_chunks_generated += 1;
            stats.total_vertices += total_verts;
            stats.total_triangles += total_tris;
            stats.total_compressed_bytes += compressed_bytes;
            stats.total_uncompressed_bytes += uncompressed_bytes;
        }
    }

    db.set_package_manifest(&ManifestMetadataRecord {
        package_name: options.package_name.clone(),
        crs_epsg: options.crs_epsg,
        bounds,
        version: 1,
        schema_version: CHUNK_VERSION,
        created_at: "2026-10-22T00:00:00Z".to_string(),
    })?;

    Ok(stats)
}
