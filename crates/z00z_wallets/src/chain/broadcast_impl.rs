//! Broadcast implementation over the canonical wallet chain and tx-store seams.

use std::sync::Mutex;

use z00z_utils::time::TimeProvider;

use super::{
    Broadcast, BroadcastError, BroadcastResult, BroadcastResultType, ChainClient, ChainClientError,
    ChainTxStatus,
};
use crate::persistence::{TxRecord, TxStatus, TxStorage, TxStorageError};
use crate::tx::{build_tx_package_digest, TxPackage};
use z00z_utils::codec::{Codec, JsonCodec};

const DEFAULT_POLL_INTERVAL_MS: u64 = 250;

/// Default `Broadcast` implementation.
///
/// This is the single wallet-owned broadcast lifecycle path for local
/// simulation:
/// - `ChainClient` owns node-facing submission and status checks
/// - `TxStorage` owns durable lifecycle persistence
/// - only the remote node transport itself may remain adapter-only
pub struct BroadcastImpl<C, S, T> {
    chain_client: C,
    tx_store: Mutex<S>,
    time_provider: T,
    poll_interval_ms: u64,
}

impl<C, S, T> BroadcastImpl<C, S, T>
where
    C: ChainClient,
    S: TxStorage,
    T: TimeProvider,
{
    /// Create a broadcast handler with the default polling interval.
    pub fn new(chain_client: C, tx_store: S, time_provider: T) -> Self {
        Self::with_poll_interval_ms(
            chain_client,
            tx_store,
            time_provider,
            DEFAULT_POLL_INTERVAL_MS,
        )
    }

    /// Create a broadcast handler with an explicit polling interval.
    pub fn with_poll_interval_ms(
        chain_client: C,
        tx_store: S,
        time_provider: T,
        poll_interval_ms: u64,
    ) -> Self {
        Self {
            chain_client,
            tx_store: Mutex::new(tx_store),
            time_provider,
            poll_interval_ms: poll_interval_ms.max(1),
        }
    }

    fn map_submit_error(error: ChainClientError) -> BroadcastError {
        match error {
            ChainClientError::Connection(message) | ChainClientError::Network(message) => {
                BroadcastError::Network(message)
            }
            ChainClientError::Rejected(message) => BroadcastError::Rejected(message),
            other => BroadcastError::Failed(other.to_string()),
        }
    }

    fn map_status_error(error: ChainClientError) -> BroadcastError {
        match error {
            ChainClientError::Connection(message) | ChainClientError::Network(message) => {
                BroadcastError::Network(message)
            }
            ChainClientError::Rejected(message) => BroadcastError::Rejected(message),
            ChainClientError::TxNotFound(tx_hash) => {
                BroadcastError::Failed(format!("transaction not found in chain status: {tx_hash}"))
            }
            other => BroadcastError::Failed(other.to_string()),
        }
    }

    fn map_storage_error(error: TxStorageError) -> BroadcastError {
        BroadcastError::Failed(format!("tx-store error: {error}"))
    }

    fn wallet_tx_id(tx_hash: &str) -> String {
        if tx_hash.starts_with("tx_") {
            tx_hash.to_string()
        } else {
            format!("tx_{tx_hash}")
        }
    }

    fn canonical_tx_id_from_bytes(tx_bytes: &[u8]) -> Option<String> {
        let package = JsonCodec.deserialize::<TxPackage>(tx_bytes).ok()?;
        let digest = build_tx_package_digest(
            &package.kind,
            &package.package_type,
            package.version,
            package.chain_id,
            &package.chain_type,
            &package.chain_name,
            &package.tx,
        )
        .ok()?;
        if digest != package.tx_digest_hex {
            return None;
        }
        Some(Self::wallet_tx_id(&package.tx_digest_hex))
    }

    fn same_canonical_tx(tx_id: &str, existing: &[u8], incoming: &[u8]) -> bool {
        matches!(
            (
                Self::canonical_tx_id_from_bytes(existing),
                Self::canonical_tx_id_from_bytes(incoming),
            ),
            (Some(existing_id), Some(incoming_id))
                if existing_id == tx_id && incoming_id == tx_id
        )
    }

    fn with_store_mut<R>(
        &self,
        op: impl FnOnce(&mut S) -> BroadcastResultType<R>,
    ) -> BroadcastResultType<R> {
        let mut guard = self
            .tx_store
            .lock()
            .expect("broadcast tx store lock poisoned");
        op(&mut guard)
    }

    fn persist_submitted(&self, tx_hash: &str, tx_bytes: &[u8]) -> BroadcastResultType<()> {
        let tx_id = Self::wallet_tx_id(tx_hash);
        let timestamp_ms = self.time_provider.compat_unix_timestamp_millis();

        self.with_store_mut(|store| match store.get(&tx_id) {
            Ok(existing) => {
                if existing.tx_bytes != tx_bytes
                    && !Self::same_canonical_tx(&tx_id, &existing.tx_bytes, tx_bytes)
                {
                    return Err(BroadcastError::Failed(format!(
                        "stored tx bytes mismatch for {tx_id}"
                    )));
                }

                match existing.status {
                    TxStatus::Pending | TxStatus::Confirmed => Ok(()),
                    TxStatus::Failed | TxStatus::Cancelled => store
                        .update_status(&tx_id, TxStatus::Pending)
                        .map_err(Self::map_storage_error),
                }
            }
            Err(TxStorageError::NotFound(_)) => {
                store
                    .put(TxRecord {
                        tx_hash: tx_id.clone(),
                        tx_bytes: tx_bytes.to_vec(),
                        imported: false,
                        status: TxStatus::Pending,
                        timestamp_ms,
                        block_height: None,
                        confirmation_evidence: None,
                    })
                    .map_err(Self::map_storage_error)?;
                store
                    .record_submitted(&tx_id)
                    .map_err(Self::map_storage_error)
            }
            Err(error) => Err(Self::map_storage_error(error)),
        })
    }

    fn persist_failed(&self, tx_hash: &str) -> BroadcastResultType<()> {
        let tx_id = Self::wallet_tx_id(tx_hash);
        self.with_store_mut(|store| match store.get(&tx_id) {
            Ok(existing) => {
                if matches!(existing.status, TxStatus::Failed) {
                    return Ok(());
                }
                store.record_failed(&tx_id).map_err(Self::map_storage_error)
            }
            Err(error) => Err(Self::map_storage_error(error)),
        })
    }

    fn persist_confirmed(&self, tx_hash: &str) -> BroadcastResultType<u64> {
        let tx_id = Self::wallet_tx_id(tx_hash);
        let tip_height = self
            .chain_client
            .get_tip_height()
            .map_err(Self::map_status_error)?;

        self.with_store_mut(|store| match store.get(&tx_id) {
            Ok(existing) => {
                if matches!(existing.status, TxStatus::Confirmed) {
                    return Ok(existing.block_height.unwrap_or(tip_height));
                }

                store
                    .record_confirmed(&tx_id, tip_height)
                    .map_err(Self::map_storage_error)?;
                Ok(tip_height)
            }
            Err(error) => Err(Self::map_storage_error(error)),
        })
    }

    fn status_result(&self, tx_hash: &str, status: ChainTxStatus) -> BroadcastResultType<bool> {
        match status {
            ChainTxStatus::Pending => Ok(false),
            ChainTxStatus::Confirmed => {
                self.persist_confirmed(tx_hash)?;
                Ok(true)
            }
            ChainTxStatus::Failed => {
                self.persist_failed(tx_hash)?;
                Err(BroadcastError::Rejected(tx_hash.to_string()))
            }
            ChainTxStatus::Replaced => {
                self.persist_failed(tx_hash)?;
                Err(BroadcastError::Replaced(tx_hash.to_string()))
            }
            ChainTxStatus::Reorged => {
                self.persist_failed(tx_hash)?;
                Err(BroadcastError::Reorg(tx_hash.to_string()))
            }
        }
    }
}

