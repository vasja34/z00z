//! Shared receiver scan helpers for Spec 6.
//!
//! These helpers freeze the post-boundary receiver detection core used by both the
//! canonical `TerminalLeaf` path and the wallet-runtime `Asset` path.
//! The decrypt-associated-data boundary is the canonical `leaf_ad_id`, not a
//! caller-chosen compatibility alias.

use subtle::ConstantTimeEq;
use z00z_core::assets::{
    decode_asset_pack, validate_serial_id_version, verify_commitment_opening, Asset,
    AssetPackVersion, PackErr,
};
use z00z_crypto::{compute_leaf_ad, compute_tag16, kdf::compute_owner_tag, Z00ZScalar};
use z00z_utils::{
    logger::{Logger, TracingLogger},
    time::{SystemTimeProvider, TimeProvider},
};

use super::asset_scan_types::WalletReveal;
use crate::{
    receiver::{DetectedAssetPack, WalletStealthOutput},
    stealth::{
        ecdh::{compute_dh_receiver, decode_r_pub},
        kdf::{compute_tag16_with_req, derive_k_dh, derive_k_dh_with_req},
        zkpack::ZkPack,
    },
};

pub(crate) struct ScanInput<'a> {
    pub serial_id: u32,
    pub leaf_ad_id: &'a [u8; 32],
    pub r_pub: &'a [u8; 32],
    pub owner_tag: &'a [u8; 32],
    pub c_amount: &'a [u8; 32],
    pub enc_pack: &'a z00z_crypto::ZkPackEncrypted,
    pub tag16: Option<u16>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DetectFail {
    Tag,
    Decrypt,
    Parse(&'static str),
    Commit,
}

#[derive(Debug)]
pub(crate) enum DetectState {
    Mine(DetectedAssetPack),
    NotMine,
    Invalid(DetectFail),
}

pub(crate) struct CachedScan {
    pub state: DetectState,
    pub owner_hit: bool,
}

pub(crate) fn check_pack_ver(serial_id: u32) -> Option<AssetPackVersion> {
    match validate_serial_id_version(serial_id) {
        AssetPackVersion::Basic | AssetPackVersion::Memo => {
            Some(validate_serial_id_version(serial_id))
        }
        AssetPackVersion::Unknown => {
            Logger::warn(
                &TracingLogger,
                &format!("serial_id={serial_id} scan skip: unknown asset pack version"),
            );
            None
        }
    }
}

pub(crate) fn scan_owned<'a, VI, RI>(
    view_sks: VI,
    owner_handle: &[u8; 32],
    input: &ScanInput<'_>,
    req_ids: RI,
) -> DetectState
where
    VI: IntoIterator<Item = &'a Z00ZScalar>,
    RI: IntoIterator<Item = [u8; 32]>,
{
    if check_pack_ver(input.serial_id).is_none() {
        return DetectState::NotMine;
    }

    let r_pub = match decode_r_pub(input.r_pub) {
        Ok(value) => value,
        Err(_) => return DetectState::NotMine,
    };

    let req_ids: Vec<[u8; 32]> = req_ids.into_iter().collect();

    for view_sk in view_sks {
        let dh = match compute_dh_receiver(view_sk, &r_pub) {
            Ok(value) => value,
            Err(_) => continue,
        };

        let state = scan_dh(owner_handle, input, &dh, &req_ids);
        if !matches!(state, DetectState::NotMine) {
            return state;
        }
    }

    DetectState::NotMine
}

fn ordered_request_candidates(
    dh: &[u8; 32],
    req_ids: impl IntoIterator<Item = [u8; 32]>,
) -> Vec<([u8; 32], Option<[u8; 32]>)> {
    let mut candidates = Vec::new();

    for req_id in req_ids {
        candidates.push((derive_k_dh_with_req(dh, &req_id), Some(req_id)));
    }

    // Keep the request-less fallback explicit and last so request-bound routes
    // cannot be shadowed by the generic candidate.
    candidates.push((derive_k_dh(dh), None));
    candidates
}

pub(crate) fn detect_pack(
    input: &ScanInput<'_>,
    k_dh: &[u8; 32],
    req_id: Option<&[u8; 32]>,
) -> DetectState {
    detect_pack_state(input, k_dh, req_id)
}

pub(crate) fn scan_cached_keys<CI>(
    owner_handle: &[u8; 32],
    input: &ScanInput<'_>,
    candidates: CI,
) -> CachedScan
where
    CI: IntoIterator<Item = ([u8; 32], Option<[u8; 32]>)>,
{
    let mut owner_hit = false;
    let mut invalid = None;

    for (k_dh, req_id) in candidates {
        let expected = compute_owner_tag(owner_handle, &k_dh);
        if !tag_eq(&expected, input.owner_tag) {
            continue;
        }

        owner_hit = true;

        match detect_pack(input, &k_dh, req_id.as_ref()) {
            DetectState::Mine(pack) => {
                return CachedScan {
                    state: DetectState::Mine(pack),
                    owner_hit: true,
                };
            }
            DetectState::Invalid(err) => {
                invalid.get_or_insert(err);
            }
            DetectState::NotMine => {}
        }
    }

    CachedScan {
        state: invalid.map_or(DetectState::NotMine, DetectState::Invalid),
        owner_hit,
    }
}

