#[tokio::test]
async fn test_tx_send_missing_timestamp() {
    let time = mock_time_with_offset(0);
    let ctx = setup_session(time.clone()).await;
    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time);

    let err = rpc
        .send_transaction(
            ctx.session.clone(),
            "alice".to_string(),
            1,
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap_err();

    assert_eq!(err.code(), -32602);
}

#[tokio::test]
async fn test_tx_send_timestamp_window() {
    let time = mock_time_with_offset(1000);
    let ctx = setup_session(time.clone()).await;
    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time);

    let now_ms = ms(BASE_TIME_SECS.saturating_add(1000));
    let too_old_ms = now_ms.saturating_sub(
        TX_SEND_TIMESTAMP_WINDOW_SECONDS
            .saturating_add(1)
            .saturating_mul(1_000),
    );

    let err = rpc
        .send_transaction(
            ctx.session.clone(),
            "alice".to_string(),
            1,
            None,
            None,
            None,
            Some(too_old_ms),
        )
        .await
        .unwrap_err();

    assert_eq!(err.code(), -32602);
}

#[tokio::test]
async fn test_tx_send_idempotent_same() {
    let time = mock_time_with_offset(10);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 50_000, 7).await;
    let recipient = mk_recv_card_compact(&ctx).await;
    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time.clone());

    let key = IdempotencyKey("12345678-1234-1234-1234-1234567890ab".to_string());

    let first = rpc
        .send_transaction(
            ctx.session.clone(),
            recipient.clone(),
            1,
            None,
            None,
            Some(key.clone()),
            Some(ms(BASE_TIME_SECS.saturating_add(10))),
        )
        .await
        .unwrap();

    let second = rpc
        .send_transaction(
            ctx.session.clone(),
            recipient,
            1,
            None,
            None,
            Some(key),
            Some(ms(BASE_TIME_SECS.saturating_add(10))),
        )
        .await
        .unwrap();

    assert_eq!(first.tx_id, second.tx_id);
    assert!(first.tx_id.0.starts_with("tx_"));
    assert_eq!(first.lifecycle, RuntimeTxLifecycle::Admitted);
    assert_eq!(second.lifecycle, RuntimeTxLifecycle::Admitted);
}

#[tokio::test]
async fn test_tx_send_limits_10() {
    let time = mock_time_with_offset(10);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 50_000, 7).await;
    let recipient = mk_recv_card_compact(&ctx).await;
    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time.clone());

    for _ in 0..10 {
        rpc.tx_send_precheck(&ctx.wallet_id).await.unwrap();
    }

    let err = rpc
        .send_transaction(
            ctx.session.clone(),
            recipient.clone(),
            1,
            None,
            None,
            None,
            Some(ms(BASE_TIME_SECS.saturating_add(10))),
        )
        .await
        .unwrap_err();

    assert_eq!(err.code(), SecurityErrorCode::RateLimitExceeded.code());

    time.advance_by(Duration::from_secs(60));
    let _ = rpc
        .send_transaction(
            ctx.session.clone(),
            recipient,
            1,
            None,
            None,
            None,
            Some(ms(BASE_TIME_SECS.saturating_add(70))),
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn test_tx_send_policy_max() {
    let time = mock_time_with_offset(10);
    let ctx = setup_session(time.clone()).await;

    let rules = PolicyRules {
        max_tx_amount: Some(10),
        max_daily_amount: None,
        allowed_assets: None,
        allowed_recipients: None,
        require_confirmation: false,
        time_restrictions: None,
    };

    let mut settings = PersistWalletSettings::stub_default();
    settings.policy_rules = Some(rules);
    ctx.service
        .set_wallet_settings(ctx.wallet_id.clone(), settings)
        .await
        .unwrap();

    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time);
    let err = rpc
        .send_transaction(
            ctx.session.clone(),
            "alice".to_string(),
            11,
            None,
            None,
            None,
            Some(ms(BASE_TIME_SECS.saturating_add(10))),
        )
        .await
        .unwrap_err();

    assert_eq!(err.code(), -32602);
}

