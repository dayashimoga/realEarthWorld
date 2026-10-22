# ADR 0014: Optional Authoritative Multiplayer

## Status
Accepted

## Context
Stages 1 through 9 delivered verified GIS ingestion, validation, streaming, multi-region geography, career gameplay, and multimodal vehicle simulation across road, rail, marine, and aviation modalities.

Stage 10 introduces **STAGE 10: Optional Authoritative Multiplayer** in accordance with `docs/ExecutionContract.md`:
> "Implement STAGE 10: Optional Authoritative Multiplayer only after standalone stability is certified."
> "Offline single-player remains fully functional and independent."
> "Real multiple-client sessions sharing the same geographically accurate world and correctly replicated vehicles without unauthorized state manipulation."

Planetary-scale multiplayer simulation requires handling continuous geographic coordinates (WGS84 / local ENU floating origins), diverse vehicle modalities (sedans, trucks, trains, ships, aircraft), and untrusted remote clients across public internet topologies.

## Decision

### 1. Dedicated `realearth-networking` Crate Decoupled from Engine Core
We introduce an isolated `engine/networking` crate (`realearth-networking`) implementing:
- Packet framing, serialization, and protocol negotiation.
- Authoritative server state machines and client network proxies.
- Spatial interest management (AoI) and distributed region-cell partitioning.
- Client prediction, reconciliation, and snapshot interpolation.
- Transport simulation (latency, jitter, packet loss injection) for automated CI certification.

Single-player offline world-viewer operates with 100% zero overhead; networking is strictly optional and feature-gated.

### 2. Hybrid Reliable / Unreliable Protocol Framing
Networking utilizes a framed packet protocol over UDP/QUIC:
- **Packet Header**: Magic bytes (`0x5245414C`), schema version (`u16`), packet sequence (`u64`), ack sequence (`u64`), ack bitfield (`u32`), and channel ID.
- **Reliable Ordered Channel**: Handshakes, session authentication, entity lifecycle (spawn/despawn), chat, career contract assignments, and cell handoff. Uses selective ACK retransmission.
- **Unreliable Sequenced Channel**: High-frequency vehicle controls (throttle, steer, brake) and authoritative snapshot replication. Discards out-of-order packets automatically to minimize latency.

### 3. Server-Authoritative Physics with Client Prediction & Reconciliation
- The server is the sole authority for physical integration, collision detection, and geographical validation.
- Clients send input packets (`ClientInputFrame`: input vectors, client tick, timestamp).
- The local client simulates immediate response (zero perceived input lag) while buffering unacknowledged inputs.
- When the server broadcasts authoritative state snapshots (`ServerEntitySnapshot`), the client verifies prediction error:
  - If error $|x_{\text{client}} - x_{\text{server}}| < \epsilon_{\text{threshold}}$, prediction continues smoothly.
  - If error exceeds threshold (due to server collision, external forces, or divergent state), client snaps to authoritative server state and re-applies buffered inputs up to current tick.
- Remote entities are rendered using smooth Hermite / linear interpolation over a configurable snapshot jitter buffer ($50\text{ ms} - 100\text{ ms}$).

### 4. Spatial Interest Management (Area of Interest - AoI)
- A global world cannot broadcast all planetary entities to every connected client.
- The authoritative server partitions the world into geographic interest cells.
- Each client subscribes to an interest radius:
  - Ground vehicles (cars, buses, trucks, trains): $500\text{m} - 1000\text{m}$.
  - Marine vessels: $2000\text{m}$.
  - Aircraft: $10,000\text{m} - 30,000\text{m}$ (altitude-scaled).
- Entities outside a client's interest cone are culled from replication, reducing network bandwidth by $>95\%$.

### 5. Security & Anti-Cheat Validation
- **Authentication**: Cryptographically verified session tokens with role-based access control (`Player`, `Spectator`, `Admin`).
- **Input Sanitization**: Rejection of NaN, infinite, or out-of-range floats ($[-1.0, 1.0]$ bounds).
- **Speed-Hack & Teleportation Defense**: Server enforces maximum acceleration ($a \le a_{\text{max}}$) and terminal speed limits per vehicle category. Impossible spatial displacements trigger immediate server snap-back.
- **Entity Authorization**: Clients can only submit inputs for entities they legitimately own; unauthorized mutation packets are logged and discarded.

## Consequences
- **Positive**: Complete multi-client multiplayer capability sharing real planetary coordinates.
- **Positive**: Strict client prediction and reconciliation ensures zero perceived input latency for drivers/pilots.
- **Positive**: Single-player offline mode remains completely pristine and untouched.
- **Negative**: Adds state serialization and network snapshot handling to test suites.
