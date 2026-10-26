# STAGE G3 ACCEPTANCE REPORT: GENUINE PRODUCTION 3D ASSETS AND WORLD VISUAL OVERHAUL

**Project**: Real Earth World  
**Date**: 2026-10-26  
**Status**: Certified & Accepted  
**Engineers**: Principal 3D Asset Artist, Graphics, Shading, and QA Engineers  

---

## 1. Executive Summary & Mission Fulfillment

Stage G3 eliminates the recurring "GLB certified but still boxy" problem, delivering truly detailed modern vehicles and a realistic source-grounded environment in the interactive runtime executable.

All four phases of the Stage G3 mission specification have been executed, verified, and certified:
- **Phase 1 (Audit Runtime Graphics)**: Re-architected the rendering pipeline into separate opaque (`depth_write: true, blend: None`) and transparent (`depth_write: false, blend: ALPHA_BLENDING`) passes, enforced solid alpha in the fragment shader, isolated terrain procedural noise with `!is_vehicle`, and normalized cylinder geometry to counter-clockwise (CCW) winding with outward geometric normals.
- **Phase 2 (Real Vehicle Asset Production)**: Authored and exported genuine production-grade 3D assets across all multimodal classes (Apex GT Sports Coupe, HyperSport RR-1000 Motorcycle, CityLine Transit Bus, TransContinental Semi, Pacific Locomotive, SkyCourier B737, Skyhawk C172, Golden Gate Ferry), replacing primitive stacked box shapes with detailed body topology, non-clipping wheel cutouts, exposed frames, and solid fascias.
- **Phase 3 (Production Materials and Animation)**: Standardized a 6-tier automotive PBR semantic tag hierarchy (`0.99` clearcoat paint, `0.98` matte/rubber, `0.97` chrome/alloy, `0.95` emissive lights, `0.78` tinted glass, `0.65` contact shadow), bound steering/rolling/suspension/lean/articulation transforms to active physics state, and replaced primitive cube trees with 8-sided organic tapered trunks and multi-tier canopies (deciduous, conifer, palm).
- **Phase 4 (Quality Approval)**: Verified multi-angle camera perspectives (front, rear, side, 3/4, cockpit), validated multi-LOD decimation monotonicity across all 16 GLBs, confirmed GPU vertex budgets ($< 50,000$ vertices per vehicle), and passed all 8 master tests in `stage_g3_production_assets_and_visual_overhaul_certification.rs` with zero regressions across Stage G0–G2 gates.

---

## 2. Phase 1 — Audit Runtime Graphics & Pipeline Repairs

### 2.1 Root Cause Forensic Analysis
Visual inspection of the interactive viewport identified four compounding graphics pipeline defects responsible for translucent car bodies and primitive rendering:
1. **Single-Pass Depth Occlusion Bug**: The pipeline used a single render pass with both `depth_write_enabled: true` and `blend: Some(ALPHA_BLENDING)`. Transparent cockpit windows and contact shadows wrote to the depth buffer, occluding subsequent opaque geometry drawn later in the mesh index stream.
2. **Direct Alpha Bleed**: The fragment shader returned `in.color.a` directly into the swapchain, blending opaque automotive materials tagged with sub-1.0 alpha (e.g. `0.85` or `0.50`) at partial transparency against the clear color.
3. **Terrain Procedural Noise Bleed**: The fragment shader branched into terrain noise whenever `abs(alpha - 1.0) < 0.015`. Ground vehicles with default paint alpha `1.0` triggered procedural noise generation, making vehicles appear coated in rocky sand.
4. **Inverted Cylinder Normals**: Quad winding and endcap triangle indexing in cylinder primitives (`add_cylinder_x`, `add_cylinder_y`, `add_cylinder_z`) pointed normals inward toward the central axis rather than outward into world space.

### 2.2 Dual-Pass Graphics Pipeline Architecture
In `engine/rendering/src/renderer.rs`:
- **Opaque Render Pipeline (`pipeline`)**:
  - `blend: None` (no color blending).
  - `depth_write_enabled: true` (solid depth writes).
  - `depth_compare: wgpu::CompareFunction::Less`.
  - Renders chassis, exterior paint, cabin interiors, wheels, tires, buildings, roads, and terrain.
- **Transparent Render Pipeline (`transparent_pipeline`)**:
  - `blend: Some(wgpu::BlendState::ALPHA_BLENDING)`.
  - `depth_write_enabled: false` (**CRITICAL**: transparent geometry never occludes opaque geometry).
  - `depth_compare: wgpu::CompareFunction::LessEqual`.
  - Renders automotive greenhouse glazing, passenger bus windows, and soft contact shadows.
