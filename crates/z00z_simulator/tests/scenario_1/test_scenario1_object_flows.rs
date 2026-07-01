use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use serde::Deserialize;
use tempfile::tempdir;
use z00z_core::{
    actions::{
        ActionDescriptorV1, ActionPoolDescriptorV1, LifecycleEffectV1, RequiredSignatureV1,
        WitnessRequirementV1,
    },
    assets::ObjectFamily,
    genesis::{
        GenesisPolicyRecord, GenesisSettlementManifest, GenesisVoucherRecord,
        GENESIS_POLICIES_FILE, GENESIS_SETTLEMENT_MANIFEST_FILE, GENESIS_VOUCHERS_FILE,
    },
    policies::{
        ConservationContributionV1, ExpiryRuleV1, PolicyDescriptorV1, ReplayProtectionV1,
        UnknownPolicyHandlingV1,
    },
    rights::{RightActionV1, RightRequirementV1, RightScopeV1},
    vouchers::{VoucherLifecycleV1, VoucherValidityWindowV1},
};
use z00z_crypto::expert::encoding::SafePassword;
use z00z_simulator::{
    config::ScenarioCfg,
    scenario_1::{support::fixture_cache, support::stage_runner_support},
};
use z00z_storage::settlement::{
    inspect_object_package, DefinitionId, FeeEnvelope, ObjectDeltaSetV1, ObjectPolicyRegistryV1,
    ObjectRejectCode, RightAction, RightLeaf, RightWitnessRefV1, RightWitnessStateV1, SerialId,
    SettlementActionV1, SettlementLeaf, SettlementObjectDeltaV1, SettlementPath,
    SettlementStateRoot, TerminalId, TerminalLeaf, VoucherAction, VoucherActionCtx,
    VoucherBackingRef, VoucherLeaf,
};
use z00z_utils::{
    codec::{Codec, JsonCodec, YamlCodec},
    io::{read_to_string, write_file},
    rng::SystemRngProvider,
};
use z00z_wallets::{
    db::redb_store::{ConfirmRef, ReceiveRef, ScanRef},
    db::{
        create_wallet_store, object_inventory_store, open_wallet_store, ObjectInventoryFilter,
        ObjectInventoryStore, ObjectSeenRef, OwnedObjectPayload, OwnedObjectPolicy,
        OwnedObjectSource, OwnedRightPayload, OwnedRightStatus, OwnedVoucherPayload,
        OwnedVoucherStatus, WalletIdentity, WalletObjectStatus, WalletPolicyAvailability,
    },
    rpc::types::common::{PersistTxId, PersistWalletId},
    tx::{validator_mandate_lock_payload_commitment, validator_mandate_lock_unlock_ready},
};

const TEST_SEED_PHRASE_24: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

static STAGE1_OUT: OnceLock<PathBuf> = OnceLock::new();

#[derive(Debug, Deserialize)]
struct Stage1SnapshotView {
    policy_count: usize,
    voucher_count: usize,
    policies_artifact_file: String,
    vouchers_artifact_file: String,
}

fn stage1_out() -> &'static PathBuf {
    STAGE1_OUT.get_or_init(|| {
        let root = fixture_cache::ensure_case("scenario1_object_flows_stage1_v1", |base| {
            let (cfg_path, design_path, out) = test_cfg_paths_in(base);
            let _ctx = stage_runner_support::run_stage_setup(&cfg_path, &design_path, &[1_u32]);
            assert!(
                out.join("genesis").exists(),
                "stage1 genesis output missing"
            );
        });
        root.join("outputs/scenario_1")
    })
}

fn test_cfg_paths_in(base: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let out = base.join("outputs/scenario_1");
    let mut cfg = ScenarioCfg::from_file(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/scenario_1/scenario_config.yaml"),
    )
    .expect("load scenario cfg");

    cfg.stage1_genesis
        .get_or_insert_with(Default::default)
        .genesis_config = z00z_core::config_paths::devnet_genesis_path()
        .to_string_lossy()
        .to_string();
    cfg.outputs.dir = out.to_string_lossy().to_string();
    if let Some(stage3) = cfg.stage3_claim.as_mut() {
        stage3.consume_bins = Some(false);
    }

    let cfg_path = base.join("scenario_config.yaml");
    let cfg_bytes = YamlCodec.serialize(&cfg).expect("serialize cfg");
    write_file(&cfg_path, &cfg_bytes).expect("write cfg");

    let design_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/scenario_1/scenario_design.yaml");
    (cfg_path, design_path, out)
}

fn bytes(mark: u8) -> [u8; 32] {
    [mark; 32]
}

fn settlement_root(mark: u8) -> SettlementStateRoot {
    SettlementStateRoot::settlement_v1(bytes(mark))
}

fn asset_leaf(path: SettlementPath) -> TerminalLeaf {
    let mut leaf = TerminalLeaf::dummy_for_scan(path.serial_id.get());
    leaf.asset_id = path.terminal_id.into_bytes();
    leaf.serial_id = path.serial_id.get();
    leaf
}

fn voucher_path(mark: u8) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new(bytes(mark)),
        SerialId::new(u32::from(mark) + 1),
        TerminalId::new(bytes(mark.wrapping_add(1))),
    )
}

fn asset_path(mark: u8) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new(bytes(mark.wrapping_add(10))),
        SerialId::new(u32::from(mark) + 2),
        TerminalId::new(bytes(mark.wrapping_add(11))),
    )
}

fn right_path(mark: u8) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new(bytes(mark.wrapping_add(20))),
        SerialId::new(u32::from(mark) + 3),
        TerminalId::new(bytes(mark.wrapping_add(21))),
    )
}

fn voucher_leaf(mark: u8) -> VoucherLeaf {
    VoucherLeaf {
        version: 1,
        terminal_id: voucher_path(mark).terminal_id,
        issuer_commitment: bytes(mark.wrapping_add(2)),
        holder_commitment: bytes(mark.wrapping_add(3)),
        beneficiary_commitment: bytes(mark.wrapping_add(4)),
        refund_target_commitment: bytes(mark.wrapping_add(5)),
        backing: VoucherBackingRef::ReserveCommitment(bytes(mark.wrapping_add(6))),
        face_value: 95,
        remaining_value: 95,
        policy_id: bytes(mark.wrapping_add(7)),
        action_pool_id: bytes(mark.wrapping_add(8)),
        lifecycle: VoucherLifecycleV1::Active,
        validity: VoucherValidityWindowV1 {
            valid_from: 1,
            valid_until: 100,
        },
        receiver_must_accept: true,
        allow_reject: true,
        replay_nonce: bytes(mark.wrapping_add(9)),
        disclosure_commitment: Some(bytes(mark.wrapping_add(10))),
        audit_commitment: Some(bytes(mark.wrapping_add(11))),
    }
}

