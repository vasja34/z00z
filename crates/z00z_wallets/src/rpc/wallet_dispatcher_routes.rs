#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub(crate) enum PrivRouteGuard {
    Touch,
    NoTouch,
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub(crate) struct PrivRouteSpec {
    pub(crate) rpc: &'static str,
    pub(crate) guard: PrivRouteGuard,
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub(crate) const PRIV_ROUTE_SPECS: &[PrivRouteSpec] = &[
    PrivRouteSpec {
        rpc: "wallet.session.lock_wallet",
        guard: PrivRouteGuard::NoTouch,
    },
    PrivRouteSpec {
        rpc: "wallet.session.show_seed_phrase",
        guard: PrivRouteGuard::NoTouch,
    },
    PrivRouteSpec {
        rpc: "wallet.backup.create_backup",
        guard: PrivRouteGuard::Touch,
    },
    PrivRouteSpec {
        rpc: "wallet.backup.list_backups",
        guard: PrivRouteGuard::Touch,
    },
    PrivRouteSpec {
        rpc: "wallet.backup.configure_backup",
        guard: PrivRouteGuard::Touch,
    },
    PrivRouteSpec {
        rpc: "wallet.key.derive_receiver",
        guard: PrivRouteGuard::NoTouch,
    },
    PrivRouteSpec {
        rpc: "wallet.key.get_receiver_card",
        guard: PrivRouteGuard::Touch,
    },
    PrivRouteSpec {
        rpc: "wallet.key.create_payment_request",
        guard: PrivRouteGuard::Touch,
    },
    PrivRouteSpec {
        rpc: "wallet.key.validate_payment_request",
        guard: PrivRouteGuard::Touch,
    },
    PrivRouteSpec {
        rpc: "wallet.key.export_public_material",
        guard: PrivRouteGuard::Touch,
    },
    PrivRouteSpec {
        rpc: "wallet.key.rotate_master_key",
        guard: PrivRouteGuard::NoTouch,
    },
    PrivRouteSpec {
        rpc: "wallet.key.list_receivers",
        guard: PrivRouteGuard::Touch,
    },
    PrivRouteSpec {
        rpc: "wallet.key.label_receiver",
        guard: PrivRouteGuard::Touch,
    },
];

#[cfg(not(target_arch = "wasm32"))]
pub fn register_wallet_methods(dispatcher: &RpcDispatcher, rpc: Arc<WalletRpcImpl>) {
    dispatcher.register_typed(
        "wallet.session.lock_wallet",
        typed_handler_cap(
            Arc::clone(&rpc),
            |rpc, session| async move { rpc.verify_no_touch_cap(session).await },
            |rpc, cap, _p: NoArgs| async move { rpc.lock_wallet_checked(cap).await },
        ),
    );

    dispatcher.register_typed(
        "wallet.session.show_seed_phrase",
        typed_handler_cap(
            Arc::clone(&rpc),
            |rpc, session| async move { rpc.verify_no_touch_cap(session).await },
            |rpc, cap, p: WalletShowSeedPhraseParams| async move {
                rpc.show_seed_phrase_checked(cap, p.password, p.confirmation)
                    .await
            },
        ),
    );

    dispatcher.register_method(
        "wallet.session.unlock_wallet",
        json_typed_handler_jsonrpsee_err(
            Arc::clone(&rpc),
            |rpc, p: WalletIdPasswordParams| async move {
                rpc.unlock_wallet(p.wallet_id, p.password).await
            },
        ),
    );

    dispatcher.register_method(
        "wallet.lifecycle.on_event",
        json_typed_handler_jsonrpsee_err(
            Arc::clone(&rpc),
            |rpc, p: WalletLifecycleParams| async move { rpc.on_lifecycle_event(p.event).await },
        ),
    );
}

/// Register storage.* RPC methods.
#[cfg(not(target_arch = "wasm32"))]
pub fn register_storage_methods(dispatcher: &RpcDispatcher, rpc: Arc<StorageRpcImpl>) {
    dispatcher.register_typed(
        "wallet.storage.compact_storage",
        typed_handler_jsonrpsee_err(
            Arc::clone(&rpc),
            |rpc, p: RuntimeCompactStorageParams| async move { rpc.compact_storage(p).await },
        ),
    );

    dispatcher.register_typed(
        "wallet.storage.get_storage_stats",
        typed_handler_jsonrpsee_err(
            Arc::clone(&rpc),
            |rpc, p: RuntimeGetStorageStatsParams| async move { rpc.get_storage_stats(p).await },
        ),
    );

    dispatcher.register_typed(
        "wallet.storage.export_storage",
        typed_handler_jsonrpsee_err(
            Arc::clone(&rpc),
            |rpc, p: RuntimeExportStorageParams| async move { rpc.export_storage(p).await },
        ),
    );
}

#[cfg(not(target_arch = "wasm32"))]
pub fn register_backup_methods(dispatcher: &RpcDispatcher, rpc: Arc<BackupRpcImpl>) {
    dispatcher.register_typed(
        "wallet.backup.create_backup",
        typed_handler_cap(
            Arc::clone(&rpc),
            |rpc, session| async move { rpc.verify_touch_cap(session).await },
            |rpc, cap, p: BackupCreateParams| async move {
                rpc.create_backup_checked(cap, p.password, p.destination)
                .await
            },
        ),
    );

    dispatcher.register_typed(
        "wallet.backup.list_backups",
        typed_handler_cap(
            Arc::clone(&rpc),
            |rpc, session| async move { rpc.verify_touch_cap(session).await },
            |rpc, cap, p: BackupListParams| async move {
                rpc.list_backups_checked(cap, p.cursor, p.limit).await
            },
        ),
    );

    dispatcher.register_typed(
        "wallet.backup.restore_backup",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: BackupRestoreParams| async move {
            rpc.restore_backup(p.backup_path, p.password, p.wallet_name)
                .await
        }),
    );

    dispatcher.register_typed(
        "wallet.backup.configure_backup",
        typed_handler_cap(
            Arc::clone(&rpc),
            |rpc, session| async move { rpc.verify_touch_cap(session).await },
            |rpc, cap, p: BackupConfigureParams| async move {
                rpc.configure_backup_checked(cap, p.settings).await
            },
        ),
    );
}

