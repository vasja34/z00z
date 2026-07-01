impl<
        Sec,
        K,
        Addr,
        WStorage,
        Assets,
        Txs,
        Receipts,
        Sel,
        Fee,
        Asm,
        Sig,
        Prover,
        Ver,
        BackupExp,
        BackupImp,
        Pol,
    >
    Z00ZWallet<
        Sec,
        K,
        Addr,
        WStorage,
        Assets,
        Txs,
        Receipts,
        Sel,
        Fee,
        Asm,
        Sig,
        Prover,
        Ver,
        BackupExp,
        BackupImp,
        Pol,
    >
{
    /// Runtime `asset.import` pre-validation path.
    pub fn import_asset(
        &self,
        _wallet_id: &WalletId,
        asset_data: String,
    ) -> RuntimeImportAssetResponse {
        let codec = JsonCodec;
        let asset_ref = |asset: &Asset| crate::rpc::types::common::RuntimeAssetRef {
            asset_id: asset.asset_id(),
            serial_id: asset.serial_id,
            symbol: asset.definition.symbol.clone(),
            class: asset.definition.class,
        };
        let wire: AssetWire = match codec.deserialize(asset_data.as_bytes()) {
            Ok(item) => item,
            Err(_) => {
                return RuntimeImportAssetResponse {
                    asset: crate::rpc::types::common::RuntimeAssetRef {
                        asset_id: [0u8; 32],
                        serial_id: 0,
                        symbol: String::new(),
                        class: AssetClass::Coin,
                    },
                    status: RuntimeOperationStatus {
                        success: false,
                        message: "IMPORT_CRYPTO_VERIFY_FAILED".to_string(),
                    },
                    is_inserted: false,
                    asset_already_exists: false,
                };
            }
        };

        let asset = match wire.clone().to_asset() {
            Ok(item) => item,
            Err(_) => {
                return RuntimeImportAssetResponse {
                    asset: crate::rpc::types::common::RuntimeAssetRef {
                        asset_id: [0u8; 32],
                        serial_id: wire.serial_id,
                        symbol: wire.definition.symbol,
                        class: wire.definition.class,
                    },
                    status: RuntimeOperationStatus {
                        success: false,
                        message: "IMPORT_CRYPTO_VERIFY_FAILED".to_string(),
                    },
                    is_inserted: false,
                    asset_already_exists: false,
                };
            }
        };

        if asset.verify_complete().is_err() {
            return RuntimeImportAssetResponse {
                asset: asset_ref(&asset),
                status: RuntimeOperationStatus {
                    success: false,
                    message: "IMPORT_CRYPTO_VERIFY_FAILED".to_string(),
                },
                is_inserted: false,
                asset_already_exists: false,
            };
        }

        RuntimeImportAssetResponse {
            asset: asset_ref(&asset),
            status: RuntimeOperationStatus {
                success: true,
                message: "IMPORT_ACCEPTED_NEW".to_string(),
            },
            is_inserted: true,
            asset_already_exists: false,
        }
    }

}