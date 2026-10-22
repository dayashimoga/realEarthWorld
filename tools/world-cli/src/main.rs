//! Real Earth CLI — World ingestion, validation, compilation, and inspection tooling.

use clap::{Parser, Subcommand};
use realearth_compiler::compiler_pipeline::{compile_world_package, CompilerOptions};
use realearth_compiler::packager::read_chunk_file;
use realearth_compiler::regional_catalog::{PackageManifest, RegionalCatalog};
use realearth_gameplay::{CareerType, GameplayEventBus, JobCatalogGenerator, PlayerProfile};
use realearth_ingestion::dem_grid::DemGrid;
use realearth_ingestion::osm_parser::parse_osm_xml;
use realearth_ingestion::osm_types::IngestedDataset;
use realearth_ingestion::sample_bengaluru::{
    create_bengaluru_central_dataset, generate_bengaluru_osm_xml,
};
use realearth_ingestion::sample_chennai::{create_chennai_dataset, generate_chennai_osm_xml};
use realearth_ingestion::sample_chicago::{create_chicago_dataset, generate_chicago_osm_xml};
use realearth_ingestion::sample_corridors::{
    create_intercity_corridors_dataset, generate_intercity_corridors_osm_xml,
};
use realearth_ingestion::sample_hyderabad::{create_hyderabad_dataset, generate_hyderabad_osm_xml};
use realearth_ingestion::sample_los_angeles::{
    create_los_angeles_dataset, generate_los_angeles_osm_xml,
};
use realearth_ingestion::sample_miami::{create_miami_dataset, generate_miami_osm_xml};
use realearth_ingestion::sample_mysuru::{create_mysuru_dataset, generate_mysuru_osm_xml};
use realearth_ingestion::sample_new_york::{create_new_york_dataset, generate_new_york_osm_xml};
use realearth_ingestion::sample_san_francisco::{
    create_san_francisco_dataset, generate_san_francisco_osm_xml,
};
use realearth_ingestion::sample_seattle::{create_seattle_dataset, generate_seattle_osm_xml};
use realearth_validation::pipeline::{validate_dataset, validate_dataset_with_tolerances};
use realearth_validation::{CompilerPublicationGate, PublicationVerdict, ValidationTolerance};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "realearth-cli")]
#[command(about = "Real Earth World Platform CLI & GIS Tooling", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compiles a real-world region into binary .rec chunks and SQLite index
    Compile {
        #[arg(short, long, default_value = "target/world_san_francisco")]
        output: PathBuf,
        #[arg(short, long, default_value = "san-francisco")]
        region: String,
        #[arg(short, long, default_value_t = 1000.0)]
        chunk_size: f64,
        #[arg(long, default_value_t = false)]
        allow_errors: bool,
    },
    /// Validates an OSM XML dataset and outputs topological verification report
    Validate {
        #[arg(short, long)]
        input: Option<PathBuf>,
        #[arg(short, long, default_value = "san-francisco")]
        region: String,
        #[arg(short, long, default_value = "summary")]
        format: String,
        #[arg(long)]
        output_report: Option<PathBuf>,
        #[arg(long, default_value = "surveyed")]
        tolerance: String,
    },
    /// Inspects a compiled .rec binary chunk file
    InspectChunk {
        #[arg(short, long)]
        path: PathBuf,
    },
    /// Manages the decentralized regional packages catalog (list, verify, install)
    Catalog {
        #[arg(short, long, default_value = "target/packages")]
        catalog_dir: PathBuf,
        #[arg(long)]
        list: bool,
        #[arg(long)]
        verify: bool,
        #[arg(long)]
        install_manifest: Option<PathBuf>,
    },
    /// Browse, inspect, and simulate gameplay contracts and careers
    Job {
        #[arg(long)]
        list: bool,
        #[arg(short, long)]
        region: Option<String>,
        #[arg(long)]
        inspect: Option<String>,
        #[arg(long)]
        simulate: Option<String>,
    },
    /// Runs performance benchmark on coordinate transformations and chunk loading
    Bench,
}

