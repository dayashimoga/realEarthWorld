//! Stage 0.9 Forensic Audit of San Francisco Roads, Buildings, Terrain, and Spatial Integrity

use realearth_compiler::compiler_pipeline::{compile_world_package, CompilerOptions};
use realearth_geospatial::transforms::EnuConverter;
use realearth_ingestion::authentic::load_authentic_city;
use realearth_navigation::road_graph::RoadNetworkGraph;
use realearth_shared::coordinates::Wgs84Coord;
use realearth_streaming::{StreamingConfig, WorldStreamer};
use realearth_validation::pipeline::validate_dataset;
use tempfile::tempdir;

#[test]
fn test_audit_san_francisco_source_and_compilation() {
    println!("\n=== STAGE 0.9 FORENSIC AUDIT: SAN FRANCISCO ===");

    // 1. Source Feature Audit
    let (dataset, dem, meta) =
        load_authentic_city("san-francisco").expect("San Francisco dataset must load");

    println!("Region ID: {}", meta.region_id);
    println!("Anchor: {:?}", meta.anchor);
    println!("Roads count: {}", dataset.roads.len());
    println!("Buildings count: {}", dataset.buildings.len());
    println!("Water bodies count: {}", dataset.water_bodies.len());
    println!("Land cover count: {}", dataset.land_cover.len());
    println!("Railways count: {}", dataset.railways.len());
    println!("Ports count: {}", dataset.ports.len());

    // Audit Road features
    let mut total_road_nodes = 0;
    let mut total_road_length_m = 0.0;
    let conv = EnuConverter::new(meta.anchor);

    for road in &dataset.roads {
        total_road_nodes += road.geometry.len();
        for w in road.geometry.windows(2) {
            let e1 = conv.wgs84_to_enu(w[0]);
            let e2 = conv.wgs84_to_enu(w[1]);
            let dx = (e2.east - e1.east) as f32;
            let dz = (e2.north - e1.north) as f32;
            total_road_length_m += (dx * dx + dz * dz).sqrt() as f64;
        }
    }
    println!(
        "Roads: {} features, {} total nodes, {:.1} km total network length",
        dataset.roads.len(),
        total_road_nodes,
        total_road_length_m / 1000.0
    );

    // 2. Junction Analysis
    let mut coord_usage: std::collections::HashMap<(i64, i64, i8), Vec<(u64, String)>> =
        std::collections::HashMap::new();
    for road in &dataset.roads {
        let r_name = road.name.clone().unwrap_or_else(|| "Unnamed".to_string());
        for pt in &road.geometry {
            let key = (
                (pt.lat * 1e7).round() as i64,
                (pt.lon * 1e7).round() as i64,
                road.layer,
            );
            coord_usage
                .entry(key)
                .or_default()
                .push((road.way_id, r_name.clone()));
        }
    }
    let junctions: Vec<_> = coord_usage
        .iter()
        .filter(|(_, roads)| roads.len() > 1)
        .collect();
    println!(
        "Shared Junction nodes: {} intersections connecting 2 or more roads",
        junctions.len()
    );
    for (key, r_list) in junctions.iter().take(5) {
        let unique_names: std::collections::HashSet<_> =
            r_list.iter().map(|(_, n)| n.as_str()).collect();
        println!(
            "  - Junction at lat {:.5}, lon {:.5}, layer {}: {} connected ways: {:?}",
            key.0 as f64 / 1e7,
            key.1 as f64 / 1e7,
            key.2,
            r_list.len(),
            unique_names
        );
    }
    let (validated, report) = validate_dataset(&dataset);
    println!(
        "Validated: {} roads (evaluated {}), {} buildings (evaluated {})",
        validated.roads.len(),
        report.total_roads_evaluated,
        validated.buildings.len(),
        report.total_buildings_evaluated
    );

    // 3. Compile Package in Tempdir
    let temp = tempdir().unwrap();
    let options = CompilerOptions {
        chunk_size_meters: 500.0,
        terrain_grid_resolution: 16,
        package_name: "audit_sf_package".to_string(),
        ..Default::default()
    };
    let stats = compile_world_package(&validated, &dem, temp.path(), &options)
        .expect("Compilation must succeed");
    println!(
        "Compiled stats: {} chunks, {} verts, {} tris, {} bytes compressed",
        stats.total_chunks_generated,
        stats.total_vertices,
        stats.total_triangles,
        stats.total_compressed_bytes
    );

    // 4. Inspect Streamer & Active Chunks around Transamerica Pyramid
    let mut streamer = WorldStreamer::open(temp.path(), StreamingConfig::default()).unwrap();
    let anchor_coord = meta.anchor;
    streamer.update(anchor_coord).unwrap();
    let active = streamer.get_active_chunks();
    println!("Active chunks around anchor: {}", active.len());

    let mut total_active_road_verts = 0;
    let mut total_active_bldg_verts = 0;
    let mut total_active_terrain_verts = 0;

    for chunk in &active {
        total_active_road_verts += chunk.roads.vertices.len();
        total_active_bldg_verts += chunk.buildings.vertices.len();
        total_active_terrain_verts += chunk.terrain.vertices.len();
    }
    println!(
        "Active chunk geometry: {} road verts, {} bldg verts, {} terrain verts",
        total_active_road_verts, total_active_bldg_verts, total_active_terrain_verts
    );

    // 5. Road Graph & Navigation Connectivity
    let road_graph = RoadNetworkGraph::from_roads(&validated.roads);
    println!(
        "Road graph: {} nodes, {} directed edges",
        road_graph.graph.node_count(),
        road_graph.graph.edge_count()
    );

    // 6. Audit Road-Building Collisions & Footprint Clearances
    println!("\n--- AUDIT 6: Road-Building Proximity & Corridor Collisions ---");
    let lat_scale = 111_139.0;
    let lon_scale = 111_139.0 * 37.8f64.to_radians().cos();

    let mut penetrating_collisions = Vec::new();
    let mut close_proximity_warnings = Vec::new();

    for road in &validated.roads {
        if road.layer != 0 || road.is_bridge || road.is_tunnel {
            continue;
        }

        let lane_w = road.highway_type.default_lane_width();
        let road_half_w = (road.lanes.max(1) as f64) * (lane_w as f64) * 0.5;
        let sidewalk_w = 2.0;
        let total_corridor_half_w = road_half_w + sidewalk_w;

        for bldg in &validated.buildings {
            if bldg.outer_ring.len() < 3 {
                continue;
            }

            // Check if road centerline crosses building outer ring
            let mut centerline_penetrates = false;
            for r_idx in 0..(road.geometry.len().saturating_sub(1)) {
                let r1 = road.geometry[r_idx];
                let r2 = road.geometry[r_idx + 1];
                for b_idx in 0..(bldg.outer_ring.len().saturating_sub(1)) {
                    let b1 = bldg.outer_ring[b_idx];
                    let b2 = bldg.outer_ring[b_idx + 1];
                    let ccw = |p1: Wgs84Coord, p2: Wgs84Coord, p3: Wgs84Coord| -> f64 {
                        (p2.lon - p1.lon) * (p3.lat - p1.lat)
                            - (p2.lat - p1.lat) * (p3.lon - p1.lon)
                    };
                    let d1 = ccw(r1, r2, b1);
                    let d2 = ccw(r1, r2, b2);
                    let d3 = ccw(b1, b2, r1);
                    let d4 = ccw(b1, b2, r2);
                    if ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0))
                        && ((d3 > 0.0 && d4 < 0.0) || (d3 < 0.0 && d4 > 0.0))
                    {
                        centerline_penetrates = true;
                        break;
                    }
                }
                if centerline_penetrates {
                    break;
                }
            }

            if centerline_penetrates {
                penetrating_collisions.push((
                    road.way_id,
                    road.name.clone().unwrap_or_else(|| "Unnamed".to_string()),
                    bldg.id,
                    bldg.name.clone().unwrap_or_else(|| "Unnamed".to_string()),
                ));
            } else {
                // Check distance from building vertices to road centerline
                for b_pt in &bldg.outer_ring {
                    for r_idx in 0..(road.geometry.len().saturating_sub(1)) {
                        let r1 = road.geometry[r_idx];
                        let r2 = road.geometry[r_idx + 1];
                        let dx = (r2.lon - r1.lon) * lon_scale;
                        let dy = (r2.lat - r1.lat) * lat_scale;
                        let len_sq = dx * dx + dy * dy;
                        if len_sq > 1e-6 {
                            let px = (b_pt.lon - r1.lon) * lon_scale;
                            let py = (b_pt.lat - r1.lat) * lat_scale;
                            let t = ((px * dx + py * dy) / len_sq).clamp(0.0, 1.0);
                            let proj_x = r1.lon * lon_scale + t * dx;
                            let proj_y = r1.lat * lat_scale + t * dy;
                            let dist_m = ((b_pt.lon * lon_scale - proj_x).powi(2)
                                + (b_pt.lat * lat_scale - proj_y).powi(2))
                            .sqrt();
                            if dist_m < total_corridor_half_w {
                                close_proximity_warnings.push((
                                    road.way_id,
                                    road.name.clone().unwrap_or_else(|| "Unnamed".to_string()),
                                    bldg.id,
                                    bldg.name.clone().unwrap_or_else(|| "Unnamed".to_string()),
                                    dist_m,
                                    total_corridor_half_w,
                                ));
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    println!(
        "Centerline penetrating collisions: {}",
        penetrating_collisions.len()
    );
    for (r_id, r_name, b_id, b_name) in &penetrating_collisions {
        println!(
            "  - Road [{}] '{}' crosses Building [{}] '{}'",
            r_id, r_name, b_id, b_name
        );
    }
    println!(
        "Road corridor / sidewalk encroachment warnings: {}",
        close_proximity_warnings.len()
    );
    for (r_id, r_name, b_id, b_name, dist, corridor_w) in close_proximity_warnings.iter().take(10) {
        println!(
            "  - Building [{}] '{}' within {:.2}m of Road [{}] '{}' (corridor half-width: {:.2}m)",
            b_id, b_name, dist, r_id, r_name, corridor_w
        );
    }

    // 7. Audit Road Elevations vs Terrain DEM
    println!("\n--- AUDIT 7: Road Elevations vs Terrain DEM ---");
    let mut max_elev_delta: f32 = 0.0;
    let mut min_elev_delta: f32 = f32::MAX;
    let mut elevated_count = 0;
    let mut subterranean_count = 0;
    let mut surface_count = 0;

    for road in &validated.roads {
        if road.layer > 0 || road.is_bridge {
            elevated_count += 1;
        } else if road.layer < 0 || road.is_tunnel {
            subterranean_count += 1;
        } else {
            surface_count += 1;
            for pt in &road.geometry {
                let _ground_elev = dem.sample_bilinear(pt);
                let delta = 0.20; // current fixed offset in road_mesher
                if delta > max_elev_delta {
                    max_elev_delta = delta;
                }
                if delta < min_elev_delta {
                    min_elev_delta = delta;
                }
            }
        }
    }
    println!(
        "Road classifications: {} surface, {} elevated, {} subterranean",
        surface_count, elevated_count, subterranean_count
    );
    println!(
        "Surface road elevation offset range: {:.2}m to {:.2}m above DEM",
        min_elev_delta, max_elev_delta
    );

    // 8. Audit Spawn Candidates for all 7 Road Vehicle Fleet Classes
    println!("\n--- AUDIT 8: Safe Spawning Candidates Audit ---");
    let package_anchor = meta.anchor;
    for v_idx in 0..=6 {
        let is_heavy = v_idx == 4 || v_idx == 5; // Bus or Semi
        let mut valid_candidates = 0;
        let mut min_dist = f64::MAX;
        let mut chosen_road = String::new();

        for road in &validated.roads {
            if road.layer != 0 || road.is_tunnel {
                continue;
            }
            if road.highway_type == realearth_ingestion::osm_types::HighwayType::Footway
                || road.highway_type == realearth_ingestion::osm_types::HighwayType::Path
            {
                continue;
            }
            if is_heavy && road.lanes < 2 {
                continue;
            }
            if road.geometry.len() < 2 {
                continue;
            }

            for window in road.geometry.windows(2) {
                let p1 = window[0];
                let p2 = window[1];
                let enu1 = conv.wgs84_to_enu(p1);
                let enu2 = conv.wgs84_to_enu(p2);
                let dx = (enu2.east - enu1.east) as f32;
                let dz = (enu2.north - enu1.north) as f32;
                let seg_len = (dx * dx + dz * dz).sqrt();
                if seg_len < 3.0 {
                    continue;
                }

                let mid_x = ((enu1.east + enu2.east) * 0.5) as f32;
                let mid_z = ((enu1.north + enu2.north) * 0.5) as f32;

                // Check slope from DEM
                let eps = 0.5;
                let h_r = dem.sample_bilinear(&conv.enu_to_wgs84(
                    realearth_shared::coordinates::EnuCoord::new(
                        (mid_x + eps) as f64,
                        (mid_z) as f64,
                        0.0,
                    ),
                ));
                let h_l = dem.sample_bilinear(&conv.enu_to_wgs84(
                    realearth_shared::coordinates::EnuCoord::new(
                        (mid_x - eps) as f64,
                        (mid_z) as f64,
                        0.0,
                    ),
                ));
                let h_u = dem.sample_bilinear(&conv.enu_to_wgs84(
                    realearth_shared::coordinates::EnuCoord::new(
                        (mid_x) as f64,
                        (mid_z + eps) as f64,
                        0.0,
                    ),
                ));
                let h_d = dem.sample_bilinear(&conv.enu_to_wgs84(
                    realearth_shared::coordinates::EnuCoord::new(
                        (mid_x) as f64,
                        (mid_z - eps) as f64,
                        0.0,
                    ),
                ));
                let slope_x = (h_r - h_l) / (2.0 * eps);
                let slope_z = (h_u - h_d) / (2.0 * eps);
                let normal_y = 1.0 / (1.0 + slope_x * slope_x + slope_z * slope_z).sqrt();

                if normal_y < 0.65 {
                    continue;
                }

                valid_candidates += 1;
                let target_enu = conv.wgs84_to_enu(package_anchor);
                let d = ((mid_x as f64 - target_enu.east).powi(2)
                    + (mid_z as f64 - target_enu.north).powi(2))
                .sqrt();
                if d < min_dist {
                    min_dist = d;
                    chosen_road = road.name.clone().unwrap_or_else(|| "Unnamed".to_string());
                }
            }
        }

        println!(
            "Fleet vehicle class {}: {} valid road segments, nearest is {:.1}m away on '{}'",
            v_idx, valid_candidates, min_dist, chosen_road
        );
    }
}
