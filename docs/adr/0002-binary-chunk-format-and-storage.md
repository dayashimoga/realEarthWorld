# ADR 0002: Binary Chunk Package Format and SQLite Spatial Index

## Status
Accepted

## Context
Raw geospatial files (OSM XML/PBF, GeoTIFF DEMs) are unsuitable for real-time streaming because:
- Parsing raw XML/PBF at runtime introduces unacceptable latency spikes.
- Raw polygon triangulation (`earcutr`) and road meshing are CPU-intensive operations that must occur offline during world compilation.
- Memory budgets require fast, predictable chunk loading with minimal allocation overhead.

## Decision
1. **Offline World Compiler**: Pre-compile all validated geospatial data into self-contained 3D spatial chunk packages (`.rec` files) partitioned into $500\text{m} \times 500\text{m}$ grid tiles.
2. **Binary Layout**: Each `.rec` file contains pre-triangulated vertex and index buffers for terrain, roads, and extruded buildings, serialized with bincode and compressed with `zstd` level 3.
3. **SQLite Metadata & Spatial Index**: Store all chunk bounding boxes, file offsets, compressed sizes, and LOD tiers in an embedded SQLite database (`chunks.db`).
4. **Asynchronous Streaming**: Query the SQLite database using camera frustum bounding boxes on Tokio worker tasks and decompress chunk bodies in parallel using Rayon.

## Consequences
- Fast cold-start loading: chunks load in under $5\text{ ms}$ per chunk.
- Predictable memory footprint governed by an LRU cache with strict eviction thresholds.
- Independent, portable compiled world distribution packages.
