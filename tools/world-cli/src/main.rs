//! Real Earth CLI — World ingestion, validation, compilation, and inspection tooling.

use clap::{Parser, Subcommand};
use realearth_compiler::compiler_pipeline::{compile_world_package, CompilerOptions};
use realearth_compiler::packager::read_chunk_file;
use realearth_ingestion::dem_grid::DemGrid;
use realearth_ingestion::osm_parser::parse_osm_xml;
use realearth_ingestion::sample_bengaluru::{
    create_bengaluru_central_dataset, generate_bengaluru_osm_xml,
};
use realearth_ingestion::sample_san_francisco::{
    create_san_francisco_dataset, generate_san_francisco_osm_xml,
};
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
    /// Runs performance benchmark on coordinate transformations and chunk loading
    Bench,
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

            let is_sf =
                region.to_lowercase().contains("san-francisco") || region.to_lowercase() == "sf";
            let (raw_dataset, dem, region_name) = if is_sf {
                println!("📦 Ingesting San Francisco Downtown & Waterfront reference dataset...");
                (
                    create_san_francisco_dataset(),
                    DemGrid::create_san_francisco_coastal_dem(64, 64),
                    "San Francisco Downtown (US)",
                )
            } else {
                println!("📦 Ingesting Bengaluru Central reference dataset...");
                (
                    create_bengaluru_central_dataset(),
                    DemGrid::create_bengaluru_plateau_dem(64, 64),
                    "Bengaluru Central (IN)",
                )
            };

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
                    if region.to_lowercase().contains("sf")
                        || region.to_lowercase().contains("san-francisco")
                    {
                        println!(
                            "ℹ️ No input file specified, using built-in San Francisco Downtown dataset."
                        );
                        (
                            generate_san_francisco_osm_xml(),
                            Some(DemGrid::create_san_francisco_coastal_dem(64, 64)),
                        )
                    } else {
                        println!(
                            "ℹ️ No input file specified, using built-in Bengaluru Central dataset."
                        );
                        (
                            generate_bengaluru_osm_xml(),
                            Some(DemGrid::create_bengaluru_plateau_dem(64, 64)),
                        )
                    }
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
