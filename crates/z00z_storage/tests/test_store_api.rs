use z00z_core::assets::AssetLeaf;
use z00z_core::vouchers::{VoucherLifecycleV1, VoucherValidityWindowV1};
use z00z_storage::settlement::{
    DefinitionId, FeeActorCtx, FeeEnvelope, FeeErr, FeeSupportCtx, ModelErr, RightActionCtx,
    RightClass, RightErr, RightLeaf, SerialId, SettlementActionV1, SettlementLeaf,
    SettlementListReq, SettlementLookup, SettlementPath, SettlementStore, SettlementStoreError,
    StoreItem, StoreOp, TerminalId, TerminalLeaf, VoucherAction, VoucherActionCtx,
    VoucherBackingRef, VoucherLeaf,
};

fn bytes(value: u8) -> [u8; 32] {
    [value; 32]
}

fn asset_item(mark: u8) -> StoreItem {
    let core = AssetLeaf::dummy_for_scan(u32::from(mark));
    let leaf = TerminalLeaf::from(core.clone());
    let path = SettlementPath::new(
        DefinitionId::new(bytes(mark)),
        SerialId::new(core.serial_id),
        TerminalId::new(core.asset_id),
    );
    StoreItem::new(path, leaf).expect("asset item")
}

fn asset_item_at(path: SettlementPath, mark: u8) -> StoreItem {
    let mut core = AssetLeaf::dummy_for_scan(path.serial_id.get());
    core.asset_id = path.terminal_id().into_bytes();
    core.serial_id = path.serial_id.get();
    core.r_pub = bytes(mark.wrapping_add(1));
    core.owner_tag = bytes(mark.wrapping_add(2));
    core.c_amount = bytes(mark.wrapping_add(3));
    StoreItem::new(path, TerminalLeaf::from(core)).expect("asset item at path")
}

fn right_path(mark: u8) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new(bytes(mark.wrapping_add(1))),
        SerialId::new(u32::from(mark) + 1),
        TerminalId::new(bytes(mark)),
    )
}

fn right_leaf(mark: u8) -> RightLeaf {
    right_leaf_with_class(mark, RightClass::MachineCapability)
}

fn right_leaf_with_class(mark: u8, right_class: RightClass) -> RightLeaf {
    RightLeaf {
        version: 1,
        terminal_id: TerminalId::new(bytes(mark)),
        right_class,
        issuer_scope: bytes(mark.wrapping_add(1)),
        provider_scope: bytes(mark.wrapping_add(2)),
        holder_commitment: bytes(mark.wrapping_add(3)),
        control_commitment: bytes(mark.wrapping_add(4)),
        beneficiary_commitment: bytes(mark.wrapping_add(5)),
        payload_commitment: bytes(mark.wrapping_add(6)),
        valid_from: 10,
        valid_until: 20,
        challenge_from: 12,
        challenge_until: 18,
        use_nonce: bytes(mark.wrapping_add(6)),
        revocation_policy_id: bytes(mark.wrapping_add(7)),
        transition_policy_id: bytes(mark.wrapping_add(8)),
        challenge_policy_id: bytes(mark.wrapping_add(9)),
        disclosure_policy_id: bytes(mark.wrapping_add(10)),
        retention_policy_id: bytes(mark.wrapping_add(11)),
    }
}

fn voucher_path(mark: u8) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new(bytes(mark.wrapping_add(50))),
        SerialId::new(u32::from(mark) + 1),
        TerminalId::new(bytes(mark)),
    )
}

fn voucher_leaf(mark: u8) -> VoucherLeaf {
    VoucherLeaf {
        version: 1,
        terminal_id: TerminalId::new(bytes(mark)),
        issuer_commitment: bytes(mark.wrapping_add(11)),
        holder_commitment: bytes(mark.wrapping_add(12)),
        beneficiary_commitment: bytes(mark.wrapping_add(13)),
        refund_target_commitment: bytes(mark.wrapping_add(14)),
        backing: VoucherBackingRef::ReserveCommitment(bytes(mark.wrapping_add(15))),
        face_value: 95,
        remaining_value: 95,
        policy_id: bytes(mark.wrapping_add(16)),
        action_pool_id: bytes(mark.wrapping_add(17)),
        lifecycle: VoucherLifecycleV1::PendingAcceptance,
        validity: VoucherValidityWindowV1 {
            valid_from: 10,
            valid_until: 40,
        },
        receiver_must_accept: true,
        allow_reject: true,
        replay_nonce: bytes(mark.wrapping_add(18)),
        disclosure_commitment: Some(bytes(mark.wrapping_add(19))),
        audit_commitment: Some(bytes(mark.wrapping_add(20))),
    }
}

