# Real Earth World Platform — User Guide

## 1. Overview
The Real Earth Viewer (`realearth-viewer`) and Command Line Tool (`realearth-cli`) provide high-performance exploration, inspection, and compilation of real-world planetary regions.

---

## 2. World Viewer Controls & Modes

### 2.1 Camera & Simulation Modes
Press **C** to toggle between **Fly Mode**, **Drive Mode**, and **Walk Mode**.

| Key / Input | Fly Mode Action | Drive Mode Action | Walk Mode Action |
| :--- | :--- | :--- | :--- |
| **W / S** | Move Forward / Backward | Vehicle Throttle / Brake-Reverse | Walk Forward / Backward |
| **A / D** | Strafe Left / Right | Steer Left / Right | Strafe Left / Right |
| **Space** | Ascend vertically | Handbrake (E-Brake) | Jump |
| **Shift** | Descend vertically | Boost / Nitro Acceleration | Sprint |
| **Right Mouse Drag** | Orbit / Look around | Chase Camera Angle | First-Person View |
| **Mouse Scroll** | Flight Speed adjustment | Camera Distance | Head Zoom |

### 2.2 Gameplay & Career Contract Controls
- **C**: Cycle camera / simulation mode (Fly $\leftrightarrow$ Walk $\leftrightarrow$ Drive).
- **T / G**: Cycle vehicle fleet preset forward / backward (Coupe $\to$ Sedan $\to$ SUV $\to$ Taxi $\to$ Bus $\to$ Heavy Truck).
- **J**: Cycle active contract assignment (Taxi Fare, Courier Delivery, Heavy Freight, Free Roam).
- **F1**: Toggle Diagnostic HUD overlay (Speedometer, Compass, Contracts, OS Working Set RSS, 1% Low FPS).
- **F2**: Toggle Wireframe debug mesh view.
- **F3**: Cycle lighting presets (Day $\to$ Sunset $\to$ Night $\to$ Overcast).
- **F5**: **Quicksave** game state (player position, wallet balance, active contract, driver XP).
- **F9**: **Quickload** deterministic game state restore.
- **R**: Reset vehicle / camera to regional anchor (San Francisco Ferry Building / Bay Bridge).
- **Esc**: Exit viewer cleanly.

---

## 3. Real Earth CLI (`realearth-cli`)

### Commands:
- `compile`: Build 3D meshes (terrain, buildings, roads) and pack into `.rec` chunk files with a `chunks.db` SQLite index.
- `validate`: Perform topological and grade separation analysis, outputting a verification report and quarantine log.
- `inspect-chunk`: Read and inspect a specific `.rec` binary chunk file.
- `catalog`: Manage regional world packages (list, verify, install).
- `job`: Browse and inspect available career contracts and freight jobs.
- `bench`: Run runtime streaming, memory, and coordinate transformation benchmarks.
- `help`: Display complete CLI documentation.