fn right_leaf(mark: u8) -> RightLeaf {
    RightLeaf {
        version: 1,
        terminal_id: right_path(mark).terminal_id,
        right_class: z00z_storage::settlement::RightClass::ServiceEntitlement,
        issuer_scope: bytes(mark.wrapping_add(30)),
        provider_scope: bytes(mark.wrapping_add(31)),
        holder_commitment: bytes(mark.wrapping_add(32)),
        control_commitment: bytes(mark.wrapping_add(33)),
        beneficiary_commitment: bytes(mark.wrapping_add(34)),
        payload_commitment: bytes(mark.wrapping_add(35)),
        valid_from: 1,
        valid_until: 100,
        challenge_from: 2,
        challenge_until: 90,
        use_nonce: bytes(mark.wrapping_add(36)),
        revocation_policy_id: bytes(mark.wrapping_add(37)),
        transition_policy_id: bytes(mark.wrapping_add(38)),
        challenge_policy_id: bytes(mark.wrapping_add(39)),
        disclosure_policy_id: bytes(mark.wrapping_add(40)),
        retention_policy_id: bytes(mark.wrapping_add(41)),
    }
}

fn fee_envelope(mark: u8, support_ref: [u8; 32]) -> FeeEnvelope {
    let support_ref = Some(support_ref);
    let budget_units = 3_u64;
    FeeEnvelope {
        version: 1,
        payer_commitment: bytes(mark),
        sponsor_commitment: [0u8; 32],
        budget_units,
        budget_commitment: FeeEnvelope::budget_bind(budget_units, support_ref),
        domain_id: bytes(mark.wrapping_add(1)),
        expires_at: 999,
        nonce: bytes(mark.wrapping_add(2)),
        transition_id: bytes(mark.wrapping_add(3)),
        replay_key: bytes(mark.wrapping_add(4)),
        support_ref,
        failure_policy_id: bytes(mark.wrapping_add(5)),
    }
}

fn voucher_redeem_policy_contract() -> (PolicyDescriptorV1, ActionPoolDescriptorV1, [u8; 32]) {
    let action = ActionDescriptorV1 {
        label: "voucher_redeem_full".to_string(),
        allowed_input_families: BTreeSet::from([ObjectFamily::Voucher]),
        allowed_output_families: BTreeSet::from([ObjectFamily::Asset]),
        lifecycle_effect: LifecycleEffectV1::Redeem,
        witness_requirements: BTreeSet::from([
            WitnessRequirementV1::Signature(RequiredSignatureV1::Holder),
            WitnessRequirementV1::AcceptanceProof,
            WitnessRequirementV1::ReplayNonce,
            WitnessRequirementV1::PriorStateRoot,
            WitnessRequirementV1::RightReference("kyc_v1".to_string()),
        ]),
        receiver_must_accept: false,
        preserves_beneficiary: true,
        preserves_refund_authority: true,
    };
    let action_id = action.action_id().expect("action id").bytes();
    let action_pool = ActionPoolDescriptorV1 {
        label: "voucher_pool_v1".to_string(),
        actions: BTreeSet::from([action]),
    };
    let policy = PolicyDescriptorV1 {
        label: "voucher_policy_v1".to_string(),
        domain_name: "z00z.simulator.scenario1.voucher_policy.v1".to_string(),
        primary_family: ObjectFamily::Voucher,
        allowed_input_families: BTreeSet::from([ObjectFamily::Voucher]),
        allowed_output_families: BTreeSet::from([ObjectFamily::Asset]),
        action_pool_id: action_pool.action_pool_id().expect("pool id"),
        action_ids: action_pool.action_ids().expect("action ids"),
        conditions: BTreeSet::new(),
        required_rights: BTreeSet::from([RightRequirementV1 {
            right_policy: "kyc_v1".to_string(),
            allowed_actions: BTreeSet::from([RightActionV1::Use]),
            scope: RightScopeV1::ObjectFamily(ObjectFamily::Voucher),
            max_uses: Some(1),
            delegation_allowed: false,
            attenuation_only: true,
        }]),
        required_signatures: BTreeSet::from([RequiredSignatureV1::Holder]),
        required_attestations: BTreeSet::new(),
        expiry_rule: ExpiryRuleV1::ValidUntil,
        replay_protection: ReplayProtectionV1::NonceAndRoot,
        conservation: ConservationContributionV1::ConditionalValue,
        unknown_policy_handling: UnknownPolicyHandlingV1::default(),
    };
    (policy, action_pool, action_id)
}

fn voucher_issue_policy_contract() -> (PolicyDescriptorV1, ActionPoolDescriptorV1, [u8; 32]) {
    let action = ActionDescriptorV1 {
        label: "voucher_issue".to_string(),
        allowed_input_families: BTreeSet::from([ObjectFamily::Voucher]),
        allowed_output_families: BTreeSet::from([ObjectFamily::Voucher]),
        lifecycle_effect: LifecycleEffectV1::Offer,
        witness_requirements: BTreeSet::new(),
        receiver_must_accept: false,
        preserves_beneficiary: true,
        preserves_refund_authority: true,
    };
    let action_id = action.action_id().expect("action id").bytes();
    let action_pool = ActionPoolDescriptorV1 {
        label: "voucher_issue_pool_v1".to_string(),
        actions: BTreeSet::from([action]),
    };
    let policy = PolicyDescriptorV1 {
        label: "voucher_issue_policy_v1".to_string(),
        domain_name: "z00z.simulator.scenario1.voucher_issue_policy.v1".to_string(),
        primary_family: ObjectFamily::Voucher,
        allowed_input_families: BTreeSet::from([ObjectFamily::Voucher]),
        allowed_output_families: BTreeSet::from([ObjectFamily::Voucher]),
        action_pool_id: action_pool.action_pool_id().expect("pool id"),
        action_ids: action_pool.action_ids().expect("action ids"),
        conditions: BTreeSet::new(),
        required_rights: BTreeSet::new(),
        required_signatures: BTreeSet::new(),
        required_attestations: BTreeSet::new(),
        expiry_rule: ExpiryRuleV1::None,
        replay_protection: ReplayProtectionV1::None,
        conservation: ConservationContributionV1::ConditionalValue,
        unknown_policy_handling: UnknownPolicyHandlingV1::default(),
    };
    (policy, action_pool, action_id)
}

