use z00z_core::assets::AssetLeaf;
use z00z_storage::settlement::{
    check_public_checkpoint_route_v1, check_public_checkpoint_v1, CheckpointPublicationProofV1,
    CheckpointPublicationV1, HjmtProofFamily, PolicySetCommitmentV1, ProofChkErr,
    PublicationModeTagV1, PublicationRouteSnapshotV1, RootGenerationTagV1, SettlementLeafFamily,
    SettlementPath, SettlementRecoveryState, SettlementStateRoot, SettlementStore,
    ShardProofContextV1, ShardRootLeafV1, StoreItem, StoreOp, TerminalId,
};

fn bytes(value: u8) -> [u8; 32] {
    [value; 32]
}

fn path(mark: u8, serial: u32) -> SettlementPath {
    SettlementPath::new(
        bytes(mark.wrapping_add(1)).into(),
        serial.into(),
        TerminalId::new(bytes(mark)),
    )
}

fn item(path: SettlementPath) -> StoreItem {
    let mut asset = AssetLeaf::dummy_for_scan(path.serial_id.get());
    asset.asset_id = path.terminal_id().into_bytes();
    StoreItem::new(path, asset).expect("store item")
}

fn leaf(
    shard_id: u32,
    shard_root: SettlementStateRoot,
    shard_epoch: u64,
    routing_generation: u64,
    route_table_digest: [u8; 32],
    policy_set: &PolicySetCommitmentV1,
    journal_checkpoint: u64,
    local_sequence: u64,
) -> ShardRootLeafV1 {
    ShardRootLeafV1::new(
        shard_id,
        shard_root.into_bytes(),
        shard_epoch,
        routing_generation,
        route_table_digest,
        policy_set.digest().expect("policy-set digest"),
        journal_checkpoint,
        local_sequence,
        0,
    )
}

fn recovery(
    version: u64,
    state_root: SettlementStateRoot,
    policy_generation: u32,
    policy_digest: [u8; 32],
    route_digest: [u8; 32],
) -> SettlementRecoveryState {
    SettlementRecoveryState::new(
        version,
        state_root,
        1,
        1,
        policy_generation,
        policy_digest,
        route_digest,
    )
}

#[test]
fn public_membership_accepts_example() {
    let mut shard_b = SettlementStore::new();
    let path_b = path(0x21, 5);
    shard_b
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(item(path_b)))])
        .expect("seed shard B");

    let proof = shard_b
        .settlement_proof_blob(&path_b)
        .expect("shard-local proof");
    let state_root = shard_b.settlement_root().expect("state root");
    let recovery = shard_b.recovery_state().expect("recovery");
    let shard_checkpoint = proof.hjmt_journal_checkpoint().expect("checkpoint");
    let policy_set = recovery.live_policy_set_v1(shard_checkpoint);
    let route_digest = bytes(0x44);

    let publication = CheckpointPublicationV1::new(
        RootGenerationTagV1::RootGeneration1,
        PublicationModeTagV1::CheckpointWindow,
        101,
        route_digest,
        SettlementStateRoot::settlement_v1(bytes(0x77)),
        vec![
            leaf(
                1,
                SettlementStateRoot::settlement_v1(bytes(0x11)),
                12,
                7,
                route_digest,
                &policy_set,
                220,
                40,
            ),
            leaf(
                2,
                state_root,
                12,
                7,
                route_digest,
                &policy_set,
                shard_checkpoint,
                41,
            ),
            leaf(
                3,
                SettlementStateRoot::settlement_v1(bytes(0x33)),
                12,
                7,
                route_digest,
                &policy_set,
                220,
                40,
            ),
        ],
    );
    let public_root = publication.public_root_v1().expect("public root");

    let public_proof = CheckpointPublicationProofV1::new(
        RootGenerationTagV1::RootGeneration1,
        public_root,
        publication,
        1,
        ShardProofContextV1::new(
            2,
            7,
            route_digest,
            u64::from(shard_b.bucket_policy().compatibility_generation()),
            shard_b.bucket_policy().bucket_policy_id(),
            HjmtProofFamily::Inclusion,
            SettlementLeafFamily::Terminal,
        ),
        policy_set,
        proof.clone(),
    );

    check_public_checkpoint_v1(&public_proof).expect("public proof contract");
    check_public_checkpoint_route_v1(
        &public_proof,
        &PublicationRouteSnapshotV1::new(7, route_digest, 101, vec![1, 2, 3]),
    )
    .expect("route snapshot contract");
    let checked = public_proof
        .verify_against_public_root_v1(public_root)
        .expect("two-layer public proof");
    assert_eq!(checked.item().path(), path_b);
    assert_eq!(public_root.generation_version(), 1);
}