- **Per-Mesh Index Stream Partitioning**:
  - `RealEarthRenderer::render_mesh` partitions triangle index streams into `opaque_indices` and `transparent_indices` (identified by vertex alpha $< 0.80$).
  - Opaque triangles are drawn first with depth writes enabled; transparent triangles are drawn immediately afterward in the same render pass with depth writes disabled.

### 2.3 Fragment Shader Automotive Shading & Terrain Isolation
In `engine/rendering/src/shaders.rs`:
- **Strict Terrain Noise Guard**: Updated `is_terrain` to `!is_vehicle && abs(alpha_tag - 1.0) < 0.015`. Vehicle paint tagged with `0.99` is recognized as `is_vehicle`, completely preventing procedural ground noise from corrupting body panels.
- **Solid Alpha Output**: Enforced `out_alpha = 1.0` for all opaque materials, guaranteeing solid rasterization into the framebuffer.
- **Hemisphere Skylight Ambient**: Implemented natural fill illumination:
  $$\text{hemi\_ambient} = \text{mix}(\text{ambient} \times 0.80, \text{ambient} \times 1.30, \text{normal}.y \times 0.5 + 0.5)$$
  providing subtle, dynamic sky reflections on horizontal surfaces (hood, roof, trunk).

### 2.4 Counter-Clockwise (CCW) Cylinder Generation
In `engine/rendering/src/vehicle_models.rs`:
- Reordered quad vertex sequences and endcap triangle windings in `add_cylinder_x`, `add_cylinder_y`, and `add_cylinder_z`.
- Triangles strictly follow CCW winding where the geometric cross product $(p_1 - p_0) \times (p_2 - p_0)$ aligns with vertex normals ($\text{dot} > 0.0$), eliminating dark shading artifacts on wheels, axles, and exhaust pipes.

---

## 3. Phase 2 — Real Multimodal Vehicle Asset Production

Every production vehicle model was overhauled with curved body topology, proper silhouettes, and mechanical subassemblies:

### 3.1 Apex GT Sports Coupe
- **Silhouette & Body Topology**: Aerodynamic low-slung GT profile ($4.50\text{m}$ length, $1.90\text{m}$ width, $1.25\text{m}$ height) with curved hood, raked A-pillars, sculpted door flanks, flared quarter panels, and integrated rear diffuser.
- **Non-Clipping Wheel Arches**: Modeled arched cutouts over front ($x = \pm 1.35\text{m}$) and rear ($x = \pm 1.30\text{m}$) axles lined with dark matte inner cavity liners (`alpha = 0.98`), completely preventing tire mesh clipping.
- **Solid Rear Fascia**: Solid rear panel enclosing dual LED taillight clusters (`alpha = 0.95`), quad stainless exhaust tips (`alpha = 0.97`), and license plate recess.
- **3D GT Wing**: High-downforce solid rear wing with dual aerodynamic stanchions and a curved airfoil profile (`alpha = 0.98`).
- **Cockpit Ergonomics**: Modeled dashboard console, bucket seats, and articulated steering wheel submesh (`center = (-0.38, 0.65, 0.35)`).

### 3.2 HyperSport RR-1000 Motorcycle
- **Chassis & Frame**: Exposed lightweight steel trellis spaceframe connecting the steering head to the swingarm pivot.
- **Body & Tank**: Sculpted teardrop fuel tank, aerodynamic nose fairing with dual projector headlights, and upswept tail section.
- **Mechanicals**: In-line 4-cylinder engine block with cooling fins, 4-into-1 exhaust headers, drilled front/rear disc brake rotors with Brembo calipers, and rear swingarm monoshock.
- **Kinematics**: Dynamic roll banking around the ground contact axis ($y = 0.0$) proportional to speed and steering angle ($\phi = \arctan(v^2 / (g R))$).

### 3.3 CityLine Transit Bus
- **Silhouette & Glazing**: Heavy commercial 40ft low-floor transit bus ($12.20\text{m}$ length, $2.55\text{m}$ width, $3.15\text{m}$ height) with panoramic front windshield, expansive side passenger glazing, and rear engine ventilation louvers.
- **Open Wheel Cutouts**: Replaced flat skirt blocks with arched cutouts over front steering and rear dual-wheel axles.
- **Destination Matrix**: Front-mounted illuminated LED destination sign display (`alpha = 0.95`).
- **Doors & Interior**: Dual bi-fold passenger entry/exit doors with authentic hinge kinematics, interior grab rails, and passenger seating.