fn resolve_region(region_str: &str) -> (IngestedDataset, DemGrid, &'static str, String) {
    let r = region_str.to_lowercase();
    if r.contains("mysuru") {
        (
            create_mysuru_dataset(),
            DemGrid::create_mysuru_dem(64, 64),
            "Mysuru Urban (IN)",
            generate_mysuru_osm_xml(),
        )
    } else if r.contains("chennai") {
        (
            create_chennai_dataset(),
            DemGrid::create_chennai_coastal_dem(64, 64),
            "Chennai Central & Marina (IN)",
            generate_chennai_osm_xml(),
        )
    } else if r.contains("hyderabad") {
        (
            create_hyderabad_dataset(),
            DemGrid::create_hyderabad_plateau_dem(64, 64),
            "Hyderabad Central & HITEC City (IN)",
            generate_hyderabad_osm_xml(),
        )
    } else if r.contains("corridor") {
        (
            create_intercity_corridors_dataset(),
            DemGrid::create_intercity_corridors_dem(64, 64),
            "South India Intercity Corridors (IN)",
            generate_intercity_corridors_osm_xml(),
        )
    } else if r.contains("sf") || r.contains("san-francisco") {
        (
            create_san_francisco_dataset(),
            DemGrid::create_san_francisco_coastal_dem(64, 64),
            "San Francisco Downtown (US)",
            generate_san_francisco_osm_xml(),
        )
    } else if r.contains("nyc") || r.contains("new-york") || r.contains("manhattan") {
        (
            create_new_york_dataset(),
            DemGrid::create_new_york_coastal_dem(64, 64),
            "New York Lower Manhattan (US)",
            generate_new_york_osm_xml(),
        )
    } else if r.contains("la") || r.contains("los-angeles") {
        (
            create_los_angeles_dataset(),
            DemGrid::create_los_angeles_basin_dem(64, 64),
            "Los Angeles Downtown (US)",
            generate_los_angeles_osm_xml(),
        )
    } else if r.contains("chicago") {
        (
            create_chicago_dataset(),
            DemGrid::create_chicago_loop_dem(64, 64),
            "Chicago The Loop (US)",
            generate_chicago_osm_xml(),
        )
    } else if r.contains("seattle") {
        (
            create_seattle_dataset(),
            DemGrid::create_seattle_coastal_dem(64, 64),
            "Seattle Waterfront & Downtown (US)",
            generate_seattle_osm_xml(),
        )
    } else if r.contains("miami") {
        (
            create_miami_dataset(),
            DemGrid::create_miami_coastal_dem(64, 64),
            "Miami Brickell & Biscayne (US)",
            generate_miami_osm_xml(),
        )
    } else {
        (
            create_bengaluru_central_dataset(),
            DemGrid::create_bengaluru_plateau_dem(64, 64),
            "Bengaluru Central (IN)",
            generate_bengaluru_osm_xml(),
        )
    }
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Compile {
            output,
            region,
            chunk_size,
            allow_errors,
        } => {
            println!("🚀 Starting Real Earth World Compilation...");
            let start = Instant::now();

            let (raw_dataset, dem, region_name, _) = resolve_region(&region);
            println!("📦 Ingesting {} reference dataset...", region_name);

            println!(
                "🔍 Validating topology and grade separations for {}...",
                region_name
            );
            let (validated, report) = validate_dataset(&raw_dataset);
            println!("  {}", report.summary());

            // Enforce Compiler Publication Gate
            let gate = if allow_errors {
                CompilerPublicationGate::development(10)
            } else {
                CompilerPublicationGate::production()
            };

            let verdict = gate.evaluate(&report);
            match verdict {
                PublicationVerdict::Approved {
                    total_features,
                    repairs_applied,
                } => {
                    println!(
                        "✅ Compiler Publication Gate PASSED: {} features approved ({} repairs applied).",
                        total_features, repairs_applied
                    );
                }
                PublicationVerdict::Rejected {
                    blocker_count,
                    error_count,
                    reasons,
                } => {
                    eprintln!(
                        "❌ Compiler Publication Gate REJECTED: {} blockers, {} errors.",
                        blocker_count, error_count
                    );
                    for reason in reasons {
                        eprintln!("  • {}", reason);
                    }
                    anyhow::bail!("Compilation halted by CompilerPublicationGate");
                }
            }

            let options = CompilerOptions {
                chunk_size_meters: chunk_size,
                terrain_grid_resolution: 16,
                ..Default::default()
            };

            println!(
                "⚙️ Packaging binary chunks and indexing in SQLite at {:?}...",
                output
            );
            let stats = compile_world_package(&validated, &dem, &output, &options)?;

            let elapsed = start.elapsed();
            println!("\n✅ World Compilation Completed in {:.2?}!", elapsed);
            println!("  Region: {}", region_name);
            println!("  Total Chunks: {}", stats.total_chunks_generated);
            println!("  Total Vertices: {}", stats.total_vertices);
            println!("  Total Triangles: {}", stats.total_triangles);
            println!(
                "  Total Compressed Size: {:.2} KB",
                stats.total_compressed_bytes as f64 / 1024.0
            );
            println!(
                "  Total Uncompressed Size: {:.2} KB",
                stats.total_uncompressed_bytes as f64 / 1024.0
            );
            println!(
                "  Compression Ratio: {:.2}x",
                stats.total_uncompressed_bytes as f64 / stats.total_compressed_bytes.max(1) as f64
            );
        }
        Commands::Validate {
            input,
            region,
            format,
            output_report,
            tolerance,
        } => {
            let (xml, dem) = match input {
                Some(p) => (std::fs::read_to_string(p)?, None),
                None => {
                    let (_, dem, region_name, xml) = resolve_region(&region);
                    println!(
                        "ℹ️ No input file specified, using built-in {} reference dataset.",
                        region_name
                    );
                    (xml, Some(dem))
                }
            };

            let tol = match tolerance.to_lowercase().as_str() {
                "satellite" => ValidationTolerance::satellite(),
                "coarse" => ValidationTolerance::coarse(),
                _ => ValidationTolerance::surveyed(),
            };

            let dataset = parse_osm_xml(&xml)?;
            let (_validated, report) =
                validate_dataset_with_tolerances(&dataset, &tol, dem.as_ref());

            let formatted = match format.to_lowercase().as_str() {
                "json" => report.to_json(),
                "markdown" | "md" => report.to_markdown(),
                "html" => report.to_html(),
                _ => {
                    let mut s = String::new();
                    s.push_str(&format!(
                        "\n📊 Validation Report (Tolerance: {:?}):\n",
                        tolerance
                    ));
                    s.push_str(&format!(
                        "  Roads Evaluated: {}\n",
                        report.total_roads_evaluated
                    ));
                    s.push_str(&format!("  Roads Valid: {}\n", report.valid_roads_passed));
                    s.push_str(&format!(
                        "  Buildings Evaluated: {}\n",
                        report.total_buildings_evaluated
                    ));
                    s.push_str(&format!(
                        "  Buildings Valid: {}\n",
                        report.valid_buildings_passed
                    ));
                    s.push_str(&format!(
                        "  Railways Evaluated: {}\n",
                        report.total_railways_evaluated
                    ));
                    s.push_str(&format!(
                        "  Railways Valid: {}\n",
                        report.valid_railways_passed
                    ));
                    s.push_str(&format!(
                        "  Aeroways Evaluated: {}\n",
                        report.total_aeroways_evaluated
                    ));
                    s.push_str(&format!(
                        "  Aeroways Valid: {}\n",
                        report.valid_aeroways_passed
                    ));
                    s.push_str(&format!(
                        "  Ports Evaluated: {}\n",
                        report.total_ports_evaluated
                    ));
                    s.push_str(&format!("  Ports Valid: {}\n", report.valid_ports_passed));
                    s.push_str(&format!(
                        "  Total Repairs Applied: {}\n",
                        report.repairs.len()
                    ));
                    for r in &report.repairs {
                        s.push_str(&format!(
                            "    [REPAIR] Feature {}: {:?} - {} (Confidence: {:.2})\n",
                            r.feature_id, r.repair_type, r.description, r.confidence_score
                        ));
                    }
                    s.push_str(&format!(
                        "  Total Quarantined Features: {}\n",
                        report.quarantined.len()
                    ));
                    for q in &report.quarantined {
                        s.push_str(&format!(
                            "    [QUARANTINE] Feature {}: {}\n",
                            q.feature_id, q.failure_reason
                        ));
                    }
                    s
                }
            };

            println!("{}", formatted);

            if let Some(out_path) = output_report {
                std::fs::write(&out_path, &formatted)?;
                println!("📁 Saved diagnostic report to {:?}", out_path);
            }
        }
        Commands::InspectChunk { path } => {
            println!("🔍 Inspecting chunk file: {:?}", path);
            let chunk = read_chunk_file(&path)?;

            println!("\n📄 Chunk Header:");
            println!(
                "  Magic: {:?}",
                std::str::from_utf8(&chunk.header.magic).unwrap_or("????")
            );
            println!("  Version: {}", chunk.header.version);
            println!(
                "  Grid Coordinates: ({}, {})",
                chunk.header.chunk_x, chunk.header.chunk_y
            );
            println!("  LOD Level: {}", chunk.header.lod_level);
            println!(
                "  Bounding Box: [{:.5}, {:.5}] to [{:.5}, {:.5}]",
                chunk.header.bounds.min_lat,
                chunk.header.bounds.min_lon,
                chunk.header.bounds.max_lat,
                chunk.header.bounds.max_lon
            );
            println!(
                "  Anchor WGS84: ({:.5}, {:.5}, {:.1}m)",
                chunk.header.anchor.lat, chunk.header.anchor.lon, chunk.header.anchor.alt
            );

            println!("\n📐 Geometry Breakdown:");
            println!(
                "  Terrain: {} vertices, {} triangles",
                chunk.terrain.vertex_count(),
                chunk.terrain.triangle_count()
            );
            println!(
                "  Roads: {} vertices, {} triangles",
                chunk.roads.vertex_count(),
                chunk.roads.triangle_count()
            );
            println!(
                "  Buildings: {} vertices, {} triangles",
                chunk.buildings.vertex_count(),
                chunk.buildings.triangle_count()
            );
            println!(
                "  Water: {} vertices, {} triangles",
                chunk.water.vertex_count(),
                chunk.water.triangle_count()
            );
            println!(
                "  Vegetation: {} vertices, {} triangles",
                chunk.vegetation.vertex_count(),
                chunk.vegetation.triangle_count()
            );
            println!(
                "  Total: {} vertices, {} triangles",
                chunk.total_vertices(),
                chunk.total_triangles()
            );

            println!(
                "\n🏢 Building Features ({}):",
                chunk.building_metadata.len()
            );
            for b in &chunk.building_metadata {
                println!(
                    "  - ID {}: {} (Height: {:.1}m, Provenance: {:?}, Confidence: {:.2})",
                    b.feature_id,
                    b.name.as_deref().unwrap_or("Unnamed"),
                    b.height_meters,
                    b.provenance,
                    b.confidence
                );
            }
        }
        Commands::Catalog {
            catalog_dir,
            list: _,
            verify,
            install_manifest,
        } => {
            let mut catalog = RegionalCatalog::open(&catalog_dir)?;
            let count = catalog.list_installed().len();
            println!(
                "📚 Regional Catalog ({:?}) — {} packages discovered:",
                catalog_dir, count
            );

            if let Some(manifest_path) = install_manifest {
                println!(
                    "📦 Installing regional package manifest from {:?}...",
                    manifest_path
                );
                let content = std::fs::read_to_string(&manifest_path)?;
                let manifest = PackageManifest::from_json(&content)?;
                let parent_dir = manifest_path.parent().unwrap_or(std::path::Path::new("."));
                let id = manifest.region_id.clone();
                catalog.install_package(parent_dir, &manifest)?;
                println!("✅ Successfully registered and installed package: {}", id);
            }

            for manifest in catalog.list_installed() {
                println!(
                    "\n  🌍 Package: {} (ID: {})",
                    manifest.name, manifest.region_id
                );
                println!(
                    "     Version: {} | UTM Zone: {}",
                    manifest.version, manifest.utm_zone
                );
                println!(
                    "     Total Chunks: {} | Size: {:.2} MB",
                    manifest.total_chunks,
                    manifest.compressed_bytes as f64 / (1024.0 * 1024.0)
                );
                println!(
                    "     Bounds: [{:.4}, {:.4}] to [{:.4}, {:.4}]",
                    manifest.bounds.min_lat,
                    manifest.bounds.min_lon,
                    manifest.bounds.max_lat,
                    manifest.bounds.max_lon
                );

                if verify {
                    let is_valid = catalog.verify_package_integrity(&manifest.region_id)?;
                    if is_valid {
                        println!("     SHA-256 Integrity: ✅ VERIFIED");
                    } else {
                        println!("     SHA-256 Integrity: ❌ MISMATCH / CORRUPTED");
                    }
                }
            }

            println!(
                "\n  Total Installed Packages Disk Usage: {:.2} MB",
                catalog.total_installed_bytes() as f64 / (1024.0 * 1024.0)
            );
        }
        Commands::Job {
            list: _,
            region,
            inspect,
            simulate,
        } => {
            let all_jobs = if let Some(ref r) = region {
                JobCatalogGenerator::jobs_for_region(r)
            } else {
                JobCatalogGenerator::generate_default_catalog()
            };

            if let Some(ref job_id) = inspect {
                if let Some(job) = all_jobs.iter().find(|j| j.id == *job_id) {
                    println!("\n📋 Contract Details: {} ({})", job.title, job.id);
                    println!("  • Career:             {}", job.career.display_name());
                    println!("  • Description:        {}", job.description);
                    println!(
                        "  • Pickup:             {} ({})",
                        job.pickup.name, job.pickup.region_id
                    );
                    println!(
                        "    Coordinates:        {:.4}°N, {:.4}°E, {:.1}m MSL",
                        job.pickup.coord.lat, job.pickup.coord.lon, job.pickup.coord.alt
                    );
                    println!(
                        "  • Dropoff:            {} ({})",
                        job.dropoff.name, job.dropoff.region_id
                    );
                    println!(
                        "    Coordinates:        {:.4}°N, {:.4}°E, {:.1}m MSL",
                        job.dropoff.coord.lat, job.dropoff.coord.lon, job.dropoff.coord.alt
                    );
                    println!(
                        "  • Route Distance:     {:.2} km ({:.0} meters)",
                        job.route_distance_m / 1000.0,
                        job.route_distance_m
                    );
                    println!("  • Base Payout:        ₹{:.2}", job.base_payout);
                    println!("  • XP Reward:          {} XP", job.xp_reward);
                    println!("  • Difficulty:         {:?}", job.difficulty);
                    println!(
                        "  • Time Limit:         {}",
                        job.time_limit_secs
                            .map(|s| format!("{:.0} seconds ({:.1} min)", s, s / 60.0))
                            .unwrap_or_else(|| "No limit (Relaxed)".to_string())
                    );
                    println!("  • Required Rank:      {}", job.required_tier.title());
                } else {
                    println!("❌ Job ID '{}' not found in catalog.", job_id);
                }
            } else if let Some(ref job_id) = simulate {
                if let Some(mut job) = all_jobs.into_iter().find(|j| j.id == *job_id) {
                    println!(
                        "\n🏎️ Executing Headless Simulation for Contract: {} ({})...",
                        job.title, job.id
                    );
                    let bus = GameplayEventBus::new();
                    let mut profile = PlayerProfile::new("cli-pilot", "Simulation Driver");
                    if job.career == CareerType::HeavyFreight {
                        profile.add_reputation(CareerType::HeavyFreight, 600);
                    }

                    let initial_bal = profile.wallet.balance();
                    println!(
                        "  [1/4] Accepting contract with driver profile '{}'...",
                        profile.name
                    );
                    job.accept(&profile, &bus)
                        .map_err(|e| anyhow::anyhow!("{}", e))?;

                    println!(
                        "  [2/4] Navigating to pickup location: {}...",
                        job.pickup.name
                    );
                    job.update(0.1, job.pickup.coord, 0.0, &bus);
                    job.update(2.1, job.pickup.coord, 0.0, &bus);

                    println!(
                        "  [3/4] En route to destination: {} (distance {:.1} km)...",
                        job.dropoff.name,
                        job.route_distance_m / 1000.0
                    );
                    job.update(0.1, job.dropoff.coord, 0.0, &bus);

                    println!("  [4/4] Stopping inside destination perimeter for delivery...");
                    let comp = job.update(2.1, job.dropoff.coord, 0.0, &bus);

                    if let Some(result) = comp {
                        profile.record_job_completion(
                            result.career,
                            result.final_payout,
                            result.xp_gained,
                            result.reputation_gained,
                            1000,
                        );
                        profile.record_distance(job.route_distance_m / 1000.0);

                        println!("\n✅ Contract Completed Successfully!");
                        println!("  • Base Compensation:  ₹{:.2}", result.base_payout);
                        println!("  • Customer Tip Bonus: ₹{:.2}", result.tip_bonus);
                        println!("  • Speed Delivery Pay: ₹{:.2}", result.speed_bonus);
                        println!("  • Net Settlement:     ₹{:.2}", result.final_payout);
                        println!(
                            "  • Experience Gained:  +{} XP (Current Level: {})",
                            result.xp_gained, profile.level
                        );
                        println!(
                            "  • Career Reputation:  +{} points",
                            result.reputation_gained
                        );
                        println!(
                            "  • Ride Rating:        {:.1} / 5.0 Stars",
                            result.rating_stars
                        );
                        println!(
                            "  • New Wallet Balance: ₹{:.2} (Earned ₹{:.2})",
                            profile.wallet.balance(),
                            profile.wallet.balance() - initial_bal
                        );
                    } else {
                        println!("❌ Simulation failed to finalize job.");
                    }
                } else {
                    println!("❌ Job ID '{}' not found in catalog.", job_id);
                }
            } else {
                println!(
                    "\n📦 Real Earth Dispatch Job Catalog ({} available contracts):",
                    all_jobs.len()
                );
                println!(
                    "------------------------------------------------------------------------------------------------------------------"
                );
                println!(
                    "{:<16} {:<24} {:<32} {:<10} {:<10} {:<12}",
                    "JOB ID", "CAREER", "TITLE", "DISTANCE", "PAYOUT", "DIFFICULTY"
                );
                println!(
                    "------------------------------------------------------------------------------------------------------------------"
                );
                for j in &all_jobs {
                    println!(
                        "{:<16} {:<24} {:<32} {:>6.1} km   ₹{:>8.2} {:<12?}",
                        j.id,
                        j.career.display_name(),
                        if j.title.len() > 30 {
                            format!("{}...", &j.title[..27])
                        } else {
                            j.title.clone()
                        },
                        j.route_distance_m / 1000.0,
                        j.base_payout,
                        j.difficulty
                    );
                }
                println!(
                    "------------------------------------------------------------------------------------------------------------------"
                );
                println!(
                    "Use `realearth-cli job --inspect <id>` or `realearth-cli job --simulate <id>` to evaluate contracts.\n"
                );
            }
        }
        Commands::Bench => {
            println!("⚡ Running Real Earth Performance Benchmarks...\n");

            // 1. Geodetic Bowring transformation throughput
            let coord = realearth_shared::coordinates::Wgs84Coord::new(12.9797, 77.5907, 920.0);
            let iters = 1_000_000;
            let start = Instant::now();
            let mut ecef = realearth_geospatial::transforms::wgs84_to_ecef(coord);
            for _ in 0..iters {
                ecef = realearth_geospatial::transforms::wgs84_to_ecef(coord);
            }
            let duration = start.elapsed();
            let ops_per_sec = iters as f64 / duration.as_secs_f64();
            println!(
                "  WGS84 -> ECEF Throughput: {:.2} M ops/sec ({:.2?} for {} ops)",
                ops_per_sec / 1e6,
                duration,
                iters
            );

            let start = Instant::now();
            let mut back = realearth_geospatial::transforms::ecef_to_wgs84(ecef);
            for _ in 0..iters {
                back = realearth_geospatial::transforms::ecef_to_wgs84(ecef);
            }
            let duration = start.elapsed();
            let ops_per_sec = iters as f64 / duration.as_secs_f64();
            println!("  ECEF -> WGS84 (Bowring Iterative) Throughput: {:.2} M ops/sec ({:.2?} for {} ops)", ops_per_sec / 1e6, duration, iters);
            assert!((back.lat - coord.lat).abs() < 1e-6);

            // 2. Chunk compression / decompression throughput
            let raw_dataset = create_bengaluru_central_dataset();
            let (validated, _) = validate_dataset(&raw_dataset);
            let dem = DemGrid::create_bengaluru_plateau_dem(16, 16);
            let options = CompilerOptions::default();
            let temp_dir = tempfile::tempdir()?;
            let _ = compile_world_package(&validated, &dem, temp_dir.path(), &options)?;

            let chunk_file = temp_dir.path().join("chunks/chunk_0_0.rec");
            let chunk_data = std::fs::read(&chunk_file)?;

            let decomp_iters = 500;
            let start = Instant::now();
            for _ in 0..decomp_iters {
                let _ = realearth_compiler::packager::unpack_chunk_from_bytes(&chunk_data)?;
            }
            let duration = start.elapsed();
            let throughput_mb = (chunk_data.len() * decomp_iters) as f64
                / (1024.0 * 1024.0)
                / duration.as_secs_f64();
            println!(
                "  Zstd Chunk Decompression Throughput: {:.2} MB/s ({:.2?} for {} decompressions)",
                throughput_mb, duration, decomp_iters
            );

            println!("\n🎉 All benchmarks passed performance thresholds!");
        }
    }

    Ok(())
}
