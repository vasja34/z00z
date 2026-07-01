#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::sync::Arc;

use z00z_core::vouchers::{VoucherLifecycleV1, VoucherValidityWindowV1};
use z00z_crypto::expert::encoding::SafePassword;
use z00z_storage::settlement::{RightClass, RightLeaf, TerminalId, VoucherBackingRef, VoucherLeaf};
use z00z_wallets::{
    db::{
        object_inventory_store, open_wallet_store, ObjectInventoryStore, ObjectSeenRef,
        OwnedObjectPolicy, OwnedObjectSource, OwnedRightPayload, OwnedRightStatus,
        OwnedVoucherPayload, OwnedVoucherStatus, WalletIdentity, WalletPolicyAvailability,
    },
    domains::hashing::compute_wallet_file_id,
    rpc::methods::{AssetRpcImpl, ObjectRpcServer},
    rpc::types::common::PersistTxId,
    rpc::types::object::RuntimeObjectListFilter,
    services::{AppService, WalletService},
};

#[path = "test_inc/test_wallet_env.inc"]
mod test_common;

const TEST_PASSWORD: &str = "Aa1!bB2@cC3#dD4$eE5%";
const TEST_SEED_PHRASE_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

fn wallet_stem(wallet_id: &str) -> String {
    let hash = compute_wallet_file_id(wallet_id);
    hex::encode(&hash[..8])
}

fn wlt_path(root: &Path, wallet_id: &str) -> PathBuf {
    root.join(format!("wallet_{}.wlt", wallet_stem(wallet_id)))
}

fn identity() -> WalletIdentity {
    WalletIdentity {
        network: "p2p".to_string(),
        chain: "devnet".to_string(),
    }
}

fn policy(tag: u8) -> OwnedObjectPolicy {
    OwnedObjectPolicy {
        policy_id: Some([tag; 32]),
        availability: WalletPolicyAvailability::Missing,
        manual_review: true,
        quarantine_reason: Some("policy missing".to_string()),
    }
}

fn quarantined_voucher(
    wallet_id: z00z_wallets::rpc::types::common::PersistWalletId,
) -> OwnedVoucherPayload {
    let terminal_id = TerminalId::new([0x51; 32]);
    let mut payload = OwnedVoucherPayload {
        version: OwnedVoucherPayload::VERSION,
        wallet_id,
        account_id: Some(51),
        terminal_id,
        voucher_leaf: VoucherLeaf {
            version: 1,
            terminal_id,
            issuer_commitment: [0x52; 32],
            holder_commitment: [0x53; 32],
            beneficiary_commitment: [0x54; 32],
            refund_target_commitment: [0x55; 32],
            backing: VoucherBackingRef::ReserveCommitment([0x56; 32]),
            face_value: 50,
            remaining_value: 50,
            policy_id: [0x57; 32],
            action_pool_id: [0x58; 32],
            lifecycle: VoucherLifecycleV1::Active,
            validity: VoucherValidityWindowV1 {
                valid_from: 10,
                valid_until: 100,
            },
            receiver_must_accept: true,
            allow_reject: true,
            replay_nonce: [0x59; 32],
            disclosure_commitment: Some([0x5A; 32]),
            audit_commitment: Some([0x5B; 32]),
        },
        status: OwnedVoucherStatus::Quarantined,
        source: OwnedObjectSource::Import,
        first_seen: Some(ObjectSeenRef {
            height: Some(10),
            hash_or_root: Some(vec![0x5C; 32]),
            local_time_ms: 1_111,
        }),
        last_updated_ms: 1_222,
        scan_ref: None,
        receive_ref: Some(z00z_wallets::db::redb_store::ReceiveRef {
            request_id: Some("voucher-req".to_string()),
            receiver_handle: Some("voucher-recv".to_string()),
            import_tx_id: Some(PersistTxId("voucher-import".to_string())),
        }),
        confirmation_ref: None,
        labels: vec!["voucher".to_string(), "quarantine".to_string()],
        policy: policy(0x57),
        holder_opening: Some(vec![0x61; 8]),
        beneficiary_opening: Some(vec![0x62; 8]),
        checksum: None,
    };
    payload.checksum = Some(payload.compute_checksum());
    payload
}

