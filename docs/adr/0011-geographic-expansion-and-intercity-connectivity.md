# ADR 0011: Geographic Expansion and Intercity Connectivity Architecture

## Status
Accepted

## Context
In Stage 6, the Real Earth World Platform established a complete, drivable vehicle simulation running on authentic bounded regions (Bengaluru Central and San Francisco Waterfront) with fixed-timestep Bevy ECS physics, Pacejka/friction-circle tire models, and autonomous IDM traffic.

However, realistic Earth-scale simulation requires expanding beyond single isolated metropolitan islands to intercity transport networks. Vehicles must be able to drive between major economic and cultural centers without geographic discontinuities, artificial teleportation seams, floating roads, or runaway memory consumption.

Specifically, the platform requires:
1. **Regional Expansion**: Authentic source data packages for four major South Indian metropolitan centers:
   - **Bengaluru** (Karnataka capital, Deccan plateau ~920m MSL, UTM Zone 43N, datum EGM96).
   - **Mysuru** (Karnataka heritage center ~760m MSL, Chamundi Hill 1060m MSL, UTM Zone 43N, datum EGM96).
   - **Chennai** (Tamil Nadu capital, Bay of Bengal coastal waterfront ~6m MSL, UTM Zone 44N, datum EGM96).
   - **Hyderabad** (Telangana capital, Deccan plateau ~540m MSL, Musi River & Hussain Sagar, UTM Zone 44N, datum EGM96).
2. **Authentic Transport Corridors**: Real connecting expressways, national highways, grade-separated trumpet interchanges, river bridges (Kaveri at Srirangapatna, Palar at Ranipet, Krishna & Tungabhadra at Kurnool, Musi at Hyderabad), railway corridors, and terrain cuttings without fabricated connectors.
3. **Scalable Regional Packaging & Multi-Region Streaming**:
   - Decentralized, versioned regional packages (`RegionPackage`, `RegionalCatalog`, `PackageManifest`).
   - Incremental installation/updates without invalidating or re-compiling existing regions.
   - Lazy loading and spatial routing across regional chunk databases.
   - Seamless cross-boundary streaming with bounded LRU memory eviction (< 1024 MB RSS) during multi-hundred-kilometer drives.

## Decision

### 1. Multi-Region Ingestion & Coordinate Standard
Each region is defined by an immutable, verified `RegionProfile`:
- Coordinates: WGS84 double-precision geodetic coordinates (`Wgs84Coord`).
- Local Projections: Automatic UTM Zone selection (Zone 43N for Bengaluru & Mysuru; Zone 44N for Chennai & Hyderabad).
- Vertical Datum: EGM96 orthometric height above Mean Sea Level (MSL), aligning coastal regions (Chennai at 0m-25m) with high Deccan plateaus (Bengaluru 920m, Mysuru 760m, Hyderabad 540m).
- Ingested features: Real surveyed OpenStreetMap highways, primary/secondary arteries, bridges (`layer: 1`), tunnels (`layer: -1`), building footprints, water bodies, and railway corridors.
- DEM Grids: High-resolution raster models capturing real geographic terrain features:
  - Chamundi Hill granite monadnock (1060m MSL) rising above Mysuru plateau.
  - Eastern Ghats escarpment descent from Hoskote/Kolar (850m) down to Chennai coastline (0m).
  - Deccan granite formations and Musi River depression across Hyderabad.

### 2. Transport Corridors & Grade Separations
We compile three major intercity arterial corridors:
1. **Bengaluru–Mysuru Expressway (NH 275)**:
   - 10-lane access-controlled expressway spanning Bengaluru Kengeri $\to$ Bidadi $\to$ Ramanagara $\to$ Channapatna $\to$ Maddur $\to$ Mandya $\to$ Srirangapatna $\to$ Mysuru Columbia Asia Junction.
   - Kaveri River Bridge: Elevated Layer 1 bridge over Kaveri River (Layer 0) at Srirangapatna.
   - South Western Railway parallel double-track mainline.
2. **Bengaluru–Chennai Economic Highway (NH 48 / NE 7)**:
   - Major corridor traversing the Eastern Ghats: Bengaluru KR Puram $\to$ Hoskote $\to$ Kolar $\to$ Chittoor $\to$ Ranipet (Palar River bridge) $\to$ Sriperumbudur $\to$ Poonamallee $\to$ Chennai Central.
   - Grade transitions from 920m plateau to coastal plain with zero topological tears.
3. **Bengaluru–Hyderabad Highway (NH 44 - North-South Expressway)**:
   - High-speed 4/6-lane expressway: Bengaluru Hebbal $\to$ Devanahalli (KIA) $\to$ Chikballapur $\to$ Anantapur $\to$ Kurnool (Tungabhadra & Krishna River bridges) $\to$ Shamshabad (RGIA) $\to$ PVNR Elevated Expressway $\to$ Hyderabad Charminar.

All highway crossings and bridges are validated against topological invariants: `layer >= 1`, `is_bridge: true`, clearance $\ge 5.5\text{m}$, and zero planar intersections with underlying waterways, railways, or local roads.

### 3. Scalable Regional Packaging (`RegionalCatalog`)
Regional world packages are isolated and self-contained:
- Each package contains:
  - `manifest.json`: Package version, region ID, bounds, SHA-256 checksums, timestamps, source provenance.
  - `chunks.db`: SQLite spatial index indexing all binary chunks within the region.
  - `chunks/*.rec`: Compressed binary chunk files (`zstd` compressed `WorldChunk`).
- `RegionalCatalog`:
  - Discovers installed packages in `packages/<region_id>/`.
  - Supports incremental install, upgrade, and removal.
  - Verifies package integrity via SHA-256 before activation.
- `MultiRegionStreamer`:
  - Maintains spatial bounding boxes of all active installed regional packages.
  - Dispatches spatial chunk queries to the appropriate regional SQLite database(s).
  - Handles border zones by loading chunks from multiple adjacent regional packages concurrently.
  - Bounds total resident memory under 1024 MB via global LRU cache eviction.

## Consequences
- Long-distance road navigation routes (e.g. Vidhana Soudha to Mysore Palace, or Vidhana Soudha to Chennai Central) can be resolved continuously via A* graph pathfinding.
- New regions can be downloaded, installed, or compiled independently without modifying or recompiling the rest of the world.
- Memory usage remains strictly bounded regardless of journey distance, as distant chunks behind the vehicle are evicted cleanly by the LRU cache.
- Full backwards compatibility is preserved for existing single-region viewers and tests.
