//! SQLite spatial indexing and chunk metadata registry (`chunks.db`).

use realearth_shared::bounds::GeoBounds;
use realearth_shared::errors::{RealEarthError, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Spatial chunk index record stored in SQLite.
#[derive(Debug, Clone, PartialEq)]
pub struct ChunkIndexRecord {
    pub chunk_x: i32,
    pub chunk_y: i32,
    pub lod_level: u32,
    pub bounds: GeoBounds,
    pub min_alt: f32,
    pub max_alt: f32,
    pub compressed_bytes: usize,
    pub uncompressed_bytes: usize,
    pub feature_count: usize,
    pub storage_path: String,
    pub file_offset: u64,
}

/// High-level package manifest stored in SQLite (`package_manifest` table).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ManifestMetadataRecord {
    pub package_name: String,
    pub crs_epsg: i32,
    pub bounds: GeoBounds,
    pub version: u32,
    pub schema_version: u32,
    pub created_at: String,
    pub source_asset_sha256: Option<String>,
}

/// Chunk checksum, source dependency hash, and neighbor connectivity metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChunkMetaRecord {
    pub chunk_x: i32,
    pub chunk_y: i32,
    pub lod_level: u32,
    pub sha256_checksum: String,
    pub source_hash: String,
    pub neighbor_n: Option<String>,
    pub neighbor_s: Option<String>,
    pub neighbor_e: Option<String>,
    pub neighbor_w: Option<String>,
}

/// SQLite database connection wrapping spatial queries for world chunks.
pub struct ChunkDatabase {
    conn: Connection,
}

