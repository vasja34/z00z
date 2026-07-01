use z00z_utils::codec::{Codec, JsonCodec};

use super::{ThinIndexEntry, ThinIndexError, ThinIndexStore, ThinSnapshotPin, ThinWalletTxPackage};

/// Selected wallet transport mode for one canonical tx package.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThinTransportMode {
    /// The canonical thick package payload is used.
    Thick,
    /// A thin helper wrapper is used.
    Thin,
}

/// Reason why the wallet defaulted back to thick transport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThinFallbackReason {
    /// No authenticated snapshot is pinned for this tx context.
    NoPinnedSnapshot,
    /// Cached helper state was present but not safe to reuse.
    CacheUnavailable(ThinIndexError),
}

/// One serialized tx transport payload built from canonical package meaning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThinTransportPayload {
    /// Selected wallet transport mode.
    pub mode: ThinTransportMode,
    /// Canonical tx digest preserved across thick and thin transports.
    pub tx_digest_hex: String,
    /// Serialized JSON payload for the chosen transport mode.
    pub payload_json: String,
    /// Thick fallback reason when thin transport was not safe to use.
    pub fallback_reason: Option<ThinFallbackReason>,
}

impl ThinTransportPayload {
    /// Return true when the selected payload is the thin helper wrapper.
    #[must_use]
    pub fn is_thin(&self) -> bool {
        matches!(self.mode, ThinTransportMode::Thin)
    }
}

fn payload_json_from_bytes(bytes: Vec<u8>, label: &str) -> Result<String, ThinIndexError> {
    String::from_utf8(bytes).map_err(|error| {
        ThinIndexError::PackageVerificationFailed(format!(
            "{label} UTF-8 serialization failed: {error}"
        ))
    })
}

pub(crate) fn build_thick_transport_from_entry(
    entry: &ThinIndexEntry,
    fallback_reason: Option<ThinFallbackReason>,
) -> Result<ThinTransportPayload, ThinIndexError> {
    Ok(ThinTransportPayload {
        mode: ThinTransportMode::Thick,
        tx_digest_hex: entry.tx_hash_hex.clone(),
        payload_json: payload_json_from_bytes(entry.tx_bytes.clone(), "thick tx package")?,
        fallback_reason,
    })
}

pub(crate) fn build_thin_transport_from_entry(
    store: &ThinIndexStore,
    pin: &ThinSnapshotPin,
    entry: &ThinIndexEntry,
    now_ms: u64,
) -> Result<ThinTransportPayload, ThinIndexError> {
    let fresh_pin = store.pin_snapshot(&pin.snapshot_digest_hex, now_ms)?;
    if fresh_pin.chain_id != pin.chain_id {
        return Err(ThinIndexError::PackageChainMismatch {
            expected: pin.chain_id.clone(),
            actual: fresh_pin.chain_id,
        });
    }
    if fresh_pin.compatibility_generation != pin.compatibility_generation {
        return Err(ThinIndexError::SnapshotGenerationMismatch {
            expected: pin.compatibility_generation,
            actual: fresh_pin.compatibility_generation,
        });
    }
    if fresh_pin.prev_root_hex != pin.prev_root_hex {
        return Err(ThinIndexError::PackageRootMismatch {
            expected: pin.prev_root_hex.clone(),
            actual: fresh_pin.prev_root_hex,
        });
    }
    if fresh_pin.checkpoint_id_hex != pin.checkpoint_id_hex {
        return Err(ThinIndexError::SnapshotContextMismatch {
            field: "checkpoint_id_hex",
            expected: pin
                .checkpoint_id_hex
                .clone()
                .unwrap_or_else(|| "-".to_string()),
            actual: fresh_pin
                .checkpoint_id_hex
                .clone()
                .unwrap_or_else(|| "-".to_string()),
        });
    }

    let snapshot_entry = store.matching_entry(&fresh_pin.snapshot_digest_hex, &entry.tx_bytes)?;
    let thin = ThinWalletTxPackage::new(&fresh_pin, &snapshot_entry)?;
    let payload_json = payload_json_from_bytes(
        JsonCodec.serialize(&thin).map_err(|error| {
            ThinIndexError::PackageVerificationFailed(format!(
                "thin tx package serialization failed: {error}"
            ))
        })?,
        "thin tx package",
    )?;

    Ok(ThinTransportPayload {
        mode: ThinTransportMode::Thin,
        tx_digest_hex: thin.tx_hash_hex.clone(),
        payload_json,
        fallback_reason: None,
    })
}

/// Build the canonical thick transport payload for one verified tx package.
pub fn build_thick_transport_payload(
    tx_bytes: &[u8],
    fallback_reason: Option<ThinFallbackReason>,
) -> Result<ThinTransportPayload, ThinIndexError> {
    let entry = ThinIndexEntry::from_tx_bytes(tx_bytes.to_vec())?;
    build_thick_transport_from_entry(&entry, fallback_reason)
}

/// Build the canonical thin transport payload from one pinned helper snapshot.
pub fn build_thin_transport_payload(
    store: &ThinIndexStore,
    pin: &ThinSnapshotPin,
    tx_bytes: &[u8],
    now_ms: u64,
) -> Result<ThinTransportPayload, ThinIndexError> {
    let entry = ThinIndexEntry::from_tx_bytes(tx_bytes.to_vec())?;
    build_thin_transport_from_entry(store, pin, &entry, now_ms)
}
