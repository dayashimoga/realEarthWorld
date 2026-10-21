# Real Earth World Platform — Performance & Profiling Strategy

## 1. Quantitative Performance Budgets (Bengaluru MVP)

| Metric | Budget | Target | Hard Failure Threshold |
|---|---|---|---|
| First Usable Frame (from cold start cache) | $\le 2.0\text{ s}$ | $1.2\text{ s}$ | $> 3.0\text{ s}$ |
| Resident Set Size (Process RAM) | $\le 1024\text{ MB}$ | $650\text{ MB}$ | $> 1200\text{ MB}$ |
| Frame Rate (Reference Hardware) | $\ge 60\text{ FPS}$ | $60\text{ FPS}$ | $< 55\text{ FPS}$ sustained |
| Frame Time (p50) | $\le 14.0\text{ ms}$ | $11.0\text{ ms}$ | $> 16.67\text{ ms}$ |
| Frame Time (p95) | $\le 18.0\text{ ms}$ | $15.0\text{ ms}$ | $> 20.0\text{ ms}$ |
| Frame Time (p99) | $\le 22.0\text{ ms}$ | $18.0\text{ ms}$ | $> 25.0\text{ ms}$ |
| Chunk Decompression Throughput | $\ge 250\text{ MB/s}$ | $400\text{ MB/s}$ | $< 100\text{ MB/s}$ |
| GPU Draw Calls per Frame | $\le 1500$ | $450$ | $> 3000$ |

---

## 2. Zero-Stutter Streaming & Geometry Optimization

1. **Geometry Batching**:
   - Merge all buildings within a chunk into a single contiguous vertex and index buffer.
   - Roads in a chunk are merged into a single draw call.
   - Terrain heightfield in a chunk is rendered as a single triangle strip or indexed mesh.
2. **Double-Buffered Asynchronous Uploads**:
   - Main render thread never calls blocking decompression or disk I/O.
   - Chunks are deserialized and decompressed on Rayon threads; staging buffers are prepared before passing pointers to the wgpu render loop.
3. **Floating Origin Stabilization**:
   - Floating origin resets only occur when the camera departs more than $5000\text{m}$ from the current origin, avoiding per-frame rebasing overhead.
4. **Criterion Micro-benchmarks**:
   - Geodetic coordinate transforms.
   - Bowring ECEF-to-Geodetic iterative vs closed-form performance.
   - Building polygon triangulation via `earcutr`.
   - Zstd decompression speed.
   - A* navigation pathfinding across 10,000 lane segments.
