# Canonical Coordinate & Vehicle Control Architecture (Stage B)

**Project:** Real Earth World — Navigable Planetary Simulation  
**Document:** `docs/CanonicalCoordinateArchitecture.md`  
**Governance Phase:** STAGE B — AUTHORITATIVE VEHICLE PHYSICS AND INPUT CORRECTNESS  
**Authoritative Date:** 2026-10-25  

---

## 1. Single Authoritative Coordinate Convention

To prevent recurring transform errors, sign inversions, and multi-system compensating hacks, the entire Real Earth World platform strictly implements a single canonical coordinate system across geographic ingestion, world meshing, rigid-body physics, vehicle dynamics, rendering, and camera projection.

### 1.1 World Coordinate Frame (Geographic ENU)
* **Handedness:** Right-Handed Cartesian coordinate system ($X \times Y = Z$).
* **World Up Vector:** $+Y = (0, 1, 0)$ pointing toward the sky (zenith), aligned with the local gravity vertical. Gravity vector $\vec{g} = (0, -9.81, 0)\text{ m/s}^2$.
* **East Axis:** $+X = (1, 0, 0)$ pointing toward geographic East.
* **North Axis:** $+Z = (0, 0, 1)$ pointing toward geographic North.
* **South Axis:** $-Z = (0, 0, -1)$ pointing toward geographic South.
* **West Axis:** $-X = (-1, 0, 0)$ pointing toward geographic West.

$$\text{Basis Matrix: } \mathbf{I}_3 = \begin{bmatrix} \hat{e}_{\text{East}} \\ \hat{e}_{\text{Up}} \\ \hat{e}_{\text{North}} \end{bmatrix} = \begin{bmatrix} 1 & 0 & 0 \\ 0 & 1 & 0 \\ 0 & 0 & 1 \end{bmatrix}$$

### 1.2 Vehicle Local Body Frame
The vehicle local coordinate frame is anchored at the chassis geometric origin (with center-of-mass vertical offset $y_{\text{com}}$):
* **Forward Axis ($\hat{f}$):** Local $+Z = (0, 0, 1)$ pointing toward the front bumper along the longitudinal centerline.
* **Right Axis ($\hat{r}$):** Local $+X = (1, 0, 0)$ pointing toward the passenger/right side along the lateral axle.
* **Up Axis ($\hat{u}$):** Local $+Y = (0, 1, 0)$ pointing toward the roof along the chassis vertical.
* **Right-Hand Rule Verification:** $\hat{r} \times \hat{u} = (1, 0, 0) \times (0, 1, 0) = (0, 0, 1) = \hat{f}$. Handedness is strictly preserved.

### 1.3 Heading, Yaw Angle & Orientation
* **Euler Yaw Angle ($\psi$):** Single rotation around the $+Y$ (Up) axis:
  $$\mathbf{R}(\psi) = \begin{bmatrix} \cos \psi & 0 & \sin \psi \\ 0 & 1 & 0 \\ -\sin \psi & 0 & \cos \psi \end{bmatrix}$$
* **Forward Direction in World Space:**
  $$\vec{f}_{\text{world}} = \mathbf{R}(\psi) \begin{bmatrix} 0 \\ 0 \\ 1 \end{bmatrix} = \begin{bmatrix} \sin \psi \\ 0 \\ \cos \psi \end{bmatrix}$$
* **Right Direction in World Space:**
  $$\vec{r}_{\text{world}} = \mathbf{R}(\psi) \begin{bmatrix} 1 \\ 0 \\ 0 \end{bmatrix} = \begin{bmatrix} \cos \psi \\ 0 \\ -\sin \psi \end{bmatrix}$$
* **Cardinal Orientations:**
  - $\psi = 0\text{ rad}$ ($0^\circ$): Forward is North $\vec{f} = (0, 0, +1)$, Right is East $\vec{r} = (+1, 0, 0)$.
  - $\psi = +\frac{\pi}{2}\text{ rad}$ ($+90^\circ$): Forward is East $\vec{f} = (+1, 0, 0)$, Right is South $\vec{r} = (0, 0, -1)$.
  - $\psi = \pi\text{ rad}$ ($180^\circ$): Forward is South $\vec{f} = (0, 0, -1)$, Right is West $\vec{r} = (-1, 0, 0)$.
  - $\psi = -\frac{\pi}{2}\text{ rad}$ ($-90^\circ$): Forward is West $\vec{f} = (-1, 0, 0)$, Right is North $\vec{r} = (0, 0, +1)$.

---

## 2. Canonical Control Input Proofs

All vehicle controls operate strictly in the vehicle's body-fixed coordinate frame and map deterministically to world space, completely independent of the camera perspective (chase, hood, cockpit, or free-look).

