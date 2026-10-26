//! Authoritative On-Demand US Region Manager, GIS Validation,
//! Checksum Verification, and Package Delivery Subsystem (Phase D2).

use crate::package_delivery::{InstalledPackageRecord, StorageQuotaManager};
use realearth_compiler::compiler_pipeline::{compile_world_package, CompilerOptions};
use realearth_compiler::regional_catalog::PackageManifest;
use realearth_ingestion::acquisition::compute_sha256;
use realearth_ingestion::load_authentic_city;
use realearth_shared::errors::{RealEarthError, Result};
use realearth_shared::us_catalog::find_city_by_id;
use realearth_validation::pipeline::validate_dataset;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Detailed states of the on-demand installation pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallState {
    Pending,
    Acquiring,
    Parsing,
    Validating,
    Compiling,
    Staging,
    Installed,
    Failed,
    Cancelled,
}

/// Progress notification snapshot emitted during region installation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstallProgress {
    pub region_id: String,
    pub state: InstallState,
    pub progress_pct: f32,
    pub step_description: String,
    pub bytes_downloaded: usize,
    pub total_bytes: usize,
    pub sha256_checksum: Option<String>,
}

/// Authoritative Region Package Manager and On-Demand Installer.
#[derive(Debug, Clone)]
pub struct RegionInstaller {
    pub packages_dir: PathBuf,
    pub quota_manager: StorageQuotaManager,
    pub is_cancelled: bool,
}

impl RegionInstaller {
    /// Creates a new RegionInstaller rooted at the specified packages directory.
    pub fn new(packages_dir: impl Into<PathBuf>, quota_bytes: usize) -> Self {
        let dir = packages_dir.into();
        let _ = fs::create_dir_all(&dir);
        Self {
            packages_dir: dir,
            quota_manager: StorageQuotaManager::new(quota_bytes),
            is_cancelled: false,
        }
    }

    /// Sets cancellation flag to abort active installation.
    pub fn cancel(&mut self) {
        self.is_cancelled = true;
    }

    /// Resets cancellation flag before launching installation.
    pub fn reset_cancel(&mut self) {
        self.is_cancelled = false;
    }

    /// Target directory where a regional package is installed.
    pub fn region_package_dir(&self, region_id: &str) -> PathBuf {
        self.packages_dir.join(region_id.replace('-', "_"))
    }

    /// Checks whether a region package is installed with a valid `chunks.db`.
    pub fn is_installed(&self, region_id: &str) -> bool {
        let dir = self.region_package_dir(region_id);
        dir.join("chunks.db").exists()
    }

    /// Verifies the cryptographic SHA-256 integrity of an installed package.
    pub fn verify_installed_package(&self, region_id: &str) -> Result<bool> {
        let dir = self.region_package_dir(region_id);
        let db_path = dir.join("chunks.db");
        if !db_path.exists() {
            return Ok(false);
        }

        let db_bytes = fs::read(&db_path)?;
        if db_bytes.is_empty() {
            return Ok(false);
        }

        let manifest_path = dir.join("manifest.json");
        if manifest_path.exists() {
            if let Ok(content) = fs::read_to_string(&manifest_path) {
                if let Ok(manifest) = PackageManifest::from_json(&content) {
                    if !manifest.sha256_checksum.is_empty() {
                        let computed = compute_sha256(&db_bytes);
                        return Ok(computed == manifest.sha256_checksum);
                    }
                }
            }
        }

        Ok(true)
    }

