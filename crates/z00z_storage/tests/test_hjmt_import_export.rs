use serde::de::DeserializeOwned;
use tempfile::tempdir;
use z00z_core::assets::AssetLeaf;
use z00z_crypto::expert::encoding::from_hex;
use z00z_storage::settlement::{
    check_batch_contract_v1, check_checkpoint_publication_contract_v1, check_live_startup_contract,
    check_shard_root_leaf_v1, BatchProofBlobV1, CheckpointPublicationV1, DefinitionId, ProofChkErr,
    RootGeneration, SerialId, SettlementPath, SettlementRecoveryState, SettlementRouteCtx,
    SettlementStateRoot, SettlementStore, ShardRootLeafV1, StoreItem, TerminalId, TerminalLeaf,
    HJMT_PROOF_ENVELOPE_VERSION,
};
use z00z_utils::codec::{Codec, JsonCodec};

const PROOF_MANIFEST: &str =
    include_str!("fixtures/hjmt_upgrade/batch_proof_v1_positive/manifest.json");
const LEAF_MANIFEST: &str = include_str!("fixtures/hjmt_upgrade/shard_root_leaf_v1/manifest.json");
const PUB_MANIFEST: &str =
    include_str!("fixtures/hjmt_upgrade/checkpoint_publication_v1/manifest.json");

#[derive(Debug, serde::Deserialize)]
struct FixtureManifest {
    #[serde(default)]
    cases: Vec<FixtureCase>,
    #[serde(default)]
    golden: Vec<FixtureCase>,
    #[serde(default)]
    tamper: Vec<FixtureCase>,
}

#[derive(Debug, serde::Deserialize)]
struct FixtureCase {
    id: String,
    #[serde(default)]
    expected_verdict: Option<String>,
    canonical_bytes_hex: Option<String>,
}

#[test]
fn test_hjmt_route_export_roundtrip() {
    let route = SettlementRouteCtx::new([0x11; 32], 3, 14, [0x22; 32]);

    let json = JsonCodec
        .serialize_pretty(&route)
        .expect("route export json");
    let roundtrip: SettlementRouteCtx = JsonCodec.deserialize(&json).expect("route import json");

    assert_eq!(roundtrip, route);
}

#[test]
fn test_hjmt_recovery_export_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let path = asset_path(0x31);

    let mut store = SettlementStore::load(temp.path())?;
    store.put_settlement_item(StoreItem::new(path, asset_leaf(path, 7_701))?)?;
    drop(store);

    let reopened = SettlementStore::load(temp.path())?;
    let recovery = reopened.recovery_state()?;
    drop(reopened);

    let json = JsonCodec.serialize_pretty(&recovery)?;
    let roundtrip: SettlementRecoveryState = JsonCodec.deserialize(&json)?;

    assert_eq!(roundtrip, recovery);
    check_live_startup_contract("hjmt", 1, recovery.root_generation, recovery.proof_version)?;
    Ok(())
}

#[test]
fn test_route_recovery_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let recovery = sample_recovery();
    let json = JsonCodec.serialize_pretty(&recovery)?;
    let roundtrip: SettlementRecoveryState = JsonCodec.deserialize(&json)?;

    assert_eq!(roundtrip, recovery);
    assert_eq!(roundtrip.route, recovery.route);
    Ok(())
}

#[test]
fn test_hjmt_fixture_export_roundtrip() {
    let leaf_manifest: FixtureManifest = load_manifest(LEAF_MANIFEST);
    let leaf_bytes = fixture_hex(accept_case(&leaf_manifest));
    let leaf = ShardRootLeafV1::from_canon(&leaf_bytes).expect("leaf decode");
    check_shard_root_leaf_v1(&leaf).expect("leaf contract");
    assert_eq!(leaf.canonical_bytes().expect("leaf encode"), leaf_bytes);

    let pub_manifest: FixtureManifest = load_manifest(PUB_MANIFEST);
    let pub_bytes = fixture_hex(accept_case(&pub_manifest));
    let publication = CheckpointPublicationV1::from_canon(&pub_bytes).expect("publication decode");
    check_checkpoint_publication_contract_v1(&publication).expect("publication contract");
    assert_eq!(
        publication.canonical_bytes().expect("publication encode"),
        pub_bytes
    );

    let proof_manifest: FixtureManifest = load_manifest(PROOF_MANIFEST);
    let proof_bytes = fixture_hex(accept_case(&proof_manifest));
    let proof = BatchProofBlobV1::decode(&proof_bytes).expect("proof decode");
    check_batch_contract_v1(&proof).expect("proof contract");
    assert_eq!(proof.encode().expect("proof encode"), proof_bytes);
}