fn quarantined_right(
    wallet_id: z00z_wallets::rpc::types::common::PersistWalletId,
) -> OwnedRightPayload {
    let terminal_id = TerminalId::new([0x71; 32]);
    let mut payload = OwnedRightPayload {
        version: OwnedRightPayload::VERSION,
        wallet_id,
        account_id: Some(71),
        terminal_id,
        right_leaf: RightLeaf {
            version: 1,
            terminal_id,
            right_class: RightClass::ServiceEntitlement,
            issuer_scope: [0x72; 32],
            provider_scope: [0x73; 32],
            holder_commitment: [0x74; 32],
            control_commitment: [0x75; 32],
            beneficiary_commitment: [0x76; 32],
            payload_commitment: [0x77; 32],
            valid_from: 10,
            valid_until: 100,
            challenge_from: 20,
            challenge_until: 90,
            use_nonce: [0x78; 32],
            revocation_policy_id: [0x79; 32],
            transition_policy_id: [0x7A; 32],
            challenge_policy_id: [0x7B; 32],
            disclosure_policy_id: [0x7C; 32],
            retention_policy_id: [0x7D; 32],
        },
        status: OwnedRightStatus::Quarantined,
        source: OwnedObjectSource::Import,
        first_seen: Some(ObjectSeenRef {
            height: Some(12),
            hash_or_root: Some(vec![0x7E; 32]),
            local_time_ms: 2_222,
        }),
        last_updated_ms: 2_333,
        scan_ref: None,
        receive_ref: Some(z00z_wallets::db::redb_store::ReceiveRef {
            request_id: Some("right-req".to_string()),
            receiver_handle: Some("right-recv".to_string()),
            import_tx_id: Some(PersistTxId("right-import".to_string())),
        }),
        confirmation_ref: None,
        labels: vec!["right".to_string(), "quarantine".to_string()],
        policy: policy(0x79),
        holder_opening: Some(vec![0x81; 8]),
        control_opening: Some(vec![0x82; 8]),
        beneficiary_opening: Some(vec![0x83; 8]),
        checksum: None,
    };
    payload.checksum = Some(payload.compute_checksum());
    payload
}

async fn create_wallet(
    output_dir: &Path,
) -> (
    Arc<WalletService>,
    z00z_wallets::rpc::types::common::PersistWalletId,
    PathBuf,
) {
    let service = Arc::new(WalletService::with_output_dir(output_dir.to_path_buf()));
    let app = AppService::with_wallet_service(Arc::clone(&service));
    let wallet_id = app
        .create_wallet(
            "quarantine-wallet".to_string(),
            TEST_PASSWORD.to_string(),
            Some(TEST_SEED_PHRASE_24.to_string()),
        )
        .await
        .expect("create wallet")
        .wallet_id;
    let path = wlt_path(output_dir, &wallet_id.0);
    (service, wallet_id, path)
}

fn promote_voucher(payload: &OwnedVoucherPayload) -> OwnedVoucherPayload {
    let mut promoted = payload.clone();
    promoted.status = OwnedVoucherStatus::Redeemable;
    promoted.policy.availability = WalletPolicyAvailability::Available;
    promoted.policy.manual_review = false;
    promoted.policy.quarantine_reason = None;
    promoted.checksum = Some(promoted.compute_checksum());
    promoted
}

fn promote_right(payload: &OwnedRightPayload) -> OwnedRightPayload {
    let mut promoted = payload.clone();
    promoted.status = OwnedRightStatus::Granted;
    promoted.policy.availability = WalletPolicyAvailability::Available;
    promoted.policy.manual_review = false;
    promoted.policy.quarantine_reason = None;
    promoted.checksum = Some(promoted.compute_checksum());
    promoted
}

