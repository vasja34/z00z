use z00z_storage::settlement::{
    DefinitionId, HjmtProofFamily, ProofBlob, ProofChkErr, SerialId, SettlementLeaf,
    SettlementLeafFamily, SettlementPath, SettlementStore, SettlementStoreError, TerminalId,
};

use z00z_storage::fixture_support::settlement_corpus;

use z00z_storage::fixture_support::settlement_corpus::{
    assert_store_matches_oracle, asset_item, asset_path, bytes, fee_actor, fee_del_ops,
    fee_envelope, fee_put_ops, list_items, load_fixture, next_policy, nonexistence_marker,
    proof_family, redb_store_with_bits, right_ctx, right_leaf, right_path, sibling_bucket_pair,
    split_ready_paths, terminal_set, transferred_right_leaf, AssetSeed, FixtureRightClass,
    OracleState, RightSeed,
};

fn extra_asset_seed() -> AssetSeed {
    AssetSeed {
        label: "asset_extra".to_string(),
        definition_mark: 14,
        serial_id: 1,
        terminal_mark: 41,
        value: 1_404,
    }
}

fn create_right_seed() -> RightSeed {
    RightSeed {
        label: "right_created".to_string(),
        definition_mark: 41,
        serial_id: 6,
        terminal_mark: 81,
        right_class: FixtureRightClass::ServiceEntitlement,
    }
}

fn seed_store(
    store: &mut SettlementStore,
    oracle: &mut OracleState,
) -> Result<
    (
        settlement_corpus::Fixture,
        Vec<z00z_storage::settlement::StoreItem>,
    ),
    Box<dyn std::error::Error>,
> {
    let fixture = load_fixture();
    let items = settlement_corpus::load_fixture_items(&fixture);
    for item in &items {
        let _ = store.put_settlement_item(item.clone())?;
        let _ = oracle.put(item.clone())?;
    }
    Ok((fixture, items))
}

