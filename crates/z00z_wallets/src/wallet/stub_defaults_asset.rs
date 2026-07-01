impl StubDefault for AssetId {
    fn stub_default() -> Self {
        [0u8; 32]
    }
}

impl StubDefault for AssetWire {
    fn stub_default() -> Self {
        let def = AssetDefinition {
            id: [0u8; 32],
            class: AssetClass::Coin,
            name: "Stub Asset".to_string(),
            symbol: "STUB".to_string(),
            decimals: 8,
            serials: 1,
            nominal: 100,
            domain_name: "stub.local".to_string(),
            version: 1,
            crypto_version: 1,
            policy_flags: 0,
            metadata: None,
        };

        AssetWire {
            definition: def,
            serial_id: 0,
            amount: 100,
            commitment: z00z_core::assets::Commitment::default(),
            range_proof: None,
            nonce: [0u8; 32],
            lock_height: None,
            is_burned: false,
            owner_pub: None,
            owner_signature: None,
            is_frozen: false,
            is_slashed: false,
            r_pub: None,
            owner_tag: None,
            enc_pack: None,
            secret: None,
            tag16: None,
            leaf_ad_id: None,
        }
    }
}

impl StubDefault for RuntimeAssetBalanceResponse {
    fn stub_default() -> Self {
        RuntimeAssetBalanceResponse {
            asset: RuntimeAssetRef {
                asset_id: AssetId::stub_default(),
                serial_id: 0,
                symbol: "STUB".to_string(),
                class: AssetClass::Coin,
            },
            total: 100,
            available: 100,
            pending: 0,
            decimals: 8,
        }
    }
}

impl StubDefault for RuntimeAssetMetadataResponse {
    fn stub_default() -> Self {
        RuntimeAssetMetadataResponse {
            asset: RuntimeAssetRef {
                asset_id: AssetId::stub_default(),
                serial_id: 0,
                symbol: "STUB".to_string(),
                class: AssetClass::Coin,
            },
            name: "Stub Asset".to_string(),
            decimals: 8,
            domain_name: "stub.local".to_string(),
            version: 1,
            metadata: None,
        }
    }
}

impl StubDefault for RuntimeAssetDetailsResponse {
    fn stub_default() -> Self {
        let def = AssetDefinition {
            id: [0u8; 32],
            class: AssetClass::Coin,
            name: "Stub Asset".to_string(),
            symbol: "STUB".to_string(),
            decimals: 8,
            serials: 100,
            nominal: 100_000_000,
            domain_name: "stub.local".to_string(),
            version: 1,
            crypto_version: 1,
            policy_flags: 0,
            metadata: None,
        };
        RuntimeAssetDetailsResponse {
            asset: RuntimeAssetRef {
                asset_id: AssetId::stub_default(),
                serial_id: 0,
                symbol: "STUB".to_string(),
                class: AssetClass::Coin,
            },
            definition: def,
            total_serials: 100,
            nominal_per_serial: 100_000_000,
            total_supply: 10_000_000_000,
            policy_flags: 0,
            crypto_version: 1,
        }
    }
}

impl StubDefault for RuntimeImportAssetResponse {
    fn stub_default() -> Self {
        RuntimeImportAssetResponse {
            asset: RuntimeAssetRef {
                asset_id: AssetId::stub_default(),
                serial_id: 0,
                symbol: "STUB".to_string(),
                class: AssetClass::Coin,
            },
            status: RuntimeOperationStatus {
                success: true,
                message: "Asset imported successfully (stub)".to_string(),
            },
            is_inserted: true,
            asset_already_exists: false,
        }
    }
}

impl StubDefault for RuntimeMergeAssetsResponse {
    fn stub_default() -> Self {
        RuntimeMergeAssetsResponse {
            asset: RuntimeAssetRef {
                asset_id: AssetId::stub_default(),
                serial_id: 0,
                symbol: "STUB".to_string(),
                class: AssetClass::Coin,
            },
            merged_count: 3,
            total_amount: 300,
            tx_id: Some(PersistTxId("stub-merge-tx-id".to_string())),
        }
    }
}

