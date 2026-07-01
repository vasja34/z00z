use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RightActionV1 {
    Grant,
    Use,
    Delegate,
    Revoke,
    Expire,
    Challenge,
    Disclose,
}

impl RightActionV1 {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Grant => "grant",
            Self::Use => "use",
            Self::Delegate => "delegate",
            Self::Revoke => "revoke",
            Self::Expire => "expire",
            Self::Challenge => "challenge",
            Self::Disclose => "disclose",
        }
    }
}