#[test]
fn test_hjmt_tampered_import_rejects() {
    let route = SettlementRouteCtx::new([0x31; 32], 4, 15, [0x41; 32]);
    let route_json = JsonCodec
        .serialize_pretty(&route)
        .expect("route export json");
    let mut route_value: serde_json::Value =
        serde_json::from_slice(&route_json).expect("route json value");
    route_value["extra"] = serde_json::json!(1);
    let route_bad = serde_json::to_vec(&route_value).expect("route tamper json");
    let route_err: Result<SettlementRouteCtx, _> = JsonCodec.deserialize(&route_bad);
    assert!(
        route_err.is_err(),
        "route import must reject unknown fields"
    );

    let recovery = sample_recovery();
    let recovery_json = JsonCodec
        .serialize_pretty(&recovery)
        .expect("recovery export json");
    let mut recovery_value: serde_json::Value =
        serde_json::from_slice(&recovery_json).expect("recovery json value");
    recovery_value["proof_version"] = serde_json::json!(u16::MAX);
    let recovery_bad = serde_json::to_vec(&recovery_value).expect("recovery tamper json");
    let recovery_bad: SettlementRecoveryState = JsonCodec
        .deserialize(&recovery_bad)
        .expect("recovery import json");
    let err = check_live_startup_contract(
        "hjmt",
        1,
        recovery_bad.root_generation,
        recovery_bad.proof_version,
    )
    .expect_err("wrong proof version must reject at startup");
    assert!(
        err.to_string()
            .contains("unsupported settlement proof version"),
        "{err}"
    );

    let pub_manifest: FixtureManifest = load_manifest(PUB_MANIFEST);
    let pub_bytes = fixture_hex(reject_case(&pub_manifest, "CPP-T-001"));
    let pub_err =
        CheckpointPublicationV1::from_canon(&pub_bytes).expect_err("tampered publication");
    assert_eq!(pub_err, ProofChkErr::PublicationOrderMix);

    let proof_manifest: FixtureManifest = load_manifest(PROOF_MANIFEST);
    let proof_bytes = fixture_hex(accept_case(&proof_manifest));
    let mut proof = BatchProofBlobV1::decode(&proof_bytes).expect("proof decode");
    proof.header.root_bind[0] ^= 0x01;
    let proof_err = BatchProofBlobV1::decode(&proof.encode().expect("proof encode"))
        .expect_err("tampered proof");
    assert_eq!(proof_err, ProofChkErr::BatchRootBindMix);
}

fn load_manifest<T: DeserializeOwned>(raw: &str) -> T {
    JsonCodec
        .deserialize(raw.as_bytes())
        .expect("manifest json")
}

fn accept_case(manifest: &FixtureManifest) -> &FixtureCase {
    if let Some(case) = manifest
        .cases
        .iter()
        .find(|case| case.expected_verdict.as_deref() == Some("accept"))
    {
        return case;
    }
    manifest.golden.first().expect("accept fixture case")
}

fn reject_case<'a>(manifest: &'a FixtureManifest, id: &str) -> &'a FixtureCase {
    manifest
        .tamper
        .iter()
        .find(|case| case.id == id)
        .expect("reject fixture case")
}

fn fixture_hex(case: &FixtureCase) -> Vec<u8> {
    from_hex(
        case.canonical_bytes_hex
            .as_deref()
            .expect("fixture canonical bytes"),
    )
    .expect("fixture hex")
}

fn asset_path(mark: u8) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new([mark; 32]),
        SerialId::new(u32::from(mark) + 1),
        TerminalId::new([mark.wrapping_add(1); 32]),
    )
}

fn asset_leaf(path: SettlementPath, mark: u32) -> TerminalLeaf {
    let mut core = AssetLeaf::dummy_for_scan(mark);
    core.asset_id = path.terminal_id().into_bytes();
    core.serial_id = path.serial_id.get();
    core.into()
}

fn sample_recovery() -> SettlementRecoveryState {
    SettlementRecoveryState::new(
        9,
        SettlementStateRoot::settlement_v1([0x61; 32]),
        RootGeneration::SettlementV1.version(),
        HJMT_PROOF_ENVELOPE_VERSION as u16,
        4,
        [0x71; 32],
        [0x81; 32],
    )
    .with_route(SettlementRouteCtx::new([0x91; 32], 7, 14, [0xA1; 32]))
}