#[tokio::test]
async fn test_tx_policy_day_cap() {
    let time = mock_time_with_offset(10);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 50_000, 7).await;

    let asset_definition_id = ctx
        .service
        .list_claimed_assets(&ctx.wallet_id)
        .await
        .expect("claimed assets")[0]
        .definition
        .id;

    let rules = PolicyRules {
        max_tx_amount: None,
        max_daily_amount: Some(7),
        allowed_assets: None,
        allowed_recipients: None,
        require_confirmation: false,
        time_restrictions: None,
    };

    let mut settings = PersistWalletSettings::stub_default();
    settings.policy_rules = Some(rules);
    ctx.service
        .set_wallet_settings(ctx.wallet_id.clone(), settings)
        .await
        .unwrap();

    let recipient = mk_recv_card_compact(&ctx).await;
    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time.clone());

    rpc.send_transaction(
        ctx.session.clone(),
        recipient.clone(),
        4,
        Some(hex::encode(asset_definition_id)),
        None,
        None,
        Some(ms(BASE_TIME_SECS.saturating_add(10))),
    )
    .await
    .unwrap();

    let err = rpc
        .send_transaction(
            ctx.session.clone(),
            recipient,
            4,
            Some(hex::encode(asset_definition_id)),
            None,
            None,
            Some(ms(BASE_TIME_SECS.saturating_add(10))),
        )
        .await
        .unwrap_err();

    assert_eq!(err.code(), -32602);
    assert!(err.message().contains("daily limit exceeded"));

    let history_path = ctx.service.wallet_history_jsonl_path(&ctx.wallet_id);
    let store = crate::persistence::tx::TxStorageImpl::new(
        history_path,
        tx_rpc_support::TimeProviderRef(time),
    );
    let (day_start_ms, day_end_ms) =
        crate::wallet::policy::utc_day_window_ms(ms(BASE_TIME_SECS.saturating_add(10)));
    let window = store
        .policy_spend_window(asset_definition_id, day_start_ms, day_end_ms)
        .expect("policy spend window");

    assert_eq!(window.spent_amount, 4);
    assert_eq!(window.pending_confirmation_count, 1);
}

#[tokio::test]
async fn test_tx_policy_confirm_gate() {
    let time = mock_time_with_offset(10);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 50_000, 7).await;

    let asset_definition_id = ctx
        .service
        .list_claimed_assets(&ctx.wallet_id)
        .await
        .expect("claimed assets")[0]
        .definition
        .id;

    let rules = PolicyRules {
        max_tx_amount: None,
        max_daily_amount: None,
        allowed_assets: None,
        allowed_recipients: None,
        require_confirmation: true,
        time_restrictions: None,
    };

    let mut settings = PersistWalletSettings::stub_default();
    settings.policy_rules = Some(rules);
    ctx.service
        .set_wallet_settings(ctx.wallet_id.clone(), settings)
        .await
        .unwrap();

    let recipient = mk_recv_card_compact(&ctx).await;
    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time.clone());

    let first = rpc
        .send_transaction(
            ctx.session.clone(),
            recipient.clone(),
            1,
            Some(hex::encode(asset_definition_id)),
            None,
            None,
            Some(ms(BASE_TIME_SECS.saturating_add(10))),
        )
        .await
        .unwrap();

    let err = rpc
        .send_transaction(
            ctx.session.clone(),
            recipient.clone(),
            1,
            Some(hex::encode(asset_definition_id)),
            None,
            None,
            Some(ms(BASE_TIME_SECS.saturating_add(10))),
        )
        .await
        .unwrap_err();

    assert_eq!(err.code(), -32602);
    assert!(err.message().contains("confirmation required"));

    let history_path = ctx.service.wallet_history_jsonl_path(&ctx.wallet_id);
    let mut store = crate::persistence::tx::TxStorageImpl::new(
        history_path.clone(),
        tx_rpc_support::TimeProviderRef(time.clone()),
    );
    crate::persistence::TxStorage::record_confirmed(&mut store, &first.tx_id.0, 12)
        .expect("confirm first tx");

    let reopened = crate::persistence::tx::TxStorageImpl::new(
        history_path,
        tx_rpc_support::TimeProviderRef(time.clone()),
    );
    let (day_start_ms, day_end_ms) =
        crate::wallet::policy::utc_day_window_ms(ms(BASE_TIME_SECS.saturating_add(10)));
    let window = reopened
        .policy_spend_window(asset_definition_id, day_start_ms, day_end_ms)
        .expect("policy spend window");
    assert_eq!(window.pending_confirmation_count, 0);

    rpc.send_transaction(
        ctx.session.clone(),
        recipient,
        1,
        Some(hex::encode(asset_definition_id)),
        None,
        None,
        Some(ms(BASE_TIME_SECS.saturating_add(10))),
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn test_tx_build_raw_tx() {
    let time = mock_time_with_offset(10);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 50_000, 7).await;
    let recipient = mk_recv_card_compact(&ctx).await;
    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time.clone());

    let resp = rpc
        .build_transaction(ctx.session.clone(), recipient, 123, None)
        .await
        .unwrap();

    let tx_bytes = resp.raw_tx.as_bytes().to_vec();
    let pkg: crate::tx::TxPackage = JsonCodec.deserialize(&tx_bytes).expect("tx package");
    let verify = crate::tx::verify_full_tx_package(&tx_bytes).expect("verify package");

    assert_eq!(resp.tx_id.0, format!("tx_{}", pkg.tx_digest_hex));
    assert!(!resp.raw_tx.is_empty());
    assert!(verify.valid, "verification errors: {:?}", verify.errors);
    assert_eq!(pkg.tx.outputs.len(), 3);
    assert_eq!(pkg.tx.outputs[0].role, crate::tx::TxOutRole::Recipient);
    assert_eq!(pkg.tx.outputs[1].role, crate::tx::TxOutRole::Change);
    assert_eq!(pkg.tx.outputs[2].role, crate::tx::TxOutRole::Fee);
    assert!(pkg.tx.fee > 0);

    let history_path = ctx.service.wallet_history_jsonl_path(&ctx.wallet_id);
    let store = crate::persistence::tx::TxStorageImpl::new(
        history_path,
        tx_rpc_support::TimeProviderRef(time),
    );
    let stored = crate::persistence::TxStorage::get(&store, &resp.tx_id.0)
        .expect("built tx must be persisted");
    assert!(!stored.imported);
    assert!(matches!(stored.status, crate::persistence::TxStatus::Pending));
}