fn voucher_ctx(leaf: &VoucherLeaf, now: u64) -> VoucherActionCtx {
    VoucherActionCtx {
        now,
        expected_holder: Some(leaf.holder_commitment),
        expected_beneficiary: Some(leaf.beneficiary_commitment),
        expected_refund_target: Some(leaf.refund_target_commitment),
        ..VoucherActionCtx::default()
    }
}

fn refund_asset_path(leaf: &VoucherLeaf, fallback_serial: u32) -> SettlementPath {
    let (definition_id, serial_id) = match leaf.backing {
        VoucherBackingRef::ConsumedAsset {
            definition_id,
            serial_id,
        } => (definition_id, serial_id),
        VoucherBackingRef::ReserveCommitment(backing)
        | VoucherBackingRef::GenesisReserve(backing) => (backing, fallback_serial),
    };
    SettlementPath::new(
        DefinitionId::new(definition_id),
        SerialId::new(serial_id),
        TerminalId::new(leaf.refund_target_commitment),
    )
}

fn transfer_leaf(prior: RightLeaf, mark: u8) -> RightLeaf {
    let mut next = prior;
    next.holder_commitment = bytes(mark.wrapping_add(30));
    next.beneficiary_commitment = bytes(mark.wrapping_add(31));
    next
}

fn right_ctx(leaf: &RightLeaf, now: u64) -> RightActionCtx {
    RightActionCtx {
        now,
        expected_holder: Some(leaf.holder_commitment),
        expected_control: Some(leaf.control_commitment),
        ..RightActionCtx::default()
    }
}

fn fee_actor(mark: u8, now: u64) -> FeeActorCtx {
    FeeActorCtx {
        now,
        payer_commitment: Some(bytes(mark.wrapping_add(40))),
        sponsor_commitment: None,
    }
}

fn fee_envelope(mark: u8, support: FeeSupportCtx) -> FeeEnvelope {
    let support_ref = Some(bytes(mark.wrapping_add(41)));
    let budget_units = support.required_units.saturating_add(1);
    FeeEnvelope {
        version: 1,
        payer_commitment: bytes(mark.wrapping_add(40)),
        sponsor_commitment: [0u8; 32],
        budget_units,
        budget_commitment: FeeEnvelope::budget_bind(budget_units, support_ref),
        domain_id: support.domain_id,
        expires_at: 80,
        nonce: bytes(mark.wrapping_add(42)),
        transition_id: support.transition_id,
        replay_key: bytes(mark.wrapping_add(43)),
        support_ref,
        failure_policy_id: bytes(mark.wrapping_add(44)),
    }
}

fn fee_put_ops(
    path: SettlementPath,
    leaf: RightLeaf,
) -> Result<Vec<StoreOp>, SettlementStoreError> {
    Ok(vec![StoreOp::Put(Box::new(StoreItem::new(path, leaf)?))])
}

fn fee_del_ops(path: SettlementPath) -> Vec<StoreOp> {
    vec![StoreOp::Delete(path)]
}

fn voucher_put_ops(
    path: SettlementPath,
    leaf: VoucherLeaf,
) -> Result<Vec<StoreOp>, SettlementStoreError> {
    Ok(vec![StoreOp::Put(Box::new(StoreItem::new(
        path,
        SettlementLeaf::Voucher(leaf),
    )?))])
}

fn assert_wrong_leaf_family(err: SettlementStoreError) {
    match err {
        SettlementStoreError::Model(ModelErr::WrongLeafFamily) => {}
        other => panic!("expected WrongLeafFamily, got {other:?}"),
    }
}

fn assert_fee_required(err: SettlementStoreError) {
    match err {
        SettlementStoreError::Fee(FeeErr::SupportRequired) => {}
        other => panic!("expected SupportRequired, got {other:?}"),
    }
}

