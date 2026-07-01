use super::{save_json, Deserialize, Path, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AuditLogRow {
    pub timestamp: u64,
    pub wallet_id: String,
    pub asset_id: String,
    pub action: String,
    pub reason_code: String,
}

pub(crate) fn parse_reason_code(text: &str) -> String {
    for token in
        text.split(|ch: char| !(ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_'))
    {
        if token.starts_with("IMPORT_") {
            return token.to_string();
        }
    }
    "IMPORT_RPC_ERROR".to_string()
}

pub(crate) fn write_audit_log(path: &Path, rows: &[AuditLogRow]) -> Result<(), String> {
    save_json(path, &rows.to_vec()).map_err(|e| e.to_string())
}
