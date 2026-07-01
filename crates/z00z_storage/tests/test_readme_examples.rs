use z00z_storage::settlement::{
    DefinitionId, FeeActorCtx, FeeEnvelope, HjmtProofFamily, SettlementLeafFamily,
    SettlementLookup, SettlementPath, SettlementStore, TerminalId,
};

use z00z_storage::fixture_support::settlement_corpus::{
    asset_item, asset_path, bytes, fee_actor, fee_del_ops, fee_envelope, fee_put_ops, load_fixture,
    load_fixture_items, next_policy, right_ctx, right_leaf, right_path, sibling_bucket_pair,
    split_ready_paths, transferred_right_leaf, AssetSeed, FixtureRightClass, HjmtEnvGuard,
    RightSeed,
};

fn doc_asset_seed(definition: u8, serial: u32, terminal: u8, value: u64) -> AssetSeed {
    AssetSeed {
        label: format!("doc_asset_{definition}_{serial}_{terminal}"),
        definition_mark: definition,
        serial_id: serial,
        terminal_mark: terminal,
        value,
    }
}

fn doc_right_seed(definition: u8, serial: u32, terminal: u8) -> RightSeed {
    RightSeed {
        label: format!("doc_right_{definition}_{serial}_{terminal}"),
        definition_mark: definition,
        serial_id: serial,
        terminal_mark: terminal,
        right_class: FixtureRightClass::MachineCapability,
    }
}

#[test]
fn test_readme_examples_cover_surface() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = load_fixture();
    let _guard = HjmtEnvGuard::with_bits("2");
    let mut store = SettlementStore::new();
    store.apply_settlement_ops(
        load_fixture_items(&fixture)
            .into_iter()
            .map(|item| z00z_storage::settlement::StoreOp::Put(Box::new(item)))
            .collect(),
    )?;

    let asset = doc_asset_seed(0x91, 3, 8, 51_000);
    let asset_item = asset_item(&asset);
    let asset_path = asset_path(&asset);
    let _ = store.put_settlement_item(asset_item)?;
    assert!(store
        .lookup_settlement(SettlementLookup::Path(asset_path))?
        .is_some());

    let right = doc_right_seed(0x92, 4, 9);
    let right_path = right_path(&right);
    let created = right_leaf(&right);
    let create_support = store.fee_support_ctx(&fee_put_ops(right_path, created)?)?;
    let create_env = fee_envelope(41, create_support);
    let create_root = store.create_right_with_fee(
        right_path,
        created,
        right_ctx(&created, 15),
        create_env,
        fee_actor(41, 15),
    )?;
    assert_eq!(store.settlement_root()?, create_root);

    let transferred = transferred_right_leaf(created, right.terminal_mark);
    let transfer_support = store.fee_support_ctx(&fee_put_ops(right_path, transferred)?)?;
    let transfer_env = fee_envelope(42, transfer_support);
    let _ = store.transfer_right_with_fee(
        right_path,
        transferred,
        right_ctx(&transferred, 15),
        transfer_env,
        fee_actor(42, 15),
    )?;

    let consume_support = store.fee_support_ctx(&fee_del_ops(right_path))?;
    let consume_env = fee_envelope(43, consume_support);
    let _ = store.consume_right_with_fee(
        right_path,
        right_ctx(&transferred, 15),
        consume_env,
        fee_actor(43, 15),
    )?;

    let deletion = store.settlement_proof_blob(&right_path)?;
    assert_eq!(
        deletion.hjmt_proof_family(),
        Some(HjmtProofFamily::Deletion)
    );
    store.validate_settlement_proof_blob(&deletion)?;

    let missing_path = SettlementPath::new(
        DefinitionId::new(bytes(0xAA)),
        z00z_storage::settlement::SerialId::new(1),
        TerminalId::new(bytes(0xBB)),
    );
    let absence =
        store.settlement_nonexistence_proof_blob(&missing_path, SettlementLeafFamily::Right)?;
    store.validate_settlement_nonexistence_proof_blob(&absence, SettlementLeafFamily::Right)?;
    assert!(store
        .lookup_settlement(SettlementLookup::Path(missing_path))?
        .is_none());

    Ok(())
}

#[test]
fn test_readme_examples_cover_metrics() -> Result<(), Box<dyn std::error::Error>> {
    {
        let _guard = HjmtEnvGuard::with_bits("1");
        let mut split_store = SettlementStore::new();
        let split_path = split_ready_paths(&mut split_store, 41, 9)[0];
        let split = split_store.split_proof(&split_path)?;
        split_store.validate_split_proof(&split)?;
    }

    {
        let _guard = HjmtEnvGuard::with_bits("2");
        let mut merge_store = SettlementStore::new();
        let (left_path, right_path) = sibling_bucket_pair(&mut merge_store, 33, 11);
        let merge = merge_store.merge_proof(&left_path, &right_path)?;
        merge_store.validate_merge_proof(&merge)?;
        let next = next_policy(&merge_store);
        let transition = merge_store.policy_transition_proof(next)?;
        merge_store.validate_policy_transition_proof(&transition, next)?;
    }

    {
        let fixture = load_fixture();
        let _guard = HjmtEnvGuard::with_bits("2");
        let mut store = SettlementStore::new();
        let items = load_fixture_items(&fixture);
        let paths = items
            .iter()
            .take(6)
            .map(|item| item.path())
            .collect::<Vec<_>>();
        let proof_path = paths[0];
        store.apply_settlement_ops(
            items
                .into_iter()
                .map(|item| z00z_storage::settlement::StoreOp::Put(Box::new(item)))
                .collect(),
        )?;
        let _ = store.settlement_proof_blob(&proof_path)?;
        let _ = store.settlement_proof_blobs(&paths)?;
        let _ = store.lookup_settlement(SettlementLookup::Terminal(paths[0].terminal_id))?;
        let cache = store.forest_cache_metrics();
        let sched = store.forest_scheduler_metrics();
        assert!(cache.proof_segment.hits + cache.proof_segment.misses > 0);
        assert!(sched.last_batch >= paths.len());
    }

    let _ = FeeActorCtx::default();
    let _ = core::mem::size_of::<FeeEnvelope>();

    Ok(())
}
