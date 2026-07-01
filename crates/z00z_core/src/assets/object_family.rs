//! Shared object-family vocabulary for asset, voucher, and right surfaces.
//!
//! Canonical public paths: `z00z_core::ObjectFamily` and
//! `z00z_core::ObjectRoleV1`.
//! Compatibility facade: `z00z_core::assets::{ObjectFamily, ObjectRoleV1}`.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectFamily {
    Asset,
    Voucher,
    Right,
}

impl ObjectFamily {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Asset => "asset",
            Self::Voucher => "voucher",
            Self::Right => "right",
        }
    }

    #[must_use]
    pub const fn is_value_bearing(self) -> bool {
        matches!(self, Self::Asset | Self::Voucher)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectRoleV1 {
    Asset,
    Voucher,
    Right,
    FeeEnvelope,
}

impl ObjectRoleV1 {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Asset => "asset",
            Self::Voucher => "voucher",
            Self::Right => "right",
            Self::FeeEnvelope => "fee_envelope",
        }
    }
}
