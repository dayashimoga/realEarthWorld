# ADR-0036: Stage 3 High-Fidelity Rendering, Architecture and Environmental Overhaul

## Status
Accepted

## Context
Following the completion of Stage 2 (Authentic San Francisco GIS Acquisition, Validation, and Binary Compilation), the Real Earth visual representation required a complete high-fidelity rendering overhaul. The baseline presentation suffered from several visual fidelity limitations:
1. Primitive rectangular building massing with uniform wall colors and severe vertical texture stretching on tall skyscrapers.
2. Roads rendered as flat uniform grey ribbons lacking lane markings, medians, curbs, sidewalks, and crosswalks.
3. Bridges rendered as floating road strips without structural box girders, safety guardrails, or vertical ground piers.
4. Terrain and green spaces rendered as flat planar geometry without 3D trees or foliage canopies.
5. Simple blue water surfaces lacking wave dynamics, Fresnel reflectance, or specular glints.
6. Flat sky and basic fog lacking atmospheric scattering, solar disk, and clouds.

Stage 3 mandates:
- **Phase 3.1 — Architectural Realism**: Authentic massing, surveyed heights, metric UVs, rooftop parapets and mechanical penthouses, varied PBR façades (glass, stone, brick, concrete, wood), and landmark geometric massing for San Francisco's iconic skyline.
- **Phase 3.2 — Roads & Urban Infrastructure**: Metric lane markings (double yellow centerline, dashed dividers, edge lines, zebra crosswalks), raised concrete curbs, wide sidewalks with slab joints, bridge box girders, guardrails, ground piers, and 3D streetlight poles.
- **Phase 3.3 — Nature & Atmosphere**: Realistic DEM material blending, 3D trees with organic trunks and tiered foliage canopies, multi-frequency Gerstner wave water animation, Fresnel reflectance, Rayleigh/Mie atmospheric skylight, and cloud deck.
- **Phase 3.4 — Production Renderer**: Single-pass PBR lighting, ACES filmic tonemapping, Depth32Float precision, frustum culling, GPU instancing/caching, bounded VRAM, and full 1080p (1920x1080) fixed-camera reference captures.

## Decisions

