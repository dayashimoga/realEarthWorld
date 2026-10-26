# STAGE H0 — GEOGRAPHIC COVERAGE AUDIT
**Project:** Real Earth World Platform  
**Audit Document:** GeographicCoverageAudit.md  
**Audit Stage:** Stage H0 (Forensic Runtime Audit & Quality Gates)  
**Date:** October 26, 2026  
**Auditing Authority:** Principal GIS, Graphics & Simulation Engineering  
**Baseline Git Commit:** `7fafd8c9df1fea96513bbcd74e15b8336fda2eba`  
**Overall Finding:** **33 OF 38 CITIES ARE UNVERIFIED / MISSING GIS DATA PACKAGES (AUDIT FAILED)**

---

## 1. Executive Forensic Summary

Earlier certification documents (such as `docs/reports/RealGISCityCoverage.md`) falsely claimed that all 38 United States metropolitan regions achieved "100% PASS ✅" with thousands of surveyed nodes and interconnected highway corridors.

Forensic code auditing and live execution of `realearth-viewer.exe` demonstrates that:
1. **Only 5 Cities Have Bundled Data:** `san-francisco`, `portland`, `new-york`, `chicago`, and `los-angeles` have sample vector/DEM directories in `assets/`. However, each dataset is restricted to a small 2.5km × 2.5km bounding box (e.g. San Francisco `chunks.db` contains strictly 30 chunks). Beyond 1.5km from the center landmark, terrain and roads abruptly terminate.
2. **33 Cities Completely Lack GIS Data Packages:** In `engine/ingestion/src/authentic.rs:260-272`, loading any of the remaining 33 cities (e.g. `orlando`, `atlanta`, `detroit`, `dallas`, `miami`, `seattle`) unconditionally returns:
   ```
   RealEarthError::Ingestion("Region '{name}' ({state}) requires authentic GIS data package (Surveyed GIS source data not bundled or verified)")
   ```
3. **The In-Game UI Misleads the User:** As photographed in **Screenshot 3**, the in-game catalog presents "38 AUDITED METROPOLITAN REGIONS" with landmarks and elevation ranges. Clicking "INSTALL & WARP" on Orlando produces an immediate runtime ingestion error toast.

In accordance with Stage H0 Governance, **all unsupported claims are formally marked UNVERIFIED, not PASS.**

---

## 2. 38 Metropolitan Regions Forensic Status Table

