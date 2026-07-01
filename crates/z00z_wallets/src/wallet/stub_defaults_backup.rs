impl StubDefault for RuntimeCreateBackupResponse {
    fn stub_default() -> Self {
        RuntimeCreateBackupResponse {
            status: RuntimeOperationStatus {
                success: true,
                message: "OK".to_string(),
            },
            backup_path: "/tmp/stub-backup.dat".to_string(),
            encrypted: true,
        }
    }
}

impl StubDefault for RuntimeListBackupsResponse {
    fn stub_default() -> Self {
        RuntimeListBackupsResponse {
            items: vec![],
            next_cursor: None,
            has_more: false,
            total_count: Some(0),
        }
    }
}

impl StubDefault for PersistBackupInfo {
    fn stub_default() -> Self {
        PersistBackupInfo {
            id: "stub-backup-id".to_string(),
            wallet_id: PersistWalletId::stub_default(),
            created_at: 0,
            size_bytes: 1024,
            encrypted: true,
        }
    }
}

impl StubDefault for RuntimeRestoreBackupResponse {
    fn stub_default() -> Self {
        RuntimeRestoreBackupResponse {
            status: RuntimeOperationStatus {
                success: true,
                message: "OK".to_string(),
            },
            wallet_id: PersistWalletId::stub_default(),
        }
    }
}

impl StubDefault for PersistBackupSettings {
    fn stub_default() -> Self {
        PersistBackupSettings {
            auto_backup_enabled: true,
            backup_interval_hours: 24,
            backup_location: "/tmp/backups".to_string(),
            encrypt_backups: true,
        }
    }
}

impl StubDefault for RuntimeBackupSettingsResponse {
    fn stub_default() -> Self {
        RuntimeBackupSettingsResponse {
            settings: PersistBackupSettings::stub_default(),
        }
    }
}

impl StubDefault for RuntimeChainSettings {
    fn stub_default() -> Self {
        RuntimeChainSettings {
            chain_type: ChainType::Testnet,
            rpc_endpoint: "http://localhost:8545".to_string(),
            use_tor: false,
        }
    }
}

impl StubDefault for RuntimeChainSettingsResponse {
    fn stub_default() -> Self {
        RuntimeChainSettingsResponse {
            settings: RuntimeChainSettings::stub_default(),
        }
    }
}

impl StubDefault for RuntimeSwitchChainResponse {
    fn stub_default() -> Self {
        RuntimeSwitchChainResponse {
            status: RuntimeOperationStatus {
                success: true,
                message: String::new(),
            },
            chain: ChainType::Testnet,
        }
    }
}