#[tokio::test]
async fn test_tx_build_rejects_voucher() {
    let time = mock_time_with_offset(10);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 50_000, 7).await;

    let voucher = test_owned_voucher_payload(ctx.wallet_id.clone(), 81);
    ctx.service
        .put_owned_object_for_tests(&ctx.wallet_id, OwnedObjectPayload::Voucher(voucher.clone()))
        .await
        .expect("voucher insert must succeed");

    let recipient = mk_recv_card_compact(&ctx).await;
    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time);
    let err = rpc
        .build_transaction(
            ctx.session,
            recipient,
            1,
            Some(hex::encode(voucher.terminal_id.as_bytes())),
        )
        .await
        .unwrap_err();

    assert_eq!(err.code(), -32602);
    assert!(err.message().contains("voucher inventory"));
}

#[tokio::test]
async fn test_tx_send_rejects_right() {
    let time = mock_time_with_offset(10);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 50_000, 7).await;

    let right = test_owned_right_payload(ctx.wallet_id.clone(), 82);
    ctx.service
        .put_owned_object_for_tests(&ctx.wallet_id, OwnedObjectPayload::Right(right.clone()))
        .await
        .expect("right insert must succeed");

    let recipient = mk_recv_card_compact(&ctx).await;
    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time);
    let err = rpc
        .send_transaction(
            ctx.session,
            recipient,
            1,
            Some(hex::encode(right.terminal_id.as_bytes())),
            None,
            None,
            Some(ms(BASE_TIME_SECS.saturating_add(10))),
        )
        .await
        .unwrap_err();

    assert_eq!(err.code(), -32602);
    assert!(err.message().contains("right inventory"));
}

#[tokio::test]
async fn test_tx_build_rejects_locks() {
    let time = mock_time_with_offset(10);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 50_000, 7).await;

    let claimed = ctx
        .service
        .list_claimed_assets(&ctx.wallet_id)
        .await
        .expect("claimed assets");
    let asset_definition_id = claimed[0].definition.id;
    for (offset, asset) in claimed.iter().enumerate() {
        let lock = test_mandate_lock_payload(
            ctx.wallet_id.clone(),
            90_u8.saturating_add(offset as u8),
            asset.asset_id(),
            asset.amount,
        );
        ctx.service
            .put_owned_object_for_tests(&ctx.wallet_id, OwnedObjectPayload::Right(lock))
            .await
            .expect("validator lock insert must succeed");
    }

    let spendable = ctx
        .service
        .list_spendable_asset_rows(&ctx.wallet_id, Some(asset_definition_id))
        .await
        .expect("spendable rows");
    assert!(
        spendable.is_empty(),
        "validator_mandate_lock_v1 must remove locked rows from ordinary spend selection"
    );

    let recipient = mk_recv_card_compact(&ctx).await;
    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time);
    let err = rpc
        .build_transaction(
            ctx.session,
            recipient,
            1,
            Some(hex::encode(asset_definition_id)),
        )
        .await
        .unwrap_err();

    assert_eq!(err.code(), -32602);
    assert!(err.message().contains("No spendable assets available"));
}

