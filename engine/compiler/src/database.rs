//! SQLite spatial indexing and chunk metadata registry (`chunks.db`).

use realearth_shared::bounds::GeoBounds;
use realearth_shared::errors::{RealEarthError, Result};
use rusqlite::{params, Connection};
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
            ",
        )
        .map_err(|e| RealEarthError::Compilation(format!("SQLite init schema error: {}", e)))?;

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
    pub fn query_bbox(&self, bbox: &GeoBounds) -> Result<Vec<ChunkIndexRecord>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT chunk_x, chunk_y, lod_level,
                        min_lat, max_lat, min_lon, max_lon,
                        min_alt, max_alt,
                        compressed_bytes, uncompressed_bytes, feature_count,
                        storage_path, file_offset
                 FROM chunks
                 WHERE min_lat <= ?2 AND max_lat >= ?1
                   AND min_lon <= ?4 AND max_lon >= ?3",
            )
            .map_err(|e| {
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

                    Ok(ChunkIndexRecord {
                        chunk_x: row.get(0)?,
                        chunk_y: row.get(1)?,
                        lod_level: row.get(2)?,
                        bounds: GeoBounds::new(min_lat, max_lat, min_lon, max_lon),
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
}
