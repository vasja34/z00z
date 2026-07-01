use z00z_core::assets::AssetPackPlain;
use z00z_crypto::{create_commitment, Z00ZScalar};

use super::{
    derive_s_out,
    output::{compute_owner_tag, constant_time_eq, TxStealthOutput},
    tag::{compute_leaf_ad, compute_tag16, compute_tag16_with_req},
    zkpack::ZkPack,
    StealthError,
};

/// Tag binding mode for lightweight sender-side validation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TagMode {
    /// Tag16 is bound to canonical leaf associated data.
    CardBound,
    /// Tag16 is bound to the payment request identifier.
    RequestBound {
        /// Request identifier bound into tag16 derivation.
        req_id: [u8; 32],
    },
}

/// Minimal sender-held context for post-build lightweight validation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SenderValidationCtx {
    /// Sender-derived ECDH key.
    pub k_dh: [u8; 32],
    /// Receiver owner handle used for owner tag derivation.
    pub owner_handle: [u8; 32],
    /// Asset identifier bound into leaf_ad.
    pub asset_id: [u8; 32],
    /// Serial identifier bound into leaf_ad.
    pub serial_id: u32,
    /// Tag binding mode for the output.
    pub tag_mode: TagMode,
}

/// Validate one lightweight output against sender-held context.
///
/// This is a lightweight accepted-flow sender self-check. It is not the final
/// public spend verifier contract for Scenario 1.
pub fn validate_output_self(
    output: &TxStealthOutput,
    ctx: &SenderValidationCtx,
    amount: u64,
) -> Result<(), StealthError> {
    let leaf_ad = compute_leaf_ad(
        &ctx.asset_id,
        ctx.serial_id,
        &output.r_pub,
        &output.owner_tag,
        &output.c_amount,
    );
    let plaintext = ZkPack::decrypt(
        &ctx.k_dh,
        &leaf_ad,
        &output.r_pub,
        &ctx.asset_id,
        ctx.serial_id,
        &output.enc_pack,
    )
    .ok_or(StealthError::InvalidStealthInput)?;
    let pack = AssetPackPlain::decode_checked(&plaintext)
        .map_err(|_| StealthError::InvalidStealthInput)?;

    if pack.value != amount {
        return Err(StealthError::InvalidStealthInput);
    }

    let exp_s_out = derive_s_out(&ctx.k_dh, &output.r_pub, ctx.serial_id);
    if !constant_time_eq(&exp_s_out, &pack.s_out) {
        return Err(StealthError::InvalidStealthInput);
    }

    let blinding =
        Z00ZScalar::try_from_bytes(pack.blinding).map_err(|_| StealthError::InvalidStealthInput)?;
    let commitment =
        create_commitment(pack.value, &blinding).map_err(|_| StealthError::InvalidStealthInput)?;
    let mut exp_c_amount = [0u8; 32];
    exp_c_amount.copy_from_slice(commitment.as_bytes());
    if !constant_time_eq(&exp_c_amount, &output.c_amount) {
        return Err(StealthError::InvalidStealthInput);
    }

    let exp_owner_tag = compute_owner_tag(&ctx.owner_handle, &ctx.k_dh);
    if !constant_time_eq(&exp_owner_tag, &output.owner_tag) {
        return Err(StealthError::InvalidStealthInput);
    }

    let exp_tag = match ctx.tag_mode {
        TagMode::CardBound => compute_tag16(&ctx.k_dh, &leaf_ad),
        TagMode::RequestBound { req_id } => compute_tag16_with_req(&ctx.k_dh, &req_id),
    };

    match output.tag16 {
        Some(tag) if tag == exp_tag => Ok(()),
        _ => Err(StealthError::InvalidStealthInput),
    }
}