#[tokio::test]
async fn test_tx_send_rejects_locks() {
    let time = mock_time_with_offset(10);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 50_000, 7).await;

    let claimed = ctx
        .service
        .list_claimed_assets(&ctx.wallet_id)
        .await
        .expect("claimed assets");
    let asset_definition_id = claimed[0].definition.id;
    for (offset, asset) in claimed.iter().enumerate() {
        let lock = test_mandate_lock_payload(
            ctx.wallet_id.clone(),
            100_u8.saturating_add(offset as u8),
            asset.asset_id(),
            asset.amount,
        );
        ctx.service
            .put_owned_object_for_tests(&ctx.wallet_id, OwnedObjectPayload::Right(lock))
            .await
            .expect("validator lock insert must succeed");
    }

    let recipient = mk_recv_card_compact(&ctx).await;
    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time);
    let err = rpc
        .send_transaction(
            ctx.session,
            recipient,
            1,
            Some(hex::encode(asset_definition_id)),
            None,
            None,
            Some(ms(BASE_TIME_SECS.saturating_add(10))),
        )
        .await
        .unwrap_err();

    assert_eq!(err.code(), -32602);
    assert!(err.message().contains("No spendable assets available"));
}

#[tokio::test]
async fn test_tx_build_keeps_assets() {
    let time = mock_time_with_offset(10);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 50_000, 7).await;
    seed_spendable_stealth_coin(&ctx, 75_000, 8).await;

    let claimed = ctx
        .service
        .list_claimed_assets(&ctx.wallet_id)
        .await
        .expect("claimed assets");
    assert!(
        claimed.len() >= 2,
        "fixture must expose at least one locked and one unlocked asset"
    );

    let locked_asset = claimed[0].clone();
    let unlocked_asset = claimed
        .iter()
        .find(|asset| asset.asset_id() != locked_asset.asset_id())
        .expect("unlocked asset")
        .clone();
    let lock = test_mandate_lock_payload(
        ctx.wallet_id.clone(),
        110,
        locked_asset.asset_id(),
        locked_asset.amount,
    );
    ctx.service
        .put_owned_object_for_tests(&ctx.wallet_id, OwnedObjectPayload::Right(lock))
        .await
        .expect("validator lock insert must succeed");

    let spendable = ctx
        .service
        .list_spendable_asset_rows(&ctx.wallet_id, None)
        .await
        .expect("spendable rows");
    let spendable_ids = spendable
        .iter()
        .map(|row| row.asset_id)
        .collect::<std::collections::BTreeSet<_>>();
    assert!(
        !spendable_ids.contains(&locked_asset.asset_id()),
        "locked asset must be removed from ordinary spend selection"
    );
    assert!(
        spendable_ids.contains(&unlocked_asset.asset_id()),
        "unrelated asset must remain selectable"
    );

    let recipient = mk_recv_card_compact(&ctx).await;
    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time);
    let build = rpc
        .build_transaction(
            ctx.session,
            recipient,
            1,
            Some(hex::encode(unlocked_asset.definition.id)),
        )
        .await
        .expect("unlocked asset definition must still build");

    assert!(!build.raw_tx.is_empty());
}

