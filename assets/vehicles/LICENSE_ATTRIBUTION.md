# Real Earth Vehicle Fleet Series 2026 — Asset Attribution & Licensing Manifest

**Release Stage**: Stage 0.8 (Complete Realistic Vehicle Visual Overhaul)  
**License**: Apache-2.0 / MIT Dual License (Open-Source, Royalty-Free, Fully Redistributable)  
**Design Authority**: Real Earth Vehicle Art & Graphics Engineering Team  
**Trademark Policy**: All 3D vehicle geometries, body panels, badging, names, and visual designs are 100% original, non-infringing automotive engineering creations. No registered manufacturer trademarks, copyrighted brand emblems, or proprietary trade dresses are reproduced.

---

## 1. Vehicle Fleet Specifications & Asset Manifest

| Asset Identifier | Commercial Classification | Dimensions (L × W × H) | Wheelbase / Track | Wheel Radius | Triangle Budget (LOD0 / LOD1 / LOD2) | PBR Material Set |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `apex_gt_sports_coupe` | High-Performance GT Sports Coupe | 4.45m × 1.92m × 1.25m | 2.50m / 1.65m | 0.33m (245/40R19) | 2,840 / 1,180 / 320 | Cherry Red Metallic Paint, Tinted Solar Glass, Satin Black Diffuser/Splitter, Carbon Ceramic Discs, Red Brembo Calipers |
| `executive_sedan` | Full-Size Luxury Executive Sedan | 4.85m × 1.86m × 1.45m | 2.70m / 1.60m | 0.33m (235/50R18) | 2,420 / 980 / 280 | Sapphire Metallic Blue, Chrome Surround, Smoke Tint Glass, Silver Multi-Spoke Alloys |
| `overland_explorer_suv` | Mid-Size All-Terrain AWD SUV | 4.90m × 1.98m × 1.75m | 2.85m / 1.72m | 0.38m (265/65R17) | 2,680 / 1,040 / 310 | Granite Gray, Matte Cladding, Roof Rails, Deep Tread All-Terrain Tires |
| `metro_yellow_cab` | Urban Municipal Medallion Taxi | 4.85m × 1.86m × 1.52m | 2.70m / 1.60m | 0.33m (235/50R18) | 2,560 / 1,020 / 290 | Municipal Yellow, Checkerboard Decals, Illuminated Roof Lantern, Fare Card Panels |
| `transcontinental_semi` | Class 8 Heavy Highway Tractor | 6.20m × 2.55m × 3.60m | 4.20m / 2.05m | 0.52m (295/75R22.5)| 3,460 / 1,420 / 410 | Crimson Cab, Chrome Dual Stacks, Dual Tandem Drive Axles, Fifth-Wheel Plate |
| `cityline_transit_bus` | 40ft Low-Floor Urban Transit Bus | 11.8m × 2.50m × 3.10m | 6.80m / 2.15m | 0.50m (275/70R22.5)| 3,120 / 1,280 / 380 | City Green Transit Livery, LED Destination Matrix, Dual Boarding Portals |
| `hypersport_rr1000` | 1000cc Supersport Motorcycle | 2.10m × 0.75m × 1.15m | 1.42m / 0.16m | 0.31m (120/70ZR17) | 2,140 / 890 / 240 | Racing Orange & Carbon, Twin Floating Discs, Trellis Frame, Monoshock |
| `pacific_locomotive` | Diesel-Electric Freight Locomotive | 18.5m × 3.10m × 4.60m | 12.5m / 1.435m| 0.55m Steel Flanged | 3,820 / 1,560 / 460 | Union Blue & Gold, Twin 3-Axle Bogies, High-Intensity Cyclops Headlamp |
| `golden_gate_ferry` | High-Speed Catamaran Passenger Ferry| 42.0m × 11.5m × 8.50m | Twin Hull (8.5m) | N/A (Waterjets) | 4,200 / 1,740 / 520 | Maritime White & Blue, Twin Sponsons, Panoramic Passenger Saloon, Radar Mast |
| `skycourier_b737` | Twin-Turbofan Narrow-Body Airliner | 38.5m × 34.3m × 12.5m | 14.8m / 5.72m | 0.45m Nose / 0.55m Main| 4,680 / 1,920 / 580 | Polished Aluminum & White, Swept Wings with Winglets, Dual CFM Turbofans |

---

## 2. glTF 2.0 / GLB Node Hierarchy Conventions

Each vehicle asset strictly conforms to the Khronos glTF 2.0 specification with standard named semantic submeshes:
1. `Chassis`: Main sculpted body panels, structural frame, bumpers, fenders, hood, doors, roof.
2. `Glass`: Transparent and tinted greenhouse surfaces, windshield, rear window, side glass.
3. `Interior`: Cockpit dashboard, instrument binnacle, center console, contoured front bucket seats.
4. `Steering_Wheel`: Driver steering wheel on steering column, articulating with physical steering deflection.
5. `Wheel_FL`: Front-Left wheel assembly (alloy rim, tire, disc rotor, brake caliper).
6. `Wheel_FR`: Front-Right wheel assembly (alloy rim, tire, disc rotor, brake caliper).
7. `Wheel_RL`: Rear-Left wheel assembly (alloy rim, tire, disc rotor, brake caliper).
8. `Wheel_RR`: Rear-Right wheel assembly (alloy rim, tire, disc rotor, brake caliper).
9. `Headlights`: Front projector lenses with dynamic emissive illumination states.
10. `BrakeLights`: Rear taillight clusters and high-mount stop lamps with dynamic braking intensification.
11. `ReverseLights`: Rear backup lamps activating on reverse gear transmission engagement.
12. `Spoiler`: Aerodynamic rear GT wing mounted to rear decklid stanchions.
13. `Exhaust`: Dual or quad polished stainless-steel exhaust tips.
14. `ContactShadow`: Ground contact ambient occlusion planar mesh with smooth radial falloff.

---

## 3. Redistribution & Commercial Use Permissions

- **Permitted Uses**: Full redistribution with source code or compiled binary executables, commercial and non-commercial simulation, interactive rendering, game distribution, and academic benchmarks.
- **Warranty Disclaimer**: Provided "AS IS", without warranty of any kind, express or implied.
