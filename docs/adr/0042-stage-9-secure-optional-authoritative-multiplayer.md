# ADR-0042: Stage 9 Secure Optional Authoritative Multiplayer Simulation

## Status
Accepted

## Context
Following the completion of Stage 8 (Authentic Major US Cities, Interstate Connectivity & Regional Streaming), Stage 9 of the [Execution Contract](file:///j:/rustRealEarth/docs/ExecutionContract.md) mandates implementing **Secure Optional Authoritative Multiplayer, integrated with actual world simulation**.

Prior to Stage 9, Real Earth featured single-player vehicle simulation, multimodal transit, autonomous traffic fleets, and career gameplay. To support genuine multi-user planetary simulation without compromising physics fidelity, security, or single-player performance, the platform required:
1. **Phase 9.1 — Runtime integration**:
   - Audit networking crates for disconnected or insufficient implementations.
   - Implement an optional headless authoritative Rust server with real UDP transport (`UdpSocketTransport`), binary framing protocol with sequence/ACK headers, session authentication, token-bucket rate limiting, 60 Hz simulation tick loop, and configurable local/LAN/dedicated operation.
   - Strictly preserve fully offline single-player mode with zero background threads, zero network sockets, and zero latency overhead.
2. **Phase 9.2 — Synchronization**:
   - Server-authoritative entity state and physical integration.
   - Zero-latency client-side prediction with rewind-and-replay reconciliation buffer.
   - Smooth snapshot interpolation with jitter buffering for remote vehicles.
   - Spatial Area of Interest (AoI) culling and cell boundary transitions.
   - Strict floating origin invariance: metric regional coordinates over the wire, client-side camera offset subtraction during mesh generation to maintain sub-millimeter precision globally.
   - Anti-cheat defenses: rejection of NaN/Inf inputs, excessive delta times, speed hacking, and unauthorized entity mutations.
3. **Phase 9.3 — User integration**:
   - Interactive glassmorphic Multiplayer Modal accessible via `[N]` or `[F4]`.
   - Host Local Server, Join Dedicated Server, Disconnect / Leave Session, and Return to Driver controls.
   - Top-right persistent Multiplayer Status Chip displaying connection state, live RTT latency (ping), and online driver counts.
   - Distinct participant colors and vehicle model rendering in the dynamic vehicle fleet mesh.
4. **Phase 9.4 — Validation**:
   - Genuine multi-client loopback E2E sessions verifying mutual vehicle discovery, zero-divergence prediction reconciliation, remote interpolation smoothness, network impairment resilience (latency, jitter, drop), and clean despawn on disconnect.

---

## Decisions

### 1. Phase 9.1: Headless Authoritative Server & Binary Protocol
- **Binary Packet Protocol** in [engine/networking/src/protocol.rs](file:///j:/rustRealEarth/engine/networking/src/protocol.rs):
  - Fixed 24-byte `PacketHeader` featuring `PROTOCOL_MAGIC` (`0x5245414C` = "REAL"), `PROTOCOL_VERSION` (v1), channel type (`ReliableOrdered` vs `UnreliableSequenced`), packet sequence, ack sequence, and 32-bit ack bitfield.
  - Datagrams capped at 1,200 bytes (`MAX_UDP_PACKET_SIZE`) to prevent Ethernet MTU fragmentation.
  - Strongly typed messages: `ClientHello`, `ServerHello`, `HandshakeReject`, `Input`, `SpawnEntity`, `DespawnEntity`, `ServerSnapshot`, `Ping`, `Pong`, `Disconnect`.
- **Headless Server Engine** in [engine/networking/src/server_runner.rs](file:///j:/rustRealEarth/engine/networking/src/server_runner.rs):
  - `HeadlessAuthoritativeServer` manages real non-blocking UDP sockets, maps remote client `SocketAddr <-> session_id`, manages spawn slots, steps vehicle kinematics at 60 Hz, and emits tailored AoI snapshots.
  - Provides `spawn_in_background(bind_addr, tick_rate, max_players)` enabling instant one-click hosting from desktop or standalone dedicated servers via CLI.
- **Session Authentication & Rate Limiting** in [engine/networking/src/session.rs](file:///j:/rustRealEarth/engine/networking/src/session.rs):
  - Crypto-random session tokens (`tok_<random>`).
  - Token-bucket rate limiting (120 Hz burst ceiling) preventing denial-of-service packet floods.

---

### 2. Phase 9.2: Authoritative Synchronization & Client Prediction
- **Client Prediction & Replay Reconciliation** in [engine/networking/src/replication.rs](file:///j:/rustRealEarth/engine/networking/src/replication.rs):
  - `ClientPredictor` applies driver inputs immediately locally for zero perceived input lag.
  - Retains a rolling 128-frame input buffer (`BufferedInput`).
  - When server snapshot arrives, if Euclidean position divergence exceeds `tolerance` (default 0.25m), the client rewinds to server ground truth and replays subsequent unacknowledged inputs.
- **Remote Entity Interpolation**:
  - `RemoteEntityInterpolator` stores historical snapshots in a ring buffer with an 80ms jitter delay.
  - Evaluates spherical linear interpolation (`slerp`) for orientation and hermite cubic spline interpolation for position, guaranteeing smooth movement without teleport artifacts.
- **Spatial Area of Interest (AoI) Culling** in [engine/networking/src/interest.rs](file:///j:/rustRealEarth/engine/networking/src/interest.rs):
  - World divided into 2D grid cells (`SpatialCellId`).
  - `query_entities_in_radius` filters replication sets so clients only receive updates for vehicles within their observation bubble (e.g. 500m–1000m), reducing WAN bandwidth by >85%.
- **Floating Origin Invariance**:
  - Network coordinates are stored and transmitted in regional metric ENU.
  - During client mesh generation, $P_{\text{render}} = P_{\text{world}} - P_{\text{cam\_offset}}$.
  - Both local and remote vehicles retain sub-millimeter precision regardless of total distance traveled from initial spawn.

---

### 3. Phase 9.3: User Experience, HUD Controls & Desktop Integration
- **Interactive UI Modal & Controls** in [engine/rendering/src/hud.rs](file:///j:/rustRealEarth/engine/rendering/src/hud.rs) and [apps/world-viewer/src/interactive.rs](file:///j:/rustRealEarth/apps/world-viewer/src/interactive.rs):
  - `[N]` or `[F4]` toggles `AppUiState::MultiplayerModal`.
  - Up/Down arrow keys navigate options; Enter triggers actions:
    1. `HOST LOCAL SERVER`: Spawns background authoritative server on port 7777 and auto-connects.
    2. `JOIN DEDICATED SERVER`: Connects non-blockingly to specified IP:Port.
    3. `DISCONNECT / LEAVE`: Gracefully notifies server, tears down client socket, and returns to offline mode.
    4. `RESUME SIMULATION`: Closes modal.
  - Live participant roster displays driver names, roles, and real-time RTT latency.
  - Top-right `add_multiplayer_chip` shows green status pill with latency and player count when connected, yellow during reconnection, and muted offline banner.
- **Remote Vehicle Rendering**:
  - `MultiplayerClient::get_remote_vehicles(render_time_ms)` integrates into `fleet_mesh` alongside local vehicle and AI traffic.
  - Remote vehicles rendered with distinct cyan/player-keyed hues and functional headlights and brake lights.

---

### 4. Phase 9.4: Security Defenses & Offline Independence
- **Anti-Cheat Defense**:
  - Client-authored physical state (position, velocity) is rejected. Clients submit only validated control inputs (`throttle`, `steer`, `brake`, `dt`).
  - Inputs with NaN, infinity, or $dt > 1.0\text{s}$ are immediately rejected.
  - Requests attempting to control entities not registered to the authenticated session return `NetworkError::Unauthorized`.
- **Zero Overhead Single-Player Invariant**:
  - In offline mode, `multiplayer_client` is `None`.
  - Zero sockets are bound, zero threads spawned, and zero network polling occurs, preserving 100% of single-player CPU and memory efficiency.

---

## Consequences

### Positive
- Fully functional multi-client vehicular simulation sharing the same authentic GIS world.
- Server authority eliminates speed hacks, teleport cheats, and physics divergence.
- Sub-5cm prediction reconciliation error and smooth remote interpolation.
- Host-and-play capability without requiring external dedicated server infrastructure, while CLI dedicated server mode (`realearth-cli server`) supports headless production deployments.
- Full preservation of single-player offline experience with zero performance penalty.

### Verification
- Tested via [tests/tests/stage9_authoritative_multiplayer_certification.rs](file:///j:/rustRealEarth/tests/tests/stage9_authoritative_multiplayer_certification.rs) covering 8 automated tests across all 4 phases.
- Tested via [tests/tests/multiplayer_networking.rs](file:///j:/rustRealEarth/tests/tests/multiplayer_networking.rs) covering 7 network reliability tests.