fn assert_holder_control_mix(err: SettlementStoreError) {
    match err {
        SettlementStoreError::Model(ModelErr::Right(RightErr::HolderControlMix)) => {}
        other => panic!("expected HolderControlMix, got {other:?}"),
    }
}

#[test]
fn test_store_api_asset_right() -> Result<(), Box<dyn std::error::Error>> {
    let mut store = SettlementStore::new();

    let asset_item = asset_item(11);
    let right_path = right_path(21);
    let right = right_leaf(21);

    let _ = store.put_settlement_item(asset_item.clone())?;
    let right_root = store.create_right_with_fee(
        right_path,
        right,
        right_ctx(&right, 15),
        fee_envelope(21, store.fee_support_ctx(&fee_put_ops(right_path, right)?)?),
        fee_actor(21, 15),
    )?;

    let loaded_asset = store
        .get_settlement_item(&asset_item.path())?
        .expect("asset item present");
    let loaded_right = store
        .lookup_settlement(SettlementLookup::Terminal(right.terminal_id))?
        .expect("right item present");

    assert!(matches!(loaded_asset.leaf(), SettlementLeaf::Terminal(_)));
    assert!(matches!(loaded_right.leaf(), SettlementLeaf::Right(_)));
    assert_eq!(loaded_right.path(), right_path);

    let page = store.list_settlement(SettlementListReq::all(10))?;
    assert_eq!(page.items().len(), 2);
    assert!(page
        .items()
        .iter()
        .any(|item| matches!(item.leaf(), SettlementLeaf::Terminal(_))));
    assert!(page
        .items()
        .iter()
        .any(|item| matches!(item.leaf(), SettlementLeaf::Right(_))));

    let proof = store.settlement_proof_item(&right_path)?;
    assert_eq!(proof.leaf(), loaded_right.leaf());
    assert_eq!(store.settlement_root()?, right_root);

    Ok(())
}

#[test]
fn test_right_class_list() -> Result<(), Box<dyn std::error::Error>> {
    let mut store = SettlementStore::new();

    let _ = store.put_settlement_item(asset_item(31))?;

    let machine_path = right_path(41);
    let machine_right = right_leaf_with_class(41, RightClass::MachineCapability);
    let _ = store.create_right_with_fee(
        machine_path,
        machine_right,
        right_ctx(&machine_right, 15),
        fee_envelope(
            41,
            store.fee_support_ctx(&fee_put_ops(machine_path, machine_right)?)?,
        ),
        fee_actor(41, 15),
    )?;

    let data_path = right_path(42);
    let data_right = right_leaf_with_class(42, RightClass::DataAccess);
    let _ = store.create_right_with_fee(
        data_path,
        data_right,
        right_ctx(&data_right, 15),
        fee_envelope(
            42,
            store.fee_support_ctx(&fee_put_ops(data_path, data_right)?)?,
        ),
        fee_actor(42, 15),
    )?;

    let page = store.list_settlement(SettlementListReq::for_right_class(
        RightClass::DataAccess,
        10,
    ))?;
    assert_eq!(page.items().len(), 1);
    assert_eq!(page.items()[0].path(), data_path);
    assert_eq!(
        page.items()[0].right_leaf()?.right_class,
        RightClass::DataAccess
    );

    let machine_page = store.list_settlement(SettlementListReq::for_right_class(
        RightClass::MachineCapability,
        10,
    ))?;
    assert_eq!(machine_page.items().len(), 1);
    assert_eq!(machine_page.items()[0].path(), machine_path);
    assert_eq!(
        machine_page.items()[0].right_leaf()?.right_class,
        RightClass::MachineCapability
    );

    Ok(())
}