fn right_consume_policy_contract() -> (PolicyDescriptorV1, ActionPoolDescriptorV1, [u8; 32]) {
    let action = ActionDescriptorV1 {
        label: "right_consume".to_string(),
        allowed_input_families: BTreeSet::from([ObjectFamily::Right]),
        allowed_output_families: BTreeSet::from([ObjectFamily::Right]),
        lifecycle_effect: LifecycleEffectV1::Use,
        witness_requirements: BTreeSet::new(),
        receiver_must_accept: false,
        preserves_beneficiary: true,
        preserves_refund_authority: true,
    };
    let action_id = action.action_id().expect("action id").bytes();
    let action_pool = ActionPoolDescriptorV1 {
        label: "right_pool_v1".to_string(),
        actions: BTreeSet::from([action]),
    };
    let policy = PolicyDescriptorV1 {
        label: "right_policy_v1".to_string(),
        domain_name: "z00z.simulator.scenario1.right_policy.v1".to_string(),
        primary_family: ObjectFamily::Right,
        allowed_input_families: BTreeSet::from([ObjectFamily::Right]),
        allowed_output_families: BTreeSet::from([ObjectFamily::Right]),
        action_pool_id: action_pool.action_pool_id().expect("pool id"),
        action_ids: action_pool.action_ids().expect("action ids"),
        conditions: BTreeSet::new(),
        required_rights: BTreeSet::new(),
        required_signatures: BTreeSet::new(),
        required_attestations: BTreeSet::new(),
        expiry_rule: ExpiryRuleV1::ValidUntil,
        replay_protection: ReplayProtectionV1::None,
        conservation: ConservationContributionV1::ZeroValueAuthority,
        unknown_policy_handling: UnknownPolicyHandlingV1::default(),
    };
    (policy, action_pool, action_id)
}

fn validator_locked_asset_policy_contract() -> (PolicyDescriptorV1, ActionPoolDescriptorV1, [u8; 32])
{
    let action = ActionDescriptorV1 {
        label: "validator_unlock_after_expiry".to_string(),
        allowed_input_families: BTreeSet::from([ObjectFamily::Asset, ObjectFamily::Right]),
        allowed_output_families: BTreeSet::from([ObjectFamily::Asset]),
        lifecycle_effect: LifecycleEffectV1::Expire,
        witness_requirements: BTreeSet::from([
            WitnessRequirementV1::Signature(RequiredSignatureV1::Controller),
            WitnessRequirementV1::RightReference("validator_mandate_lock_v1".to_string()),
            WitnessRequirementV1::ReplayNonce,
            WitnessRequirementV1::PriorStateRoot,
        ]),
        receiver_must_accept: false,
        preserves_beneficiary: true,
        preserves_refund_authority: true,
    };
    let action_id = action.action_id().expect("action id").bytes();
    let action_pool = ActionPoolDescriptorV1 {
        label: "validator_lock_pool_v1".to_string(),
        actions: BTreeSet::from([action]),
    };
    let policy = PolicyDescriptorV1 {
        label: "validator_lock_policy_v1".to_string(),
        domain_name: "z00z.simulator.scenario1.validator_lock_policy.v1".to_string(),
        primary_family: ObjectFamily::Asset,
        allowed_input_families: BTreeSet::from([ObjectFamily::Asset, ObjectFamily::Right]),
        allowed_output_families: BTreeSet::from([ObjectFamily::Asset]),
        action_pool_id: action_pool.action_pool_id().expect("pool id"),
        action_ids: action_pool.action_ids().expect("action ids"),
        conditions: BTreeSet::new(),
        required_rights: BTreeSet::from([RightRequirementV1 {
            right_policy: "validator_mandate_lock_v1".to_string(),
            allowed_actions: BTreeSet::from([RightActionV1::Use]),
            scope: RightScopeV1::ObjectFamily(ObjectFamily::Asset),
            max_uses: Some(1),
            delegation_allowed: false,
            attenuation_only: true,
        }]),
        required_signatures: BTreeSet::from([RequiredSignatureV1::Controller]),
        required_attestations: BTreeSet::new(),
        expiry_rule: ExpiryRuleV1::ValidUntil,
        replay_protection: ReplayProtectionV1::NonceAndRoot,
        conservation: ConservationContributionV1::FinalValue,
        unknown_policy_handling: UnknownPolicyHandlingV1::default(),
    };
    (policy, action_pool, action_id)
}

fn validator_lock_leaf(mark: u8, locked_asset_id: [u8; 32], locked_amount: u64) -> RightLeaf {
    let terminal_id = TerminalId::new(bytes(mark.wrapping_add(60)));
    let mut leaf = RightLeaf {
        version: 1,
        terminal_id,
        right_class: z00z_storage::settlement::RightClass::ValidatorMandate,
        issuer_scope: bytes(mark.wrapping_add(61)),
        provider_scope: bytes(mark.wrapping_add(62)),
        holder_commitment: bytes(mark.wrapping_add(63)),
        control_commitment: bytes(mark.wrapping_add(64)),
        beneficiary_commitment: bytes(mark.wrapping_add(65)),
        payload_commitment: [0u8; 32],
        valid_from: 1,
        valid_until: 100,
        challenge_from: 101,
        challenge_until: 140,
        use_nonce: bytes(mark.wrapping_add(66)),
        revocation_policy_id: bytes(mark.wrapping_add(67)),
        transition_policy_id: bytes(mark.wrapping_add(68)),
        challenge_policy_id: bytes(mark.wrapping_add(69)),
        disclosure_policy_id: bytes(mark.wrapping_add(70)),
        retention_policy_id: bytes(mark.wrapping_add(71)),
    };
    leaf.payload_commitment =
        validator_mandate_lock_payload_commitment(&locked_asset_id, locked_amount, &leaf);
    leaf
}