#[cfg(not(target_arch = "wasm32"))]
pub fn register_key_methods(dispatcher: &RpcDispatcher, rpc: Arc<KeyRpcImpl>) {
    dispatcher.register_typed(
        "wallet.key.derive_receiver",
        typed_handler_cap(
            Arc::clone(&rpc),
            |rpc, session| async move { rpc.verify_no_touch_cap(session).await },
            |rpc, cap, p: KeyDeriveParams| async move {
                rpc.derive_receiver_checked(cap, p.path).await
            },
        ),
    );

    dispatcher.register_typed(
        "wallet.key.get_receiver_card",
        typed_handler_cap(
            Arc::clone(&rpc),
            |rpc, session| async move { rpc.verify_touch_cap(session).await },
            |rpc, cap, _p: KeyCardParams| async move { rpc.get_receiver_card_checked(cap).await },
        ),
    );

    dispatcher.register_typed(
        "wallet.key.create_payment_request",
        typed_handler_cap(
            Arc::clone(&rpc),
            |rpc, session| async move { rpc.verify_touch_cap(session).await },
            |rpc, cap, p: KeyCreatePaymentRequestParams| async move {
                rpc.create_payment_request_checked(cap, p.amount, p.expiry_secs, p.metadata)
                    .await
            },
        ),
    );

    dispatcher.register_typed(
        "wallet.key.validate_payment_request",
        typed_handler_cap(
            Arc::clone(&rpc),
            |rpc, session| async move { rpc.verify_touch_cap(session).await },
            |rpc, cap, p: KeyValidatePaymentRequestParams| async move {
                rpc.validate_payment_request_checked(cap, p.request_compact)
                    .await
            },
        ),
    );

    dispatcher.register_typed(
        "wallet.key.export_public_material",
        typed_handler_cap(
            Arc::clone(&rpc),
            |rpc, session| async move { rpc.verify_touch_cap(session).await },
            |rpc, cap, p: KeyExportPublicParams| async move {
                rpc.export_public_material_checked(cap, p.account, p.password)
                    .await
            },
        ),
    );

    dispatcher.register_typed(
        "wallet.key.rotate_master_key",
        typed_handler_cap(
            Arc::clone(&rpc),
            |rpc, session| async move { rpc.verify_rotate_cap(session).await },
            |rpc, cap, p: KeyRotateParams| async move {
                rpc.rotate_master_key_checked(cap, p.password, p.confirmation)
                    .await
            },
        ),
    );

    dispatcher.register_typed(
        "wallet.key.list_receivers",
        typed_handler_cap(
            Arc::clone(&rpc),
            |rpc, session| async move { rpc.verify_touch_cap(session).await },
            |rpc, cap, p: KeyListReceiversParams| async move {
                rpc.list_receivers_checked(cap, p.limit, p.cursor, p.filter)
                    .await
            },
        ),
    );

    dispatcher.register_typed(
        "wallet.key.validate_receiver_card",
        typed_handler_jsonrpsee_err(
            Arc::clone(&rpc),
            |rpc, p: KeyValidateReceiverCardParams| async move {
                rpc.validate_receiver_card(p.card_compact).await
            },
        ),
    );

    dispatcher.register_typed(
        "wallet.key.label_receiver",
        typed_handler_cap(
            Arc::clone(&rpc),
            |rpc, session| async move { rpc.verify_touch_cap(session).await },
            |rpc, cap, p: KeyLabelReceiverParams| async move {
                rpc.label_receiver_checked(cap, p.receiver_id, p.label).await
            },
        ),
    );
}

