impl AssetRpcImpl {
    fn log_receive_reject(
        &self,
        wallet_id: &str,
        asset_id_hex: &str,
        action: &str,
        reason: ReceiveReject,
        detail: Option<&str>,
        message: &str,
    ) {
        let logger = z00z_utils::logger::TracingLogger;
        let mut line = format!(
            "wallet_id={wallet_id} asset_id={asset_id_hex} action={action} reason={} {message}",
            reason.log_code()
        );
        if let Some(detail) = detail {
            line.push_str(&format!(" detail={detail}"));
        }
        if reason.is_alerting() {
            z00z_utils::logger::Logger::warn(&logger, &line);
        } else {
            z00z_utils::logger::Logger::debug(&logger, &line);
        }
    }

    fn log_receive_info(
        &self,
        wallet_id: &str,
        asset_id_hex: &str,
        amount: u64,
        action: &str,
        message: &str,
    ) {
        let logger = z00z_utils::logger::TracingLogger;
        let line = format!(
            "wallet_id={wallet_id} asset_id={asset_id_hex} amount={amount} action={action} {message}"
        );
        z00z_utils::logger::Logger::info(&logger, &line);
    }

    fn log_send_info(
        &self,
        wallet_id: &str,
        asset_id_hex: &str,
        tx_id: &str,
        action: &str,
        is_request: bool,
        has_tag16: bool,
        message: &str,
    ) {
        let logger = z00z_utils::logger::TracingLogger;
        let line = format!(
            "wallet_id={wallet_id} asset_id={asset_id_hex} tx_id={tx_id} action={action} is_request={is_request} has_tag16={has_tag16} {message}"
        );
        z00z_utils::logger::Logger::info(&logger, &line);
    }

    async fn receive_asset_impl(
        &self,
        session: SessionToken,
        asset_id: AssetId,
    ) -> RpcResult<RuntimeReceiveAssetResponse> {
        self.verify_session(&session).await?;
        let wallet_id = session.wallet_id.clone();
        self.reject_non_asset_alias(&wallet_id, asset_id).await?;

        let leaf = self.lookup_receive_asset(&wallet_id, asset_id).await?;
        let result = self
            .service
            .scan_asset_report(&wallet_id, &leaf)
            .await
            .map_err(|reason| {
                let asset_id_hex = hex::encode(asset_id);
                self.log_receive_reject(
                    &wallet_id.0,
                    &asset_id_hex,
                    "receive_precheck_reject",
                    reason,
                    None,
                    "stealth receive rejected before outward mapping",
                );
                Self::recv_err(reason)
            })?;
        let status = result.status;

        match status {
            ReceiveStatus::Detected => {
                let canonical_id = leaf.asset_id();
                let recv_keys = self
                    .service
                    .receiver_keys(&wallet_id)
                    .await
                    .map_err(|error| {
                        let asset_id_hex = hex::encode(asset_id);
                        let error_text = error.to_string();
                        self.log_receive_reject(
                            &wallet_id.0,
                            &asset_id_hex,
                            "receive_runtime_reject",
                            ReceiveReject::RuntimeFail,
                            Some(&error_text),
                            "stealth receive public-key load failed",
                        );
                        Self::recv_err(ReceiveReject::RuntimeFail)
                    })?;
                let scanner = StealthOutputScanner::from_keys(&recv_keys);
                let ScanResult::Mine { wallet_output } = scanner.scan_leaf(&leaf) else {
                    let asset_id_hex = hex::encode(asset_id);
                    self.log_receive_reject(
                        &wallet_id.0,
                        &asset_id_hex,
                        "receive_runtime_reject",
                        ReceiveReject::RuntimeFail,
                        None,
                        "receive report detected but owned output reconstruction failed",
                    );
                    return Err(Self::recv_err(ReceiveReject::RuntimeFail));
                };

                self.log_receive_info(
                    &wallet_id.0,
                    &hex::encode(canonical_id),
                    wallet_output.amount,
                    "receive_scanner_mine",
                    "stealth receive scanner detected owned output",
                );

                Ok(RuntimeReceiveAssetResponse {
                    asset: RuntimeAssetRef {
                        asset_id: canonical_id,
                        serial_id: leaf.serial_id,
                        symbol: leaf.definition.symbol.clone(),
                        class: leaf.definition.class,
                    },
                    status: WalletService::recv_code(status).to_string(),
                    owner_handle: hex::encode(recv_keys.owner_handle),
                    view_key: hex::encode(recv_keys.view_pk.as_bytes()),
                    expires_at: None,
                })
            }
            ReceiveStatus::InvalidProof => {
                let reject = result.reject.unwrap_or(ReceiveReject::InvalidProof);
                let asset_id_hex = hex::encode(asset_id);
                self.log_receive_reject(
                    &wallet_id.0,
                    &asset_id_hex,
                    "receive_scanner_invalid_proof",
                    reject,
                    None,
                    "stealth receive scanner produced MaybeMine",
                );
                Err(Self::recv_err(reject))
            }
            ReceiveStatus::NotMine => {
                let reject = result.reject.unwrap_or(ReceiveReject::NotMine);
                let asset_id_hex = hex::encode(asset_id);
                self.log_receive_reject(
                    &wallet_id.0,
                    &asset_id_hex,
                    "receive_scanner_not_mine",
                    reject,
                    None,
                    "stealth receive scanner did not match wallet",
                );
                Err(Self::recv_err(reject))
            }
        }
    }

