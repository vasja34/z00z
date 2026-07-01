use std::borrow::Cow;

use super::{AssetClass, AssetDefinition, AssetError};
use crate::assets::policy_flags::validate_flags;
use crate::config_name::validate_domain_name;

impl AssetDefinition {
    pub(crate) fn validate_fields(&self) -> Result<(), AssetError> {
        if self.serials == 0 {
            return Err(AssetError::InvalidAsset(Cow::Borrowed(
                "serials must be > 0",
            )));
        }

        if self.nominal == 0 && !matches!(self.class, AssetClass::Nft | AssetClass::Void) {
            return Err(AssetError::InvalidAsset(Cow::Borrowed(
                "nominal must be > 0 (except for NFT and Void assets)",
            )));
        }

        if matches!(self.class, AssetClass::Nft | AssetClass::Void) && self.decimals != 0 {
            return Err(AssetError::InvalidDecimals(Cow::Owned(format!(
                "{} assets must have 0 decimals, got {}",
                self.class, self.decimals
            ))));
        }

        if !validate_flags(self.policy_flags) {
            return Err(AssetError::InvalidAsset(Cow::Owned(format!(
                "policy_flags contains reserved bits: {:#010b}",
                self.policy_flags
            ))));
        }

        if self.name.len() > 64 {
            return Err(AssetError::InvalidAsset(Cow::Owned(format!(
                "name too long: {} > 64 characters",
                self.name.len()
            ))));
        }

        if self.symbol.len() > 16 {
            return Err(AssetError::InvalidAsset(Cow::Owned(format!(
                "symbol too long: {} > 16 characters",
                self.symbol.len()
            ))));
        }

        if self.domain_name.len() > 253 {
            return Err(AssetError::InvalidAsset(Cow::Owned(format!(
                "domain_name too long: {} > 253 characters",
                self.domain_name.len()
            ))));
        }
        validate_domain_name("asset.domain_name", self.domain_name.as_str())?;

        if let Some(ref meta) = self.metadata {
            let total_size: usize = meta
                .iter()
                .map(|(key, value)| key.len() + value.len())
                .sum();
            if total_size > 1024 {
                return Err(AssetError::InvalidMetadata(Cow::Owned(format!(
                    "metadata too large: {} > 1024 bytes",
                    total_size
                ))));
            }
        }

        Ok(())
    }
}
