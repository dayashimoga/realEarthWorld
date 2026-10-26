# Gameplay and UI End-to-End Certification

**Document:** `docs/GameplayAndUIE2E.md`  
**Version:** 0.6.0  
**Tested Binary SHA-256:** `60c87e8972bc14e0ae1e6148c78f1a71a31138468aa9f2c2c627433561af5ff6`  
**Host Platform:** Windows 11 x86_64  
**Primary Source Modules:** [`engine/gameplay/src/`](file:///j:/rustRealEarth/engine/gameplay/src), [`apps/world-viewer/src/interactive.rs`](file:///j:/rustRealEarth/apps/world-viewer/src/interactive.rs)  
**Certification Suites:** [`tests/tests/stage_g4_final_runtime_and_release_gates_certification.rs`](file:///j:/rustRealEarth/tests/tests/stage_g4_final_runtime_and_release_gates_certification.rs)  
**Status:** **100% VERIFIED PASS**

---

## 1. Executive Summary

A game engine is only as good as its interactive gameplay loop. The Real Earth World Platform features a fully integrated gameplay framework with multimodal vehicle career paths (Free Roam, Taxi Chauffeur, Municipal Transit Bus, Express Courier, Heavy Freight, Rail Engineer, Ferry Captain, Aviation Pilot), surveyed geographic destination waypoints, economy wallet balance accounting, multi-vehicle garage management, and schema-migrated save/load persistence.

In Stage G4, end-to-end gameplay integrity was verified across standalone extracted release packages, memory persistence cycles, and dynamic job catalogs.

---

## 2. Gameplay & UI E2E Verification Matrix

| Gameplay System | Source Implementation | Tested Binary SHA | Platform | Verification Test | Expected Behavior | Actual Empirical Result | Status |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Interactive Driving Lifecycle** | [`main.rs`](file:///j:/rustRealEarth/apps/world-viewer/src/main.rs), [`dynamics.rs`](file:///j:/rustRealEarth/engine/vehicles/src/dynamics.rs) | `60c87e89...` | Windows x64 | `test_g4_phase1_actual_playable_test_matrix_and_interactive_lifecycle` | Standalone packaged executable launches, drives forward, steers, brakes, and recovers | Extracted package executed cleanly in isolated tempdir; all controls verified | **VERIFIED** |
| **Multimodal Fleet Selection** | [`garage.rs`](file:///j:/rustRealEarth/engine/gameplay/src/garage.rs), [`vehicle_models.rs`](file:///j:/rustRealEarth/engine/rendering/src/vehicle_models.rs) | `60c87e89...` | Windows x64 | `test_g4_phase1_actual_playable_test_matrix_and_interactive_lifecycle` | Fleet selection spans sports car, bus, truck, motorcycle, train, boat, aircraft | All multimodal types generate valid 3D meshes and certified models | **VERIFIED** |
| **Career Job Catalog Generation** | [`destinations.rs`](file:///j:/rustRealEarth/engine/gameplay/src/destinations.rs) | `60c87e89...` | Windows x64 | `test_g4_phase6_authoritative_multiplayer_and_career_persistence` | Generates authentic jobs between surveyed waypoints with valid distances & payouts | San Francisco jobs catalog generated; route distances > 500m; payouts valid | **VERIFIED** |
| **Wallet & Economy Persistence** | [`economy.rs`](file:///j:/rustRealEarth/engine/gameplay/src/economy.rs), [`save.rs`](file:///j:/rustRealEarth/engine/gameplay/src/save.rs) | `60c87e89...` | Windows x64 | `test_g4_phase6_authoritative_multiplayer_and_career_persistence` | Player wallet balance, completed missions, and unlocked fleet persist cleanly | Wallet (₹15,000.00), reputation, and garage fleet roundtripped without loss | **VERIFIED** |
| **Save Schema V1 -> V2 Migration** | [`save.rs`](file:///j:/rustRealEarth/engine/gameplay/src/save.rs#L100) | `60c87e89...` | Windows x64 | `test_g4_phase1_actual_playable_test_matrix_and_interactive_lifecycle` | Legacy save files automatically migrate to active schema without data loss | Forward migration injects default game settings and updates schema to v2 | **VERIFIED** |
| **Roadside Vehicle Recovery** | [`recovery.rs`](file:///j:/rustRealEarth/engine/gameplay/src/recovery.rs), [`ground.rs`](file:///j:/rustRealEarth/engine/vehicles/src/ground.rs) | `60c87e89...` | Windows x64 | `test_g4_phase1_actual_playable_test_matrix_and_interactive_lifecycle` | Stranded, inverted, or fallen vehicle safely recovers onto road surface | Instant orientation reset to upright pose on nearest surveyed ground elevation | **VERIFIED** |

---

## 3. Career Paths & Transport Contracts

The platform implements 8 authentic transport career tracks:

1. **Free Roam & Scenic Exploration:** Freely explore surveyed world cities, discover architectural landmarks (Transamerica Pyramid, Space Needle, Willis Tower, One WTC), and earn distance milestones.
2. **Urban Taxi & Chauffeur:** Pick up passengers at surveyed addresses, maintain smooth driving without collisions for comfort rating tips (up to 35% tip on 5-star comfort).
3. **Municipal Transit Bus:** Operate 40ft low-floor transit buses along scheduled municipal routes, timing arrivals, boarding commuters at curbside shelters with pneumatic doors.
4. **Express Courier & Delivery:** Dispatch fragile parcels and express packages within tight delivery time windows across metropolitan street networks.
5. **Intercity Heavy Freight:** Haul 53ft semitrailers across regional highway corridors while obeying legal speed limits and vehicle mass dynamics.
6. **Railroad Engineer:** Command diesel-electric freight and commuter trains along authentic rail centerlines, managing tractive throttle and air brakes.
7. **Catamaran Ferry Captain:** Navigate high-speed passenger ferries across shipping fairways and waterways, combating tidal currents and docking at terminal slips.
8. **Commercial & General Aviation Pilot:** Fly turbofan jetliners and high-wing monoplanes along airway waypoints, executing instrument ILS approaches and crosswind landings.

---

## 4. Save/Load Persistence Schema

Persistent saves are stored in structured JSON under the user profile directory:
```json
{
  "schema_version": 2,
  "timestamp_unix": 1730000000,
  "profile": {
    "player_id": "pilot-speed",
    "name": "SpeedDemon",
    "level": 3,
    "xp": 1420,
    "wallet": {
      "balance": 15000.0,
      "transactions": []
    },
    "reputation": {
      "FreeRoam": 450,
      "TaxiPassenger": 120
    },
    "completed_jobs": 25,
    "failed_jobs": 1,
    "garage": {
      "vehicles": [
        {
          "id": "hypersport_rr1000",
          "model_name": "HyperSport RR-1000",
          "category": "Motorcycle",
          "condition_pct": 100.0
        }
      ],
      "active_index": 0
    }
  },
  "active_region_id": "san-francisco",
  "vehicle_position": {
    "lat": 37.7749,
    "lon": -122.4194,
    "alt": 15.0
  },
  "settings": {
    "difficulty": "Standard",
    "metric_units": true,
    "currency_symbol": "₹",
    "abs_enabled": true,
    "tcs_enabled": true,
    "esc_enabled": true
  }
}
```
All schema variations from version 1 are forward-migrated on load via [`SaveMigrationEngine`](file:///j:/rustRealEarth/engine/gameplay/src/save.rs#L100).
