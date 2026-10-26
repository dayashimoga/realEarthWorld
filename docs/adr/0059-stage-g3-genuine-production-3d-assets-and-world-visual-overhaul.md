# ADR-0059: Stage G3 — Genuine Production 3D Assets and World Visual Overhaul

## Status
**Accepted** (2026-10-26)

## Context
Following the delivery of authentic GIS compilation and continuous earth streaming in Stage G2 ([ADR-0058](file:///j:/rustRealEarth/docs/adr/0058-stage-g2-authentic-gis-world-generation-and-continuous-streaming.md)), a comprehensive visual audit of the interactive executable revealed several critical graphics and asset defects that undermined real-world fidelity:

1. **The "GLB Certified but Still Boxy" Paradox:** Assets previously designated as "certified" merely satisfied Khronos glTF binary serialization and arbitrary vertex counts, but contained flat blocky geometries, unsegmented trailers, flat bus skirts clipping through rotating wheels, and primitive rail cab boxes.
2. **Translucent / Hollow Vehicle Body Artifacts:** Vehicles exhibited see-through body panels where tires, interior seats, and opposite ground were visible through the exterior paint. This was caused by two compounding graphics pipeline bugs:
   - The rendering pipeline used a single pass with `blend: Some(ALPHA_BLENDING)` AND `depth_write_enabled: true`. Transparent polygons (windows, contact shadows) wrote to the depth buffer, occluding subsequent opaque cockpit and chassis geometry drawn later in the draw call.
   - The fragment shader returned `in.color.a` directly into the framebuffer, blending opaque automotive materials with alpha tags (such as `0.85` or `0.50`) at partial transparency against the background.
3. **Terrain Shader Noise Pollution on Vehicles:** The fragment shader detected terrain geometry using `abs(in.color.a - 1.0) < 0.015`. Drivable vehicles and traffic at ground level with paint alpha `1.0` triggered the procedural terrain noise branch, causing vehicles to appear coated in rocky sand and noise textures.
4. **Inverted Cylinder Normals and Triangle Winding:** Cylinder generation primitives (`add_cylinder_x`, `add_cylinder_y`, `add_cylinder_z`) used inconsistent quad index ordering and endcap triangle windings, generating geometric normals that pointed inwards toward the central axis rather than outwards into world space.
5. **Primitive "Minecraft" Foliage:** City trees were rendered as stacked primitive box extrusions masquerading as trunks and canopies, conflicting sharply with the realism goals of the platform.

---

## Decisions

### 1. Dual-Pass Graphics Pipeline & Depth-Write Separation
In [`engine/rendering/src/renderer.rs`](file:///j:/rustRealEarth/engine/rendering/src/renderer.rs):
- **Opaque Render Pipeline (`pipeline`):** Configured with `blend: None`, `depth_write_enabled: true`, and `depth_compare: wgpu::CompareFunction::Less`. Draws solid body paint, interiors, wheels, buildings, roads, and terrain with guaranteed occlusion.
- **Transparent Render Pipeline (`transparent_pipeline`):** Configured with `blend: Some(wgpu::BlendState::ALPHA_BLENDING)`, `depth_write_enabled: false` (CRITICAL: never write depth for transparent elements), and `depth_compare: wgpu::CompareFunction::LessEqual`.
- **Per-Mesh Draw Partitioning:** In `RealEarthRenderer::render_mesh`, triangle indices are partitioned into `opaque_indices` and `transparent_indices` (identified by vertex alpha $< 0.94$). Opaque geometry is drawn first with full depth writes, followed immediately by transparent glass and contact shadows with depth writes disabled in the same render pass.

### 2. Dedicated Automotive PBR Semantics & Terrain Shading Isolation
In [`engine/rendering/src/shaders.rs`](file:///j:/rustRealEarth/engine/rendering/src/shaders.rs):
- **Automotive Material Tag Hierarchy:**
  - `0.99`: Exterior Clearcoat Metallic Body Paint (Opaque, dual-lobe clearcoat specular reflection + sky ambient fill).
  - `0.98`: Matte Carbon Fiber, Tire Rubber, Interior Upholstery (Opaque, low specular, roughness $0.85$).
  - `0.97`: Polished Chrome, Alloy Rims, Steel Sills (Opaque, high metallic specular with $F_0 = 0.65$).
  - `0.95`: Active Lighting Emissives (Opaque, active Xenon headlights, LED brake lights, destination signs).
  - `0.78`: Tinted Automotive Greenhouse Glass (Transparent, blended alpha $0.75..0.90$, high Fresnel reflection).
  - `0.65`: Ground Contact Shadow Plane (Transparent, blended soft ground occlusion alpha $0.55$).
- **Strict Terrain Noise Guard:** Updated `is_terrain` definition to `!is_vehicle && abs(alpha_tag - 1.0) < 0.015`. Vehicle paint tagged with `0.99` is strictly recognized as `is_vehicle`, completely preventing procedural ground noise from corrupting car and truck body panels.
- **Solid Alpha Output:** Enforced `out_alpha = 1.0` for all opaque materials, ensuring solid rasterization into the framebuffer.
- **Hemisphere Skylight Ambient:** Implemented `hemi_ambient = mix(ambient * 0.80, ambient * 1.30, normal.y * 0.5 + 0.5)` for dynamic natural fill reflections on horizontal vehicle surfaces.

### 3. CCW Cylinder Generator Normal Normalization
In [`engine/rendering/src/vehicle_models.rs`](file:///j:/rustRealEarth/engine/rendering/src/vehicle_models.rs):
- Corrected quad winding orders and endcap triangle orientations in `add_cylinder_x`, `add_cylinder_y`, and `add_cylinder_z`.
- Triangles now strictly follow counter-clockwise (CCW) winding where the geometric cross product $(p_1 - p_0) \times (p_2 - p_0)$ points outward radially, matching vertex normal vectors and eliminating dark shading artifacts.

### 4. Real Multimodal Vehicle 3D Geometry Overhaul
- **Apex GT Sports Coupe:** Modeled non-clipping wheel arches with dark matte inner cavity liners, solid rear fascia enclosing tail details, 3D solid GT wing with cambered airfoil and stanchions, quad stainless exhaust pipes, and calibrated cockpit camera anchors with articulated steering wheel.
- **HyperSport RR-1000 Motorcycle:** Modeled exposed steel trellis spaceframe, teardrop fuel tank, swingarm monoshock, drilled disc brake rotors with Brembo calipers, and dynamic cornering roll banking around the ground contact axis.
- **CityLine Transit Bus:** Replaced solid skirt blocks with open wheel arch cutouts over front and rear axles, added illuminated LED destination sign matrix (`alpha = 0.95`), white fiberglass roof, and bi-fold passenger door opening kinematics.
- **TransContinental Semi:** Replaced flat trailer container with PBR freight white (`alpha = 0.99`), embossed aluminum corrugation ribs (`alpha = 0.97`), rear dual swing doors with locking hardware, DOT C2 reflective safety tape, and fifth-wheel yaw articulation.
- **Pacific Locomotive:** Replaced crude cab box with sloped aerodynamic collision nose face, chamfered corner cheeks, raked windshield, heavy underframe sill, V-wedge snowplow pilot with ditch light housings, 3 rooftop dynamic brake cooling fans, twin 1,435mm standard-gauge bogies, and AAR knuckle couplers.

### 5. Environmental Foliage & Infrastructure Overhaul
In [`engine/rendering/src/city_interactions_mesh.rs`](file:///j:/rustRealEarth/engine/rendering/src/city_interactions_mesh.rs):
- Completely eliminated primitive stacked box foliage.
- Implemented `add_tapered_cylinder` for 8-sided smooth organic tapered wooden trunks.
- **Broadleaf Deciduous:** Multi-layered organic canopy lobes (`add_foliage_dome`) with varying natural green hues and spreading branch scaffolds.
- **Evergreen Conifers:** Slender tapered central mast trunk with 4 cascading drooping conical needle bough tiers (`add_foliage_cone`).
- **Coastal Palms:** Slender segmented curving trunk with crown fruit pod and 10 radiating arching fronds featuring natural outward and downward droop.
- Validated roadside street furniture including cobra-head LED streetlights and municipal bus stop passenger shelters.

---

## Verification & Master Certification

### Master Test Suite (`stage_g3_production_assets_and_visual_overhaul_certification.rs`)
Authored 8 comprehensive, authoritative tests passing with 0 failures:
1. `test_g3_phase1_runtime_graphics_pipeline_and_transparency_resolution`: Validates dual-pass pipeline architecture, depth-write suppression on transparent pass, terrain noise isolation, and hemisphere skylight ambient fill.
2. `test_g3_phase2_automotive_palette_and_pbr_semantic_classification`: Validates distinct PBR semantic alpha tags (0.99, 0.98, 0.97, 0.95, 0.78, 0.65) and outward CCW cylinder winding across X, Y, and Z axes.
3. `test_g3_phase3_sports_car_curved_topology_and_cockpit_steering`: Validates Apex GT curved topology, wheel arch cutouts, dark cavity liners, solid rear fascia, 3D GT wing, cockpit camera anchors, and steering wheel submesh.
4. `test_g3_phase4_motorcycle_trellis_frame_and_lean_articulation`: Validates HyperSport RR-1000 frame, wheels, disc brakes, and dynamic cornering roll banking kinematics.
5. `test_g3_phase5_transit_bus_and_semi_truck_heavy_commercial_assets`: Validates CityLine Transit Bus (open wheel arches, bi-fold doors, destination LED matrix) and TransContinental Semi (corrugated 53ft trailer, dual locking doors, fifth-wheel articulation).
6. `test_g3_phase6_locomotive_and_aerospace_multimodal_assets`: Validates Pacific Locomotive (aerodynamic wide-nose cab, 3 rooftop fans, underframe sill, bogies), SkyCourier B737, Skyhawk C172, and Golden Gate Ferry.
7. `test_g3_phase7_environmental_foliage_and_urban_infrastructure`: Validates organic tree generation (deciduous domes, conifer cascading cones, arching palms), streetlights, and bus shelters.
8. `test_g3_phase8_full_regression_and_gpu_budget_preservation`: Validates multi-LOD decimation monotonicity, GPU triangle budgets ($< 50,000$ vertices per vehicle), and verified on-disk presence of all 16 GLB binary assets.

### Regression Suite Invariance
- `stage_g2_authentic_gis_and_continuous_streaming_certification`: 8/8 tests passed.
- `stage_g1_vehicle_physics_and_controls_certification`: 8/8 tests passed.
- `stage_g0_acceptance_and_defect_reproduction`: 6/6 tests passed.
- `stage_g0_real_executable_validation`: 7/7 tests passed.
- `stage_d_gis_streaming_and_city_transitions`: 5/5 tests passed.
- `stage_b_remediation_certification`: 8/8 tests passed.
- `stage_c_premium_models_and_animation`: 6/6 tests passed.

### Distribution Packaging
- Compiled native 64-bit release binaries (`realearth-viewer.exe`, `realearth-cli.exe`).
- Serialized all 16 certified vehicle GLBs to `assets/vehicles/`.
- Packaged `dist/windows/realearth-windows-x64.zip` (9.17 MB, SHA-256: `60c87e8972bc14e0ae1e6148c78f1a71a31138468aa9f2c2c627433561af5ff6`).
- Updated `dist/certification_matrix.json` with entries `G3-01` through `G3-08` marked `VERIFIED`.
