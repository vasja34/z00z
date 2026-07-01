use std::collections::BTreeMap;
use std::sync::Arc;

use blake2::digest::consts::U32;
use blake2::{Blake2b, Digest};
use z00z_core::assets::nonce::derive_nonce;
use z00z_core::assets::{Asset, AssetClass, AssetDefinition};
use z00z_core::genesis::{
    compute_genesis_rights_digest, create_asset_definition, derive_deterministic_rng_seed,
    derive_genesis_blinding, ensure_terminal_collision_free, ChainType, GenesisRightLeaf,
    GenesisRightRecord, GenesisSettlementCorpus, GENESIS_RIGHTS_REPLAY_DIGEST_LABEL,
};
use z00z_core::rights::RightClassConfig;
use z00z_utils::codec::{Codec, JsonCodec};
use z00z_utils::rng::DeterministicRngProvider;

const FIXTURE_JSON: &str =
    include_str!("../../z00z_storage/tests/fixtures/test_settlement_corpus_fixture.json");
const FIXTURE_SHA256: [u8; 32] = [
    0x20, 0xda, 0xde, 0xfb, 0x03, 0xa3, 0x1e, 0xbd, 0xd1, 0x4a, 0x08, 0x47, 0xf0, 0x14, 0xeb, 0xac,
    0x54, 0x9a, 0x07, 0x13, 0x61, 0x0d, 0xb9, 0xa2, 0x32, 0xeb, 0x63, 0x73, 0x22, 0x4b, 0xb7, 0x6a,
];

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
struct Fixture {
    version: u32,
    network: String,
    assets: Vec<AssetSeed>,
    rights: Vec<RightSeed>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
struct AssetSeed {
    label: String,
    definition_mark: u8,
    serial_id: u32,
    terminal_mark: u8,
    value: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum FixtureRightClass {
    MachineCapability,
    DataAccess,
    ServiceEntitlement,
    ValidatorMandate,
    OneTimeUse,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
struct RightSeed {
    label: String,
    definition_mark: u8,
    serial_id: u32,
    terminal_mark: u8,
    right_class: FixtureRightClass,
}

fn load_fixture() -> Fixture {
    let digest: [u8; 32] = Blake2b::<U32>::digest(FIXTURE_JSON.as_bytes()).into();
    assert_eq!(digest, FIXTURE_SHA256, "settlement corpus fixture drifted",);
    JsonCodec
        .deserialize(FIXTURE_JSON.as_bytes())
        .expect("settlement core fixture")
}

fn bytes(value: u8) -> [u8; 32] {
    [value; 32]
}

fn right_class(class: FixtureRightClass) -> RightClassConfig {
    match class {
        FixtureRightClass::MachineCapability => RightClassConfig::MachineCapability,
        FixtureRightClass::DataAccess => RightClassConfig::DataAccess,
        FixtureRightClass::ServiceEntitlement => RightClassConfig::ServiceEntitlement,
        FixtureRightClass::ValidatorMandate => RightClassConfig::ValidatorMandate,
        FixtureRightClass::OneTimeUse => RightClassConfig::OneTimeUse,
    }
}

fn definition_for_mark(mark: u8, max_serial: u32) -> AssetDefinition {
    create_asset_definition(
        &z00z_core::genesis::genesis_config::AssetConfigEntry {
            id: format!("settlement_asset_{mark}"),
            class: AssetClass::Coin,
            name: format!("Settlement Asset {mark}"),
            symbol: format!("P{mark:02}"),
            description: None,
            domain_name: format!("settlement.{mark}.z00z"),
            policy: z00z_core::genesis::genesis_config::PolicyConfig {
                decimals: 8,
                serials: max_serial + 8,
                nominal: 10_000,
                ..Default::default()
            },
            metadata: None,
        },
        &bytes(42),
        ChainType::Devnet,
    )
    .expect("fixture definition")
}

fn build_asset(seed: &AssetSeed, definition: Arc<AssetDefinition>) -> Asset {
    let genesis_seed = bytes(42);
    let blinding = derive_genesis_blinding(
        &genesis_seed,
        &definition.id,
        seed.serial_id,
        ChainType::Devnet,
    )
    .expect("fixture blinding");
    let nonce = derive_nonce(&genesis_seed, u64::from(seed.serial_id), 0, &[0u8; 32]);
    let rng_seed = derive_deterministic_rng_seed(
        &genesis_seed,
        &definition.id,
        seed.serial_id,
        ChainType::Devnet,
    );
    let mut rng = DeterministicRngProvider::from_seed(rng_seed).rng();
    Asset::new(
        definition,
        seed.serial_id,
        seed.value,
        &blinding,
        nonce,
        &mut rng,
    )
    .expect("fixture asset")
}

fn build_right_record(index: u32, seed: &RightSeed) -> GenesisRightRecord {
    GenesisRightRecord {
        right_id: seed.label.clone(),
        right_index: index,
        definition_id: bytes(seed.definition_mark),
        serial_id: seed.serial_id,
        domain_name: format!("{}.settlement.z00z", seed.label),
        metadata_purpose: format!("settlement:{}", seed.label),
        leaf: GenesisRightLeaf {
            version: 1,
            terminal_id: bytes(seed.terminal_mark),
            right_class: right_class(seed.right_class),
            issuer_scope: bytes(seed.terminal_mark.wrapping_add(1)),
            provider_scope: bytes(seed.terminal_mark.wrapping_add(2)),
            holder_commitment: bytes(seed.terminal_mark.wrapping_add(3)),
            control_commitment: bytes(seed.terminal_mark.wrapping_add(4)),
            beneficiary_commitment: bytes(seed.terminal_mark.wrapping_add(5)),
            payload_commitment: bytes(seed.terminal_mark.wrapping_add(6)),
            valid_from: 10,
            valid_until: 20,
            challenge_from: 12,
            challenge_until: 18,
            use_nonce: bytes(seed.terminal_mark.wrapping_add(7)),
            revocation_policy_id: bytes(seed.terminal_mark.wrapping_add(8)),
            transition_policy_id: bytes(seed.terminal_mark.wrapping_add(9)),
            challenge_policy_id: bytes(seed.terminal_mark.wrapping_add(10)),
            disclosure_policy_id: bytes(seed.terminal_mark.wrapping_add(11)),
            retention_policy_id: bytes(seed.terminal_mark.wrapping_add(12)),
        },
    }
}

fn build_corpus() -> GenesisSettlementCorpus {
    let fixture = load_fixture();
    let mut corpus = GenesisSettlementCorpus::new();
    let max_serials = fixture
        .assets
        .iter()
        .fold(BTreeMap::new(), |mut acc, asset| {
            acc.entry(asset.definition_mark)
                .and_modify(|slot: &mut u32| *slot = (*slot).max(asset.serial_id))
                .or_insert(asset.serial_id);
            acc
        });
    let definitions = max_serials
        .into_iter()
        .map(|(mark, max_serial)| (mark, Arc::new(definition_for_mark(mark, max_serial))))
        .collect::<BTreeMap<_, _>>();
    for asset in &fixture.assets {
        corpus.push(
            build_asset(
                asset,
                definitions
                    .get(&asset.definition_mark)
                    .expect("fixture definition")
                    .clone(),
            ),
            AssetClass::Coin,
        );
    }
    corpus.rights = fixture
        .rights
        .iter()
        .enumerate()
        .map(|(idx, seed)| build_right_record(idx as u32, seed))
        .collect();
    corpus
}

#[test]
fn test_maps_core_genesis_contracts() {
    let fixture = load_fixture();
    let corpus = build_corpus();
    let digest_a =
        compute_genesis_rights_digest(&corpus.rights, GENESIS_RIGHTS_REPLAY_DIGEST_LABEL);
    let digest_b =
        compute_genesis_rights_digest(&build_corpus().rights, GENESIS_RIGHTS_REPLAY_DIGEST_LABEL);

    assert_eq!(fixture.version, 1);
    assert_eq!(fixture.network, "devnet");
    assert_eq!(corpus.total_count(), fixture.assets.len());
    assert_eq!(corpus.total_right_count(), fixture.rights.len());
    assert_eq!(
        corpus.total_leaf_count(),
        fixture.assets.len() + fixture.rights.len()
    );
    assert_eq!(digest_a, digest_b);
    assert!(ensure_terminal_collision_free(&corpus).is_ok());

    let def_counts = corpus
        .flatten()
        .into_iter()
        .fold(BTreeMap::new(), |mut acc, asset| {
            *acc.entry(asset.definition.id).or_insert(0usize) += 1;
            acc
        });
    assert_eq!(def_counts.len(), 3);
    assert_eq!(def_counts.values().sum::<usize>(), fixture.assets.len());
    assert!(def_counts.values().any(|count| *count == 2));
}
