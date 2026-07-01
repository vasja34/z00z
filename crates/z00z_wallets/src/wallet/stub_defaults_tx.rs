use crate::rpc::types::tx::RuntimeTxLifecycle;

impl StubDefault for PersistTxId {
    fn stub_default() -> Self {
        PersistTxId::new("stub-tx-id-0000".to_string())
    }
}

impl StubDefault for TxStatus {
    fn stub_default() -> Self {
        TxStatus::Pending
    }
}

impl StubDefault for PersistTxInfo {
    fn stub_default() -> Self {
        PersistTxInfo {
            id: PersistTxId::stub_default(),
            wallet_id: PersistWalletId::stub_default(),
            status: TxStatus::Pending,
            lifecycle: RuntimeTxLifecycle::Created,
            amount: 100,
            fee: 1,
            timestamp: 0,
            receipt: None,
        }
    }
}

impl StubDefault for RuntimeSendTxResponse {
    fn stub_default() -> Self {
        RuntimeSendTxResponse {
            tx_id: PersistTxId::stub_default(),
            status: TxStatus::Pending,
            lifecycle: RuntimeTxLifecycle::Created,
        }
    }
}

impl StubDefault for RuntimeTxHistoryResponse {
    fn stub_default() -> Self {
        RuntimeTxHistoryResponse {
            wallet_id: PersistWalletId::stub_default(),
            transactions: vec![PersistTxInfo::stub_default()],
            total_count: 1,
        }
    }
}

impl StubDefault for RuntimeTxDetailsResponse {
    fn stub_default() -> Self {
        RuntimeTxDetailsResponse {
            tx_id: PersistTxId::stub_default(),
            wallet_id: PersistWalletId::stub_default(),
            status: TxStatus::Pending,
            lifecycle: RuntimeTxLifecycle::Created,
            amount: 100,
            fee: 1,
            inputs: vec![[0u8; 32]],
            outputs: vec![[0u8; 32]],
            timestamp: 0,
            confirmations: 0,
            receipt: None,
            receipt_verified: false,
        }
    }
}

impl StubDefault for RuntimeListPendingTxResponse {
    fn stub_default() -> Self {
        RuntimeListPendingTxResponse {
            transactions: vec![PersistTxInfo::stub_default()],
            count: 1,
        }
    }
}

impl StubDefault for RuntimeBuildTxResponse {
    fn stub_default() -> Self {
        RuntimeBuildTxResponse {
            tx_id: PersistTxId::stub_default(),
            raw_tx: "stub-raw-transaction-data".to_string(),
        }
    }
}

impl StubDefault for RuntimeEstimateFeeResponse {
    fn stub_default() -> Self {
        RuntimeEstimateFeeResponse {
            estimated_fee: 1,
            fee_per_byte: 1,
        }
    }
}

impl StubDefault for RuntimeExportTxResponse {
    fn stub_default() -> Self {
        RuntimeExportTxResponse {
            success: true,
            export_path: None,
        }
    }
}