### 2.1 Forward Motion ([W] / Up Arrow)
* **Authoritative Input:** `road_inputs.throttle = 1.0, road_inputs.gear_mode = TransmissionMode::Drive`.
* **Longitudinal Force:** $\vec{F}_{\text{drive}} = +F_{\text{engine}} \cdot \vec{f}_{\text{world}}$.
* **Acceleration:** $\vec{a} = \frac{\vec{F}_{\text{drive}}}{m} = +\frac{F_{\text{engine}}}{m} \begin{bmatrix} \sin \psi \\ 0 \\ \cos \psi \end{bmatrix}$.
* **Proof Across Cardinal Headings:**
  1. **North ($\psi = 0$):** $\vec{a} = (0, 0, +a)$, displacement $\Delta z > 0$ (Northward).
  2. **East ($\psi = \pi/2$):** $\vec{a} = (+a, 0, 0)$, displacement $\Delta x > 0$ (Eastward).
  3. **South ($\psi = \pi$):** $\vec{a} = (0, 0, -a)$, displacement $\Delta z < 0$ (Southward).
  4. **West ($\psi = -\pi/2$):** $\vec{a} = (-a, 0, 0)$, displacement $\Delta x < 0$ (Westward).

### 2.2 Service Braking & Reverse Motion ([S] / Down Arrow)
* **State 1: Moving Forward ($v_{\text{fwd}} > 0.35\text{ m/s}$):**
  - **Authoritative Input:** `road_inputs.brake = 0.95, road_inputs.throttle = 0.0, road_inputs.gear_mode = TransmissionMode::Drive`.
  - **Braking Force:** $\vec{F}_{\text{brake}} = -F_{\text{brake}} \cdot \frac{\vec{v}}{\|\vec{v}\|}$, opposing current motion.
  - **Result:** Progressive monotonic deceleration until $v_{\text{fwd}} \to 0$.
* **State 2: Standstill or Reversing ($v_{\text{fwd}} \le 0.35\text{ m/s}$):**
  - **Authoritative Input:** `road_inputs.throttle = 0.85, road_inputs.brake = 0.0, road_inputs.gear_mode = TransmissionMode::Reverse`.
  - **Longitudinal Force:** $\vec{F}_{\text{reverse}} = -F_{\text{rev\_engine}} \cdot \vec{f}_{\text{world}}$.
  - **Proof Across Cardinal Headings:**
    1. **North ($\psi = 0$):** Displacement $\Delta z < 0$ (Southward).
    2. **East ($\psi = \pi/2$):** Displacement $\Delta x < 0$ (Westward).
    3. **South ($\psi = \pi$):** Displacement $\Delta z > 0$ (Northward).
    4. **West ($\psi = -\pi/2$):** Displacement $\Delta x > 0$ (Eastward).

### 2.3 Steering Left ([A] / Left Arrow)
* **Authoritative Input:** `road_inputs.steer < 0` (normalized to $[-1.0, 0)$).
* **Front Wheel Angle:** $\delta = \text{steer} \cdot \delta_{\max} < 0$. Front wheels rotate toward $-X$ (Left).
* **Kinematic Yaw Rate:** $\dot{\psi} = \frac{v_{\text{fwd}}}{L} \tan \delta$. When $v_{\text{fwd}} > 0$ and $\delta < 0$, $\dot{\psi} < 0$.
* **Tire Lateral Force:** Slip angle $\alpha_{\text{front}} > 0$, generating lateral force $\vec{F}_{\text{lat}} = -C_\alpha \alpha < 0$ along $\vec{r}$ (pushing front of car toward $-X$).
* **Torque Around Up Axis:** $\vec{\tau} = \vec{r}_{\text{wheel}} \times \vec{F}_{\text{wheel}} = (0, 0, L_{\text{front}}) \times (-F_x, 0, 0) = (0, -L F_x, 0)$. Negative torque around $+Y$.
* **Proof Across Cardinal Headings:**
  1. **Facing North ($\psi = 0$):** Negative yaw rotates forward vector from $(0, 0, +1)$ toward $(-1, 0, 0)$ (West/Left).
  2. **Facing East ($\psi = \pi/2$):** Negative yaw rotates forward vector from $(+1, 0, 0)$ toward $(0, 0, +1)$ (North/Left).
  3. **Facing South ($\psi = \pi$):** Negative yaw rotates forward vector from $(0, 0, -1)$ toward $(+1, 0, 0)$ (East/Left).
  4. **Facing West ($\psi = -\pi/2$):** Negative yaw rotates forward vector from $(-1, 0, 0)$ toward $(0, 0, -1)$ (South/Left).

