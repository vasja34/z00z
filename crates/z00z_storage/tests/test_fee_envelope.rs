use tempfile::tempdir;
use z00z_core::assets::{AssetLeaf, AssetPackPlain};
use z00z_core::vouchers::{VoucherLifecycleV1, VoucherValidityWindowV1};
use z00z_crypto::ZkPackEncrypted;
use z00z_storage::settlement::{
    DefinitionId, FeeActorCtx, FeeEnvelope, FeeReplayKey, FeeSupportCtx, SerialId, SettlementLeaf,
    SettlementPath, SettlementStore, StoreItem, StoreOp, TerminalId, TerminalLeaf,
    VoucherActionCtx, VoucherBackingRef, VoucherLeaf,
};

const HJMT_JOURNAL_TABLE: redb::TableDefinition<&[u8], &[u8]> =
    redb::TableDefinition::new("settlement_hjmt_journal");
const DB_FILE: &str = "settlement_state.redb";
const PROOF_SRC: &str = include_str!("../src/settlement/proof.rs");

fn bytes(value: u8) -> [u8; 32] {
    [value; 32]
}

fn path(definition: u8, serial: u32, asset: u8) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new(bytes(definition)),
        SerialId::new(serial),
        TerminalId::new(bytes(asset)),
    )
}

fn leaf(path: SettlementPath, value: u64) -> TerminalLeaf {
    let payload = AssetPackPlain {
        value,
        blinding: bytes(3),
        s_out: bytes(4),
    }
    .to_bytes();

    AssetLeaf {
        asset_id: path.terminal_id().into_bytes(),
        serial_id: path.serial_id.get(),
        r_pub: bytes(1),
        owner_tag: bytes(2),
        c_amount: bytes(5),
        enc_pack: ZkPackEncrypted {
            version: 1,
            ciphertext: payload,
            tag: [0u8; 16],
        },
        range_proof: vec![9u8; 4],
        tag16: 11,
    }
    .into()
}

fn item(path: SettlementPath, value: u64) -> StoreItem {
    StoreItem::new(path, leaf(path, value)).expect("store item")
}

fn voucher_path(mark: u8) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new(bytes(mark.wrapping_add(20))),
        SerialId::new(u32::from(mark) + 1),
        TerminalId::new(bytes(mark)),
    )
}

fn voucher_leaf(mark: u8, support_ref: [u8; 32]) -> VoucherLeaf {
    VoucherLeaf {
        version: 1,
        terminal_id: TerminalId::new(bytes(mark)),
        issuer_commitment: bytes(mark.wrapping_add(1)),
        holder_commitment: bytes(mark.wrapping_add(2)),
        beneficiary_commitment: bytes(mark.wrapping_add(3)),
        refund_target_commitment: bytes(mark.wrapping_add(4)),
        backing: VoucherBackingRef::ReserveCommitment(support_ref),
        face_value: 50,
        remaining_value: 50,
        policy_id: bytes(mark.wrapping_add(5)),
        action_pool_id: bytes(mark.wrapping_add(6)),
        lifecycle: VoucherLifecycleV1::PendingAcceptance,
        validity: VoucherValidityWindowV1 {
            valid_from: 10,
            valid_until: 40,
        },
        receiver_must_accept: true,
        allow_reject: true,
        replay_nonce: bytes(mark.wrapping_add(7)),
        disclosure_commitment: None,
        audit_commitment: None,
    }
}

fn fee_envelope(mark: u8, support: FeeSupportCtx) -> FeeEnvelope {
    let budget_units = support.required_units;
    let support_ref = Some(bytes(mark.wrapping_add(8)));
    FeeEnvelope {
        version: 1,
        payer_commitment: bytes(mark),
        sponsor_commitment: bytes(mark.wrapping_add(1)),
        budget_units,
        budget_commitment: FeeEnvelope::budget_bind(budget_units, support_ref),
        domain_id: support.domain_id,
        expires_at: 50,
        nonce: bytes(mark.wrapping_add(4)),
        transition_id: support.transition_id,
        replay_key: bytes(mark.wrapping_add(6)),
        support_ref,
        failure_policy_id: bytes(mark.wrapping_add(7)),
    }
}

fn support(mark: u8) -> FeeSupportCtx {
    FeeSupportCtx {
        required_units: u64::from(mark) + 2,
        domain_id: bytes(mark.wrapping_add(3)),
        transition_id: bytes(mark.wrapping_add(5)),
    }
}

