use thiserror::Error;
use z00z_core::Asset;

use crate::{key::ReceiverKeys, receiver::check_stealth_own, WalletError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnershipPolicy {
    Keyring,
    Challenge,
}

#[cfg(feature = "ownership_policy_keyring")]
pub const TRANSPARENT_OWN_POLICY: OwnershipPolicy = OwnershipPolicy::Keyring;

#[cfg(all(
    not(feature = "ownership_policy_keyring"),
    feature = "ownership_policy_challenge"
))]
pub const TRANSPARENT_OWN_POLICY: OwnershipPolicy = OwnershipPolicy::Challenge;

#[cfg(not(any(
    feature = "ownership_policy_keyring",
    feature = "ownership_policy_challenge"
)))]
pub const TRANSPARENT_OWN_POLICY: OwnershipPolicy = OwnershipPolicy::Keyring;

#[derive(Debug, Clone)]
pub struct WalletOwnershipCtx {
    pub key_set: Vec<[u8; 32]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum OwnershipError {
    #[error("missing owner public key")]
    MissingOwnerPub,
    #[error("missing owner signature")]
    MissingOwnerSig,
    #[error("missing owner tag")]
    MissingOwnerTag,
    #[error("invalid owner signature")]
    BadOwnerSig,
    #[error("owner is not in key set")]
    OwnerNotInSet,
    #[error("asset is not stealth")]
    NotStealth,
    #[error("invalid stealth signature")]
    BadStealthSig,
    #[error("stealth owner does not match")]
    OwnerNotMatch,
    #[error("ownership policy is disabled")]
    PolicyOff,
}

fn keyring_check(asset: &Asset, ctx: &WalletOwnershipCtx) -> Result<(), OwnershipError> {
    let owner_pub = asset
        .owner_pub
        .as_ref()
        .ok_or(OwnershipError::MissingOwnerPub)?;
    let owner_bytes = owner_pub.to_bytes();
    if !ctx.key_set.iter().any(|item| item == &owner_bytes) {
        return Err(OwnershipError::OwnerNotInSet);
    }
    Ok(())
}

fn challenge_check(_asset: &Asset, _ctx: &WalletOwnershipCtx) -> Result<(), OwnershipError> {
    Err(OwnershipError::PolicyOff)
}

pub fn check_transparent_policy(
    asset: &Asset,
    ctx: &WalletOwnershipCtx,
    policy: OwnershipPolicy,
) -> Result<(), OwnershipError> {
    match policy {
        OwnershipPolicy::Keyring => keyring_check(asset, ctx),
        OwnershipPolicy::Challenge => challenge_check(asset, ctx),
    }
}

pub fn check_transparent_ownership(
    asset: &Asset,
    ctx: &WalletOwnershipCtx,
) -> Result<(), OwnershipError> {
    asset
        .owner_pub
        .as_ref()
        .ok_or(OwnershipError::MissingOwnerPub)?;

    if asset.owner_signature.is_none() {
        return Err(OwnershipError::MissingOwnerSig);
    }

    asset
        .verify_owner_signature()
        .map_err(|_| OwnershipError::BadOwnerSig)?;

    check_transparent_policy(asset, ctx, TRANSPARENT_OWN_POLICY)
}

pub fn check_stealth_ownership(asset: &Asset, keys: &ReceiverKeys) -> Result<(), OwnershipError> {
    if !asset.is_stealth() {
        return Err(OwnershipError::NotStealth);
    }

    if asset.owner_tag.is_none() {
        return Err(OwnershipError::MissingOwnerTag);
    }

    check_stealth_own(asset, keys).map_err(|err| match err {
        WalletError::NotOwned => OwnershipError::OwnerNotMatch,
        WalletError::InvalidAssetPack("missing owner_tag") => OwnershipError::MissingOwnerTag,
        _ => OwnershipError::OwnerNotMatch,
    })?;

    if asset.owner_signature.is_some() {
        asset
            .verify_owner_signature()
            .map_err(|_| OwnershipError::BadStealthSig)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        check_transparent_ownership, check_transparent_policy, OwnershipError, OwnershipPolicy,
        WalletOwnershipCtx, TRANSPARENT_OWN_POLICY,
    };
    use z00z_core::assets::AssetClass;

    #[test]
    fn test_policy_mutex() {
        match TRANSPARENT_OWN_POLICY {
            OwnershipPolicy::Keyring => {}
            OwnershipPolicy::Challenge => {}
        }
    }

    #[test]
    fn test_policy_off() {
        let asset = z00z_core::genesis::asset_std::asset_from_dev_class(AssetClass::Coin, 1, 10)
            .expect("asset");
        let owner = asset.owner_pub.as_ref().expect("owner pub").to_bytes();
        let ctx = WalletOwnershipCtx {
            key_set: vec![owner],
        };

        let err = check_transparent_policy(&asset, &ctx, OwnershipPolicy::Challenge)
            .expect_err("challenge policy must be disabled");
        assert_eq!(err, OwnershipError::PolicyOff);

        let ok = check_transparent_ownership(&asset, &ctx);
        assert!(ok.is_ok());
    }
}