#[cfg(not(target_arch = "wasm32"))]
pub fn register_asset_methods(dispatcher: &RpcDispatcher, rpc: Arc<AssetRpcImpl>) {
    dispatcher.register_typed(
        "wallet.asset.list_assets",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: AssetListParams| async move {
            rpc.list_assets(p.wallet_id, p.limit, p.cursor, p.filter)
                .await
        }),
    );

    dispatcher.register_typed(
        "wallet.asset.add_asset",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: AssetAddParams| async move {
            rpc.add_asset(p.session, p.asset_data).await
        }),
    );

    dispatcher.register_typed(
        "wallet.asset.get_asset_balance",
        typed_handler_jsonrpsee_err(
            Arc::clone(&rpc),
            |rpc, p: AssetWalletAssetIdParams| async move {
                rpc.get_asset_balance(p.wallet_id, p.asset_id).await
            },
        ),
    );

    dispatcher.register_typed(
        "wallet.asset.get_asset_details",
        typed_handler_jsonrpsee_err(
            Arc::clone(&rpc),
            |rpc, p: AssetWalletAssetIdParams| async move {
                rpc.get_asset_details(p.wallet_id, p.asset_id).await
            },
        ),
    );

    dispatcher.register_typed(
        "wallet.asset.import_asset",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: AssetImportParams| async move {
            rpc.import_asset(p.session, p.asset_data).await
        }),
    );

    dispatcher.register_typed(
        "wallet.asset.merge_assets",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: AssetMergeParams| async move {
            rpc.merge_assets(p.session, p.asset_ids).await
        }),
    );

    dispatcher.register_typed(
        "wallet.asset.get_asset_metadata",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: AssetMetadataParams| async move {
            rpc.get_asset_metadata(p.asset_id).await
        }),
    );

    dispatcher.register_typed(
        "wallet.asset.receive_asset",
        typed_handler_jsonrpsee_err(
            Arc::clone(&rpc),
            |rpc, p: AssetSessionAssetIdParams| async move {
                rpc.receive_asset(p.session, p.asset_id).await
            },
        ),
    );

    dispatcher.register_typed(
        "wallet.asset.send_asset",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: AssetSendParams| async move {
            rpc.send_asset(p.session, p.asset_id, p.recipient, p.amount)
                .await
        }),
    );

    dispatcher.register_typed(
        "wallet.asset.split_asset",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: AssetSplitParams| async move {
            rpc.split_asset(p.session, p.asset_id, p.amounts).await
        }),
    );

    dispatcher.register_typed(
        "wallet.asset.stake_assets",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: AssetStakeParams| async move {
            rpc.stake_assets(p.session, p.asset_id, p.amount).await
        }),
    );

    dispatcher.register_typed(
        "wallet.asset.swap_assets",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: AssetSwapParams| async move {
            rpc.swap_assets(p.session, p.from_asset_id, p.to_asset_id, p.amount)
                .await
        }),
    );

    dispatcher.register_typed(
        "wallet.asset.unstake_assets",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: AssetUnstakeParams| async move {
            rpc.unstake_assets(p.session, p.stake_id).await
        }),
    );
}

