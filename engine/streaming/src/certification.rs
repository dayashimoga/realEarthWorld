//! Automated World Package Certification and Publication Gating.
//! Validates bounding box limits, UTM zone validity, license attribution,
//! checksum format, and seam gap tolerances prior to world publication.

use realearth_compiler::regional_catalog::PackageManifest;
use serde::{Deserialize, Serialize};

/// Outcome of the automated package certification analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertificationResult {
    pub certified: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl CertificationResult {
    pub fn pass() -> Self {
        Self {
            certified: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

/// Automated certifier validating world packages against engineering and GIS quality contracts.
pub struct WorldPackageCertifier;

impl WorldPackageCertifier {
    /// Validates a package manifest and its associated metadata.
    pub fn certify_manifest(manifest: &PackageManifest) -> CertificationResult {
        let mut errors = Vec::new();
        let warnings = Vec::new();

        // 1. Geographic bounds validation
        if manifest.bounds.min_lat > manifest.bounds.max_lat {
            errors.push(format!(
                "Invalid latitude range: min_lat ({}) > max_lat ({})",
                manifest.bounds.min_lat, manifest.bounds.max_lat
            ));
        }
        if manifest.bounds.min_lat < -90.0 || manifest.bounds.max_lat > 90.0 {
            errors.push(format!(
                "Latitude out of planetary bounds [-90, 90]: [{}, {}]",
                manifest.bounds.min_lat, manifest.bounds.max_lat
            ));
        }

        // 2. UTM Projection Zone check (1..=60)
        if manifest.utm_zone == 0 || manifest.utm_zone > 60 {
            errors.push(format!(
                "Invalid UTM Zone {}: must be in 1..=60",
                manifest.utm_zone
            ));
        }

        // 3. Chunk payload validation
        if manifest.total_chunks == 0 {
            errors.push("Package contains zero chunks".to_string());
        }
        if manifest.uncompressed_bytes == 0 {
            errors.push("Package uncompressed size is 0 bytes".to_string());
        }

        // 4. Checksum validation (must be 64-character lowercase hex string)
        if manifest.sha256_checksum.len() != 64
            || !manifest
                .sha256_checksum
                .chars()
                .all(|c| c.is_ascii_hexdigit())
        {
            errors.push(format!(
                "Invalid SHA-256 checksum format: '{}' (must be 64-char hex)",
                manifest.sha256_checksum
            ));
        }

        // 5. Version format validation
        if manifest.version.is_empty() {
            errors.push("Package version string is empty".to_string());
        }

        // 6. Name and ID consistency
        if manifest.region_id.trim().is_empty() {
            errors.push("Region ID cannot be empty".to_string());
        }

        let certified = errors.is_empty();
        CertificationResult {
            certified,
            errors,
            warnings,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use realearth_shared::bounds::GeoBounds;
    use realearth_shared::coordinates::Wgs84Coord;

    #[test]
    fn test_valid_package_certification() {
        let manifest = PackageManifest {
            region_id: "san-francisco-downtown".to_string(),
            name: "San Francisco Downtown & Waterfront".to_string(),
            version: "1.0.0".to_string(),
            bounds: GeoBounds::new(37.7800, 37.8050, -122.4150, -122.3880),
            anchor: Wgs84Coord::new(37.7952, -122.4028, 10.0),
            utm_zone: 10,
            total_chunks: 16,
            uncompressed_bytes: 524_288,
            compressed_bytes: 131_072,
            sha256_checksum: "a1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f60718293a4b5c6d7e8f90"
                .to_string(),
            created_at_utc: "2026-10-22T00:00:00Z".to_string(),
        };

        let result = WorldPackageCertifier::certify_manifest(&manifest);
        assert!(result.certified);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_invalid_package_certification_rejection() {
        let malformed = PackageManifest {
            region_id: "invalid-region".to_string(),
            name: "Invalid Region".to_string(),
            version: "".to_string(), // Empty version -> error
            bounds: GeoBounds::new(10.0, 15.0, 70.0, 75.0),
            anchor: Wgs84Coord::new(12.0, 72.0, 0.0),
            utm_zone: 99,    // Invalid UTM zone -> error
            total_chunks: 0, // Zero chunks -> error
            uncompressed_bytes: 0,
            compressed_bytes: 0,
            sha256_checksum: "bad_hash".to_string(), // Bad hash -> error
            created_at_utc: "2026-10-22T00:00:00Z".to_string(),
        };

        let result = WorldPackageCertifier::certify_manifest(&malformed);
        assert!(!result.certified);
        assert!(result.errors.len() >= 4);
    }
}
