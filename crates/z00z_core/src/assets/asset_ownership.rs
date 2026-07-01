use std::borrow::Cow;

use z00z_crypto::{DomainHasher, KernelSignature, Z00ZRistrettoPoint, Z00ZScalar};

use super::assets::{Asset, AssetClass, AssetError};
use crate::domains::OwnerSignatureDomain;

impl Asset {
    fn owner_message_bytes(&self) -> Vec<u8> {
        #[cfg(debug_assertions)]
        {
            debug_assert!(
                self.serial_id < self.definition.serials,
                "serial_id out of range in to_owner_message()"
            );
        }

        let mut hasher = DomainHasher::<OwnerSignatureDomain>::new_with_label("owner");

        hasher.update(self.definition_id());
        hasher.update(self.serial_id.to_le_bytes());
        hasher.update(self.amount.to_le_bytes());
        hasher.update(self.commitment.as_bytes());
        hasher.update(self.nonce);
        hasher.update(self.lock_height.unwrap_or(0).to_le_bytes());

        hasher.update([self.range_proof.is_some() as u8]);
        if let Some(proof) = self.range_proof.as_ref() {
            hasher.update((proof.len() as u32).to_le_bytes());
            hasher.update(proof);
        }

        hasher.update([self.is_burned as u8]);
        hasher.update([self.is_frozen as u8]);
        hasher.update([self.is_slashed as u8]);

        hasher.update([self.owner_pub.is_some() as u8]);
        if let Some(owner_pub) = self.owner_pub.as_ref() {
            hasher.update(owner_pub.as_bytes());
        }

        hasher.update([self.r_pub.is_some() as u8]);
        if let Some(r_pub) = self.r_pub {
            hasher.update(r_pub);
        }

        hasher.update([self.owner_tag.is_some() as u8]);
        if let Some(owner_tag) = self.owner_tag {
            hasher.update(owner_tag);
        }

        hasher.update([self.enc_pack.is_some() as u8]);
        if let Some(enc_pack) = self.enc_pack.as_ref() {
            hasher.update([enc_pack.version]);
            hasher.update((enc_pack.ciphertext.len() as u32).to_le_bytes());
            hasher.update(&enc_pack.ciphertext);
            hasher.update(enc_pack.tag);
        }

        // `secret` is runtime-only stealth material and is intentionally excluded.
        // Public verifiers and AssetWire snapshots must remain able to validate the
        // owner signature without ever receiving raw `s_out` bytes.

        hasher.update([self.tag16.is_some() as u8]);
        if let Some(tag16) = self.tag16 {
            hasher.update(tag16.to_le_bytes());
        }

        hasher.update([self.leaf_ad_id.is_some() as u8]);
        if let Some(leaf_ad_id) = self.leaf_ad_id {
            hasher.update(leaf_ad_id);
        }

        let hash = hasher.finalize();
        hash.as_ref().to_vec()
    }

    /// Set owner public key.
    pub fn with_owner(mut self, owner: Z00ZRistrettoPoint) -> Self {
        self.owner_pub = Some(owner);
        self
    }

    /// Set owner signature.
    pub fn with_owner_signature(mut self, signature: KernelSignature) -> Self {
        self.owner_signature = Some(signature);
        self
    }

    /// Construct canonical signing message for owner signature.
    pub fn to_owner_message(&self) -> Vec<u8> {
        self.owner_message_bytes()
    }

    /// Sign Asset with owner's secret key.
    pub fn sign_owner(
        &self,
        secret: &Z00ZScalar,
        rng: &mut (impl rand::RngCore + rand::CryptoRng),
    ) -> Result<KernelSignature, AssetError> {
        if self.serial_id >= self.definition.serials {
            return Err(AssetError::InvalidAsset(Cow::Borrowed(
                "serial_id out of range",
            )));
        }

        if self.amount == 0 && !matches!(self.definition.class, AssetClass::Nft | AssetClass::Void)
        {
            return Err(AssetError::InvalidAsset(Cow::Borrowed(
                "amount must be non-zero for Coin/Token assets",
            )));
        }

        self.definition.validate()?;
        self.validate_stealth_consistency()?;

        if let Some(ref existing_pub) = self.owner_pub {
            let derived_pub = Z00ZRistrettoPoint::from_scalar(secret).map_err(|error| {
                AssetError::InvalidSignature(Cow::Owned(format!(
                    "Key derivation failed: {}",
                    error
                )))
            })?;
            if &derived_pub != existing_pub {
                return Err(AssetError::InvalidSignature(Cow::Borrowed(
                    "Secret key doesn't match existing owner_pub",
                )));
            }
        }

        let message = self.to_owner_message();
        z00z_crypto::sign_kernel_signature(secret, &message, rng).map_err(|error| {
            AssetError::InvalidSignature(Cow::Owned(format!("Signing failed: {}", error)))
        })
    }

    /// Verify owner signature.
    pub fn verify_owner_signature(&self) -> Result<(), AssetError> {
        self.definition.validate()?;
        self.validate_stealth_consistency()?;

        let owner_pub = self.owner_pub.as_ref().ok_or({
            AssetError::InvalidSignature(Cow::Borrowed(
                "owner_pub is required for signature verification",
            ))
        })?;

        let signature = self.owner_signature.as_ref().ok_or({
            AssetError::InvalidSignature(Cow::Borrowed(
                "owner_signature is required for verification",
            ))
        })?;

        let message = self.to_owner_message();
        if z00z_crypto::verify_kernel_signature(signature, owner_pub, message.as_slice()) {
            Ok(())
        } else {
            Err(AssetError::InvalidSignature(Cow::Borrowed(
                "Signature verification failed",
            )))
        }
    }
}
