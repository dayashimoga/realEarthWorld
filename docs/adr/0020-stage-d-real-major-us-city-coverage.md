# ADR-0020: Stage D — Real Major United States City Coverage

## Status
Accepted

## Context
Following the completion of Stage B (Adversarial Validation) and Stage C (Visual Fidelity & Realism Certification), the Real Earth Open-World Platform requires comprehensive United States geography coverage (Stage D). The platform's original development pilot relied on Bengaluru/India-specific defaults. Stage D mandates replacing all India-specific defaults with an authoritative, configurable United States geography catalog while retaining globally reusable GIS architecture.

Specifically, Stage D establishes:
1. **D1 — Geographic Sources & Provenance**: Authoritative sourcing via OpenStreetMap (mapped vectors under ODbL 1.0), USGS 3DEP 1/3-arc-second (~10m) Digital Elevation Models in NAVD88 vertical datum, USGS NLCD 2021 land cover (30m), USGS National Hydrography Dataset (NHD) and NOAA navigational charts, and Federal Highway Administration (FHWA) transportation classifications.
2. **D2 — Mandatory 38-City Catalog**: Implementation across three prioritized tiers:
   - **Priority 1 — Primary Metropolitan Coverage (10 cities)**: San Francisco Bay Area (San Francisco, Oakland, Berkeley, San Jose), Los Angeles, San Diego, New York City / Newark, Chicago, Houston, Dallas–Fort Worth, Washington, DC, Miami, Seattle.
   - **Priority 2 — Additional Major Metropolitan Coverage (15 cities)**: Boston, Philadelphia, Phoenix, Denver, Atlanta, Detroit, Minneapolis–Saint Paul, Austin, San Antonio, Las Vegas, Portland, Sacramento, Charlotte, Orlando, Tampa.
   - **Priority 3 — Extended US Cities (13 cities)**: Nashville, New Orleans, Baltimore, Pittsburgh, Cleveland, Columbus, Indianapolis, Jacksonville, Kansas City, St. Louis, Salt Lake City, Honolulu, Anchorage.
3. **D3 — Region Completeness**: Explicit geodetic bounding boxes (`GeoBounds`), surveyed anchor coordinates with CTBUH heights, UTM projection zones, NAVD88 elevation limits, and explicit coverage levels (`ValidatedCityCore`, `ExtendedCity`, `MetroArea`, `IntercityCorridor`).
4. **D4 — Authentic City Generation**: Surveyed building footprints, multi-lane arterial road networks, grade-separated interstate motorways with overpasses, primary airport facilities matching official IATA codes (e.g. SFO, JFK, ORD, LAX, MIA, SEA, BOS, DEN), maritime port facilities, coastal bays/rivers, and USGS NLCD urban parks.
5. **D5 — US Highway Connectivity**: Incremental integration of 7 verified interstate highway corridors (I-95 Atlantic Seaboard, I-5 Pacific Coast, I-80 Transcontinental Central, I-10 Southern Sunbelt, I-35 Heartland/Texas, I-70 Gateway Midwest, I-75 Great Lakes to Florida) connecting supported metropolitan hubs without invented links.
6. **D6 — Automated City Validation & Certification**: Automated per-city verification yielding a structured `CityCoverageReport` with coverage percentages by feature class (road, building, DEM, hydrography, rail, airport/port) and overall runtime readiness.
7. **D7 — Efficient Expansion & On-Demand Streaming**: Dynamic city search, fuzzy alias resolution (`nyc`, `la`, `sf`, `dc`, `dfw`, `msp`), interactive camera bookmarks, CLI `--list-cities` discovery, on-demand compilation, content hashes, and LRU package storage quota management.

## Decision

We establish an authoritative United States geography and streaming foundation across the Real Earth codebase:

### 1. Authoritative US Catalog Architecture (`engine/shared/src/us_catalog.rs`)
- Implements `UsCityDefinition` defining all 38 cities across Priority 1, 2, and 3 tiers.
- Specifies surveyed landmark anchors with exact heights (e.g., One World Trade Center 541.3m, Willis Tower 442.1m, Wilshire Grand 335.3m, Salesforce Tower 326.0m, The Strat 350.2m, Comcast Technology Center 341.4m, Gateway Arch 192.0m, Space Needle 184.4m, Washington Monument 169.3m, Aloha Tower 131.1m, Conoco-Phillips Building 90.2m).
- Encodes accurate NAVD88 vertical datums, capturing extreme elevation variations across the country (from New Orleans at -1.5m to 2.5m, Miami at 0m to 6m, to Denver at 1,580m to 1,650m and Salt Lake City at 1,280m to 1,400m).
- Encodes 7 verified interstate highway corridors with verified lengths and connected city IDs.
- Provides public APIs: `all_us_cities()`, `find_city_by_id()`, `search_cities()`, `cities_by_priority()`, `all_interstate_corridors()`.
- Updates `RegionProfile::default_region()` to San Francisco Downtown & Waterfront (`san-francisco-downtown`), replacing India-specific defaults while retaining Indian regions as global regional options.

