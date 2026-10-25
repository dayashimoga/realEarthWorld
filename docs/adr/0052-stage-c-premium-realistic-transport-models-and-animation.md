# ADR-0052: Stage C — Premium Realistic Transport Models and Animation

## Status
**Accepted** (2026-10-25)

## Context
Following the authoritative vehicle physics, canonical coordinate conventions, and input correctness certified in Stage B ([ADR-0051](file:///j:/rustRealEarth/docs/adr/0051-stage-b-systemic-defect-remediation-and-certification.md)), Stage C executes the visual fidelity and animation mandate defined in `docs/RecoveryContract.md`:
1. **Asset Pipeline & glTF Ingestion (Phase C1):** Complete glTF/GLB 2.0 binary asset ingestion, Khronos-compliant node hierarchies, licensed asset manifest attribution, PBR metallic-roughness materials, and submesh alpha depth sorting.
2. **First Reference Car Overhaul (Phase C2):** Upgrade the Apex GT Sports Coupe into a reference-quality automotive model:
   - Eliminate wheel/chassis clipping by introducing authentic radiused wheel arch cutouts with dark inner wheel cavity liners.
   - Eliminate floating/disconnected rear components by building a solid, enclosed rear fascia with an upper decklid shelf, mid bumper face, recessed license plate cove with embossed plate, and lower apron connecting to the carbon diffuser.
   - Replace overly simplistic 2D spoiler geometry with a 3D enclosed carbon fiber airfoil, solid stanchions, trailing Gurney flap, and 3D endplates.
   - Add front and rear amber turn signal indicator clusters (`VehicleSubmeshType::TurnSignals`).
   - Realign quad polished stainless steel exhaust pipes longitudinally along the Z-axis (`add_cylinder_z`).
   - Detailed cockpit interior (racing bucket seats, digital display screens, center console, articulated steering wheel) and verified camera viewpoints.
3. **Multimodal Transport Classes (Phase C3):** Verify 12 distinct, authored transport models across Coupe, Sedan, SUV, Taxi, Semi-Truck, Transit Bus, Motorcycle, Locomotive, Coach, Ferry, Airliner, and General Aviation.
4. **Physics-Driven Animations & Optimization (Phase C4):** Verify dynamic wheel roll/steering, motorcycle cornering lean banking, truck trailer articulation, transit bus bi-fold doors, train bogies, aircraft rotors/propellers, marine radar masts, and monotonic multi-LOD decimation.
5. **Visual Certification & Non-Regression (Phase C5):** Export all 16 GLB binary containers to `assets/vehicles/` on disk, validate binary headers, and guarantee zero regressions on Stage B physics.

---

## Decisions

### 1. Submesh Depth Sorting & Alpha Blending Pipeline ([`engine/rendering/src/gltf_asset.rs`](file:///j:/rustRealEarth/engine/rendering/src/gltf_asset.rs))
* **Depth Ordering:** In `compose_multimodal_world_mesh`, sorted submeshes such that `VehicleAlphaMode::Opaque` geometry (chassis, interior, wheels, engine) is rasterized first, followed by `VehicleAlphaMode::Mask`, and finally `VehicleAlphaMode::Blend` (tinted greenhouse glass, contact shadows).
* **Depth Buffer Integrity:** Eliminates occlusion defects where tinted glass windshields or side windows prematurely wrote into depth buffers and clipped interior dashboards, seats, or steering wheels.
* **Semantic Rigging Support:** Added direct match routing for `VehicleSubmeshType::TurnSignals`.

### 2. Apex GT Sports Coupe Anatomical Overhaul ([`engine/rendering/src/vehicle_models.rs`](file:///j:/rustRealEarth/engine/rendering/src/vehicle_models.rs))
* **Non-Clipping Wheel Arches:** Replaced flat continuous fender quads with 3-section fender panels containing radiused wheel arch cutouts rising to $y = 0.72\text{m}$ (front) and $y = 0.74\text{m}$ (rear), comfortably clearing the $0.66\text{m}$ tire tops.
* **Dark Wheel Well Cavity Liners:** Inset matte dark composite panels (`[0.05, 0.05, 0.06, 1.0]`) enclosing the inner wheel tubs, eliminating hollow see-through voids behind wheels.
* **Solid Grounded Rear Fascia:** Built a four-tier vertical rear enclosure connecting the decklid edge ($y=0.82\text{m}, z=-2.12\text{m}$) down to the carbon diffuser ($y=0.32\text{m}, z=-2.22\text{m}$), embedding the full-width OLED taillight bar, reverse lights, recessed dark license plate cove, and stamped registration plate.
* **3D Solid Aerodynamic GT Wing:** Replaced 1-sided 2D quad stanchions and airfoil with 3D solid carbon uprights (`add_box_at`), a 3D enclosed airfoil camber box, trailing Gurney flap lip, and aerodynamic endplates.
* **Dynamic Amber Turn Signals:** Added `VehicleSubmeshType::TurnSignals` with front headlight brow indicators and rear taillight outer modules (`AutomotivePalette::TURN_SIGNAL_AMBER`, emissive `[1.0, 0.65, 0.05]`).
* **Longitudinal Exhaust Geometry:** Replaced transverse X-axis cylinders with longitudinal Z-axis cylinders (`add_cylinder_z`) protruding rearwards through the lower bumper fascia.

### 3. Compilation & Export of All 16 GLB Binary Assets ([`assets/vehicles/`](file:///j:/rustRealEarth/assets/vehicles/))
* Generated and certified all 16 Khronos glTF 2.0 / GLB binary files on disk:
  - `apex_gt_sports_coupe_lod0.glb` (206,888 bytes)
  - `apex_gt_sports_coupe_lod1.glb` (167,624 bytes)
  - `apex_gt_sports_coupe_lod2.glb` (133,968 bytes)
  - `executive_sedan_lod0.glb` (111,852 bytes)
  - `executive_sedan_lod1.glb` (73,452 bytes)
  - `executive_sedan_lod2.glb` (40,228 bytes)
  - `overland_explorer_suv_lod0.glb` (113,252 bytes)
  - `metro_yellow_cab_lod0.glb` (113,152 bytes)
  - `transcontinental_semi_lod0.glb` (202,072 bytes)
  - `cityline_transit_bus_lod0.glb` (212,724 bytes)
  - `hypersport_rr1000_lod0.glb` (105,920 bytes)
  - `pacific_locomotive_lod0.glb` (349,672 bytes)
  - `passenger_coach_lod0.glb` (304,552 bytes)
  - `golden_gate_ferry_lod0.glb` (34,964 bytes)
  - `skycourier_b737_lod0.glb` (61,948 bytes)
  - `skyhawk_c172_lod0.glb` (46,068 bytes)
* Confirmed licensed asset attribution manifest in [`assets/vehicles/LICENSE_ATTRIBUTION.md`](file:///j:/rustRealEarth/assets/vehicles/LICENSE_ATTRIBUTION.md).

---

## Verification & Certification

All Stage C requirements and Stage B non-regression invariants are certified by automated test suites passing 100%:

1. **Stage C Master Certification Suite** ([`tests/tests/stage_c_premium_models_and_animation.rs`](file:///j:/rustRealEarth/tests/tests/stage_c_premium_models_and_animation.rs)):
   - `test_phase_c1_asset_pipeline_and_gltf_ingestion` (glTF 2.0 magic, manifest, depth sorting, fallback rejection): **PASSED**
   - `test_phase_c2_apex_gt_reference_car_fidelity` (wheel arches, solid rear fascia, 3D spoiler, interior, camera anchors): **PASSED**
   - `test_phase_c3_multimodal_transport_classes` (all 12 transport classes uniquely authored with class features): **PASSED**
   - `test_phase_c4_physics_driven_animations_and_lod` (bus doors, bike lean, truck trailer, train bogies, propeller spin, LOD decimation): **PASSED**
   - `test_phase_c5_export_all_16_glb_assets_and_verify` (all 16 GLBs verified on disk): **PASSED**
   - `test_stage_b_physics_and_coordinate_regression_invariance` (fixed timestep, steer self-centering, building collision): **PASSED**

2. **Stage B Authoritative Physics & Coordinates** ([`tests/tests/stage_b_authoritative_physics_and_inputs.rs`](file:///j:/rustRealEarth/tests/tests/stage_b_authoritative_physics_and_inputs.rs)):
   - 9 passed, 0 failed (100% pass rate).

3. **Stage B Remediation Suite** ([`tests/tests/stage_b_remediation_certification.rs`](file:///j:/rustRealEarth/tests/tests/stage_b_remediation_certification.rs)):
   - 8 passed, 0 failed (100% pass rate).

4. **Stage 1.6 Multimodal Animation Suite** ([`tests/tests/stage1_6_vehicle_visual_and_animation_certification.rs`](file:///j:/rustRealEarth/tests/tests/stage1_6_vehicle_visual_and_animation_certification.rs)):
   - 10 passed, 0 failed (100% pass rate).

5. **Stage 0.8 Vehicle Visual Suite** ([`tests/tests/stage0_8_vehicle_visual_certification.rs`](file:///j:/rustRealEarth/tests/tests/stage0_8_vehicle_visual_certification.rs)):
   - 6 passed, 0 failed (100% pass rate).

6. **Static Analysis & Workspace Health**:
   - `cargo check --workspace`: 0 errors, 0 warnings.
