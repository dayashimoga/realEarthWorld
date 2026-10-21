# ADR 0008: Comprehensive GIS Validation and Controlled Repair Architecture

## Status
Accepted

## Context
Stage 4 of the Global Engineering Contract requires expanding the validation and repair foundation of Real Earth into an industrial-strength, comprehensive geospatial verification and controlled repair engine. Real-world OpenStreetMap and government GIS datasets inevitably contain topological and geometric flaws: duplicate nodes, open rings, inverted polygon winding orders, self-intersecting geometries, disconnected road stubs, conflicting grade separations, building foundation anomalies on steep DEM slopes, water polygon inconsistencies, and railway/airport/maritime topological defects.

Under our mission (World First -> Simulation Second -> Gameplay Third), the platform must guarantee that:
1. **Zero Data Fabrication:** The engine must never invent roads, buildings, or infrastructure where none exist in source data.
2. **Deterministic, Confidence-Scored Repair:** Corrections are applied only when mathematically justified (e.g., closing an unclosed ring where endpoints match within threshold, culling duplicate consecutive vertices, correcting winding order to CCW). Every repair must be recorded in an audit trail with before/after geometry snapshots and a confidence score ($0.0$ to $1.0$).
3. **Strict Quarantine of Ambiguous Defects:** Defects that cannot be safely resolved with high confidence ($\ge 0.80$) must be quarantined to isolate them from downstream world compilation, accompanied by explicit failure classifications (`Blocker`, `Error`, `Warning`, `Info`).
4. **Preservation of Authentic Geometry:** Correct, valid source features must pass through bit-exact and unchanged.
5. **Multi-Format Diagnostics & Publication Gates:** Validation output must support machine-readable JSON, human-readable Markdown, styled HTML dashboards, visual error overlay coordinates for the 3D viewer, and strict compiler publication gates that reject compilations with unresolved blocking errors.

## Decisions

### 1. Granular Deep Validators (`engine/validation`)
We organize deep validation into dedicated, modular domain validators:
- **`RoadNetworkValidator`:** Evaluates road network connectivity (detecting dead-end stubs vs connected graphs), lane count consistency ($\ge 1$), speed limit ranges ($5 \le v \le 160$ km/h), one-way road flow integrity, ramp merge angles, hairpin turn angles, and grade-separation crossings (bridges `layer > 0` vs ground `layer 0` vs tunnels `layer < 0`).
- **`RailwayValidator`:** Evaluates railway continuity, minimum curve radius, track gauge consistency, and grade separation when crossing highways (demanding explicit level crossings or vertical layer separation).
- **`BuildingDeepValidator`:** Evaluates self-intersections (bow-tie polygons), inter-building footprint overlaps ($> 5\%$ area collision), minimum footprint threshold ($4.0\text{ m}^2$), and foundation elevation feasibility against underlying DEM slopes ($< 45^\circ$) and coastal water levels ($\ge 0.0\text{m}$ MSL).
- **`TerrainWaterValidator`:** Evaluates DEM raster continuity, detecting step discontinuities (terrain cracks) between adjacent grid cells ($\Delta z / \Delta d \le 1.5$), and verifies water polygon closure and elevation consistency.
- **`AerowayPortValidator`:** Evaluates runway centerline continuity, minimum runway length/width, taxiway junctions, and maritime pier/dock connections to coastlines with positive freeboard above water.
- **`ChunkIntegrityValidator`:** Verifies coordinate continuity across chunk borders, ensuring inter-chunk features connect seamlessly without micro-gaps ($< 1\text{mm}$).

### 2. Controlled Reversible Repair Engine (`repair_engine.rs`)
We introduce an explicit `RepairEngine` with reversible operations:
- Every applied correction produces a `ControlledRepair` struct containing:
  - `feature_id: u64`
  - `feature_type: FeatureKind`
  - `rule_id: String` (e.g. `RULE_CCW_WINDING`, `RULE_CLOSE_RING`, `RULE_CULL_DUPLICATES`, `RULE_SNAP_JUNCTION`)
  - `confidence_score: f32`
  - `before_geometry: GeometrySnapshot`
  - `after_geometry: GeometrySnapshot`
  - `is_reversible: bool`
- If downstream validation fails or conflicts with an adjacent feature, the repair engine can roll back the transaction.
- If confidence is $< 0.80$, repair is rejected and the feature is quarantined.

### 3. Hierarchical Diagnostic Classifications & Multi-Format Reporting
- **Severities:**
  - `Blocker`: Fatal error halting compilation (corrupted CRS, NaN coordinate, zero-vertex geometry).
  - `Error`: Feature-level defect causing quarantine (unresolvable self-intersection, overlapping footprint conflict).
  - `Warning`: Non-fatal defect safely repaired with high confidence ($\ge 0.80$).
  - `Info`: Non-geometric metadata inference (e.g. height inferred from building levels).
- **Reports:**
  - `report.to_json()`: Complete machine-readable audit trail.
  - `report.to_markdown()`: GitHub-flavored documentation table.
  - `report.to_html()`: Interactive HTML dashboard with KPI cards and severity-badged tables.
  - `report.to_overlay()`: Geolocated markers for 3D viewer debug visualization.
- **Publication Gate:**
  - `CompilerPublicationGate::evaluate(&report)` enforces that zero `Blocker` defects exist and quarantined `Error` count does not exceed the configurable release threshold ($0$ for strict production).

## Consequences
- The platform can process imperfect, dirty, real-world OpenStreetMap or governmental GIS files without crashing or producing corrupt geometry.
- Authentic geometry remains 100% unaltered, satisfying the global engineering contract.
- Compilation publication gates prevent bad data from reaching the runtime simulation or streaming cache.
