//! Missing-Data Diagnostics and Transparent Planetary Basemap Fallback.
//! Strictly upholds the "No Fabrication Rule": identifies geographic voids
//! (missing DEM, roads, buildings) and supplies explicit low-resolution
//! basemap geometry with clear provenance labeling.

use realearth_compiler::chunk::WorldChunk;
use realearth_compiler::mesh::{ChunkMesh, Vertex};
use realearth_geospatial::GlobalCellKey;
use realearth_shared::bounds::GeoBounds;
use realearth_shared::coordinates::Wgs84Coord;
use serde::{Deserialize, Serialize};

/// Type classification for detected missing geographic features.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissingDataCategory {
    DigitalElevationModel,
    RoadNetwork,
    BuildingFootprints,
    WaterBoundaries,
}

/// A specific diagnostic record detailing a missing data void.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissingDataRecord {
    pub cell: GlobalCellKey,
    pub category: MissingDataCategory,
    pub bounds: GeoBounds,
    pub recommendation: String,
}

/// Comprehensive diagnostic report of geographic completeness within an area.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct MissingDataReport {
    pub total_cells_scanned: usize,
    pub missing_records: Vec<MissingDataRecord>,
    pub has_elevation_void: bool,
    pub has_transport_void: bool,
    pub has_building_void: bool,
}

impl MissingDataReport {
    pub fn is_fully_complete(&self) -> bool {
        self.missing_records.is_empty()
    }
}

/// Diagnostic scanner identifying spatial data gaps across geographic regions.
pub struct MissingDataDiagnostics;

impl MissingDataDiagnostics {
    /// Scans a collection of planetary cells to detect missing coverage.
    pub fn scan_cells(
        cells: &[GlobalCellKey],
        has_dem: impl Fn(&GlobalCellKey) -> bool,
        has_roads: impl Fn(&GlobalCellKey) -> bool,
        has_buildings: impl Fn(&GlobalCellKey) -> bool,
    ) -> MissingDataReport {
        let mut report = MissingDataReport {
            total_cells_scanned: cells.len(),
            missing_records: Vec::new(),
            has_elevation_void: false,
            has_transport_void: false,
            has_building_void: false,
        };

        for cell in cells {
            let bounds = cell.bounds();

            if !has_dem(cell) {
                report.has_elevation_void = true;
                report.missing_records.push(MissingDataRecord {
                    cell: *cell,
                    category: MissingDataCategory::DigitalElevationModel,
                    bounds,
                    recommendation: "Fall back to Copernicus GLO-90 or Planetary Basemap DEM"
                        .to_string(),
                });
            }

            if !has_roads(cell) {
                report.has_transport_void = true;
                report.missing_records.push(MissingDataRecord {
                    cell: *cell,
                    category: MissingDataCategory::RoadNetwork,
                    bounds,
                    recommendation: "Preserve terrain continuity; no fabricated roads permitted"
                        .to_string(),
                });
            }

            if !has_buildings(cell) {
                report.has_building_void = true;
                report.missing_records.push(MissingDataRecord {
                    cell: *cell,
                    category: MissingDataCategory::BuildingFootprints,
                    bounds,
                    recommendation:
                        "Leave terrain open; no procedural building fabrication without survey"
                            .to_string(),
                });
            }
        }

        report
    }
}

/// Provider of transparent, un-fabricated coarse planetary basemap geometry
/// for unmapped ocean, rural, or missing-data void regions.
pub struct FallbackBasemapProvider;

impl FallbackBasemapProvider {
    /// Generates a placeholder coarse terrain chunk for a missing planetary cell
    /// labeled with explicit fallback provenance.
    pub fn generate_coarse_terrain_chunk(cell: &GlobalCellKey) -> WorldChunk {
        let bounds = cell.bounds();
        let center = cell.center();

        let mut terrain = ChunkMesh::default();
        let norm = [0.0, 1.0, 0.0];
        // Neutral earthy green-gray color distinguishing basemap fallback
        let col = [0.4, 0.5, 0.35, 1.0];

        // 4 corner vertices in local meters relative to center anchor
        let min_wgs = Wgs84Coord::new(bounds.min_lat, bounds.min_lon, 0.0);
        let max_wgs = Wgs84Coord::new(bounds.max_lat, bounds.max_lon, 0.0);

        let d_lat_m = (max_wgs.lat - min_wgs.lat) * 111_000.0;
        let d_lon_m = (max_wgs.lon - min_wgs.lon) * 111_000.0 * center.lat.to_radians().cos().abs();

        let half_w = (d_lon_m * 0.5) as f32;
        let half_h = (d_lat_m * 0.5) as f32;

        terrain
            .vertices
            .push(Vertex::new([-half_w, 0.0, -half_h], norm, [0.0, 0.0], col));
        terrain
            .vertices
            .push(Vertex::new([half_w, 0.0, -half_h], norm, [1.0, 0.0], col));
        terrain
            .vertices
            .push(Vertex::new([-half_w, 0.0, half_h], norm, [0.0, 1.0], col));
        terrain
            .vertices
            .push(Vertex::new([half_w, 0.0, half_h], norm, [1.0, 1.0], col));

        terrain.indices.extend_from_slice(&[0, 1, 2, 1, 3, 2]);

        WorldChunk {
            header: realearth_compiler::chunk::ChunkHeader {
                magic: *realearth_compiler::chunk::CHUNK_MAGIC,
                version: realearth_compiler::chunk::CHUNK_VERSION,
                chunk_x: cell.x as i32,
                chunk_y: cell.y as i32,
                lod_level: cell.level as u32,
                bounds,
                anchor: center,
                min_elevation: 0.0,
                max_elevation: 0.0,
            },
            terrain,
            roads: ChunkMesh::default(),
            buildings: ChunkMesh::default(),
            water: ChunkMesh::default(),
            vegetation: ChunkMesh::default(),
            building_metadata: Vec::new(),
            road_metadata: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_data_diagnostics_scan() {
        let cell = GlobalCellKey::new(10, 500, 500).unwrap();
        let cells = vec![cell];

        // Scenario: DEM exists, but roads and buildings are missing
        let report = MissingDataDiagnostics::scan_cells(
            &cells,
            |_| true,  // DEM present
            |_| false, // Roads missing
            |_| false, // Buildings missing
        );

        assert!(!report.is_fully_complete());
        assert!(!report.has_elevation_void);
        assert!(report.has_transport_void);
        assert!(report.has_building_void);
        assert_eq!(report.missing_records.len(), 2);
    }

    #[test]
    fn test_fallback_basemap_chunk_generation() {
        let cell = GlobalCellKey::new(4, 8, 8).unwrap();
        let chunk = FallbackBasemapProvider::generate_coarse_terrain_chunk(&cell);

        assert_eq!(chunk.terrain.vertices.len(), 4);
        assert_eq!(chunk.terrain.indices.len(), 6);
        assert!(
            chunk.roads.vertices.is_empty(),
            "Never fabricate roads in fallback"
        );
        assert!(
            chunk.buildings.vertices.is_empty(),
            "Never fabricate buildings in fallback"
        );
    }
}
