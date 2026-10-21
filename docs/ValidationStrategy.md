# Real Earth World Platform — Validation & Repair Strategy

## 1. Principles of Geometric & Topological Integrity

In accordance with Section 3 and Section 7 of the Global Engineering Contract:
1. **Never fabricate geographic facts.**
2. **Never silently drop or hide geometric defects.**
3. **Repairs must be safe, traceable, confidence-aware, and mathematically reversible.**
4. **Unresolvable defects must be isolated into a structured quarantine log.**

---

## 2. Validation Rules & Acceptance Criteria

### 2.1 Road Topology Rules
- **Rule R1 (Vertex Snapping & Junction Integrity)**: Road segments sharing a geographic junction within tolerance ($\epsilon = 0.05\text{m}$) must reference the exact same node ID. Spurious disjoint endpoints are repaired by topological node welding if and only if both ways have identical layer tags.
- **Rule R2 (Grade Separation Invariance)**:
  - Two road segments intersecting in 2D projection ($x, y$) that have different `layer` values (e.g. Layer 0 vs Layer 1) or where one is tagged `bridge=yes` and the other is at ground level MUST NOT share a node.
  - Grade separated crossings must maintain a minimum physical vertical clearance of $4.5\text{m}$ (urban overpasses) or $5.5\text{m}$ (rail/highway flyovers).
  - Any automatic repair that collapses a bridge onto a ground road is treated as a fatal defect.
- **Rule R3 (No Degenerate Road Segments)**: Segments with length $\le 0.1\text{m}$ or consecutive duplicate vertices are culled, logging a traceable repair event.

### 2.2 Building Polygon Rules
- **Rule B1 (Closure & Non-Self-Intersection)**: Building polygon exterior rings must be closed (first vertex == last vertex) and have $\ge 3$ distinct vertices. Polygons with self-intersecting boundaries (bowtie polygons) must either be split into valid sub-polygons using the Even-Odd / Non-Zero winding rule or quarantined.
- **Rule B2 (Winding Order)**: Exterior rings must be counter-clockwise (CCW); interior holes must be clockwise (CW). Inverted rings are automatically reversed and flagged with `RepairedWindingOrder`.
- **Rule B3 (Collinear Edge Simplification)**: Collinear vertices along straight building walls ($\text{area} = 0$) within $\epsilon = 0.01\text{m}$ tolerance are simplified to reduce draw overhead while preserving footprint area.
- **Rule B4 (Minimum Building Area)**: Footprints with area $< 4.0\text{ m}^2$ are tagged as micro-structures or quarantined if degenerate.

---

## 3. Provenance & Quarantine Logging

Every validation run outputs a detailed `ValidationReport`:
- **Total Features Evaluated**
- **Valid Features Passed**
- **Repaired Features**: Logged with `feature_id`, `repair_type`, `original_geometry`, `repaired_geometry`, and `confidence_score` ($0.0 \to 1.0$).
- **Quarantined Features**: Logged with `feature_id`, `failure_reason`, and `raw_geojson`.

Quarantined features are saved to a separate database table (`quarantine_features`) and excluded from production runtime compilation packages to prevent physics or rendering crashes.
