# ADR 0004: GIS Validation, Grade Separation and Quarantine Policy

## Status
Accepted

## Context
Raw crowd-sourced GIS datasets like OpenStreetMap contain real-world defects including self-intersecting polygon rings, collinear points, unclosed boundaries, and missing height attributes. Naive GIS importers often perform destructive repairs (e.g., arbitrarily merging vertices or collapsing vertical layers) that destroy real-world physical structures like multi-level flyovers or bridges.

## Decision
1. **Grade Separation Invariance**: Elevated expressways, bridges, and flyovers tagged with `layer > 0` or `bridge = yes` must never be topologically welded to ground-level roads simply because their 2D projections intersect.
2. **Safe & Traceable Repairs**: Geometric repairs (such as counter-clockwise ring orientation or duplicate vertex removal) must log an audit record with an assigned confidence score ($0.0 \to 1.0$).
3. **Quarantine Rather Than Fabricate**: Unrecoverable geometries (e.g. invalid self-intersecting polygons that cannot be resolved without geographic ambiguity) must be routed to a quarantine log (`quarantine_features`) rather than fabricated or hallucinated.
4. **Height Provenance**: Heights are tagged with explicit provenance flags: `Surveyed` (explicit metadata), `InferredLevels` (levels count $\times 3.5\text{m}$), or `DefaultCategory` ($8.0\text{m}$ default).

## Consequences
- Guaranteed preservation of complex grade-separated urban intersections (e.g. Bangalore Elevated Tollway, Hebbal Flyover).
- Absolute auditability of all geometry modifications.
- High reliability: corrupted external data cannot crash the world compiler or physics engine.
