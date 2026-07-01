use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use z00z_core::assets::Asset;

use crate::key::ReceiverKeys;
use crate::receiver::{PaymentRequest, ScanResult, StealthOutputScanner};

/// Optional parallel batch wrapper over `StealthOutputScanner`.
///
/// This type owns batching and rayon execution only. It does not own receiver
/// crypto validation, claimed persistence, or a second receive pipeline.
/// Canonical ownership detection continues to live in `leaf_scan.rs`,
/// `wallet_asset_scanner.rs`, and shared private receiver helpers.
#[derive(Clone, Debug)]
pub struct OptimizedScanner {
    base: StealthOutputScanner,
    batch_size: usize,
}

impl OptimizedScanner {
    /// Create optional optimized scanner with configured batch size.
    pub fn new(keys: &ReceiverKeys, batch_size: usize) -> Self {
        Self {
            base: StealthOutputScanner::from_keys(keys),
            batch_size: batch_size.max(1),
        }
    }

    /// Register an active payment request for req-bound scan paths.
    pub fn add_request(&mut self, request: &PaymentRequest) {
        self.base.add_request(request);
    }

    /// Scan a batch of leaves without changing canonical detector semantics.
    pub fn scan_batch(&self, leaves: &[Asset]) -> Vec<ScanResult> {
        if leaves.len() <= self.batch_size {
            return leaves
                .iter()
                .map(|leaf| self.base.scan_leaf(leaf))
                .collect();
        }

        leaves
            .par_iter()
            .map(|leaf| self.base.scan_leaf(leaf))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use z00z_core::Asset;

    use super::{OptimizedScanner, ScanResult, StealthOutputScanner};
    use crate::key::{ReceiverKeys, ReceiverSecret};
    use crate::receiver::Tag16Context;
    use crate::receiver::{PaymentRequest, ReceiverCard, RequestParams};
    use crate::stealth::{
        build_tx_output_unchecked, compute_dh_receiver, decode_r_pub, derive_k_dh, SenderWallet,
    };
    use z00z_utils::time::{SystemTimeProvider, TimeProvider};

    #[test]
    fn test_optimized_scanner_base_detector() {
        let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
        let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
        let card = receiver_card(&receiver_keys);
        let mut scanner = OptimizedScanner::new(&receiver_keys, 1);
        let mut baseline = StealthOutputScanner::from_keys(&receiver_keys);

        let mut mine = make_asset(100, [9u8; 32]);
        let mut sender_wallet = SenderWallet::new([7u8; 32]);
        let output = build_tx_output_unchecked(
            &card,
            None,
            &mut sender_wallet,
            &[8u8; 32],
            0,
            100,
            &[9u8; 32],
        )
        .expect("output");

        mine.r_pub = Some(output.r_pub);
        mine.owner_tag = Some(output.owner_tag);
        mine.commitment = z00z_crypto::Commitment::from_bytes(&output.c_amount)
            .expect("commitment")
            .as_commitment()
            .clone();
        mine.enc_pack = Some(output.enc_pack);
        mine.tag16 = output.tag16;

        let r_pub = decode_r_pub(&output.r_pub).expect("decode r_pub");
        let dh = compute_dh_receiver(receiver_keys.reveal_view_sk(), &r_pub).expect("dh");
        let k_dh = derive_k_dh(&dh);
        let tag16 = output.tag16.expect("tag16");
        scanner
            .base
            .add_tag_context(tag16, Tag16Context { k_dh, req_id: None });
        baseline.add_tag_context(tag16, Tag16Context { k_dh, req_id: None });

        let mut maybe_mine = mine.clone();
        maybe_mine.serial_id = 2_000_000;

        let mut maybe_mine_owner_tag = mine.clone();
        maybe_mine_owner_tag.owner_tag = Some([0xA5; 32]);

        let not_mine = make_asset(101, [10u8; 32]);
        let leaves = vec![mine, maybe_mine_owner_tag, maybe_mine, not_mine];
        let before = SystemTimeProvider.compat_unix_timestamp();
        let results = scanner.scan_batch(&leaves);
        let baseline_results: Vec<_> = leaves.iter().map(|leaf| baseline.scan_leaf(leaf)).collect();
        let after = SystemTimeProvider.compat_unix_timestamp();

        assert_eq!(results.len(), baseline_results.len());
        for (wrapper, canonical) in results.iter().zip(baseline_results.iter()) {
            assert_scan_result_equivalent(wrapper, canonical);
            assert_scan_result_in_window(wrapper, before, after);
            assert_scan_result_in_window(canonical, before, after);
        }
    }

    #[test]
    fn test_optimized_scan_base_detector() {
        let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
        let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
        let card = receiver_card(&receiver_keys);
        let request = PaymentRequest::generate(
            &receiver_keys,
            RequestParams {
                amount: Some(111),
                expiry_seconds: 3_600,
                memo: None,
                payment_id: None,
            },
            1,
        )
        .expect("request");
        let asset_id = [21u8; 32];

        let mut mine = make_asset(111, asset_id);
        let mut sender_wallet = SenderWallet::new([13u8; 32]);
        let output = build_tx_output_unchecked(
            &card,
            Some(&request),
            &mut sender_wallet,
            &[14u8; 32],
            0,
            111,
            &asset_id,
        )
        .expect("output");

        mine.r_pub = Some(output.r_pub);
        mine.owner_tag = Some(output.owner_tag);
        mine.commitment = z00z_crypto::Commitment::from_bytes(&output.c_amount)
            .expect("commitment")
            .as_commitment()
            .clone();
        mine.enc_pack = Some(output.enc_pack);
        mine.tag16 = output.tag16;

        let mut scanner = OptimizedScanner::new(&receiver_keys, 1);
        scanner.add_request(&request);
        let mut baseline = StealthOutputScanner::from_keys(&receiver_keys);
        baseline.add_request(&request);

        let not_mine = make_asset(112, [15u8; 32]);
        let leaves = vec![mine, not_mine];
        let before = SystemTimeProvider.compat_unix_timestamp();
        let results = scanner.scan_batch(&leaves);
        let baseline_results: Vec<_> = leaves.iter().map(|leaf| baseline.scan_leaf(leaf)).collect();
        let after = SystemTimeProvider.compat_unix_timestamp();

        assert_eq!(results.len(), baseline_results.len());
        for (wrapper, canonical) in results.iter().zip(baseline_results.iter()) {
            assert_scan_result_equivalent(wrapper, canonical);
            assert_scan_result_in_window(wrapper, before, after);
            assert_scan_result_in_window(canonical, before, after);
        }
    }

    #[test]
    fn test_optimized_matches_base_detector() {
        let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
        let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
        let card = receiver_card(&receiver_keys);
        let mut scanner = OptimizedScanner::new(&receiver_keys, 3);
        let mut baseline = StealthOutputScanner::from_keys(&receiver_keys);

        let mut mine = make_asset(120, [17u8; 32]);
        let mut sender_wallet = SenderWallet::new([16u8; 32]);
        let output = build_tx_output_unchecked(
            &card,
            None,
            &mut sender_wallet,
            &[18u8; 32],
            0,
            120,
            &[17u8; 32],
        )
        .expect("output");

        mine.r_pub = Some(output.r_pub);
        mine.owner_tag = Some(output.owner_tag);
        mine.commitment = z00z_crypto::Commitment::from_bytes(&output.c_amount)
            .expect("commitment")
            .as_commitment()
            .clone();
        mine.enc_pack = Some(output.enc_pack);
        mine.tag16 = output.tag16;

        let r_pub = decode_r_pub(&output.r_pub).expect("decode r_pub");
        let dh = compute_dh_receiver(receiver_keys.reveal_view_sk(), &r_pub).expect("dh");
        let k_dh = derive_k_dh(&dh);
        let tag16 = output.tag16.expect("tag16");
        scanner
            .base
            .add_tag_context(tag16, Tag16Context { k_dh, req_id: None });
        baseline.add_tag_context(tag16, Tag16Context { k_dh, req_id: None });

        let not_mine = make_asset(121, [19u8; 32]);
        let leaves = vec![mine, not_mine];
        let before = SystemTimeProvider.compat_unix_timestamp();
        let results = scanner.scan_batch(&leaves);
        let baseline_results: Vec<_> = leaves.iter().map(|leaf| baseline.scan_leaf(leaf)).collect();
        let after = SystemTimeProvider.compat_unix_timestamp();

        assert_eq!(results.len(), baseline_results.len());
        for (wrapper, canonical) in results.iter().zip(baseline_results.iter()) {
            assert_scan_result_equivalent(wrapper, canonical);
            assert_scan_result_in_window(wrapper, before, after);
            assert_scan_result_in_window(canonical, before, after);
        }
    }

    fn receiver_card(keys: &ReceiverKeys) -> ReceiverCard {
        ReceiverCard {
            version: 1,
            owner_handle: keys.owner_handle,
            view_pk: keys.view_pk.as_bytes().try_into().expect("view pk"),
            identity_pk: keys.identity_pk.as_bytes().try_into().expect("identity pk"),
            card_id: None,
            metadata: None,
            signature: [0u8; 64],
        }
    }

    fn make_asset(amount: u64, asset_id: [u8; 32]) -> Asset {
        let mut asset = z00z_core::genesis::asset_std::asset_from_dev_cfg("z00z", 0, amount)
            .expect("std asset");
        asset.leaf_ad_id = Some(asset_id);
        asset
    }

    fn assert_scan_result_equivalent(left: &ScanResult, right: &ScanResult) {
        match (left, right) {
            (
                ScanResult::Mine {
                    wallet_output: left,
                },
                ScanResult::Mine {
                    wallet_output: right,
                },
            ) => {
                assert_eq!(left.asset_id, right.asset_id);
                assert_eq!(left.serial_id, right.serial_id);
                assert_eq!(left.pack_version, right.pack_version);
                assert_eq!(left.amount, right.amount);
                assert_eq!(left.asset_secret, right.asset_secret);
                assert_eq!(left.blinding, right.blinding);
                assert_eq!(left.memo, right.memo);
                assert_eq!(left.r_pub, right.r_pub);
                assert_eq!(left.owner_tag, right.owner_tag);
            }
            (ScanResult::NotMine, ScanResult::NotMine) => {}
            (
                ScanResult::MaybeMine {
                    tag16_match: left_tag16,
                    m1_failed: left_m1,
                },
                ScanResult::MaybeMine {
                    tag16_match: right_tag16,
                    m1_failed: right_m1,
                },
            ) => {
                assert_eq!((*left_tag16, *left_m1), (*right_tag16, *right_m1));
            }
            _ => {
                panic!("optimized scanner diverged from canonical detector: {left:?} vs {right:?}")
            }
        }
    }

    fn assert_scan_result_in_window(result: &ScanResult, before: u64, after: u64) {
        if let ScanResult::Mine { wallet_output } = result {
            assert!(
                wallet_output.decrypted_at >= before && wallet_output.decrypted_at <= after,
                "decrypted_at out of window: {} not in [{}..={}]",
                wallet_output.decrypted_at,
                before,
                after
            );
        }
    }
}
