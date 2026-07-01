use proptest::prelude::*;
use proptest::test_runner::Config;
use z00z_storage::settlement::{
    ProofChkErr, RightActionCtx, SettlementLeaf, SettlementLeafFamily, SettlementStore,
    SettlementStoreError,
};

use z00z_storage::fixture_support::settlement_corpus;

use z00z_storage::fixture_support::settlement_corpus::{
    assert_store_matches_oracle, asset_item, asset_path, fee_actor, fee_del_ops, fee_envelope,
    fee_put_ops, list_items, load_fixture, next_policy, redb_store, right_ctx, right_leaf,
    right_path, sibling_bucket_pair, split_ready_paths, transferred_right_leaf, AssetSeed,
    FixtureRightClass, HjmtEnvGuard, OracleState, RightSeed,
};

fn prop_cfg() -> Config {
    Config {
        cases: 8,
        ..Config::default()
    }
}

fn seeded_asset(seed: u64) -> AssetSeed {
    let mark = 101u8.wrapping_add(seed as u8);
    AssetSeed {
        label: format!("prop_asset_{mark}"),
        definition_mark: 70u8.wrapping_add((seed as u8) % 11),
        serial_id: 10 + ((seed % 5) as u32),
        terminal_mark: mark,
        value: 5_000 + (seed % 1_000),
    }
}

fn seeded_right(seed: u64) -> RightSeed {
    let mark = 151u8.wrapping_add(seed as u8);
    let class = match seed % 5 {
        0 => FixtureRightClass::MachineCapability,
        1 => FixtureRightClass::DataAccess,
        2 => FixtureRightClass::ServiceEntitlement,
        3 => FixtureRightClass::ValidatorMandate,
        _ => FixtureRightClass::OneTimeUse,
    };
    RightSeed {
        label: format!("prop_right_{mark}"),
        definition_mark: 90u8.wrapping_add((seed as u8) % 11),
        serial_id: 20 + ((seed % 5) as u32),
        terminal_mark: mark,
        right_class: class,
    }
}

fn isolated_store(
) -> Result<(HjmtEnvGuard, tempfile::TempDir, SettlementStore), Box<dyn std::error::Error>> {
    redb_store()
}

fn isolated_store_with_bits(
    bits: &str,
) -> Result<(HjmtEnvGuard, tempfile::TempDir, SettlementStore), Box<dyn std::error::Error>> {
    let env = HjmtEnvGuard::with_bits(bits);
    let temp = tempfile::tempdir()?;
    let store = SettlementStore::load(temp.path())?;
    Ok((env, temp, store))
}

proptest! {
    #![proptest_config(prop_cfg())]

    #[test]
    fn test_property_sequences_match_oracle(seed in 0u64..64) {
        let (_env, _temp, mut store) = isolated_store().expect("isolated store");
        let mut oracle = OracleState::default();
        let fixture = load_fixture();
        for item in settlement_corpus::load_fixture_items(&fixture) {
            store.put_settlement_item(item.clone()).expect("seed store");
            oracle.put(item).expect("seed oracle");
        }

        let asset = seeded_asset(seed);
        let right = seeded_right(seed);
        let asset_item = asset_item(&asset);
        let right_path = right_path(&right);
        let created = right_leaf(&right);
        let transferred = transferred_right_leaf(created, right.terminal_mark);

        store.put_settlement_item(asset_item.clone()).expect("asset put");
        oracle.put(asset_item).expect("oracle asset put");
        assert_store_matches_oracle(&store, &oracle);

        store
            .create_right_with_fee(
                right_path,
                created,
                right_ctx(&created, 15),
                fee_envelope(
                    right.terminal_mark,
                    store
                        .fee_support_ctx(
                            &fee_put_ops(right_path, SettlementLeaf::Right(created))
                                .expect("fee put ops"),
                        )
                        .expect("fee support"),
                ),
                fee_actor(right.terminal_mark, 15),
            )
            .expect("create right");
        oracle.create_right(right_path, created).expect("oracle create");
        assert_store_matches_oracle(&store, &oracle);

        store
            .transfer_right_with_fee(
                right_path,
                transferred,
                right_ctx(&transferred, 15),
                fee_envelope(
                    right.terminal_mark.wrapping_add(1),
                    store
                        .fee_support_ctx(
                            &fee_put_ops(right_path, SettlementLeaf::Right(transferred))
                                .expect("fee put ops"),
                        )
                        .expect("fee support"),
                ),
                fee_actor(right.terminal_mark.wrapping_add(1), 15),
            )
            .expect("transfer right");
        oracle
            .transfer_right(right_path, transferred, 15)
            .expect("oracle transfer");
        assert_store_matches_oracle(&store, &oracle);

        match seed % 3 {
            0 => {
                store
                    .consume_right_with_fee(
                        right_path,
                        right_ctx(&transferred, 15),
                        fee_envelope(
                            right.terminal_mark.wrapping_add(2),
                            store.fee_support_ctx(&fee_del_ops(right_path)).expect("fee del support"),
                        ),
                        fee_actor(right.terminal_mark.wrapping_add(2), 15),
                    )
                    .expect("consume right");
                oracle.consume_right(right_path, 15).expect("oracle consume");
            }
            1 => {
                store
                    .revoke_right_with_fee(
                        right_path,
                        right_ctx(&transferred, 15),
                        fee_envelope(
                            right.terminal_mark.wrapping_add(2),
                            store.fee_support_ctx(&fee_del_ops(right_path)).expect("fee del support"),
                        ),
                        fee_actor(right.terminal_mark.wrapping_add(2), 15),
                    )
                    .expect("revoke right");
                oracle.revoke_right(right_path, 15).expect("oracle revoke");
            }
            _ => {
                store
                    .expire_right(
                        right_path,
                        RightActionCtx {
                            now: 25,
                            ..RightActionCtx::default()
                        },
                    )
                    .expect("expire right");
                oracle.expire_right(right_path, 25).expect("oracle expire");
            }
        }

        assert_store_matches_oracle(&store, &oracle);
    }
}

