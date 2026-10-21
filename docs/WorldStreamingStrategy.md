# Real Earth World Platform — World Streaming Strategy

## 1. Spatial Partitioning & Chunking Scheme

To stream continuous planetary terrain and urban topology without boundary artifacts or massive upfront memory usage, the world is divided into discrete **Spatial Chunks**:

- **Chunk Extent**: $500\text{m} \times 500\text{m}$ (Level 0 fine chunk) or $2000\text{m} \times 2000\text{m}$ (Level 1 coarse chunk).
- **Coordinate Space**: Each chunk has an integer grid key $(C_x, C_y)$ and an anchor coordinate in WGS84 and ECEF.
- **Content Boundaries**: Buildings and road segments are assigned to chunks based on their bounding box centroid. Cross-chunk roads are cleanly cut or split at chunk boundaries with seam vertex matching to avoid visual tears.

---

## 2. SQLite Spatial Database Index (`chunks.db`)

Each compiled world package contains an embedded SQLite database indexing all chunk files:

```sql
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
```

---

## 3. Asynchronous Streaming Architecture

The runtime streaming engine operates on a multi-tier pipeline:

```
Camera Position (ECEF)
       │
       ▼
[Streaming Controller] ──(Frustum & Range Query)──► [SQLite Spatial Index]
       │
       ▼ (Candidate Chunks)
[Priority Queue] (Sorted by Distance & Velocity Vector)
       │
       ├──► [Tokio Async I/O Threadpool] (Disk Read .rec)
       │           │
       │           ▼
       ├──► [Rayon Worker Pool] (Zstd Decompression & Mesh Unpack)
       │           │
       │           ▼
       ├──► [Ready Queue] (Main Thread GPU Buffer Allocation)
       │
       ▼
[LRU Chunk Cache] ──(Evict oldest out-of-range chunks when RAM > Budget)
```

### Memory Budgeting & Eviction:
- **Max Memory Budget**: $1024\text{ MB}$ RSS (configurable).
- When total active chunk memory exceeds $85\%$ of the budget ($870\text{ MB}$), chunks farthest from the camera and outside the viewing frustum are evicted.
- GPU buffers (vertex buffers, index buffers) are dropped immediately upon eviction to prevent VRAM exhaustion.