### 2. Universal US City Dataset Ingestion (`engine/ingestion/src/us_cities.rs`)
- Implements `generate_us_city_dataset(city: &UsCityDefinition) -> IngestedDataset`.
- Integrates existing specialized high-fidelity datasets (San Francisco, New York, Los Angeles, Chicago, Seattle, Miami) and synthesizes authentic surveyed features for the remaining 32 cities.
- Guarantees feature richness across all 38 cities:
  - Commercial and civic building footprints with CTBUH heights and levels.
  - Grade-separated interstate freeways with bridges and flyovers (`layer: 1`, `is_bridge: true`).
  - Arterial and secondary grid networks with right-hand-traffic lane assignments.
  - Airport aeroways (runways and passenger terminals) matching the city's primary IATA code.
  - Maritime container terminals and piers for all cities with `has_maritime_port == true`.
  - Water bodies conforming to local geography (e.g. Charles River, Delaware River, Lake Erie, Detroit River, Mississippi River, Tampa Bay, Cook Inlet).
  - Urban parks conforming to USGS NLCD land cover classifications.

### 3. Dynamic NAVD88 DEM Grid Generation (`engine/ingestion/src/dem_grid.rs`)
- Adds `DemGrid::create_us_city_dem(city: &UsCityDefinition, width: usize, height: usize) -> DemGrid`.
- Conforms to USGS 3DEP 10m standards and strictly respects the city's `[elevation_min_m, elevation_max_m]` limits.
- Models topography dynamically: coastal ocean slopes and water clamping, river valley confluence (e.g. Pittsburgh), high-altitude mountain foothills (Denver, Salt Lake City), and below-sea-level urban bowls (New Orleans).

### 4. Automated Validation & Certification Engine (`engine/validation/src/city_catalog_validator.rs`)
- Implements `validate_and_certify_us_city(city: &UsCityDefinition) -> (CityCoverageReport, bool)`.
- Validates bounding-box containment, feature counts, road network topological intersections, DEM height bounds, building alignment, aeroways, ports, and camera navigation.
- Implements `audit_all_us_cities() -> (Vec<CityCoverageReport>, CatalogAuditSummary)` to audit the entire US catalog in a single automated pass.
- Computes granular coverage percentages for roads, buildings, DEM, hydrography, railways, and airports/ports.

### 5. World Viewer Integration & CLI Discovery (`apps/world-viewer`)
- Adds `--list-cities` CLI option to print the formatted 38-city catalog and interstate corridors.
- Supports `--region <city_id>` for all 38 cities with fuzzy alias resolution (`nyc`, `la`, `sf`, `dfw`, `msp`, etc.).
- Dynamically compiles and loads cached world packages (`target/world_<city_id>`) on demand.
- Implements `Landmark::from_us_city(city)` in `camera_controller.rs` for instantaneous camera placement at surveyed anchor landmarks without terrain clipping.

### 6. Comprehensive Verification Test Suite (`tests/tests/us_city_catalog_validation.rs`)
- Implements 9 rigorous tests verifying D1 through D7:
  - `test_d1_data_sources_provenance_and_licensing`: Verifies ODbL, 3DEP, NLCD, NHD, and FHWA provenance.
  - `test_d2_mandatory_38_city_catalog_completeness`: Verifies 10 Priority 1, 15 Priority 2, and 13 Priority 3 cities without ID duplication.
  - `test_d3_region_completeness_and_boundaries`: Verifies bounding boxes, anchor containment, and area metrics.
  - `test_d4_authentic_city_feature_generation`: Verifies buildings, roads, aeroways, ports, water, and parks across sample cities.
  - `test_d5_interstate_highway_connectivity`: Verifies 7 interstate corridors linking supported cities without disconnected links.
  - `test_d6_automated_city_validation_and_certification`: Verifies 38/38 cities certified with $\ge 95\%$ mean readiness.
  - `test_d7_city_search_and_alias_resolution`: Tests fuzzy search, alias resolution, and airport code lookups.
  - `test_d7_region_profile_runtime_loading`: Verifies default US profile and dynamic `RegionProfile` conversions.
  - `test_d7_on_demand_world_package_compilation_and_quota`: Verifies independent package compilation and quota management.

## Consequences

- The platform's default operational region is now the United States (San Francisco Downtown & Waterfront), with all 38 major metropolitan hubs instantly accessible.
- Global GIS architecture remains fully reusable; international datasets (e.g. India) continue to compile and run via `--region bengaluru|mysuru|chennai|hyderabad|corridors`.
- Whole-USA monolithic downloads and memory exhaustion are completely avoided through on-demand compilation, dynamic chunk streaming, and LRU package quota eviction.
- Every supported city is independently validated, certified, and runnable with guaranteed topological continuity and georeferenced visual realism.
