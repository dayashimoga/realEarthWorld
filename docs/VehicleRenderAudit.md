# STAGE H0 — VEHICLE MODEL RENDERING & ASSET PIPELINE AUDIT
**Project:** Real Earth World Platform  
**Audit Document:** VehicleRenderAudit.md  
**Audit Stage:** Stage H0 (Forensic Runtime Audit & Quality Gates)  
**Date:** October 26, 2026  
**Auditing Authority:** Principal Graphics, Rendering & 3D Asset Engineering  
**Baseline Git Commit:** `7fafd8c9df1fea96513bbcd74e15b8336fda2eba`  
**Overall Finding:** **VEHICLE ASSET PIPELINE COLLAPSES MULTI-MATERIAL ASSETS INTO MONOCHROMATIC BOX PRIMITIVES (AUDIT FAILED)**

---

## 1. Executive Summary & Forensic Pipeline Trace

In live runtime gameplay of `realearth-viewer.exe`, vehicle models do not present the promised PBR multi-material fidelity:
- As photographed in **Screenshot 4**, the **CityLine Transit Bus** appears as a flat, solid, monochromatic cyan slab. Individual structural components—windshield glass, side windows, rubber tires, LED destination headers, door frames, and headlights—have disappeared into a monolithic cyan box.
- As photographed in **Screenshot 2**, the **Passenger Locomotive** appears as a primitive grey shoebox without bogies, couplers, or pantographs.
- As photographed in **Screenshot 5**, the **Commercial Airliner** is composed of crude extruded cylinders and flat planar boxes with zero airfoil camber.

### Asset-to-GPU Pipeline Trace:
```
[Asset Bytes on Disk] (assets/models/*.glb or procedural fallback)
       │
       ▼
[GLTF Importer] (engine/rendering/src/gltf_asset.rs)
  Parses nodes, meshes, primitives, vertex colors, PBR metallic-roughness
       │
       ▼
[Vehicle Mesh Composer] (engine/rendering/src/gltf_asset.rs: `compose_multimodal_world_mesh`)
  ❌ FLAW 1: `override_paint_color` unconditionally overwrites vertex colors for all
              vertices belonging to `VehicleSubmeshType::ChassisBody`, obliterating
              black window frames, dark grilles, headlights, and signs into cyan.
  ❌ FLAW 2: If external .glb asset fails loading, system silently falls back to
              `engine/rendering/src/vehicle_models.rs` procedural models.
       │
       ▼
[Procedural Model Generator] (engine/rendering/src/vehicle_models.rs: `build_cityline_transit_bus`)
  ❌ FLAW 3: Built using rectangular box primitives (`add_box_at`). Only 17.5% of
              surface normals are non-axis-aligned (<25% threshold), producing
              sharp orthogonal edges instead of curved aerodynamic body panels.
       │
       ▼
[Renderer Vertex Buffer & Pipeline] (engine/rendering/src/pipeline.rs)
  Draws 3,864 vertices with uniform cyan vertex color and flat normals.
```

---

## 2. Forensic Analysis of 7 Multimodal Vehicle Models

| Transport Mode | Visual Type | Source Asset | Runtime Geometry Type | Curved Normal Ratio | Vertex Color Palette Count | Observed Visual Defect | Forensic Audit Status |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Transit Bus** | `VehicleVisualType::TransitBus` | `build_cityline_transit_bus` | Procedural Box Composite | **17.5%** (Limit >25%) | 13 (Obscured by Cyan Override) | **Monochromatic Cyan Slab (Screenshot 4)**: Uniform paint color obliterates tinted glass and detail. | **REJECTED ❌** |
| **Train Consist** | `VehicleVisualType::Train` | `build_heavy_locomotive` | Procedural Box Composite | **11.2%** (Limit >25%) | 8 | **Crude Grey Box (Screenshot 2)**: Missing bogies, track couplers, wheels floating on water. | **REJECTED ❌** |
| **Aircraft** | `VehicleVisualType::Aircraft` | `build_commercial_airliner` | Procedural Cylinder / Box | **21.4%** (Limit >25%) | 9 | **Cardboard Cylinder (Screenshot 5)**: Primitive geometry; immediate stall warning at 15m elevation. | **REJECTED ❌** |
| **Sports Car** | `VehicleVisualType::SportsCoupe` | `build_sports_coupe_mesh` | Procedural Mesh | **28.6%** | 16 | Acceptable curvature, but steering yaw inverted relative to camera. | **PARTIAL ⚠️** |
| **Motorcycle** | `VehicleVisualType::Motorcycle` | `build_motorcycle_mesh` | Procedural Composite | **22.1%** | 10 | Coarse wheels and rider placeholder. | **REJECTED ❌** |
| **Heavy Truck** | `VehicleVisualType::HeavyTruck` | `build_semi_truck_mesh` | Procedural Composite | **19.8%** | 12 | Monochromatic cab override collapses grille and windshield. | **REJECTED ❌** |
| **Marine Vessel** | `VehicleVisualType::Marine` | `build_marine_ferry` | Procedural Box Composite | **16.4%** | 11 | Blocky hull with no hydrodynamic curvature. | **REJECTED ❌** |

---

## 3. Detailed Root Causes

### 1. Monochromatic Vertex Color Override
In `engine/rendering/src/gltf_asset.rs:480-486`:
```rust
if let Some(custom_color) = override_paint_color {
    for vertex in &mut composed.vertices {
        if vertex.submesh_type == VehicleSubmeshType::ChassisBody {
            vertex.color = custom_color; // ❌ Destroys tinted glass, headlights, and trim!
        }
    }
}
```
When `custom_color` (such as the default cyan/transit green `[0.10, 0.65, 0.40, 1.0]`) is applied, any vertex tagged as `ChassisBody` is uniformly painted cyan. Because the procedural model generator tagged windshield frames, roof spoilers, and destination headers as `ChassisBody`, the entire front and top of the bus collapsed into a single cyan block.

### 2. Lack of Curvature Normals in Procedural Generation
In `engine/rendering/src/vehicle_models.rs:2371` (`build_cityline_transit_bus`), the bus body is built using `add_box_at`:
- `add_box_at` generates orthogonal faces where normal vectors are strictly `[±1, 0, 0]`, `[0, ±1, 0]`, or `[0, 0, ±1]`.
- Out of 3,864 vertices, only 676 (17.5%) have non-axis-aligned normals (e.g. from the wheel arches).
- Over 82.5% of the vehicle surface is composed of 90-degree planar rectangles, giving the bus an unmistakable low-poly "shoebox" appearance.

---

## 4. Quality Gate Acceptance Standards

For Stage H4 vehicle asset remediation:
1. `test_h0_gate_vehicle_rendering_visual_fidelity` must pass:
   - Composed mesh must exhibit at least 4 visually distinct surface color clusters in the final draw call (Body Paint, Tinted Glass, Dark Trim/Rubber, Emissive Lights).
   - Greater than 25.0% of surface vertices must feature continuous curved normals.
2. `override_paint_color` must strictly affect body panels, preserving tinted glass (`alpha < 0.8`), rubber wheels, and chrome/black trim.
3. Authentic artist-authored glTF models (`.glb`) must load with multi-material PBR shaders, falling back to procedural meshes only with explicit logged warnings.
