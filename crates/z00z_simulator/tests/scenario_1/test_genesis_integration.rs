use std::{collections::HashSet, path::Path, path::PathBuf, sync::OnceLock};

use z00z_core::assets::{Asset, AssetClass};
use z00z_simulator::config::ScenarioCfg;
use z00z_utils::{
    codec::{Codec, YamlCodec},
    io::{load_bincode, write_file},
};

use z00z_simulator::scenario_1::support::fixture_cache;
use z00z_simulator::scenario_1::support::stage_runner_support;

static S1_OUT: OnceLock<PathBuf> = OnceLock::new();

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

fn stage1_out() -> &'static PathBuf {
    S1_OUT.get_or_init(|| {
        let root = fixture_cache::ensure_case("genesis_integration_stage1_v1", |base| {
            let (cfg_path, design_path, out) = test_cfg_paths_in(base);
            let _ctx = stage_runner_support::run_stage_setup(&cfg_path, &design_path, &[1_u32]);
            assert!(out.join("genesis").exists());
        });
        root.join("outputs/scenario_1")
    })
}

fn load_all_genesis_bins(out: &Path) -> Vec<Asset> {
    list_gen_bins(out)
        .into_iter()
        .flat_map(|path| load_bincode::<Vec<Asset>>(&path).expect("load bin"))
        .collect()
}

fn list_gen_bins(out: &Path) -> Vec<PathBuf> {
    let base = out.join("genesis");
    let mut files = Vec::new();
    let entries = std::fs::read_dir(&base).expect("read genesis dir");

    for entry in entries {
        let path = entry.expect("read dir entry").path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|item| item.to_str()) else {
            continue;
        };
        if name.starts_with("genesis_") && name.ends_with(".bin") {
            files.push(path);
        }
    }

    files.sort();
    assert!(!files.is_empty(), "no genesis_*.bin files found");
    files
}

#[test]
fn test_stage1_bin_class_purity() {
    for bin_path in list_gen_bins(stage1_out()) {
        let file = bin_path
            .file_name()
            .and_then(|item| item.to_str())
            .expect("bin file name");
        let assets: Vec<Asset> = load_bincode(&bin_path).expect("load genesis bin");
        assert!(!assets.is_empty(), "{file} bin must not be empty");
        let expected = assets[0].definition.class;
        for item in &assets {
            assert_eq!(
                item.definition.class, expected,
                "{file}: unexpected class {:?}",
                item.definition.class
            );
        }
    }
}

#[test]
fn test_stage1_extractor_output_valid() {
    let out_gen = stage1_out().join("genesis");
    let path = out_gen.join("assets_extract_coins_0_99.bin");
    assert!(path.exists(), "extractor bin not found: {}", path.display());

    let assets: Vec<Asset> = load_bincode(&path).expect("load extractor output");
    assert!(!assets.is_empty(), "extractor output must not be empty");

    for asset in &assets {
        assert_eq!(
            asset.definition.class,
            AssetClass::Coin,
            "non-Coin in extractor output"
        );
        assert!(
            asset.serial_id <= 99,
            "serial_id {} out of range [0,99]",
            asset.serial_id
        );
    }
}

#[test]
fn test_stage1_nonces_unique() {
    let assets = load_all_genesis_bins(stage1_out());
    assert!(!assets.is_empty(), "genesis bins are empty");

    let mut seen: HashSet<[u8; 32]> = HashSet::new();
    for asset in &assets {
        let nonce_bytes: [u8; 32] = asset.nonce;
        assert!(
            seen.insert(nonce_bytes),
            "nonce collision at serial_id={}",
            asset.serial_id
        );
    }
}

#[test]
fn test_stage1_serial_range_complete() {
    for bin_path in list_gen_bins(stage1_out()) {
        let file = bin_path
            .file_name()
            .and_then(|item| item.to_str())
            .expect("bin file name")
            .to_string();
        let assets: Vec<Asset> = load_bincode(&bin_path).expect("load genesis file");
        assert!(!assets.is_empty(), "{file}: bin is empty");

        let def_symbol = assets[0].definition.symbol.clone();
        assert_eq!(
            file,
            format!("genesis_{def_symbol}.bin"),
            "{def_symbol}: unexpected bin file name"
        );

        let mut serials: Vec<u32> = assets.iter().map(|asset| asset.serial_id).collect();
        serials.sort_unstable();
        let original_len = serials.len();
        serials.dedup();

        assert_eq!(
            serials.len(),
            original_len,
            "{def_symbol}: duplicate serial IDs"
        );
        assert_eq!(
            *serials.first().expect("first serial"),
            0,
            "{def_symbol}: serials don't start at 0"
        );
        assert_eq!(
            *serials.last().expect("last serial"),
            (original_len - 1) as u32,
            "{def_symbol}: serial gap detected"
        );
    }
}

#[test]
fn test_stage1_analyzer_report_present() {
    let path = stage1_out()
        .join("genesis")
        .join("genesis_analysis_genesis.md");
    assert!(path.exists(), "analyzer report missing: {}", path.display());

    let content = std::fs::read_to_string(&path).expect("failed to read analyzer report");
    assert!(
        content.len() > 500,
        "analyzer report too short: {} bytes",
        content.len()
    );
    assert!(
        content.contains('|'),
        "analyzer report has no table rows (no '|' character)"
    );
}

#[test]
fn test_stage1_logger_all_steps() {
    let path = stage1_out().join("logs/logger.json");
    assert!(path.exists(), "logger.json not found");
    let content = std::fs::read_to_string(&path).expect("read logger.json");

    let steps = [
        "S1-1", "S1-2", "S1-3", "S1-4", "S1-5", "S1-6", "S1-7", "S1-8",
    ];
    for step in steps {
        assert!(content.contains(step), "missing log entry for {step}");
    }

    for step in steps {
        let mut has_ok = false;
        for line in content.lines().filter(|line| !line.trim().is_empty()) {
            let value: serde_json::Value =
                serde_json::from_str(line).expect("invalid JSON in logger.json");
            let value_step = value["step"].as_str().unwrap_or_default();
            if value_step != step {
                continue;
            }

            let status = value["status"].as_str().unwrap_or_default();
            if status == "fail" {
                panic!("stage 1 step {step} has fail entry: {line}");
            }
            if status == "ok" {
                has_ok = true;
            }
        }
        assert!(has_ok, "stage 1 step {step} has no ok entry");
    }
}