#[test]
fn test_store_api_right_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let mut store = SettlementStore::new();

    let asset = asset_item(51);
    let asset_root = store.put_settlement_item(asset.clone())?;
    assert_wrong_leaf_family(
        store
            .consume_right(asset.path(), RightActionCtx::default())
            .expect_err("asset path must not be consumed as right"),
    );
    assert_eq!(store.settlement_root()?, asset_root);

    let transfer_path = right_path(61);
    let created = right_leaf(61);
    let _ = store.create_right_with_fee(
        transfer_path,
        created,
        right_ctx(&created, 15),
        fee_envelope(
            61,
            store.fee_support_ctx(&fee_put_ops(transfer_path, created)?)?,
        ),
        fee_actor(61, 15),
    )?;
    let transferred = transfer_leaf(created, 61);
    let _ = store.transfer_right_with_fee(
        transfer_path,
        transferred,
        right_ctx(&transferred, 15),
        fee_envelope(
            62,
            store.fee_support_ctx(&fee_put_ops(transfer_path, transferred)?)?,
        ),
        fee_actor(62, 15),
    )?;
    let challenged = transfer_leaf(transferred, 62);
    let _ = store.challenge_right_with_fee(
        transfer_path,
        challenged,
        right_ctx(&challenged, 15),
        fee_envelope(
            63,
            store.fee_support_ctx(&fee_put_ops(transfer_path, challenged)?)?,
        ),
        fee_actor(63, 15),
    )?;
    let _ = store.consume_right_with_fee(
        transfer_path,
        right_ctx(&challenged, 15),
        fee_envelope(64, store.fee_support_ctx(&fee_del_ops(transfer_path))?),
        fee_actor(64, 15),
    )?;
    assert!(store.get_settlement_item(&transfer_path)?.is_none());

    let revoke_path = right_path(71);
    let revoked = right_leaf(71);
    let _ = store.create_right_with_fee(
        revoke_path,
        revoked,
        right_ctx(&revoked, 15),
        fee_envelope(
            71,
            store.fee_support_ctx(&fee_put_ops(revoke_path, revoked)?)?,
        ),
        fee_actor(71, 15),
    )?;
    let _ = store.revoke_right_with_fee(
        revoke_path,
        right_ctx(&revoked, 15),
        fee_envelope(72, store.fee_support_ctx(&fee_del_ops(revoke_path))?),
        fee_actor(72, 15),
    )?;
    assert!(store.get_settlement_item(&revoke_path)?.is_none());

    let expire_path = right_path(81);
    let expired = right_leaf(81);
    let _ = store.create_right_with_fee(
        expire_path,
        expired,
        right_ctx(&expired, 15),
        fee_envelope(
            81,
            store.fee_support_ctx(&fee_put_ops(expire_path, expired)?)?,
        ),
        fee_actor(81, 15),
    )?;
    let _ = store.expire_right(
        expire_path,
        RightActionCtx {
            now: 25,
            ..RightActionCtx::default()
        },
    )?;
    assert!(store.get_settlement_item(&expire_path)?.is_none());

    Ok(())
}

#[test]
fn test_store_api_fee_support() -> Result<(), Box<dyn std::error::Error>> {
    let mut store = SettlementStore::new();
    let asset = asset_item(55);
    let seeded_root = store.put_settlement_item(asset)?;

    let create_path = right_path(56);
    let created = right_leaf(56);
    assert_fee_required(
        store
            .create_right(create_path, created, right_ctx(&created, 15))
            .expect_err("create without fee must reject"),
    );
    assert_eq!(store.settlement_root()?, seeded_root);

    let _ = store.create_right_with_fee(
        create_path,
        created,
        right_ctx(&created, 15),
        fee_envelope(
            56,
            store.fee_support_ctx(&fee_put_ops(create_path, created)?)?,
        ),
        fee_actor(56, 15),
    )?;
    let root_after_create = store.settlement_root()?;

    let transferred = transfer_leaf(created, 56);
    assert_fee_required(
        store
            .transfer_right(create_path, transferred, right_ctx(&transferred, 15))
            .expect_err("transfer without fee must reject"),
    );
    assert_fee_required(
        store
            .challenge_right(create_path, transferred, right_ctx(&transferred, 15))
            .expect_err("challenge without fee must reject"),
    );
    assert_fee_required(
        store
            .consume_right(create_path, right_ctx(&created, 15))
            .expect_err("consume without fee must reject"),
    );
    assert_eq!(store.settlement_root()?, root_after_create);

    let revoke_path = right_path(57);
    let revoked = right_leaf(57);
    let _ = store.create_right_with_fee(
        revoke_path,
        revoked,
        right_ctx(&revoked, 15),
        fee_envelope(
            57,
            store.fee_support_ctx(&fee_put_ops(revoke_path, revoked)?)?,
        ),
        fee_actor(57, 15),
    )?;
    let root_before_revoke = store.settlement_root()?;
    assert_fee_required(
        store
            .revoke_right(revoke_path, right_ctx(&revoked, 15))
            .expect_err("revoke without fee must reject"),
    );
    assert_eq!(store.settlement_root()?, root_before_revoke);

    Ok(())
}