#[tokio::test]
async fn quarantine_survives_backup_restore_and_only_changes_on_explicit_promotion() {
    let _env = test_common::WalletEnvGuard::new("p2p", "devnet");

    let src_temp = tempfile::tempdir().expect("src tempdir");
    let src_output = src_temp.path().join("wallets-src");
    let (src_service, wallet_id, src_wlt) = create_wallet(&src_output).await;

    let voucher = quarantined_voucher(wallet_id.clone());
    let right = quarantined_right(wallet_id.clone());
    let session = open_wallet_store(
        &src_wlt,
        &wallet_id,
        &SafePassword::from(TEST_PASSWORD),
        &identity(),
    )
    .expect("open source wallet");
    let _ = object_inventory_store()
        .put_voucher(&session, voucher.clone())
        .expect("store voucher");
    let _ = object_inventory_store()
        .put_right(&session, right.clone())
        .expect("store right");
    drop(session);

    let backup_dir = src_temp.path().join("backups");
    let backup = src_service
        .create_backup(
            &wallet_id,
            SafePassword::from(TEST_PASSWORD),
            Some(backup_dir.to_string_lossy().to_string()),
        )
        .await
        .expect("create backup");

    let dst_temp = tempfile::tempdir().expect("dst tempdir");
    let dst_output = dst_temp.path().join("wallets-dst");
    let dst_service = Arc::new(WalletService::with_output_dir(dst_output.clone()));
    let restored = dst_service
        .restore_backup(
            backup.backup_path,
            SafePassword::from(TEST_PASSWORD),
            Some("restored-quarantine".to_string()),
        )
        .await
        .expect("restore backup");
    dst_service
        .unlock_wallet_in_memory(&restored.wallet_id, &SafePassword::from(TEST_PASSWORD))
        .await
        .expect("unlock restored wallet");

    let rpc = AssetRpcImpl::with_wallet_service(Arc::clone(&dst_service));
    let filtered = rpc
        .list_objects(
            restored.wallet_id.clone(),
            Some(10),
            None,
            Some(RuntimeObjectListFilter {
                account_id: None,
                family: None,
                policy_availability: Some(WalletPolicyAvailability::Missing),
                holder_commitment_hex: None,
            }),
        )
        .await
        .expect("list quarantined objects");
    assert_eq!(filtered.items.len(), 2);
    for item in &filtered.items {
        let policy = item.policy.as_ref().expect("policy state");
        assert_eq!(policy.availability, WalletPolicyAvailability::Missing);
        assert_eq!(policy.quarantine_reason.as_deref(), Some("policy missing"));
    }

    let vouchers = rpc
        .list_vouchers(
            restored.wallet_id.clone(),
            Some(10),
            None,
            Some(OwnedVoucherStatus::Quarantined),
        )
        .await
        .expect("list quarantined vouchers");
    let rights = rpc
        .list_rights(
            restored.wallet_id.clone(),
            Some(10),
            None,
            Some(OwnedRightStatus::Quarantined),
        )
        .await
        .expect("list quarantined rights");
    assert_eq!(vouchers.items.len(), 1);
    assert_eq!(rights.items.len(), 1);

    dst_service
        .unregister_wallet(&restored.wallet_id)
        .await
        .expect("release restored wallet session");

    let dst_wlt = wlt_path(&dst_output, &restored.wallet_id.0);
    let session = open_wallet_store(
        &dst_wlt,
        &restored.wallet_id,
        &SafePassword::from(TEST_PASSWORD),
        &identity(),
    )
    .expect("open restored wallet");
    object_inventory_store()
        .replace_voucher(&session, promote_voucher(&voucher))
        .expect("promote voucher");
    object_inventory_store()
        .replace_right(&session, promote_right(&right))
        .expect("promote right");
    drop(session);

    dst_service
        .unlock_wallet_in_memory(&restored.wallet_id, &SafePassword::from(TEST_PASSWORD))
        .await
        .expect("unlock restored wallet after promotion");

    let missing_after = rpc
        .list_objects(
            restored.wallet_id.clone(),
            Some(10),
            None,
            Some(RuntimeObjectListFilter {
                account_id: None,
                family: None,
                policy_availability: Some(WalletPolicyAvailability::Missing),
                holder_commitment_hex: None,
            }),
        )
        .await
        .expect("list missing-policy objects after promotion");
    assert!(
        missing_after.items.is_empty(),
        "promotion must not leave rows in missing-policy quarantine"
    );

    let redeemable = rpc
        .list_vouchers(
            restored.wallet_id.clone(),
            Some(10),
            None,
            Some(OwnedVoucherStatus::Redeemable),
        )
        .await
        .expect("list promoted vouchers");
    let granted = rpc
        .list_rights(
            restored.wallet_id.clone(),
            Some(10),
            None,
            Some(OwnedRightStatus::Granted),
        )
        .await
        .expect("list promoted rights");
    assert_eq!(redeemable.items.len(), 1);
    assert_eq!(granted.items.len(), 1);
}
