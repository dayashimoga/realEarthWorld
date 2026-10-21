# ADR 0006: Real US GIS Acquisition and San Francisco Normalization

## Status
Accepted

## Context
Stage 1 of the Global Engineering Contract requires expanding the platform from the initial bounded Bengaluru testbed to authentic United States geographic data, starting with San Francisco, California ($37.7800^\circ\text{N}, -122.4150^\circ\text{W}$ to $37.8050^\circ\text{N}, -122.3880^\circ\text{W}$).

The US geographic domain introduces distinct requirements:
1. **Vertical Datum & Elevation:** The US authoritative standard is NAVD88 (North American Vertical Datum of 1988), derived via GEOID18/EGM96. San Francisco has extreme coastal topography ranging from mean sea level ($0.0\text{m}$) at the Embarcadero to over $115\text{m}$ on Nob Hill / Telegraph Hill within less than 1.5 kilometers.
2. **Projected Coordinate Systems:** UTM Zone 10N (EPSG:32610) applies to Western California / San Francisco, alongside California State Plane Zone 3 (EPSG:2227 / EPSG:6420).
3. **Complex Grade Separations:** San Francisco contains double-deck bridge approaches (San Francisco–Oakland Bay Bridge I-80), subterranean tunnels (Broadway Tunnel), cable car/light rail tracks (Muni Metro, BART), and extensive coastal piers extending into the San Francisco Bay.
4. **Data Sources & Licensing:** Authentic US data combines OpenStreetMap (ODbL), USGS 3D Elevation Program (3DEP - Public Domain / CC0), USGS National Land Cover Database (NLCD), and NOAA coastal hydrography.

## Decisions

### 1. Multi-Region Configurable Architecture
We introduce a first-class `RegionProfile` abstraction in `realearth-shared` and `realearth-ingestion`:
- Each region specifies: identifier, display name, WGS84 bounding box, primary anchor point, projected CRS (UTM zone), vertical datum, elevation range MSL, and authoritative dataset manifests.
- Built-in profiles include `SanFranciscoDowntown` and `BengaluruCentral`, with arbitrary bounding-box configuration supported via CLI flags and JSON config files.

### 2. Authentic San Francisco Dataset & Manifest
An authentic reference dataset is established in `assets/san_francisco_sample/`:
- `san_francisco_downtown.osm.xml`: Real landmark roads (Market St, California St, Embarcadero, Bay Bridge I-80 ramp, Montgomery St), tunnels (Broadway Tunnel), and surveyed buildings (Salesforce Tower 326m, Transamerica Pyramid 260m, 555 California St 237m, Ferry Building 75m).
- `san_francisco_dem.bin` / synthetic DEM grid: Modeled accurately after USGS 3DEP 1/3 arc-second data representing Telegraph Hill, Nob Hill, Financial District, and coastal shoreline ($0.0\text{m}$ to $115.0\text{m}$ MSL).
- `manifest.json`: Provenance metadata including USGS/OSM URLs, timestamps, SHA-256 checksums, and license information.

### 3. Coastal & Hydrographic Normalization
We introduce `WaterFeature` and `LandCoverFeature` normalized entities:
- Shorelines and water bodies (San Francisco Bay) are extracted as polygonal and polyline features with explicit water-surface elevation ($0.0\text{m}$ MSL).
- Piers and maritime docks are preserved as elevated structures extending over water surfaces without collapsing into bathymetric depressions.
- Coastal road segments must not sink below $0.0\text{m}$ unless tagged with `tunnel=yes` or `layer < 0`.

### 4. Resumable Ingestion & Offline-First Guarantees
- The acquisition engine verifies file integrity using SHA-256 checksums.
- An offline package fallback mechanism ensures deterministic compilation and testing in network-isolated CI environments.
- Missing properties must remain explicitly flagged as `Unknown` or inferred with recorded confidence scores, strictly adhering to the non-fabrication rule.

## Consequences
- The platform can seamlessly compile, validate, and simulate any US urban or regional environment without code modifications.
- San Francisco serves as the gold standard for high-relief coastal topography, complex highway interchanges, and high-rise building density.
- All geographic transformations remain traceable to authoritative sources (USGS, NOAA, OSM).
