use std::borrow::Cow;

use super::{
    amount::is_amount_in_range,
    assets::{Asset, AssetClass, AssetError},
};

impl Asset {
    /// Validate stealth fields consistency.
    pub fn validate_stealth_consistency(&self) -> Result<(), AssetError> {
        let has_r_pub = self.r_pub.is_some();
        let has_owner_tag = self.owner_tag.is_some();
        let has_enc_pack = self.enc_pack.is_some();
        let has_tag16 = self.tag16.is_some();
        let has_leaf_ad_id = self.leaf_ad_id.is_some();

        let has_any = has_r_pub || has_owner_tag || has_enc_pack;
        let has_all = has_r_pub && has_owner_tag && has_enc_pack;

        if has_any && !has_all {
            return Err(AssetError::InvalidStealth(Cow::Borrowed(
                "partial stealth fields are not allowed",
            )));
        }

        if has_tag16 && !has_all {
            return Err(AssetError::InvalidStealth(Cow::Borrowed(
                "tag16 requires full stealth fields",
            )));
        }

        if has_leaf_ad_id && !has_all {
            return Err(AssetError::InvalidStealth(Cow::Borrowed(
                "leaf_ad_id requires full stealth fields",
            )));
        }

        if has_all && !has_tag16 {
            return Err(AssetError::InvalidStealth(Cow::Borrowed(
                "full stealth fields require tag16",
            )));
        }

        if has_all && !has_leaf_ad_id {
            return Err(AssetError::InvalidStealth(Cow::Borrowed(
                "full stealth fields require leaf_ad_id",
            )));
        }

        if self.secret.is_some() && !has_all {
            return Err(AssetError::InvalidStealth(Cow::Borrowed(
                "secret requires full stealth fields",
            )));
        }

        Ok(())
    }

    /// Derive asset secret handle from output secret: H(s_out).
    pub fn derive_asset_secret(s_out: &[u8; 32]) -> [u8; 32] {
        z00z_crypto::hash::poseidon2_hash(b"Z00Z/ASSET_SECRET", &[s_out])
    }

    /// Validate Asset integrity.
    pub fn validate(&self) -> Result<(), AssetError> {
        self.definition.validate()?;
        self.validate_stealth_consistency()?;

        if self.serial_id >= self.definition.serials {
            return Err(AssetError::InvalidAsset(Cow::Owned(format!(
                "serial_id {} exceeds definition.serials {}",
                self.serial_id, self.definition.serials
            ))));
        }

        if self.amount == 0 && !matches!(self.definition.class, AssetClass::Nft | AssetClass::Void)
        {
            return Err(AssetError::InvalidAsset(Cow::Borrowed(
                "amount must be non-zero for non-NFT/Void assets",
            )));
        }

        if self.is_burned && !self.definition.is_burnable() {
            return Err(AssetError::InvalidAsset(Cow::Owned(format!(
                "Asset {} cannot be burned (burnable flag is false)",
                hex::encode(self.definition.id)
            ))));
        }

        if self.nonce == [0u8; 32] {
            #[cfg(not(test))]
            {
                return Err(AssetError::InvalidAsset(Cow::Borrowed(
                    "Zero nonce forbidden in production. Use derive_nonce() to generate secure nonces.",
                )));
            }
        }

        #[cfg(not(test))]
        {
            match &self.range_proof {
                None => {
                    return Err(AssetError::InvalidAsset(Cow::Borrowed(
                        "range_proof is required in production",
                    )));
                }
                Some(proof) if proof.is_empty() => {
                    return Err(AssetError::InvalidAsset(Cow::Borrowed(
                        "range_proof cannot be empty in production",
                    )));
                }
                _ => {}
            }
        }

        if self.owner_signature.is_some() {
            self.verify_owner_signature()?;
        }

        Ok(())
    }

    pub fn add_amount(left: u64, right: u64) -> Result<u64, AssetError> {
        left.checked_add(right).ok_or(AssetError::AmountOverflow)
    }

    pub fn sub_amount(left: u64, right: u64) -> Result<u64, AssetError> {
        left.checked_sub(right).ok_or(AssetError::AmountUnderflow)
    }

    pub fn mul_amount(left: u64, right: u64) -> Result<u64, AssetError> {
        left.checked_mul(right).ok_or(AssetError::AmountOverflow)
    }

    pub fn validate_amount(&self) -> Result<(), AssetError> {
        if !is_amount_in_range(self.amount) {
            return Err(AssetError::InvalidAsset(Cow::Borrowed(
                "amount exceeds range-proof policy",
            )));
        }

        self.verify_range_proof()
    }

    /// Set lock height.
    pub fn with_lock_height(mut self, height: u64) -> Self {
        self.lock_height = Some(height);
        self
    }

    /// Mark as burn output.
    pub fn with_burn(mut self) -> Result<Self, AssetError> {
        if !self.definition.is_burnable() {
            return Err(AssetError::BurnNotAllowedStructured {
                definition_id: self.definition.id,
                policy_flags: self.definition.policy_flags,
            });
        }
        self.is_burned = true;
        Ok(self)
    }

    /// Mark as slashed.
    pub fn with_slashed(mut self) -> Self {
        self.is_slashed = true;
        self
    }

    /// Complete cryptographic verification of Asset.
    pub fn verify_complete(&self) -> Result<(), AssetError> {
        self.validate()?;
        self.validate_amount()?;
        Ok(())
    }
}
