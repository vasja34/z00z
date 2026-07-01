use std::collections::BTreeMap;

use crate::tx::TxPackage;

use super::thin_types::{
    ThinIndexEntry, ThinIndexError, ThinSnapshot, ThinSnapshotPin, ThinWalletTxPackage,
};

/// In-memory signed-index helper store for thin wallet transport.
#[derive(Debug, Default, Clone)]
pub struct ThinIndexStore {
    snapshots: BTreeMap<String, ThinSnapshot>,
    context_digests: BTreeMap<String, String>,
}

impl ThinIndexStore {
    /// Create an empty thin index store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Publish one signed helper snapshot.
    pub fn publish_snapshot(&mut self, snapshot: ThinSnapshot) -> Result<(), ThinIndexError> {
        snapshot.check_shape()?;
        let context_key = snapshot.context_key();
        if let Some(existing_digest) = self.context_digests.get(&context_key) {
            if existing_digest != &snapshot.snapshot_digest_hex {
                return Err(ThinIndexError::SnapshotConflict {
                    context_key,
                    existing_digest: existing_digest.clone(),
                    new_digest: snapshot.snapshot_digest_hex.clone(),
                });
            }
        }
        self.context_digests
            .insert(context_key, snapshot.snapshot_digest_hex.clone());
        self.snapshots
            .insert(snapshot.snapshot_digest_hex.clone(), snapshot);
        Ok(())
    }

    /// Fetch one published snapshot by digest.
    pub fn snapshot(&self, snapshot_digest_hex: &str) -> Result<ThinSnapshot, ThinIndexError> {
        self.snapshots
            .get(snapshot_digest_hex)
            .cloned()
            .ok_or_else(|| ThinIndexError::SnapshotMissing(snapshot_digest_hex.to_string()))
    }

    /// Pin one signed snapshot for wallet-side thin requests.
    pub fn pin_snapshot(
        &self,
        snapshot_digest_hex: &str,
        now_ms: u64,
    ) -> Result<ThinSnapshotPin, ThinIndexError> {
        let snapshot = self
            .snapshots
            .get(snapshot_digest_hex)
            .ok_or_else(|| ThinIndexError::SnapshotMissing(snapshot_digest_hex.to_string()))?;
        ThinSnapshotPin::new(snapshot, now_ms)
    }

    /// Publish a new snapshot and pin it immediately.
    pub fn refresh_snapshot(
        &mut self,
        snapshot: ThinSnapshot,
        now_ms: u64,
    ) -> Result<ThinSnapshotPin, ThinIndexError> {
        let digest = snapshot.snapshot_digest_hex.clone();
        self.publish_snapshot(snapshot)?;
        self.pin_snapshot(&digest, now_ms)
    }

    /// Lookup the exact helper entry that matches one canonical tx package.
    pub fn matching_entry(
        &self,
        snapshot_digest_hex: &str,
        tx_bytes: &[u8],
    ) -> Result<ThinIndexEntry, ThinIndexError> {
        let candidate = ThinIndexEntry::from_tx_bytes(tx_bytes.to_vec())?;
        let snapshot = self.snapshot(snapshot_digest_hex)?;
        snapshot.check_shape()?;
        snapshot.entry(&candidate.entry_id_hex).cloned()
    }

    /// Resolve one thin wrapper back into canonical thick package bytes.
    pub fn resolve_package(
        &self,
        thin: &ThinWalletTxPackage,
        now_ms: u64,
    ) -> Result<(Vec<u8>, TxPackage), ThinIndexError> {
        thin.verify_metadata()?;
        let snapshot = self
            .snapshots
            .get(&thin.snapshot_digest_hex)
            .ok_or_else(|| ThinIndexError::SnapshotMissing(thin.snapshot_digest_hex.clone()))?;
        snapshot.verify_at(now_ms)?;
        if snapshot.context.chain_id != thin.chain_id {
            return Err(ThinIndexError::PackageChainMismatch {
                expected: snapshot.context.chain_id.clone(),
                actual: thin.chain_id.clone(),
            });
        }
        if snapshot.context.compatibility_generation != thin.compatibility_generation {
            return Err(ThinIndexError::SnapshotGenerationMismatch {
                expected: snapshot.context.compatibility_generation,
                actual: thin.compatibility_generation,
            });
        }
        if snapshot.context.prev_root_hex != thin.prev_root_hex {
            return Err(ThinIndexError::PackageRootMismatch {
                expected: snapshot.context.prev_root_hex.clone(),
                actual: thin.prev_root_hex.clone(),
            });
        }
        if snapshot.context.checkpoint_id_hex != thin.checkpoint_id_hex {
            return Err(ThinIndexError::SnapshotContextMismatch {
                field: "checkpoint_id_hex",
                expected: snapshot
                    .context
                    .checkpoint_id_hex
                    .clone()
                    .unwrap_or_else(|| "-".to_string()),
                actual: thin
                    .checkpoint_id_hex
                    .clone()
                    .unwrap_or_else(|| "-".to_string()),
            });
        }
        let entry = snapshot.entry(&thin.snapshot_entry_id_hex)?;
        let (tx_bytes, package) = entry.verify_and_load()?;
        if entry.tx_hash_hex != thin.tx_hash_hex {
            return Err(ThinIndexError::PackageDigestMismatch {
                expected: entry.tx_hash_hex.clone(),
                actual: thin.tx_hash_hex.clone(),
            });
        }
        if entry.package_kind != thin.package_kind {
            return Err(ThinIndexError::PackageKindMismatch {
                expected: entry.package_kind.clone(),
                actual: thin.package_kind.clone(),
            });
        }
        if entry.package_type != thin.package_type {
            return Err(ThinIndexError::PackageTypeMismatch {
                expected: entry.package_type.clone(),
                actual: thin.package_type.clone(),
            });
        }
        if entry.input_refs != thin.input_refs {
            return Err(ThinIndexError::InputRefMismatch);
        }
        Ok((tx_bytes, package))
    }
}