impl StubDefault for RuntimeSplitAssetResponse {
    fn stub_default() -> Self {
        RuntimeSplitAssetResponse {
            original_asset_id: AssetId::stub_default(),
            splits: vec![
                RuntimeAssetAmount {
                    asset: RuntimeAssetRef {
                        asset_id: AssetId::stub_default(),
                        serial_id: 0,
                        symbol: "STUB".to_string(),
                        class: AssetClass::Coin,
                    },
                    amount: 50,
                },
                RuntimeAssetAmount {
                    asset: RuntimeAssetRef {
                        asset_id: AssetId::stub_default(),
                        serial_id: 0,
                        symbol: "STUB".to_string(),
                        class: AssetClass::Coin,
                    },
                    amount: 50,
                },
            ],
            tx_id: Some(PersistTxId("stub-split-tx-id".to_string())),
        }
    }
}

impl StubDefault for RuntimeStakeAssetsResponse {
    fn stub_default() -> Self {
        RuntimeStakeAssetsResponse {
            stake_id: "stub-stake-id".to_string(),
            asset: RuntimeAssetRef {
                asset_id: AssetId::stub_default(),
                serial_id: 0,
                symbol: "STUB".to_string(),
                class: AssetClass::Coin,
            },
            amount: 100,
            start_time: 0,
            end_time: 0,
            apy: 5.0,
        }
    }
}

impl StubDefault for RuntimeUnstakeAssetsResponse {
    fn stub_default() -> Self {
        RuntimeUnstakeAssetsResponse {
            stake_id: "stub-stake-id".to_string(),
            asset: RuntimeAssetRef {
                asset_id: AssetId::stub_default(),
                serial_id: 0,
                symbol: "STUB".to_string(),
                class: AssetClass::Coin,
            },
            amount: 100,
            reward: 5,
            unstaked_at: 0,
        }
    }
}

impl StubDefault for RuntimeSwapAssetsResponse {
    fn stub_default() -> Self {
        RuntimeSwapAssetsResponse {
            from_asset_id: AssetId::stub_default(),
            from_serial_id: 0,
            from_symbol: "STUB1".to_string(),
            from_class: AssetClass::Coin,
            to_asset_id: AssetId::stub_default(),
            to_serial_id: 0,
            to_symbol: "STUB2".to_string(),
            to_class: AssetClass::Coin,
            from_amount: 100,
            to_amount: 95,
            exchange_rate: 0.95,
            fee: 5,
            tx_id: PersistTxId("stub-swap-tx-id".to_string()),
        }
    }
}

impl StubDefault for RuntimeSendAssetResponse {
    fn stub_default() -> Self {
        RuntimeSendAssetResponse {
            tx_id: PersistTxId("stub-send-asset-tx-id".to_string()),
            asset: RuntimeAssetRef {
                asset_id: AssetId::stub_default(),
                serial_id: 0,
                symbol: "STUB".to_string(),
                class: AssetClass::Coin,
            },
            owner_handle: "00".repeat(32),
            amount: 100,
            recipient: "invalid-recipient-fixture".to_string(),
            fee: 1,
            status: "pending".to_string(),
        }
    }
}

impl StubDefault for RuntimeReceiveAssetResponse {
    fn stub_default() -> Self {
        RuntimeReceiveAssetResponse {
            asset: RuntimeAssetRef {
                asset_id: AssetId::stub_default(),
                serial_id: 0,
                symbol: "STUB".to_string(),
                class: AssetClass::Coin,
            },
            status: "RECEIVE_DETECTED".to_string(),
            owner_handle: "00".repeat(32),
            view_key: "stub-view-key".to_string(),
            expires_at: None,
        }
    }
}

impl StubDefault for RuntimeAddAssetResponse {
    fn stub_default() -> Self {
        RuntimeAddAssetResponse {
            asset: RuntimeAssetRef {
                asset_id: AssetId::stub_default(),
                serial_id: 0,
                symbol: "STUB".to_string(),
                class: AssetClass::Coin,
            },
            status: RuntimeOperationStatus {
                success: true,
                message: "Asset added successfully (stub)".to_string(),
            },
        }
    }
}