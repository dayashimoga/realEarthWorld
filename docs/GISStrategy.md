# Real Earth World Platform — GIS Strategy

## 1. Geometric & Geodetic Reference Standards

All spatial calculations in Real Earth adhere strictly to International Geodetic Standards:
- **Horizontal Reference Datum**: WGS84 (World Geodetic System 1984, EPSG:4326).
  - Semi-major axis $a = 6,378,137.0\text{ meters}$.
  - Flattening $f = 1 / 298.257223563$.
  - Semi-minor axis $b = a(1 - f) \approx 6,356,752.314245\text{ meters}$.
  - First eccentricity squared $e^2 = \frac{a^2 - b^2}{a^2} = 2f - f^2 \approx 0.00669437999014$.
- **Geocentric Cartesian Coordinates**: Earth-Centered Earth-Fixed (ECEF, EPSG:4978).
  - Triple $(X, Y, Z)$ in SI meters with double-precision `f64`.
- **Local Tangent Plane**: East-North-Up (ENU).
  - Defined by an anchor origin $(\phi_0, \lambda_0, h_0)$ in WGS84.
  - Used for regional chunk generation, physics simulations, and collision detection within local tangent domains.
- **Vertical Reference Datum**:
  - Raw GPS coordinates use WGS84 ellipsoidal height.
  - Real Earth tracks vertical datum metadata, distinguishing between Ellipsoidal Height ($h$) and Orthometric Height ($H$ above geoid / mean sea level), using the relationship $h = H + N$, where $N$ is geoid undulation from EGM96/EGM2008 models.

---

## 2. Ingestion & Feature Attribution

### 2.1 Road Networks (OpenStreetMap)
- Roads are extracted from OSM `way` elements with `highway` tags.
- Road categories mapped to standard lane widths and speed profiles:
  - `motorway` / `trunk`: 3.75m per lane, multi-lane divided.
  - `primary` / `secondary`: 3.5m per lane.
  - `tertiary` / `residential`: 3.0m per lane.
  - `service` / `track`: 2.5m per lane.
- Vertical Layering & Grade Separation:
  - `layer`: Integer value (default `0`). Negative values indicate subterranean or underpasses; positive values indicate overpasses/bridges.
  - `bridge`: `yes` / `viaduct` indicates elevated structural road ribbons.
  - `tunnel`: `yes` indicates subterranean or covered tubes.
  - Rule: Intersecting ways with different `layer` attributes or `bridge`/`tunnel` mismatches DO NOT form an intersection. They must maintain distinct vertical clearances (minimum 5.0m clearance for road overpasses).

### 2.2 Building Footprints & Height Attribution
- Building polygons are parsed from closed OSM ways and relations (`type=multipolygon`).
- Height attribution policy:
  1. **Surveyed Truth**: If `height` tag exists (e.g. `height=42.5` or `height=42.5m`), height is extracted directly with provenance `HeightProvenance::Surveyed`.
  2. **Inferred from Levels**: If `building:levels` tag exists, height is computed as $\text{levels} \times 3.5\text{m} + 1.0\text{m}$ (roof parapet) with provenance `HeightProvenance::InferredLevels`.
  3. **Default Category**: If neither is present, inferred by building category (`residential` $\to 9.0\text{m}$, `commercial` $\to 15.0\text{m}$, default $\to 8.0\text{m}$) with provenance `HeightProvenance::DefaultCategory`.
- Every building in the world package carries this provenance flag, ensuring visual inference is never falsely represented as surveyed reality.

### 2.3 Terrain Elevation Integration (Copernicus DEM / NASA SRTM)
- Raster DEM tiles store elevation samples on regular geographic grids (e.g. $1\text{ arc-second} \approx 30\text{m}$).
- Continuous elevation query at coordinate $(\phi, \lambda)$ is evaluated via bilinear or bicubic interpolation:
  $$h(\phi, \lambda) = \sum_{i=0}^1 \sum_{j=0}^1 w_{ij} h(i, j)$$
- Road networks and building foundations sample DEM elevation at their centroid or along polyline vertices to guarantee physical ground contact.