#[test]
fn test_replay_live_mixed_operations() -> Result<(), Box<dyn std::error::Error>> {
    let mut store = SettlementStore::new();
    let mut oracle = OracleState::default();
    let (fixture, _items) = seed_store(&mut store, &mut oracle)?;

    assert_store_matches_oracle(&store, &oracle);

    let extra_asset = extra_asset_seed();
    let extra_item = asset_item(&extra_asset);
    let _ = store.put_settlement_item(extra_item.clone())?;
    let _ = oracle.put(extra_item)?;
    assert_store_matches_oracle(&store, &oracle);

    let created_seed = create_right_seed();
    let created_path = right_path(&created_seed);
    let created_leaf = right_leaf(&created_seed);
    let _ = store.create_right_with_fee(
        created_path,
        created_leaf,
        right_ctx(&created_leaf, 15),
        fee_envelope(
            created_seed.terminal_mark,
            store.fee_support_ctx(&fee_put_ops(
                created_path,
                SettlementLeaf::Right(created_leaf),
            )?)?,
        ),
        fee_actor(created_seed.terminal_mark, 15),
    )?;
    let _ = oracle.create_right(created_path, created_leaf)?;
    assert_store_matches_oracle(&store, &oracle);

    let transferred = transferred_right_leaf(created_leaf, created_seed.terminal_mark);
    let _ = store.transfer_right_with_fee(
        created_path,
        transferred,
        right_ctx(&transferred, 15),
        fee_envelope(
            created_seed.terminal_mark.wrapping_add(1),
            store.fee_support_ctx(&fee_put_ops(
                created_path,
                SettlementLeaf::Right(transferred),
            )?)?,
        ),
        fee_actor(created_seed.terminal_mark.wrapping_add(1), 15),
    )?;
    let _ = oracle.transfer_right(created_path, transferred, 15)?;
    assert_store_matches_oracle(&store, &oracle);

    let machine_path = right_path(&fixture.rights[0]);
    let machine_leaf = right_leaf(&fixture.rights[0]);
    let _ = store.consume_right_with_fee(
        machine_path,
        right_ctx(&machine_leaf, 15),
        fee_envelope(90, store.fee_support_ctx(&fee_del_ops(machine_path))?),
        fee_actor(90, 15),
    )?;
    let _ = oracle.consume_right(machine_path, 15)?;
    assert_store_matches_oracle(&store, &oracle);

    let validator_path = right_path(&fixture.rights[3]);
    let validator_leaf = right_leaf(&fixture.rights[3]);
    let _ = store.revoke_right_with_fee(
        validator_path,
        right_ctx(&validator_leaf, 15),
        fee_envelope(91, store.fee_support_ctx(&fee_del_ops(validator_path))?),
        fee_actor(91, 15),
    )?;
    let _ = oracle.revoke_right(validator_path, 15)?;
    assert_store_matches_oracle(&store, &oracle);

    let data_path = right_path(&fixture.rights[1]);
    let _ = store.expire_right(
        data_path,
        z00z_storage::settlement::RightActionCtx {
            now: 25,
            ..z00z_storage::settlement::RightActionCtx::default()
        },
    )?;
    let _ = oracle.expire_right(data_path, 25)?;
    assert_store_matches_oracle(&store, &oracle);

    let delete_path = asset_path(&fixture.assets[3]);
    let _ = store.del_settlement_item(&delete_path)?;
    let _ = oracle.delete(delete_path)?;
    assert_store_matches_oracle(&store, &oracle);

    let surviving_asset_path = asset_path(&fixture.assets[0]);
    let surviving_asset_blob = store.settlement_proof_blob(&surviving_asset_path)?;
    assert_eq!(
        proof_family(&surviving_asset_blob),
        HjmtProofFamily::Inclusion
    );
    assert_eq!(
        surviving_asset_blob.hjmt_leaf_family(),
        Some(SettlementLeafFamily::Terminal)
    );
    store.validate_settlement_proof_blob(&surviving_asset_blob)?;

    let surviving_right_path = right_path(&fixture.rights[2]);
    let surviving_right_blob = store.settlement_proof_blob(&surviving_right_path)?;
    assert_eq!(
        proof_family(&surviving_right_blob),
        HjmtProofFamily::Inclusion
    );
    assert_eq!(
        surviving_right_blob.hjmt_leaf_family(),
        Some(SettlementLeafFamily::Right)
    );
    store.validate_settlement_proof_blob(&surviving_right_blob)?;

    let deleted_asset_blob = store.settlement_proof_blob(&delete_path)?;
    assert_eq!(proof_family(&deleted_asset_blob), HjmtProofFamily::Deletion);
    store.validate_settlement_proof_blob(&deleted_asset_blob)?;

    let missing_asset_path = SettlementPath::new(
        DefinitionId::new(bytes(51)),
        SerialId::new(1),
        TerminalId::new(bytes(52)),
    );
    let missing_asset = store
        .settlement_nonexistence_proof_blob(&missing_asset_path, SettlementLeafFamily::Terminal)?;
    assert_eq!(proof_family(&missing_asset), HjmtProofFamily::NonExistence);
    store.validate_settlement_nonexistence_proof_blob(
        &missing_asset,
        SettlementLeafFamily::Terminal,
    )?;

    let missing_right_path = SettlementPath::new(
        DefinitionId::new(bytes(52)),
        SerialId::new(1),
        z00z_storage::settlement::TerminalId::new(bytes(82)),
    );
    let missing_right = store
        .settlement_nonexistence_proof_blob(&missing_right_path, SettlementLeafFamily::Right)?;
    assert_eq!(proof_family(&missing_right), HjmtProofFamily::NonExistence);
    store
        .validate_settlement_nonexistence_proof_blob(&missing_right, SettlementLeafFamily::Right)?;

    let err = store
        .settlement_nonexistence_proof_blob(&surviving_asset_path, SettlementLeafFamily::Terminal)
        .expect_err("present asset must reject absence proof");
    assert!(matches!(err, SettlementStoreError::PathMiss));

    let legacy_like = ProofBlob::new(
        surviving_right_blob.item().clone(),
        surviving_right_blob.terminal_leaf_hash(),
        surviving_right_blob.backend_root(),
        surviving_right_blob.definition_proof().to_vec(),
        surviving_right_blob.serial_proof().to_vec(),
        surviving_right_blob.terminal_proof().to_vec(),
    );
    let root_before_legacy = store.settlement_root()?;
    let rows_before_legacy = list_items(&store)?;
    let err = store
        .validate_settlement_proof_blob(&legacy_like)
        .expect_err("legacy envelope-less proof must reject");
    assert!(matches!(
        err,
        SettlementStoreError::Proof(ProofChkErr::ProofFamilyMix)
    ));
    assert_eq!(store.settlement_root()?, root_before_legacy);
    assert_eq!(list_items(&store)?, rows_before_legacy);

    let err = store
        .validate_settlement_nonexistence_proof_blob(&missing_asset, SettlementLeafFamily::Right)
        .expect_err("leaf-family drift must reject");
    assert!(matches!(
        err,
        SettlementStoreError::Proof(ProofChkErr::LeafMix)
    ));

    let marker = nonexistence_marker(missing_asset_path, SettlementLeafFamily::Terminal);
    assert!(matches!(marker, SettlementLeaf::Terminal(_)));

    Ok(())
}

