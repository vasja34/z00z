impl TxRpcImpl {
    async fn send_transaction_impl(
        &self,
        session: SessionToken,
        recipient: String,
        amount: u64,
        asset_id: Option<String>,
        memo: Option<String>,
        idempotency_key: Option<IdempotencyKey>,
        timestamp: Option<u64>,
    ) -> RpcResult<RuntimeSendTxResponse> {
        let wallet_id = session.wallet_id.clone();
        self.verify_session(&session).await?;

        if amount == 0 {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "Invalid amount: must be > 0".to_string(),
                None::<()>,
            ));
        }

        let timestamp = timestamp.ok_or_else(|| {
            ErrorObjectOwned::owned(-32602, "Missing timestamp".to_string(), None::<()>)
        })?;

        if timestamp < 1_000_000_000_000 {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "Invalid timestamp: must be Unix milliseconds".to_string(),
                None::<()>,
            ));
        }

        let diff_ms = self.now_ms().abs_diff(timestamp);
        let window_ms = TX_SEND_TIMESTAMP_WINDOW_SECONDS.saturating_mul(1_000);

        if diff_ms > window_ms {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "Invalid timestamp: outside allowed window".to_string(),
                None::<()>,
            ));
        }

        if let Some(key) = &idempotency_key {
            if !key.is_valid() {
                return Err(ErrorObjectOwned::owned(
                    -32602,
                    "Invalid idempotency_key".to_string(),
                    None::<()>,
                ));
            }

            if let Some(cached) = self.idempotency_get(&wallet_id, key).await {
                return Ok(cached);
            }
        }

        match self.tx_send_precheck(&wallet_id).await {
            Ok(()) => {}
            Err((retry_after_seconds, current_count, max_requests)) => {
                let data = RuntimeRateLimitError {
                    method: "wallet.tx.send_transaction".to_string(),
                    tier: "tx_send".to_string(),
                    current_count,
                    max_requests,
                    window_seconds: tx_rpc_rate_limits::TX_SEND_RATE_LIMIT_WINDOW,
                    retry_after_seconds,
                };

                return Err(ErrorObjectOwned::owned(
                    SecurityErrorCode::RateLimitExceeded.code(),
                    SecurityErrorCode::RateLimitExceeded.message().to_string(),
                    Some(data),
                ));
            }
        }

        if memo.as_ref().is_some_and(|value| value.len() > 512) {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "Invalid memo: too long".to_string(),
                None::<()>,
            ));
        }

        let parsed_asset_id = Self::parse_asset_id_hex(asset_id.clone())?;
        self.reject_non_asset_cash_id(&wallet_id, parsed_asset_id).await?;
        self.validate_policy(&wallet_id, parsed_asset_id, &recipient, amount)
            .await?;

        let build = self
            .build_transaction_impl(session.clone(), recipient, amount, asset_id)
            .await?;
        let tx_bytes = build.raw_tx.as_bytes().to_vec();
        let package: crate::tx::TxPackage = z00z_utils::codec::JsonCodec
            .deserialize(&tx_bytes)
            .map_err(|error| {
                ErrorObjectOwned::owned(
                    -32603,
                    format!("Built package decode failed: {error}"),
                    None::<()>,
                )
            })?;

        let admitted = self
            .broadcast_transaction_impl(
                session.clone(),
                build.raw_tx,
                idempotency_key.clone(),
                Some(timestamp),
            )
            .await?;
        let tx_id = admitted.tx_id;
        let fee = package.tx.fee;
        let stored = self.load_stored_tx_record(&wallet_id, &tx_id).await?;
        let info = self.project_tx_info(&wallet_id, stored).await?;

        let resp = RuntimeSendTxResponse {
            tx_id: tx_id.clone(),
            status: TxStatus::Pending,
            lifecycle: info.lifecycle,
        };

        self.upsert_tx_record_at(&wallet_id, &tx_id, TxStatus::Pending, amount, fee, timestamp)
            .await;

        if let Some(key) = idempotency_key {
            self.idempotency_put(&wallet_id, key, &resp).await;
        }

        Ok(resp)
    }
}
