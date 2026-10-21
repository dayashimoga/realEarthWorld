# Real Earth World Platform — Setup & Configuration

## 1. Prerequisites

- **Rust**: Version 1.80+ or latest stable (tested on Rust 1.99.1).
- **Git**: 2.30+.
- **Vulkan / DirectX 12 / Metal**: GPU with support for WebGPU / wgpu standard.
- **SQLite 3**: Bundled automatically via `rusqlite` feature flag.

---

## 2. Quickstart & One-Command Workflows

### 2.1 Build Workspace
```bash
cargo build --workspace
```

### 2.2 Run Full Test Suite
```bash
cargo test --workspace
```

### 2.3 Run Linter & Formatting Checks
```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
```

### 2.4 Run Headless World Ingestion & Compilation (Bengaluru Sample)
```bash
cargo run --bin realearth-cli -- compile --input assets/bengaluru_sample --output target/world_bengaluru
```

### 2.5 Launch Interactive World Viewer
```bash
cargo run --bin realearth-viewer
```

---

## 3. Configuration Parameters (`realearth.toml`)

```toml
[world]
region = "bengaluru_central"
bbox = [12.9600, 12.9850, 77.5800, 77.6150] # min_lat, max_lat, min_lon, max_lon
chunk_size_meters = 500.0

[streaming]
max_memory_mb = 1024
view_distance_meters = 3500.0
worker_threads = 4

[rendering]
backend = "auto" # "vulkan", "dx12", "metal", "auto"
vsync = true
msaa_samples = 4
wireframe_debug = false

[simulation]
tick_rate_hz = 60
time_scale = 1.0 # 1.0 = real time
utc_offset_hours = 5.5 # Indian Standard Time (IST)
```
