//! Canonical full-leaf detection authority for receiver ownership checks.

use z00z_core::assets::AssetLeaf;
use z00z_storage::settlement::TerminalLeaf;

use super::asset_scan_support::{scan_owned, DetectFail, DetectState, ScanInput};
use crate::{
    key::ReceiverKeys,
    receiver::{DetectedAssetPack, ReceiveNext, ReceiveReject, ReceiveReport, ReceiveStatus},
    WalletError,
};

#[doc(hidden)]
pub trait ScanLeafView {
    fn serial_id(&self) -> u32;
    fn asset_id(&self) -> &[u8; 32];
    fn r_pub(&self) -> &[u8; 32];
    fn owner_tag(&self) -> &[u8; 32];
    fn c_amount(&self) -> &[u8; 32];
    fn enc_pack(&self) -> &z00z_crypto::ZkPackEncrypted;
    fn tag16(&self) -> u16;
}

impl ScanLeafView for AssetLeaf {
    fn serial_id(&self) -> u32 {
        self.serial_id
    }

    fn asset_id(&self) -> &[u8; 32] {
        &self.asset_id
    }

    fn r_pub(&self) -> &[u8; 32] {
        &self.r_pub
    }

    fn owner_tag(&self) -> &[u8; 32] {
        &self.owner_tag
    }

    fn c_amount(&self) -> &[u8; 32] {
        &self.c_amount
    }

    fn enc_pack(&self) -> &z00z_crypto::ZkPackEncrypted {
        &self.enc_pack
    }

    fn tag16(&self) -> u16 {
        self.tag16
    }
}

impl ScanLeafView for TerminalLeaf {
    fn serial_id(&self) -> u32 {
        self.serial_id
    }

    fn asset_id(&self) -> &[u8; 32] {
        &self.asset_id
    }

    fn r_pub(&self) -> &[u8; 32] {
        &self.r_pub
    }

    fn owner_tag(&self) -> &[u8; 32] {
        &self.owner_tag
    }

    fn c_amount(&self) -> &[u8; 32] {
        &self.c_amount
    }

    fn enc_pack(&self) -> &z00z_crypto::ZkPackEncrypted {
        &self.enc_pack
    }

    fn tag16(&self) -> u16 {
        self.tag16
    }
}

impl ScanLeafView for &AssetLeaf {
    fn serial_id(&self) -> u32 {
        self.serial_id
    }

    fn asset_id(&self) -> &[u8; 32] {
        &self.asset_id
    }

    fn r_pub(&self) -> &[u8; 32] {
        &self.r_pub
    }

    fn owner_tag(&self) -> &[u8; 32] {
        &self.owner_tag
    }

    fn c_amount(&self) -> &[u8; 32] {
        &self.c_amount
    }

    fn enc_pack(&self) -> &z00z_crypto::ZkPackEncrypted {
        &self.enc_pack
    }

    fn tag16(&self) -> u16 {
        self.tag16
    }
}

impl ScanLeafView for &TerminalLeaf {
    fn serial_id(&self) -> u32 {
        self.serial_id
    }

    fn asset_id(&self) -> &[u8; 32] {
        &self.asset_id
    }

    fn r_pub(&self) -> &[u8; 32] {
        &self.r_pub
    }

    fn owner_tag(&self) -> &[u8; 32] {
        &self.owner_tag
    }

    fn c_amount(&self) -> &[u8; 32] {
        &self.c_amount
    }

    fn enc_pack(&self) -> &z00z_crypto::ZkPackEncrypted {
        &self.enc_pack
    }

    fn tag16(&self) -> u16 {
        self.tag16
    }
}

pub(crate) fn receiver_scan_input(
    receiver_keys: &ReceiverKeys,
    serial_id: u32,
    leaf_ad_id: &[u8; 32],
    r_pub: &[u8; 32],
    owner_tag: &[u8; 32],
    c_amount: &[u8; 32],
    enc_pack: &z00z_crypto::ZkPackEncrypted,
    tag16: Option<u16>,
) -> Result<Option<DetectedAssetPack>, WalletError> {
    let input = ScanInput {
        serial_id,
        leaf_ad_id,
        r_pub,
        owner_tag,
        c_amount,
        enc_pack,
        tag16,
    };

    map_leaf_pack(scan_owned(
        std::iter::once(receiver_keys.reveal_view_sk()),
        &receiver_keys.owner_handle,
        &input,
        std::iter::empty(),
    ))
}

fn map_leaf_pack(state: DetectState) -> Result<Option<DetectedAssetPack>, WalletError> {
    match state {
        DetectState::Mine(pack) => Ok(Some(pack)),
        DetectState::NotMine => Ok(None),
        DetectState::Invalid(DetectFail::Tag | DetectFail::Decrypt) => Ok(None),
        DetectState::Invalid(DetectFail::Parse(msg)) => Err(WalletError::InvalidAssetPack(msg)),
        DetectState::Invalid(DetectFail::Commit) => Err(WalletError::CommitmentMismatch),
    }
}

