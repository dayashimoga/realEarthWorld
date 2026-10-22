# ADR-0019: Stage C — Visual Fidelity and Realism Certification

## Status
Accepted

## Context
Following the successful completion of Stage B (Adversarial and Independent Validation), the Real Earth Open-World Platform must undergo comprehensive visual fidelity and realism certification (Stage C). While earlier phases verified topological graph correctness, compiler reproducibility, and kinematic physics equations, true visual realism requires verifying that the rendered scenes look authentic, georeferenced, aesthetically balanced, and free from immersion-breaking visual glitches.

Specifically, Stage C mandates:
1. **C1 — Real-World Geography**: Comparing rendered landmarks, street grids, grade-separated bridges, coastlines, and hills against independent authoritative survey records, elevation datums, and dated ground truth. Correctly differentiating source-confirmed geometric details from procedural visual approximations.
2. **C2 — Geometry & Placement**: Detecting and repairing floating/sinking objects, overlapping buildings, roads slicing through buildings, substandard flyover clearances, excessive road slopes, shoreline elevation tears, and vegetation encroaching onto roadways.
3. **C3 — Graphics & Materials**: Auditing terrain meshes, architectural textures, procedural road markings (double yellow centerlines, white dashed lane stripes), realistic vegetation, aquatic water shaders with Fresnel reflectance, and solar ephemeris lighting (noon, golden hour, twilight, night).
4. **C4 — Rendering Correctness**: Verifying shader pipeline execution, depth precision (`Depth32Float` to prevent planetary z-fighting), occlusion culling, asset loading, resolution changes, dynamic quality presets (Low, Medium, High, Adaptive), and frame pacing.
5. **C5 — Visual Regression & Perceptual Testing**: Deterministic off-screen framebuffer captures from fixed camera poses, altitudes, and orientations; automated image difference metrics (MSE, PSNR, SSIM, non-zero pixel coverage, mean luminance); and formal human review protocols.
6. **C6 — Cross-Platform UI/UX**: Auditing UI responsiveness, readability, accessibility (WCAG AA contrast $\ge 4.5:1$), touch target sizes ($\ge 48 \times 48\text{ dp}$), desktop keyboard event loop latency ($< 16.67\text{ ms}$), and loading/error feedback.

## Decision

We establish an end-to-end Visual Fidelity and Realism Certification framework comprising:

### 1. Scene Placement Auditor (`engine/validation/src/scene_placement_auditor.rs`)
- Implements automated spatial queries detecting:
  - Floating buildings: base elevation $> h_{\text{DEM}} + 1.5\text{ m}$.
  - Sunken buildings: roof elevation $\le h_{\text{DEM}}$.
  - Overlapping buildings: polygon footprint intersection without party wall flags.
  - Road-through-structure: at-grade road polyline intersecting building envelope.
  - Excessive road slopes: grade $> 25\%$ (urban) or $> 12\%$ (motorway).
  - Insufficient flyover clearance: overpass clearance $< 4.5\text{ m}$.
  - Coastal water misalignment: water surface vs shoreline terrain $> 8.0\text{ m}$.
- Provides automated controlled repairs (`repair_scene_placement`) that snap building foundations to local terrain and elevate flyover spans.

### 2. WGSL PBR Shader Pipeline & Material Audit (`engine/rendering`)
- Validates bidirectional reflectance distribution function (BRDF) in `WORLD_SHADER_WGSL`:
  - Diffuse illumination modulated by solar zenith angle and day factor.
  - Procedural road markings enabled for asphalt ribbon fragments: double yellow median line ($v \in [0.485, 0.515]$) and dashed white lane lines ($v \in [0.238, 0.262], [0.738, 0.762]$ with periodic dash frequency).
  - Fresnel reflectance ($F_0 = 0.04$) on water surfaces and glazed architectural facades.
  - Rayleigh distance haze and sunset/twilight color grading.

### 3. Dedicated Test Suite (`tests/tests/visual_fidelity_certification.rs`)
We author 6 comprehensive test suites covering C1 through C6:
- `test_c1_georeferenced_landmark_and_topography_ground_truth`: Audits positions, elevations, and scale of 14 iconic landmarks across 7 representative metropolitan regions against CTBUH & USGS references.
- `test_c2_geometric_placement_and_anomaly_detection`: Tests automated detection and repair of floating structures, road-building collisions, bridge clearances, and unlevel water bodies.
- `test_c3_graphics_materials_lighting_and_atmosphere`: Audits WGSL PBR shaders, procedural road marking shaders, Fresnel water reflectance, and atmospheric Rayleigh haze across solar ephemeris hours.
- `test_c4_rendering_pipeline_depth_precision_and_presets`: Tests headless rendering across Low, Medium, High, and Adaptive presets; verifies depth buffer allocation (`Depth32Float`), frustum culling effectiveness, and resolution resize.
- `test_c5_deterministic_visual_regression_and_image_metrics`: Renders deterministic offscreen framebuffers for NYC, SF, and Seattle; runs image analysis (mean luminance, RGB standard deviation, non-zero pixel ratio) and perceptual metrics (MSE, PSNR, SSIM comparisons).
- `test_c6_ui_ux_accessibility_and_cross_platform_responsiveness`: Audits desktop keyboard event latency, mobile touch target sizes ($48\times 48\text{dp}$ minimum), HUD text contrast ratio ($\ge 4.5:1$ per WCAG AA), and error/loading state feedback.

### 4. Headless Multi-City Visual Capture
- Upgraded `apps/world-viewer` to accept dynamic `--screenshot <path>` and generate region-specific PPM snapshots (`target/screenshots/{region}_view.ppm`).
- Automated capture runs across representative cities (New York, San Francisco, Chicago, Seattle, Miami, Los Angeles, Bengaluru) to provide permanent visual evidence.

## Consequences

- Real-world geography is audited against authoritative ground truth, clearly distinguishing surveyed facts from procedural approximations.
- Immersion-breaking geometric placement defects (floating buildings, roads slicing through buildings) are prevented during ingestion and compilation.
- Visual regression is verified mathematically through perceptual metrics (SSIM, PSNR) and backed by documented human review criteria.
- Cross-platform UI standards conform to WCAG 2.1 AA accessibility and platform guidelines.
