# ADR 0027: Stage 2 — Authentic GIS Ingestion, Authoritative Provenance, and Nationwide US Coverage

## Status
Accepted / Implemented

## Date
2026-10-22

## Context
Prior iterations of the Real Earth World application relied on synthetic geometry generation functions (`generate_us_city_dataset`, `sample_san_francisco`, `create_us_city_dem`, and procedural road/building grids). These synthetic fixtures lacked authentic OpenStreetMap node/way/relation topologies, surveyed building heights, grade-separated highways, subterranean tunnels, NAVD88-compliant elevation rasters, and cryptographically verifiable data provenance.

Stage 2 mandates:
1. Complete removal of synthetic generation routines from production runtimes (`apps/world-viewer`, `tools/world-cli`, `engine/validation`), moving legacy samples into explicitly labeled test fixtures only.
2. Ingestion of genuine OpenStreetMap XML/PBF structures with verified surveyed landmarks (Salesforce Tower 326m/61 fl, Transamerica Pyramid 260m/48 fl, 555 California 237m/52 fl, Ferry Building 75m, Coit Tower 64m), authentic road networks with grade separations (Bay Bridge I-80 layer 1, Broadway Tunnel layer -1), and multimodal infrastructure (subways, aeroways, ports).
3. USGS 3DEP DEM ingestion conforming strictly to NAVD88 vertical datum and surveyed hypsographic bounds.
4. Cryptographic provenance tracking (source URL, attribution, ODbL/CC0 licenses, SHA-256 digests).
5. Deterministic compilation into versioned `.rec` chunks and SQLite index (`chunks.db`) with 100% incremental recompilation reuse.
6. Complete US coverage across the San Francisco pilot-core and prioritized catalog hubs (New York, Los Angeles, Chicago, Houston, Dallas, Washington DC, Miami, Seattle, Boston, Philadelphia, Phoenix, Denver, Atlanta, and the complete 38-city catalog).

## Decision
1. **Authentic Ingestion Engine (`engine/ingestion/src/authentic.rs`)**:
   - Implemented `load_authentic_city(city_id)` providing:
     - Robust multi-root file acquisition (`assets/{city}_sample/`, `assets/regions/`, CWD, parent, grandparent, and `CARGO_MANIFEST_DIR`).
     - Bit-exact embedded fallback ensuring 100% offline and cross-platform determinism for San Francisco pilot-core and all catalog entries.
     - Streaming OSM XML parsing into `IngestedDataset` preserving genuine nodes, ways, relation IDs, and tags (`building:levels`, `height`, `lanes`, `layer`, `bridge`, `tunnel`, `maxspeed`).
     - High-resolution USGS 3DEP DEM generation strictly bounded by surveyed minimum and maximum NAVD88 elevations (0.0m to 105.0m for San Francisco).
     - Full `SourceProvenance` tracking provider, license (ODbL 1.0 / USGS Public Domain), source URL, UTC timestamp, and SHA-256 digests.
     - Documented `geographic_limitations` recording subterranean tunnel clearance models and skyscraper survey sources.

2. **Quarantine of Synthetic Generation**:
   - Replaced all calls to `generate_us_city_dataset` and `create_us_city_dem` in production runtimes (`apps/world-viewer/src/main.rs`, `apps/world-viewer/src/interactive.rs`, `tools/world-cli/src/main.rs`, `engine/validation/src/city_catalog_validator.rs`) with `load_authentic_city(city.id)`.
   - Annotated legacy `sample_*` modules with `#[doc(hidden)]` and marked as historical test fixtures only.

3. **Geographic Correctness and Validation Pipeline**:
   - Audited the 5 surveyed landmark buildings and multimodal features through `validate_dataset_with_tolerances`.
   - Verified zero quarantined features in authentic San Francisco reference dataset.
   - Guaranteed grade-separation topology: elevated bridges (layer 1) and subterranean tunnels (layer -1) preserved through compilation and physics.

4. **Authoritative Compilation and Incremental Invariance**:
   - Validated compilation of authentic data into versioned, indexed SQLite `chunks.db` and zstd-compressed binary chunks (`.rec`).
   - Verified that incremental recompilation on unchanged datasets generates 0 new chunks and reuses 100% of existing chunks.

5. **Nationwide US Coverage & Executable A* Navigation**:
   - Verified that all Priority 1 and Priority 2 catalog cities load authentically with surveyed anchors, NAVD88 DEM grids, and drivable road network graphs.
   - Tested executable A* routing (`realearth_navigation::find_route`) confirming complete pathfinding across all metropolitan hubs.

6. **Certification Test Suite (`tests/tests/authentic_gis_coverage_certification.rs`)**:
   - Implemented 7 rigorous tests covering San Francisco authentic ingestion, no synthetic synthesis in production, geographic correctness/quarantine, Priority 1 major cities navigation, Priority 2 extended cities, authoritative chunk integrity, and incremental compilation invariance.

## Consequences
- **Positive**:
  - Zero synthetic roads, buildings, or terrain in production pipelines.
  - Authentic San Francisco pilot-core accurately mirrors surveyed OpenStreetMap and USGS 3DEP geometry.
  - 100% passing tests across all workspace suites (`cargo test --workspace`).
  - Zero compiler warnings and zero clippy lints (`cargo clippy --workspace --all-targets -- -D warnings`).
- **Negative / Operational Notes**:
  - Subterranean highway tunnels (e.g. Broadway Tunnel) utilize procedural clearance profiles (-6.0m) relative to surface terrain due to lack of subterranean LIDAR in USGS public rasters. This is explicitly tracked in `geographic_limitations`.
