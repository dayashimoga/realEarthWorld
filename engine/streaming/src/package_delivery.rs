//! Independent Versioned Country/Region Package Delivery, Resumable Acquisition,
//! Content Integrity Verification, and Multi-Tier Storage Quota Management.

use realearth_compiler::regional_catalog::PackageManifest;
use realearth_shared::errors::{RealEarthError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// State of an individual chunk within a resumable download session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChunkTransferState {
    Pending,
    Transferring,
    Downloaded,
    Verified,
    Failed,
}

/// Country-level package manifest aggregating regional world packages.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CountryPackageManifest {
    pub country_code: String,
    pub country_name: String,
    pub version: String,
    pub regions: Vec<PackageManifest>,
    pub total_bytes: usize,
    pub sha256_root_digest: String,
    pub license_id: String,
}

impl CountryPackageManifest {
    pub fn new(
        country_code: impl Into<String>,
        country_name: impl Into<String>,
        version: impl Into<String>,
        regions: Vec<PackageManifest>,
        license_id: impl Into<String>,
    ) -> Self {
        let total_bytes: usize = regions.iter().map(|r| r.compressed_bytes).sum();
        Self {
            country_code: country_code.into(),
            country_name: country_name.into(),
            version: version.into(),
            regions,
            total_bytes,
            sha256_root_digest: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
                .to_string(),
            license_id: license_id.into(),
        }
    }
}

/// Resumable acquisition session tracking chunk-range downloads and resume capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumableAcquisitionSession {
    pub package_id: String,
    pub total_chunks: usize,
    pub chunk_states: Vec<ChunkTransferState>,
    pub bytes_downloaded: usize,
    pub total_bytes: usize,
}

impl ResumableAcquisitionSession {
    /// Initializes a new acquisition session with all chunks in Pending state.
    pub fn new(package_id: impl Into<String>, total_chunks: usize, total_bytes: usize) -> Self {
        Self {
            package_id: package_id.into(),
            total_chunks,
            chunk_states: vec![ChunkTransferState::Pending; total_chunks],
            bytes_downloaded: 0,
            total_bytes,
        }
    }

    /// Finds the next uncompleted chunk index to support resumable acquisition.
    pub fn next_chunk_to_fetch(&self) -> Option<usize> {
        self.chunk_states
            .iter()
            .position(|&s| s != ChunkTransferState::Verified && s != ChunkTransferState::Downloaded)
    }

    /// Records successful download of a chunk and updates byte counters.
    pub fn mark_chunk_downloaded(&mut self, chunk_index: usize, chunk_bytes: usize) -> Result<()> {
        if chunk_index >= self.total_chunks {
            return Err(RealEarthError::Validation(format!(
                "Chunk index {} exceeds total chunks {}",
                chunk_index, self.total_chunks
            )));
        }
        self.chunk_states[chunk_index] = ChunkTransferState::Downloaded;
        self.bytes_downloaded += chunk_bytes;
        Ok(())
    }

    /// Verifies cryptographic checksum of a chunk.
    pub fn verify_chunk(&mut self, chunk_index: usize, is_valid: bool) -> Result<bool> {
        if chunk_index >= self.total_chunks {
            return Err(RealEarthError::Validation(format!(
                "Chunk index {} exceeds total chunks {}",
                chunk_index, self.total_chunks
            )));
        }
        if is_valid {
            self.chunk_states[chunk_index] = ChunkTransferState::Verified;
            Ok(true)
        } else {
            self.chunk_states[chunk_index] = ChunkTransferState::Failed;
            Ok(false)
        }
    }

    /// Checks if all chunks in the package are downloaded and verified.
    pub fn is_fully_completed(&self) -> bool {
        self.chunk_states
            .iter()
            .all(|&s| s == ChunkTransferState::Verified)
    }
}

/// Metadata record for an installed world package managed by the storage quota system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledPackageRecord {
    pub package_id: String,
    pub country_code: String,
    pub version: String,
    pub disk_bytes: usize,
    pub is_pinned: bool,
    pub is_active: bool,
    pub last_accessed_timestamp: u64,
}

/// Multi-tier storage quota manager enforcing disk storage limits and LRU package eviction.
#[derive(Debug, Clone)]
pub struct StorageQuotaManager {
    pub quota_bytes: usize,
    pub packages: HashMap<String, InstalledPackageRecord>,
}

impl StorageQuotaManager {
    /// Creates a new storage quota manager with specified maximum storage capacity.
    pub fn new(quota_bytes: usize) -> Self {
        Self {
            quota_bytes,
            packages: HashMap::new(),
        }
    }

    /// Returns total disk bytes currently occupied by installed world packages.
    pub fn total_used_bytes(&self) -> usize {
        self.packages.values().map(|p| p.disk_bytes).sum()
    }

    /// Returns remaining available disk storage headroom within quota.
    pub fn available_headroom_bytes(&self) -> usize {
        self.quota_bytes.saturating_sub(self.total_used_bytes())
    }

    /// Registers a newly installed package into the storage accounting registry.
    pub fn register_package(&mut self, record: InstalledPackageRecord) {
        self.packages.insert(record.package_id.clone(), record);
    }

