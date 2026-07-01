#[test]
fn broadcast_retry_reject() {
    let mut attempts = 0u32;
    let (total_attempts, result) = run_with_retry(3, || {
        attempts += 1;
        if attempts == 1 {
            return Err(BroadcastError::Rejected("no retry".to_string()));
        }
        Ok(())
    });

    assert!(result.is_err());
    assert_eq!(total_attempts, 1);
}

#[test]
fn broadcast_retry_transient() {
    let mut attempts = 0u32;
    let (total_attempts, result) = run_with_retry(3, || {
        attempts += 1;
        if attempts <= 2 {
            return Err(BroadcastError::Network("transient".to_string()));
        }
        Ok(())
    });

    assert!(result.is_ok());
    assert_eq!(total_attempts, 3);
}

#[test]
fn broadcast_retry_exhausted() {
    let (total_attempts, result) =
        run_with_retry(2, || Err(BroadcastError::Network("always".to_string())));

    assert!(result.is_err());
    assert_eq!(total_attempts, 3);
}

#[test]
fn broadcast_retry_timeout() {
    let mut attempts = 0u32;
    let (total_attempts, result) = run_with_retry(3, || {
        attempts += 1;
        if attempts <= 2 {
            return Err(BroadcastError::Timeout);
        }
        Ok(())
    });

    assert!(result.is_ok());
    assert_eq!(total_attempts, 3);
}

#[test]
fn broadcast_retry_one_shot() {
    let mut attempts = 0u32;
    let (total_attempts, result) = run_with_retry(0, || {
        attempts += 1;
        Err(BroadcastError::Network("single shot".to_string()))
    });

    assert!(matches!(result, Err(BroadcastError::Network(_))));
    assert_eq!(attempts, 1);
    assert_eq!(total_attempts, 1);
}

#[test]
fn broadcast_retry_rbf_terminal() {
    let (total_attempts, result) =
        run_with_retry(3, || Err(BroadcastError::Replaced("rbf".to_string())));

    assert!(matches!(result, Err(BroadcastError::Replaced(_))));
    assert_eq!(total_attempts, 1);
}

#[test]
fn broadcast_retry_reorg_terminal() {
    let (total_attempts, result) =
        run_with_retry(3, || Err(BroadcastError::Reorg("rollback".to_string())));

    assert!(matches!(result, Err(BroadcastError::Reorg(_))));
    assert_eq!(total_attempts, 1);
}

#[tokio::test]
async fn test_tx_cancel_non_pending() {
    let time = mock_time_with_offset(10);
    let ctx = setup_session(time.clone()).await;

    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time);

    let err = rpc
        .cancel_transaction(
            ctx.session.clone(),
            PersistTxId::new("tx_does_not_exist".to_string()),
        )
        .await
        .unwrap_err();

    assert_eq!(err.code(), -32602);
}

#[tokio::test]
async fn test_tx_cancel_pending_tx() {
    let time = mock_time_with_offset(10);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 50_000, 7).await;
    let recipient = mk_recv_card_compact(&ctx).await;

    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time);

    let build = rpc
        .build_transaction(ctx.session.clone(), recipient, 10, None)
        .await
        .unwrap();

    let broadcast = rpc
        .broadcast_transaction(ctx.session.clone(), build.raw_tx)
        .await
        .unwrap();

    let before = rpc
        .list_pending_transactions(
            ctx.session.clone(),
            RuntimePaginationParams {
                limit: Some(50),
                cursor: None,
                include_total: Some(true),
            },
        )
        .await
        .unwrap();
    assert_eq!(before.items.len(), 1);

    let resp = rpc
        .cancel_transaction(ctx.session.clone(), broadcast.tx_id.clone())
        .await
        .unwrap();
    assert!(resp.status.success);

    let after = rpc
        .list_pending_transactions(
            ctx.session.clone(),
            RuntimePaginationParams {
                limit: Some(50),
                cursor: None,
                include_total: Some(true),
            },
        )
        .await
        .unwrap();
    assert_eq!(after.items.len(), 0);
}

#[tokio::test]
async fn tx_admits_pending() {
    let time = mock_time_with_offset(11);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 50_000, 7).await;
    let recipient = mk_recv_card_compact(&ctx).await;

    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time);

    let build = rpc
        .build_transaction(ctx.session.clone(), recipient, 10, None)
        .await
        .unwrap();
    let broadcast = rpc
        .broadcast_transaction(ctx.session.clone(), build.raw_tx)
        .await
        .unwrap();

    let evidence = rpc
        .load_confirmation_evidence(&broadcast.tx_id)
        .await
        .expect("broadcast must expose typed confirmation evidence");
    assert_eq!(evidence.tx_id, broadcast.tx_id.0);
    assert!(evidence.verified);

    let details = rpc
        .get_transaction_details(ctx.session.clone(), broadcast.tx_id)
        .await
        .unwrap();
    assert!(matches!(details.status, TxStatus::Pending));
    assert!(details.receipt.is_none());
    assert_eq!(
        tx_history_kinds(&ctx),
        vec![
            WalletTxHistoryEntryKind::Created,
            WalletTxHistoryEntryKind::Submitted,
            WalletTxHistoryEntryKind::Admitted,
        ]
    );
}

#[tokio::test]
async fn test_imported_receiver_submitter_role() {
    let time = mock_time_with_offset(12);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 50_000, 7).await;
    let recipient = mk_recv_card_compact(&ctx).await;

    let sender_rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time.clone());
    let requests = Arc::new(Mutex::new(Vec::new()));
    let receiver_rpc = rpc_with_recording_admitter(
        Arc::clone(&ctx.service),
        time.clone(),
        Arc::clone(&requests),
    );

    let tx_id = sender_rpc
        .send_transaction(
            ctx.session.clone(),
            recipient,
            10,
            None,
            None,
            Some(IdempotencyKey(
                "12121212121212121212121212121212".to_string(),
            )),
            Some(ms(BASE_TIME_SECS.saturating_add(12))),
        )
        .await
        .unwrap()
        .tx_id;

    let export = sender_rpc
        .export_transaction(ctx.session.clone(), tx_id.clone())
        .await
        .unwrap();
    let path = export.export_path.expect("export path");
    let contents = z00z_utils::io::read_to_string(&path).unwrap();

    let imported = receiver_rpc
        .import_transaction(ctx.session.clone(), contents.clone())
        .await
        .unwrap();
    assert_eq!(imported.tx_id, tx_id);

    let broadcast = receiver_rpc
        .broadcast_transaction(ctx.session.clone(), portable_tx_bytes_from_export(&contents))
        .await
        .unwrap();
    assert_eq!(broadcast.tx_id, tx_id);

    let captured = requests.lock().expect("captured requests");
    let last = captured.last().expect("captured admission request");
    assert_eq!(last.tx_id, tx_id);
    assert_eq!(last.submitter_role, TxSubmitterRole::Receiver);
}