#[cfg(not(target_arch = "wasm32"))]
pub fn register_object_methods(dispatcher: &RpcDispatcher, rpc: Arc<AssetRpcImpl>) {
    dispatcher.register_typed(
        "wallet.object.list_objects",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: ObjectListParams| async move {
            rpc.list_objects(p.wallet_id, p.limit, p.cursor, p.filter)
                .await
        }),
    );

    dispatcher.register_typed(
        "wallet.object.list_vouchers",
        typed_handler_jsonrpsee_err(
            Arc::clone(&rpc),
            |rpc, p: ObjectVoucherListParams| async move {
                rpc.list_vouchers(p.wallet_id, p.limit, p.cursor, p.status)
                    .await
            },
        ),
    );

    dispatcher.register_typed(
        "wallet.object.list_rights",
        typed_handler_jsonrpsee_err(
            Arc::clone(&rpc),
            |rpc, p: ObjectRightListParams| async move {
                rpc.list_rights(p.wallet_id, p.limit, p.cursor, p.status)
                    .await
            },
        ),
    );

    dispatcher.register_typed(
        "wallet.object.preview_package",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: ObjectPackageParams| async move {
            rpc.preview_package(p.session, p.request).await
        }),
    );

    dispatcher.register_typed(
        "wallet.object.build_package",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: ObjectPackageParams| async move {
            rpc.build_package(p.session, p.request).await
        }),
    );

    dispatcher.register_typed(
        "wallet.object.accept_voucher",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: ObjectPackageParams| async move {
            rpc.accept_voucher(p.session, p.request).await
        }),
    );

    dispatcher.register_typed(
        "wallet.object.reject_voucher",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: ObjectPackageParams| async move {
            rpc.reject_voucher(p.session, p.request).await
        }),
    );

    dispatcher.register_typed(
        "wallet.object.redeem_voucher",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: ObjectPackageParams| async move {
            rpc.redeem_voucher(p.session, p.request).await
        }),
    );

    dispatcher.register_typed(
        "wallet.object.refund_voucher",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: ObjectPackageParams| async move {
            rpc.refund_voucher(p.session, p.request).await
        }),
    );

    dispatcher.register_typed(
        "wallet.object.transfer_voucher",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: ObjectPackageParams| async move {
            rpc.transfer_voucher(p.session, p.request).await
        }),
    );

    dispatcher.register_typed(
        "wallet.object.delegate_right",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: ObjectPackageParams| async move {
            rpc.delegate_right(p.session, p.request).await
        }),
    );

    dispatcher.register_typed(
        "wallet.object.consume_right",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: ObjectPackageParams| async move {
            rpc.consume_right(p.session, p.request).await
        }),
    );

    dispatcher.register_typed(
        "wallet.object.revoke_right",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: ObjectPackageParams| async move {
            rpc.revoke_right(p.session, p.request).await
        }),
    );

    dispatcher.register_typed(
        "wallet.object.challenge_right",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: ObjectPackageParams| async move {
            rpc.challenge_right(p.session, p.request).await
        }),
    );
}

#[cfg(not(target_arch = "wasm32"))]
pub fn register_tx_methods(dispatcher: &RpcDispatcher, rpc: Arc<TxRpcImpl>) {
    dispatcher.register_typed(
        "wallet.tx.send_transaction",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: TxSendParams| async move {
            rpc.send_transaction(
                p.session,
                p.recipient,
                p.amount,
                p.asset_id,
                p.memo,
                p.idempotency_key,
                p.timestamp,
            )
            .await
        }),
    );

    dispatcher.register_typed(
        "wallet.tx.broadcast_transaction",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: TxBroadcastParams| async move {
            rpc.broadcast_transaction(p.session, p.tx_data).await
        }),
    );

    dispatcher.register_typed(
        "wallet.tx.build_transaction",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: TxBuildParams| async move {
            rpc.build_transaction(p.session, p.recipient, p.amount, p.asset_id)
                .await
        }),
    );

    dispatcher.register_typed(
        "wallet.tx.verify_transaction_package",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: TxVerifyPkgParams| async move {
            rpc.verify_transaction_package(p.session, p.tx_data).await
        }),
    );

    dispatcher.register_typed(
        "wallet.tx.cancel_transaction",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: TxWalletTxIdParams| async move {
            rpc.cancel_transaction(p.session, p.tx_id).await
        }),
    );

    dispatcher.register_typed(
        "wallet.tx.get_transaction_details",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: TxWalletTxIdParams| async move {
            rpc.get_transaction_details(p.session, p.tx_id).await
        }),
    );

    dispatcher.register_typed(
        "wallet.tx.estimate_transaction_fee",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: TxEstimateFeeParams| async move {
            rpc.estimate_transaction_fee(p.session, p.recipient, p.amount, p.asset_id)
                .await
        }),
    );

    dispatcher.register_typed(
        "wallet.tx.export_transaction",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: TxWalletTxIdParams| async move {
            rpc.export_transaction(p.session, p.tx_id).await
        }),
    );

    dispatcher.register_typed(
        "wallet.tx.import_transaction",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: TxBroadcastParams| async move {
            rpc.import_transaction(p.session, p.tx_data).await
        }),
    );

    dispatcher.register_typed(
        "wallet.tx.reconcile_transaction",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: TxWalletTxIdParams| async move {
            rpc.reconcile_transaction(p.session, p.tx_id).await
        }),
    );

    dispatcher.register_typed(
        "wallet.tx.get_transaction_history",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: TxGetHistoryParams| async move {
            rpc.get_transaction_history(p.session, p.pagination, p.filter, p.sort)
                .await
        }),
    );

    dispatcher.register_typed(
        "wallet.tx.list_pending_transactions",
        typed_handler_jsonrpsee_err(Arc::clone(&rpc), |rpc, p: TxListPendingParams| async move {
            rpc.list_pending_transactions(p.session, p.pagination).await
        }),
    );
}
