impl TxRpcImpl {
    async fn upsert_tx_record(
        &self,
        wallet_id: &PersistWalletId,
        tx_id: &PersistTxId,
        status: TxStatus,
        amount: u64,
        fee: u64,
    ) {
        self.upsert_tx_record_at(wallet_id, tx_id, status, amount, fee, self.now_ms())
            .await;
    }

    async fn upsert_tx_record_at(
        &self,
        wallet_id: &PersistWalletId,
        tx_id: &PersistTxId,
        status: TxStatus,
        amount: u64,
        fee: u64,
        timestamp_ms: u64,
    ) {
        tx_runtime_state::upsert_tx_record(
            &self.pending_txs,
            wallet_id,
            tx_id,
            status,
            amount,
            fee,
            timestamp_ms,
        )
        .await;
    }

    async fn cache_tx_bytes(&self, tx_id: &PersistTxId, tx_bytes: Vec<u8>) {
        tx_runtime_state::upsert_tx_bytes(&self.pending_tx_bytes, tx_id, tx_bytes).await;
    }

    pub(crate) async fn store_confirmation_evidence(&self, evidence: TxConfirmationEvidence) {
        tx_runtime_state::upsert_confirmation_evidence(&self.confirmation_evidence, evidence).await;
    }

    pub(crate) async fn load_confirmation_evidence(
        &self,
        tx_id: &PersistTxId,
    ) -> Option<TxConfirmationEvidence> {
        tx_runtime_state::find_confirmation_evidence(&self.confirmation_evidence, tx_id).await
    }

    async fn load_tx_history_rows(
        &self,
        wallet_id: &PersistWalletId,
    ) -> RpcResult<Vec<crate::backup::WalletTxHistoryJsonlEntry>> {
        if let Some(store) = &self.tx_store {
            let store = store.read().await;
            return store.list_history_rows().map_err(|error| {
                ErrorObjectOwned::owned(-32603, format!("tx journal load failed: {error}"), None::<()>)
            });
        }

        self.wallet_tx_store(wallet_id)
            .list_history_rows()
            .map_err(|error| {
                ErrorObjectOwned::owned(-32603, format!("tx journal load failed: {error}"), None::<()>)
            })
    }

    async fn load_tx_latest_kind(
        &self,
        wallet_id: &PersistWalletId,
        tx_hash: &str,
    ) -> RpcResult<Option<crate::backup::WalletTxHistoryEntryKind>> {
        let rows = self.load_tx_history_rows(wallet_id).await?;
        Ok(tx_runtime_state::latest_tx_history_kind(&rows, tx_hash))
    }

    async fn project_tx_info(
        &self,
        wallet_id: &PersistWalletId,
        record: crate::persistence::TxRecord,
    ) -> RpcResult<PersistTxInfo> {
        let latest_kind = self.load_tx_latest_kind(wallet_id, &record.tx_hash).await?;
        Ok(tx_runtime_state::tx_record_to_tx_info(
            wallet_id.clone(),
            record,
            latest_kind,
        ))
    }

    async fn persist_pending_tx(
        &self,
        wallet_id: &PersistWalletId,
        tx_id: &PersistTxId,
        tx_bytes: Vec<u8>,
        amount: u64,
        fee: u64,
        imported: bool,
        timestamp_ms: u64,
    ) -> RpcResult<()> {
        if let Some(store) = &self.tx_store {
            let mut store = store.write().await;
            let existing = store.get(&tx_id.0);
            match existing {
                Ok(existing) => {
                    if imported && !existing.imported {
                        let mut promoted = existing;
                        promoted.imported = true;
                        store.record_imported(promoted).map_err(|error| {
                            ErrorObjectOwned::owned(
                                -32603,
                                format!("tx journal persist failed: {error}"),
                                None::<()>,
                            )
                        })?;
                    }
                }
                Err(crate::persistence::TxStorageError::NotFound(_)) => {
                    let record = crate::persistence::TxRecord {
                        tx_hash: tx_id.0.clone(),
                        tx_bytes: tx_bytes.clone(),
                        imported,
                        status: crate::persistence::TxStatus::Pending,
                        timestamp_ms,
                        block_height: None,
                        confirmation_evidence: None,
                    };

                    if imported {
                        store.record_imported(record)
                    } else {
                        store.put(record)
                    }
                    .map_err(|error| {
                        ErrorObjectOwned::owned(
                            -32603,
                            format!("tx journal persist failed: {error}"),
                            None::<()>,
                        )
                    })?;
                }
                Err(error) => {
                    return Err(ErrorObjectOwned::owned(
                        -32603,
                        format!("tx journal load failed: {error}"),
                        None::<()>,
                    ));
                }
            }
        } else {
            let mut store = self.wallet_tx_store(wallet_id);
            let existing = store.get(&tx_id.0);
            match existing {
                Ok(existing) => {
                    if imported && !existing.imported {
                        let mut promoted = existing;
                        promoted.imported = true;
                        store.record_imported(promoted).map_err(|error| {
                            ErrorObjectOwned::owned(
                                -32603,
                                format!("tx journal persist failed: {error}"),
                                None::<()>,
                            )
                        })?;
                    }
                }
                Err(crate::persistence::TxStorageError::NotFound(_)) => {
                    let record = crate::persistence::TxRecord {
                        tx_hash: tx_id.0.clone(),
                        tx_bytes: tx_bytes.clone(),
                        imported,
                        status: crate::persistence::TxStatus::Pending,
                        timestamp_ms,
                        block_height: None,
                        confirmation_evidence: None,
                    };

                    if imported {
                        store.record_imported(record)
                    } else {
                        store.put(record)
                    }
                    .map_err(|error| {
                        ErrorObjectOwned::owned(
                            -32603,
                            format!("tx journal persist failed: {error}"),
                            None::<()>,
                        )
                    })?;
                }
                Err(error) => {
                    return Err(ErrorObjectOwned::owned(
                        -32603,
                        format!("tx journal load failed: {error}"),
                        None::<()>,
                    ));
                }
            }
        }

        self.cache_tx_bytes(tx_id, tx_bytes.clone()).await;
        self.upsert_tx_record_at(wallet_id, tx_id, TxStatus::Pending, amount, fee, timestamp_ms)
            .await;
        Ok(())
    }

