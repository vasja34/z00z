//! Shared voucher/right fixtures for unit tests.

use crate::db::redb_store::{ConfirmRef, ReceiveRef, ScanRef};
use crate::db::{
    ObjectSeenRef, OwnedObjectPolicy, OwnedObjectSource, OwnedRightPayload, OwnedRightStatus,
    OwnedVoucherPayload, OwnedVoucherStatus, WalletPolicyAvailability,
};
use crate::rpc::types::common::{PersistTxId, PersistWalletId};
use crate::tx::{validator_mandate_lock_payload_commitment, VALIDATOR_MANDATE_LOCK_PROFILE_ID};
use z00z_core::vouchers::{VoucherLifecycleV1, VoucherValidityWindowV1};
use z00z_storage::settlement::{RightClass, RightLeaf, TerminalId, VoucherBackingRef, VoucherLeaf};

pub(crate) fn test_owned_object_policy_available(policy_id: [u8; 32]) -> OwnedObjectPolicy {
    OwnedObjectPolicy {
        policy_id: Some(policy_id),
        availability: WalletPolicyAvailability::Available,
        manual_review: false,
        quarantine_reason: None,
    }
}

pub(crate) fn test_owned_voucher_payload(
    wallet_id: PersistWalletId,
    tag: u8,
) -> OwnedVoucherPayload {
    let terminal_id = TerminalId::new([tag; 32]);
    let mut payload = OwnedVoucherPayload {
        version: OwnedVoucherPayload::VERSION,
        wallet_id,
        account_id: Some(tag as u128),
        terminal_id,
        voucher_leaf: VoucherLeaf {
            version: 1,
            terminal_id,
            issuer_commitment: [tag; 32],
            holder_commitment: [tag.wrapping_add(1); 32],
            beneficiary_commitment: [tag.wrapping_add(2); 32],
            refund_target_commitment: [tag.wrapping_add(3); 32],
            backing: VoucherBackingRef::ReserveCommitment([tag.wrapping_add(4); 32]),
            face_value: 50,
            remaining_value: 50,
            policy_id: [tag.wrapping_add(5); 32],
            action_pool_id: [tag.wrapping_add(6); 32],
            lifecycle: VoucherLifecycleV1::Active,
            validity: VoucherValidityWindowV1 {
                valid_from: 10,
                valid_until: 100,
            },
            receiver_must_accept: true,
            allow_reject: true,
            replay_nonce: [tag.wrapping_add(7); 32],
            disclosure_commitment: Some([tag.wrapping_add(8); 32]),
            audit_commitment: Some([tag.wrapping_add(9); 32]),
        },
        status: OwnedVoucherStatus::Redeemable,
        source: OwnedObjectSource::Import,
        first_seen: Some(ObjectSeenRef {
            height: Some(10),
            hash_or_root: Some(vec![tag; 32]),
            local_time_ms: 1_111,
        }),
        last_updated_ms: 1_222,
        scan_ref: Some(ScanRef {
            start_height: 8,
            end_height: 10,
            cursor_hash: vec![tag.wrapping_add(10); 32],
        }),
        receive_ref: Some(ReceiveRef {
            request_id: Some(format!("voucher-req-{tag}")),
            receiver_handle: Some(format!("voucher-recv-{tag}")),
            import_tx_id: Some(PersistTxId(format!("voucher-import-{tag}"))),
        }),
        confirmation_ref: Some(ConfirmRef {
            checkpoint_id_hex: Some(format!("voucher-cp-{tag}")),
            state_root_hex: Some(format!("voucher-root-{tag}")),
            evidence_id: Some(format!("voucher-ev-{tag}")),
        }),
        labels: vec!["voucher".to_string(), format!("tag-{tag}")],
        policy: test_owned_object_policy_available([tag.wrapping_add(5); 32]),
        holder_opening: Some(vec![tag; 8]),
        beneficiary_opening: Some(vec![tag.wrapping_add(1); 8]),
        checksum: None,
    };
    payload.checksum = Some(payload.compute_checksum());
    payload
}