#[test]
fn test_store_api_control_bind() -> Result<(), Box<dyn std::error::Error>> {
    let mut store = SettlementStore::new();

    let path = right_path(91);
    let created = right_leaf(91);
    let _ = store.create_right_with_fee(
        path,
        created,
        right_ctx(&created, 15),
        fee_envelope(91, store.fee_support_ctx(&fee_put_ops(path, created)?)?),
        fee_actor(91, 15),
    )?;
    let root_after_create = store.settlement_root()?;

    let transferred = transfer_leaf(created, 91);
    let mut ctx = right_ctx(&transferred, 15);
    ctx.expected_control = Some(bytes(0xEE));
    assert_holder_control_mix(
        store
            .transfer_right_with_fee(
                path,
                transferred,
                ctx,
                fee_envelope(92, store.fee_support_ctx(&fee_put_ops(path, transferred)?)?),
                fee_actor(92, 15),
            )
            .expect_err("wrong control binding must reject"),
    );
    assert_eq!(store.settlement_root()?, root_after_create);

    Ok(())
}

#[test]
fn test_voucher_issue_accept_paths() -> Result<(), Box<dyn std::error::Error>> {
    let mut store = SettlementStore::new();

    let source_path = SettlementPath::new(
        DefinitionId::new(bytes(150)),
        SerialId::new(1),
        TerminalId::new(bytes(151)),
    );
    let source_item = asset_item_at(source_path, 150);
    let _ = store.put_settlement_item(source_item)?;

    let voucher_path = voucher_path(101);
    let mut issued = voucher_leaf(101);
    issued.backing = VoucherBackingRef::ConsumedAsset {
        definition_id: source_path.definition_id.into_bytes(),
        serial_id: source_path.serial_id.get(),
    };
    let issue_ops = vec![
        StoreOp::Delete(source_path),
        StoreOp::Put(Box::new(StoreItem::new(
            voucher_path,
            SettlementLeaf::Voucher(issued.clone()),
        )?)),
    ];
    let issue_root = store.issue_voucher_with_fee(
        Some((source_path, 95)),
        voucher_path,
        issued.clone(),
        voucher_ctx(&issued, 15),
        fee_envelope(101, store.fee_support_ctx(&issue_ops)?),
        fee_actor(101, 15),
    )?;
    assert!(store.get_settlement_item(&source_path)?.is_none());
    assert_eq!(
        store
            .get_settlement_item(&voucher_path)?
            .expect("voucher")
            .voucher_leaf()?
            .lifecycle,
        VoucherLifecycleV1::PendingAcceptance
    );
    let issue_delta = store.latest_object_delta().expect("issue delta");
    assert_eq!(
        issue_delta.selected_action,
        SettlementActionV1::Voucher(VoucherAction::Issue)
    );
    assert_eq!(issue_delta.expected_new_root, issue_root);

    let mut accepted = issued.clone();
    accepted.lifecycle = VoucherLifecycleV1::Active;
    let accept_ops = voucher_put_ops(voucher_path, accepted.clone())?;
    let _ = store.accept_voucher_with_fee(
        voucher_path,
        accepted.clone(),
        VoucherActionCtx {
            acceptance_confirmed: true,
            ..voucher_ctx(&accepted, 16)
        },
        fee_envelope(102, store.fee_support_ctx(&accept_ops)?),
        fee_actor(102, 16),
    )?;

    let mut residual = accepted.clone();
    residual.remaining_value = 60;
    residual.lifecycle = VoucherLifecycleV1::PartiallyRedeemed;
    let partial_asset_path = SettlementPath::new(
        DefinitionId::new(bytes(152)),
        SerialId::new(1),
        TerminalId::new(bytes(153)),
    );
    let partial_asset = asset_item_at(partial_asset_path, 151);
    let partial_ops = vec![
        StoreOp::Put(Box::new(StoreItem::new(
            voucher_path,
            SettlementLeaf::Voucher(residual.clone()),
        )?)),
        StoreOp::Put(Box::new(partial_asset.clone())),
    ];
    let _ = store.redeem_voucher_partial_with_fee(
        voucher_path,
        residual.clone(),
        partial_asset.clone(),
        35,
        VoucherActionCtx {
            acceptance_confirmed: true,
            ..voucher_ctx(&accepted, 17)
        },
        fee_envelope(103, store.fee_support_ctx(&partial_ops)?),
        fee_actor(103, 17),
    )?;
    assert_eq!(
        store
            .get_settlement_item(&voucher_path)?
            .expect("residual")
            .voucher_leaf()?
            .remaining_value,
        60
    );

    let full_asset_path = SettlementPath::new(
        DefinitionId::new(bytes(154)),
        SerialId::new(1),
        TerminalId::new(bytes(155)),
    );
    let full_asset = asset_item_at(full_asset_path, 152);
    let full_ops = vec![
        StoreOp::Delete(voucher_path),
        StoreOp::Put(Box::new(full_asset.clone())),
    ];
    let _ = store.redeem_voucher_full_with_fee(
        voucher_path,
        full_asset,
        60,
        VoucherActionCtx {
            acceptance_confirmed: true,
            ..voucher_ctx(&residual, 18)
        },
        fee_envelope(104, store.fee_support_ctx(&full_ops)?),
        fee_actor(104, 18),
    )?;
    assert!(store.get_settlement_item(&voucher_path)?.is_none());
    assert_eq!(
        store
            .latest_object_delta()
            .expect("redeem delta")
            .selected_action,
        SettlementActionV1::Voucher(VoucherAction::RedeemFull)
    );

    Ok(())
}