#[test]
fn test_property_reordering_keeps_root() -> Result<(), Box<dyn std::error::Error>> {
    let asset_a = seeded_asset(1);
    let asset_b = seeded_asset(2);
    let right_a = seeded_right(3);
    let right_b = seeded_right(4);

    let _env = HjmtEnvGuard::new();
    let left_temp = tempfile::tempdir()?;
    let right_temp = tempfile::tempdir()?;
    let mut left = SettlementStore::load(left_temp.path())?;
    let mut right = SettlementStore::load(right_temp.path())?;

    let left_ops = vec![
        asset_item(&asset_a),
        asset_item(&asset_b),
        settlement_corpus::right_item(&right_a),
        settlement_corpus::right_item(&right_b),
    ];
    let right_ops = vec![
        settlement_corpus::right_item(&right_b),
        asset_item(&asset_b),
        settlement_corpus::right_item(&right_a),
        asset_item(&asset_a),
    ];

    for item in left_ops {
        let _ = left.put_settlement_item(item)?;
    }
    for item in right_ops {
        let _ = right.put_settlement_item(item)?;
    }

    assert_eq!(left.settlement_root()?, right.settlement_root()?);
    assert_eq!(list_items(&left)?, list_items(&right)?);
    Ok(())
}

#[test]
fn test_reject_paths_preserve_state() -> Result<(), Box<dyn std::error::Error>> {
    let (_env, _temp, mut store) = isolated_store()?;
    let mut oracle = OracleState::default();
    let fixture = load_fixture();
    for item in settlement_corpus::load_fixture_items(&fixture) {
        let _ = store.put_settlement_item(item.clone())?;
        let _ = oracle.put(item)?;
    }
    let stable = settlement_corpus::right_item(&fixture.rights[2]);
    let stable_path = stable.path();
    let stable_leaf = *stable.right_leaf()?;
    assert_store_matches_oracle(&store, &oracle);

    let replay_support = store.fee_support_ctx(&fee_del_ops(stable_path))?;
    let replay_env = fee_envelope(201, replay_support);
    let _ = store.consume_right_with_fee(
        stable_path,
        right_ctx(&stable_leaf, 15),
        replay_env,
        fee_actor(201, 15),
    )?;
    let _ = oracle.consume_right(stable_path, 15)?;
    assert_store_matches_oracle(&store, &oracle);

    let create = seeded_right(7);
    let create_path = right_path(&create);
    let create_leaf = right_leaf(&create);
    let _ = store.create_right_with_fee(
        create_path,
        create_leaf,
        right_ctx(&create_leaf, 15),
        fee_envelope(
            create.terminal_mark,
            store.fee_support_ctx(&fee_put_ops(
                create_path,
                SettlementLeaf::Right(create_leaf),
            )?)?,
        ),
        fee_actor(create.terminal_mark, 15),
    )?;
    let _ = oracle.create_right(create_path, create_leaf)?;
    assert_store_matches_oracle(&store, &oracle);
    let root_before = store.settlement_root()?;
    let rows_before = list_items(&store)?;

    let transferred = transferred_right_leaf(create_leaf, create.terminal_mark);
    let err = store
        .transfer_right(create_path, transferred, right_ctx(&transferred, 15))
        .expect_err("missing fee must reject");
    assert!(matches!(err, SettlementStoreError::Fee(_)));
    assert_eq!(store.settlement_root()?, root_before);
    assert_eq!(list_items(&store)?, rows_before);

    let err = store
        .consume_right_with_fee(
            create_path,
            right_ctx(&create_leaf, 15),
            replay_env,
            fee_actor(201, 15),
        )
        .expect_err("replay envelope must reject");
    assert!(matches!(err, SettlementStoreError::Fee(_)));
    assert_eq!(store.settlement_root()?, root_before);
    assert_eq!(list_items(&store)?, rows_before);

    let present_asset = asset_path(&fixture.assets[0]);
    let present_asset_blob = store.settlement_proof_blob(&present_asset)?;
    store.validate_settlement_proof_blob(&present_asset_blob)?;
    let err = store
        .settlement_nonexistence_proof_blob(&present_asset, SettlementLeafFamily::Terminal)
        .expect_err("present asset must reject nonexistence proof");
    assert!(matches!(err, SettlementStoreError::PathMiss));
    assert_eq!(store.settlement_root()?, root_before);
    assert_eq!(list_items(&store)?, rows_before);

    Ok(())
}