pub(crate) fn test_owned_right_payload(wallet_id: PersistWalletId, tag: u8) -> OwnedRightPayload {
    let terminal_id = TerminalId::new([tag; 32]);
    let mut payload = OwnedRightPayload {
        version: OwnedRightPayload::VERSION,
        wallet_id,
        account_id: Some((tag as u128) + 100),
        terminal_id,
        right_leaf: RightLeaf {
            version: 1,
            terminal_id,
            right_class: RightClass::ServiceEntitlement,
            issuer_scope: [tag; 32],
            provider_scope: [tag.wrapping_add(1); 32],
            holder_commitment: [tag.wrapping_add(2); 32],
            control_commitment: [tag.wrapping_add(3); 32],
            beneficiary_commitment: [tag.wrapping_add(4); 32],
            payload_commitment: [tag.wrapping_add(5); 32],
            valid_from: 10,
            valid_until: 100,
            challenge_from: 20,
            challenge_until: 90,
            use_nonce: [tag.wrapping_add(6); 32],
            revocation_policy_id: [tag.wrapping_add(7); 32],
            transition_policy_id: [tag.wrapping_add(8); 32],
            challenge_policy_id: [tag.wrapping_add(9); 32],
            disclosure_policy_id: [tag.wrapping_add(10); 32],
            retention_policy_id: [tag.wrapping_add(11); 32],
        },
        status: OwnedRightStatus::Granted,
        source: OwnedObjectSource::Import,
        first_seen: Some(ObjectSeenRef {
            height: Some(12),
            hash_or_root: Some(vec![tag.wrapping_add(12); 32]),
            local_time_ms: 2_222,
        }),
        last_updated_ms: 2_333,
        scan_ref: Some(ScanRef {
            start_height: 10,
            end_height: 12,
            cursor_hash: vec![tag.wrapping_add(13); 32],
        }),
        receive_ref: Some(ReceiveRef {
            request_id: Some(format!("right-req-{tag}")),
            receiver_handle: Some(format!("right-recv-{tag}")),
            import_tx_id: Some(PersistTxId(format!("right-import-{tag}"))),
        }),
        confirmation_ref: Some(ConfirmRef {
            checkpoint_id_hex: Some(format!("right-cp-{tag}")),
            state_root_hex: Some(format!("right-root-{tag}")),
            evidence_id: Some(format!("right-ev-{tag}")),
        }),
        labels: vec!["right".to_string(), format!("tag-{tag}")],
        policy: test_owned_object_policy_available([tag.wrapping_add(8); 32]),
        holder_opening: Some(vec![tag.wrapping_add(14); 8]),
        control_opening: Some(vec![tag.wrapping_add(15); 8]),
        beneficiary_opening: Some(vec![tag.wrapping_add(16); 8]),
        checksum: None,
    };
    payload.checksum = Some(payload.compute_checksum());
    payload
}

pub(crate) fn test_mandate_lock_payload(
    wallet_id: PersistWalletId,
    tag: u8,
    asset_id: [u8; 32],
    amount: u64,
) -> OwnedRightPayload {
    let terminal_id = TerminalId::new([tag.wrapping_add(32); 32]);
    let mut payload = OwnedRightPayload {
        version: OwnedRightPayload::VERSION,
        wallet_id,
        account_id: Some((tag as u128) + 200),
        terminal_id,
        right_leaf: RightLeaf {
            version: 1,
            terminal_id,
            right_class: RightClass::ValidatorMandate,
            issuer_scope: [tag; 32],
            provider_scope: [tag.wrapping_add(1); 32],
            holder_commitment: [tag.wrapping_add(2); 32],
            control_commitment: [tag.wrapping_add(3); 32],
            beneficiary_commitment: [tag.wrapping_add(4); 32],
            payload_commitment: [0u8; 32],
            valid_from: 10,
            valid_until: 100,
            challenge_from: 101,
            challenge_until: 140,
            use_nonce: [tag.wrapping_add(6); 32],
            revocation_policy_id: [tag.wrapping_add(7); 32],
            transition_policy_id: [tag.wrapping_add(8); 32],
            challenge_policy_id: [tag.wrapping_add(9); 32],
            disclosure_policy_id: [tag.wrapping_add(10); 32],
            retention_policy_id: [tag.wrapping_add(11); 32],
        },
        status: OwnedRightStatus::Granted,
        source: OwnedObjectSource::Import,
        first_seen: Some(ObjectSeenRef {
            height: Some(12),
            hash_or_root: Some(vec![tag.wrapping_add(12); 32]),
            local_time_ms: 2_222,
        }),
        last_updated_ms: 2_333,
        scan_ref: Some(ScanRef {
            start_height: 10,
            end_height: 12,
            cursor_hash: vec![tag.wrapping_add(13); 32],
        }),
        receive_ref: Some(ReceiveRef {
            request_id: Some(format!("validator-lock-req-{tag}")),
            receiver_handle: Some(format!("validator-lock-recv-{tag}")),
            import_tx_id: Some(PersistTxId(format!("validator-lock-import-{tag}"))),
        }),
        confirmation_ref: Some(ConfirmRef {
            checkpoint_id_hex: Some(format!("validator-lock-cp-{tag}")),
            state_root_hex: Some(format!("validator-lock-root-{tag}")),
            evidence_id: Some(format!("validator-lock-ev-{tag}")),
        }),
        labels: vec![
            "right".to_string(),
            VALIDATOR_MANDATE_LOCK_PROFILE_ID.to_string(),
            format!("tag-{tag}"),
        ],
        policy: test_owned_object_policy_available([tag.wrapping_add(8); 32]),
        holder_opening: Some(vec![tag.wrapping_add(14); 8]),
        control_opening: Some(vec![tag.wrapping_add(15); 8]),
        beneficiary_opening: Some(vec![tag.wrapping_add(16); 8]),
        checksum: None,
    };
    payload.right_leaf.payload_commitment =
        validator_mandate_lock_payload_commitment(&asset_id, amount, &payload.right_leaf);
    payload.checksum = Some(payload.compute_checksum());
    payload
}
