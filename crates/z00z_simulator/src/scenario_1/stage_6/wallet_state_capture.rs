use super::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WalletItemRow {
    pub(crate) asset_id_hex: String,
    pub(crate) serial_id: u32,
    pub(crate) class: String,
    pub(crate) amount: u64,
    pub(crate) status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SerialDistRow {
    pub(crate) serial_id: u32,
    pub(crate) row_count: usize,
    pub(crate) total_amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WalletStateRow {
    pub(crate) actor: String,
    pub(crate) wallet_id: String,
    pub(crate) wlt_path: String,
    pub(crate) wlt_exists: bool,
    pub(crate) wlt_size_bytes: u64,
    pub(crate) page_count: usize,
    pub(crate) item_count: usize,
    pub(crate) serial_dist: Vec<SerialDistRow>,
    pub(crate) items: Vec<WalletItemRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WalletStateDump {
    pub(crate) stage: u32,
    pub(crate) phase: String,
    pub(crate) generated_at: String,
    pub(crate) wallets: Vec<WalletStateRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WalletDiffRow {
    pub(crate) actor: String,
    pub(crate) wallet_id: String,
    pub(crate) asset_id_hex: String,
    pub(crate) serial_id: u32,
    pub(crate) class: String,
    pub(crate) output_role: Option<String>,
    pub(crate) before_amount: Option<u64>,
    pub(crate) after_amount: Option<u64>,
    pub(crate) status: String,
    pub(crate) lifecycle_status: String,
    pub(crate) tx_digest_hex: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WalletDiffDump {
    pub(crate) stage: u32,
    pub(crate) generated_at: String,
    pub(crate) rows: Vec<WalletDiffRow>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct PendingRow {
    pub(crate) actor: String,
    pub(crate) wallet_id: String,
    pub(crate) asset_id_hex: String,
    pub(crate) serial_id: u32,
    pub(crate) class: String,
    pub(crate) amount: u64,
    pub(crate) lifecycle_status: String,
    pub(crate) output_role: Option<String>,
    pub(crate) tx_digest_hex: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ConfirmRow {
    pub(crate) actor: String,
    pub(crate) wallet_id: String,
    pub(crate) asset_id_hex: String,
    pub(crate) serial_id: u32,
    pub(crate) class: String,
    pub(crate) amount: u64,
    pub(crate) lifecycle_status: String,
    pub(crate) output_role: Option<String>,
    pub(crate) tx_digest_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SelectedInputRow {
    pub(crate) actor: String,
    pub(crate) wallet_id: String,
    pub(crate) asset_id_hex: String,
    pub(crate) serial_id: u32,
    pub(crate) class: String,
    pub(crate) symbol: String,
    pub(crate) amount: u64,
}

#[derive(Default)]
pub(crate) struct SenderPersist {
    pub(crate) spent_marked: usize,
    pub(crate) change_imported: usize,
    pub(crate) tracked_amount_changed: usize,
}
