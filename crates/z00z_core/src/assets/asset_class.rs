use std::{borrow::Cow, fmt, str::FromStr};

use super::asset_error::AssetError;

/// Canonical asset classes from `assets_spec_release.md` §§2.1–2.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum AssetClass {
    Coin,
    Token,
    Nft,
    Void,
}

impl fmt::Display for AssetClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let repr = match self {
            AssetClass::Coin => "Coin",
            AssetClass::Token => "Token",
            AssetClass::Nft => "Nft",
            AssetClass::Void => "Void",
        };
        write!(f, "{}", repr)
    }
}

impl FromStr for AssetClass {
    type Err = AssetError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Coin" | "coin" => Ok(AssetClass::Coin),
            "Token" | "token" => Ok(AssetClass::Token),
            "Nft" | "nft" | "NFT" => Ok(AssetClass::Nft),
            "Void" | "void" => Ok(AssetClass::Void),
            _ => Err(AssetError::InvalidClass(Cow::Owned(format!(
                "Unknown asset class: {}",
                s
            )))),
        }
    }
}

impl AssetClass {
    /// Returns unique domain byte for asset ID derivation.
    pub const fn class_byte(self) -> u8 {
        match self {
            AssetClass::Coin => 0x01,
            AssetClass::Token => 0x02,
            AssetClass::Nft => 0x03,
            AssetClass::Void => 0x04,
        }
    }
}