    async fn journal_admission(
        &self,
        wallet_id: &PersistWalletId,
        tx_id: &PersistTxId,
    ) -> RpcResult<()> {
        if let Some(store) = &self.tx_store {
            let mut store = store.write().await;
            store.record_submitted(&tx_id.0).map_err(|error| {
                ErrorObjectOwned::owned(
                    -32603,
                    format!("tx submission journal failed: {error}"),
                    None::<()>,
                )
            })?;
            store.record_admitted(&tx_id.0).map_err(|error| {
                ErrorObjectOwned::owned(
                    -32603,
                    format!("tx admission journal failed: {error}"),
                    None::<()>,
                )
            })?;
        } else {
            let mut store = self.wallet_tx_store(wallet_id);
            store.record_submitted(&tx_id.0).map_err(|error| {
                ErrorObjectOwned::owned(
                    -32603,
                    format!("tx submission journal failed: {error}"),
                    None::<()>,
                )
            })?;
            store.record_admitted(&tx_id.0).map_err(|error| {
                ErrorObjectOwned::owned(
                    -32603,
                    format!("tx admission journal failed: {error}"),
                    None::<()>,
                )
            })?;
        }
        Ok(())
    }

    async fn confirm_stored_tx_evidence(
        &self,
        wallet_id: &PersistWalletId,
        tx_id: &PersistTxId,
        evidence: TxConfirmationEvidence,
    ) -> RpcResult<()> {
        if let Some(store) = &self.tx_store {
            let mut store = store.write().await;
            store
                .record_confirmation_evidence(&tx_id.0, evidence)
                .map_err(|error| {
                    ErrorObjectOwned::owned(
                        -32603,
                        format!("tx confirmation journal failed: {error}"),
                        None::<()>,
                    )
                })?;
        } else {
            let mut store = self.wallet_tx_store(wallet_id);
            store
                .record_confirmation_evidence(&tx_id.0, evidence)
                .map_err(|error| {
                    ErrorObjectOwned::owned(
                        -32603,
                        format!("tx confirmation journal failed: {error}"),
                        None::<()>,
                    )
                })?;
        }
        Ok(())
    }

    async fn rollback_persisted_tx(
        &self,
        wallet_id: &PersistWalletId,
        tx_id: &PersistTxId,
        previous: Option<crate::persistence::TxRecord>,
    ) -> RpcResult<()> {
        if let Some(store) = &self.tx_store {
            let mut store = store.write().await;
            restore_tx_journal_state(&mut **store, tx_id, previous).map_err(|error| {
                ErrorObjectOwned::owned(
                    -32603,
                    format!("tx journal rollback failed: {error}"),
                    None::<()>,
                )
            })?;
        } else {
            let mut store = self.wallet_tx_store(wallet_id);
            restore_tx_journal_state(&mut store, tx_id, previous).map_err(|error| {
                ErrorObjectOwned::owned(
                    -32603,
                    format!("tx journal rollback failed: {error}"),
                    None::<()>,
                )
            })?;
        }

        Ok(())
    }