| # | City ID | Metropolitan Area | State | Catalog Tier | Bundled Assets on Disk | Ingestion Status (`load_authentic_city`) | Prior Certified Claim | Forensic Audit Status |
| :- | :--- | :--- | :- | :- | :--- | :--- | :--- | :--- |
| 1 | `san-francisco` | San Francisco Bay Area | CA | P1-National | `assets/san_francisco_sample` (30 chunks, 2.7km bbox) | OK (Sample Bounded) | PASS ✅ | **PARTIALLY VERIFIED (BOUNDED 2.5KM)** |
| 2 | `portland` | Portland Metro | OR | P2-Major | `assets/portland_sample` (24 chunks) | OK (Sample Bounded) | PASS ✅ | **PARTIALLY VERIFIED (BOUNDED 2.5KM)** |
| 3 | `new-york` | New York City / Newark | NY | P1-National | `assets/new_york_sample` (38 chunks) | OK (Sample Bounded) | PASS ✅ | **PARTIALLY VERIFIED (BOUNDED 2.5KM)** |
| 4 | `chicago` | Chicago Metro | IL | P1-National | `assets/chicago_sample` (28 chunks) | OK (Sample Bounded) | PASS ✅ | **PARTIALLY VERIFIED (BOUNDED 2.5KM)** |
| 5 | `los-angeles` | Greater Los Angeles | CA | P1-National | `assets/los_angeles_sample` (32 chunks) | OK (Sample Bounded) | PASS ✅ | **PARTIALLY VERIFIED (BOUNDED 2.5KM)** |
| 6 | `orlando` | Greater Orlando | FL | P2-Major | **NONE** | **FAILED (Ingestion Error)** | PASS ✅ | **UNVERIFIED / BLOCKED_NO_DATA ❌** |
| 7 | `atlanta` | Metro Atlanta | GA | P2-Major | **NONE** | **FAILED (Ingestion Error)** | PASS ✅ | **UNVERIFIED / BLOCKED_NO_DATA ❌** |
| 8 | `detroit` | Metro Detroit | MI | P2-Major | **NONE** | **FAILED (Ingestion Error)** | PASS ✅ | **UNVERIFIED / BLOCKED_NO_DATA ❌** |
| 9 | `minneapolis` | Minneapolis–Saint Paul | MN | P2-Major | **NONE** | **FAILED (Ingestion Error)** | PASS ✅ | **UNVERIFIED / BLOCKED_NO_DATA ❌** |
| 10 | `austin` | Greater Austin | TX | P2-Major | **NONE** | **FAILED (Ingestion Error)** | PASS ✅ | **UNVERIFIED / BLOCKED_NO_DATA ❌** |
| 11 | `san-antonio` | Greater San Antonio | TX | P2-Major | **NONE** | **FAILED (Ingestion Error)** | PASS ✅ | **UNVERIFIED / BLOCKED_NO_DATA ❌** |
| 12 | `las-vegas` | Las Vegas Valley | NV | P2-Major | **NONE** | **FAILED (Ingestion Error)** | PASS ✅ | **UNVERIFIED / BLOCKED_NO_DATA ❌** |
| 13 | `sacramento` | Greater Sacramento | CA | P2-Major | **NONE** | **FAILED (Ingestion Error)** | PASS ✅ | **UNVERIFIED / BLOCKED_NO_DATA ❌** |
| 14 | `charlotte` | Charlotte Metro | NC | P2-Major | **NONE** | **FAILED (Ingestion Error)** | PASS ✅ | **UNVERIFIED / BLOCKED_NO_DATA ❌** |
| 15 | `houston` | Greater Houston | TX | P1-National | **NONE** | **FAILED (Ingestion Error)** | PASS ✅ | **UNVERIFIED / BLOCKED_NO_DATA ❌** |
| 16 | `dallas` | Dallas–Fort Worth | TX | P1-National | **NONE** | **FAILED (Ingestion Error)** | PASS ✅ | **UNVERIFIED / BLOCKED_NO_DATA ❌** |
| 17 | `washington-dc` | Washington Metro | DC | P1-National | **NONE** | **FAILED (Ingestion Error)** | PASS ✅ | **UNVERIFIED / BLOCKED_NO_DATA ❌** |
| 18 | `miami` | Miami Metro | FL | P1-National | **NONE** | **FAILED (Ingestion Error)** | PASS ✅ | **UNVERIFIED / BLOCKED_NO_DATA ❌** |
| 19 | `seattle` | Seattle Metro | WA | P1-National | **NONE** | **FAILED (Ingestion Error)** | PASS ✅ | **UNVERIFIED / BLOCKED_NO_DATA ❌** |
| 20 | `boston` | Greater Boston | MA | P1-National | **NONE** | **FAILED (Ingestion Error)** | PASS ✅ | **UNVERIFIED / BLOCKED_NO_DATA ❌** |
| 21 | `denver` | Denver Metro | CO | P2-Major | **NONE** | **FAILED (Ingestion Error)** | PASS ✅ | **UNVERIFIED / BLOCKED_NO_DATA ❌** |
| 22 | `philadelphia` | Delaware Valley | PA | P2-Major | **NONE** | **FAILED (Ingestion Error)** | PASS ✅ | **UNVERIFIED / BLOCKED_NO_DATA ❌** |
| 23 | `phoenix` | Phoenix Metro | AZ | P2-Major | **NONE** | **FAILED (Ingestion Error)** | PASS ✅ | **UNVERIFIED / BLOCKED_NO_DATA ❌** |
| 24 | `san-diego` | San Diego County | CA | P2-Major | **NONE** | **FAILED (Ingestion Error)** | PASS ✅ | **UNVERIFIED / BLOCKED_NO_DATA ❌** |
| 25 | `san-jose` | Silicon Valley | CA | P2-Major | **NONE** | **FAILED (Ingestion Error)** | PASS ✅ | **UNVERIFIED / BLOCKED_NO_DATA ❌** |
| 26 | `tampa` | Tampa Bay Area | FL | P2-Major | **NONE** | **FAILED (Ingestion Error)** | PASS ✅ | **UNVERIFIED / BLOCKED_NO_DATA ❌** |
| 27 | `salt-lake-city` | Salt Lake City Metro | UT | P2-Major | **NONE** | **FAILED (Ingestion Error)** | PASS ✅ | **UNVERIFIED / BLOCKED_NO_DATA ❌** |
| 28 | `nashville` | Nashville Metro | TN | P2-Major | **NONE** | **FAILED (Ingestion Error)** | PASS ✅ | **UNVERIFIED / BLOCKED_NO_DATA ❌** |
| 29 | `pittsburgh` | Pittsburgh Metro | PA | P2-Major | **NONE** | **FAILED (Ingestion Error)** | PASS ✅ | **UNVERIFIED / BLOCKED_NO_DATA ❌** |
| 30 | `baltimore` | Baltimore Metro | MD | P2-Major | **NONE** | **FAILED (Ingestion Error)** | PASS ✅ | **UNVERIFIED / BLOCKED_NO_DATA ❌** |
| 31 | `st-louis` | Greater St. Louis | MO | P2-Major | **NONE** | **FAILED (Ingestion Error)** | PASS ✅ | **UNVERIFIED / BLOCKED_NO_DATA ❌** |
| 32 | `columbus` | Columbus Metro | OH | P3-Regional | **NONE** | **FAILED (Ingestion Error)** | PASS ✅ | **UNVERIFIED / BLOCKED_NO_DATA ❌** |
| 33 | `indianapolis` | Indianapolis Metro | IN | P3-Regional | **NONE** | **FAILED (Ingestion Error)** | PASS ✅ | **UNVERIFIED / BLOCKED_NO_DATA ❌** |
| 34 | `kansas-city` | Kansas City Metro | MO | P3-Regional | **NONE** | **FAILED (Ingestion Error)** | PASS ✅ | **UNVERIFIED / BLOCKED_NO_DATA ❌** |
| 35 | `cleveland` | Greater Cleveland | OH | P3-Regional | **NONE** | **FAILED (Ingestion Error)** | PASS ✅ | **UNVERIFIED / BLOCKED_NO_DATA ❌** |
| 36 | `cincinnati` | Cincinnati Tri-State | OH | P3-Regional | **NONE** | **FAILED (Ingestion Error)** | PASS ✅ | **UNVERIFIED / BLOCKED_NO_DATA ❌** |
| 37 | `raleigh` | Research Triangle | NC | P3-Regional | **NONE** | **FAILED (Ingestion Error)** | PASS ✅ | **UNVERIFIED / BLOCKED_NO_DATA ❌** |
| 38 | `milwaukee` | Milwaukee Metro | WI | P3-Regional | **NONE** | **FAILED (Ingestion Error)** | PASS ✅ | **UNVERIFIED / BLOCKED_NO_DATA ❌** |

