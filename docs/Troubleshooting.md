# Real Earth World Platform — Troubleshooting Guide

## 1. Common Issues & Diagnoses

### Issue 1: wgpu Adapter Not Found (Headless CI / Virtual Environments)
- **Symptom**: `Failed to request wgpu adapter` or `No suitable GPU found`.
- **Cause**: CI or container environment lacks a physical graphics device.
- **Resolution**: Enable software rasterization fallback (`wgpu::Backends::all()` or SwiftShader/llvmpipe), or run headless simulation tests (`cargo test --workspace --lib`). Headless rendering uses dummy or offscreen buffers.

### Issue 2: Jittering Buildings Far from (0, 0, 0)
- **Symptom**: Geometry shivers or trembles as the camera moves.
- **Cause**: Bypassing the `FloatingOrigin` system and submitting raw ECEF double coordinates cast directly to single-precision float without origin rebasing.
- **Resolution**: Always subtract `FloatingOrigin.anchor_ecef` from object world coordinates before casting to `f32` GPU uniform matrices.

### Issue 3: Grade Separation Artifacts (Bridge merged with ground highway)
- **Symptom**: An elevated expressway is snapped down to ground level at an underpass crossing.
- **Cause**: Naive node snapping applied across different `layer` values.
- **Resolution**: The `realearth-validation` engine strictly forbids welding nodes between features where `layer_a != layer_b` or `bridge_a != bridge_b`. Verify `ValidationReport.quarantined_features`.

### Issue 4: Black / Blank Screen on Window Startup
- **Symptom**: Window opens but displays only a solid black or blank frame.
- **Cause**: Rendering pipeline was disconnected from window lifecycle; surface swapchain lost or unconfigured.
- **Resolution**: In Stage 1, `apps/world-viewer/src/interactive.rs` implemented full `winit::application::ApplicationHandler` surface lifecycle management with automatic recreation on resize, DPI change, and surface error handling (`Timeout`, `Outdated`, `Lost`, `OutOfMemory`).

### Issue 5: Vehicle Sinking into Ground or Tunneling
- **Symptom**: Player car or traffic fleet sinks beneath terrain or drops through road bridges.
- **Cause**: Missing DEM surface normal alignment and lack of restorative vertical spring forces.
- **Resolution**: In Stage 3, `engine/vehicles/src/ground.rs` implemented 4-wheel bilinear DEM normal sampling, signed penetration depth calculation, progressive non-linear bump-stop springs, and an absolute physical chassis safety clamp.

### Issue 6: Missing Regional Data in Standalone Mode
- **Symptom**: Executable starts but reports missing chunks or loads a flat plane.
- **Cause**: Running outside repository root without bundled data directory.
- **Resolution**: In Stage 4, `resolve_world_package_dir` was updated to search relative to `std::env::current_exe().parent()`. Ensure the bundled `data/world_san_francisco/` directory is extracted alongside `realearth-viewer.exe`, or unpack `dist/regions/realearth-region-san-francisco.zip` into `data/`.