fn validator_locked_asset_package(
    policy: &PolicyDescriptorV1,
    action_pool: &ActionPoolDescriptorV1,
    action_id: [u8; 32],
    mark: u8,
    witness_state: RightWitnessStateV1,
    include_right_delta: bool,
    has_replay_nonce: bool,
) -> z00z_storage::settlement::RuntimeObjectPackageV1 {
    let prior_root = settlement_root(mark.wrapping_add(90));
    let next_root = settlement_root(mark.wrapping_add(91));
    let policy_hash = policy.policy_id().expect("policy id").bytes();
    let action_pool_id = action_pool.action_pool_id().expect("pool id").bytes();
    let locked_amount = 25_u64;
    let locked_asset_path = asset_path(mark.wrapping_add(20));
    let unlocked_asset_path = asset_path(mark.wrapping_add(21));
    let mut deleted_objects = vec![SettlementObjectDeltaV1::deleted(
        locked_asset_path,
        SettlementLeaf::Terminal(asset_leaf(locked_asset_path)),
        Some(locked_amount),
    )];
    if include_right_delta {
        let right_path = right_path(mark.wrapping_add(22));
        let mut right = validator_lock_leaf(
            mark,
            locked_asset_path.terminal_id.into_bytes(),
            locked_amount,
        );
        right.terminal_id = right_path.terminal_id;
        deleted_objects.push(SettlementObjectDeltaV1::deleted(
            right_path,
            SettlementLeaf::Right(right),
            None,
        ));
    }
    let delta_set = ObjectDeltaSetV1::new(
        SettlementActionV1::AssetMutation,
        policy_hash,
        None,
        deleted_objects,
        vec![SettlementObjectDeltaV1::created(
            unlocked_asset_path,
            SettlementLeaf::Terminal(asset_leaf(unlocked_asset_path)),
            Some(locked_amount),
        )],
        Vec::new(),
        None,
        prior_root,
        next_root,
    );

    z00z_storage::settlement::RuntimeObjectPackageV1 {
        primary_family: ObjectFamily::Asset,
        selected_action: SettlementActionV1::AssetMutation,
        selected_action_id: action_id,
        policy_descriptor_hash: policy_hash,
        action_pool_id,
        required_rights: vec![RightWitnessRefV1 {
            right_policy: "validator_mandate_lock_v1".to_string(),
            witness_state,
        }],
        object_witnesses: z00z_storage::settlement::ObjectWitnessBundleV1 {
            signatures: BTreeSet::from([RequiredSignatureV1::Controller]),
            attestation_labels: BTreeSet::new(),
            has_acceptance_proof: false,
            has_replay_nonce,
            has_prior_root_binding: true,
            has_disclosure_commitment: false,
        },
        delta_set,
        fee_support_ref: None,
        prior_root,
        expected_new_root: next_root,
    }
}

fn asset_wrapped_voucher_policy_contract() -> (PolicyDescriptorV1, ActionPoolDescriptorV1, [u8; 32])
{
    let (mut policy, action_pool, action_id) = voucher_redeem_policy_contract();
    policy.label = "asset_wrapped_voucher_policy_v1".to_string();
    policy.primary_family = ObjectFamily::Asset;
    policy.allowed_input_families = BTreeSet::from([ObjectFamily::Asset]);
    policy.allowed_output_families = BTreeSet::from([ObjectFamily::Asset]);
    policy.conservation = ConservationContributionV1::FinalValue;
    (policy, action_pool, action_id)
}

fn voucher_redeem_package(
    policy: &PolicyDescriptorV1,
    action_pool: &ActionPoolDescriptorV1,
    action_id: [u8; 32],
    mark: u8,
    mut voucher: VoucherLeaf,
) -> z00z_storage::settlement::RuntimeObjectPackageV1 {
    let prior_root = settlement_root(mark);
    let next_root = settlement_root(mark.wrapping_add(1));
    let policy_hash = policy.policy_id().expect("policy id").bytes();
    let action_pool_id = action_pool.action_pool_id().expect("pool id").bytes();
    voucher.policy_id = policy_hash;
    voucher.action_pool_id = action_pool_id;
    let voucher_path = voucher_path(mark);
    voucher.terminal_id = voucher_path.terminal_id;
    let created_asset_path = asset_path(mark);
    let delta_set = ObjectDeltaSetV1::new(
        SettlementActionV1::Voucher(VoucherAction::RedeemFull),
        policy_hash,
        Some(VoucherActionCtx {
            now: 20,
            expected_holder: Some(voucher.holder_commitment),
            expected_beneficiary: Some(voucher.beneficiary_commitment),
            expected_refund_target: Some(voucher.refund_target_commitment),
            acceptance_confirmed: true,
            ..VoucherActionCtx::default()
        }),
        vec![SettlementObjectDeltaV1::deleted(
            voucher_path,
            SettlementLeaf::Voucher(voucher.clone()),
            None,
        )],
        vec![SettlementObjectDeltaV1::created(
            created_asset_path,
            SettlementLeaf::Terminal(asset_leaf(created_asset_path)),
            Some(voucher.remaining_value),
        )],
        Vec::new(),
        None,
        prior_root,
        next_root,
    );

    z00z_storage::settlement::RuntimeObjectPackageV1 {
        primary_family: ObjectFamily::Voucher,
        selected_action: SettlementActionV1::Voucher(VoucherAction::RedeemFull),
        selected_action_id: action_id,
        policy_descriptor_hash: policy_hash,
        action_pool_id,
        required_rights: vec![RightWitnessRefV1 {
            right_policy: "kyc_v1".to_string(),
            witness_state: RightWitnessStateV1::Present,
        }],
        object_witnesses: z00z_storage::settlement::ObjectWitnessBundleV1 {
            signatures: BTreeSet::from([RequiredSignatureV1::Holder]),
            attestation_labels: BTreeSet::new(),
            has_acceptance_proof: true,
            has_replay_nonce: true,
            has_prior_root_binding: true,
            has_disclosure_commitment: false,
        },
        delta_set,
        fee_support_ref: None,
        prior_root,
        expected_new_root: next_root,
    }
}

fn voucher_issue_package(
    policy: &PolicyDescriptorV1,
    action_pool: &ActionPoolDescriptorV1,
    action_id: [u8; 32],
    mark: u8,
    mut voucher: VoucherLeaf,
) -> z00z_storage::settlement::RuntimeObjectPackageV1 {
    let prior_root = settlement_root(mark.wrapping_add(50));
    let next_root = settlement_root(mark.wrapping_add(51));
    let policy_hash = policy.policy_id().expect("policy id").bytes();
    let action_pool_id = action_pool.action_pool_id().expect("pool id").bytes();
    voucher.policy_id = policy_hash;
    voucher.action_pool_id = action_pool_id;
    let voucher_path = voucher_path(mark.wrapping_add(50));
    voucher.terminal_id = voucher_path.terminal_id;
    let delta_set = ObjectDeltaSetV1::new(
        SettlementActionV1::Voucher(VoucherAction::Issue),
        policy_hash,
        Some(VoucherActionCtx {
            now: 10,
            expected_holder: Some(voucher.holder_commitment),
            expected_beneficiary: Some(voucher.beneficiary_commitment),
            expected_refund_target: Some(voucher.refund_target_commitment),
            ..VoucherActionCtx::default()
        }),
        Vec::new(),
        vec![SettlementObjectDeltaV1::created(
            voucher_path,
            SettlementLeaf::Voucher(voucher),
            None,
        )],
        Vec::new(),
        None,
        prior_root,
        next_root,
    );

    z00z_storage::settlement::RuntimeObjectPackageV1 {
        primary_family: ObjectFamily::Voucher,
        selected_action: SettlementActionV1::Voucher(VoucherAction::Issue),
        selected_action_id: action_id,
        policy_descriptor_hash: policy_hash,
        action_pool_id,
        required_rights: Vec::new(),
        object_witnesses: z00z_storage::settlement::ObjectWitnessBundleV1 {
            signatures: BTreeSet::new(),
            attestation_labels: BTreeSet::new(),
            has_acceptance_proof: false,
            has_replay_nonce: false,
            has_prior_root_binding: false,
            has_disclosure_commitment: false,
        },
        delta_set,
        fee_support_ref: None,
        prior_root,
        expected_new_root: next_root,
    }
}

