use std::fmt;

/// Background scan strategy selector.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScanStrategy {
    /// Scan only candidates that passed tag16 prefilter.
    TagFilterOnly,
    /// Use balanced mode between strict filtering and full scan.
    Balanced,
    /// Run full scan over all candidates.
    FullScan,
}

/// Wallet-owned stealth output descriptor.
/// This is a transient detection DTO, not a claimed-asset persistence record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DetectedAssetPack {
    /// Explicit decoded asset-pack lane.
    pub pack_version: z00z_core::assets::AssetPackVersion,
    /// Decrypted output amount.
    pub value: u64,
    /// Commitment blinding bytes.
    pub blinding: [u8; 32],
    /// Output secret bytes.
    pub s_out: [u8; 32],
    /// Optional wallet-local memo kept inside the encrypted pack.
    pub memo: Option<Vec<u8>>,
}

impl DetectedAssetPack {
    /// Build one wallet-local detected pack from the canonical core decode surface.
    pub fn from_decoded(decoded: z00z_core::assets::DecodedAssetPack) -> Self {
        match decoded {
            z00z_core::assets::DecodedAssetPack::Basic(pack) => Self {
                pack_version: z00z_core::assets::AssetPackVersion::Basic,
                value: pack.value,
                blinding: pack.blinding,
                s_out: pack.s_out,
                memo: None,
            },
            z00z_core::assets::DecodedAssetPack::Memo(pack) => Self {
                pack_version: z00z_core::assets::AssetPackVersion::Memo,
                value: pack.value,
                blinding: pack.blinding,
                s_out: pack.s_out,
                memo: (!pack.memo.is_empty()).then_some(pack.memo),
            },
        }
    }

    /// Return the common commitment-opening pack shared by the basic and memo lanes.
    pub fn opening_pack(&self) -> z00z_core::assets::AssetPackPlain {
        z00z_core::assets::AssetPackPlain {
            value: self.value,
            blinding: self.blinding,
            s_out: self.s_out,
        }
    }

    /// Encode the detected pack back into its canonical plaintext bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>, z00z_core::assets::PackErr> {
        match self.pack_version {
            z00z_core::assets::AssetPackVersion::Basic => Ok(self.opening_pack().to_bytes()),
            z00z_core::assets::AssetPackVersion::Memo => {
                z00z_core::assets::AssetPackPlainMemo {
                    value: self.value,
                    blinding: self.blinding,
                    s_out: self.s_out,
                    memo: self.memo.clone().unwrap_or_default(),
                }
                .encode_checked()
            }
            z00z_core::assets::AssetPackVersion::Unknown => {
                Err(z00z_core::assets::PackErr::BadVer)
            }
        }
    }
}

impl PartialEq<z00z_core::assets::AssetPackPlain> for DetectedAssetPack {
    fn eq(&self, other: &z00z_core::assets::AssetPackPlain) -> bool {
        self.pack_version == z00z_core::assets::AssetPackVersion::Basic
            && self.value == other.value
            && self.blinding == other.blinding
            && self.s_out == other.s_out
    }
}

impl PartialEq<DetectedAssetPack> for z00z_core::assets::AssetPackPlain {
    fn eq(&self, other: &DetectedAssetPack) -> bool {
        other == self
    }
}

#[cfg(test)]
mod detected_pack_tests {
    use super::DetectedAssetPack;

    #[test]
    fn test_detected_pack_oversized_memo() {
        let pack = DetectedAssetPack {
            pack_version: z00z_core::assets::AssetPackVersion::Memo,
            value: 7,
            blinding: [9u8; 32],
            s_out: [3u8; 32],
            memo: Some(vec![1u8; z00z_core::assets::AssetPackPlainMemo::MEMO_MAX + 1]),
        };

        assert_eq!(pack.to_bytes(), Err(z00z_core::assets::PackErr::BadMemo));
    }

    #[test]
    fn test_detected_pack_unknown_lane() {
        let pack = DetectedAssetPack {
            pack_version: z00z_core::assets::AssetPackVersion::Unknown,
            value: 11,
            blinding: [5u8; 32],
            s_out: [7u8; 32],
            memo: None,
        };

        assert_eq!(pack.to_bytes(), Err(z00z_core::assets::PackErr::BadVer));
    }
}

/// Explicit availability state for wallet reveal fields.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum WalletReveal<T> {
    /// The value was recovered and is present in this DTO.
    Present(T),
    /// The value exists but was intentionally redacted from this DTO.
    Redacted,
    /// The value was not available when this DTO was built.
    Unavailable,
}

impl<T> WalletReveal<T> {
    /// Build a present reveal state.
    pub fn present(value: T) -> Self {
        Self::Present(value)
    }