---

## 3. Geographic Boundary & World Disappearance Analysis

In the 5 partially verified cities, chunk databases are strictly limited to downtown micro-regions:
- **San Francisco Sample Bounding Box:**
  - Latitude: 37.780°N to 37.805°N (~2,775 meters North–South)
  - Longitude: -122.415°W to -122.388°W (~2,376 meters East–West)
- **Failure Mode:**
  - Driving along highways at high speeds (e.g. 390 km/h = 108.3 m/s) covers 2.5km in just 23 seconds.
  - Upon reaching the bounding box edge, `WorldStreamer::update` queries `chunks.db` for coordinates outside the sample grid.
  - Because no records exist, active chunks covering the vehicle drop to 0, `is_collision_ready` returns `false`, and the renderer frustum-culls all distant chunks.
  - As photographed in **Screenshot 1**, the vehicle is left suspended over an infinite flat void with zero terrain, buildings, or road geometry.

---

## 4. Quality Gate Acceptance Standard

Before any geographic coverage claim may be certified:
1. `test_h0_gate_catalog_city_transition_integrity` must execute `load_authentic_city` for all catalog regions without panic or ingestion errors.
2. `test_h0_gate_continuous_world_streaming_distance` must verify continuous chunk streaming and DEM collision readiness at 3.5km, 5.0km, and 10.0km travel distances.
3. The in-game catalog UI must clearly differentiate between offline bundled packages, online downloadable packages, and un-surveyed regions.
