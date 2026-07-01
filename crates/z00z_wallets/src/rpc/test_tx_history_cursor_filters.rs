#[tokio::test]
async fn test_tx_get_paginates_cursor() {
    let time = mock_time_with_offset(10);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 200_000, 7).await;
    let recipient = mk_recv_card_compact(&ctx).await;

    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time.clone());

    let build1 = rpc
        .build_transaction(ctx.session.clone(), recipient.clone(), 10, None)
        .await
        .unwrap();

    let tx1 = rpc
        .broadcast_transaction(ctx.session.clone(), build1.raw_tx)
        .await
        .unwrap()
        .tx_id;
    time.advance_by(Duration::from_secs(1));

    let build2 = rpc
        .build_transaction(ctx.session.clone(), recipient.clone(), 20, None)
        .await
        .unwrap();

    let tx2 = rpc
        .broadcast_transaction(ctx.session.clone(), build2.raw_tx)
        .await
        .unwrap()
        .tx_id;
    time.advance_by(Duration::from_secs(1));

    let build3 = rpc
        .build_transaction(ctx.session.clone(), recipient, 30, None)
        .await
        .unwrap();

    let tx3 = rpc
        .broadcast_transaction(ctx.session.clone(), build3.raw_tx)
        .await
        .unwrap()
        .tx_id;

    let page1 = rpc
        .get_transaction_history(
            ctx.session.clone(),
            RuntimePaginationParams {
                limit: Some(2),
                cursor: None,
                include_total: Some(true),
            },
            None,
            None,
        )
        .await
        .unwrap();

    assert_eq!(page1.items.len(), 2);
    assert!(page1.has_more);
    assert_eq!(page1.total_count, Some(3));
    assert_eq!(page1.items[0].id, tx3);
    assert_eq!(page1.items[1].id, tx2);

    let cursor = page1.next_cursor.clone().expect("next_cursor");
    assert_eq!(cursor, tx2.0);

    let page2 = rpc
        .get_transaction_history(
            ctx.session.clone(),
            RuntimePaginationParams {
                limit: Some(999),
                cursor: Some(cursor),
                include_total: Some(false),
            },
            None,
            None,
        )
        .await
        .unwrap();

    assert_eq!(page2.items.len(), 1);
    assert!(!page2.has_more);
    assert_eq!(page2.items[0].id, tx1);
    assert!(page2.next_cursor.is_none());
}

#[tokio::test]
async fn test_tx_get_rejects_invalid() {
    let time = mock_time_with_offset(10);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 50_000, 7).await;
    let recipient = mk_recv_card_compact(&ctx).await;

    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time);
    let build = rpc
        .build_transaction(ctx.session.clone(), recipient, 10, None)
        .await
        .unwrap();
    let _ = rpc
        .broadcast_transaction(ctx.session.clone(), build.raw_tx)
        .await
        .unwrap();

    let err = rpc
        .get_transaction_history(
            ctx.session.clone(),
            RuntimePaginationParams {
                limit: Some(1),
                cursor: Some("does-not-exist".to_string()),
                include_total: None,
            },
            None,
            None,
        )
        .await
        .unwrap_err();

    assert_eq!(err.code(), -32602);
}