#[test]
fn historical_proof_survives_publication() {
    let mut shard_b = SettlementStore::new();
    let path_b = path(0x31, 6);
    shard_b
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(item(path_b)))])
        .expect("seed shard B");

    let old_proof = shard_b
        .settlement_proof_blob(&path_b)
        .expect("old shard-local proof");
    let old_root = shard_b.settlement_root().expect("old root");
    let old_policy = shard_b.bucket_policy();
    let old_checkpoint = old_proof.hjmt_journal_checkpoint().expect("old checkpoint");
    let route_digest_v7 = bytes(0x54);
    let old_recovery = shard_b.recovery_state().expect("old recovery");
    let old_policy_set = old_recovery.live_policy_set_v1(old_checkpoint);
    let old_publication = CheckpointPublicationV1::new(
        RootGenerationTagV1::RootGeneration1,
        PublicationModeTagV1::CheckpointWindow,
        101,
        route_digest_v7,
        SettlementStateRoot::settlement_v1(bytes(0x80)),
        vec![leaf(
            2,
            old_root,
            12,
            7,
            route_digest_v7,
            &old_policy_set,
            old_checkpoint,
            41,
        )],
    );
    let old_public_root = old_publication.public_root_v1().expect("old public root");
    let old_public_proof = CheckpointPublicationProofV1::new(
        RootGenerationTagV1::RootGeneration1,
        old_public_root,
        old_publication.clone(),
        0,
        ShardProofContextV1::new(
            2,
            7,
            route_digest_v7,
            u64::from(old_policy.compatibility_generation()),
            old_policy.bucket_policy_id(),
            HjmtProofFamily::Inclusion,
            SettlementLeafFamily::Terminal,
        ),
        old_policy_set.clone(),
        old_proof.clone(),
    );
    old_public_proof
        .verify_against_public_root_v1(old_public_root)
        .expect("historical proof valid at publication 101");

    shard_b
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(item(path(0x32, 7))))])
        .expect("advance shard B");
    let new_root = shard_b.settlement_root().expect("new root");
    let route_digest_v8 = bytes(0x55);
    let new_recovery = recovery(
        202,
        new_root,
        shard_b.bucket_policy().compatibility_generation(),
        shard_b.bucket_policy().bucket_policy_id(),
        route_digest_v8,
    );
    let new_policy_set = new_recovery.live_policy_set_v1(102);
    let new_publication = CheckpointPublicationV1::new(
        RootGenerationTagV1::RootGeneration1,
        PublicationModeTagV1::CheckpointWindow,
        102,
        route_digest_v8,
        old_publication.public_root_v1().expect("old public root"),
        vec![leaf(
            2,
            new_root,
            13,
            8,
            route_digest_v8,
            &new_policy_set,
            202,
            42,
        )],
    );

    old_public_proof
        .verify_against_public_root_v1(old_public_root)
        .expect("historical proof remains bound to old publication");
    new_publication
        .check_prior_root_v1(old_public_root)
        .expect("publication continuity");
    new_publication
        .check_monotonic_successor_v1(&old_publication)
        .expect("monotonic successor");

    let wrong_route = CheckpointPublicationProofV1::new(
        RootGenerationTagV1::RootGeneration1,
        new_publication.public_root_v1().expect("new public root"),
        new_publication,
        0,
        ShardProofContextV1::new(
            2,
            7,
            route_digest_v7,
            u64::from(old_policy.compatibility_generation()),
            old_policy.bucket_policy_id(),
            HjmtProofFamily::Inclusion,
            SettlementLeafFamily::Terminal,
        ),
        old_policy_set,
        old_proof,
    );
    let err = wrong_route
        .verify_v1()
        .expect_err("route-generation drift must reject");
    assert_eq!(err, ProofChkErr::PublicationProofRouteMix);
}

#[test]
fn test_rejects_post_checkpoint_policy() {
    let mut shard_b = SettlementStore::new();
    let path_b = path(0x39, 9);
    shard_b
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(item(path_b)))])
        .expect("seed shard B");

    let proof = shard_b
        .settlement_proof_blob(&path_b)
        .expect("shard-local proof");
    let state_root = shard_b.settlement_root().expect("state root");
    let shard_checkpoint = proof.hjmt_journal_checkpoint().expect("checkpoint");
    let route_digest = bytes(0x57);
    let policy_set = PolicySetCommitmentV1::singleton_live(
        u64::from(shard_b.bucket_policy().compatibility_generation()),
        shard_b.bucket_policy().bucket_policy_id(),
        shard_checkpoint + 1,
    );
    let publication = CheckpointPublicationV1::new(
        RootGenerationTagV1::RootGeneration1,
        PublicationModeTagV1::CheckpointWindow,
        111,
        route_digest,
        SettlementStateRoot::settlement_v1(bytes(0x83)),
        vec![leaf(
            2,
            state_root,
            14,
            9,
            route_digest,
            &policy_set,
            shard_checkpoint,
            43,
        )],
    );
    let public_root = publication.public_root_v1().expect("public root");

    let public_proof = CheckpointPublicationProofV1::new(
        RootGenerationTagV1::RootGeneration1,
        public_root,
        publication,
        0,
        ShardProofContextV1::new(
            2,
            9,
            route_digest,
            u64::from(shard_b.bucket_policy().compatibility_generation()),
            shard_b.bucket_policy().bucket_policy_id(),
            HjmtProofFamily::Inclusion,
            SettlementLeafFamily::Terminal,
        ),
        policy_set,
        proof,
    );

    let err = public_proof
        .verify_against_public_root_v1(public_root)
        .expect_err("policy activation after proof checkpoint must reject");
    assert_eq!(err, ProofChkErr::PublicationPolicyMix);
}