#[tokio::test]
async fn test_reserves_asset_until_cancel() {
    let time = mock_time_with_offset(11);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 50_000, 7).await;
    let asset_rpc = crate::rpc::methods::AssetRpcImpl::with_dependencies_and_wallet_service(
        time.clone(),
        Arc::clone(&ctx.service),
    );
    let original_asset_id = ctx
        .service
        .list_claimed_assets(&ctx.wallet_id)
        .await
        .expect("claimed assets")
        .into_iter()
        .next()
        .expect("seeded asset")
        .asset_id();
    let recipient = mk_recv_card_compact(&ctx).await;
    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time);

    let build = rpc
        .build_transaction(ctx.session.clone(), recipient, 123, None)
        .await
        .expect("build transaction");

    let pending = crate::rpc::methods::AssetRpcServer::get_asset_balance(
        &asset_rpc,
        ctx.wallet_id.clone(),
        original_asset_id,
    )
    .await
    .expect("pending balance");
    assert_eq!(pending.pending, pending.total);
    assert_eq!(pending.available, 0);

    rpc.cancel_transaction(ctx.session.clone(), build.tx_id)
        .await
        .expect("cancel transaction");

    let released = crate::rpc::methods::AssetRpcServer::get_asset_balance(
        &asset_rpc,
        ctx.wallet_id.clone(),
        original_asset_id,
    )
    .await
    .expect("released balance");
    assert_eq!(released.pending, 0);
    assert_eq!(released.available, released.total);
}

#[tokio::test]
async fn test_tx_build_rolls_back() {
    struct FailPutStore;

    impl crate::persistence::TxStorage for FailPutStore {
        fn put(
            &mut self,
            _record: crate::persistence::TxRecord,
        ) -> crate::persistence::TxStorageResult<()> {
            Err(crate::persistence::TxStorageError::Database(
                "forced put failure".to_string(),
            ))
        }

        fn record_imported(
            &mut self,
            record: crate::persistence::TxRecord,
        ) -> crate::persistence::TxStorageResult<()> {
            self.put(record)
        }

        fn record_exported(
            &mut self,
            _tx_hash: &str,
        ) -> crate::persistence::TxStorageResult<()> {
            Ok(())
        }

        fn get(
            &self,
            tx_hash: &str,
        ) -> crate::persistence::TxStorageResult<crate::persistence::TxRecord> {
            Err(crate::persistence::TxStorageError::NotFound(
                tx_hash.to_string(),
            ))
        }

        fn list(
            &self,
        ) -> crate::persistence::TxStorageResult<Vec<crate::persistence::TxRecord>>
        {
            Ok(Vec::new())
        }

        fn list_history_rows(
            &self,
        ) -> crate::persistence::TxStorageResult<
            Vec<crate::backup::WalletTxHistoryJsonlEntry>,
        > {
            Ok(Vec::new())
        }

        fn list_by_status(
            &self,
            _status: crate::persistence::TxStatus,
        ) -> crate::persistence::TxStorageResult<Vec<crate::persistence::TxRecord>>
        {
            Ok(Vec::new())
        }

        fn update_status(
            &mut self,
            _tx_hash: &str,
            _status: crate::persistence::TxStatus,
        ) -> crate::persistence::TxStorageResult<()> {
            Ok(())
        }

        fn record_submitted(
            &mut self,
            tx_hash: &str,
        ) -> crate::persistence::TxStorageResult<()> {
            Err(crate::persistence::TxStorageError::NotFound(
                tx_hash.to_string(),
            ))
        }

        fn record_admitted(
            &mut self,
            tx_hash: &str,
        ) -> crate::persistence::TxStorageResult<()> {
            Err(crate::persistence::TxStorageError::NotFound(
                tx_hash.to_string(),
            ))
        }

        fn record_confirmed(
            &mut self,
            tx_hash: &str,
            _block_height: u64,
        ) -> crate::persistence::TxStorageResult<()> {
            Err(crate::persistence::TxStorageError::NotFound(
                tx_hash.to_string(),
            ))
        }

        fn record_cancelled(
            &mut self,
            tx_hash: &str,
        ) -> crate::persistence::TxStorageResult<()> {
            Err(crate::persistence::TxStorageError::NotFound(
                tx_hash.to_string(),
            ))
        }

        fn record_conflicted(
            &mut self,
            tx_hash: &str,
        ) -> crate::persistence::TxStorageResult<()> {
            Err(crate::persistence::TxStorageError::NotFound(
                tx_hash.to_string(),
            ))
        }

        fn record_already_spent(
            &mut self,
            tx_hash: &str,
        ) -> crate::persistence::TxStorageResult<()> {
            Err(crate::persistence::TxStorageError::NotFound(
                tx_hash.to_string(),
            ))
        }

        fn delete(
            &mut self,
            _tx_hash: &str,
        ) -> crate::persistence::TxStorageResult<()> {
            Ok(())
        }
    }

    let time = mock_time_with_offset(12);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 50_000, 7).await;
    let asset_rpc = crate::rpc::methods::AssetRpcImpl::with_dependencies_and_wallet_service(
        time.clone(),
        Arc::clone(&ctx.service),
    );
    let asset_ids = ctx
        .service
        .list_claimed_assets(&ctx.wallet_id)
        .await
        .expect("claimed assets")
        .into_iter()
        .map(|asset| asset.asset_id())
        .collect::<Vec<_>>();
    let tx_store: Arc<
        tokio::sync::RwLock<
            Box<dyn crate::persistence::TxStorage + Send + Sync>,
        >,
    > = Arc::new(tokio::sync::RwLock::new(Box::new(FailPutStore)));
    let rpc = TxRpcImpl::with_dependencies_and_tx_store(
        Arc::clone(&ctx.service),
        time,
        tx_store,
    );
    let recipient = mk_recv_card_compact(&ctx).await;

    let err = rpc
        .build_transaction(ctx.session.clone(), recipient, 123, None)
        .await
        .unwrap_err();
    assert_eq!(err.code(), -32603);

    for asset_id in asset_ids {
        let balance = crate::rpc::methods::AssetRpcServer::get_asset_balance(
            &asset_rpc,
            ctx.wallet_id.clone(),
            asset_id,
        )
        .await
        .expect("balance after rollback");
        assert_eq!(balance.pending, 0);
        assert_eq!(balance.available, balance.total);
    }
}