#[test]
fn test_voucher_fail_closed_paths() -> Result<(), Box<dyn std::error::Error>> {
    let mut store = SettlementStore::new();

    let issued_path = voucher_path(111);
    let issued = voucher_leaf(111);
    let issue_ops = voucher_put_ops(issued_path, issued.clone())?;
    let _ = store.issue_voucher_with_fee(
        None,
        issued_path,
        issued.clone(),
        voucher_ctx(&issued, 15),
        fee_envelope(111, store.fee_support_ctx(&issue_ops)?),
        fee_actor(111, 15),
    )?;

    let forced_asset_path = SettlementPath::new(
        DefinitionId::new(bytes(160)),
        SerialId::new(1),
        TerminalId::new(bytes(161)),
    );
    let forced_asset = asset_item_at(forced_asset_path, 160);
    let forced_ops = vec![
        StoreOp::Delete(issued_path),
        StoreOp::Put(Box::new(forced_asset.clone())),
    ];
    let err = store
        .redeem_voucher_full_with_fee(
            issued_path,
            forced_asset,
            95,
            voucher_ctx(&issued, 16),
            fee_envelope(112, store.fee_support_ctx(&forced_ops)?),
            fee_actor(112, 16),
        )
        .expect_err("forced acceptance must reject");
    assert!(err.to_string().contains("forced acceptance"));

    let mut accepted = issued.clone();
    accepted.lifecycle = VoucherLifecycleV1::Active;
    let accept_ops = voucher_put_ops(issued_path, accepted.clone())?;
    let _ = store.accept_voucher_with_fee(
        issued_path,
        accepted.clone(),
        VoucherActionCtx {
            acceptance_confirmed: true,
            ..voucher_ctx(&accepted, 17)
        },
        fee_envelope(113, store.fee_support_ctx(&accept_ops)?),
        fee_actor(113, 17),
    )?;

    let mut transferred = accepted.clone();
    transferred.holder_commitment = bytes(200);
    let transfer_ops = voucher_put_ops(issued_path, transferred.clone())?;
    let err = store
        .transfer_voucher_with_fee(
            issued_path,
            transferred,
            VoucherActionCtx {
                acceptance_confirmed: true,
                ..voucher_ctx(&accepted, 18)
            },
            fee_envelope(114, store.fee_support_ctx(&transfer_ops)?),
            fee_actor(114, 18),
        )
        .expect_err("non-transferable voucher must reject");
    assert!(err.to_string().contains("not allowed by policy"));

    let mut unbacked = voucher_leaf(112);
    unbacked.backing = VoucherBackingRef::ReserveCommitment([0u8; 32]);
    let unbacked_path = voucher_path(112);
    let unbacked_ops = voucher_put_ops(unbacked_path, unbacked.clone())?;
    let err = store
        .issue_voucher_with_fee(
            None,
            unbacked_path,
            unbacked,
            voucher_ctx(&voucher_leaf(112), 19),
            fee_envelope(115, store.fee_support_ctx(&unbacked_ops)?),
            fee_actor(115, 19),
        )
        .expect_err("zero reserve backing must reject");
    assert!(err.to_string().contains("reserve backing must not be zero"));

    Ok(())
}

