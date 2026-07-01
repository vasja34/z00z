//! Method risk policy and safe field allowlists.

use crate::rpc::types::security::RiskLevel;

pub fn risk_for_method(method: &str) -> RiskLevel {
    match method {
        "wallet.session.show_seed_phrase" => RiskLevel::Critical,
        "wallet.key.rotate_master_key" => RiskLevel::Critical,
        "app.wallet.delete_wallet" => RiskLevel::Critical,

        "wallet.session.unlock_wallet" => RiskLevel::High,
        "app.wallet.create_wallet" => RiskLevel::High,
        "wallet.tx.send_transaction" => RiskLevel::High,

        m if m.starts_with("wallet.key.") => RiskLevel::Medium,

        _ => RiskLevel::Low,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_show_seed_is_critical() {
        assert_eq!(
            risk_for_method("wallet.session.show_seed_phrase"),
            RiskLevel::Critical
        );
    }

    #[test]
    fn test_rotate_master_is_critical() {
        assert_eq!(
            risk_for_method("wallet.key.rotate_master_key"),
            RiskLevel::Critical
        );
    }

    #[test]
    fn test_delete_wallet_is_critical() {
        assert_eq!(
            risk_for_method("app.wallet.delete_wallet"),
            RiskLevel::Critical
        );
    }

    #[test]
    fn test_unlock_wallet_is_high() {
        assert_eq!(
            risk_for_method("wallet.session.unlock_wallet"),
            RiskLevel::High
        );
    }

    #[test]
    fn test_create_wallet_is_high() {
        assert_eq!(risk_for_method("app.wallet.create_wallet"), RiskLevel::High);
    }

    #[test]
    fn test_send_transaction_is_high() {
        assert_eq!(
            risk_for_method("wallet.tx.send_transaction"),
            RiskLevel::High
        );
    }

    #[test]
    fn test_wallet_key_defaults_medium() {
        assert_eq!(
            risk_for_method("wallet.key.anything_else"),
            RiskLevel::Medium
        );
    }

    #[test]
    fn test_unknown_methods_to_low() {
        assert_eq!(risk_for_method("wallet.unknown.method"), RiskLevel::Low);
        assert_eq!(risk_for_method("some.other.namespace"), RiskLevel::Low);
    }
}