#[test]
fn test_rejects_checkpoint_drift() {
    let mut shard_b = SettlementStore::new();
    let path_b = path(0x3A, 10);
    shard_b
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(item(path_b)))])
        .expect("seed shard B");

    let proof = shard_b
        .settlement_proof_blob(&path_b)
        .expect("shard-local proof");
    let state_root = shard_b.settlement_root().expect("state root");
    let shard_checkpoint = proof.hjmt_journal_checkpoint().expect("checkpoint");
    let route_digest = bytes(0x58);
    let recovery = shard_b.recovery_state().expect("recovery");
    let policy_set = recovery.live_policy_set_v1(shard_checkpoint);
    let publication = CheckpointPublicationV1::new(
        RootGenerationTagV1::RootGeneration1,
        PublicationModeTagV1::CheckpointWindow,
        112,
        route_digest,
        SettlementStateRoot::settlement_v1(bytes(0x84)),
        vec![leaf(
            2,
            state_root,
            15,
            9,
            route_digest,
            &policy_set,
            shard_checkpoint,
            44,
        )],
    );
    let public_root = publication.public_root_v1().expect("public root");
    let tampered_proof = proof.with_hjmt_journal_checkpoint(Some(shard_checkpoint + 1));
    let public_proof = CheckpointPublicationProofV1::new(
        RootGenerationTagV1::RootGeneration1,
        public_root,
        publication,
        0,
        ShardProofContextV1::new(
            2,
            9,
            route_digest,
            u64::from(shard_b.bucket_policy().compatibility_generation()),
            shard_b.bucket_policy().bucket_policy_id(),
            HjmtProofFamily::Inclusion,
            SettlementLeafFamily::Terminal,
        ),
        policy_set,
        tampered_proof,
    );

    let err = public_proof
        .verify_against_public_root_v1(public_root)
        .expect_err("journal checkpoint drift must reject");
    assert_eq!(err, ProofChkErr::PublicationProofCheckpointMix);
}

#[test]
fn test_cross_shard_counterexample_rejects() {
    let mut shard_b = SettlementStore::new();
    let path_b = path(0x41, 8);
    shard_b
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(item(path_b)))])
        .expect("seed shard B");

    let proof = shard_b
        .settlement_proof_blob(&path_b)
        .expect("shard-local proof");
    let state_root = shard_b.settlement_root().expect("state root");
    let route_digest = bytes(0x66);
    let recovery = shard_b.recovery_state().expect("recovery");
    let policy_set =
        recovery.live_policy_set_v1(proof.hjmt_journal_checkpoint().expect("checkpoint"));
    let publication = CheckpointPublicationV1::new(
        RootGenerationTagV1::RootGeneration1,
        PublicationModeTagV1::CheckpointWindow,
        101,
        route_digest,
        SettlementStateRoot::settlement_v1(bytes(0x99)),
        vec![
            leaf(
                2,
                state_root,
                12,
                7,
                route_digest,
                &policy_set,
                proof.hjmt_journal_checkpoint().expect("checkpoint"),
                41,
            ),
            leaf(
                3,
                SettlementStateRoot::settlement_v1(bytes(0x67)),
                12,
                7,
                route_digest,
                &policy_set,
                220,
                40,
            ),
        ],
    );
    let public_root = publication.public_root_v1().expect("public root");

    let cross_shard = CheckpointPublicationProofV1::new(
        RootGenerationTagV1::RootGeneration1,
        public_root,
        publication,
        1,
        ShardProofContextV1::new(
            2,
            7,
            route_digest,
            u64::from(shard_b.bucket_policy().compatibility_generation()),
            shard_b.bucket_policy().bucket_policy_id(),
            HjmtProofFamily::Inclusion,
            SettlementLeafFamily::Terminal,
        ),
        policy_set,
        proof,
    );

    let err = cross_shard
        .verify_against_public_root_v1(public_root)
        .expect_err("cross-shard public proof must reject");
    assert_eq!(err, ProofChkErr::PublicationProofShardMix);
}
