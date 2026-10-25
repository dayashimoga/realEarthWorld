# Non-Destructive Recovery Contract — Stage A Baseline

**Project:** Real Earth World — Navigable Planetary Simulation  
**Repository:** [https://github.com/dayashimoga/realEarthWorld](https://github.com/dayashimoga/realEarthWorld)  
**Governance Phase:** STAGE A — FORENSIC ROOT-CAUSE RECOVERY AND NO-REGRESSION FOUNDATION  
**Authoritative Date:** 2026-10-25  

---

## 1. Purpose and Non-Destructive Mandate

This Recovery Contract establishes an immutable baseline for the Real Earth World simulation platform. To permanently eliminate recurring incomplete fixes, regression churn, and undocumented code modifications, all future development stages must adhere strictly to the terms of this contract.

> [!IMPORTANT]
> **Core Mandate:** Under no circumstances may any existing working feature, rendering subsystem, menu interface, physics solver, authentic GIS database, or platform build target be removed, corrupted, or replaced with untested stubs. Every future modification must be accompanied by reproducible test gates and proof of no regression against prior certified functionality.

---

## 2. Immutable Baseline Records

### 2.1 Git Repository State
* **Recovery Branch:** `recovery/stage_a_forensic_foundation`
* **Base HEAD Commit:** [`d85d8677e5d87171d3d63bca2d897f1f41489ecf`](file:///j:/rustRealEarth)
* **Commit Message:** `fix(docs): resolve broken intra-doc links in vehicle_mesh docs`
* **Tree Cleanliness:** Working tree verified clean prior to branch creation.

### 2.2 Toolchain & Compiler Environment
* **Rust Compiler (`rustc`):** `rustc 1.99.1 (58cf8f9c3 2026-10-13)`
* **Cargo Package Manager:** `cargo 1.99.1 (5f94df478 2026-08-27)`
* **Host Operating System:** Windows 10/11 x86_64
* **Target Triplet:** `x86_64-pc-windows-msvc`

### 2.3 Packaged Platform Artifacts & Hashes
| Artifact | Path | Size (Bytes) | SHA-256 Digest |
|---|---|---|---|
| **Windows Release Package** | [`dist/windows/realearth-windows-x64.zip`](file:///j:/rustRealEarth/dist/windows/realearth-windows-x64.zip) | 9,134,813 | `948180155CE871B76FCA222939D19E47D6C4FB156E2185F45D8535EABA90D744` |
| **San Francisco Authentic Package** | `dist/packages/san-francisco.rep` | 8,829,140 | `E18824838B40EF6E96E14B86052E9B5DF094BE06775796DF8EF8A53B3674681A` |
| **Release Certification Matrix** | [`dist/certification_matrix.json`](file:///j:/rustRealEarth/dist/certification_matrix.json) | 6,554 | `8BF605B8A1D34870ED93ED9A74E9D087790E9549FA5EB0E2D64670D8CFB9C899` |

### 2.4 Baseline Screenshot Inventory
* **Car Driving Viewport & HUD:** [`media_1792936134873.png`](file:///C:/Users/daya/.gemini/antigravity-ide/brain/187a9d25-511a-4079-ace0-19850887050f/.user_uploaded/media_1792936134873.png) (San Francisco Bay Area, Apex GT Sports Coupe, Gear D1, 2 km/h, road spawn angle offset).
* **Aviation High-Altitude Chase:** [`media_1792936143100.png`](file:///C:/Users/daya/.gemini/antigravity-ide/brain/187a9d25-511a-4079-ace0-19850887050f/.user_uploaded/media_1792936143100.png) (Airliner at 171 kts, 174m MSL, boundary escape trajectory).
* **Aviation Alignment:** [`media_1792936153559.png`](file:///C:/Users/daya/.gemini/antigravity-ide/brain/187a9d25-511a-4079-ace0-19850887050f/.user_uploaded/media_1792936153559.png) (Airliner cockpit telemetry & artificial horizon HUD).
* **Heavy Truck Chase Occlusion:** [`media_1792936170926.png`](file:///C:/Users/daya/.gemini/antigravity-ide/brain/187a9d25-511a-4079-ace0-19850887050f/.user_uploaded/media_1792936170926.png) (Articulated 15m trailer wall occluding 90% of screen at 18m camera chase distance).

---

## 3. Mandatory Governance Rules for Future Stages

1. **Strictly Preserve Prior Functionality:**
   Any commit that causes a previously passing test in `tests/tests/` to fail is rejected immediately.
2. **Deterministic Test Verification:**
   Every reported defect must have an independent, automated test reproducing the failure prior to remediation, and verifying the fix afterward. Passing unit tests alone do not constitute proof of visible correctness; graphical, telemetry, or packaged artifact tests are mandatory.
3. **Traceability of Distributed Builds:**
   All release archives in `dist/` must be generated through reproducible automation from verified Git commits and authentic GIS source packages. No unversioned binaries may be distributed.
4. **Honest Reporting:**
   Engineers must report `PASS`, `FAIL`, or `BLOCKED` with mathematical precision and telemetry logs. Mock tests, stub assertions, and screenshot-only claims are explicitly prohibited.
5. **Machine-Readable Defect Tracking:**
   All defects must be documented in `docs/RootCauseMatrix.json` and mirrored in `docs/RootCauseMatrix.md`.
