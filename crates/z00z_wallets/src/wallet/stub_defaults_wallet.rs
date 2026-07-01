impl StubDefault for PersistWalletId {
    fn stub_default() -> Self {
        PersistWalletId("00000000-0000-0000-0000-000000000000".to_string())
    }
}

impl StubDefault for PersistWalletInfo {
    fn stub_default() -> Self {
        PersistWalletInfo {
            id: PersistWalletId::stub_default(),
            name: "stub-wallet".to_string(),
            created_at: 0,
            is_locked: true,
        }
    }
}

impl StubDefault for RuntimeCreateWalletResponse {
    fn stub_default() -> Self {
        RuntimeCreateWalletResponse {
            wallet_id: PersistWalletId::stub_default(),
            name: "stub-wallet".to_string(),
            seed_phrase: "<redacted>".to_string(),
            password_strength_score: 0,
            created_at: 0,
        }
    }
}

impl StubDefault for SessionToken {
    fn stub_default() -> Self {
        SessionToken {
            token: "stub-session-token".to_string(),
            wallet_id: PersistWalletId::stub_default(),
            created_at: 0,
            expires_at: 3600000,
            last_activity_at: 0,
            permissions: vec![],
        }
    }
}

impl StubDefault for RuntimeOperationStatusWithWallet {
    fn stub_default() -> Self {
        RuntimeOperationStatusWithWallet {
            status: RuntimeOperationStatus {
                success: true,
                message: String::new(),
            },
            wallet_id: PersistWalletId::stub_default(),
        }
    }
}

impl StubDefault for RuntimeOperationStatusWithTx {
    fn stub_default() -> Self {
        RuntimeOperationStatusWithTx {
            status: RuntimeOperationStatus {
                success: true,
                message: String::new(),
            },
            tx_id: PersistTxId::stub_default(),
        }
    }
}

impl StubDefault for RuntimeDeleteWalletResponse {
    fn stub_default() -> Self {
        RuntimeDeleteWalletResponse {
            status: RuntimeOperationStatus {
                success: true,
                message: String::new(),
            },
            wallet_id: PersistWalletId::stub_default(),
            deleted: true,
        }
    }
}

impl StubDefault for PersistWalletSettings {
    fn stub_default() -> Self {
        PersistWalletSettings {
            auto_lock_timeout: 900,
            default_fee: "0".to_string(),
            currency_display: "Z00Z".to_string(),
            policy_rules: None,
            created_at: 0,
            updated_at: 0,
        }
    }
}

impl StubDefault for RuntimeShowSeedPhraseResponse {
    fn stub_default() -> Self {
        RuntimeShowSeedPhraseResponse {
            encrypted_payload: crate::rpc::types::common::RuntimeEncryptedResponse::stub(
                "show-seed-phrase",
            ),
        }
    }
}

impl StubDefault for RuntimeExportWalletResponse {
    fn stub_default() -> Self {
        RuntimeExportWalletResponse {
            success: true,
            export_path: None,
            encrypted_payload: None,
        }
    }
}

impl StubDefault for RuntimeImportWalletResponse {
    fn stub_default() -> Self {
        RuntimeImportWalletResponse {
            status: RuntimeOperationStatus {
                success: true,
                message: String::new(),
            },
            wallet_id: PersistWalletId::stub_default(),
            name: "stub-wallet".to_string(),
        }
    }
}

impl StubDefault for RuntimeKeyDeriveResponse {
    fn stub_default() -> Self {
        RuntimeKeyDeriveResponse {
            public_key: "stub-public-key".to_string(),
        }
    }
}

impl StubDefault for RuntimeExportPublicKeyResponse {
    fn stub_default() -> Self {
        RuntimeExportPublicKeyResponse {
            public_key: "stub-public-key".to_string(),
        }
    }
}
