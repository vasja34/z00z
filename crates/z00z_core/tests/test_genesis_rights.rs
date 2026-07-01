use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, OnceLock};

use serde_json::Value;
use z00z_core::config_paths::{core_config_path, devnet_genesis_path, DEVNET_GENESIS_CONFIG};
use z00z_core::genesis::genesis_config::{load_genesis_config, GenesisConfig};
use z00z_core::genesis::validator::compute_genesis_state_hash;
use z00z_core::genesis::{
    compute_genesis_rights_digest, create_asset_definition, ensure_terminal_collision_free,
    generate_genesis_policies, generate_genesis_settlement_corpus, ChainType, GenesisSeed,
    GenesisSettlementCorpus, GENESIS_RIGHTS_REPLAY_DIGEST_LABEL,
};
use z00z_crypto::hash::sha256_256_simple;
use z00z_utils::prelude::{NoopLogger, NoopMetrics};

const CANONICAL_RIGHTS_MANIFEST: &str = include_str!("test_genesis_rights_manifest.json");

#[derive(Clone)]
struct CorpusBuildInputs {
    config: GenesisConfig,
    definitions: Vec<z00z_core::AssetDefinition>,
    genesis_seed: [u8; 32],
    network: ChainType,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
struct CanonicalCorpusSummary {
    normalized_corpus_sha256: String,
    rights_sha256: String,
    rights_len: usize,
    state_hash: [u8; 32],
    rights_digest: [u8; 32],
}

fn canonical_manifest() -> &'static HashMap<String, CanonicalCorpusSummary> {
    static VALUE: OnceLock<HashMap<String, CanonicalCorpusSummary>> = OnceLock::new();
    VALUE.get_or_init(|| {
        serde_json::from_str(CANONICAL_RIGHTS_MANIFEST).expect("decode genesis rights manifest")
    })
}

fn canonical_genesis_path(name: &str) -> PathBuf {
    core_config_path(name)
}

fn build_inputs(name: &str) -> Result<CorpusBuildInputs, Box<dyn std::error::Error>> {
    let path = canonical_genesis_path(name);
    let config = load_genesis_config(path.to_str().expect("utf8 path"))?;
    let genesis_seed = GenesisSeed::from_config(&config)?;
    let network = ChainType::from_str(&config.chain.chain_type)?;
    let definitions = config
        .assets
        .iter()
        .map(|asset| create_asset_definition(asset, genesis_seed.as_bytes(), network))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(CorpusBuildInputs {
        config,
        definitions,
        genesis_seed: *genesis_seed.as_bytes(),
        network,
    })
}

fn build_corpus(
    inputs: &CorpusBuildInputs,
) -> Result<GenesisSettlementCorpus, Box<dyn std::error::Error>> {
    let policies = generate_genesis_policies(&inputs.config.assets, &inputs.config.policies)?;
    Ok(generate_genesis_settlement_corpus(
        &inputs.definitions,
        &inputs.config.rights,
        &inputs.config.vouchers,
        &policies,
        &inputs.genesis_seed,
        inputs.config.chain.id,
        inputs.network,
        Arc::new(NoopLogger),
        Arc::new(NoopMetrics),
    )?)
}

fn normalized_corpus_json(
    corpus: &GenesisSettlementCorpus,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut value = serde_json::to_value(corpus)?;
    for key in ["coins", "tokens", "nfts", "voids"] {
        let assets = value
            .get_mut(key)
            .and_then(Value::as_array_mut)
            .ok_or("canonical corpus must serialize class vectors as arrays")?;
        for asset in assets {
            let record = asset
                .as_object_mut()
                .ok_or("canonical corpus asset must serialize as an object")?;
            record.remove("range_proof");
            record.remove("owner_signature");
        }
    }
    Ok(value)
}

