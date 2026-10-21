# ADR 0009: Production World Streaming and Performance Optimization Architecture

## Status
Accepted

## Context
Stage 5 of the Global Engineering Contract mandates **Production World Streaming and Performance Optimization**. The Real Earth World Platform is designed to scale from municipal districts (San Francisco downtown, Bengaluru central) to planetary scale with millions of managed spatial features. To achieve the 60 FPS reference target ($\le 16.67\text{ ms/frame}$) under a strict resident memory budget ($\le 1024\text{ MB}$ RSS) with sub-2-second startup times, the runtime streaming and rendering architecture must evolve beyond naive synchronous on-demand chunk loading.

Real-world interactive navigation involves dynamic velocity, sharp turns, high-speed highway travel (e.g., $120\text{ km/h}$ on the I-80 Bay Bridge approach), and instant camera teleportation. Under these conditions, naive streaming suffers from:
1. **Render-thread blocking**: Synchronous file I/O and zstd decompression on the main render thread causing severe frame stalls ($> 50\text{ ms}$).
2. **LOD thrashing (ping-pong popping)**: Chunks repeatedly swapping between LOD0 and LOD1 when a camera hovers near the distance boundary.
3. **Runaway memory allocation & cache churn**: Loading chunks into RAM without accounting for both CPU heap and GPU VRAM footprint.
4. **Stale I/O queues upon teleportation**: Wasted I/O bandwidth and CPU cycles decompressing chunks that are no longer in view after an instant landmark jump.
5. **Frame buffer churn**: Recreating `wgpu::Buffer` allocations every frame instead of caching persistent GPU buffers for active chunks.
6. **Premature physics eviction**: Evicting chunks currently hosting active vehicle wheels or collision contacts.

## Decisions

### 1. Hierarchical Geographic Indexing & Spatial Multi-LOD (`realearth-streaming`)
- Chunks are organized into hierarchical spatial levels ($L \in \{0, 1, 2\}$):
  - **LOD0 (Fine)**: $500\text{m}$ cells, detailed 3D building geometry, road ribbons, curbs, vegetation zones, full terrain mesh.
  - **LOD1 (Medium)**: $1000\text{m}$ cells, simplified building footprints, primary arterial road network, decimated terrain.
  - **LOD2 (Regional)**: $2000\text{m}$ cells, regional topography, major highway corridors, sea/water boundaries.
- The SQLite `chunks.db` and in-memory spatial index store bounding boxes, LOD levels, and neighbor adjacency links for seam stitching.

### 2. Velocity- and Direction-Aware Prefetching Lookahead Cone
- The streaming query computes the camera velocity vector $\mathbf{v} = (v_x, v_y, v_z)$ from consecutive frames.
- A lookahead cone extends in the direction of motion: $\mathbf{p}_{\text{lookahead}} = \mathbf{p}_{\text{cam}} + \mathbf{v} \cdot t_{\text{lookahead}}$ ($t_{\text{lookahead}} \approx 2.0\text{ s}$).
- The query bounding box is constructed as an anisotropic bounding volume: a radial baseline radius ($r_{\text{base}}$) covering $360^\circ$ immediate surroundings plus an elongated forward lobe proportional to $\|\mathbf{v}\|$.
- Candidate chunks within the forward lookahead cone are enqueued before entering the visual frustum.

### 3. Prioritized Cancellable Tokio I/O with Rayon Background Decoding
- Chunk loading tasks are managed by an asynchronous streaming pipeline (`AsyncStreamingManager`):
  - **Prioritized Work Queue**:
    1. `ImmediateFrustum / CriticalCollision`: Chunks currently in the viewing frustum or directly beneath the camera/vehicle ($< 300\text{m}$).
    2. `VelocityCone`: Chunks along the forward trajectory predicted to be visible within $2\text{ seconds}$.
    3. `PeripheralSurrounding`: Chunks within outer radius for smooth peripheral rotation.
  - **Cancellation Tokens**: Each in-flight prefetch request is associated with a cancellation token. When the camera teleports (distance delta $> 500\text{m}$ in a single tick) or changes heading abruptly, all pending stale prefetch jobs are instantly aborted.
  - **Rayon Thread Pool**: File reading uses Tokio non-blocking file I/O; zstd decompression and mesh unpacking execute in parallel on Rayon worker threads, ensuring the render thread is never blocked.
  - **Bounded Backpressure**: A maximum in-flight task limit (e.g., 16 parallel tasks) prevents unbounded queue growth and memory spikes during high-velocity traversal.

