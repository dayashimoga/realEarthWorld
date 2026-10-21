//! Building polygon footprint verification, winding order repair, and area validation.

use crate::provenance::ValidatedHeight;
use crate::report::{QuarantineLog, RepairLog, RepairType};
use realearth_ingestion::osm_types::{BuildingType, OsmBuildingFeature};
use realearth_shared::coordinates::Wgs84Coord;

/// Validated building feature ready for 3D extrusion.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedBuilding {
    pub id: u64,
    pub name: Option<String>,
    pub building_type: BuildingType,
    pub outer_ring: Vec<Wgs84Coord>,
    pub inner_rings: Vec<Vec<Wgs84Coord>>,
    pub height: ValidatedHeight,
    pub area_m2: f64,
}

/// Calculates signed planar area of a 2D ring in degree coordinates.
/// Positive = Counter-Clockwise (CCW), Negative = Clockwise (CW).
pub fn signed_ring_area(ring: &[Wgs84Coord]) -> f64 {
    if ring.len() < 3 {
        return 0.0;
    }
    let mut sum = 0.0;
    for i in 0..ring.len() {
        let j = (i + 1) % ring.len();
        sum += (ring[j].lon - ring[i].lon) * (ring[j].lat + ring[i].lat);
    }
    sum * 0.5
}

/// Approximates metric ground area in square meters for a WGS84 polygon.
pub fn approximate_metric_area_m2(ring: &[Wgs84Coord]) -> f64 {
    if ring.len() < 3 {
        return 0.0;
    }
    let mean_lat = ring.iter().map(|p| p.lat).sum::<f64>() / ring.len() as f64;
    let lat_rad = mean_lat.to_radians();

    // Meters per degree
    let m_per_deg_lat = 111_132.954 - 559.822 * (2.0 * lat_rad).cos();
    let m_per_deg_lon = 111_412.84 * lat_rad.cos();

    let mut area = 0.0;
    for i in 0..ring.len() {
        let j = (i + 1) % ring.len();
        let x_i = ring[i].lon * m_per_deg_lon;
        let y_i = ring[i].lat * m_per_deg_lat;
        let x_j = ring[j].lon * m_per_deg_lon;
        let y_j = ring[j].lat * m_per_deg_lat;
        area += x_i * y_j - x_j * y_i;
    }
    (area * 0.5).abs()
}

/// Validates, repairs, and attests a building feature.
pub fn validate_building(
    building: &OsmBuildingFeature,
    repairs: &mut Vec<RepairLog>,
    quarantined: &mut Vec<QuarantineLog>,
) -> Option<ValidatedBuilding> {
    if building.outer_ring.len() < 3 {
        quarantined.push(QuarantineLog {
            feature_id: building.id,
            failure_reason: format!(
                "Building polygon has < 3 vertices ({})",
                building.outer_ring.len()
            ),
        });
        return None;
    }

    let mut clean_ring = building.outer_ring.clone();

    // 1. Ensure closed ring: first vertex must equal last vertex
    let first = clean_ring[0];
    let last = clean_ring[clean_ring.len() - 1];
    let d_lat = (first.lat - last.lat).abs();
    let d_lon = (first.lon - last.lon).abs();

    if d_lat > 1e-7 || d_lon > 1e-7 {
        clean_ring.push(first);
        repairs.push(RepairLog {
            feature_id: building.id,
            repair_type: RepairType::UnclosedRingClosed,
            description: "Appended initial vertex to close open polygon ring".to_string(),
            confidence_score: 1.0,
        });
    }

    if clean_ring.len() < 4 {
        quarantined.push(QuarantineLog {
            feature_id: building.id,
            failure_reason: "Closed polygon ring has < 4 vertices".to_string(),
        });
        return None;
    }

    // 2. Ensure Counter-Clockwise (CCW) winding order for exterior ring
    let area_deg = signed_ring_area(&clean_ring);
    if area_deg.abs() < 1e-12 {
        quarantined.push(QuarantineLog {
            feature_id: building.id,
            failure_reason: "Degenerate collinear polygon with near-zero area".to_string(),
        });
        return None;
    }

    // In latitude/longitude (where Y=lat, X=lon), signed_ring_area with sum(dx * (y1+y2)) is > 0 for CW
    if area_deg > 0.0 {
        clean_ring.reverse();
        repairs.push(RepairLog {
            feature_id: building.id,
            repair_type: RepairType::InvertedWindingOrderReversed,
            description: "Reversed clockwise exterior ring to counter-clockwise (CCW)".to_string(),
            confidence_score: 1.0,
        });
    }

    // 3. Minimum footprint ground area check (>= 4.0 m^2)
    let area_m2 = approximate_metric_area_m2(&clean_ring);
    if area_m2 < 4.0 {
        quarantined.push(QuarantineLog {
            feature_id: building.id,
            failure_reason: format!(
                "Footprint area too small ({:.2} m^2 < 4.0 m^2 threshold)",
                area_m2
            ),
        });
        return None;
    }

    // 4. Height attribution with provenance
    let validated_height = ValidatedHeight::from_building(building);
    if building.explicit_height.is_none() {
        if let Some(levels) = building.levels {
            repairs.push(RepairLog {
                feature_id: building.id,
                repair_type: RepairType::HeightInferredFromLevels,
                description: format!(
                    "Inferred height {:.1}m from {} building levels",
                    validated_height.height_meters, levels
                ),
                confidence_score: 0.85,
            });
        } else {
            repairs.push(RepairLog {
                feature_id: building.id,
                repair_type: RepairType::HeightAssignedDefaultCategory,
                description: format!(
                    "Assigned default category height {:.1}m for {:?}",
                    validated_height.height_meters, building.building_type
                ),
                confidence_score: 0.50,
            });
        }
    }

    Some(ValidatedBuilding {
        id: building.id,
        name: building.name.clone(),
        building_type: building.building_type,
        outer_ring: clean_ring,
        inner_rings: building.inner_rings.clone(),
        height: validated_height,
        area_m2,
    })
}