fn actor(mark: u8, now: u64) -> FeeActorCtx {
    FeeActorCtx {
        now,
        payer_commitment: Some(bytes(mark)),
        sponsor_commitment: Some(bytes(mark.wrapping_add(1))),
    }
}

fn journal_bytes(root: &std::path::Path, version: u64) -> Vec<u8> {
    let db = redb::Database::create(root.join(DB_FILE)).expect("open db");
    let read = db.begin_read().expect("begin read");
    let table = read.open_table(HJMT_JOURNAL_TABLE).expect("journal table");
    table
        .get(version.to_be_bytes().as_slice())
        .expect("journal get")
        .expect("journal row")
        .value()
        .to_vec()
}

#[test]
fn test_valid_accepts_pre_xfer() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let mut store = SettlementStore::load(temp.path())?;
    let seeded_root = store.put_settlement_item(item(path(7, 1, 9), 700))?;
    let ops = vec![StoreOp::Put(Box::new(item(path(7, 1, 9), 701)))];
    let fee_support = store.fee_support_ctx(&ops)?;

    let envelope = fee_envelope(10, fee_support);
    let next_root = store.apply_fee_ops(ops, envelope, actor(10, 12))?;

    assert_ne!(next_root.into_bytes(), seeded_root.into_bytes());
    assert_eq!(
        store.settlement_root()?.into_bytes(),
        next_root.into_bytes()
    );
    assert!(store
        .fee_replay_rec(&FeeReplayKey::new(bytes(16)))?
        .is_some());
    Ok(())
}

#[test]
fn test_valid_replay_rejects_reload() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let mut store = SettlementStore::load(temp.path())?;
    let seeded_root = store.put_settlement_item(item(path(6, 1, 7), 600))?;
    let ops = vec![StoreOp::Put(Box::new(item(path(6, 1, 7), 601)))];
    let envelope = fee_envelope(12, store.fee_support_ctx(&ops)?);

    let next_root = store.apply_fee_ops(ops.clone(), envelope, actor(12, 12))?;

    assert_ne!(next_root.into_bytes(), seeded_root.into_bytes());
    assert_eq!(
        store.settlement_root()?.into_bytes(),
        next_root.into_bytes()
    );
    assert!(store
        .fee_replay_rec(&FeeReplayKey::new(bytes(18)))?
        .is_some());

    drop(store);

    let mut reloaded = SettlementStore::load(temp.path())?;
    assert_eq!(
        reloaded.settlement_root()?.into_bytes(),
        next_root.into_bytes()
    );
    let err = reloaded
        .apply_fee_ops(ops, envelope, actor(12, 12))
        .expect_err("replayed fee support must reject after reload");
    let err = err.to_string();
    assert!(
        err.contains("replay binding is invalid")
            || err.contains("domain binding mismatch")
            || err.contains("transition binding mismatch"),
        "unexpected fee rejection: {err}"
    );
    Ok(())
}

#[test]
fn test_envelope_rejects_pre_mutation() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let seeded_root = {
        let mut store = SettlementStore::load(temp.path())?;
        store.put_settlement_item(item(path(8, 1, 9), 800))?
    };
    let before_journal = journal_bytes(temp.path(), 1);
    let ops = vec![StoreOp::Put(Box::new(item(path(8, 1, 9), 801)))];

    let mut envelope = fee_envelope(
        20,
        SettlementStore::load(temp.path())?.fee_support_ctx(&ops)?,
    );
    envelope.budget_units = 0;
    envelope.budget_commitment = FeeEnvelope::budget_bind(0, envelope.support_ref);

    {
        let mut store = SettlementStore::load(temp.path())?;
        let err = store
            .apply_fee_ops(ops, envelope, actor(20, 12))
            .expect_err("budget mismatch must reject");
        assert!(err.to_string().contains("budget is insufficient"));
        assert_eq!(
            store.settlement_root()?.into_bytes(),
            seeded_root.into_bytes()
        );
        assert!(store
            .fee_replay_rec(&FeeReplayKey::new(bytes(26)))?
            .is_none());
    }

    assert_eq!(journal_bytes(temp.path(), 1), before_journal);
    Ok(())
}

#[test]
fn test_expired_fee_support_rejects() {
    let envelope = fee_envelope(30, support(30));
    let err = envelope
        .validate_support(support(30), actor(30, 60), false)
        .expect_err("stale fee support must reject");
    assert_eq!(err.to_string(), "fee envelope expiry is stale");
}

