# ADR 0001: Workspace Topology and Coordinate Systems

## Status
Accepted

## Context
A planetary-scale simulation requires double-precision (`f64`) geographic positioning to prevent spatial quantization errors, while GPU rendering pipelines operate on 32-bit floats (`f32`) for hardware efficiency. Placing an object at Earth's equatorial radius ($\approx 6,378,137\text{ m}$) using single-precision floats results in precision loss on the order of tens of centimeters to a full meter, manifesting as severe visual jitter and physics instabilities.

## Decision
1. **Workspace Architecture**: Structure the codebase as a Cargo workspace with decoupled domain crates (`engine/geospatial`, `engine/ingestion`, `engine/validation`, `engine/compiler`, `engine/streaming`, `engine/rendering`, `engine/simulation`, `engine/vehicles`, `engine/navigation`, `engine/network`, `apps/world-viewer`, `tools/world-cli`).
2. **Double-Precision Geodetic Core**: Store all planetary and chunk positions in WGS84 latitude/longitude/altitude and ECEF $(X, Y, Z)$ using `f64`.
3. **Local Tangent Plane**: Express local features inside chunks in East-North-Up (ENU) coordinates.
4. **Floating Origin for GPU**: Maintain an active floating origin anchor $\mathbf{A}_{\text{origin}} \in \text{ECEF}$ on the CPU. Rebase all vertex positions and camera transformations relative to $\mathbf{A}_{\text{origin}}$ before casting to `f32` uniforms for the GPU vertex shader. When camera distance exceeds $5,000\text{m}$, seamlessly re-anchor the origin.

## Consequences
- Guaranteed sub-millimeter precision at any point on Earth.
- Zero vertex jitter in the wgpu rendering pipeline.
- Clean separation between planetary coordinates and local simulation coordinates.