    async fn record_exported_tx(
        &self,
        wallet_id: &PersistWalletId,
        tx_id: &PersistTxId,
    ) -> RpcResult<()> {
        if let Some(store) = &self.tx_store {
            let mut store = store.write().await;
            store.record_exported(&tx_id.0).map_err(|error| {
                ErrorObjectOwned::owned(-32603, format!("tx export journal failed: {error}"), None::<()>)
            })?;
        } else {
            let mut store = self.wallet_tx_store(wallet_id);
            store.record_exported(&tx_id.0).map_err(|error| {
                ErrorObjectOwned::owned(-32603, format!("tx export journal failed: {error}"), None::<()>)
            })?;
        }

        Ok(())
    }

    async fn record_conflicted_tx(
        &self,
        wallet_id: &PersistWalletId,
        tx_id: &PersistTxId,
    ) -> RpcResult<()> {
        if let Some(store) = &self.tx_store {
            let mut store = store.write().await;
            store.record_conflicted(&tx_id.0).map_err(|error| {
                ErrorObjectOwned::owned(
                    -32603,
                    format!("tx conflict journal failed: {error}"),
                    None::<()>,
                )
            })?;
        } else {
            let mut store = self.wallet_tx_store(wallet_id);
            store.record_conflicted(&tx_id.0).map_err(|error| {
                ErrorObjectOwned::owned(
                    -32603,
                    format!("tx conflict journal failed: {error}"),
                    None::<()>,
                )
            })?;
        }

        Ok(())
    }

    async fn record_already_spent_tx(
        &self,
        wallet_id: &PersistWalletId,
        tx_id: &PersistTxId,
    ) -> RpcResult<()> {
        if let Some(store) = &self.tx_store {
            let mut store = store.write().await;
            store.record_already_spent(&tx_id.0).map_err(|error| {
                ErrorObjectOwned::owned(
                    -32603,
                    format!("tx already-spent journal failed: {error}"),
                    None::<()>,
                )
            })?;
        } else {
            let mut store = self.wallet_tx_store(wallet_id);
            store.record_already_spent(&tx_id.0).map_err(|error| {
                ErrorObjectOwned::owned(
                    -32603,
                    format!("tx already-spent journal failed: {error}"),
                    None::<()>,
                )
            })?;
        }

        Ok(())
    }

    async fn has_already_spent_input_conflict(
        &self,
        wallet_id: &PersistWalletId,
        tx_id: &PersistTxId,
        input_ids: &[[u8; 32]],
    ) -> RpcResult<bool> {
        for asset_id in input_ids {
            let payload = self
                .service
                .lookup_owned_asset_payload(wallet_id, *asset_id)
                .await
                .map_err(crate::rpc::error_mapping::map_wallet_error_to_rpc)?;
            let Some(payload) = payload else {
                continue;
            };

            if matches!(
                payload.status,
                crate::db::redb_store::OwnedAssetStatus::PendingSpend
                    | crate::db::redb_store::OwnedAssetStatus::Spent
            ) && payload.spend_ref.as_ref() != Some(tx_id)
            {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

fn restore_tx_journal_state<S>(
    store: &mut S,
    tx_id: &PersistTxId,
    previous: Option<crate::persistence::TxRecord>,
) -> crate::persistence::TxStorageResult<()>
where
    S: crate::persistence::TxStorage + ?Sized,
{
    if let Some(record) = previous {
        let restore_kind = store
            .list_history_rows()?
            .into_iter()
            .rev()
            .find(|row| row.tx_hash == tx_id.0 && row.record == record)
            .map(|row| row.entry_kind)
            .unwrap_or_else(|| fallback_restore_kind(&record));
        store.restore_snapshot(record, restore_kind)?;
        return Ok(());
    }

    match store.get(&tx_id.0) {
        Ok(_) => store.delete(&tx_id.0)?,
        Err(crate::persistence::TxStorageError::NotFound(_)) => {}
        Err(error) => return Err(error),
    }

    Ok(())
}

fn fallback_restore_kind(
    record: &crate::persistence::TxRecord,
) -> crate::backup::WalletTxHistoryEntryKind {
    if record.imported {
        return crate::backup::WalletTxHistoryEntryKind::Imported;
    }

    match record.status {
        crate::persistence::TxStatus::Pending => crate::backup::WalletTxHistoryEntryKind::Created,
        crate::persistence::TxStatus::Confirmed => {
            crate::backup::WalletTxHistoryEntryKind::Confirmed
        }
        crate::persistence::TxStatus::Failed => crate::backup::WalletTxHistoryEntryKind::Failed,
        crate::persistence::TxStatus::Cancelled => {
            crate::backup::WalletTxHistoryEntryKind::Cancelled
        }
    }
}

pub(super) fn run_with_retry<F>(max_retries: u32, mut op: F) -> (u32, Result<(), BroadcastError>)
where
    F: FnMut() -> Result<(), BroadcastError>,
{
    tx_rpc_broadcast::run_with_retry(max_retries, &mut op)
}
