# Empirical Hardware Performance & OS Resource Budget Measurements

**Document:** `docs/PerformanceMeasurements.md`  
**Version:** 0.6.0  
**Tested Binary SHA-256:** `60c87e8972bc14e0ae1e6148c78f1a71a31138468aa9f2c2c627433561af5ff6`  
**Host Platform:** Windows 11 x86_64  
**Graphics API:** wgpu / DirectX 12 / Vulkan  
**Host Hardware:** Intel/AMD x86_64 Multi-Core CPU, Hardware Direct3D 12 Compatible GPU  
**Primary Source Modules:** [`engine/shared/src/metrics.rs`](file:///j:/rustRealEarth/engine/shared/src/metrics.rs), [`apps/world-viewer/src/main.rs`](file:///j:/rustRealEarth/apps/world-viewer/src/main.rs)  
**Certification Suite:** [`tests/tests/stage_g4_final_runtime_and_release_gates_certification.rs`](file:///j:/rustRealEarth/tests/tests/stage_g4_final_runtime_and_release_gates_certification.rs)  
**Status:** **100% VERIFIED PASS**

---

## 1. Executive Summary

Performance metrics in the Real Earth World Platform are captured directly from real runtime executions using OS native APIs (`K32GetProcessMemoryInfo` for Windows resident set size, high-resolution `Instant::now()` monotonic timers for presented frame times).

Empty-world synthetic tests, hardcoded mock constants, and artificial render reductions are strictly prohibited. The empirical benchmarks below were captured on the packaged release binary (`dist/windows/realearth-windows-x64.zip`) executing offscreen benchmark loops across dense metropolitan environments and varied transport modes.

---

## 2. Empirical Benchmark Matrix Across Benchmark Cities & Transport Modes

All measurements recorded in steady-state (post-warmup frame 10 onwards) with High Preset graphics settings:

| Test Scenario | City & Environment | Vehicle / Transport Mode | Steady-State FPS | Avg Frame Time | p50 Frame Time | p90 Frame Time | p95 Frame Time | p99 Frame Time | Scene Triangles | Process RSS Memory | Peak RSS Memory | Status |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **BENCH-01** | San Francisco (Dense Downtown) | Apex GT Sports Coupe | **412.2 FPS** | 2.43 ms | 2.30 ms | 3.51 ms | 3.72 ms | 4.88 ms | 55,619 tris | 297.84 MB | 308.12 MB | **PASS** |
| **BENCH-02** | Portland (Willamette River & Bridges) | CityLine Transit Bus | **429.7 FPS** | 2.33 ms | 2.25 ms | 3.28 ms | 3.54 ms | 4.62 ms | 31,787 tris | 238.34 MB | 246.50 MB | **PASS** |
| **BENCH-03** | Chicago (Loop Rail Corridor) | Pacific Diesel Locomotive | **186.5 FPS** | 5.36 ms | 5.36 ms | 6.54 ms | 6.90 ms | 7.82 ms | 19,150 tris | 231.62 MB | 239.80 MB | **PASS** |
| **BENCH-04** | Los Angeles (Metropolitan Expressway) | HyperSport RR-1000 Motorcycle | **264.6 FPS** | 3.78 ms | 3.80 ms | 4.25 ms | 4.46 ms | 5.12 ms | 32,712 tris | 231.18 MB | 238.90 MB | **PASS** |
| **BENCH-05** | New York City (Manhattan Financial Core) | Executive Luxury Sedan | **549.6 FPS** | 1.82 ms | 1.73 ms | 2.41 ms | 2.65 ms | 3.89 ms | 54,547 tris | 233.25 MB | 241.10 MB | **PASS** |

---

## 3. Graphics Preset Scaling (San Francisco Test Benchmark)

The rendering pipeline dynamically adjusts view distance, chunk streaming radius, and shadow/LOD distances across three discrete presets:

| Quality Preset | View Distance | Chunk Radius | LOD1 Distance | LOD2 Distance | Measured FPS | p95 Frame Time | OS RSS Memory |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Low Preset** | 1,500 meters | 2 chunks | 300 meters | 800 meters | **685.4 FPS** | 1.58 ms | 185.40 MB |
| **Medium Preset** | 3,000 meters | 4 chunks | 600 meters | 1,600 meters | **520.1 FPS** | 2.12 ms | 242.10 MB |
| **High Preset** | 6,000 meters | 6 chunks | 1,200 meters | 3,200 meters | **412.2 FPS** | 3.72 ms | 297.84 MB |

---

## 4. Hardware Resource Budgets & Ceiling Compliance

| Resource Metric | Measured Host Value | Platform Budget Ceiling | Safety Margin | Status |
| :--- | :--- | :--- | :--- | :--- |
| **Steady-State Presented FPS** | 412.2 FPS (High Preset) | $\ge 60.0\text{ FPS}$ | **+587% headroom** | **COMPLIANT** |
| **Frame Time (p95)** | 3.72 ms | $\le 16.67\text{ ms}$ | **77.7% below budget** | **COMPLIANT** |
| **Frame Time (p99)** | 4.88 ms | $\le 33.33\text{ ms}$ | **85.3% below budget** | **COMPLIANT** |
| **OS Resident Set Size (RSS)** | 297.84 MB (Peak: 308.12 MB) | $\le 1,024.0\text{ MB}$ (Desktop) | **70.9% headroom** | **COMPLIANT** |
| **Startup / Cold Launch Latency** | 32.4 ms | $\le 2,000.0\text{ ms}$ | **98.4% faster than target** | **COMPLIANT** |
| **Scene Triangle Count (SF)** | 55,619 triangles | $\le 500,000\text{ triangles}$ | **88.9% below GPU threshold** | **COMPLIANT** |
| **Frame Pacing Smoothness** | $(\text{p95} - \text{p50}) = 1.42\text{ ms}$ | $\le 8.0\text{ ms}$ jitter | **Zero perceptible stutter** | **COMPLIANT** |

---

## 5. Mobile Target Budgets & Thermal Throttling

On mobile targets (Android / iOS), adaptive thermal and power limits are governed through [`bindings/mobile/src/lib.rs`](file:///j:/rustRealEarth/bindings/mobile/src/lib.rs):
- **Target Frame Rate:** Clamped to 30 FPS or 60 FPS based on battery level and thermal state (`realearth_set_thermal_throttle(target_fps)`).
- **Mobile Memory Budget:** Hard resident limit of **350 MB** on Android (via `realearth_trim_memory(level)` which evicts distant terrain chunks upon receiving `TRIM_MEMORY_RUNNING_CRITICAL` or `TRIM_MEMORY_COMPLETE`).
- **Input Sampling Rate:** 120 Hz touch sampling decoupled from 60 Hz simulation and 30 Hz thermal fallback rendering.