#[tokio::test]
async fn test_tx_build_skips_pending() {
    let time = mock_time_with_offset(13);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 50_000, 7).await;
    let recipient = mk_recv_card_compact(&ctx).await;
    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time);

    let first = rpc
        .build_transaction(ctx.session.clone(), recipient.clone(), 123, None)
        .await
        .expect("first build");
    let first_summary =
        crate::rpc::methods::tx_runtime_state::tx_package_summary(first.raw_tx.as_bytes())
            .expect("first summary");

    let second = rpc
        .build_transaction(ctx.session.clone(), recipient, 123, None)
        .await
        .expect("second build");
    let second_summary =
        crate::rpc::methods::tx_runtime_state::tx_package_summary(second.raw_tx.as_bytes())
            .expect("second summary");

    assert!(first_summary
        .inputs
        .iter()
        .all(|asset_id| !second_summary.inputs.contains(asset_id)));
}

#[tokio::test]
async fn test_build_uses_injected_rng() {
    let first = {
        let time_a = mock_time_with_offset(14);
        let ctx_a = setup_session_with_rng(time_a.clone(), SeqTxTestRng::new(77)).await;
        seed_spendable_stealth_coin(&ctx_a, 50_000, 7).await;
        let recv_keys_a = ctx_a
            .service
            .receiver_keys(&ctx_a.wallet_id)
            .await
            .expect("receiver keys A");
        let spendable_a = ctx_a
            .service
            .list_spendable_asset_rows(&ctx_a.wallet_id, None)
            .await
            .expect("spendable rows A")
            .into_iter()
            .map(|asset| hex::encode(asset.asset_id))
            .collect::<Vec<_>>();
        let recipient_a = mk_recv_card_compact(&ctx_a).await;
        let rpc_a = TxRpcImpl::with_dependencies(Arc::clone(&ctx_a.service), time_a);

        (
            hex::encode(recv_keys_a.owner_handle),
            spendable_a,
            rpc_a
                .build_transaction(ctx_a.session.clone(), recipient_a, 123, None)
                .await
                .expect("first deterministic build"),
        )
    };

    let second = {
        let time_b = mock_time_with_offset(14);
        let ctx_b = setup_session_with_rng(time_b.clone(), SeqTxTestRng::new(77)).await;
        seed_spendable_stealth_coin(&ctx_b, 50_000, 7).await;
        let recv_keys_b = ctx_b
            .service
            .receiver_keys(&ctx_b.wallet_id)
            .await
            .expect("receiver keys B");
        let spendable_b = ctx_b
            .service
            .list_spendable_asset_rows(&ctx_b.wallet_id, None)
            .await
            .expect("spendable rows B")
            .into_iter()
            .map(|asset| hex::encode(asset.asset_id))
            .collect::<Vec<_>>();
        let recipient_b = mk_recv_card_compact(&ctx_b).await;
        let rpc_b = TxRpcImpl::with_dependencies(Arc::clone(&ctx_b.service), time_b);

        (
            hex::encode(recv_keys_b.owner_handle),
            spendable_b,
            rpc_b
                .build_transaction(ctx_b.session.clone(), recipient_b, 123, None)
                .await
                .expect("second deterministic build"),
        )
    };

    let (owner_handle_a, spendable_a, first) = first;
    let (owner_handle_b, spendable_b, second) = second;

    if owner_handle_a != owner_handle_b || spendable_a != spendable_b {
        println!("owner_handle_a: {owner_handle_a}");
        println!("owner_handle_b: {owner_handle_b}");
        println!("spendable_a: {:?}", spendable_a);
        println!("spendable_b: {:?}", spendable_b);
    }

    assert_eq!(owner_handle_a, owner_handle_b);
    assert_eq!(spendable_a, spendable_b);

    if first.tx_id != second.tx_id || first.raw_tx != second.raw_tx {
        let pkg_a: crate::tx::TxPackage = JsonCodec
            .deserialize(first.raw_tx.as_bytes())
            .expect("package A");
        let pkg_b: crate::tx::TxPackage = JsonCodec
            .deserialize(second.raw_tx.as_bytes())
            .expect("package B");
        let spend_a = pkg_a.tx.proof.spend.as_ref().expect("spend proof A");
        let spend_b = pkg_b.tx.proof.spend.as_ref().expect("spend proof B");
        let auth_a = pkg_a.tx.auth.spend.as_ref().expect("spend auth A");
        let auth_b = pkg_b.tx.auth.spend.as_ref().expect("spend auth B");
        println!("tx_id_a: {}", first.tx_id.0);
        println!("tx_id_b: {}", second.tx_id.0);
        println!("prev_root_a: {}", spend_a.prev_root_hex);
        println!("prev_root_b: {}", spend_b.prev_root_hex);
        println!("proof_hex_a: {}", spend_a.proof_hex);
        println!("proof_hex_b: {}", spend_b.proof_hex);
        println!("receiver_card_a: {}", auth_a.receiver_card_compact);
        println!("receiver_card_b: {}", auth_b.receiver_card_compact);
        println!("spend_sig_a: {}", auth_a.spend_sig_hex);
        println!("spend_sig_b: {}", auth_b.spend_sig_hex);
    }

    assert_eq!(first.tx_id, second.tx_id);
    assert_eq!(first.raw_tx, second.raw_tx);
}