    /// Build an explicitly redacted reveal state.
    pub fn redacted() -> Self {
        Self::Redacted
    }

    /// Build an explicitly unavailable reveal state.
    pub fn unavailable() -> Self {
        Self::Unavailable
    }

    /// Return whether the reveal is present.
    pub fn is_present(&self) -> bool {
        matches!(self, Self::Present(_))
    }

    /// Return whether the reveal is present.
    pub fn is_some(&self) -> bool {
        self.is_present()
    }

    /// Return whether the reveal is absent.
    pub fn is_none(&self) -> bool {
        !self.is_present()
    }

    /// Return whether the reveal was intentionally redacted.
    pub fn is_redacted(&self) -> bool {
        matches!(self, Self::Redacted)
    }

    /// Return whether the reveal was unavailable.
    pub fn is_unavailable(&self) -> bool {
        matches!(self, Self::Unavailable)
    }

    /// Borrow the present value, if any.
    pub fn as_ref(&self) -> Option<&T> {
        match self {
            Self::Present(value) => Some(value),
            Self::Redacted | Self::Unavailable => None,
        }
    }

    /// Extract the present value or panic with the provided message.
    pub fn expect(self, message: &str) -> T {
        match self {
            Self::Present(value) => value,
            Self::Redacted | Self::Unavailable => panic!("{message}"),
        }
    }

    /// Convert the reveal into an option for compatibility helpers.
    pub fn into_option(self) -> Option<T> {
        match self {
            Self::Present(value) => Some(value),
            Self::Redacted | Self::Unavailable => None,
        }
    }
}

impl<T> fmt::Debug for WalletReveal<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Present(_) => f.write_str("Present(<redacted>)"),
            Self::Redacted => f.write_str("Redacted"),
            Self::Unavailable => f.write_str("Unavailable"),
        }
    }
}

impl<T> From<Option<T>> for WalletReveal<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => Self::Present(value),
            None => Self::Unavailable,
        }
    }
}

impl<T> From<WalletReveal<T>> for Option<T> {
    fn from(value: WalletReveal<T>) -> Self {
        value.into_option()
    }
}

impl<T: PartialEq> PartialEq<Option<T>> for WalletReveal<T> {
    fn eq(&self, other: &Option<T>) -> bool {
        match (self, other) {
            (Self::Present(left), Some(right)) => left == right,
            (Self::Redacted, None) | (Self::Unavailable, None) => true,
            _ => false,
        }
    }
}

impl<T: PartialEq> PartialEq<WalletReveal<T>> for Option<T> {
    fn eq(&self, other: &WalletReveal<T>) -> bool {
        other == self
    }
}

/// Wallet-owned stealth output descriptor.
/// This is a transient detection DTO, not a claimed-asset persistence record.
#[derive(Clone, PartialEq, Eq)]
pub struct WalletStealthOutput {
    /// Canonical terminal asset id for the detected output.
    pub asset_id: [u8; 32],
    /// Asset serial index.
    pub serial_id: u32,
    /// Explicit decoded asset-pack lane.
    pub pack_version: z00z_core::assets::AssetPackVersion,
    /// Decrypted output amount.
    pub amount: u64,
    /// Explicit present/redacted/unavailable asset secret state.
    pub asset_secret: WalletReveal<[u8; 32]>,
    /// Explicit present/redacted/unavailable blinding state.
    pub blinding: WalletReveal<[u8; 32]>,
    /// Optional wallet-local memo kept private after decryption.
    pub memo: Option<Vec<u8>>,
    /// Ephemeral public key.
    pub r_pub: [u8; 32],
    /// Owner tag bytes.
    pub owner_tag: [u8; 32],
    /// Discovery timestamp.
    pub decrypted_at: u64,
}

impl fmt::Debug for WalletStealthOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let memo = self.memo.as_ref().map(|_| "<redacted>");
        f.debug_struct("WalletStealthOutput")
            .field("asset_id", &hex::encode(self.asset_id))
            .field("serial_id", &self.serial_id)
            .field("pack_version", &self.pack_version)
            .field("amount", &self.amount)
            .field("asset_secret", &self.asset_secret)
            .field("blinding", &self.blinding)
            .field("memo", &memo)
            .field("r_pub", &hex::encode(self.r_pub))
            .field("owner_tag", &hex::encode(self.owner_tag))
            .field("decrypted_at", &self.decrypted_at)
            .finish()
    }
}