#[test]
fn test_property_split_merge_terminals() -> Result<(), Box<dyn std::error::Error>> {
    let (_split_env, _split_temp, mut split_store) = isolated_store_with_bits("1")?;
    let before_terms = settlement_corpus::terminal_set(&list_items(&split_store)?);

    let split_left = split_ready_paths(&mut split_store, 41, 9)[0];
    let split_before = settlement_corpus::terminal_set(&list_items(&split_store)?);
    let split = split_store.split_proof(&split_left)?;
    split_store.validate_split_proof(&split)?;
    assert_eq!(
        settlement_corpus::terminal_set(&list_items(&split_store)?),
        split_before
    );
    drop(split_store);
    drop(_split_env);

    let (_merge_env, _merge_temp, mut merge_store) = isolated_store_with_bits("2")?;
    let (merge_left, merge_right) = sibling_bucket_pair(&mut merge_store, 33, 11);
    let merge_before = settlement_corpus::terminal_set(&list_items(&merge_store)?);
    let merge = merge_store.merge_proof(&merge_left, &merge_right)?;
    merge_store.validate_merge_proof(&merge)?;
    assert_eq!(
        settlement_corpus::terminal_set(&list_items(&merge_store)?),
        merge_before
    );

    let next = next_policy(&merge_store);
    let transition = merge_store.policy_transition_proof(next)?;
    merge_store.validate_policy_transition_proof(&transition, next)?;
    assert!(transition.next_epoch > transition.prior_epoch);
    assert_eq!(transition.prior_root, merge_store.settlement_root()?);
    assert_eq!(transition.next_policy_id, next.bucket_policy_id());
    assert!(
        settlement_corpus::terminal_set(&list_items(&merge_store)?).len() >= before_terms.len()
    );

    Ok(())
}

#[test]
fn test_property_reload_idempotent() -> Result<(), Box<dyn std::error::Error>> {
    let (_env, temp, mut store) = redb_store()?;
    let fixture = load_fixture();
    for item in settlement_corpus::load_fixture_items(&fixture) {
        let _ = store.put_settlement_item(item)?;
    }
    let root_before = store.settlement_root()?;
    let rows_before = list_items(&store)?;
    drop(store);

    let first = SettlementStore::load(temp.path())?;
    assert_eq!(first.settlement_root()?, root_before);
    assert_eq!(list_items(&first)?, rows_before);
    drop(first);

    let second = SettlementStore::load(temp.path())?;
    assert_eq!(second.settlement_root()?, root_before);
    assert_eq!(list_items(&second)?, rows_before);
    Ok(())
}

#[test]
fn test_property_malformed_proofs_reject() -> Result<(), Box<dyn std::error::Error>> {
    let (_env, _temp, mut store) = isolated_store()?;
    let target = asset_item(&load_fixture().assets[0]);
    let path = target.path();
    let leaf = target.terminal_leaf()?.clone();
    let root = store.put_settlement_item(target)?;
    let blob = store.settlement_proof_blob(&path)?;

    let mut truncated = blob.encode()?;
    truncated.truncate(truncated.len().saturating_sub(7));
    let err = z00z_storage::settlement::chk_blob_settlement_inclusion(
        &truncated,
        root,
        &path,
        blob.item().def_leaf(),
        blob.item().ser_leaf(),
        leaf.clone(),
    )
    .expect_err("truncated proof must reject");
    assert!(matches!(err, ProofChkErr::Codec(_)));

    let mut detached = blob.encode()?;
    detached.extend_from_slice(b"settlement-detached");
    let err = z00z_storage::settlement::chk_blob_settlement_inclusion(
        &detached,
        root,
        &path,
        blob.item().def_leaf(),
        blob.item().ser_leaf(),
        leaf,
    )
    .expect_err("detached payload must reject");
    assert!(matches!(err, ProofChkErr::Codec(_)));

    let wrong_family = blob
        .clone()
        .with_hjmt_leaf_family(SettlementLeafFamily::Right);
    let err = store
        .validate_settlement_proof_blob(&wrong_family)
        .expect_err("leaf-family drift must reject");
    assert!(matches!(
        err,
        SettlementStoreError::Proof(ProofChkErr::LeafMix)
    ));

    Ok(())
}