#[test]
fn test_voucher_reject_expire_refund() -> Result<(), Box<dyn std::error::Error>> {
    let mut store = SettlementStore::new();

    let pending_path = voucher_path(121);
    let pending = voucher_leaf(121);
    let pending_ops = voucher_put_ops(pending_path, pending.clone())?;
    let _ = store.issue_voucher_with_fee(
        None,
        pending_path,
        pending.clone(),
        voucher_ctx(&pending, 15),
        fee_envelope(121, store.fee_support_ctx(&pending_ops)?),
        fee_actor(121, 15),
    )?;

    let reject_asset_path = refund_asset_path(&pending, 1);
    let reject_asset = asset_item_at(reject_asset_path, 170);
    let reject_ops = vec![
        StoreOp::Delete(pending_path),
        StoreOp::Put(Box::new(reject_asset.clone())),
    ];
    let _ = store.reject_voucher_with_fee(
        pending_path,
        reject_asset,
        95,
        VoucherActionCtx {
            policy_allows_reject: true,
            ..voucher_ctx(&pending, 16)
        },
        fee_envelope(122, store.fee_support_ctx(&reject_ops)?),
        fee_actor(122, 16),
    )?;
    assert!(store.get_settlement_item(&pending_path)?.is_none());

    let active_path = voucher_path(122);
    let mut active = voucher_leaf(122);
    active.lifecycle = VoucherLifecycleV1::Active;
    active.validity.valid_until = 20;
    let active_ops = voucher_put_ops(active_path, active.clone())?;
    let _ = store.issue_voucher_with_fee(
        None,
        active_path,
        active.clone(),
        voucher_ctx(&active, 17),
        fee_envelope(123, store.fee_support_ctx(&active_ops)?),
        fee_actor(123, 17),
    )?;

    let mut expired = active.clone();
    expired.lifecycle = VoucherLifecycleV1::Expired;
    let _ = store.expire_voucher(
        active_path,
        expired.clone(),
        VoucherActionCtx {
            now: 25,
            acceptance_confirmed: true,
            ..voucher_ctx(&active, 25)
        },
    )?;

    let refund_asset_path = refund_asset_path(&expired, 1);
    let refund_asset = asset_item_at(refund_asset_path, 171);
    let refund_ops = vec![
        StoreOp::Delete(active_path),
        StoreOp::Put(Box::new(refund_asset.clone())),
    ];
    let _ = store.refund_voucher_with_fee(
        active_path,
        refund_asset,
        95,
        VoucherActionCtx {
            now: 30,
            acceptance_confirmed: true,
            policy_allows_refund: true,
            ..voucher_ctx(&expired, 30)
        },
        fee_envelope(124, store.fee_support_ctx(&refund_ops)?),
        fee_actor(124, 30),
    )?;
    assert!(store.get_settlement_item(&active_path)?.is_none());

    Ok(())
}

