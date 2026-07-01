#[tokio::test]
async fn test_tx_get_sorts_timestamp() {
    let time = mock_time_with_offset(500);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 200_000, 7).await;
    let recipient_a = mk_recv_card_compact(&ctx).await;
    let recipient_b = mk_recv_card_compact(&ctx).await;
    let recipient_c = mk_recv_card_compact(&ctx).await;

    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time.clone());

    let tx_10 = rpc
        .send_transaction(
            ctx.session.clone(),
            recipient_a,
            10,
            None,
            None,
            Some(IdempotencyKey(
                "dddddddddddddddddddddddddddddddd".to_string(),
            )),
            Some(ms(BASE_TIME_SECS.saturating_add(500))),
        )
        .await
        .unwrap()
        .tx_id;

    time.advance_by(Duration::from_secs(1));

    let tx_30 = rpc
        .send_transaction(
            ctx.session.clone(),
            recipient_b,
            30,
            None,
            None,
            Some(IdempotencyKey(
                "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".to_string(),
            )),
            Some(ms(BASE_TIME_SECS.saturating_add(501))),
        )
        .await
        .unwrap()
        .tx_id;

    time.advance_by(Duration::from_secs(1));

    let tx_20 = rpc
        .send_transaction(
            ctx.session.clone(),
            recipient_c,
            20,
            None,
            None,
            Some(IdempotencyKey(
                "ffffffffffffffffffffffffffffffff".to_string(),
            )),
            Some(ms(BASE_TIME_SECS.saturating_add(502))),
        )
        .await
        .unwrap()
        .tx_id;

    let ts_asc = rpc
        .get_transaction_history(
            ctx.session.clone(),
            RuntimePaginationParams {
                limit: Some(50),
                cursor: None,
                include_total: Some(true),
            },
            None,
            Some(RuntimeTxHistorySort {
                by: TxHistorySortBy::Timestamp,
                direction: SortDirection::Asc,
            }),
        )
        .await
        .unwrap();

    assert_eq!(ts_asc.total_count, Some(3));
    assert_eq!(ts_asc.items[0].id, tx_10);
    assert_eq!(ts_asc.items[1].id, tx_30);
    assert_eq!(ts_asc.items[2].id, tx_20);

    let amount_asc = rpc
        .get_transaction_history(
            ctx.session.clone(),
            RuntimePaginationParams {
                limit: Some(50),
                cursor: None,
                include_total: Some(true),
            },
            None,
            Some(RuntimeTxHistorySort {
                by: TxHistorySortBy::Amount,
                direction: SortDirection::Asc,
            }),
        )
        .await
        .unwrap();

    assert_eq!(amount_asc.items[0].amount, 10);
    assert_eq!(amount_asc.items[1].amount, 20);
    assert_eq!(amount_asc.items[2].amount, 30);

    let amount_desc = rpc
        .get_transaction_history(
            ctx.session.clone(),
            RuntimePaginationParams {
                limit: Some(50),
                cursor: None,
                include_total: Some(true),
            },
            None,
            Some(RuntimeTxHistorySort {
                by: TxHistorySortBy::Amount,
                direction: SortDirection::Desc,
            }),
        )
        .await
        .unwrap();

    assert_eq!(amount_desc.items[0].amount, 30);
    assert_eq!(amount_desc.items[1].amount, 20);
    assert_eq!(amount_desc.items[2].amount, 10);
}

#[tokio::test]
async fn test_tx_history_includes_receipt() {
    let time = mock_time_with_offset(10);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 50_000, 7).await;
    let recipient = mk_recv_card_compact(&ctx).await;

    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time);

    let _ = rpc
        .send_transaction(
            ctx.session.clone(),
            recipient,
            10,
            None,
            None,
            Some(IdempotencyKey(
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            )),
            Some(ms(BASE_TIME_SECS.saturating_add(10))),
        )
        .await
        .unwrap();

    let page = rpc
        .get_transaction_history(
            ctx.session.clone(),
            RuntimePaginationParams {
                limit: Some(1),
                cursor: None,
                include_total: Some(true),
            },
            None,
            None,
        )
        .await
        .unwrap();

    let json = JsonCodec
        .serialize(&page)
        .and_then(|bytes| JsonCodec.deserialize::<z00z_utils::codec::Value>(&bytes))
        .unwrap();
    let items = json
        .get("items")
        .and_then(|value| value.as_array())
        .expect("items array");

    assert_eq!(items.len(), 1);

    let first = items[0].as_object().expect("tx item object");
    assert!(first.contains_key("receipt"));
    assert!(first.get("receipt").expect("receipt").is_null());
}

#[tokio::test]
async fn test_tx_get_includes_receipt() {
    let time = mock_time_with_offset(1000);
    let ctx = setup_session(time.clone()).await;
    seed_spendable_stealth_coin(&ctx, 50_000, 7).await;
    let recipient = mk_recv_card_compact(&ctx).await;

    let rpc = TxRpcImpl::with_dependencies(Arc::clone(&ctx.service), time);

    let tx_id = rpc
        .send_transaction(
            ctx.session.clone(),
            recipient,
            10,
            None,
            None,
            Some(IdempotencyKey(
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            )),
            Some(ms(BASE_TIME_SECS.saturating_add(1000))),
        )
        .await
        .unwrap()
        .tx_id;

    let details = rpc
        .get_transaction_details(ctx.session.clone(), tx_id.clone())
        .await
        .unwrap();

    assert_eq!(details.tx_id, tx_id);
    assert!(details.receipt.is_none());
    assert!(!details.receipt_verified);
    assert_eq!(details.lifecycle, RuntimeTxLifecycle::Admitted);

    let json = JsonCodec
        .serialize(&details)
        .and_then(|bytes| JsonCodec.deserialize::<z00z_utils::codec::Value>(&bytes))
        .unwrap();
    let obj = json.as_object().expect("details object");
    assert!(obj.contains_key("lifecycle"));
    assert!(obj.contains_key("receipt"));
    assert!(obj.get("receipt").expect("receipt").is_null());
    assert!(obj.contains_key("receipt_verified"));
}
