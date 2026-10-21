# Real Earth World Platform — User Guide

## 1. Overview
The Real Earth Viewer (`realearth-viewer`) and Command Line Tool (`realearth-cli`) provide high-performance exploration, inspection, and compilation of real-world planetary regions.

---

## 2. World Viewer Controls

| Input | Action |
|---|---|
| **W / A / S / D** | Fly camera Forward / Left / Backward / Right |
| **Space / Shift** | Ascend / Descend vertically |
| **Right Mouse Drag** | Pitch and Yaw camera orientation |
| **Mouse Scroll** | Adjust camera flight speed ($10\text{ m/s} \leftrightarrow 500\text{ m/s}$) |
| **F1** | Toggle Real-time Diagnostic HUD (FPS, memory, chunks loaded, draw calls) |
| **F2** | Toggle Wireframe debug rendering |
| **F3** | Cycle Sun Time-of-Day (Dawn $\to$ Noon $\to$ Dusk $\to$ Midnight) |
| **R** | Reset camera to Bengaluru Vidhana Soudha anchor |
| **Esc** | Exit viewer |

---

## 3. Real Earth CLI (`realearth-cli`)

### Commands:
- `ingest`: Parse raw OSM XML and DEM files into normalized GIS feature records.
- `validate`: Perform topological and grade separation analysis, outputting a verification report and quarantine log.
- `compile`: Build 3D meshes (terrain, buildings, roads) and pack into `.rec` chunk files with a `chunks.db` SQLite index.
- `inspect-chunk`: Read and inspect a specific `.rec` binary chunk file.
- `bench`: Run runtime streaming and coordinate transformation micro-benchmarks.
