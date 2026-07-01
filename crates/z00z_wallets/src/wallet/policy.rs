//! Policy enforcement.

use crate::persistence::tx::TxPolicySpendWindow;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use z00z_core::assets::registry::AssetId;
use z00z_core::Asset;
use z00z_utils::time::TimeProvider;

/// Policy rules.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyRules {
    /// Maximum per-transaction amount.
    pub max_tx_amount: Option<u64>,
    /// Maximum daily amount.
    pub max_daily_amount: Option<u64>,
    /// Allowlist of assets.
    pub allowed_assets: Option<Vec<AssetId>>,
    /// Allowlist of receiver or recipient identifiers.
    pub allowed_recipients: Option<Vec<String>>,
    /// Require prior spend confirmation before allowing another send.
    pub require_confirmation: bool,
    /// Optional time restrictions.
    pub time_restrictions: Option<TimeRestrictions>,
}

/// Time-based restrictions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimeRestrictions {
    /// Allowed hour start (0-23).
    pub allowed_hours_start: u8,
    /// Allowed hour end (0-23).
    pub allowed_hours_end: u8,
    /// Allowed days of week (0=Sunday, 1=Monday, ...).
    pub allowed_days: Vec<u8>,
}

/// Policy errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PolicyError {
    /// Generic policy violation.
    #[error("policy violation: {0}")]
    Violation(String),

    /// Amount exceeds configured limit.
    #[error("amount exceeds limit: {amount} > {limit}")]
    AmountExceeded {
        /// Amount requested for spending.
        amount: u64,
        /// Maximum allowed amount.
        limit: u64,
    },

    /// Daily limit exceeded.
    #[error("daily limit exceeded: {spent} + {amount} > {limit}")]
    DailyLimitExceeded {
        /// Amount already spent within the current day window.
        spent: u64,
        /// Amount requested for spending.
        amount: u64,
        /// Maximum daily spend limit.
        limit: u64,
    },

    /// Prior spend remains unconfirmed and blocks the next spend.
    #[error("confirmation required: {pending} pending spend(s) remain unconfirmed")]
    ConfirmationRequired {
        /// Number of locally-originated pending spend flows still awaiting
        /// confirmation.
        pending: usize,
    },

    /// Asset is not allowed.
    #[error("asset not allowed")]
    AssetNotAllowed,

    /// Recipient is not allowed.
    #[error("recipient not allowed")]
    RecipientNotAllowed,
}

/// Policy result type.
pub type PolicyResult<T> = std::result::Result<T, PolicyError>;

/// Typed wallet spend context derived from durable tx history.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PolicySpendContext {
    /// Total amount already spent for the active asset within the current UTC
    /// day window.
    pub spent_today: u64,
    /// Count of locally-originated pending spend flows that still require
    /// confirmation before another spend may proceed.
    pub pending_confirmation_count: usize,
}

impl From<TxPolicySpendWindow> for PolicySpendContext {
    fn from(window: TxPolicySpendWindow) -> Self {
        Self {
            spent_today: window.spent_amount,
            pending_confirmation_count: window.pending_confirmation_count,
        }
    }
}

/// Return the inclusive-exclusive UTC day window containing `now_ms`.
pub(crate) fn utc_day_window_ms(now_ms: u64) -> (u64, u64) {
    const DAY_MS: u64 = 86_400_000;
    let start = (now_ms / DAY_MS) * DAY_MS;
    (start, start.saturating_add(DAY_MS))
}

/// Policy trait.
///
/// Implementation requirements:
/// - use `z00z_utils::time::TimeProvider` for time-based restrictions;
/// - derive daily spend and pending-confirmation state from the canonical
///   tx-history JSONL store;
/// - provide clear typed violation messages for user-visible feedback.
pub trait Policy {
    /// Validate a spend against policy rules.
    fn validate_spend(&self, asset: &Asset, amount: u64, recipient: &str) -> PolicyResult<()>;

    /// Validate a spend against policy rules using durable spend history.
    fn validate_spend_with_context(
        &self,
        asset: &Asset,
        amount: u64,
        recipient: &str,
        context: &PolicySpendContext,
    ) -> PolicyResult<()>;

    /// Load current policy rules.
    fn rules(&self) -> PolicyResult<PolicyRules>;
}

/// Default Policy implementation.
#[derive(Debug, Clone)]
pub struct PolicyImpl<T: TimeProvider> {
    rules: PolicyRules,
    time_provider: T,
}

impl<T: TimeProvider> PolicyImpl<T> {
    /// Create a new policy implementation with the given rules.
    pub fn new(rules: PolicyRules, time_provider: T) -> Self {
        Self {
            rules,
            time_provider,
        }
    }

    fn is_asset_allowed(rules: &PolicyRules, asset_id: &AssetId) -> bool {
        match &rules.allowed_assets {
            None => true,
            Some(list) => list.iter().any(|id| id == asset_id),
        }
    }