#[test]
fn test_refund_rejects_target() {
    let mut store = SettlementStore::new();

    let active_path = voucher_path(123);
    let mut active = voucher_leaf(123);
    active.lifecycle = VoucherLifecycleV1::Expired;
    let active_ops = voucher_put_ops(active_path, active.clone()).expect("voucher ops");
    let _ = store
        .issue_voucher_with_fee(
            None,
            active_path,
            active.clone(),
            voucher_ctx(&active, 20),
            fee_envelope(
                130,
                store.fee_support_ctx(&active_ops).expect("support ctx"),
            ),
            fee_actor(130, 20),
        )
        .expect("issue must succeed");

    let refund_path = SettlementPath::new(
        DefinitionId::new(match active.backing {
            VoucherBackingRef::ReserveCommitment(backing)
            | VoucherBackingRef::GenesisReserve(backing) => backing,
            VoucherBackingRef::ConsumedAsset { definition_id, .. } => definition_id,
        }),
        SerialId::new(1),
        TerminalId::new(bytes(201)),
    );
    let refund_asset = asset_item_at(refund_path, 201);
    let refund_ops = vec![
        StoreOp::Delete(active_path),
        StoreOp::Put(Box::new(refund_asset.clone())),
    ];
    let err = store
        .refund_voucher_with_fee(
            active_path,
            refund_asset,
            95,
            VoucherActionCtx {
                now: 30,
                acceptance_confirmed: true,
                policy_allows_refund: true,
                ..voucher_ctx(&active, 30)
            },
            fee_envelope(
                131,
                store.fee_support_ctx(&refund_ops).expect("support ctx"),
            ),
            fee_actor(131, 30),
        )
        .expect_err("wrong refund target must reject");

    assert!(err.to_string().contains("refund output target"));
}

#[test]
fn test_refund_rejects_source() {
    let mut store = SettlementStore::new();

    let active_path = voucher_path(124);
    let mut active = voucher_leaf(124);
    active.lifecycle = VoucherLifecycleV1::Expired;
    active.backing = VoucherBackingRef::ConsumedAsset {
        definition_id: bytes(210),
        serial_id: 7,
    };
    let _ = store
        .put_settlement_item(
            StoreItem::new(active_path, SettlementLeaf::Voucher(active.clone()))
                .expect("voucher item"),
        )
        .expect("voucher insert must succeed");

    let refund_path = SettlementPath::new(
        DefinitionId::new(bytes(211)),
        SerialId::new(7),
        TerminalId::new(active.refund_target_commitment),
    );
    let refund_asset = asset_item_at(refund_path, 202);
    let refund_ops = vec![
        StoreOp::Delete(active_path),
        StoreOp::Put(Box::new(refund_asset.clone())),
    ];
    let err = store
        .refund_voucher_with_fee(
            active_path,
            refund_asset,
            95,
            VoucherActionCtx {
                now: 30,
                acceptance_confirmed: true,
                policy_allows_refund: true,
                ..voucher_ctx(&active, 30)
            },
            fee_envelope(
                133,
                store.fee_support_ctx(&refund_ops).expect("support ctx"),
            ),
            fee_actor(133, 30),
        )
        .expect_err("wrong source context must reject");

    assert!(err.to_string().contains("source context"));
}

#[test]
fn test_store_refund_reserve_bad() {
    let mut store = SettlementStore::new();

    let active_path = voucher_path(125);
    let mut active = voucher_leaf(125);
    active.lifecycle = VoucherLifecycleV1::Expired;
    active.backing = VoucherBackingRef::ReserveCommitment(bytes(212));
    let _ = store
        .put_settlement_item(
            StoreItem::new(active_path, SettlementLeaf::Voucher(active.clone()))
                .expect("voucher item"),
        )
        .expect("voucher insert must succeed");

    let refund_path = SettlementPath::new(
        DefinitionId::new(bytes(213)),
        SerialId::new(9),
        TerminalId::new(active.refund_target_commitment),
    );
    let refund_asset = asset_item_at(refund_path, 203);
    let refund_ops = vec![
        StoreOp::Delete(active_path),
        StoreOp::Put(Box::new(refund_asset.clone())),
    ];
    let err = store
        .refund_voucher_with_fee(
            active_path,
            refund_asset,
            95,
            VoucherActionCtx {
                now: 30,
                acceptance_confirmed: true,
                policy_allows_refund: true,
                ..voucher_ctx(&active, 30)
            },
            fee_envelope(
                134,
                store.fee_support_ctx(&refund_ops).expect("support ctx"),
            ),
            fee_actor(134, 30),
        )
        .expect_err("wrong reserve context must reject");

    assert!(err.to_string().contains("reserve backing"));
}
