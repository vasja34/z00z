use super::{
    decode_asset_pkg_json, payload_has_secret_field, Asset, AssetError, Codec, JsonCodec,
    PersistWalletId, PersistWalletInfo, RuntimeCreateWalletResponse, RuntimeDeleteWalletResponse,
    RuntimeEncryptedResponse, RuntimeExportWalletResponse, RuntimeImportAssetResponse,
    RuntimeImportWalletResponse, RuntimeLockWalletResponse, RuntimeShowSeedPhraseResponse,
    SessionToken, Z00ZWallet,
};

/// Structural audit facade for RPC wiring checks.
///
/// This wrapper exposes per-RPC structural stubs with the canonical RPC method
/// names and signatures, without colliding with the real `WalletService`
/// async methods.
pub struct WalletServiceReachability<'a> {
    pub(super) reachability_wallet: &'a ReachabilityWallet,
}

impl<'a> WalletServiceReachability<'a> {
    /// Structural audit hook for `wallet.list`.
    pub fn list_wallets(&self) -> Vec<PersistWalletInfo> {
        let _ = self.reachability_wallet;
        Vec::new()
    }

    /// Structural audit hook for `wallet.create`.
    pub fn create_wallet(&self, name: String, _password: String) -> RuntimeCreateWalletResponse {
        let _ = self.reachability_wallet;
        RuntimeCreateWalletResponse {
            wallet_id: PersistWalletId::default(),
            name,
            seed_phrase: "<redacted>".to_string(),
            password_strength_score: 0,
            created_at: 0,
        }
    }

    /// Structural audit hook for `wallet.delete`.
    pub fn delete_wallet(
        &self,
        id: &PersistWalletId,
        password: String,
    ) -> RuntimeDeleteWalletResponse {
        let _ = (id, password, self.reachability_wallet);
        RuntimeDeleteWalletResponse {
            status: crate::rpc::types::common::RuntimeOperationStatus {
                success: false,
                message: "WALLET_DELETE_NOT_AVAILABLE_PHASE044_SERVICE_GUARD".to_string(),
            },
            wallet_id: PersistWalletId::default(),
            deleted: false,
        }
    }

    /// Structural audit hook for `wallet.export`.
    pub fn export_wallet(
        &self,
        id: &PersistWalletId,
        password: String,
    ) -> RuntimeExportWalletResponse {
        let _ = (id, password, self.reachability_wallet);
        RuntimeExportWalletResponse {
            success: false,
            export_path: None,
            encrypted_payload: None,
        }
    }

    /// Structural audit hook for `wallet.import`.
    pub fn import_wallet(
        &self,
        data: String,
        _password: String,
        name: String,
    ) -> RuntimeImportWalletResponse {
        let _ = (data, self.reachability_wallet);
        RuntimeImportWalletResponse {
            status: crate::rpc::types::common::RuntimeOperationStatus {
                success: false,
                message: "WALLET_IMPORT_NOT_AVAILABLE_PHASE044_SERVICE_GUARD".to_string(),
            },
            wallet_id: PersistWalletId::default(),
            name,
        }
    }

    /// Structural audit hook for `wallet.session.lock_wallet`.
    pub fn lock_wallet(&self, id: &PersistWalletId) -> RuntimeLockWalletResponse {
        let _ = (id, self.reachability_wallet);
        RuntimeLockWalletResponse {
            status: crate::rpc::types::common::RuntimeOperationStatus {
                success: false,
                message: "WALLET_LOCK_NOT_AVAILABLE_PHASE044_SERVICE_GUARD".to_string(),
            },
            wallet_id: PersistWalletId::default(),
        }
    }

    /// Structural audit hook for `wallet.show_seed_phrase`.
    pub fn show_seed_phrase(&self, session: SessionToken) -> RuntimeShowSeedPhraseResponse {
        let _ = (session, self.reachability_wallet);
        RuntimeShowSeedPhraseResponse {
            encrypted_payload: RuntimeEncryptedResponse::stub(""),
        }
    }

