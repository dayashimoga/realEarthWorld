# ADR-0018: Stage B — Adversarial and Independent Validation Framework

## Status
Accepted

## Context
Following the completion of Stages 0–12 and the forensic audit of the Real Earth Open-World Platform, we require an independent, adversarial validation phase (Stage B). Previously, test suites primarily validated the internal consistency of engine subsystems against engine-generated expectations. While these tests confirmed regression prevention, true production certification requires:
1. Validating engine outputs against **trustworthy, independently retrieved authoritative test oracles** (authoritative geodetic coordinates, standard closed-form mathematics, surveyed architectural records, USGS/Copernicus elevation benchmarks).
2. Deepening verification through **property-based invariants, metamorphic relations, negative boundary testing, randomized inputs, fuzzing, stress, and soak workloads**.
3. Enforcing rigorous **GIS topological invariants**, including coordinate datum transformations, drive-side conventions across regions, grade-separated vertical clearances, and seam continuity.
4. Validating compiler reproducibility, deterministic chunk hashing, and resilient streaming under corruption.
5. Verifying physical realism across vehicle classes (automotive, train, marine, aviation) and authoritative simulation contracts.
6. Executing multi-region user journeys simulating real-world gameplay and network lifecycle under normal and interrupted conditions.
7. Practicing **honest evaluation** by injecting deliberate defects into acceptance fixtures to prove validators fail when implementations are subtly incorrect, while explicitly recording untestable/unsupported edge conditions.

## Decision
We implement a dedicated, reproducible adversarial validation harness in `tests/tests/adversarial_validation.rs` covering all seven Stage B pillars:

1. **B1 — Trustworthy Test Oracles**:
   - Closed-form Vincenty and Haversine models implemented strictly independently from engine libraries to serve as non-self-referential ground truth.
   - Authoritative geodetic baselines (Royal Observatory Greenwich to Paris Observatory, NYC One WTC to Empire State Building, equatorial 1-degree arc length).
   - Authoritative landmark architectural height catalog (One WTC 541.3m, Willis Tower 442.1m, Empire State 381.0m, Chrysler 319.0m, Columbia Center 284.0m, Space Needle 184.4m, Wilshire Grand 335.3m, Panorama Tower 264.6m).
   - Reference elevation points (Denver 1609.3m, Bengaluru 920.0m, Twin Peaks 282.0m, Seattle Waterfront 38.0m, Miami coastal MSL).
   - Independent Dijkstra graph pathfinding oracle verifying engine A* optimality and distance fidelity.

2. **B2 — Test Depth & Adversarial Rigor**:
   - Metamorphic Relation 1 (Routing Symmetry): In bidirectional road topologies, $d(A, B) \equiv d(B, A)$.
   - Metamorphic Relation 2 (Translation Invariance): Uniform ENU coordinate shifts preserve all pairwise Euclidean distances.
   - Metamorphic Relation 3 (Polygon Area Invariance): Cyclic shifts of polygon vertex order preserve the Shoelace area.
   - Metamorphic Relation 4 (Scale Monotonicity): Doubling speed on a route halves the transit time.
   - Boundary & Negative Testing: Latitudes outside $[-90^\circ, +90^\circ]$, longitudes outside $[-180^\circ, +180^\circ]$, NaN coordinates, degenerate 1-vertex ways, bow-tie self-intersections.
   - Fuzzing & Corruption: Feeding malformed byte streams into network packet decoders and corrupted headers into chunk loaders.
   - Stress & Soak: Rapid-burst spatial queries and sustained 1000-tick physics loops under peak load.

3. **B3 — GIS Integrity & Regional Conventions**:
   - WGS84 $\leftrightarrow$ ECEF round-trip error must be $< 10^{-6}\text{ m}$.
   - Regional drive-side verification: US regions (NY, LA, Chicago, Seattle, Miami) enforce Right-Hand Drive (`DriveSide::RightHandDrive`), while India (Bengaluru) enforces Left-Hand Drive (`DriveSide::LeftHandDrive`).
   - Grade-separated bridge overpass vertical separation ($\ge 6.0\text{ m}$) without coplanar node collapse.

4. **B4 — World Compiler & Streaming Integrity**:
   - Bit-exact chunk recompile reproducibility and cryptographic SHA-256 validation.
   - Storage quota and LRU cache eviction under simulated memory pressure.
   - Bit-flip corruption detection and recovery.

5. **B5 — Vehicle Dynamics & Multimodal Physics**:
   - Kinetic energy braking distance ($d \approx v^2 / (2 \mu g)$).
   - Slope gravity reaction force and hill-holding equilibrium.
   - Multimodal physics invariants (aircraft aerodynamic stall, marine buoyancy displacement, train Davis resistance and ATP braking).
   - Traffic car-following IDM headway safety.
   - Save/load game profile serialization round-trip fidelity.

6. **B6 — End-to-End User Journeys**:
   - Multi-step cross-regional user journey across New York and Seattle with ground driving, bridge crossing, maritime vessel switching, day/night solar cycle, and UDP multiplayer state sync.

7. **B7 — Honest Evaluation & Deliberate Defect Injection**:
   - Fixtures with intentionally broken bridge elevations, corrupted topological edges, invalid packet timestamps, and out-of-range velocities ($> 500\text{ m/s}$) must cause publication gates and validators to fail with Blocker diagnostics.
   - Explicit recording of hardware-untestable conditions (physical GPU monitor VSync sync signals and live orbital GPS satellite receiver locks).

## Consequences
- Every subsystem is evaluated against external, mathematical, and physical ground truth.
- High test coverage and confidence with zero regression.
- Fully automated, reproducible execution via `cargo test --test adversarial_validation`.