#[tokio::test]
async fn test_tx_honors_asset_id() {
    let time = mock_time_with_offset(10);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 50_000, 7).await;
    let recipient = mk_recv_card_compact(&ctx).await;
    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time);

    let missing_asset = hex::encode([9u8; 32]);
    let err = rpc
        .send_transaction(
            ctx.session.clone(),
            recipient,
            10,
            Some(missing_asset),
            None,
            None,
            Some(ms(BASE_TIME_SECS.saturating_add(10))),
        )
        .await
        .unwrap_err();

    assert_eq!(err.code(), -32602);
}

#[tokio::test]
async fn test_tx_verify_partial_stealth() {
    let time = mock_time_with_offset(10);
    let ctx = setup_session(time.clone()).await;
    let recv_keys = ctx.service.receiver_keys(&ctx.wallet_id).await.unwrap();
    let scanner = StealthOutputScanner::from_keys(&recv_keys);

    let mut asset =
        z00z_core::genesis::asset_std::asset_from_dev_cfg("z00z", 0, 55).expect("asset");
    asset.r_pub = Some([7u8; 32]);

    let output = crate::tx::TxOutputWire {
        role: crate::tx::TxOutRole::Recipient,
        asset_wire: AssetPkgWire::from_asset(&asset),
    };

    let owned = TxRpcImpl::build_owned_out(&output, &scanner).expect("build owned out");
    assert!(owned.is_none());
}

#[tokio::test]
async fn test_tx_build_limits_20() {
    let time = mock_time_with_offset(10);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 50_000, 7).await;
    let recipient = mk_recv_card_compact(&ctx).await;
    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time.clone());

    for _ in 0..20 {
        rpc.tx_build_precheck(&ctx.wallet_id).await.unwrap();
    }

    let err = rpc
        .build_transaction(ctx.session.clone(), recipient.clone(), 1, None)
        .await
        .unwrap_err();

    assert_eq!(err.code(), SecurityErrorCode::RateLimitExceeded.code());

    time.advance_by(Duration::from_secs(60));
    let _ = rpc
        .build_transaction(ctx.session.clone(), recipient, 1, None)
        .await
        .unwrap();
}