#[tokio::test]
async fn test_tx_get_filters_status() {
    let time = mock_time_with_offset(100);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 200_000, 7).await;
    let recipient_a = mk_recv_card_compact(&ctx).await;
    let recipient_b = mk_recv_card_compact(&ctx).await;
    let recipient_c = mk_recv_card_compact(&ctx).await;

    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time.clone());

    let tx_a = rpc
        .send_transaction(
            ctx.session.clone(),
            recipient_a,
            10,
            None,
            None,
            Some(IdempotencyKey(
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            )),
            Some(ms(BASE_TIME_SECS.saturating_add(100))),
        )
        .await
        .unwrap()
        .tx_id;

    time.advance_by(Duration::from_secs(1));

    let tx_b = rpc
        .send_transaction(
            ctx.session.clone(),
            recipient_b,
            20,
            None,
            None,
            Some(IdempotencyKey(
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            )),
            Some(ms(BASE_TIME_SECS.saturating_add(101))),
        )
        .await
        .unwrap()
        .tx_id;

    time.advance_by(Duration::from_secs(1));

    let tx_c = rpc
        .send_transaction(
            ctx.session.clone(),
            recipient_c,
            30,
            None,
            None,
            Some(IdempotencyKey(
                "cccccccccccccccccccccccccccccccc".to_string(),
            )),
            Some(ms(BASE_TIME_SECS.saturating_add(102))),
        )
        .await
        .unwrap()
        .tx_id;

    let _ = rpc
        .cancel_transaction(ctx.session.clone(), tx_b.clone())
        .await
        .unwrap();

    let cancelled_only = rpc
        .get_transaction_history(
            ctx.session.clone(),
            RuntimePaginationParams {
                limit: Some(50),
                cursor: None,
                include_total: Some(true),
            },
            Some(RuntimeTxHistoryFilter {
                from_date: None,
                to_date: None,
                status: Some(TxStatus::Cancelled),
                min_amount: None,
                max_amount: None,
            }),
            None,
        )
        .await
        .unwrap();

    assert_eq!(cancelled_only.total_count, Some(1));
    assert_eq!(cancelled_only.items.len(), 1);
    assert_eq!(cancelled_only.items[0].id, tx_b);

    let amount_filtered = rpc
        .get_transaction_history(
            ctx.session.clone(),
            RuntimePaginationParams {
                limit: Some(50),
                cursor: None,
                include_total: Some(true),
            },
            Some(RuntimeTxHistoryFilter {
                from_date: None,
                to_date: None,
                status: None,
                min_amount: Some(15),
                max_amount: Some(35),
            }),
            None,
        )
        .await
        .unwrap();

    assert_eq!(amount_filtered.total_count, Some(2));
    assert!(amount_filtered.items.iter().any(|tx| tx.id == tx_c));
    assert!(amount_filtered.items.iter().any(|tx| tx.id == tx_b));

    let date_filtered = rpc
        .get_transaction_history(
            ctx.session.clone(),
            RuntimePaginationParams {
                limit: Some(50),
                cursor: None,
                include_total: Some(true),
            },
            Some(RuntimeTxHistoryFilter {
                from_date: Some(ms(BASE_TIME_SECS.saturating_add(100))),
                to_date: Some(ms(BASE_TIME_SECS.saturating_add(101))),
                status: None,
                min_amount: None,
                max_amount: None,
            }),
            None,
        )
        .await
        .unwrap();

    assert_eq!(date_filtered.total_count, Some(2));
    assert!(date_filtered.items.iter().any(|tx| tx.id == tx_a));
    assert!(date_filtered.items.iter().any(|tx| tx.id == tx_b));
}

#[tokio::test]
async fn test_tx_get_rejects_filter() {
    let time = mock_time_with_offset(100);
    let ctx = setup_session(time.clone()).await;

    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time);

    let err = rpc
        .get_transaction_history(
            ctx.session.clone(),
            RuntimePaginationParams {
                limit: Some(1),
                cursor: None,
                include_total: None,
            },
            Some(RuntimeTxHistoryFilter {
                from_date: Some(ms(BASE_TIME_SECS.saturating_add(200))),
                to_date: Some(ms(BASE_TIME_SECS.saturating_add(100))),
                status: None,
                min_amount: None,
                max_amount: None,
            }),
            None,
        )
        .await
        .unwrap_err();

    assert_eq!(err.code(), -32602);

    let err = rpc
        .get_transaction_history(
            ctx.session.clone(),
            RuntimePaginationParams {
                limit: Some(1),
                cursor: None,
                include_total: None,
            },
            Some(RuntimeTxHistoryFilter {
                from_date: None,
                to_date: None,
                status: None,
                min_amount: Some(10),
                max_amount: Some(1),
            }),
            None,
        )
        .await
        .unwrap_err();

    assert_eq!(err.code(), -32602);
}
