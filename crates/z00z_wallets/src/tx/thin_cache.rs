use std::collections::BTreeMap;

use super::{
    thin_builder::{build_thick_transport_from_entry, build_thin_transport_from_entry},
    ThinFallbackReason, ThinIndexEntry, ThinIndexError, ThinIndexStore, ThinSnapshot,
    ThinSnapshotPin, ThinTransportPayload,
};

/// Wallet-local cache of authenticated thin snapshot pins.
#[derive(Debug, Default, Clone)]
pub struct ThinSnapshotCache {
    pins: BTreeMap<String, ThinSnapshotPin>,
}

impl ThinSnapshotCache {
    /// Create an empty thin snapshot cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn cache_key(pin: &ThinSnapshotPin) -> String {
        format!(
            "{}:{}:{}:{}",
            pin.chain_id,
            pin.compatibility_generation,
            pin.prev_root_hex,
            pin.checkpoint_id_hex.as_deref().unwrap_or("-")
        )
    }

    fn remove_digest(&mut self, snapshot_digest_hex: &str) {
        self.pins
            .retain(|_, pin| pin.snapshot_digest_hex != snapshot_digest_hex);
    }

    fn remove_digests<I>(&mut self, snapshot_digests: I)
    where
        I: IntoIterator<Item = String>,
    {
        for digest in snapshot_digests {
            self.remove_digest(&digest);
        }
    }

    /// Remember one authenticated snapshot pin in the wallet-local cache.
    pub fn remember_pin(&mut self, pin: ThinSnapshotPin) {
        self.pins.insert(Self::cache_key(&pin), pin);
    }

    /// Clear every cached snapshot pin.
    pub fn clear(&mut self) {
        self.pins.clear();
    }

    /// Pin one snapshot from the helper store and cache it for later transport reuse.
    pub fn pin_snapshot(
        &mut self,
        store: &ThinIndexStore,
        snapshot_digest_hex: &str,
        now_ms: u64,
    ) -> Result<ThinSnapshotPin, ThinIndexError> {
        let pin = store.pin_snapshot(snapshot_digest_hex, now_ms)?;
        self.remember_pin(pin.clone());
        Ok(pin)
    }

    /// Publish a refreshed snapshot, pin it immediately, and cache the new pin.
    pub fn refresh_snapshot(
        &mut self,
        store: &mut ThinIndexStore,
        snapshot: ThinSnapshot,
        now_ms: u64,
    ) -> Result<ThinSnapshotPin, ThinIndexError> {
        let pin = store.refresh_snapshot(snapshot, now_ms)?;
        self.remember_pin(pin.clone());
        Ok(pin)
    }

    /// Build one canonical transport payload from cached helper state when safe.
    ///
    /// Thin helper references are expanded before runtime admission. If cached
    /// helper state is missing, stale, or inconsistent, the wallet defaults to
    /// the canonical thick package payload.
    pub fn build_transport(
        &mut self,
        store: &ThinIndexStore,
        tx_bytes: &[u8],
        now_ms: u64,
    ) -> Result<ThinTransportPayload, ThinIndexError> {
        let entry = ThinIndexEntry::from_tx_bytes(tx_bytes.to_vec())?;

        let mut candidate_pins = Vec::new();
        let mut last_error = None;
        let mut stale_digests = Vec::new();

        for pin in self.pins.values().filter(|pin| {
            pin.chain_id == entry.chain_id && pin.prev_root_hex == entry.prev_root_hex
        }) {
            if now_ms > pin.expires_at_ms {
                stale_digests.push(pin.snapshot_digest_hex.clone());
                last_error = Some(ThinIndexError::SnapshotExpired {
                    expires_at_ms: pin.expires_at_ms,
                    now_ms,
                });
                continue;
            }
            candidate_pins.push(pin.clone());
        }

        for digest in stale_digests {
            self.remove_digest(&digest);
        }

        candidate_pins.sort_by_key(|pin| (pin.compatibility_generation, pin.expires_at_ms));
        candidate_pins.reverse();

        if candidate_pins.is_empty() {
            let fallback_reason = last_error
                .map(ThinFallbackReason::CacheUnavailable)
                .or(Some(ThinFallbackReason::NoPinnedSnapshot));
            return build_thick_transport_from_entry(&entry, fallback_reason);
        }

        let Some(pin) = candidate_pins.first().cloned() else {
            return build_thick_transport_from_entry(
                &entry,
                Some(
                    last_error
                        .map(ThinFallbackReason::CacheUnavailable)
                        .unwrap_or(ThinFallbackReason::NoPinnedSnapshot),
                ),
            );
        };

        match build_thin_transport_from_entry(store, &pin, &entry, now_ms) {
            Ok(payload) => return Ok(payload),
            Err(error) => {
                let conflicting_digests = candidate_pins
                    .into_iter()
                    .map(|candidate| candidate.snapshot_digest_hex)
                    .collect::<Vec<_>>();
                self.remove_digests(conflicting_digests);
                last_error = Some(error);
            }
        }

        build_thick_transport_from_entry(
            &entry,
            Some(
                last_error
                    .map(ThinFallbackReason::CacheUnavailable)
                    .unwrap_or(ThinFallbackReason::NoPinnedSnapshot),
            ),
        )
    }
}
