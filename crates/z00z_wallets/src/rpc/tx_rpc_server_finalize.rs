impl TxRpcImpl {
    async fn estimate_transaction_fee_impl(
        &self,
        session: SessionToken,
        recipient: String,
        amount: u64,
        asset_id: Option<String>,
    ) -> RpcResult<RuntimeEstimateFeeResponse> {
        self.verify_session(&session).await?;

        if recipient.trim().is_empty() {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "Invalid recipient: must not be empty".to_string(),
                None::<()>,
            ));
        }

        if amount == 0 {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "Invalid amount: must be > 0".to_string(),
                None::<()>,
            ));
        }

        let fee_per_byte: u64 = 1;
        let asset_len = asset_id.as_ref().map(|value| value.len()).unwrap_or(0) as u64;
        let recipient_len = recipient.len() as u64;
        let estimated_size_bytes = 200u64
            .saturating_add(recipient_len)
            .saturating_add(asset_len)
            .saturating_add(8);
        let estimated_fee = fee_per_byte.saturating_mul(estimated_size_bytes).max(1);

        Ok(RuntimeEstimateFeeResponse {
            estimated_fee,
            fee_per_byte,
        })
    }

    async fn export_transaction_impl(
        &self,
        session: SessionToken,
        tx_id: PersistTxId,
    ) -> RpcResult<RuntimeExportTxResponse> {
        let wallet_id = session.wallet_id.clone();
        self.verify_session(&session).await?;

        let stored = self.load_stored_tx_record(&wallet_id, &tx_id).await?;
        let info = self.project_tx_info(&wallet_id, stored.clone()).await?;
        let payload = self.portable_tx_package(
            &tx_id,
            stored.tx_bytes,
            tx_rpc_support::package_status_from_lifecycle(info.lifecycle),
        )?;

        let export_path = {
            let _guard = tx_export_write_lock()
                .lock()
                .unwrap_or_else(|err| err.into_inner());
            let export_dir =
                tx_runtime_state::prepare_tx_export_root(self.service.output_dir())
                    .map_err(|error| {
                        ErrorObjectOwned::owned(
                            -32603,
                            format!("Export failed: {error}"),
                            None::<()>,
                        )
                    })?;

            let hash = compute_wallet_file_id(&wallet_id.0);
            let wallet_id_hex = hex::encode(&hash[..8]);
            let export_path = export_dir.join(format!("tx_{wallet_id_hex}.json"));
            let codec = z00z_utils::codec::JsonCodec;
            let bytes = codec.serialize(&payload).map_err(|error| {
                ErrorObjectOwned::owned(-32603, format!("Export failed: {error}"), None::<()>)
            })?;
            z00z_utils::io::write_file(&export_path, &bytes).map_err(|error| {
                ErrorObjectOwned::owned(-32603, format!("Export failed: {error}"), None::<()>)
            })?;
            export_path
        };
        self.record_exported_tx(&wallet_id, &tx_id).await?;

        Ok(RuntimeExportTxResponse {
            success: true,
            export_path: Some(export_path.to_string_lossy().to_string()),
        })
    }

    async fn import_transaction_impl(
        &self,
        session: SessionToken,
        tx_data: String,
    ) -> RpcResult<RuntimeImportTxResponse> {
        let wallet_id = session.wallet_id.clone();
        self.verify_session(&session).await?;

        let (tx_bytes, package) = self.parse_portable_tx(&tx_data)?;
        let package_lifecycle = tx_rpc_support::lifecycle_from_package_status(&package.status);
        let verify = crate::tx::verify_full_tx_package(&tx_bytes).map_err(|error| {
            crate::rpc::error_mapping::runtime_tx_error_response(
                -32602,
                format!("Invalid tx package: {error}"),
                vec![crate::rpc::error_mapping::map_message_error_code(
                    &error.to_string(),
                )],
                Some(RuntimeTxLifecycle::Failed),
            )
        })?;
        if !verify.valid {
            return Err(crate::rpc::error_mapping::runtime_tx_error_response(
                -32602,
                format!("Invalid tx package: {}", verify.errors.join("; ")),
                crate::rpc::error_mapping::map_verify_error_codes(
                    &verify.errors,
                ),
                Some(RuntimeTxLifecycle::Failed),
            ));
        }
        let (chain_id, _, _) = self.tx_wallet_chain_meta()?;
        if package.chain_id != chain_id {
            return Err(crate::rpc::error_mapping::runtime_tx_error_response(
                -32602,
                "Imported package chain id does not match wallet chain".to_string(),
                vec![RuntimeTxErrorCode::WrongChain],
                Some(package_lifecycle),
            ));
        }
        if !is_import_ready(&package.status) {
            return Err(crate::rpc::error_mapping::runtime_tx_error_response(
                -32602,
                "Imported package is not import-ready".to_string(),
                vec![RuntimeTxErrorCode::NotImportReady],
                Some(package_lifecycle),
            ));
        }

        let owned_outputs = self.scan_pkg_outputs(&wallet_id, &package).await?;
        if owned_outputs.is_empty() {
            return Err(crate::rpc::error_mapping::runtime_tx_error_response(
                -32602,
                "Imported package has no wallet-owned outputs".to_string(),
                vec![RuntimeTxErrorCode::NoOwnedOutputs],
                Some(package_lifecycle),
            ));
        }
        let import_assets = self.collect_owned_assets(&wallet_id, &package).await?;

        let tx_id = PersistTxId::new(format!("tx_{}", package.tx_digest_hex));
        let summary = tx_runtime_state::tx_package_summary(&tx_bytes).ok_or_else(|| {
            crate::rpc::error_mapping::runtime_tx_error_response(
                -32602,
                "Invalid tx package summary".to_string(),
                vec![RuntimeTxErrorCode::InvalidPackage],
                Some(RuntimeTxLifecycle::Failed),
            )
        })?;
        let previous_record = self.try_load_stored_tx_record(&wallet_id, &tx_id).await?;
        if let Some(existing) = &previous_record {
            let existing_pkg: crate::tx::TxPackage = z00z_utils::codec::JsonCodec
                .deserialize(&existing.tx_bytes)
                .map_err(|error| {
                    crate::rpc::error_mapping::runtime_tx_error_response(
                        -32603,
                        format!("Stored tx package decode failed: {error}"),
                        vec![RuntimeTxErrorCode::InternalError],
                        Some(RuntimeTxLifecycle::Failed),
                    )
                })?;
            if !Self::same_tx_package_ignoring_status(&existing_pkg, &package) {
                self.record_conflicted_tx(&wallet_id, &tx_id).await?;
                return Err(crate::rpc::error_mapping::runtime_tx_error_response(
                    -32602,
                    "Imported package tx payload conflicts with existing tx id".to_string(),
                    vec![RuntimeTxErrorCode::DuplicateConflict],
                    Some(RuntimeTxLifecycle::Conflicted),
                ));
            }
        }
        if self
            .has_already_spent_input_conflict(&wallet_id, &tx_id, &summary.inputs)
            .await?
        {
            let latest_kind = self.load_tx_latest_kind(&wallet_id, &tx_id.0).await?;
            if previous_record.is_none() {
                self.persist_pending_tx(
                    &wallet_id,
                    &tx_id,
                    tx_bytes.clone(),
                    summary.amount,
                    summary.fee,
                    false,
                    self.now_ms(),
                )
                .await?;
            }
            if !matches!(
                latest_kind,
                Some(crate::backup::WalletTxHistoryEntryKind::AlreadySpent)
            ) {
                self.record_already_spent_tx(&wallet_id, &tx_id).await?;
            }
            return Err(crate::rpc::error_mapping::runtime_tx_error_response(
                -32602,
                "Imported package input is already spent by another tx".to_string(),
                vec![RuntimeTxErrorCode::AlreadySpent],
                Some(RuntimeTxLifecycle::AlreadySpent),
            ));
        }
        self.persist_pending_tx(
            &wallet_id,
            &tx_id,
            tx_bytes.clone(),
            summary.amount,
            summary.fee,
            true,
            self.now_ms(),
        )
        .await?;
        if let Err(error) = self
            .service
            .import_claimed_assets(&wallet_id, import_assets.as_slice())
            .await
        {
            self.rollback_persisted_tx(&wallet_id, &tx_id, previous_record)
                .await?;
            let tx_error_code =
                crate::rpc::error_mapping::map_wallet_error_code(&error);
            let lifecycle = match tx_error_code {
                RuntimeTxErrorCode::DuplicateConflict => RuntimeTxLifecycle::Conflicted,
                RuntimeTxErrorCode::AlreadySpent => RuntimeTxLifecycle::AlreadySpent,
                _ => RuntimeTxLifecycle::Failed,
            };
            match tx_error_code {
                RuntimeTxErrorCode::DuplicateConflict => {
                    self.record_conflicted_tx(&wallet_id, &tx_id).await?;
                }
                RuntimeTxErrorCode::AlreadySpent => {
                    self.record_already_spent_tx(&wallet_id, &tx_id).await?;
                }
                _ => {}
            }
            return Err(crate::rpc::error_mapping::runtime_tx_error_response(
                if matches!(tx_error_code, RuntimeTxErrorCode::InternalError) {
                    -32603
                } else {
                    -32602
                },
                format!("Imported package rejected: {error}"),
                vec![tx_error_code],
                Some(lifecycle),
            ));
        }

        let stored = self.load_stored_tx_record(&wallet_id, &tx_id).await?;
        let info = self.project_tx_info(&wallet_id, stored).await?;

        Ok(RuntimeImportTxResponse {
            tx_id,
            status: TxStatus::Pending,
            lifecycle: info.lifecycle,
            imported_outputs: owned_outputs,
            error_codes: Vec::new(),
        })
    }

    async fn reconcile_transaction_impl(
        &self,
        session: SessionToken,
        tx_id: PersistTxId,
    ) -> RpcResult<RuntimeReconcileTxResponse> {
        let wallet_id = session.wallet_id.clone();
        self.verify_session(&session).await?;

        let record = self.load_stored_tx_record(&wallet_id, &tx_id).await?;
        let tx_bytes = record.tx_bytes.clone();
        let package: crate::tx::TxPackage = z00z_utils::codec::JsonCodec
            .deserialize(&tx_bytes)
            .map_err(|error| {
                ErrorObjectOwned::owned(-32602, format!("Invalid tx package: {error}"), None::<()>)
            })?;
        let summary = tx_runtime_state::tx_package_summary(&tx_bytes).ok_or_else(|| {
            ErrorObjectOwned::owned(-32602, "Invalid tx package summary".to_string(), None::<()>)
        })?;
        let (chain_id, _, _) = self.tx_wallet_chain_meta()?;
        let evidence = match record.confirmation_evidence.clone() {
            Some(evidence) => evidence,
            None => self.load_confirmation_evidence(&tx_id).await.ok_or_else(|| {
                ErrorObjectOwned::owned(
                    -32602,
                    "Missing confirmation evidence".to_string(),
                    None::<()>,
                )
            })?,
        };
        let confirmation = self.validate_confirmation_evidence(&tx_id, &package, chain_id, &evidence)?;

        if matches!(record.status, crate::persistence::TxStatus::Confirmed) {
            let info = self.project_tx_info(&wallet_id, record.clone()).await?;
            return Ok(RuntimeReconcileTxResponse {
                tx_id,
                status: TxStatus::Confirmed,
                lifecycle: info.lifecycle,
                confirmation,
            });
        }

        let owned_outputs = self.collect_owned_assets(&wallet_id, &package).await?;
        let previous_record = record.clone();

        self.confirm_stored_tx_evidence(&wallet_id, &tx_id, evidence).await?;

        if let Err(error) = self
            .service
            .confirm_claimed_asset_spend(
                &wallet_id,
                &tx_id,
                summary.inputs.as_slice(),
                owned_outputs.as_slice(),
                if record.imported {
                    crate::db::redb_store::OwnedAssetSource::Import
                } else {
                    crate::db::redb_store::OwnedAssetSource::Change
                },
            )
            .await
        {
            self.rollback_persisted_tx(&wallet_id, &tx_id, Some(previous_record))
                .await?;
            return Err(crate::rpc::error_mapping::map_wallet_error_to_rpc(error));
        }

        self.upsert_tx_record(
            &wallet_id,
            &tx_id,
            TxStatus::Confirmed,
            summary.amount,
            summary.fee,
        )
        .await;
        tx_runtime_state::attach_tx_receipt(
            &self.pending_txs,
            &tx_id,
            tx_rpc_admission::receipt_to_persist(&confirmation),
        )
        .await;
        let stored = self.load_stored_tx_record(&wallet_id, &tx_id).await?;
        let info = self.project_tx_info(&wallet_id, stored).await?;

        Ok(RuntimeReconcileTxResponse {
            tx_id,
            status: TxStatus::Confirmed,
            lifecycle: info.lifecycle,
            confirmation,
        })
    }

    async fn collect_owned_assets(
        &self,
        wallet_id: &PersistWalletId,
        package: &crate::tx::TxPackage,
    ) -> RpcResult<Vec<z00z_core::Asset>> {
        let recv_keys = self
            .service
            .receiver_keys(wallet_id)
            .await
            .map_err(crate::rpc::error_mapping::map_wallet_error_to_rpc)?;
        let scanner = crate::receiver::StealthOutputScanner::from_keys(&recv_keys);
        let mut owned_outputs = Vec::new();

        for output in &package.tx.outputs {
            if tx_rpc_support::build_owned_out(output, &scanner)?.is_none() {
                continue;
            }

            let asset = output.asset_wire.clone().to_asset().map_err(|error| {
                crate::rpc::error_mapping::runtime_tx_error_response(
                    -32602,
                    format!("Invalid reconcile output asset: {error}"),
                    vec![crate::rpc::error_mapping::map_message_error_code(
                        &error.to_string(),
                    )],
                    Some(RuntimeTxLifecycle::Failed),
                )
            })?;

            if owned_outputs
                .iter()
                .map(|existing: &z00z_core::Asset| existing.asset_id())
                .any(|existing_id| existing_id == asset.asset_id())
            {
                continue;
            }

            owned_outputs.push(asset);
        }

        Ok(owned_outputs)
    }

    fn validate_confirmation_evidence(
        &self,
        tx_id: &PersistTxId,
        package: &crate::tx::TxPackage,
        chain_id: u32,
        evidence: &TxConfirmationEvidence,
    ) -> RpcResult<RuntimeConfirmationReceipt> {
        let expected_tx_id = format!("tx_{}", package.tx_digest_hex);
        if tx_id.0 != expected_tx_id || evidence.tx_id != tx_id.0 {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "Confirmation evidence tx id mismatch".to_string(),
                None::<()>,
            ));
        }
        if package.chain_id != chain_id || evidence.chain_id != chain_id {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "Confirmation evidence chain mismatch".to_string(),
                None::<()>,
            ));
        }
        if evidence.tx_hash_hex != package.tx_digest_hex {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "Confirmation evidence tx hash mismatch".to_string(),
                None::<()>,
            ));
        }
        if evidence.block_height == 0 || !evidence.verified {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "Confirmation evidence is not verified".to_string(),
                None::<()>,
            ));
        }
        Self::validate_evidence_hex(&evidence.checkpoint_id_hex, "checkpoint")?;
        Self::validate_evidence_hex(&evidence.prev_root_hex, "prev_root")?;
        Self::validate_evidence_hex(&evidence.new_root_hex, "new_root")?;

        let summary = tx_runtime_state::tx_package_summary(&z00z_utils::codec::JsonCodec
            .serialize(package)
            .map_err(|error| {
                ErrorObjectOwned::owned(-32603, format!("Package encode failed: {error}"), None::<()>)
            })?)
            .ok_or_else(|| {
                ErrorObjectOwned::owned(-32602, "Invalid tx package summary".to_string(), None::<()>)
            })?;
        let spent_ids = summary.inputs.into_iter().map(hex::encode).collect::<Vec<_>>();
        let created_ids = summary
            .outputs
            .into_iter()
            .map(hex::encode)
            .collect::<Vec<_>>();
        if evidence.spent_asset_ids_hex != spent_ids {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "Confirmation evidence spent inputs mismatch".to_string(),
                None::<()>,
            ));
        }
        if evidence.created_asset_ids_hex != created_ids {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "Confirmation evidence created outputs mismatch".to_string(),
                None::<()>,
            ));
        }

        Ok(tx_rpc_admission::evidence_to_confirmation(evidence))
    }

    fn validate_evidence_hex(value: &str, label: &str) -> RpcResult<()> {
        if value.len() != 64 || hex::decode(value).is_err() {
            return Err(ErrorObjectOwned::owned(
                -32602,
                format!("Invalid confirmation evidence {label}"),
                None::<()>,
            ));
        }
        Ok(())
    }

    async fn load_stored_tx_record(
        &self,
        wallet_id: &PersistWalletId,
        tx_id: &PersistTxId,
    ) -> RpcResult<crate::persistence::TxRecord> {
        self.try_load_stored_tx_record(wallet_id, tx_id)
            .await?
            .ok_or_else(|| ErrorObjectOwned::owned(-32602, "Unknown tx_id".to_string(), None::<()>))
    }

    async fn try_load_stored_tx_record(
        &self,
        wallet_id: &PersistWalletId,
        tx_id: &PersistTxId,
    ) -> RpcResult<Option<crate::persistence::TxRecord>> {
        if let Some(store) = &self.tx_store {
            let store = store.read().await;
            return match store.get(&tx_id.0) {
                Ok(record) => Ok(Some(record)),
                Err(crate::persistence::TxStorageError::NotFound(_)) => Ok(None),
                Err(error) => Err(ErrorObjectOwned::owned(
                    -32603,
                    format!("tx journal load failed: {error}"),
                    None::<()>,
                )),
            };
        }

        let store = self.wallet_tx_store(wallet_id);
        match store.get(&tx_id.0) {
            Ok(record) => Ok(Some(record)),
            Err(crate::persistence::TxStorageError::NotFound(_)) => Ok(None),
            Err(error) => Err(ErrorObjectOwned::owned(
                -32603,
                format!("tx journal load failed: {error}"),
                None::<()>,
            )),
        }
    }

    fn portable_tx_package(
        &self,
        tx_id: &PersistTxId,
        tx_bytes: Vec<u8>,
        package_status: &str,
    ) -> RpcResult<PortableWalletTxPackage> {
        let mut package: crate::tx::TxPackage = z00z_utils::codec::JsonCodec
            .deserialize(&tx_bytes)
            .map_err(|error| {
                ErrorObjectOwned::owned(-32603, format!("Export failed: {error}"), None::<()>)
            })?;
        let tx_hash_hex = package.tx_digest_hex.clone();
        let expected_tx_id = format!("tx_{tx_hash_hex}");
        if tx_id.0 != expected_tx_id {
            return Err(ErrorObjectOwned::owned(
                -32603,
                "Export failed: tx id does not match package digest".to_string(),
                None::<()>,
            ));
        }
        package.status = package_status.to_string();

        let chain_id = package.chain_id.to_string();
        let metadata_hash_hex = Self::portable_metadata_hash(1, &chain_id, &tx_hash_hex);
        let tx_bytes = z00z_utils::codec::JsonCodec
            .serialize(&package)
            .map_err(|error| {
                ErrorObjectOwned::owned(-32603, format!("Export failed: {error}"), None::<()>)
            })?;
        Ok(PortableWalletTxPackage {
            package_version: 1,
            chain_id,
            tx_hash_hex,
            tx_bytes,
            metadata_hash_hex,
        })
    }

    fn parse_portable_tx(&self, tx_data: &str) -> RpcResult<(Vec<u8>, crate::tx::TxPackage)> {
        let portable: PortableWalletTxPackage = z00z_utils::codec::JsonCodec
            .deserialize(tx_data.as_bytes())
            .map_err(|error| {
                crate::rpc::error_mapping::runtime_tx_error_response(
                    -32602,
                    format!("Invalid portable transaction package: {error}"),
                    vec![RuntimeTxErrorCode::InvalidEncoding],
                    Some(RuntimeTxLifecycle::Failed),
                )
            })?;

        if portable.package_version != 1 {
            return Err(crate::rpc::error_mapping::runtime_tx_error_response(
                -32602,
                "Unsupported portable transaction package version".to_string(),
                vec![RuntimeTxErrorCode::UnsupportedPackageVersion],
                Some(RuntimeTxLifecycle::Failed),
            ));
        }

        let expected = Self::portable_metadata_hash(
            portable.package_version,
            &portable.chain_id,
            &portable.tx_hash_hex,
        );
        if portable.metadata_hash_hex != expected {
            return Err(crate::rpc::error_mapping::runtime_tx_error_response(
                -32602,
                "Invalid portable transaction metadata hash".to_string(),
                vec![RuntimeTxErrorCode::InvalidDigest],
                Some(RuntimeTxLifecycle::Failed),
            ));
        }

        let package: crate::tx::TxPackage = z00z_utils::codec::JsonCodec
            .deserialize(&portable.tx_bytes)
            .map_err(|error| {
                crate::rpc::error_mapping::runtime_tx_error_response(
                    -32602,
                    format!("Invalid portable tx package: {error}"),
                    vec![RuntimeTxErrorCode::InvalidPackage],
                    Some(RuntimeTxLifecycle::Failed),
                )
            })?;
        if package.tx_digest_hex != portable.tx_hash_hex {
            return Err(crate::rpc::error_mapping::runtime_tx_error_response(
                -32602,
                "Portable transaction digest conflicts with tx bytes".to_string(),
                vec![RuntimeTxErrorCode::InvalidDigest],
                Some(RuntimeTxLifecycle::Failed),
            ));
        }
        if package.chain_id.to_string() != portable.chain_id {
            return Err(crate::rpc::error_mapping::runtime_tx_error_response(
                -32602,
                "Portable transaction chain id conflicts with tx bytes".to_string(),
                vec![RuntimeTxErrorCode::WrongChain],
                Some(RuntimeTxLifecycle::Failed),
            ));
        }

        Ok((portable.tx_bytes, package))
    }

    fn portable_metadata_hash(version: u16, chain_id: &str, tx_hash_hex: &str) -> String {
        let version_bytes = version.to_le_bytes();
        hex::encode(z00z_crypto::blake2b_hash(
            b"z00z.wallet.portable.metadata.v1",
            &[&version_bytes, chain_id.as_bytes(), tx_hash_hex.as_bytes()],
        ))
    }

    fn same_tx_package_ignoring_status(
        left: &crate::tx::TxPackage,
        right: &crate::tx::TxPackage,
    ) -> bool {
        let mut left = left.clone();
        let mut right = right.clone();
        left.status.clear();
        right.status.clear();
        left == right
    }
}

fn tx_export_write_lock() -> &'static std::sync::Mutex<()> {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(()))
}
