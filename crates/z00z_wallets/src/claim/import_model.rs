//! Claim import response mapping model.

use z00z_utils::codec::Value;

use super::service::ClaimImportOutcome;

/// Normalized claim import decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportEval {
    /// Log action key.
    pub action: &'static str,
    /// Stable reason code.
    pub code: String,
    /// Typed outcome.
    pub outcome: ClaimImportOutcome,
    /// Whether row should be persisted as claimed.
    pub is_claimed: bool,
    /// Counter delta for newly inserted rows.
    pub add_new: usize,
    /// Counter delta for duplicate rows.
    pub add_dup: usize,
    /// Counter delta for rejected rows.
    pub add_rej: usize,
}

/// Map successful RPC payload to normalized decision.
pub fn map_import_ok(resp: &Value) -> ImportEval {
    let success = resp
        .get("success")
        .and_then(|row| row.as_bool())
        .unwrap_or(true);
    if !success {
        return ImportEval {
            action: "import_rejected",
            code: read_msg(resp).unwrap_or("IMPORT_UNKNOWN_ERROR").to_string(),
            outcome: ClaimImportOutcome::Rejected,
            is_claimed: false,
            add_new: 0,
            add_dup: 0,
            add_rej: 1,
        };
    }

    let msg = read_msg(resp).unwrap_or("");
    if msg == "asset_already_exists" {
        return ImportEval {
            action: "import_accepted",
            code: "IMPORT_ALREADY_EXISTS".to_string(),
            outcome: ClaimImportOutcome::AlreadyExists,
            is_claimed: true,
            add_new: 0,
            add_dup: 1,
            add_rej: 0,
        };
    }

    ImportEval {
        action: "import_accepted",
        code: "IMPORT_ACCEPTED_NEW".to_string(),
        outcome: ClaimImportOutcome::Accepted,
        is_claimed: true,
        add_new: 1,
        add_dup: 0,
        add_rej: 0,
    }
}

/// Map RPC error text to normalized decision.
pub fn map_import_err(text: &str) -> ImportEval {
    ImportEval {
        action: "import_rejected",
        code: parse_reason_code(text),
        outcome: ClaimImportOutcome::Rejected,
        is_claimed: false,
        add_new: 0,
        add_dup: 0,
        add_rej: 1,
    }
}

/// Map replay response to stable reason code.
pub fn map_replay_code(resp: &Value) -> String {
    let msg = read_msg(resp).unwrap_or("");
    if msg == "asset_already_exists" {
        return "IMPORT_ALREADY_EXISTS".to_string();
    }
    "IMPORT_UNKNOWN_ERROR".to_string()
}

fn read_msg(resp: &Value) -> Option<&str> {
    resp.get("message")
        .and_then(|row| row.as_str())
        .or_else(|| {
            resp.get("status")
                .and_then(|row| row.get("message"))
                .and_then(|row| row.as_str())
        })
}

fn parse_reason_code(text: &str) -> String {
    for token in
        text.split(|ch: char| !(ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_'))
    {
        if token.starts_with("IMPORT_") {
            return token.to_string();
        }
    }
    "IMPORT_RPC_ERROR".to_string()
}

#[cfg(test)]
mod tests {
    use super::{map_import_err, map_import_ok, map_replay_code};
    use z00z_utils::codec::json;

    #[test]
    fn test_map_ok_new() {
        let resp = json!({"success": true});
        let got = map_import_ok(&resp);
        assert_eq!(got.action, "import_accepted");
        assert_eq!(got.code, "IMPORT_ACCEPTED_NEW");
        assert!(got.is_claimed);
    }

    #[test]
    fn test_map_ok_dup() {
        let resp = json!({"success": true, "message": "asset_already_exists"});
        let got = map_import_ok(&resp);
        assert_eq!(got.code, "IMPORT_ALREADY_EXISTS");
        assert!(got.is_claimed);
    }

    #[test]
    fn test_map_err_code() {
        let got = map_import_err("rpc failed IMPORT_REJECTED_BAD_SIG row");
        assert_eq!(got.action, "import_rejected");
        assert_eq!(got.code, "IMPORT_REJECTED_BAD_SIG");
        assert!(!got.is_claimed);
    }

    #[test]
    fn test_replay_code() {
        let resp = json!({"message": "asset_already_exists"});
        assert_eq!(map_replay_code(&resp), "IMPORT_ALREADY_EXISTS");
    }
}