impl<C, S, T> Broadcast for BroadcastImpl<C, S, T>
where
    C: ChainClient,
    S: TxStorage,
    T: TimeProvider,
{
    fn broadcast(&self, tx_bytes: &[u8]) -> BroadcastResultType<BroadcastResult> {
        let tx_hash = self
            .chain_client
            .submit_transaction(tx_bytes)
            .map_err(Self::map_submit_error)?;
        self.persist_submitted(&tx_hash, tx_bytes)?;

        Ok(BroadcastResult {
            tx_hash,
            submitted_at: self.time_provider.compat_unix_timestamp_millis(),
        })
    }

    fn broadcast_with_retry(
        &self,
        tx_bytes: &[u8],
        max_retries: u32,
    ) -> BroadcastResultType<BroadcastResult> {
        let max_attempts = max_retries.saturating_add(1);
        let mut attempts = 0u32;

        loop {
            attempts = attempts.saturating_add(1);
            match self.broadcast(tx_bytes) {
                Ok(result) => return Ok(result),
                Err(error)
                    if attempts < max_attempts
                        && matches!(
                            error,
                            BroadcastError::Network(_) | BroadcastError::Timeout
                        ) =>
                {
                    continue;
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn is_confirmed(&self, tx_hash: &str) -> BroadcastResultType<bool> {
        let status = self
            .chain_client
            .get_transaction_status(tx_hash)
            .map_err(Self::map_status_error)?;
        self.status_result(tx_hash, status)
    }

    fn wait_for_confirmation(&self, tx_hash: &str, timeout_ms: u64) -> BroadcastResultType<u64> {
        let mut elapsed_ms = 0u64;

        loop {
            match self
                .chain_client
                .get_transaction_status(tx_hash)
                .map_err(Self::map_status_error)?
            {
                ChainTxStatus::Pending => {
                    if elapsed_ms >= timeout_ms {
                        return Err(BroadcastError::Timeout);
                    }
                    elapsed_ms = elapsed_ms.saturating_add(self.poll_interval_ms);
                }
                ChainTxStatus::Confirmed => return self.persist_confirmed(tx_hash),
                ChainTxStatus::Failed => {
                    self.persist_failed(tx_hash)?;
                    return Err(BroadcastError::Rejected(tx_hash.to_string()));
                }
                ChainTxStatus::Replaced => {
                    self.persist_failed(tx_hash)?;
                    return Err(BroadcastError::Replaced(tx_hash.to_string()));
                }
                ChainTxStatus::Reorged => {
                    self.persist_failed(tx_hash)?;
                    return Err(BroadcastError::Reorg(tx_hash.to_string()));
                }
            }
        }
    }
}
