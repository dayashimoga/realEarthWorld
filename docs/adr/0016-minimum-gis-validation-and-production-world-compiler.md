# ADR 0016: Minimum GIS Validation and Production World Compiler

## Status
Accepted

## Context
Stage 2 of the Real Earth World Platform establishes the foundational compilation pipeline from raw GIS ingestion outputs (OpenStreetMap XML and Digital Elevation Model rasters) into validated, partitioned 3D binary world chunks.

Key requirements:
1. **Blocking Validation**: Pre-compilation validation must detect geometric, topological, coordinate, grade-separation, and water-conflict defects. Critical defects must halt compilation or quarantine malformed features; diagnostics must follow a strict severity hierarchy (`BLOCKER`, `ERROR`, `WARNING`, `INFO`). No fabricated geometry or silent modifications may occur.
2. **Production World Compiler**: Geography must be partitioned hierarchically into spatial chunk grids. For each chunk, the compiler must generate 3D meshes (terrain conforming to DEM elevations, multi-lane road ribbons with bridge parapets/piers, extruded building walls with `earcutr` roof triangulation, specular water surfaces, and vegetation areas), LOD meshes, collision representations, and road routing graphs while preserving factual source provenance.
3. **Binary Storage & Incremental Compilation**: World chunks must be stored in compact, versioned, zstd-compressed binary files (`.rec`) with SQLite metadata manifests (`chunks.db`) tracking CRS EPSG codes, geographic envelopes, source feature IDs, dependency metadata, and SHA-256 digests. The compiler must support deterministic incremental compilation (skipping unchanged chunks) and corruption detection upon reload.
4. **Validation Proof**: The compiled world package must be loadable in headless mode with bit-exact repeatability and identical geographic alignment, proving zero topological gaps and no critical seams.

## Decisions

### 1. Blocking Validation Pipeline (`realearth-validation`)
- Implement strict coordinate sanity verification: reject any coordinates containing `NaN`, infinity, or out-of-range geodetic coordinates ($|\text{lat}| > 90^\circ$ or $|\text{lon}| > 180^\circ$) as `BLOCKER` defects.
- Implement road topology and grade-separation verification: enforce that overpasses, flyovers, and elevated structures (`layer > 0` or `bridge = yes`) maintain vertical clearance and never collapse into planar junctions with ground roads.
- Implement polygon geometry validation: cull duplicate vertices, detect self-intersecting bowties, enforce CCW winding order, and quarantine unrepairable degeneracies.
- Implement water-conflict validation: detect building footprints submerged within water bodies without dock or pier metadata.
- Enforce the `CompilerPublicationGate`: reject packages containing any unresolved `BLOCKER` or unquarantined `ERROR` defects.

### 2. Hierarchical Spatial World Compiler (`realearth-compiler`)
- Partition geographic areas into regular metric chunk grids (e.g. $500\text{m} \times 500\text{m}$ or $1000\text{m} \times 1000\text{m}$) computed with metric latitude/longitude scale factors.
- Generate specialized meshes for each feature class:
  - **Terrain Mesh**: Regular grid heightfield sampled from bilinear DEM interpolation with per-vertex surface normals and UV coordinates.
  - **Road Mesh**: Continuous road ribbons with lane widths, centerlines, edge vertices, bridge piers, and elevated clearance profiles.
  - **Building Mesh**: 3D extruded vertical wall quads and planar roof triangulation with explicit `HeightProvenance` attribution.
  - **Water Mesh**: Specular water surface geometry adhering to closed water polygons.
  - **Vegetation Mesh**: Urban parks and green space planar meshes.
  - **LOD Meshes**: Support multi-resolution LOD levels (LOD 0 fine, LOD 1 medium, LOD 2 coarse).
  - **Navigation & Routing**: Extract topological road graphs with directed edges and intersection nodes.

### 3. Binary Storage, Checksums & Incremental Compilation
- Serialize chunk geometry into versioned binary streams (`CHUNK_MAGIC = b"RECH"`, `CHUNK_VERSION = 1`) compressed with `zstd`.
- Maintain SQLite manifest index (`chunks.db`) storing chunk coordinates, bounding boxes, min/max elevations, compressed/uncompressed sizes, feature counts, and storage paths.
- Add manifest metadata tracking projection CRS (e.g. EPSG:32643 for Bengaluru, EPSG:32610 for San Francisco), version strings, and schema versions.
- Implement deterministic incremental compilation: calculate a source feature digest for each chunk. If source features and DEM are unchanged and the on-disk `.rec` file matches the stored checksum, skip meshing and compression to achieve fast incremental rebuilds.
- Implement corruption detection: verify file headers and SHA-256 digests on chunk reload, triggering fallback recovery if tampered or truncated.

## Consequences
- World compilation is fully repeatable: running compilation twice on identical inputs yields bit-exact chunk outputs.
- Build times for regional packages are minimized via incremental changed-chunk rebuilds.
- Headless simulation, world viewing, and navigation can reload compiled chunks without re-parsing raw OSM XML or raster DEM files.
- The "No Fabrication Rule" is maintained across all compilation stages.
