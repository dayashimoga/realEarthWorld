//! Binary chunk packaging and Zstandard compression/decompression (.rec files).

use crate::chunk::{WorldChunk, CHUNK_MAGIC};
use realearth_shared::errors::{RealEarthError, Result};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

/// Serializes and compresses a WorldChunk into a binary buffer using Zstandard (level 3).
pub fn pack_chunk_to_bytes(chunk: &WorldChunk) -> Result<Vec<u8>> {
    let uncompressed = bincode::serialize(chunk)
        .map_err(|e| RealEarthError::Serialization(format!("Chunk serialization failed: {}", e)))?;

    let compressed = zstd::encode_all(&uncompressed[..], 3)
        .map_err(|e| RealEarthError::Serialization(format!("Chunk compression failed: {}", e)))?;

    Ok(compressed)
}

/// Decompresses and deserializes a binary buffer into a WorldChunk.
pub fn unpack_chunk_from_bytes(bytes: &[u8]) -> Result<WorldChunk> {
    let decompressed = zstd::decode_all(bytes)
        .map_err(|e| RealEarthError::Serialization(format!("Chunk decompression failed: {}", e)))?;

    let chunk: WorldChunk = bincode::deserialize(&decompressed).map_err(|e| {
        RealEarthError::Serialization(format!("Chunk deserialization failed: {}", e))
    })?;

    if &chunk.header.magic != CHUNK_MAGIC {
        return Err(RealEarthError::Serialization(format!(
            "Invalid chunk magic header: expected {:?}, got {:?}",
            CHUNK_MAGIC, chunk.header.magic
        )));
    }

    Ok(chunk)
}

/// Writes a compressed WorldChunk to a `.rec` file on disk.
pub fn write_chunk_file(chunk: &WorldChunk, path: &Path) -> Result<usize> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let compressed = pack_chunk_to_bytes(chunk)?;
    let mut file = File::create(path)?;
    file.write_all(&compressed)?;
    Ok(compressed.len())
}

/// Reads and decompresses a WorldChunk from a `.rec` file on disk.
pub fn read_chunk_file(path: &Path) -> Result<WorldChunk> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    unpack_chunk_from_bytes(&buffer)
}