### 2.4 Steering Right ([D] / Right Arrow)
* **Authoritative Input:** `road_inputs.steer > 0` (normalized to $(0, +1.0]$).
* **Front Wheel Angle:** $\delta = \text{steer} \cdot \delta_{\max} > 0$. Front wheels rotate toward $+X$ (Right).
* **Kinematic Yaw Rate:** $\dot{\psi} = \frac{v_{\text{fwd}}}{L} \tan \delta > 0$. Positive yaw rate around $+Y$.
* **Torque Around Up Axis:** $\vec{\tau} = (0, 0, L_{\text{front}}) \times (+F_x, 0, 0) = (0, +L F_x, 0)$. Positive torque around $+Y$.
* **Proof Across Cardinal Headings:**
  1. **Facing North ($\psi = 0$):** Positive yaw rotates forward vector from $(0, 0, +1)$ toward $(+1, 0, 0)$ (East/Right).
  2. **Facing East ($\psi = \pi/2$):** Positive yaw rotates forward vector from $(+1, 0, 0)$ toward $(0, 0, -1)$ (South/Right).
  3. **Facing South ($\psi = \pi$):** Positive yaw rotates forward vector from $(0, 0, -1)$ toward $(-1, 0, 0)$ (West/Right).
  4. **Facing West ($\psi = -\pi/2$):** Positive yaw rotates forward vector from $(-1, 0, 0)$ toward $(0, 0, +1)$ (North/Right).

---

## 3. Input Lifecycle & Focus Invariants

To eliminate stuck keys, runaway acceleration, or ghost steering:

1. **Focus Loss (`WindowEvent::Focused(false)`):** Invokes `reset_all_inputs()`, clearing `pressed_keys`, steering accumulator, throttles, and mouse drags.
2. **Pause Menu (`KeyCode::Escape` / `PauseSimulation`):** Clears all inputs and forces `road_inputs.gear_mode = TransmissionMode::Park`.
3. **Modals & Overlays (Garage [G], Catalog [U], Settings [O], Jobs [J]):** Clears all input buffers prior to modal presentation.
4. **Vehicle Switch (`execute_spawn_vehicle`):**
   - Calls `reset_all_inputs()`.
   - Initializes target vehicle at valid road candidate with zero linear and angular velocities.
   - Synchronizes `previous_vehicle_pos` and `previous_vehicle_orient` to active entity to eliminate rendering interpolation stretch.
   - Resets camera perspective tracking.
5. **Region Transition (`load_region`):** Full reset of inputs, cache anchors, and physics accumulators before and after world compilation.

---

## 4. Vehicle Dynamics & Static Equilibria Architecture

* **Authoritative Fixed Timestep:** Physics executes at fixed $dt = \frac{1}{60}\text{ s}$ ($60\text{ Hz}$) with a bounded accumulator ($\le 0.20\text{ s}$) to prevent spiral-of-death under render load spikes.
* **Transmission Park Mode (`TransmissionMode::Park`):**
  - Mechanical parking pawl unconditionally zeroes linear and angular velocities: $\vec{v} = \vec{0}, \vec{\omega} = \vec{0}$.
  - Holds vehicle stationary on all supported slopes (up to 30%+ grade).
* **Automatic Hill-Hold Assist (HHA):**
  - When in Drive (`TransmissionMode::Drive`) with zero throttle (`throttle == 0.0`), grounded wheels $\ge 2$, and slope grade $\le 0.25$ (25% grade), HHA holds the vehicle motionless without backward drift.
* **Torque Converter Idle Forward Creep:**
  - Idling in Drive on level/mild slopes produces gentle forward creep force ($8\%$ peak engine force), scaling down as speed approaches $2.5\text{ m/s}$.
* **Active Caster Self-Centering Damping:**
  - When steering input is released (`inputs.steer.abs() < 0.01`), active damping on yaw rate ($\omega_y \leftarrow \omega_y \times 0.88$) prevents lateral positive-feedback divergence down slopes.

---

## 5. Building Collision Outward Separation Hyperplane

* **Normal Direction Rule:** In [`engine/vehicles/src/building_collision.rs`](file:///j:/rustRealEarth/engine/vehicles/src/building_collision.rs), whenever vehicle collision point $\vec{p}$ is classified *inside* the footprint polygon via point-in-polygon ray casting:
  $$\vec{n}_{\text{collision}} = -\frac{\vec{p} - \vec{p}_{\text{closest}}}{\|\vec{p} - \vec{p}_{\text{closest}}\|}$$
* **Separation Displacement:** Resolution adds $\vec{n}_{\text{collision}} \cdot d_{\text{pen}}$ to vehicle position, pointing strictly outward toward the street exterior. Inward penetration trapping is mathematically impossible.