    async fn send_asset_impl(
        &self,
        session: SessionToken,
        asset_id: AssetId,
        recipient: String,
        amount: u64,
    ) -> RpcResult<RuntimeSendAssetResponse> {
        self.verify_session(&session).await?;
        let wallet_id = session.wallet_id.clone();
        self.reject_non_asset_alias(&wallet_id, asset_id).await?;

        if amount == 0 {
            return Err(jsonrpsee::types::ErrorObjectOwned::owned(
                -32602,
                "Invalid amount: must be > 0".to_string(),
                None::<()>,
            ));
        }

        let target = self.parse_send_target(&recipient)?;
        self.verify_send_tofu(&wallet_id, &target).await?;

        if self
            .quarantine_ids(&wallet_id)
            .await
            .iter()
            .any(|id| id == &asset_id)
        {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "Asset is quarantined and not spendable".to_string(),
                None::<()>,
            ));
        }

        match self.asset_send_precheck(&wallet_id).await {
            Ok(()) => {}
            Err((retry_after_seconds, current_count, max_requests)) => {
                let data = RuntimeRateLimitError {
                    method: "asset.send".to_string(),
                    tier: "asset_send".to_string(),
                    current_count,
                    max_requests,
                    window_seconds: ASSET_SEND_RATE_LIMIT_WINDOW,
                    retry_after_seconds,
                };

                return Err(ErrorObjectOwned::owned(
                    SecurityErrorCode::RateLimitExceeded.code(),
                    SecurityErrorCode::RateLimitExceeded.message().to_string(),
                    Some(data),
                ));
            }
        }

        tx_rpc_support::validate_policy(
            self.wallet_service(),
            &self.time_provider,
            &wallet_id,
            asset_id,
            &recipient,
            amount,
        )
        .await?;

        let tx_seed = self.random_seed();
        let tx_digest = self.random_seed();
        let mut sender_wallet = SenderWallet::new(tx_seed);
        let mut pins = self
            .service
            .load_tofu(&wallet_id)
            .await
            .map_err(map_wallet_error_to_rpc)?;
        let chain_id = wallet_chain_id()?;

        let (card, stealth) = match &target {
            SendTarget::Card(card) => {
                let card = card.clone();
                let stealth = build_card_stealth_output_validated(
                    &card,
                    BuildCheck {
                        pins: &mut pins,
                        chain_id,
                    },
                    &mut sender_wallet,
                    &tx_digest,
                    0,
                    amount,
                    &asset_id,
                )
                .map_err(|_| {
                    ErrorObjectOwned::owned(
                        -32602,
                        "SEND_STEALTH_BUILD_FAILED".to_string(),
                        None::<()>,
                    )
                })?;
                (card, stealth)
            }
            SendTarget::Request(request) => {
                let card = self.request_to_card(request);
                let stealth = build_tx_stealth_output_validated(
                    &card,
                    Some(request),
                    BuildCheck {
                        pins: &mut pins,
                        chain_id,
                    },
                    &mut sender_wallet,
                    &tx_digest,
                    0,
                    amount,
                    &asset_id,
                )
                .map_err(|_| {
                    ErrorObjectOwned::owned(
                        -32602,
                        "SEND_STEALTH_BUILD_FAILED".to_string(),
                        None::<()>,
                    )
                })?;
                (card, stealth)
            }
        };

        let mut resp = self
            .service
            .send_asset(&wallet_id, asset_id, recipient.clone(), amount);
        resp.status = "stealth_submitted".to_string();
        resp.owner_handle = hex::encode(card.owner_handle);
        resp.recipient = recipient;
        resp.amount = amount;

        self.log_send_info(
            &wallet_id.0,
            &hex::encode(asset_id),
            &resp.tx_id.0,
            "stealth_send_submitted",
            matches!(target, SendTarget::Request(_)),
            stealth.tag16.is_some(),
            "stealth send constructed and submitted",
        );

        Ok(resp)
    }
}