### 1. Phase 3.1: Metric UVs, Architectural Realism & Landmark Massing
- **Metric UV Parameterization**: Facade UVs are parameterized directly in meters ($u$ along polygon perimeter in meters, $v$ elevation in meters). This eliminates texture stretching regardless of whether a building is a 6-meter two-story home or a 326-meter skyscraper.
- **Rooftop Infrastructure**: Extruded rooftop parapet walls ($+0.8\text{m}$) and mechanical penthouses/cooling towers ($+3.5\text{m}$) with industrial metallic coloration.
- **Authentic Landmark Massing** in [engine/compiler/src/building_mesher.rs](file:///j:/rustRealEarth/engine/compiler/src/building_mesher.rs):
  - **Transamerica Pyramid**: 212m tapering pyramidal main trunk, East/West lateral wing setbacks, and a 48m hollow needle spire reaching the surveyed 260m CTBUH pinnacle.
  - **Salesforce Tower**: 3-tiered chamfered setback profile tapering toward the sky with a crowned lattice architectural screen reaching 326m.
  - **555 California**: Carnelian granite bay-window piers, bronze spandrels, and stepped setback penthouse reaching 237m.
  - **San Francisco Ferry Building**: 200m Beaux-Arts waterfront hall and 75m 4-tier clock tower modeled after Seville's Giralda.
  - **Coit Tower**: 12-sided fluted Art Deco concrete cylinder with recessed observation arcade reaching 64m on Telegraph Hill.
- **Material Encoding via Vertex Alpha**:
  - Glass curtain walls (`0.80`)
  - Carnelian granite / ashlar stone (`0.82`)
  - Red running-bond brick (`0.84`)
  - Reinforced concrete (`0.86`)
  - Victorian wood siding (`0.88`)
  - Industrial metal / HVAC (`0.70`)

### 2. Phase 3.2: Roads and Urban Infrastructure Meshing
- **Metric Road Carriageway Ribbon** in [engine/compiler/src/road_mesher.rs](file:///j:/rustRealEarth/engine/compiler/src/road_mesher.rs):
  - Retained strict continuous miter ribbon geometry (2 vertices per centerline node, upward-facing $+Y$ CCW winding) in `mesh_road`.
  - Shader procedural markings: double solid yellow centerlines with black median gap, white dashed lane dividers ($3\text{m}$ dash, $9\text{m}$ gap), solid white edge fog lines, and zebra crosswalk bars ($0.5\text{m}$ stripes) before intersections.
- **Complementary Urban Infrastructure** in `mesh_road_infrastructure`:
  - **Curbs & Sidewalks**: Raised concrete curbs ($+0.16\text{m}$) and wide pedestrian sidewalks ($2.0\text{m}$) with transverse contraction joints every $2.0\text{m}$.
  - **Bridge Box Girders & Guardrails**: Solid deck soffits underneath bridge carriageways ($1.6\text{m}$ depth), concrete highway guardrail barriers ($+0.95\text{m}$), and heavy vertical ground support piers ($1.8\text{m} \times 1.8\text{m}$) spaced every $35\text{m}$ connecting elevated bridge approaches to underlying terrain.
  - **Streetlight Poles**: 3D steel tubular poles ($6.2\text{m}$ height) with cantilevered luminaire arms ($2.4\text{m}$ reach) extending across driving lanes along major urban arterials.

### 3. Phase 3.3: Nature, Foliage, Water, and Atmosphere
- **3D Trees & Urban Parks** in [engine/compiler/src/vegetation_mesher.rs](file:///j:/rustRealEarth/engine/compiler/src/vegetation_mesher.rs):
  - Populates 3D procedural trees across surveyed OSM park polygons and green land cover.
  - Trees feature organic wood trunks ($2.4\text{m}$ height, $0.22\text{m}$ radius) and tiered octagonal foliage canopies ($4.8\text{m}$ canopy height) rooted directly on the underlying USGS 3DEP elevation model.
- **Dynamic Gerstner Water Waves** in [engine/rendering/src/shaders.rs](file:///j:/rustRealEarth/engine/rendering/src/shaders.rs):
  - Multi-frequency Gerstner wave simulation driving surface normal perturbation.
  - Fresnel reflectance blending deep oceanic blue (`#0f2e4d`) with shallow coastal turquoise (`#236b8e`) and foam crests.
  - Directional sunlight specular glints and sun path reflection.
- **Atmospheric Scattering & Sky Model**:
  - Solar Rayleigh/Mie atmospheric scattering gradient from horizon to zenith.
  - Warm sun disk with solar glow.
  - Procedural cumulus cloud deck modulating ambient illumination.

### 4. Phase 3.4: Production Rendering Pipeline & Validation
- **PBR Shading with ACES Filmic Tonemapping**:
  - Cook-Torrance GGX microfacet specular approximation with Schlick Fresnel.
  - ACES filmic tonemapping curve mapping high dynamic range lighting to standard sRGB display gamuts.
- **Depth32Float Precision & Memory Budgets**:
  - Clean depth rejection with reversed-Z / 32-bit floating point depth buffers.
  - GPU cache tracking resident VRAM allocations ($0.45\text{MB}$ in benchmark scenes).
- **1080p Fixed-Camera Reference Verification**:
  - Upgraded [apps/world-viewer/src/main.rs](file:///j:/rustRealEarth/apps/world-viewer/src/main.rs) with `--1080p` flag ($1920 \times 1080$ resolution).
  - Captured verified reference frames for skyline, dense streets, bridges, coastline, hills, and urban parks.

## Consequences

* **Positive**:
  - Real-world authentic San Francisco geometry is immediately recognizable with distinctive architectural landmarks (Transamerica Pyramid, Salesforce Tower, 555 California, Ferry Building, Coit Tower).
  - Roads feature accurate metric markings, raised curbs, and wide sidewalks without texture stretching.
  - Elevated bridge structures feature box girders, safety barriers, and ground piers.
  - Water and atmosphere provide cinematic, physically plausible illumination.
  - Exceptional performance: **947–1843 FPS** at 1080p, **0.54–1.53 ms** p95 frame latency, **232 MB** RSS memory.
  - 100% test pass rate across the workspace (all 23 test suites passing).
* **Reference Artifacts**:
  - Skyline 1080p: [sf_stage3_skyline.png](file:///C:/Users/daya/.gemini/antigravity-ide/brain/c5a543a1-6e1e-4fdf-83e0-1043a873219b/sf_stage3_skyline.png)
  - Streets 1080p: [sf_stage3_streets.png](file:///C:/Users/daya/.gemini/antigravity-ide/brain/c5a543a1-6e1e-4fdf-83e0-1043a873219b/sf_stage3_streets.png)
  - Bridge 1080p: [sf_stage3_bridge.png](file:///C:/Users/daya/.gemini/antigravity-ide/brain/c5a543a1-6e1e-4fdf-83e0-1043a873219b/sf_stage3_bridge.png)
  - Coastline 1080p: [sf_stage3_coastline.png](file:///C:/Users/daya/.gemini/antigravity-ide/brain/c5a543a1-6e1e-4fdf-83e0-1043a873219b/sf_stage3_coastline.png)
  - Hills 1080p: [sf_stage3_hills.png](file:///C:/Users/daya/.gemini/antigravity-ide/brain/c5a543a1-6e1e-4fdf-83e0-1043a873219b/sf_stage3_hills.png)
  - Parks 1080p: [sf_stage3_parks.png](file:///C:/Users/daya/.gemini/antigravity-ide/brain/c5a543a1-6e1e-4fdf-83e0-1043a873219b/sf_stage3_parks.png)