fn write_canonical_json(value: &Value, out: &mut String) -> Result<(), Box<dyn std::error::Error>> {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(flag) => out.push_str(if *flag { "true" } else { "false" }),
        Value::Number(number) => out.push_str(&serde_json::to_string(number)?),
        Value::String(text) => out.push_str(&serde_json::to_string(text)?),
        Value::Array(items) => {
            out.push('[');
            for (idx, item) in items.iter().enumerate() {
                if idx > 0 {
                    out.push(',');
                }
                write_canonical_json(item, out)?;
            }
            out.push(']');
        }
        Value::Object(map) => {
            out.push('{');
            let mut keys: Vec<_> = map.keys().collect();
            keys.sort();
            for (idx, key) in keys.iter().enumerate() {
                if idx > 0 {
                    out.push(',');
                }
                out.push_str(&serde_json::to_string(key)?);
                out.push(':');
                write_canonical_json(
                    map.get(*key)
                        .unwrap_or_else(|| panic!("canonical json key vanished: {key}")),
                    out,
                )?;
            }
            out.push('}');
        }
    }

    Ok(())
}

fn canonical_json_sha256(value: &Value) -> Result<String, Box<dyn std::error::Error>> {
    let mut canonical = String::new();
    write_canonical_json(value, &mut canonical)?;
    Ok(hex::encode(sha256_256_simple(canonical.as_bytes())))
}

fn corpus_summary(
    corpus: &GenesisSettlementCorpus,
) -> Result<CanonicalCorpusSummary, Box<dyn std::error::Error>> {
    Ok(CanonicalCorpusSummary {
        normalized_corpus_sha256: canonical_json_sha256(&normalized_corpus_json(corpus)?)?,
        rights_sha256: canonical_json_sha256(&serde_json::to_value(&corpus.rights)?)?,
        rights_len: corpus.rights.len(),
        state_hash: compute_genesis_state_hash(corpus),
        rights_digest: compute_genesis_rights_digest(
            &corpus.rights,
            GENESIS_RIGHTS_REPLAY_DIGEST_LABEL,
        ),
    })
}

#[test]
fn test_genesis_rights_deterministic() -> Result<(), Box<dyn std::error::Error>> {
    let name = DEVNET_GENESIS_CONFIG;
    let path = canonical_genesis_path(name);
    let inputs = build_inputs(name)?;
    let config = load_genesis_config(path.to_str().expect("utf8 path"))?;
    let expected = canonical_manifest()
        .get(name)
        .unwrap_or_else(|| panic!("missing canonical genesis rights manifest entry: {name}"));
    let live = build_corpus(&inputs)?;
    let live_summary = corpus_summary(&live)?;

    assert!(
        !config.rights.is_empty(),
        "{name} must keep canonical rights"
    );
    assert!(
        live.total_right_count() >= config.rights.len(),
        "{name} generated fewer rights than templates",
    );

    if &live_summary != expected {
        let retry = build_corpus(&inputs)?;
        let retry_summary = corpus_summary(&retry)?;
        assert_eq!(
            live_summary, retry_summary,
            "{name} live corpus is nondeterministic",
        );
        assert_eq!(
            retry_summary, *expected,
            "{name} canonical genesis rights snapshot drifted",
        );
    }

    assert!(
        ensure_terminal_collision_free(&live).is_ok(),
        "{name} generated terminal collisions",
    );

    Ok(())
}

#[test]
fn test_legacy_small_filename_removed() {
    let canonical = devnet_genesis_path();
    let legacy_name = ["genesis", "config", "devnet", "small.yaml"].join("_");
    let divergent_name = ["genesis", "config", "devnet-small.yaml"].join("_");
    let legacy_small =
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("src/genesis/{legacy_name}"));
    let divergent =
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("src/genesis/{divergent_name}"));

    assert!(canonical.exists(), "canonical devnet config must exist");
    assert!(
        !legacy_small.exists(),
        "legacy devnet_small config must not remain live",
    );
    assert!(
        !divergent.exists(),
        "settlement migration must not introduce a divergent hyphenated devnet_small config",
    );
}
