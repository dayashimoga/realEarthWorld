# ADR 0015: Earth-Scale World Infrastructure

## Status
Accepted

## Context
Following the completion of Stage 10 (Optional Authoritative Multiplayer), Real Earth requires scaling from regional interconnected corridors (San Francisco, Bengaluru, Mysuru, Chennai, Hyderabad) to true **Earth-Scale World Infrastructure** (Stage 11).

Key engineering challenges:
1. **No Full-Earth Downloads**: Storing the entire planet at centimeter resolution would require hundreds of terabytes of storage. The engine must support on-demand, regionalized acquisition and caching.
2. **Coordinate Precision & Continental Geodesics**: Traditional planar projections distort at continental scales. The engine must support global WGS84 geodetic coordinates, high-precision ellipsoidal geodesic solvers (Vincenty and Haversine), antimeridian wraparound continuity, and automatic UTM/national CRS conversions.
3. **Storage Quotas & Multi-Tier Cache**: Memory and disk cannot grow unbounded. A multi-tier cache hierarchy (L1 RAM chunk cache, L2 local SSD package storage, L3 remote/CDN) must enforce storage quotas and LRU package eviction while protecting active and collision-pinned regions.
4. **Resumable Delivery & Content Integrity**: Regional packages must be downloadable on-demand with resumable chunk-range requests, cryptographic SHA-256 hash trees, and schema migration compatibility.
5. **Missing-Data Diagnostics & No Fabrication Rule**: The world has uneven mapping density. The engine must detect data voids (missing DEM, roads, or buildings) and fall back to low-resolution planetary basemaps with explicit provenance labeling, strictly avoiding fabricated geometry.
6. **Planetary Scalability (10M Managed Objects)**: The streaming and spatial indexing infrastructure must scale to 10,000,000 managed spatial objects while keeping memory bounded (< 1024 MB RSS) and maintaining 60 FPS reference performance.

## Decisions

### 1. Planetary Hierarchical Spatial Indexing (`PlanetarySpatialIndex`, `GlobalCellKey`)
- Implement a Discrete Global Grid System (DGGS) based on hierarchical geodetic quadkeys across zoom levels 0 (Planetary Root: $[-180, 180] \times [-90, 90]$) to 18 (sub-meter chunk).
- Support bit-packed quadkeys and base-4 string representations for efficient hierarchical parent-child traversal, bounding box queries, and spatial hashing.
- Handle antimeridian crossings ($[-180^\circ, +180^\circ]$) and polar singularities seamlessly without coordinate wrapping discontinuities.

### 2. High-Precision Planetary Geodesics & Multi-CRS Projection Registry
- Implement Vincenty's high-precision inverse geodesic solver on the WGS84 reference ellipsoid ($< 0.5\text{ mm}$ error across continental distances) with spherical Haversine fallback.
- Support universal UTM auto-zoning across all 60 longitudinal zones for both Northern and Southern hemispheres (EPSG:32601..32660 and EPSG:32701..32760).
- Integrate an extensible CRS registry including EPSG:4326 (WGS84), EPSG:3857 (Web Mercator), EPSG:27700 (British National Grid), EPSG:2154 (Lambert-93 France), EPSG:25832 (ETRS89 Germany), EPSG:6677 (JGD2011 Japan), and EPSG:7755 (India National Grid).
- Standardize vertical datum models: WGS84 Ellipsoidal, EGM96 Geoid, EGM2008 Geoid, NAVD88, EVRF2019, and Mean Sea Level (MSL).

### 3. Worldwide Source Provenance, Licensing & Quality Validation
- Define `GlobalDataSourceManifest` capturing dataset provenance, provider identities, update frequencies, and license terms (OpenStreetMap ODbL 1.0, Copernicus GLO-30/90, USGS 3DEP, NASA SRTM, ESA WorldCover CC-BY-4.0, UK OS OpenData, IGN France, GSI Japan).
- Enforce strict license attribution isolation and quarantine policies.
- Define region-specific quality validation profiles assessing DEM vertical resolution, slope continuity, building height confidence, and coastal boundary closure.

### 4. Independent Versioned Country/Region Packages & Resumable Delivery
- Package world regions into independent, self-contained packages (`CountryPackage`, `RegionalPackage`) containing metadata manifests, SQLite spatial chunk indexes (`chunks.db`), zstd-compressed binary chunk meshes (`.rec`), and license attributions.
- Implement `ResumableDownloadSession` supporting chunk-range acquisition, interruption recovery, and SHA-256 checksum verification.
- Implement `StorageQuotaManager` with configurable disk quotas (e.g., 50 GB), disk storage accounting, and LRU eviction of unpinned inactive packages when storage budgets are exceeded.
- Support clean, atomic installation and uninstallation of individual regions and countries without engine restart or neighbor disruption.

### 5. Missing-Data Diagnostics & Planetary Basemap Fallback
- Implement `MissingDataDiagnostics` to identify spatial voids in elevation, transportation, and building coverage.
- Provide `FallbackBasemapProvider` producing coarse planetary basemap terrain and coastal boundaries labeled explicitly with `FeatureProvenance::PlanetaryBasemapFallback`.
- Strictly enforce the "No Fabrication Rule": never fabricate missing buildings or road connections.

### 6. Planetary Scalability & 10M Managed Objects Benchmark
- Implement adaptive global-to-local LOD streaming: Level 0 (Global Planetary Ellipsoid, $> 100\text{ km}$ altitude) down to Level 4 (Street/Building scale, $< 1\text{ km}$).
- Implement simulation relevance budgets prioritizing streaming for high-speed aviation ($20\text{ km}$ prefetch cone) vs ground vehicles ($2\text{ km}$ cone).
- Implement dynamic planetary floating origin re-anchoring when camera moves $> 50\text{ km}$ from the current anchor, preserving 32-bit GPU floating-point precision ($< 1\text{ mm}$ error).
- Automate world package certification via `WorldPackageCertifier`.
- Demonstrate scalability by indexing, querying, and streaming 10,000,000 managed spatial objects within $< 1024$ MB RSS and maintaining $\ge 60$ FPS / high headless simulation rate.

## Consequences
- New countries and metropolitan areas worldwide can be added incrementally without architectural changes or full-Earth downloads.
- Storage and memory remain strictly bounded under all operating conditions.
- Geographic truth and licensing compliance are preserved across global boundaries.
- Offline single-player and standalone operations remain fully functional.
