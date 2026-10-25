# ADR-0048: Stage 1.5-R — Windows Mouse and Complete UI Interaction Repair

## Status
**Accepted** (2026-10-25)

## Context
During Windows application evaluation of [`apps/world-viewer/src/interactive.rs`](file:///j:/rustRealEarth/apps/world-viewer/src/interactive.rs) and the heads-up display subsystem in [`engine/rendering/src/hud.rs`](file:///j:/rustRealEarth/engine/rendering/src/hud.rs), critical input unresponsiveness, menu click failures, and camera dragging conflicts were identified:

1. **In-Game Left-Click Hijacking & Camera Rotation Conflict**:
   - In [`interactive.rs`](file:///j:/rustRealEarth/apps/world-viewer/src/interactive.rs), `AppUiState::InGame` mapped both `MouseButton::Left` and `MouseButton::Right` indiscriminately to `self.mouse_drag_active = true`.
   - Consequently, clicking on any in-game interactive HUD elements, contextual action prompts, or dashboard chips unintentionally induced 3D camera rotation and viewport spin rather than dispatching domain actions.

2. **Missing Mouse Handlers for Modals**:
   - `AppUiState::MultiplayerModal` and `AppUiState::RailRouteModal` were absent from `WindowEvent::MouseInput` and `WindowEvent::MouseWheel` matches, falling through to `_ => {}`. Clicking lobby actions or rail routes was completely ignored by mouse input.
   - `MultiplayerLayout` and `RailRouteLayout` did not exist in [`engine/rendering/src/hud.rs`](file:///j:/rustRealEarth/engine/rendering/src/hud.rs), preventing hit-testing of modal buttons.

3. **Garage Selection Preview Bug**:
   - In `AppUiState::GarageSelect`, clicking a vehicle in the list updated `self.garage_selected_idx = idx;` but failed to trigger vehicle preview loading or chassis preparation (`apply_garage_selection(idx)`). The 3D viewport behind the modal never updated until the player clicked Spawn.

4. **Input Loss on Window Focus Loss & Occlusion**:
   - Window occlusion, cursor departure from the window client area, and focus loss left stale dragging flags active, resulting in uncontrolled rotation upon cursor re-entry.

5. **Lack of Mouse Sensitivity Setting**:
   - The settings menu offered only 5 options without configurable mouse look sensitivity, leaving users on high-DPI displays or TrackPoint devices without cursor speed calibration.

## Decision

1. **Central Pointer Router & Priority Ordering**:
   - Implemented a deterministic 3-tier input priority router in [`interactive.rs`](file:///j:/rustRealEarth/apps/world-viewer/src/interactive.rs):
     - **Priority 1 (Modal UI Overlays)**: If any modal is active (`StartScreen`, `PauseMenu`, `GarageSelect`, `JobBrowser`, `SettingsMenu`, `CatalogModal`, `DebriefModal`, `MultiplayerModal`, `RailRouteModal`), all left clicks, releases, and wheel scrolls are exclusively consumed by the modal. Zero camera rotation or vehicle control is permitted.
     - **Priority 2 (Interactive Non-Modal HUD & Dock)**: When `InGame`, left-clicks hit-test the `InGameHudLayout` (Pause button, Camera perspective button, Park brake toggle, Dock chips: Garage, Jobs, Cities, Multiplayer, Routes, Debug overlay, and Contextual prompts). If hit, the corresponding `UiDomainAction` is dispatched and camera drag is inhibited.
     - **Priority 3 (World Picking & Camera Orbit)**: 100% camera isolation is established: Left-click NEVER rotates the camera. Camera orbit and free-look is exclusively driven by **Right-Click Drag** (`MouseButton::Right`), scaled by configurable mouse sensitivity.
     - **Touch Event Parity**: `WindowEvent::Touch` phases (`Started`, `Moved`, `Ended`, `Cancelled`) route through the identical hit-test and domain action pipeline.

2. **Dedicated Interactive Layouts in HUD Subsystem**:
   - In [`engine/rendering/src/hud.rs`](file:///j:/rustRealEarth/engine/rendering/src/hud.rs), implemented and exported:
     - `MultiplayerLayout` and `MultiplayerHit`: Computes responsive geometry for 4 action buttons (`HostLocal`, `JoinDedicated`, `Disconnect`, `ResumeSimulation`) and Close button.
     - `RailRouteLayout` and `RailRouteHit`: Computes responsive geometry for surveyed route cards, `DispatchRoute` button, and Close button.
     - `InGameHudLayout` and `InGameHudHit`: Computes responsive top bar (`Pause`, `CameraCycle`), bottom dock (`Garage`, `Jobs`, `Cities`, `Multiplayer`, `Routes`, `Park`, `Debug`), and contextual prompts.

3. **Instant Garage Preview & Double-Click Activation**:
   - In `AppUiState::GarageSelect`, single-clicking any vehicle immediately calls `self.apply_garage_selection(idx)`, updating the 3D vehicle model in the background viewport.
   - Implemented double-click detection ($\le 350\text{ms}$, $\le 10\text{px}$ radius):
     - Double-clicking in Garage immediately selects, spawns, and returns to driving.
     - Double-clicking in City Catalog immediately warps and loads the target city.
     - Double-clicking in Job Browser immediately accepts the contract.
     - Double-clicking in Rail Route Selector immediately dispatches the train consist.

4. **Window Focus Loss & Occlusion Input Purging**:
   - `WindowEvent::Focused(false)`, `WindowEvent::Occluded(true)`, window minimization, and `WindowEvent::CursorLeft` trigger `self.reset_all_inputs()`, purging keys, mouse drag flags, cursor tracking, and smoothed inputs.

5. **Configurable Mouse Look Sensitivity**:
   - Added a 6th preference to the Settings Menu: `MOUSE LOOK SENSITIVITY` cycling through `1.0x (NORMAL)`, `1.5x (FAST)`, `2.0x (ULTRA)`, and `0.5x (PRECISE)`.

## Verification

The fixes were validated via a 7-part master certification test suite in [`tests/tests/stage1_5_mouse_and_ui_interaction_certification.rs`](file:///j:/rustRealEarth/tests/tests/stage1_5_mouse_and_ui_interaction_certification.rs):
- **Part 1**: Deterministic hit-testing at center, corners, and boundaries across all 9 UI layouts and in-game HUD across 720p, 900p, 1080p, 1440p, and 4K UHD.
- **Part 2**: DPI scale factor invariance ($1.0\times$ to $2.0\times$) ensuring physical-to-logical coordinate parity.
- **Part 3**: Complete camera isolation: left drag does not rotate camera; right drag rotates camera; modal windows completely suppress camera dragging.
- **Part 4**: Garage vehicle selection -> live 3D preview -> spawn lifecycle and double-click instant spawn.
- **Part 5**: Geographic city catalog selection, double-click warp, and contract acceptance.
- **Part 6**: Input purging on focus loss and window occlusion.
- **Part 7**: In-game HUD dock chips and pause button click transitions.
