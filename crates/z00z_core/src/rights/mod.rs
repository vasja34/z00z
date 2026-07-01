//! Canonical rights vocabulary and rights-config ownership.

mod config;
mod right_action;
mod right_policy;

pub(crate) use config::parse_rights_from_yaml;
pub use config::{load_rights_from_yaml, RightClassConfig, RightsConfigEntry};
pub use right_action::RightActionV1;
pub use right_policy::{RightPolicyV1, RightRequirementV1, RightScopeV1};

#[cfg(test)]
#[path = "test_rights_config.rs"]
mod test_rights_config;