fn detect_pack_state(
    input: &ScanInput<'_>,
    k_dh: &[u8; 32],
    req_id: Option<&[u8; 32]>,
) -> DetectState {
    let Some(pack_version) = check_pack_ver(input.serial_id) else {
        return DetectState::NotMine;
    };

    let leaf_ad = compute_kdf_ad(input);
    if !has_valid_tag16(input, k_dh, &leaf_ad, req_id) {
        return DetectState::Invalid(DetectFail::Tag);
    }

    let payload = match decrypt_pack(input, k_dh, &leaf_ad) {
        Some(value) => value,
        None => return DetectState::Invalid(DetectFail::Decrypt),
    };

    let pack = match parse_pack(input, pack_version, &payload) {
        Ok(value) => value,
        Err(err) => return DetectState::Invalid(err),
    };

    if let Err(err) = verify_pack_commitment(input, &pack) {
        return DetectState::Invalid(err);
    }

    DetectState::Mine(pack)
}

pub(crate) fn make_wallet_output(
    leaf: &Asset,
    pack: &DetectedAssetPack,
    r_pub: &[u8; 32],
    owner_tag: &[u8; 32],
) -> WalletStealthOutput {
    WalletStealthOutput {
        asset_id: leaf.asset_id(),
        serial_id: leaf.serial_id,
        pack_version: pack.pack_version,
        amount: pack.value,
        asset_secret: WalletReveal::present(pack.s_out),
        blinding: WalletReveal::present(pack.blinding),
        memo: pack.memo.clone(),
        r_pub: *r_pub,
        owner_tag: *owner_tag,
        decrypted_at: SystemTimeProvider.compat_unix_timestamp(),
    }
}

fn compute_kdf_ad(input: &ScanInput<'_>) -> [u8; 32] {
    compute_leaf_ad(
        input.leaf_ad_id,
        input.serial_id,
        input.r_pub,
        input.owner_tag,
        input.c_amount,
    )
}

fn has_valid_tag16(
    input: &ScanInput<'_>,
    k_dh: &[u8; 32],
    leaf_ad: &[u8; 32],
    req_id: Option<&[u8; 32]>,
) -> bool {
    let expected = match req_id {
        Some(req_id) => compute_tag16_with_req(k_dh, req_id),
        None => compute_tag16(k_dh, leaf_ad),
    };
    input.tag16.is_none_or(|tag16| expected == tag16)
}

fn decrypt_pack(input: &ScanInput<'_>, k_dh: &[u8; 32], leaf_ad: &[u8; 32]) -> Option<Vec<u8>> {
    ZkPack::decrypt(
        k_dh,
        leaf_ad,
        input.r_pub,
        input.leaf_ad_id,
        input.serial_id,
        input.enc_pack,
    )
}

fn parse_pack(
    input: &ScanInput<'_>,
    pack_version: AssetPackVersion,
    payload: &[u8],
) -> Result<DetectedAssetPack, DetectFail> {
    decode_asset_pack(payload, pack_version)
        .map(DetectedAssetPack::from_decoded)
        .map_err(|error| match error {
            PackErr::BadLen => {
                Logger::debug(
                    &TracingLogger,
                    &format!(
                        "serial_id={} got={} pack_version={:?} scan skip: invalid asset pack length",
                        input.serial_id,
                        payload.len(),
                        pack_version
                    ),
                );
                DetectFail::Parse("wrong length")
            }
            PackErr::BadBlind => DetectFail::Parse("invalid blinding"),
            PackErr::BadMemo => DetectFail::Parse("bad memo"),
            PackErr::BadVer => DetectFail::Parse("unsupported version"),
        })
}

fn verify_pack_commitment(
    input: &ScanInput<'_>,
    pack: &DetectedAssetPack,
) -> Result<(), DetectFail> {
    let blinding = match Z00ZScalar::try_from_bytes(pack.blinding) {
        Ok(value) => value,
        Err(_) => return Err(DetectFail::Parse("invalid blinding")),
    };

    let commitment = match z00z_crypto::Commitment::from_bytes(input.c_amount) {
        Ok(value) => value,
        Err(_) => return Err(DetectFail::Parse("invalid commitment")),
    };

    if verify_commitment_opening(commitment.as_commitment(), pack.value, &blinding).unwrap_or(false)
    {
        Ok(())
    } else {
        Err(DetectFail::Commit)
    }
}

fn scan_dh(
    owner_handle: &[u8; 32],
    input: &ScanInput<'_>,
    dh: &[u8; 32],
    req_ids: &[[u8; 32]],
) -> DetectState {
    scan_cached_keys(
        owner_handle,
        input,
        ordered_request_candidates(dh, req_ids.iter().copied()),
    )
    .state
}

fn tag_eq(left: &[u8; 32], right: &[u8; 32]) -> bool {
    left.ct_eq(right).into()
}

#[cfg(test)]
#[path = "test_asset_scan_support_suite.rs"]
mod test_asset_scan_support_suite;