fn map_leaf_report(state: DetectState) -> Result<ReceiveReport, WalletError> {
    match state {
        DetectState::Mine(_) => Ok(ReceiveReport {
            status: ReceiveStatus::Detected,
            reject: None,
            next: ReceiveNext::ReportOnly,
        }),
        DetectState::NotMine => Ok(ReceiveReport {
            status: ReceiveStatus::NotMine,
            reject: Some(ReceiveReject::NotMine),
            next: ReceiveNext::ReportOnly,
        }),
        DetectState::Invalid(DetectFail::Tag | DetectFail::Decrypt) => Ok(ReceiveReport {
            status: ReceiveStatus::InvalidProof,
            reject: Some(ReceiveReject::InvalidProof),
            next: ReceiveNext::ReportOnly,
        }),
        DetectState::Invalid(DetectFail::Parse(msg)) => Err(WalletError::InvalidAssetPack(msg)),
        DetectState::Invalid(DetectFail::Commit) => Err(WalletError::CommitmentMismatch),
    }
}

/// Canonical full-leaf detection authority for Spec 6 receiver ownership checks.
/// Scan one terminal leaf for ownership and decrypt payload when owned.
pub fn receiver_scan_leaf(
    receiver_keys: &ReceiverKeys,
    leaf: impl ScanLeafView,
) -> Result<Option<DetectedAssetPack>, WalletError> {
    map_leaf_pack(map_scan_leaf_state(receiver_keys, leaf))
}

/// Map one canonical full-leaf scan into the shared receive/report contract.
pub fn receiver_scan_report(
    receiver_keys: &ReceiverKeys,
    leaf: impl ScanLeafView,
) -> Result<ReceiveReport, WalletError> {
    map_leaf_report(map_scan_leaf_state(receiver_keys, leaf))
}

fn map_scan_leaf_state(receiver_keys: &ReceiverKeys, leaf: impl ScanLeafView) -> DetectState {
    let input = ScanInput {
        serial_id: leaf.serial_id(),
        leaf_ad_id: leaf.asset_id(),
        r_pub: leaf.r_pub(),
        owner_tag: leaf.owner_tag(),
        c_amount: leaf.c_amount(),
        enc_pack: leaf.enc_pack(),
        tag16: Some(leaf.tag16()),
    };

    scan_owned(
        std::iter::once(receiver_keys.reveal_view_sk()),
        &receiver_keys.owner_handle,
        &input,
        std::iter::empty(),
    )
}

#[cfg(test)]
mod tests {
    use z00z_core::assets::AssetLeaf;

    use super::{receiver_scan_leaf, receiver_scan_report};
    use crate::{
        key::{ReceiverKeys, ReceiverSecret},
        receiver::ReceiverCard,
        stealth::{build_tx_output_unchecked, SenderWallet},
    };

    #[test]
    fn test_leaf_scan_ok() {
        let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
        let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
        let leaf = make_leaf(&receiver_keys, 91);

        let pack = receiver_scan_leaf(&receiver_keys, &leaf)
            .expect("scan")
            .expect("owned pack");

        assert_eq!(pack.value, 91);
    }

    #[test]
    fn test_leaf_scan_tag_miss() {
        let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
        let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
        let mut leaf = make_leaf(&receiver_keys, 92);
        leaf.tag16 ^= 1;

        let pack = receiver_scan_leaf(&receiver_keys, &leaf).expect("scan");
        assert!(pack.is_none());
    }

    #[test]
    fn test_leaf_scan_ad_drift() {
        let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
        let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
        let mut leaf = make_leaf(&receiver_keys, 93);
        leaf.asset_id[0] ^= 1;

        let pack = receiver_scan_leaf(&receiver_keys, &leaf).expect("scan");
        assert!(pack.is_none());

        let report = receiver_scan_report(&receiver_keys, &leaf).expect("report");
        assert_eq!(report.status.rpc_code(), "RECEIVE_INVALID_PROOF");
    }

    fn make_leaf(keys: &ReceiverKeys, amount: u64) -> AssetLeaf {
        let card = ReceiverCard {
            version: 1,
            owner_handle: keys.owner_handle,
            view_pk: keys.view_pk.as_bytes().try_into().expect("view pk"),
            identity_pk: keys.identity_pk.as_bytes().try_into().expect("identity pk"),
            card_id: None,
            metadata: None,
            signature: [0u8; 64],
        };
        let mut sender_wallet = SenderWallet::new([31u8; 32]);
        let asset_id = [41u8; 32];
        let output = build_tx_output_unchecked(
            &card,
            None,
            &mut sender_wallet,
            &[32u8; 32],
            0,
            amount,
            &asset_id,
        )
        .expect("output");

        AssetLeaf {
            asset_id,
            serial_id: 0,
            r_pub: output.r_pub,
            owner_tag: output.owner_tag,
            c_amount: output.c_amount,
            enc_pack: output.enc_pack,
            range_proof: Vec::new(),
            tag16: output.tag16.expect("tag16"),
        }
    }
}