fn right_value_package(
    policy: &PolicyDescriptorV1,
    action_pool: &ActionPoolDescriptorV1,
    action_id: [u8; 32],
    mark: u8,
) -> z00z_storage::settlement::RuntimeObjectPackageV1 {
    let prior_root = settlement_root(mark.wrapping_add(80));
    let next_root = settlement_root(mark.wrapping_add(81));
    let policy_hash = policy.policy_id().expect("policy id").bytes();
    let action_pool_id = action_pool.action_pool_id().expect("pool id").bytes();
    let path = right_path(mark);
    let leaf = right_leaf(mark);
    let delta_set = ObjectDeltaSetV1::new(
        SettlementActionV1::Right(RightAction::Consume),
        policy_hash,
        None,
        vec![SettlementObjectDeltaV1::deleted(
            path,
            SettlementLeaf::Right(leaf),
            Some(1),
        )],
        Vec::new(),
        Vec::new(),
        None,
        prior_root,
        next_root,
    );

    z00z_storage::settlement::RuntimeObjectPackageV1 {
        primary_family: ObjectFamily::Right,
        selected_action: SettlementActionV1::Right(RightAction::Consume),
        selected_action_id: action_id,
        policy_descriptor_hash: policy_hash,
        action_pool_id,
        required_rights: Vec::new(),
        object_witnesses: z00z_storage::settlement::ObjectWitnessBundleV1 {
            signatures: BTreeSet::new(),
            attestation_labels: BTreeSet::new(),
            has_acceptance_proof: false,
            has_replay_nonce: false,
            has_prior_root_binding: false,
            has_disclosure_commitment: false,
        },
        delta_set,
        fee_support_ref: None,
        prior_root,
        expected_new_root: next_root,
    }
}