### 4. Dual CPU/GPU Memory Accounting & Collision-Critical Pinning
- `LruChunkCache` tracks both:
  - `cpu_bytes`: Vertex/index data, building metadata, and road graph structures in RAM.
  - `gpu_bytes`: Resident VRAM allocated for vertex and index buffers on the graphics device.
- **Collision-Critical Pinning (`pinned: bool`)**: Chunks containing the immediate camera location or active physics collision bodies are marked pinned. The LRU eviction algorithm will never evict a pinned chunk, preventing collision drop-through bugs.
- Eviction selects the oldest, unpinned, non-visible chunks farthest from the camera when total memory exceeds `StreamingConfig::max_memory_mb * eviction_threshold_ratio`.

### 5. LOD Distance Hysteresis & Seam Continuity
- To eliminate LOD boundary popping, LOD selection employs a deadband hysteresis buffer ($\Delta_{\text{hyst}} = 100\text{m}$):
  - If a chunk is currently at LOD0, it only transitions to LOD1 if distance $> D_{\text{threshold}} + \Delta_{\text{hyst}}$.
  - If a chunk is currently at LOD1, it only transitions to LOD0 if distance $< D_{\text{threshold}} - \Delta_{\text{hyst}}$.
- Seam stitching: Adjacent chunk boundary vertices share identical coordinate and elevation values, guaranteed by the compiler's boundary snapping rules and verified by the `ChunkIntegrityValidator`.

### 6. Persistent GPU Buffer Caching & Graphics Quality Presets (`realearth-rendering`)
- **Persistent GPU Buffers**: `GpuChunkCache` maintains resident `wgpu::Buffer` handles for each active chunk's terrain, road, building, water, and vegetation meshes. Chunks are uploaded to GPU memory asynchronously once upon load; frames render from persistent buffers with zero per-frame allocation overhead.
- **Graphics Quality Presets**:
  - `Low`: View distance $1500\text{m}$, LOD threshold $800\text{m}$, lightweight shading, disabled specular water reflections.
  - `Medium`: View distance $2500\text{m}$, LOD threshold $1500\text{m}$, standard Blinn-Phong lighting, distance fog.
  - `High`: View distance $4000\text{m}$, LOD threshold $2200\text{m}$, full specular sun highlights, realistic road markings, atmospheric haze.
  - `Adaptive`: Automatic frame-time regulator monitoring running p95 frame latency. If frame time exceeds $16.0\text{ ms}$, view distance and LOD distances are pulled in dynamically; if frame time remains $< 12.0\text{ ms}$, fidelity is scaled up.
- **Shader Upgrades**: WGSL shader updated with road markings (white lane dividers, yellow centerlines), dynamic sun/moon ephemeris lighting, sky haze gradient, and specular Fresnel water shading.

### 7. Profiling, Telemetry & Planetary Scale Demonstration
- Built-in telemetry tracking:
  - Percentile frame times: p50, p95, p99 (ms).
  - Active draw calls, CPU frame time, GPU time, RSS memory, and GPU VRAM.
  - Cache hit rate, miss rate, eviction rate, and streaming queue stalls.
- Automated benchmarks:
  - Dense Urban Scenario (Downtown San Francisco financial district).
  - High-Speed Highway Scenario ($120\text{ km/h}$ Bay Bridge traversal).
  - Rapid Teleportation Scenario (instant landmark jumps).
  - Planetary Scale Demonstration: 100K and 1M managed objects in the spatial streaming index maintaining $\ge 60\text{ FPS}$ and $< 1024\text{ MB}$ RSS.

## Consequences
- Guaranteed smooth 60 FPS traversal without main-thread I/O stalls or memory leaks.
- Zero visual ping-pong popping at LOD boundaries due to hysteresis.
- Constant, deterministic memory consumption adhering to strict platform budgets.
- Proven scalability to 1,000,000 managed objects while maintaining responsive interactive performance.