    /// Structural audit hook for `wallet.unlock`.
    pub fn unlock_wallet(&self, id: &PersistWalletId, password: String) -> SessionToken {
        let _ = (id, password, self.reachability_wallet);
        SessionToken {
            token: String::new(),
            wallet_id: PersistWalletId::default(),
            created_at: 0,
            expires_at: 0,
            last_activity_at: 0,
            permissions: Vec::new(),
        }
    }

    /// Runtime pre-validation path called by `asset.import`.
    ///
    /// Decodes the frozen asset DTO payload, applies the forbidden-field
    /// guard, and runs cryptographic checks before returning an explicit
    /// import status.
    pub fn import_asset(
        &self,
        wallet_id: &PersistWalletId,
        asset_data: String,
    ) -> RuntimeImportAssetResponse {
        let _ = wallet_id;
        let asset = match decode_import_candidate_asset(asset_data.as_bytes()) {
            Ok(asset) => asset,
            Err(code) => return build_import_asset_response(None, false, code),
        };

        let asset_ref = runtime_asset_ref(&asset);

        match asset.verify_complete() {
            Ok(()) => build_import_asset_response(Some(asset_ref), true, "IMPORT_ACCEPTED_NEW"),
            Err(error) => {
                build_import_asset_response(Some(asset_ref), false, import_asset_error_code(&error))
            }
        }
    }
}

pub(super) type ReachabilityWallet =
    Z00ZWallet<(), (), (), (), (), (), (), (), (), (), (), (), (), (), (), ()>;

// Hash domains live in `crate::domains`.

fn build_import_asset_response(
    asset: Option<crate::rpc::types::common::RuntimeAssetRef>,
    is_inserted: bool,
    message: &str,
) -> RuntimeImportAssetResponse {
    RuntimeImportAssetResponse {
        asset: asset.unwrap_or(crate::rpc::types::common::RuntimeAssetRef {
            asset_id: [0u8; 32],
            serial_id: 0,
            symbol: String::new(),
            class: z00z_core::assets::AssetClass::Coin,
        }),
        status: crate::rpc::types::common::RuntimeOperationStatus {
            success: is_inserted,
            message: message.to_string(),
        },
        is_inserted,
        asset_already_exists: false,
    }
}

fn runtime_asset_ref(asset: &Asset) -> crate::rpc::types::common::RuntimeAssetRef {
    crate::rpc::types::common::RuntimeAssetRef {
        asset_id: asset.asset_id(),
        serial_id: asset.serial_id,
        symbol: asset.definition.symbol.clone(),
        class: asset.definition.class,
    }
}

fn import_asset_error_code(error: &AssetError) -> &'static str {
    match error {
        AssetError::InvalidStealth(_) => "IMPORT_STEALTH_INCONSISTENT",
        AssetError::InvalidAsset(_) => "IMPORT_MALFORMED_JSON",
        _ => "IMPORT_CRYPTO_VERIFY_FAILED",
    }
}

fn decode_import_candidate_asset(asset_data: &[u8]) -> Result<Asset, &'static str> {
    let codec = JsonCodec;
    let json_value: z00z_utils::codec::Value = codec
        .deserialize(asset_data)
        .map_err(|_| "IMPORT_MALFORMED_JSON")?;

    let has_secret = json_value
        .as_object()
        .map(|root| root.contains_key("secret"))
        .unwrap_or(false);

    if !has_secret && payload_has_secret_field(asset_data).is_err() {
        return Err("IMPORT_MALFORMED_JSON");
    }

    if has_secret {
        return Err("IMPORT_SECRET_FIELD_FORBIDDEN");
    }

    let dto = decode_asset_pkg_json(asset_data).map_err(|error| import_asset_error_code(&error))?;
    let wire = dto
        .to_wire()
        .map_err(|error| import_asset_error_code(&error))?;
    wire.to_asset()
        .map_err(|error| import_asset_error_code(&error))
}