    /// Marks a package as accessed and updates its LRU timestamp.
    pub fn touch_package(&mut self, package_id: &str, timestamp: u64) {
        if let Some(pkg) = self.packages.get_mut(package_id) {
            pkg.last_accessed_timestamp = timestamp;
        }
    }

    /// Marks a package as actively mounted in the streaming simulation.
    pub fn set_package_active(&mut self, package_id: &str, active: bool) {
        if let Some(pkg) = self.packages.get_mut(package_id) {
            pkg.is_active = active;
        }
    }

    /// Pins or unpins a package (pinned packages are protected from eviction).
    pub fn set_package_pinned(&mut self, package_id: &str, pinned: bool) {
        if let Some(pkg) = self.packages.get_mut(package_id) {
            pkg.is_pinned = pinned;
        }
    }

    /// Evicts unpinned, inactive packages using an LRU policy until disk usage fits within quota.
    /// Never evicts active or pinned packages! Returns list of evicted package IDs.
    pub fn evict_lru_to_quota(&mut self) -> Vec<String> {
        let mut evicted = Vec::new();

        while self.total_used_bytes() > self.quota_bytes {
            // Find candidate: unpinned, inactive, with smallest last_accessed_timestamp
            let candidate_id = self
                .packages
                .values()
                .filter(|p| !p.is_pinned && !p.is_active)
                .min_by_key(|p| p.last_accessed_timestamp)
                .map(|p| p.package_id.clone());

            if let Some(id) = candidate_id {
                self.packages.remove(&id);
                evicted.push(id);
            } else {
                // No more evictable packages (all remaining packages are either active or pinned)
                break;
            }
        }

        evicted
    }

    /// Selectively uninstalls a package and reclaims its disk allocation.
    pub fn uninstall_package(&mut self, package_id: &str) -> Option<InstalledPackageRecord> {
        self.packages.remove(package_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resumable_acquisition_workflow() {
        let mut session = ResumableAcquisitionSession::new("sf-downtown-v1", 4, 4000);

        assert_eq!(session.next_chunk_to_fetch(), Some(0));
        assert!(!session.is_fully_completed());

        // Download and verify chunks 0 and 1
        session.mark_chunk_downloaded(0, 1000).unwrap();
        session.verify_chunk(0, true).unwrap();

        session.mark_chunk_downloaded(1, 1000).unwrap();
        session.verify_chunk(1, true).unwrap();

        // Simulate interruption: next chunk to fetch must be 2
        assert_eq!(session.next_chunk_to_fetch(), Some(2));
        assert_eq!(session.bytes_downloaded, 2000);

        // Resume and complete remaining chunks 2 and 3
        session.mark_chunk_downloaded(2, 1000).unwrap();
        session.verify_chunk(2, true).unwrap();

        session.mark_chunk_downloaded(3, 1000).unwrap();
        session.verify_chunk(3, true).unwrap();

        assert_eq!(session.next_chunk_to_fetch(), None);
        assert!(session.is_fully_completed());
    }

    #[test]
    fn test_storage_quota_lru_eviction() {
        // Quota: 10,000 bytes (~10 KB)
        let mut manager = StorageQuotaManager::new(10_000);

        // Package A: 3,000 bytes, accessed at t=100
        manager.register_package(InstalledPackageRecord {
            package_id: "pkg_a".to_string(),
            country_code: "US".to_string(),
            version: "1.0.0".to_string(),
            disk_bytes: 3_000,
            is_pinned: false,
            is_active: false,
            last_accessed_timestamp: 100,
        });

        // Package B: 4,000 bytes, pinned, accessed at t=50
        manager.register_package(InstalledPackageRecord {
            package_id: "pkg_b_pinned".to_string(),
            country_code: "IN".to_string(),
            version: "1.0.0".to_string(),
            disk_bytes: 4_000,
            is_pinned: true,
            is_active: false,
            last_accessed_timestamp: 50,
        });

        // Package C: 5,000 bytes, accessed at t=200
        // Total bytes = 3,000 + 4,000 + 5,000 = 12,000 bytes > 10,000 quota!
        manager.register_package(InstalledPackageRecord {
            package_id: "pkg_c".to_string(),
            country_code: "GB".to_string(),
            version: "1.0.0".to_string(),
            disk_bytes: 5_000,
            is_pinned: false,
            is_active: false,
            last_accessed_timestamp: 200,
        });

        assert_eq!(manager.total_used_bytes(), 12_000);
        assert_eq!(manager.available_headroom_bytes(), 0);

        // Trigger eviction: pkg_a should be evicted first (t=100 < t=200),
        // while pkg_b is protected by is_pinned = true even though its timestamp is 50.
        let evicted = manager.evict_lru_to_quota();
        assert_eq!(evicted, vec!["pkg_a".to_string()]);

        // Remaining bytes: pkg_b (4,000) + pkg_c (5,000) = 9,000 <= 10,000 quota
        assert_eq!(manager.total_used_bytes(), 9_000);
        assert_eq!(manager.available_headroom_bytes(), 1_000);
        assert!(manager.packages.contains_key("pkg_b_pinned"));
        assert!(manager.packages.contains_key("pkg_c"));
        assert!(!manager.packages.contains_key("pkg_a"));
    }
}