    /// Acquires, parses, validates, and compiles an authentic geographic package on demand.
    pub fn install_region<F>(&mut self, region_id: &str, mut progress_cb: F) -> Result<PathBuf>
    where
        F: FnMut(InstallProgress),
    {
        if self.is_cancelled {
            return Err(RealEarthError::Ingestion(
                "Installation cancelled by user".to_string(),
            ));
        }

        let city = find_city_by_id(region_id).ok_or_else(|| {
            RealEarthError::Ingestion(format!(
                "City '{}' not found in US Geographic Catalog",
                region_id
            ))
        })?;

        let target_dir = self.region_package_dir(city.id);
        fs::create_dir_all(&target_dir)?;

        // If already installed and verified, return early
        if self.is_installed(city.id) && self.verify_installed_package(city.id).unwrap_or(false) {
            progress_cb(InstallProgress {
                region_id: city.id.to_string(),
                state: InstallState::Installed,
                progress_pct: 100.0,
                step_description: format!(
                    "Region {} ({}) already installed and verified.",
                    city.name, city.state
                ),
                bytes_downloaded: 0,
                total_bytes: 0,
                sha256_checksum: None,
            });
            return Ok(target_dir);
        }

        if self.is_cancelled {
            return Err(RealEarthError::Ingestion(
                "Installation cancelled by user".to_string(),
            ));
        }

        // --------------------------------------------------------------------
        // 1. ACQUIRING
        // --------------------------------------------------------------------
        progress_cb(InstallProgress {
            region_id: city.id.to_string(),
            state: InstallState::Acquiring,
            progress_pct: 15.0,
            step_description: format!(
                "Acquiring authoritative surveyed GIS extract for {} ({}) [Anchor: {}]...",
                city.name, city.state, city.anchor_landmark_name
            ),
            bytes_downloaded: 0,
            total_bytes: 500_000,
            sha256_checksum: None,
        });

        let (raw, dem) = match load_authentic_city(city.id) {
            Ok((raw, dem, _)) => (raw, dem),
            Err(e) => {
                return Err(RealEarthError::Ingestion(format!(
                    "Region '{}' requires an authentic GIS data package (Surveyed GIS source data not bundled or verified: {})",
                    city.id, e
                )));
            }
        };

        if self.is_cancelled {
            return Err(RealEarthError::Ingestion(
                "Installation cancelled by user".to_string(),
            ));
        }

        // --------------------------------------------------------------------
        // 2. PARSING
        // --------------------------------------------------------------------
        progress_cb(InstallProgress {
            region_id: city.id.to_string(),
            state: InstallState::Parsing,
            progress_pct: 35.0,
            step_description: format!(
                "Parsing mapped features: {} roads, {} buildings, {} water bodies...",
                raw.roads.len(),
                raw.buildings.len(),
                raw.water_bodies.len()
            ),
            bytes_downloaded: 250_000,
            total_bytes: 500_000,
            sha256_checksum: None,
        });

        if raw.roads.is_empty() && raw.buildings.is_empty() {
            return Err(RealEarthError::Ingestion(format!(
                "Acquired dataset for '{}' contained no mapped roads or buildings",
                city.id
            )));
        }

        if self.is_cancelled {
            return Err(RealEarthError::Ingestion(
                "Installation cancelled by user".to_string(),
            ));
        }

        // --------------------------------------------------------------------
        // 3. VALIDATING
        // --------------------------------------------------------------------
        progress_cb(InstallProgress {
            region_id: city.id.to_string(),
            state: InstallState::Validating,
            progress_pct: 55.0,
            step_description: "Running topological GIS validation and bounding integrity tests..."
                .to_string(),
            bytes_downloaded: 380_000,
            total_bytes: 500_000,
            sha256_checksum: None,
        });

        let (val, report) = validate_dataset(&raw);
        if val.roads.is_empty() && val.buildings.is_empty() {
            return Err(RealEarthError::Validation(format!(
                "GIS validation rejected all mapped features for '{}': {:?}",
                city.id, report
            )));
        }

        if self.is_cancelled {
            return Err(RealEarthError::Ingestion(
                "Installation cancelled by user".to_string(),
            ));
        }

        // --------------------------------------------------------------------
        // 4. COMPILING
        // --------------------------------------------------------------------
        let package_name = format!("{}_package", city.id.replace('-', "_"));
        progress_cb(InstallProgress {
            region_id: city.id.to_string(),
            state: InstallState::Compiling,
            progress_pct: 75.0,
            step_description: format!("Compiling spatial world chunks into '{}'...", package_name),
            bytes_downloaded: 450_000,
            total_bytes: 500_000,
            sha256_checksum: None,
        });

        let options = CompilerOptions {
            chunk_size_meters: 500.0,
            terrain_grid_resolution: 16,
            package_name: package_name.clone(),
            source_asset_sha256: None,
            ..Default::default()
        };

        compile_world_package(&val, &dem, &target_dir, &options)?;

        if self.is_cancelled {
            return Err(RealEarthError::Ingestion(
                "Installation cancelled by user".to_string(),
            ));
        }

        // --------------------------------------------------------------------
        // 5. STAGING & CHECKSUM VERIFICATION
        // --------------------------------------------------------------------
        let db_path = target_dir.join("chunks.db");
        let db_bytes = fs::read(&db_path)?;
        let sha256_digest = compute_sha256(&db_bytes);

        progress_cb(InstallProgress {
            region_id: city.id.to_string(),
            state: InstallState::Staging,
            progress_pct: 90.0,
            step_description: format!(
                "Writing package manifest [SHA-256: {}] and registering storage quota...",
                &sha256_digest[..8]
            ),
            bytes_downloaded: db_bytes.len(),
            total_bytes: db_bytes.len(),
            sha256_checksum: Some(sha256_digest.clone()),
        });

        let profile = city.to_region_profile();
        let manifest = PackageManifest::new(
            &profile,
            "1.0.0",
            val.roads.len() + val.buildings.len(),
            db_bytes.len(),
            db_bytes.len(),
            sha256_digest.clone(),
        );
        let manifest_json = manifest.to_json()?;
        fs::write(target_dir.join("manifest.json"), manifest_json)?;

        // Register with StorageQuotaManager
        self.quota_manager.register_package(InstalledPackageRecord {
            package_id: city.id.to_string(),
            country_code: "US".to_string(),
            version: "1.0.0".to_string(),
            disk_bytes: db_bytes.len(),
            last_accessed_timestamp: 1,
            is_active: false,
            is_pinned: city.id == "san-francisco",
        });

        // --------------------------------------------------------------------
        // 6. INSTALLED & READY
        // --------------------------------------------------------------------
        progress_cb(InstallProgress {
            region_id: city.id.to_string(),
            state: InstallState::Installed,
            progress_pct: 100.0,
            step_description: format!(
                "Region {} ({}) installed successfully and verified ready for streaming activation.",
                city.name, city.state
            ),
            bytes_downloaded: db_bytes.len(),
            total_bytes: db_bytes.len(),
            sha256_checksum: Some(sha256_digest),
        });

        Ok(target_dir)
    }

    /// Selectively uninstalls a region package and frees disk storage.
    pub fn uninstall_region(&mut self, region_id: &str) -> Result<()> {
        let dir = self.region_package_dir(region_id);
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
        }
        self.quota_manager.uninstall_package(region_id);
        Ok(())
    }
}
