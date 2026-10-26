# RealEarth GIS Coverage Report — Production Certification

## Executive Summary
This report certifies the authentic geospatial intelligence and coverage across all 38 surveyed major metropolitan areas in the United States catalog ([`engine/geospatial/src/catalog.rs`](file:///j:/rustRealEarth/engine/geospatial/src/catalog.rs)). Every city in RealEarth is grounded in surveyed OpenStreetMap (OSM) vectors, USGS 3D Elevation Program (3DEP) digital elevation models, and CTBUH skyscraper height registries. Synthetic fallback roads, random procedural blocks, and ungrounded terrain meshes have been completely eliminated.

---

## 1. 38 Major US Metropolitan Areas Catalog

All 38 metropolitan definitions are verified for coordinate validity, bounding box containment of origin anchors, and positive geographic spans ($lat_{\text{span}} > 0, lon_{\text{span}} > 0$).

| Metropolitan Area | State | Anchor Latitude | Anchor Longitude | Bounding Box Extents (SW $\to$ NE) | Signature Landmark / Skyscraper Height |
|---|---|---|---|---|---|
| **San Francisco** | CA | 37.7749° N | -122.4194° W | [37.7081, -122.5180] $\to$ [37.8324, -122.3550] | Salesforce Tower (326m), Transamerica Pyramid (260m) |
| **New York City** | NY | 40.7128° N | -74.0060° W | [40.4960, -74.2557] $\to$ [40.9152, -73.7003] | One World Trade Center (541m), Empire State Building (381m) |
| **Los Angeles** | CA | 34.0522° N | -118.2437° W | [33.7037, -118.6682] $\to$ [34.3373, -118.1553] | Wilshire Grand Center (335m), U.S. Bank Tower (310m) |
| **Chicago** | IL | 41.8781° N | -87.6298° W | [41.6443, -87.9401] $\to$ [42.0231, -87.5237] | Willis Tower (442m), St. Regis Chicago (363m) |
| **Seattle** | WA | 47.6062° N | -122.3321° W | [47.4955, -122.4359] $\to$ [47.7341, -122.2359] | Columbia Center (284m), Space Needle (184m) |
| **Miami** | FL | 25.7617° N | -80.1918° W | [25.7090, -80.3193] $\to$ [25.8558, -80.1310] | Panorama Tower (265m), Four Seasons Hotel (240m) |
| **Austin** | TX | 30.2672° N | -97.7431° W | [30.1500, -97.9000] $\to$ [30.4000, -97.6000] | The Independent (210m), Waterline (312m) |
| **Boston** | MA | 42.3601° N | -71.0589° W | [42.2279, -71.1912] $\to$ [42.3970, -70.9860] | 200 Clarendon / John Hancock (241m), Prudential Tower (228m) |
| **Denver** | CO | 39.7392° N | -104.9903° W | [39.6143, -105.1099] $\to$ [39.9142, -104.6003] | Republic Plaza (218m), 1801 California Street (216m) |
| **Washington D.C.** | DC | 38.9072° N | -77.0369° W | [38.7916, -77.1198] $\to$ [38.9955, -76.9094] | Washington Monument (169m), Basilica of National Shrine (100m) |
| **Atlanta** | GA | 33.7490° N | -84.3880° W | [33.6478, -84.5511] $\to$ [33.8868, -84.2896] | Bank of America Plaza (312m), SunTrust Plaza (265m) |
| **Dallas** | TX | 32.7767° N | -96.7970° W | [32.6180, -96.9989] $\to$ [33.0238, -96.5644] | Bank of America Plaza (281m), Renaissance Tower (270m) |
| **Houston** | TX | 29.7604° N | -95.3698° W | [29.5236, -95.7881] $\to$ [30.1107, -95.0145] | JPMorgan Chase Tower (305m), Wells Fargo Plaza (302m) |
| **Phoenix** | AZ | 33.4484° N | -112.0740° W | [33.2903, -112.3240] $\to$ [33.7483, -111.9250] | Chase Tower (147m), US Bank Center (124m) |
| **Philadelphia** | PA | 39.9526° N | -75.1652° W | [39.8670, -75.2803] $\to$ [40.1380, -74.9558] | Comcast Technology Center (342m), One Liberty Place (288m) |
| **San Diego** | CA | 32.7157° N | -117.1611° W | [32.5348, -117.2824] $\to$ [33.1142, -116.9081] | One America Plaza (152m), Symphony Towers (152m) |
| **Portland** | OR | 45.5152° N | -122.6784° W | [45.4325, -122.8368] $\to$ [45.6529, -122.4720] | Wells Fargo Center (166m), Park Avenue West Tower (153m) |
| **Las Vegas** | NV | 36.1699° N | -115.1398° W | [36.0000, -115.3500] $\to$ [36.3200, -115.0000] | The Strat Tower (350m), Fontainebleau Las Vegas (206m) |
| **Detroit** | MI | 42.3314° N | -83.0458° W | [42.2552, -83.2880] $\to$ [42.4502, -82.9104] | Renaissance Center (221m), One Detroit Center (189m) |
| **Minneapolis** | MN | 44.9778° N | -93.2650° W | [44.8902, -93.3291] $\to$ [45.0513, -93.1938] | IDS Center (241m), Capella Tower (237m) |
| **St. Louis** | MO | 38.6270° N | -90.1994° W | [38.5320, -90.3205] $\to$ [38.7744, -90.1665] | Gateway Arch (192m), One Metropolitan Square (181m) |
| **Baltimore** | MD | 39.2904° N | -76.6122° W | [39.1972, -76.7113] $\to$ [39.3720, -76.5297] | Transamerica Tower (161m), 100 Light Street (152m) |
| **Pittsburgh** | PA | 40.4406° N | -79.9959° W | [40.3615, -80.0955] $\to$ [40.5012, -79.8658] | U.S. Steel Tower (256m), BNY Mellon Center (221m) |
| **Cincinnati** | OH | 39.1031° N | -84.5120° W | [39.0400, -84.6000] $\to$ [39.2000, -84.3800] | Great American Tower (203m), Carew Tower (175m) |
| **Cleveland** | OH | 41.4993° N | -81.6944° W | [41.3900, -81.8200] $\to$ [41.5800, -81.5300] | Key Tower (289m), Terminal Tower (216m) |
| **Kansas City** | MO | 39.0997° N | -94.5786° W | [38.9800, -94.7000] $\to$ [39.2200, -94.4000] | One Kansas City Place (190m), Town Pavilion (180m) |
| **Indianapolis**| IN | 39.7684° N | -86.1581° W | [39.6300, -86.3200] $\to$ [39.9200, -85.9500] | Salesforce Tower Indianapolis (214m), OneAmerica Tower (162m) |
| **Columbus** | OH | 39.9612° N | -82.9988° W | [39.8500, -83.1500] $\to$ [40.1000, -82.8000] | Rhodes State Office Tower (192m), LeVeque Tower (169m) |
| **Charlotte** | NC | 35.2271° N | -80.8431° W | [35.1000, -80.9800] $\to$ [35.3500, -80.7000] | Bank of America Corporate Center (265m), 550 South Tryon (241m)|
| **Nashville** | TN | 36.1627° N | -86.7816° W | [36.0500, -86.9200] $\to$ [36.2500, -86.6500] | AT&T Building / Batman Bldg (188m), 505 Nashville (166m) |
| **Milwaukee** | WI | 43.0389° N | -87.9065° W | [42.9200, -88.0500] $\to$ [43.1500, -87.8500] | U.S. Bank Center (183m), Northwestern Mutual Tower (169m) |
| **Salt Lake City**| UT| 40.7608° N | -111.8910° W| [40.6500, -112.0500] $\to$ [40.8500, -111.8000]| Wells Fargo Center (129m), Astra Tower (137m) |
| **Orlando** | FL | 28.5383° N | -81.3792° W | [28.4000, -81.5000] $\to$ [28.6500, -81.2500] | SunTrust Center (134m), The Vue at Lake Eola (130m) |
| **Tampa** | FL | 27.9506° N | -82.4572° W | [27.8500, -82.5800] $\to$ [28.1000, -82.3500] | 100 North Tampa (176m), Bank of America Plaza (176m) |
| **Sacramento** | CA | 38.5816° N | -121.4944° W| [38.4800, -121.6000] $\to$ [38.6800, -121.3800]| Wells Fargo Center (130m), U.S. Bank Tower (123m) |
| **San Jose** | CA | 37.3382° N | -121.8863° W| [37.2000, -122.0500] $\to$ [37.4500, -121.7500]| 200 Park Avenue (91m), Miro Towers (91m) |
| **Oakland** | CA | 37.8044° N | -122.2711° W| [37.7200, -122.3500] $\to$ [37.8800, -122.1500]| Ordway Building (123m), Kaiser Center (119m) |
| **Honolulu** | HI | 21.3069° N | -157.8583° W| [21.2500, -157.9500] $\to$ [21.3800, -157.7500]| First Hawaiian Center (131m), Moana Pacific (122m) |

---

## 2. Ingestion Pipeline & Elevation Normalization

1. **Vector Processing (`realearth-compiler`):**
   - High-precision parser extracts OSM tags: `highway=*`, `building=*`, `railway=*`, `waterway=*`, and `natural=*`.
   - Polygons are decomposed into indexed triangle lists with texture coordinates mapped to physical metric scales.
2. **Terrain Geodesics & DEM Raster Sampling (`realearth-geospatial`):**
   - WGS84 ellipsoidal coordinates $(\phi, \lambda, h)$ project into local topocentric East-North-Up (ENU) coordinates centered on the metropolitan anchor.
   - Elevation DEM bicubic interpolation prevents mesh tearing at chunk boundaries and guarantees positive road clearances over water.
3. **Zero Synthetic Fallbacks Gate:**
   - Automated test suite `us_city_catalog_validation::test_all_38_cities_authenticated_bounds` verifies that all 38 cities resolve to authentic compiled chunks and bounding boxes without fallback cubes or synthetic grid streets.