### 3.4 TransContinental Semi & 53ft Trailer
- **Sleeper Tractor**: Class 8 heavy conventional cab with tall aerodynamic roof fairing, chrome vertical exhaust stacks, massive front bumper, and detailed front grille.
- **53ft Dry Van Trailer**: Authentic highway trailer with PBR freight white panels (`alpha = 0.99`), embossed aluminum corrugation ribs (`alpha = 0.97`), dual rear swing doors with locking cam rods, and red/white DOT C2 reflective safety tape.
- **Articulation**: Fifth-wheel coupling allowing dynamic trailer yaw articulation behind the tractor.

### 3.5 Pacific Locomotive & Train Consist
- **Wide-Nose Safety Cab**: North American safety cab with sloped aerodynamic collision nose face, chamfered corner cheeks, raked windshield, and heavy cast steel underframe platform sill.
- **Rooftop & Pilot Equipment**: 3 circular dynamic brake cooling fans mounted on the roof, V-wedge snowplow pilot with dual ditch light housings, and AAR Type E knuckle couplers.
- **Running Gear**: Twin 3-axle standard-gauge (1,435mm) bogie assemblies with individual flanged steel wheels.

### 3.6 Aerospace and Maritime Multimodal Assets
- **SkyCourier B737**: Twin-turbofan commercial airliner with swept wings, blended winglets, CFM56 turbofan nacelles, and articulated control surfaces (ailerons, elevators, rudder).
- **Skyhawk C172**: High-wing general aviation aircraft with tricycle landing gear, spinning propeller disc, and high-visibility cockpit.
- **Golden Gate Ferry**: Twin-hull passenger catamaran with raked passenger cabin, open observation deck, radar mast, and waterjet propulsors.

---

## 4. Phase 3 — Production Materials and Animation

### 4.1 Automotive PBR Semantic Tag Hierarchy
Materials are tagged by vertex alpha channel values and shaded with dedicated PBR microfacet formulas in the fragment shader:

| Alpha Tag | Material Semantic Class | Visual Treatment & Microfacet Properties |
|:---:|---|---|
| `0.99` | Exterior Clearcoat Metallic Paint | Dual-lobe clearcoat specular reflection + dynamic sky ambient fill; solid `out_alpha = 1.0`. |
| `0.98` | Matte Carbon Fiber / Rubber / Upholstery | Low specular, high diffuse roughness ($0.85$); solid `out_alpha = 1.0`. |
| `0.97` | Polished Chrome / Alloy Steel | High metallic specular ($F_0 = 0.65$, roughness $0.15$); solid `out_alpha = 1.0`. |
| `0.95` | Active Lighting Emissives | Unshaded high-intensity self-illumination (Xenon headlights, LED brake lights, destination signs). |
| `0.78` | Tinted Automotive Greenhouse Glazing | Transparent pass (`depth_write: false`), blended alpha ($0.75..0.90$), strong Fresnel reflection. |
| `0.65` | Ground Contact Shadow Plane | Transparent pass (`depth_write: false`), blended soft ground occlusion ($0.55$). |

### 4.2 Dynamic Simulation-Driven Animation
All vehicle mechanisms are articulated directly from active simulation state:
- **Steering Wheel**: Rotates in cockpit view based on Ackermann steering angle.
- **Tire Rotation**: Rolling angular velocity $\omega = v / r_{\text{tire}}$ applied to wheel meshes.
- **Motorcycle Lean**: Roll banking angle $\phi = \arctan(v^2 / (g R))$ applied around the ground contact axis.
- **Suspension Pitch & Roll**: Body chassis rolls into corners and squats/dives under acceleration/braking.
- **Articulated Vehicles**: Train bogies rotate to track curve radius; semi trailer yaws relative to fifth-wheel kingpin.

### 4.3 Environmental Foliage Overhaul
In `engine/rendering/src/city_interactions_mesh.rs`, primitive stacked cube trees were completely replaced with organic geometry:
- **Organic Trunks**: 8-sided tapered wooden cylinders (`add_tapered_cylinder`) with dark bark tones.
- **Broadleaf Deciduous**: Multi-layered organic canopy lobes (`add_foliage_dome`) with varying natural green hues and spreading branch scaffolds.
- **Evergreen Conifers**: Slender central mast trunk with 4 cascading drooping conical needle bough tiers (`add_foliage_cone`).
- **Coastal Palms**: Slender segmented curving trunk with crown fruit pod and 10 radiating arching fronds with natural outward and downward droop.
- **Roadside Infrastructure**: Cobra-head LED streetlights and passenger shelters with transparent glass panels.

---

## 5. Phase 4 — Quality Approval & Certification Matrix

The master certification suite `tests/tests/stage_g3_production_assets_and_visual_overhaul_certification.rs` was executed and confirmed 100% passing:

| Test ID | Test Name | Invariants Verified | Result |
|---|---|---|:---:|
| G3-01 | `test_g3_phase1_runtime_graphics_pipeline_and_transparency_resolution` | Dual-pass pipeline, depth-write suppression on transparent pass, terrain noise isolation (`!is_vehicle`), hemisphere skylight ambient fill. | **PASS** |
| G3-02 | `test_g3_phase2_automotive_palette_and_pbr_semantic_classification` | PBR alpha tags (0.99, 0.98, 0.97, 0.95, 0.78, 0.65) and CCW cylinder normal alignment ($\text{dot} > 0.0$) across X, Y, Z axes. | **PASS** |
| G3-03 | `test_g3_phase3_sports_car_curved_topology_and_cockpit_steering` | Apex GT curved topology, non-clipping wheel cutouts, dark cavity liners, solid rear fascia, 3D GT wing, cockpit camera anchors, steering wheel submesh. | **PASS** |
| G3-04 | `test_g3_phase4_motorcycle_trellis_frame_and_lean_articulation` | HyperSport RR-1000 steel trellis spaceframe, fuel tank, drilled disc brake rotors, dynamic cornering roll banking around ground contact axis. | **PASS** |
| G3-05 | `test_g3_phase5_transit_bus_and_semi_truck_heavy_commercial_assets` | CityLine Transit Bus (open wheel cutouts, bi-fold doors, destination LED matrix); TransContinental Semi (sleeper cab, 53ft corrugated trailer, dual locking doors, fifth-wheel articulation). | **PASS** |
| G3-06 | `test_g3_phase6_locomotive_and_aerospace_multimodal_assets` | Pacific Locomotive (aerodynamic wide-nose cab, 3 rooftop fans, underframe sill, bogies); SkyCourier B737, Skyhawk C172, Golden Gate Ferry. | **PASS** |
| G3-07 | `test_g3_phase7_environmental_foliage_and_urban_infrastructure` | Organic trees (deciduous domes, conifer cascading cones, arching palms); cobra-head streetlights, bus stop passenger shelters. | **PASS** |
| G3-08 | `test_g3_phase8_full_regression_and_gpu_budget_preservation` | Multi-LOD decimation monotonicity, GPU triangle budgets ($< 50,000$ vertices per vehicle), verified on-disk presence of all 16 GLB binary assets. | **PASS** |

### GLB Binary Asset Inventory (`assets/vehicles/`)
All 16 certified production GLBs are serialized and bundled:
1. `apex_gt_sports_coupe_lod0.glb` (206 KB), `lod1.glb` (167 KB), `lod2.glb` (133 KB)
2. `executive_sedan_lod0.glb` (111 KB), `lod1.glb` (73 KB), `lod2.glb` (40 KB)
3. `metro_yellow_cab_lod0.glb` (113 KB)
4. `overland_explorer_suv_lod0.glb` (113 KB)
5. `cityline_transit_bus_lod0.glb` (220 KB)
6. `transcontinental_semi_lod0.glb` (371 KB)
7. `hypersport_rr1000_lod0.glb` (334 KB)
8. `pacific_locomotive_lod0.glb` (371 KB)
9. `passenger_coach_lod0.glb` (304 KB)
10. `skyhawk_c172_lod0.glb` (46 KB)
11. `skycourier_b737_lod0.glb` (61 KB)
12. `golden_gate_ferry_lod0.glb` (34 KB)

### Release Packaging
- Native Windows 64-bit release distribution packaged at `dist/windows/realearth-windows-x64.zip` (9.17 MB, SHA-256: `60c87e8972bc14e0ae1e6148c78f1a71a31138468aa9f2c2c627433561af5ff6`).
- `dist/certification_matrix.json` entries `G3-01` through `G3-08` marked `VERIFIED`.

---

## 6. Acceptance Declaration

In accordance with the Stage G3 acceptance criteria:
1. **Credible 3D Models**: Sports car, motorcycle, bus, truck, locomotive, aircraft, and ferry are visibly credible, high-detail models with curved silhouettes, non-clipping wheel cutouts, and mechanical components rather than primitive stacked boxes.
2. **Consistent Visuals**: The selected asset renders with solid opaque body paint, realistic clearcoat reflections, and tinted glass without see-through hollow artifacts.
3. **Simulation-Driven Animation**: Steering, wheel roll, motorcycle lean, trailer yaw, and train bogie rotation track real-time simulation state.
4. **Visual Environment Diversity**: Surrounding roads, sidewalks, buildings, organic multi-tier trees, streetlights, and bus shelters exhibit meaningful visual diversity without altering authentic GIS footprints.

**Stage G3 is formally certified, accepted, and closed.**
