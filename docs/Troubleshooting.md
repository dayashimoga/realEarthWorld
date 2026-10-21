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