/// Scan result marker.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScanResult {
    /// Leaf belongs to wallet.
    Mine {
        /// Decoded output details.
        wallet_output: Box<WalletStealthOutput>,
    },
    /// Leaf does not belong to wallet.
    NotMine,
    /// Leaf matched prefilter but failed M1 validation.
    MaybeMine {
        /// Tag16 prefilter matched local cache.
        tag16_match: bool,
        /// Owner tag verification failed.
        m1_failed: bool,
    },
}

impl ScanResult {
    /// Return the public receive status for this scan result.
    pub fn recv_status(&self) -> ReceiveStatus {
        self.recv_report().status
    }

    /// Convert one scan result into the shared receive/report contract.
    pub fn recv_report(&self) -> ReceiveReport {
        match self {
            Self::Mine { .. } => ReceiveReport {
                status: ReceiveStatus::Detected,
                reject: None,
                next: ReceiveNext::ReportOnly,
            },
            Self::NotMine => ReceiveReport {
                status: ReceiveStatus::NotMine,
                reject: Some(ReceiveReject::NotMine),
                next: ReceiveNext::ReportOnly,
            },
            Self::MaybeMine { .. } => ReceiveReport {
                status: ReceiveStatus::InvalidProof,
                reject: Some(ReceiveReject::InvalidProof),
                next: ReceiveNext::ReportOnly,
            },
        }
    }
}

/// Stable public receive status vocabulary for scan/report surfaces.
/// `InvalidProof` is a frozen compatibility label for outward status/RPC
/// surfaces and keeps downstream proof verification outside this scanner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReceiveStatus {
    /// Owned output detected; persistence is a separate step.
    Detected,
    /// Wallet matched a receive candidate boundary but scanner-side candidate
    /// validation failed; this status code does not imply downstream
    /// range-proof verification happened here.
    InvalidProof,
    /// Output does not belong to the wallet.
    NotMine,
}

/// Stable internal rejection taxonomy for receive/report surfaces.
/// `InvalidProof` remains the compatibility reject label for detector-side
/// candidate failures before any downstream proof verifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReceiveReject {
    /// Output does not belong to the wallet.
    NotMine,
    /// A receive candidate matched the wallet boundary but scanner-side
    /// candidate validation failed before any downstream proof verifier.
    InvalidProof,
    /// Runtime asset shape was malformed before scanner completion.
    InvalidInput,
    /// Runtime/service failure blocked receive execution.
    RuntimeFail,
}

/// Frozen receive-to-persist decision point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReceiveNext {
    /// Report detection only; do not persist claimed state.
    ReportOnly,
    /// Persist through wallet-native claimed-asset boundary.
    PersistClaim,
}

impl ReceiveNext {
    /// Return whether this receive step must persist claimed wallet state.
    pub fn should_persist(self) -> bool {
        matches!(self, Self::PersistClaim)
    }
}

/// Shared receive outcome contract for scan/report entry points.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReceiveReport {
    /// Stable public status.
    pub status: ReceiveStatus,
    /// Stable internal rejection taxonomy, if rejected.
    pub reject: Option<ReceiveReject>,
    /// Frozen next-step decision.
    pub next: ReceiveNext,
}

impl ReceiveStatus {
    /// Return the frozen public RPC/status code for this receive status.
    /// Compatibility code names stay stable even when the detector/report path
    /// stops short of downstream proof verification.
    pub fn rpc_code(self) -> &'static str {
        match self {
            Self::Detected => "RECEIVE_DETECTED",
            Self::InvalidProof => "RECEIVE_INVALID_PROOF",
            Self::NotMine => "RECEIVE_NOT_MINE",
        }
    }
}

impl ReceiveReject {
    /// Map one internal receive rejection into the stable public status vocabulary.
    /// Detector-side failures keep the existing outward compatibility label and
    /// do not imply a downstream proof verifier executed here.
    pub fn recv_status(self) -> ReceiveStatus {
        match self {
            Self::NotMine => ReceiveStatus::NotMine,
            Self::InvalidProof | Self::InvalidInput => ReceiveStatus::InvalidProof,
            Self::RuntimeFail => ReceiveStatus::InvalidProof,
        }
    }

    pub(crate) fn rpc_code(self) -> &'static str {
        match self {
            Self::NotMine => ReceiveStatus::NotMine.rpc_code(),
            Self::InvalidProof => ReceiveStatus::InvalidProof.rpc_code(),
            Self::InvalidInput => "RECEIVE_INVALID_INPUT",
            Self::RuntimeFail => ReceiveStatus::InvalidProof.rpc_code(),
        }
    }

    pub(crate) fn log_code(self) -> &'static str {
        self.rpc_code()
    }

    /// Return whether this rejection should raise an alert.
    pub fn is_alerting(self) -> bool {
        !matches!(self, Self::NotMine)
    }
}