fn voucher_payload(wallet_id: PersistWalletId, tag: u8) -> OwnedVoucherPayload {
    let terminal_id = TerminalId::new(bytes(tag));
    let mut payload = OwnedVoucherPayload {
        version: OwnedVoucherPayload::VERSION,
        wallet_id,
        account_id: Some(u128::from(tag)),
        terminal_id,
        voucher_leaf: VoucherLeaf {
            version: 1,
            terminal_id,
            issuer_commitment: bytes(tag.wrapping_add(1)),
            holder_commitment: bytes(tag.wrapping_add(2)),
            beneficiary_commitment: bytes(tag.wrapping_add(3)),
            refund_target_commitment: bytes(tag.wrapping_add(4)),
            backing: VoucherBackingRef::ReserveCommitment(bytes(tag.wrapping_add(5))),
            face_value: 50,
            remaining_value: 50,
            policy_id: bytes(tag.wrapping_add(6)),
            action_pool_id: bytes(tag.wrapping_add(7)),
            lifecycle: VoucherLifecycleV1::Active,
            validity: VoucherValidityWindowV1 {
                valid_from: 10,
                valid_until: 100,
            },
            receiver_must_accept: true,
            allow_reject: true,
            replay_nonce: bytes(tag.wrapping_add(8)),
            disclosure_commitment: Some(bytes(tag.wrapping_add(9))),
            audit_commitment: Some(bytes(tag.wrapping_add(10))),
        },
        status: OwnedVoucherStatus::Redeemable,
        source: OwnedObjectSource::Import,
        first_seen: Some(ObjectSeenRef {
            height: Some(10),
            hash_or_root: Some(bytes(tag).to_vec()),
            local_time_ms: 1_111,
        }),
        last_updated_ms: 1_222,
        scan_ref: Some(ScanRef {
            start_height: 8,
            end_height: 10,
            cursor_hash: bytes(tag.wrapping_add(11)).to_vec(),
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
        policy: OwnedObjectPolicy {
            policy_id: Some(bytes(tag.wrapping_add(6))),
            availability: WalletPolicyAvailability::Available,
            manual_review: false,
            quarantine_reason: None,
        },
        holder_opening: Some(vec![tag; 8]),
        beneficiary_opening: Some(vec![tag.wrapping_add(1); 8]),
        checksum: None,
    };
    payload.checksum = Some(payload.compute_checksum());
    payload
}

fn right_payload(wallet_id: PersistWalletId, tag: u8) -> OwnedRightPayload {
    let terminal_id = TerminalId::new(bytes(tag.wrapping_add(30)));
    let mut payload = OwnedRightPayload {
        version: OwnedRightPayload::VERSION,
        wallet_id,
        account_id: Some(u128::from(tag) + 100),
        terminal_id,
        right_leaf: RightLeaf {
            version: 1,
            terminal_id,
            right_class: z00z_storage::settlement::RightClass::ServiceEntitlement,
            issuer_scope: bytes(tag.wrapping_add(31)),
            provider_scope: bytes(tag.wrapping_add(32)),
            holder_commitment: bytes(tag.wrapping_add(33)),
            control_commitment: bytes(tag.wrapping_add(34)),
            beneficiary_commitment: bytes(tag.wrapping_add(35)),
            payload_commitment: bytes(tag.wrapping_add(36)),
            valid_from: 10,
            valid_until: 100,
            challenge_from: 20,
            challenge_until: 90,
            use_nonce: bytes(tag.wrapping_add(37)),
            revocation_policy_id: bytes(tag.wrapping_add(38)),
            transition_policy_id: bytes(tag.wrapping_add(39)),
            challenge_policy_id: bytes(tag.wrapping_add(40)),
            disclosure_policy_id: bytes(tag.wrapping_add(41)),
            retention_policy_id: bytes(tag.wrapping_add(42)),
        },
        status: OwnedRightStatus::Granted,
        source: OwnedObjectSource::Import,
        first_seen: Some(ObjectSeenRef {
            height: Some(12),
            hash_or_root: Some(bytes(tag.wrapping_add(43)).to_vec()),
            local_time_ms: 2_222,
        }),
        last_updated_ms: 2_333,
        scan_ref: Some(ScanRef {
            start_height: 10,
            end_height: 12,
            cursor_hash: bytes(tag.wrapping_add(44)).to_vec(),
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
        policy: OwnedObjectPolicy {
            policy_id: Some(bytes(tag.wrapping_add(39))),
            availability: WalletPolicyAvailability::Available,
            manual_review: false,
            quarantine_reason: None,
        },
        holder_opening: Some(vec![tag.wrapping_add(45); 8]),
        control_opening: Some(vec![tag.wrapping_add(46); 8]),
        beneficiary_opening: Some(vec![tag.wrapping_add(47); 8]),
        checksum: None,
    };
    payload.checksum = Some(payload.compute_checksum());
    payload
}

#[test]
fn test_scenario1_object_flows_matrix_contract() {
    let cfg = ScenarioCfg::from_file(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/scenario_1/scenario_config.yaml"),
    )
    .expect("load cfg");
    let matrix = cfg
        .object_flow_matrix
        .expect("object_flow_matrix must be present");

    assert_eq!(matrix.positive.len(), 18);
    assert_eq!(matrix.negative.len(), 15);

    let positive_ids = matrix
        .positive
        .iter()
        .map(|item| item.id.as_str())
        .collect::<BTreeSet<_>>();
    let negative_ids = matrix
        .negative
        .iter()
        .map(|item| item.id.as_str())
        .collect::<BTreeSet<_>>();

    assert_eq!(
        positive_ids,
        BTreeSet::from([
            "asset_alice_to_bob",
            "asset_bob_to_charlie",
            "voucher_issue_offer",
            "voucher_accept",
            "voucher_transfer",
            "voucher_redeem_full",
            "voucher_redeem_partial",
            "voucher_reject_refund",
            "voucher_expiry",
            "right_grant",
            "right_delegate",
            "right_consume",
            "right_revoke",
            "right_expiry",
            "right_challenge",
            "right_gated_voucher_action",
            "validator_lock_unlock_after_expiry",
            "fee_supported_transition",
        ])
    );
    assert_eq!(
        negative_ids,
        BTreeSet::from([
            "policy_unknown_reject",
            "voucher_invalid_backing",
            "voucher_non_transferable_transfer_reject",
            "voucher_forced_acceptance",
            "voucher_double_redeem",
            "voucher_as_cash_reject",
            "voucher_expired_use_reject",
            "right_missing_for_voucher_action",
            "right_expired_for_voucher_action",
            "right_revoked_for_voucher_action",
            "right_replay_reject",
            "validator_lock_unlock_without_right_delta_reject",
            "validator_lock_unlock_replay_reject",
            "right_as_value_reject",
            "wrong_family_proof_reject",
        ])
    );

    let positive_actors = matrix
        .positive
        .iter()
        .flat_map(|item| item.actors.iter().map(String::as_str))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        positive_actors,
        BTreeSet::from(["alice", "bob", "charlie", "sequencer"])
    );

    let tri_actor = matrix
        .positive
        .iter()
        .find(|item| item.id == "right_gated_voucher_action")
        .expect("right_gated_voucher_action row");
    assert_eq!(tri_actor.actors, vec!["alice", "bob", "charlie"]);
    assert_eq!(
        tri_actor.required_rights,
        vec!["service_entitlement", "validator_mandate"]
    );

    let validator_unlock = matrix
        .positive
        .iter()
        .find(|item| item.id == "validator_lock_unlock_after_expiry")
        .expect("validator_lock_unlock_after_expiry row");
    assert_eq!(validator_unlock.actors, vec!["alice"]);
    assert_eq!(validator_unlock.required_rights, vec!["validator_mandate"]);
    assert_eq!(validator_unlock.policy_label, "validator_lock_policy_v1");
    assert!(
        validator_unlock
            .evidence_files
            .iter()
            .any(|file| file == "right_flow.json"),
        "validator lock unlock flow must stay anchored to right_flow.json",
    );

    for reject_id in [
        "validator_lock_unlock_without_right_delta_reject",
        "validator_lock_unlock_replay_reject",
    ] {
        let row = matrix
            .negative
            .iter()
            .find(|item| item.id == reject_id)
            .expect("validator lock reject row");
        assert_eq!(row.actors, vec!["alice"]);
        assert_eq!(row.required_rights, vec!["validator_mandate"]);
        assert_eq!(row.policy_label, "validator_lock_policy_v1");
        assert!(
            row.evidence_files
                .iter()
                .any(|file| file == "right_flow.json"),
            "{reject_id} must stay anchored to right_flow.json",
        );
    }

    for case in matrix.positive.iter().chain(matrix.negative.iter()) {
        assert!(
            !case.policy_label.trim().is_empty(),
            "{} policy_label must stay populated",
            case.id
        );
        assert!(
            !case.evidence_files.is_empty(),
            "{} evidence_files must stay populated",
            case.id
        );
        assert!(
            case.evidence_files
                .iter()
                .all(|file| file.ends_with(".json") || file.ends_with(".md")),
            "{} evidence files must remain packet artifacts",
            case.id
        );
        if matches!(case.family.as_str(), "voucher" | "cross_object") {
            assert!(
                case.evidence_files
                    .iter()
                    .any(|file| file == "voucher_flow.json"),
                "{} must stay anchored to voucher_flow.json",
                case.id
            );
        }
        if matches!(case.family.as_str(), "right" | "cross_object") {
            assert!(
                case.evidence_files
                    .iter()
                    .any(|file| file == "right_flow.json")
                    || case.id == "wrong_family_proof_reject"
                    || case.id == "policy_unknown_reject",
                "{} must stay anchored to right_flow.json when rights participate",
                case.id
            );
        }
        if case.expected_verdict != "accepted" {
            assert!(
                case.expected_verdict.starts_with("rejected:OBJECT_"),
                "{} reject verdict must stay on canonical OBJECT_* code path",
                case.id
            );
        }
    }
}

#[test]
fn test_stage1_policy_voucher_artifacts() {
    let genesis_dir = stage1_out().join("genesis");
    let snapshot_path = stage1_out().join("stage_1_snapshot.json");
    let policies_path = genesis_dir.join(GENESIS_POLICIES_FILE);
    let vouchers_path = genesis_dir.join(GENESIS_VOUCHERS_FILE);
    let manifest_path = genesis_dir.join(GENESIS_SETTLEMENT_MANIFEST_FILE);

    let snapshot: Stage1SnapshotView = JsonCodec
        .deserialize(
            read_to_string(&snapshot_path)
                .expect("read stage1 snapshot")
                .as_bytes(),
        )
        .expect("decode stage1 snapshot");
    let policies: Vec<GenesisPolicyRecord> = JsonCodec
        .deserialize(
            read_to_string(&policies_path)
                .expect("read policies artifact")
                .as_bytes(),
        )
        .expect("decode policies artifact");
    let vouchers: Vec<GenesisVoucherRecord> = JsonCodec
        .deserialize(
            read_to_string(&vouchers_path)
                .expect("read vouchers artifact")
                .as_bytes(),
        )
        .expect("decode vouchers artifact");
    let manifest: GenesisSettlementManifest = JsonCodec
        .deserialize(
            read_to_string(&manifest_path)
                .expect("read manifest artifact")
                .as_bytes(),
        )
        .expect("decode manifest artifact");

    assert!(
        !policies.is_empty(),
        "phase059 policies artifact must not be empty"
    );
    assert!(
        !vouchers.is_empty(),
        "phase059 vouchers artifact must not be empty"
    );
    assert_eq!(snapshot.policies_artifact_file, GENESIS_POLICIES_FILE);
    assert_eq!(snapshot.vouchers_artifact_file, GENESIS_VOUCHERS_FILE);
    assert_eq!(snapshot.policy_count, policies.len());
    assert_eq!(snapshot.voucher_count, vouchers.len());
    assert_eq!(manifest.policy_count, policies.len());
    assert_eq!(manifest.voucher_count, vouchers.len());
    assert_eq!(manifest.policies_artifact, GENESIS_POLICIES_FILE);
    assert_eq!(manifest.vouchers_artifact, GENESIS_VOUCHERS_FILE);
    assert!(
        manifest.leaf_count >= manifest.asset_count + manifest.right_count + manifest.voucher_count,
        "manifest leaf_count must cover voucher/right/asset leaves",
    );
}

#[test]
fn test_reject_codes() {
    let (policy, action_pool, action_id) = voucher_redeem_policy_contract();
    let mut registry = ObjectPolicyRegistryV1::default();
    registry
        .register(policy.clone(), action_pool.clone())
        .expect("register voucher redeem policy");

    let voucher = voucher_leaf(41);
    let package = voucher_redeem_package(&policy, &action_pool, action_id, 41, voucher.clone());
    let accepted = inspect_object_package(
        &package,
        &registry,
        package.prior_root,
        package.expected_new_root,
    );
    assert_eq!(accepted.reject, None);

    let unknown_policy = inspect_object_package(
        &package,
        &ObjectPolicyRegistryV1::default(),
        package.prior_root,
        package.expected_new_root,
    );
    assert_eq!(unknown_policy.reject, Some(ObjectRejectCode::UnknownPolicy));

    let missing_right = inspect_object_package(
        &z00z_storage::settlement::RuntimeObjectPackageV1 {
            required_rights: vec![RightWitnessRefV1 {
                right_policy: "kyc_v1".to_string(),
                witness_state: RightWitnessStateV1::Missing,
            }],
            ..package.clone()
        },
        &registry,
        package.prior_root,
        package.expected_new_root,
    );
    assert_eq!(missing_right.reject, Some(ObjectRejectCode::MissingRight));

    let forced_acceptance = inspect_object_package(
        &z00z_storage::settlement::RuntimeObjectPackageV1 {
            object_witnesses: z00z_storage::settlement::ObjectWitnessBundleV1 {
                has_acceptance_proof: false,
                ..package.object_witnesses.clone()
            },
            ..package.clone()
        },
        &registry,
        package.prior_root,
        package.expected_new_root,
    );
    assert_eq!(
        forced_acceptance.reject,
        Some(ObjectRejectCode::ForcedAcceptance)
    );

    let fee_boundary = inspect_object_package(
        &z00z_storage::settlement::RuntimeObjectPackageV1 {
            fee_support_ref: Some(bytes(90)),
            delta_set: ObjectDeltaSetV1 {
                fee_envelope: Some(fee_envelope(91, bytes(91))),
                ..package.delta_set.clone()
            },
            ..package.clone()
        },
        &registry,
        package.prior_root,
        package.expected_new_root,
    );
    assert_eq!(fee_boundary.reject, Some(ObjectRejectCode::FeeBoundary));

    let wrong_family = inspect_object_package(
        &z00z_storage::settlement::RuntimeObjectPackageV1 {
            primary_family: ObjectFamily::Asset,
            ..package.clone()
        },
        &registry,
        package.prior_root,
        package.expected_new_root,
    );
    assert_eq!(
        wrong_family.reject,
        Some(ObjectRejectCode::WrongFamilyProof)
    );

    let mut redeemed_voucher = voucher.clone();
    redeemed_voucher.lifecycle = VoucherLifecycleV1::Redeemed;
    redeemed_voucher.remaining_value = 0;
    let double_redeem_pkg =
        voucher_redeem_package(&policy, &action_pool, action_id, 42, redeemed_voucher);
    let double_redeem = inspect_object_package(
        &double_redeem_pkg,
        &registry,
        double_redeem_pkg.prior_root,
        double_redeem_pkg.expected_new_root,
    );
    assert_eq!(double_redeem.reject, Some(ObjectRejectCode::DoubleRedeem));

    let mut expired_voucher = voucher.clone();
    expired_voucher.validity.valid_until = 5;
    let expired_pkg = voucher_redeem_package(&policy, &action_pool, action_id, 43, expired_voucher);
    let expired = inspect_object_package(
        &expired_pkg,
        &registry,
        expired_pkg.prior_root,
        expired_pkg.expected_new_root,
    );
    assert_eq!(expired.reject, Some(ObjectRejectCode::ExpiredVoucherUse));

    let (issue_policy, issue_pool, issue_action_id) = voucher_issue_policy_contract();
    let mut issue_registry = ObjectPolicyRegistryV1::default();
    issue_registry
        .register(issue_policy.clone(), issue_pool.clone())
        .expect("register voucher issue policy");
    let mut unbacked_voucher = voucher_leaf(44);
    unbacked_voucher.backing = VoucherBackingRef::ReserveCommitment([0u8; 32]);
    let issue_pkg = voucher_issue_package(
        &issue_policy,
        &issue_pool,
        issue_action_id,
        44,
        unbacked_voucher,
    );
    let invalid_backing = inspect_object_package(
        &issue_pkg,
        &issue_registry,
        issue_pkg.prior_root,
        issue_pkg.expected_new_root,
    );
    assert_eq!(
        invalid_backing.reject,
        Some(ObjectRejectCode::InvalidBacking)
    );

    let (asset_policy, asset_pool, asset_action_id) = asset_wrapped_voucher_policy_contract();
    let mut asset_registry = ObjectPolicyRegistryV1::default();
    asset_registry
        .register(asset_policy.clone(), asset_pool.clone())
        .expect("register asset wrapped voucher policy");
    let asset_wrapped_pkg = voucher_redeem_package(
        &asset_policy,
        &asset_pool,
        asset_action_id,
        45,
        voucher.clone(),
    );
    let voucher_as_cash = inspect_object_package(
        &asset_wrapped_pkg,
        &asset_registry,
        asset_wrapped_pkg.prior_root,
        asset_wrapped_pkg.expected_new_root,
    );
    assert_eq!(
        voucher_as_cash.reject,
        Some(ObjectRejectCode::VoucherUsedAsCash)
    );

    let (right_policy, right_pool, right_action_id) = right_consume_policy_contract();
    let mut right_registry = ObjectPolicyRegistryV1::default();
    right_registry
        .register(right_policy.clone(), right_pool.clone())
        .expect("register right consume policy");
    let right_value_pkg = right_value_package(&right_policy, &right_pool, right_action_id, 46);
    let right_as_value = inspect_object_package(
        &right_value_pkg,
        &right_registry,
        right_value_pkg.prior_root,
        right_value_pkg.expected_new_root,
    );
    assert_eq!(
        right_as_value.reject,
        Some(ObjectRejectCode::RightUsedAsValue)
    );

    let (lock_policy, lock_pool, lock_action_id) = validator_locked_asset_policy_contract();
    let mut lock_registry = ObjectPolicyRegistryV1::default();
    lock_registry
        .register(lock_policy.clone(), lock_pool.clone())
        .expect("register validator lock policy");
    let lock_pkg = validator_locked_asset_package(
        &lock_policy,
        &lock_pool,
        lock_action_id,
        47,
        RightWitnessStateV1::Present,
        true,
        true,
    );
    let lock_leaf = lock_pkg
        .delta_set
        .deleted_objects
        .iter()
        .find_map(|delta| match delta.prior_leaf.as_ref() {
            Some(SettlementLeaf::Right(right)) => Some(*right),
            _ => None,
        })
        .expect("validator lock right");
    assert!(validator_mandate_lock_unlock_ready(&lock_leaf, 101));
    lock_leaf
        .validate_action(
            RightAction::Expire,
            z00z_storage::settlement::RightActionCtx {
                now: 101,
                ..z00z_storage::settlement::RightActionCtx::default()
            },
            None,
        )
        .expect("lock must become unlockable only after expiry");
    let lock_unlock = inspect_object_package(
        &lock_pkg,
        &lock_registry,
        lock_pkg.prior_root,
        lock_pkg.expected_new_root,
    );
    assert_eq!(lock_unlock.reject, None);

    let missing_right_delta_pkg = validator_locked_asset_package(
        &lock_policy,
        &lock_pool,
        lock_action_id,
        48,
        RightWitnessStateV1::Present,
        false,
        true,
    );
    let missing_right_delta = inspect_object_package(
        &missing_right_delta_pkg,
        &lock_registry,
        missing_right_delta_pkg.prior_root,
        missing_right_delta_pkg.expected_new_root,
    );
    assert_eq!(
        missing_right_delta.reject,
        Some(ObjectRejectCode::MissingRight)
    );

    let lock_replay_pkg = validator_locked_asset_package(
        &lock_policy,
        &lock_pool,
        lock_action_id,
        49,
        RightWitnessStateV1::Present,
        true,
        false,
    );
    let lock_replay = inspect_object_package(
        &lock_replay_pkg,
        &lock_registry,
        lock_replay_pkg.prior_root,
        lock_replay_pkg.expected_new_root,
    );
    assert_eq!(lock_replay.reject, Some(ObjectRejectCode::Replay));
}

#[test]
fn test_scenario1_object_flows_wallet_inventory_for_alice_bob_charlie() {
    let dir = tempdir().expect("tempdir");
    let identity = WalletIdentity {
        network: "p2p".to_string(),
        chain: "devnet".to_string(),
    };
    let password = SafePassword::from("Phase059Wallet!");
    let object_store = object_inventory_store();

    for (index, actor) in ["alice", "bob", "charlie"].iter().enumerate() {
        let wallet_id = PersistWalletId(format!("phase059_{actor}"));
        let wallet_path = dir.path().join(format!("{actor}.wlt"));
        create_wallet_store(
            &wallet_path,
            &wallet_id,
            &password,
            TEST_SEED_PHRASE_24,
            &identity,
            SystemRngProvider,
        )
        .expect("create wallet");

        let session =
            open_wallet_store(&wallet_path, &wallet_id, &password, &identity).expect("open wallet");

        let tag = u8::try_from(index + 1).expect("actor tag");
        let voucher = voucher_payload(wallet_id.clone(), tag);
        let right = right_payload(wallet_id.clone(), tag);
        object_store
            .put_voucher(&session, voucher.clone())
            .expect("insert voucher");
        object_store
            .put_right(&session, right.clone())
            .expect("insert right");

        let page = object_store
            .list_wallet_inventory(&session, ObjectInventoryFilter::default(), None, usize::MAX)
            .expect("list owned objects");
        assert_eq!(
            page.items.len(),
            2,
            "{actor} must keep voucher + right inventory"
        );
        assert!(page
            .items
            .iter()
            .any(|item| matches!(item.payload, OwnedObjectPayload::Voucher(_))));
        assert!(page
            .items
            .iter()
            .any(|item| matches!(item.payload, OwnedObjectPayload::Right(_))));

        let vouchers = object_store
            .list_voucher_claims(
                &session,
                Some(OwnedVoucherStatus::Redeemable),
                None,
                usize::MAX,
            )
            .expect("list vouchers");
        assert_eq!(
            vouchers.len(),
            1,
            "{actor} voucher inventory must stay non-cash"
        );
        assert_eq!(vouchers[0].terminal_id, voucher.terminal_id);

        let rights = object_store
            .list_right_inventory(&session, Some(OwnedRightStatus::Granted), None, usize::MAX)
            .expect("list rights");
        assert_eq!(rights.len(), 1, "{actor} right inventory must persist");
        assert_eq!(rights[0].terminal_id, right.terminal_id);

        let voucher_lookup = object_store
            .lookup_non_asset_object(&session, voucher.terminal_id.as_bytes())
            .expect("lookup voucher")
            .expect("voucher object present");
        assert!(matches!(
            voucher_lookup.payload,
            OwnedObjectPayload::Voucher(_)
        ));

        let right_lookup = object_store
            .lookup_non_asset_object(&session, right.terminal_id.as_bytes())
            .expect("lookup right")
            .expect("right object present");
        assert!(matches!(right_lookup.payload, OwnedObjectPayload::Right(_)));

        let redeemable_page = object_store
            .list_wallet_inventory(
                &session,
                ObjectInventoryFilter {
                    status: Some(WalletObjectStatus::Voucher(OwnedVoucherStatus::Redeemable)),
                    ..ObjectInventoryFilter::default()
                },
                None,
                usize::MAX,
            )
            .expect("list redeemable objects");
        assert_eq!(redeemable_page.items.len(), 1);
    }
}
