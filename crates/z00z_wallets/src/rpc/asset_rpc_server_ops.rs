impl AssetRpcImpl {
    async fn split_asset_impl(
        &self,
        session: SessionToken,
        asset_id: AssetId,
        amounts: Vec<u64>,
    ) -> RpcResult<RuntimeSplitAssetResponse> {
        self.verify_session(&session).await?;
        let wallet_id = session.wallet_id.clone();
        self.reject_non_asset_alias(&wallet_id, asset_id).await?;

        if amounts.is_empty() {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "Invalid amounts: must provide at least one split amount".to_string(),
                None::<()>,
            ));
        }

        if amounts.contains(&0) {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "Invalid amounts: split amounts must be > 0".to_string(),
                None::<()>,
            ));
        }

        let original = self
            .load_wallet_asset(&wallet_id, asset_id, "Unknown asset_id for this wallet")
            .await?;
        let original_asset_id = original.asset_id();

        let total = amounts.iter().try_fold(0u64, |acc, amount| {
            acc.checked_add(*amount).ok_or_else(|| {
                ErrorObjectOwned::owned(
                    -32602,
                    "Invalid amounts: overflow while summing split amounts".to_string(),
                    None::<()>,
                )
            })
        })?;

        if total != original.amount {
            return Err(ErrorObjectOwned::owned(
                -32602,
                format!(
                    "Invalid split amounts: sum={} must equal original amount={} ",
                    total, original.amount
                ),
                None::<()>,
            ));
        }

        let split_outputs = amounts
            .iter()
            .enumerate()
            .map(|(i, amount)| {
                self.build_local_mutation_output(
                    &wallet_id,
                    "split_asset",
                    original.definition.clone(),
                    original.serial_id,
                    *amount,
                    i,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let tx_id = self
            .local_mutation_exec(&wallet_id, "split_asset", &[original.clone()], &split_outputs)
            .submit()
            .await?;

        Ok(RuntimeSplitAssetResponse {
            original_asset_id,
            splits: split_outputs
                .iter()
                .map(|asset| RuntimeAssetAmount {
                    asset: Self::runtime_asset_ref(asset),
                    amount: asset.amount,
                })
                .collect(),
            tx_id: Some(tx_id),
        })
    }

    async fn stake_assets_impl(
        &self,
        session: SessionToken,
        asset_id: AssetId,
        amount: u64,
    ) -> RpcResult<RuntimeStakeAssetsResponse> {
        self.verify_session(&session).await?;
        let wallet_id = session.wallet_id.clone();
        self.reject_non_asset_alias(&wallet_id, asset_id).await?;

        if amount == 0 {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "Invalid amount: must be > 0".to_string(),
                None::<()>,
            ));
        }

        let asset = self
            .load_wallet_asset(&wallet_id, asset_id, "Unknown asset_id for this wallet")
            .await?;
        let canonical_asset_id = asset.asset_id();

        if amount > asset.amount {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "Invalid amount: exceeds available amount".to_string(),
                None::<()>,
            ));
        }

        // Compatibility stake ids self-describe the echo surface so unstake can
        // round-trip without introducing a parallel mutable stake plane.
        let stake_id = self.next_stake_id(canonical_asset_id, amount).await;
        let start_time = self.now_ms();
        let end_time = start_time.saturating_add(86_400_000);
        let stake_output = self.build_local_mutation_output(
            &wallet_id,
            "stake_assets",
            asset.definition.clone(),
            asset.serial_id,
            amount,
            0,
        )?;
        let _tx_id = self
            .local_mutation_exec(&wallet_id, "stake_assets", &[asset.clone()], &[stake_output])
            .submit()
            .await?;

        Ok(RuntimeStakeAssetsResponse {
            stake_id,
            asset: RuntimeAssetRef {
                asset_id: canonical_asset_id,
                serial_id: asset.serial_id,
                symbol: asset.definition.symbol.clone(),
                class: asset.definition.class,
            },
            amount,
            start_time,
            end_time,
            apy: 0.0,
        })
    }

    async fn swap_assets_impl(
        &self,
        session: SessionToken,
        from_asset_id: AssetId,
        to_asset_id: AssetId,
        amount: u64,
    ) -> RpcResult<RuntimeSwapAssetsResponse> {
        self.verify_session(&session).await?;
        let wallet_id = session.wallet_id.clone();
        self.reject_non_asset_alias(&wallet_id, from_asset_id).await?;
        self.reject_non_asset_alias(&wallet_id, to_asset_id).await?;

        if amount == 0 {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "Invalid amount: must be > 0".to_string(),
                None::<()>,
            ));
        }

        if from_asset_id == to_asset_id {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "Invalid swap: from_asset_id must differ from to_asset_id".to_string(),
                None::<()>,
            ));
        }

        let from_asset = self
            .load_wallet_asset(
                &wallet_id,
                from_asset_id,
                "Unknown from_asset_id for this wallet",
            )
            .await?;
        let canonical_from_id = from_asset.asset_id();
        let to_asset = self
            .load_wallet_asset(
                &wallet_id,
                to_asset_id,
                "Unknown to_asset_id for this wallet",
            )
            .await?;
        let canonical_to_id = to_asset.asset_id();

        if amount > from_asset.amount {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "Invalid amount: exceeds available amount".to_string(),
                None::<()>,
            ));
        }
        let swap_output = self.build_local_mutation_output(
            &wallet_id,
            "swap_assets",
            to_asset.definition.clone(),
            to_asset.serial_id,
            amount,
            0,
        )?;
        let tx_id = self
            .local_mutation_exec(
                &wallet_id,
                "swap_assets",
                &[from_asset.clone()],
                &[swap_output],
            )
            .submit()
            .await?;

        Ok(RuntimeSwapAssetsResponse {
            from_asset_id: canonical_from_id,
            from_serial_id: from_asset.serial_id,
            from_symbol: from_asset.definition.symbol.clone(),
            from_class: from_asset.definition.class,
            to_asset_id: canonical_to_id,
            to_serial_id: to_asset.serial_id,
            to_symbol: to_asset.definition.symbol.clone(),
            to_class: to_asset.definition.class,
            from_amount: amount,
            to_amount: amount,
            exchange_rate: 1.0,
            fee: 0,
            tx_id,
        })
    }

    async fn unstake_assets_impl(
        &self,
        session: SessionToken,
        stake_id: String,
    ) -> RpcResult<RuntimeUnstakeAssetsResponse> {
        self.verify_session(&session).await?;
        let wallet_id = session.wallet_id.clone();

        let (asset_id, amount) = asset_rpc_stakes::parse_stake_id(&stake_id)
            .ok_or_else(|| {
                ErrorObjectOwned::owned(
                    -32602,
                    "Unknown stake_id for this wallet".to_string(),
                    None::<()>,
                )
            })?;

        if amount == 0 {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "Unknown stake_id for this wallet".to_string(),
                None::<()>,
            ));
        }
        let unstaked_at = self.now_ms();

        let asset = self
            .load_wallet_asset(&wallet_id, asset_id, "Unknown stake_id for this wallet")
            .await?;

        if amount > asset.amount {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "Unknown stake_id for this wallet".to_string(),
                None::<()>,
            ));
        }
        let unstake_output = self.build_local_mutation_output(
            &wallet_id,
            "unstake_assets",
            asset.definition.clone(),
            asset.serial_id,
            amount,
            0,
        )?;
        let _tx_id = self
            .local_mutation_exec(
                &wallet_id,
                "unstake_assets",
                &[asset.clone()],
                &[unstake_output],
            )
            .submit()
            .await?;

        Ok(RuntimeUnstakeAssetsResponse {
            stake_id,
            asset: RuntimeAssetRef {
                asset_id,
                serial_id: asset.serial_id,
                symbol: asset.definition.symbol.clone(),
                class: asset.definition.class,
            },
            amount,
            reward: 0,
            unstaked_at,
        })
    }
}