#[test]
fn test_wrong_sponsor_rejects() {
    let envelope = fee_envelope(40, support(40));
    let mut fee_actor = actor(40, 12);
    fee_actor.sponsor_commitment = Some(bytes(90));
    let err = envelope
        .validate_support(support(40), fee_actor, false)
        .expect_err("wrong sponsor must reject");
    assert_eq!(err.to_string(), "fee envelope sponsor binding mismatch");
}

#[test]
fn test_wrong_transition_binding_rejects() {
    let envelope = fee_envelope(50, support(50));
    let mut fee_support = support(50);
    fee_support.transition_id = bytes(77);
    let err = envelope
        .validate_support(fee_support, actor(50, 12), false)
        .expect_err("wrong transition must reject");
    assert_eq!(err.to_string(), "fee envelope transition binding mismatch");
}

#[test]
fn test_wrong_domain_binding_rejects() {
    let envelope = fee_envelope(52, support(52));
    let mut fee_support = support(52);
    fee_support.domain_id = bytes(91);
    let err = envelope
        .validate_support(fee_support, actor(52, 12), false)
        .expect_err("wrong domain must reject");
    assert_eq!(err.to_string(), "fee envelope domain binding mismatch");
}

#[test]
fn test_support_needs_full_xfer() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let mut store = SettlementStore::load(temp.path())?;
    store.put_settlement_item(item(path(11, 1, 9), 1100))?;

    let err = store
        .apply_fee_ops(Vec::new(), fee_envelope(54, support(54)), actor(54, 12))
        .expect_err("fee support must not consume outside an asset transition");
    assert!(err
        .to_string()
        .contains("fee support commit requires settlement ops"));
    assert!(store
        .fee_replay_rec(&FeeReplayKey::new(bytes(60)))?
        .is_none());
    Ok(())
}

#[test]
fn test_support_stays_off_verifier() {
    assert!(
        !PROOF_SRC.contains("FeeEnvelope"),
        "proof verifier surface must stay free of fee support authority"
    );
}

#[test]
fn test_support_keeps_blob_surface() -> Result<(), Box<dyn std::error::Error>> {
    let mut plain_store = SettlementStore::new();
    let mut fee_store = SettlementStore::new();
    let target = path(13, 1, 9);
    let target_settlement = target;

    plain_store.put_settlement_item(item(target, 1_300))?;
    fee_store.put_settlement_item(item(target, 1_300))?;

    let plain_root = plain_store.put_settlement_item(item(target, 1_301))?;
    let fee_ops = vec![StoreOp::Put(Box::new(item(target, 1_301)))];
    let fee_root = fee_store.apply_fee_ops(
        fee_ops.clone(),
        fee_envelope(90, fee_store.fee_support_ctx(&fee_ops)?),
        actor(90, 12),
    )?;

    assert_eq!(fee_root.into_bytes(), plain_root.into_bytes());

    let plain_blob = plain_store.settlement_proof_blob(&target_settlement)?;
    let fee_blob = fee_store.settlement_proof_blob(&target_settlement)?;
    assert_eq!(fee_blob.item(), plain_blob.item());
    assert_eq!(
        fee_blob.terminal_leaf_hash(),
        plain_blob.terminal_leaf_hash()
    );
    assert_eq!(fee_blob.backend_root(), plain_blob.backend_root());
    assert_eq!(fee_blob.root_bind_ver(), plain_blob.root_bind_ver());
    assert_eq!(fee_blob.root_bind(), plain_blob.root_bind());
    assert_eq!(fee_blob.definition_proof(), plain_blob.definition_proof());
    assert_eq!(fee_blob.serial_proof(), plain_blob.serial_proof());
    assert_eq!(fee_blob.terminal_proof(), plain_blob.terminal_proof());
    Ok(())
}

#[test]
fn test_support_cannot_back_voucher() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let mut store = SettlementStore::load(temp.path())?;
    let support_ref = bytes(100);
    let path = voucher_path(92);
    let voucher = voucher_leaf(92, support_ref);
    let ops = vec![StoreOp::Put(Box::new(StoreItem::new(
        path,
        SettlementLeaf::Voucher(voucher.clone()),
    )?))];
    let envelope = fee_envelope(92, store.fee_support_ctx(&ops)?);
    assert_eq!(envelope.support_ref, Some(support_ref));

    let err = store
        .issue_voucher_with_fee(
            None,
            path,
            voucher,
            VoucherActionCtx::default(),
            envelope,
            actor(92, 12),
        )
        .expect_err("fee support must not become voucher backing");

    assert!(err
        .to_string()
        .contains("fee support must not become voucher backing"));
    Ok(())
}