impl ChunkDatabase {
    /// Opens or creates a new SQLite chunks database file.
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)
            .map_err(|e| RealEarthError::Compilation(format!("SQLite open error: {}", e)))?;

        conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA busy_timeout = 5000;

            CREATE TABLE IF NOT EXISTS chunks (
                chunk_x INTEGER NOT NULL,
                chunk_y INTEGER NOT NULL,
                lod_level INTEGER NOT NULL,
                min_lat REAL NOT NULL,
                max_lat REAL NOT NULL,
                min_lon REAL NOT NULL,
                max_lon REAL NOT NULL,
                min_alt REAL NOT NULL,
                max_alt REAL NOT NULL,
                compressed_bytes INTEGER NOT NULL,
                uncompressed_bytes INTEGER NOT NULL,
                feature_count INTEGER NOT NULL,
                storage_path TEXT NOT NULL,
                file_offset INTEGER NOT NULL,
                PRIMARY KEY (chunk_x, chunk_y, lod_level)
            );

            CREATE INDEX IF NOT EXISTS idx_chunks_spatial
            ON chunks (min_lat, max_lat, min_lon, max_lon);

            CREATE TABLE IF NOT EXISTS package_manifest (
                package_name TEXT PRIMARY KEY,
                crs_epsg INTEGER NOT NULL,
                min_lat REAL NOT NULL,
                max_lat REAL NOT NULL,
                min_lon REAL NOT NULL,
                max_lon REAL NOT NULL,
                version INTEGER NOT NULL,
                schema_version INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                source_asset_sha256 TEXT
            );

            CREATE TABLE IF NOT EXISTS chunk_metadata (
                chunk_x INTEGER NOT NULL,
                chunk_y INTEGER NOT NULL,
                lod_level INTEGER NOT NULL,
                sha256_checksum TEXT NOT NULL,
                source_hash TEXT NOT NULL,
                neighbor_n TEXT,
                neighbor_s TEXT,
                neighbor_e TEXT,
                neighbor_w TEXT,
                PRIMARY KEY (chunk_x, chunk_y, lod_level)
            );
            ",
        )
        .map_err(|e| RealEarthError::Compilation(format!("SQLite init schema error: {}", e)))?;

        // Backward compatibility migration for pre-existing databases without source_asset_sha256
        let _ = conn.execute(
            "ALTER TABLE package_manifest ADD COLUMN source_asset_sha256 TEXT",
            [],
        );

        Ok(Self { conn })
    }

    /// Inserts or replaces a chunk index record into the database.
    pub fn insert_chunk(&self, rec: &ChunkIndexRecord) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO chunks (
                    chunk_x, chunk_y, lod_level,
                    min_lat, max_lat, min_lon, max_lon,
                    min_alt, max_alt,
                    compressed_bytes, uncompressed_bytes, feature_count,
                    storage_path, file_offset
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                params![
                    rec.chunk_x,
                    rec.chunk_y,
                    rec.lod_level,
                    rec.bounds.min_lat,
                    rec.bounds.max_lat,
                    rec.bounds.min_lon,
                    rec.bounds.max_lon,
                    rec.min_alt,
                    rec.max_alt,
                    rec.compressed_bytes as i64,
                    rec.uncompressed_bytes as i64,
                    rec.feature_count as i64,
                    rec.storage_path,
                    rec.file_offset as i64,
                ],
            )
            .map_err(|e| {
                RealEarthError::Compilation(format!("SQLite insert chunk error: {}", e))
            })?;

        Ok(())
    }

    /// Queries all chunks intersecting a geographic bounding box.
    /// Fully handles bounding boxes spanning the 180° antimeridian.
    pub fn query_bbox(&self, bbox: &GeoBounds) -> Result<Vec<ChunkIndexRecord>> {
        let sql = if bbox.crosses_antimeridian() {
            "SELECT chunk_x, chunk_y, lod_level,
                    min_lat, max_lat, min_lon, max_lon,
                    min_alt, max_alt,
                    compressed_bytes, uncompressed_bytes, feature_count,
                    storage_path, file_offset
             FROM chunks
             WHERE min_lat <= ?2 AND max_lat >= ?1
               AND (max_lon >= ?3 OR min_lon <= ?4)"
        } else {
            "SELECT chunk_x, chunk_y, lod_level,
                    min_lat, max_lat, min_lon, max_lon,
                    min_alt, max_alt,
                    compressed_bytes, uncompressed_bytes, feature_count,
                    storage_path, file_offset
             FROM chunks
             WHERE min_lat <= ?2 AND max_lat >= ?1
               AND min_lon <= ?4 AND max_lon >= ?3"
        };

        let mut stmt = self.conn.prepare(sql).map_err(|e| {
            RealEarthError::Compilation(format!("SQLite prepare query error: {}", e))
        })?;

        let rows = stmt
            .query_map(
                params![bbox.min_lat, bbox.max_lat, bbox.min_lon, bbox.max_lon],
                |row| {
                    let min_lat: f64 = row.get(3)?;
                    let max_lat: f64 = row.get(4)?;
                    let min_lon: f64 = row.get(5)?;
                    let max_lon: f64 = row.get(6)?;

                    let bounds = if min_lon > max_lon {
                        GeoBounds::new_antimeridian(min_lat, max_lat, min_lon, max_lon)
                    } else {
                        GeoBounds::new(min_lat, max_lat, min_lon, max_lon)
                    };

                    Ok(ChunkIndexRecord {
                        chunk_x: row.get(0)?,
                        chunk_y: row.get(1)?,
                        lod_level: row.get(2)?,
                        bounds,
                        min_alt: row.get(7)?,
                        max_alt: row.get(8)?,
                        compressed_bytes: row.get::<_, i64>(9)? as usize,
                        uncompressed_bytes: row.get::<_, i64>(10)? as usize,
                        feature_count: row.get::<_, i64>(11)? as usize,
                        storage_path: row.get(12)?,
                        file_offset: row.get::<_, i64>(13)? as u64,
                    })
                },
            )
            .map_err(|e| RealEarthError::Compilation(format!("SQLite query map error: {}", e)))?;

        let mut results = Vec::new();
        for r in rows {
            results.push(
                r.map_err(|e| RealEarthError::Compilation(format!("SQLite row error: {}", e)))?,
            );
        }
        Ok(results)
    }

    /// Returns the total number of indexed chunks.
    pub fn count_chunks(&self) -> Result<usize> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM chunks", [], |row| row.get(0))
            .map_err(|e| RealEarthError::Compilation(format!("SQLite count error: {}", e)))?;
        Ok(count as usize)
    }

    /// Returns the total feature count across all stored chunks.
    pub fn total_feature_count(&self) -> Result<usize> {
        let count: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(feature_count), 0) FROM chunks",
                [],
                |row| row.get(0),
            )
            .map_err(|e| {
                RealEarthError::Compilation(format!("SQLite total features error: {}", e))
            })?;
        Ok(count as usize)
    }

    /// Sets or updates the package manifest.
    pub fn set_package_manifest(&self, manifest: &ManifestMetadataRecord) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO package_manifest (
                    package_name, crs_epsg,
                    min_lat, max_lat, min_lon, max_lon,
                    version, schema_version, created_at,
                    source_asset_sha256
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    manifest.package_name,
                    manifest.crs_epsg,
                    manifest.bounds.min_lat,
                    manifest.bounds.max_lat,
                    manifest.bounds.min_lon,
                    manifest.bounds.max_lon,
                    manifest.version,
                    manifest.schema_version,
                    manifest.created_at,
                    manifest.source_asset_sha256,
                ],
            )
            .map_err(|e| {
                RealEarthError::Compilation(format!("SQLite set manifest error: {}", e))
            })?;
        Ok(())
    }

    /// Retrieves package manifest by name.
    pub fn get_package_manifest(
        &self,
        package_name: &str,
    ) -> Result<Option<ManifestMetadataRecord>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT package_name, crs_epsg, min_lat, max_lat, min_lon, max_lon, version, schema_version, created_at, source_asset_sha256
                 FROM package_manifest WHERE package_name = ?1",
            )
            .map_err(|e| RealEarthError::Compilation(format!("SQLite prepare manifest query error: {}", e)))?;

        let mut rows = stmt
            .query_map(params![package_name], |row| {
                let name: String = row.get(0)?;
                let crs: i32 = row.get(1)?;
                let min_lat: f64 = row.get(2)?;
                let max_lat: f64 = row.get(3)?;
                let min_lon: f64 = row.get(4)?;
                let max_lon: f64 = row.get(5)?;
                let ver: u32 = row.get(6)?;
                let s_ver: u32 = row.get(7)?;
                let created: String = row.get(8)?;
                let source_hash: Option<String> = row.get(9).ok();

                Ok(ManifestMetadataRecord {
                    package_name: name,
                    crs_epsg: crs,
                    bounds: GeoBounds::new(min_lat, max_lat, min_lon, max_lon),
                    version: ver,
                    schema_version: s_ver,
                    created_at: created,
                    source_asset_sha256: source_hash,
                })
            })
            .map_err(|e| {
                RealEarthError::Compilation(format!("SQLite query manifest error: {}", e))
            })?;

        if let Some(r) = rows.next() {
            let manifest =
                r.map_err(|e| RealEarthError::Compilation(format!("SQLite row error: {}", e)))?;
            Ok(Some(manifest))
        } else {
            Ok(None)
        }
    }

    /// Returns true if the package manifest exists and its recorded source_asset_sha256 matches expected_hash
    /// and its recorded schema_version matches the current CHUNK_VERSION.
    pub fn is_package_source_hash_matching(&self, package_name: &str, expected_hash: &str) -> bool {
        match self.get_package_manifest(package_name) {
            Ok(Some(manifest)) => {
                manifest.source_asset_sha256.as_deref() == Some(expected_hash)
                    && manifest.schema_version == crate::chunk::CHUNK_VERSION
            }
            _ => false,
        }
    }

    /// Inserts or updates chunk checksum, source hash, and neighbor metadata.
    pub fn insert_chunk_metadata(&self, meta: &ChunkMetaRecord) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO chunk_metadata (
                    chunk_x, chunk_y, lod_level,
                    sha256_checksum, source_hash,
                    neighbor_n, neighbor_s, neighbor_e, neighbor_w
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    meta.chunk_x,
                    meta.chunk_y,
                    meta.lod_level,
                    meta.sha256_checksum,
                    meta.source_hash,
                    meta.neighbor_n,
                    meta.neighbor_s,
                    meta.neighbor_e,
                    meta.neighbor_w,
                ],
            )
            .map_err(|e| {
                RealEarthError::Compilation(format!("SQLite insert chunk metadata error: {}", e))
            })?;
        Ok(())
    }

    /// Retrieves chunk metadata (checksums, source hash, neighbors).
    pub fn get_chunk_metadata(
        &self,
        chunk_x: i32,
        chunk_y: i32,
        lod_level: u32,
    ) -> Result<Option<ChunkMetaRecord>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT chunk_x, chunk_y, lod_level, sha256_checksum, source_hash,
                        neighbor_n, neighbor_s, neighbor_e, neighbor_w
                 FROM chunk_metadata
                 WHERE chunk_x = ?1 AND chunk_y = ?2 AND lod_level = ?3",
            )
            .map_err(|e| {
                RealEarthError::Compilation(format!(
                    "SQLite prepare chunk metadata query error: {}",
                    e
                ))
            })?;

        let mut rows = stmt
            .query_map(params![chunk_x, chunk_y, lod_level], |row| {
                Ok(ChunkMetaRecord {
                    chunk_x: row.get(0)?,
                    chunk_y: row.get(1)?,
                    lod_level: row.get(2)?,
                    sha256_checksum: row.get(3)?,
                    source_hash: row.get(4)?,
                    neighbor_n: row.get(5)?,
                    neighbor_s: row.get(6)?,
                    neighbor_e: row.get(7)?,
                    neighbor_w: row.get(8)?,
                })
            })
            .map_err(|e| {
                RealEarthError::Compilation(format!("SQLite query chunk metadata error: {}", e))
            })?;

        if let Some(r) = rows.next() {
            let meta =
                r.map_err(|e| RealEarthError::Compilation(format!("SQLite row error: {}", e)))?;
            Ok(Some(meta))
        } else {
            Ok(None)
        }
    }

    /// Verifies the cryptographic integrity of a chunk on disk against recorded checksum in the manifest.
    pub fn verify_chunk_integrity(
        &self,
        chunk_x: i32,
        chunk_y: i32,
        lod_level: u32,
        file_path: &Path,
    ) -> Result<bool> {
        let meta = match self.get_chunk_metadata(chunk_x, chunk_y, lod_level)? {
            Some(m) => m,
            None => return Ok(false),
        };

        if !file_path.exists() {
            return Ok(false);
        }

        let bytes = std::fs::read(file_path)?;
        let computed_checksum = realearth_ingestion::acquisition::compute_sha256(&bytes);
        Ok(computed_checksum.eq_ignore_ascii_case(&meta.sha256_checksum))
    }

    /// Invalidates and removes a chunk from both spatial index and metadata registry.
    pub fn invalidate_chunk(&self, chunk_x: i32, chunk_y: i32, lod_level: u32) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM chunks WHERE chunk_x = ?1 AND chunk_y = ?2 AND lod_level = ?3",
                params![chunk_x, chunk_y, lod_level],
            )
            .map_err(|e| {
                RealEarthError::Compilation(format!("SQLite invalidate chunk error: {}", e))
            })?;

        self.conn
            .execute(
                "DELETE FROM chunk_metadata WHERE chunk_x = ?1 AND chunk_y = ?2 AND lod_level = ?3",
                params![chunk_x, chunk_y, lod_level],
            )
            .map_err(|e| {
                RealEarthError::Compilation(format!(
                    "SQLite invalidate chunk_metadata error: {}",
                    e
                ))
            })?;

        Ok(())
    }
}