    fn is_recipient_allowed(rules: &PolicyRules, recipient: &str) -> bool {
        match &rules.allowed_recipients {
            None => true,
            Some(list) => list.iter().any(|addr| addr == recipient),
        }
    }

    fn validate_time_restrictions(&self, restrictions: &TimeRestrictions) -> PolicyResult<()> {
        let start = restrictions.allowed_hours_start;
        let end = restrictions.allowed_hours_end;

        if start > 23 || end > 23 {
            return Err(PolicyError::Violation(
                "time restriction: allowed_hours_* must be within 0..=23".to_string(),
            ));
        }

        if restrictions.allowed_days.is_empty() {
            return Err(PolicyError::Violation(
                "time restriction: no allowed_days configured".to_string(),
            ));
        }

        if restrictions.allowed_days.iter().any(|d| *d > 6) {
            return Err(PolicyError::Violation(
                "time restriction: allowed_days values must be within 0..=6".to_string(),
            ));
        }

        let now_ms = self.time_provider.try_unix_timestamp_ms().map_err(|e| {
            PolicyError::Violation(format!("time restriction: clock unavailable: {e}"))
        })?;
        let now_secs = now_ms / 1000;
        let hour_utc = ((now_secs / 3600) % 24) as u8;
        let days_since_epoch = now_secs / 86_400;

        // 1970-01-01 is Thursday. With 0=Sunday, Thursday=4.
        let weekday = ((days_since_epoch + 4) % 7) as u8;

        if !restrictions.allowed_days.contains(&weekday) {
            return Err(PolicyError::Violation(
                "time restriction: day not allowed".to_string(),
            ));
        }

        let hour_allowed = if start == end {
            true
        } else if start < end {
            hour_utc >= start && hour_utc <= end
        } else {
            hour_utc >= start || hour_utc <= end
        };

        if !hour_allowed {
            return Err(PolicyError::Violation(
                "time restriction: hour not allowed".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_definition_with_context(
        &self,
        definition_id: &AssetId,
        amount: u64,
        recipient: &str,
        context: &PolicySpendContext,
    ) -> PolicyResult<()> {
        if let Some(limit) = self.rules.max_tx_amount {
            if amount > limit {
                return Err(PolicyError::AmountExceeded { amount, limit });
            }
        }

        if let Some(limit) = self.rules.max_daily_amount {
            let total = context.spent_today.saturating_add(amount);
            if total > limit {
                return Err(PolicyError::DailyLimitExceeded {
                    spent: context.spent_today,
                    amount,
                    limit,
                });
            }
        }

        if !Self::is_asset_allowed(&self.rules, definition_id) {
            return Err(PolicyError::AssetNotAllowed);
        }

        if !Self::is_recipient_allowed(&self.rules, recipient) {
            return Err(PolicyError::RecipientNotAllowed);
        }

        if let Some(restrictions) = &self.rules.time_restrictions {
            self.validate_time_restrictions(restrictions)?;
        }

        if self.rules.require_confirmation && context.pending_confirmation_count > 0 {
            return Err(PolicyError::ConfirmationRequired {
                pending: context.pending_confirmation_count,
            });
        }

        Ok(())
    }

    /// Validate a spend using the canonical live asset definition identifier
    /// plus durable spend-history context.
    pub fn validate_spend_definition_with_context(
        &self,
        definition_id: &AssetId,
        amount: u64,
        recipient: &str,
        context: &PolicySpendContext,
    ) -> PolicyResult<()> {
        self.validate_definition_with_context(definition_id, amount, recipient, context)
    }
}

impl<T: TimeProvider> Policy for PolicyImpl<T> {
    fn validate_spend(&self, asset: &Asset, amount: u64, recipient: &str) -> PolicyResult<()> {
        self.validate_spend_with_context(asset, amount, recipient, &PolicySpendContext::default())
    }

    fn validate_spend_with_context(
        &self,
        asset: &Asset,
        amount: u64,
        recipient: &str,
        context: &PolicySpendContext,
    ) -> PolicyResult<()> {
        self.validate_definition_with_context(&asset.definition.id, amount, recipient, context)
    }

    fn rules(&self) -> PolicyResult<PolicyRules> {
        Ok(self.rules.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type MockTimeProvider = z00z_utils::time::MockTimeProvider;

    fn create_test_asset(serial_id: u32) -> Asset {
        z00z_core::genesis::asset_std::asset_from_dev_cfg("z00z", serial_id, 1000)
            .expect("valid std asset")
    }

    #[test]
    fn test_validate_spend_amount_exceeded() {
        let rules = PolicyRules {
            max_tx_amount: Some(10),
            max_daily_amount: None,
            allowed_assets: None,
            allowed_recipients: None,
            require_confirmation: false,
            time_restrictions: None,
        };
        let policy = PolicyImpl::new(rules, MockTimeProvider::default());
        let asset = create_test_asset(0);

        let err = policy.validate_spend(&asset, 11, "alice");
        assert!(matches!(err, Err(PolicyError::AmountExceeded { .. })));
    }

    #[test]
    fn test_validate_spend_disallowed_asset() {
        let asset = create_test_asset(0);
        let allowed: AssetId = [2u8; 32];

        let rules = PolicyRules {
            max_tx_amount: None,
            max_daily_amount: None,
            allowed_assets: Some(vec![allowed]),
            allowed_recipients: None,
            require_confirmation: false,
            time_restrictions: None,
        };
        let policy = PolicyImpl::new(rules, MockTimeProvider::default());

        let err = policy.validate_spend(&asset, 100, "alice");
        assert!(matches!(err, Err(PolicyError::AssetNotAllowed)));
    }

    #[test]
    fn test_validate_spend_disallowed_recipient() {
        let asset = create_test_asset(0);

        let rules = PolicyRules {
            max_tx_amount: None,
            max_daily_amount: None,
            allowed_assets: None,
            allowed_recipients: Some(vec!["bob".to_string()]),
            require_confirmation: false,
            time_restrictions: None,
        };
        let policy = PolicyImpl::new(rules, MockTimeProvider::default());

        let err = policy.validate_spend(&asset, 100, "alice");
        assert!(matches!(err, Err(PolicyError::RecipientNotAllowed)));
    }

    #[test]
    fn test_validate_spend_time_restrictions() {
        let asset = create_test_asset(0);

        // Allow only Sunday (0) and Monday (1)
        let rules = PolicyRules {
            max_tx_amount: None,
            max_daily_amount: None,
            allowed_assets: None,
            allowed_recipients: None,
            require_confirmation: false,
            time_restrictions: Some(TimeRestrictions {
                allowed_hours_start: 9,
                allowed_hours_end: 17,
                allowed_days: vec![0, 1], // Sunday, Monday
            }),
        };

        // Use a known timestamp: 1970-01-01 00:00:00 UTC (Thursday)
        let policy = PolicyImpl::new(rules, MockTimeProvider::default());

        // Should fail - Thursday is not in allowed days
        let err = policy.validate_spend(&asset, 100, "alice");
        assert!(matches!(err, Err(PolicyError::Violation(_))));
    }

    #[test]
    fn test_validate_spend_success() {
        let rules = PolicyRules {
            max_tx_amount: Some(1000),
            max_daily_amount: None,
            allowed_assets: None,
            allowed_recipients: None,
            require_confirmation: false,
            time_restrictions: None,
        };
        let policy = PolicyImpl::new(rules, MockTimeProvider::default());
        let asset = create_test_asset(0);

        // Should succeed
        policy.validate_spend(&asset, 500, "alice").unwrap();
    }

    #[test]
    fn test_daily_limit_context() {
        let rules = PolicyRules {
            max_tx_amount: None,
            max_daily_amount: Some(10),
            allowed_assets: None,
            allowed_recipients: None,
            require_confirmation: false,
            time_restrictions: None,
        };
        let policy = PolicyImpl::new(rules, MockTimeProvider::default());
        let asset = create_test_asset(0);
        let context = PolicySpendContext {
            spent_today: 7,
            pending_confirmation_count: 0,
        };

        let err = policy.validate_spend_with_context(&asset, 4, "alice", &context);
        assert_eq!(
            err,
            Err(PolicyError::DailyLimitExceeded {
                spent: 7,
                amount: 4,
                limit: 10,
            })
        );
    }

    #[test]
    fn test_confirmation_required_context() {
        let rules = PolicyRules {
            max_tx_amount: None,
            max_daily_amount: None,
            allowed_assets: None,
            allowed_recipients: None,
            require_confirmation: true,
            time_restrictions: None,
        };
        let policy = PolicyImpl::new(rules, MockTimeProvider::default());
        let asset = create_test_asset(0);
        let context = PolicySpendContext {
            spent_today: 0,
            pending_confirmation_count: 2,
        };

        let err = policy.validate_spend_with_context(&asset, 1, "alice", &context);
        assert_eq!(err, Err(PolicyError::ConfirmationRequired { pending: 2 }));
    }

    #[test]
    fn test_uses_live_definition_id() {
        let asset = create_test_asset(0);
        let definition_id = asset.definition.id;
        let other_definition_id = [7u8; 32];

        let rules = PolicyRules {
            max_tx_amount: None,
            max_daily_amount: None,
            allowed_assets: Some(vec![definition_id]),
            allowed_recipients: None,
            require_confirmation: false,
            time_restrictions: None,
        };
        let policy = PolicyImpl::new(rules, MockTimeProvider::default());

        policy
            .validate_spend_definition_with_context(
                &definition_id,
                1,
                "alice",
                &PolicySpendContext::default(),
            )
            .unwrap();

        let err = policy.validate_spend_definition_with_context(
            &other_definition_id,
            1,
            "alice",
            &PolicySpendContext::default(),
        );
        assert_eq!(err, Err(PolicyError::AssetNotAllowed));
    }
}