#[test]
fn test_adaptive_proof_reload_stable() -> Result<(), Box<dyn std::error::Error>> {
    let (split_env, split_temp, mut split_store) = redb_store_with_bits(Some("1"))?;
    let split_paths = split_ready_paths(&mut split_store, 41, 9);
    let split_left = split_paths[0];
    let split_right = split_paths[1];
    let split_root_before = split_store.settlement_root()?;
    let split_terms_before = terminal_set(&list_items(&split_store)?);
    let split = split_store.split_proof(&split_left)?;
    split_store.validate_split_proof(&split)?;
    assert_eq!(split_left.definition_id, split_right.definition_id);
    assert_eq!(split_store.settlement_root()?, split_root_before);
    assert_eq!(terminal_set(&list_items(&split_store)?), split_terms_before);
    drop(split_store);

    let reloaded_split = SettlementStore::load(split_temp.path())?;
    reloaded_split.validate_split_proof(&split)?;
    assert_eq!(
        terminal_set(&list_items(&reloaded_split)?),
        split_terms_before
    );
    drop(reloaded_split);
    drop(split_env);

    let (_merge_env, merge_temp, mut merge_store) = redb_store_with_bits(Some("2"))?;
    let (merge_left, merge_right) = sibling_bucket_pair(&mut merge_store, 33, 11);
    let merge_root_before = merge_store.settlement_root()?;
    let merge_terms_before = terminal_set(&list_items(&merge_store)?);
    let merge = merge_store.merge_proof(&merge_left, &merge_right)?;
    merge_store.validate_merge_proof(&merge)?;
    assert_eq!(merge_store.settlement_root()?, merge_root_before);
    assert_eq!(terminal_set(&list_items(&merge_store)?), merge_terms_before);

    let policy = next_policy(&merge_store);
    let trans_root_before = merge_store.settlement_root()?;
    let trans_terms_before = terminal_set(&list_items(&merge_store)?);
    let transition = merge_store.policy_transition_proof(policy)?;
    merge_store.validate_policy_transition_proof(&transition, policy)?;
    assert_eq!(merge_store.settlement_root()?, trans_root_before);
    assert_eq!(terminal_set(&list_items(&merge_store)?), trans_terms_before);

    let batch_paths = vec![merge_left, merge_right];
    let batch = merge_store.settlement_proof_blobs(&batch_paths)?;
    let batch_paths_out = batch
        .iter()
        .map(|blob| blob.item().path())
        .collect::<Vec<_>>();
    assert_eq!(batch_paths_out, batch_paths);
    drop(merge_store);

    let reloaded_merge = SettlementStore::load(merge_temp.path())?;
    reloaded_merge.validate_merge_proof(&merge)?;
    reloaded_merge.validate_policy_transition_proof(&transition, policy)?;
    assert_eq!(
        terminal_set(&list_items(&reloaded_merge)?),
        trans_terms_before
    );

    Ok(())
}
