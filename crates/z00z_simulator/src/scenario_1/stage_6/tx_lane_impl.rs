use serde::{Deserialize, Serialize};
use std::sync::Arc;
use z00z_core::AssetWire;
use z00z_crypto::expert::encoding::to_hex;
use z00z_crypto::Z00ZScalar;
use z00z_networks_rpc::RpcTransport;
use z00z_storage::settlement::TerminalLeaf;
use z00z_storage::snapshot::{PrepFsStore, PrepSnapshotStore};
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::{path_exists, read_to_string, save_json},
};

use z00z_wallets::{
    chain::ReceiverCardRecord,
    receiver::{decode_card_compact, encode_card_compact, ReceiverCard},
    stealth::{
        kdf::{compute_leaf_ad, compute_tag16},
        zkpack::ZkPack,
    },
    tx::{
        build_public_spend_contract, build_tx_package_digest,
        verify_plaintext_balance_with_fee as core_plain_balance,
        verify_self_decrypt as core_verify_self_decrypt, verify_tx_public_spend_contract,
        OutputBundle, TxInputWire, TxPackage, TxWire,
    },
};

pub(crate) use super::build_confirm_rows;
pub(crate) use super::find_actor;
pub(crate) use super::SelectedInputRow;
use super::{
    build_canon_snapshot, build_outputs_cfg, build_pending_rows, build_prep_file, calc_fee,
    canonical_input_asset_id, check_zero_send, decode_output_pack, has_change_hint,
    list_sender_inputs_distinct_serials, load_claim_post_store, load_stage4_verified_card,
    parse_asset_class, persist_sender_state, prep_membership_witnesses, run_bob_checks,
    run_fee_checks, send_target_cfg, split_amount_cfg, to_tx_output_wires, unique_serial_count,
    verify_commitment_balance_gate, verify_fee_matches_formula,
    verify_spend_witness_gate_membership, verify_tx_package, Stage4Snap,
};
pub(crate) use super::{
    build_wallet_diff, capture_wallet_states, validate_confirm_rows, write_wallet_report_md,
    write_wallet_report_xlsx,
};
#[cfg(test)]
use super::{pick_sender_rows, prep_leaf, prep_root, prep_store, verify_spend_witness_gate};

use crate::config::{Stage4FeeSinkCfg, Stage4TxPrepareCfg};
use crate::scenario_1::claim_pkg_consumer::load_claim_packages;
use crate::scenario_1::stage_4::export_pre_tx_view;
use crate::{DesignStage, SimContext, StageResult};

use super::super::stage_2::deterministic_seed_phrase_24;
use super::super::stage_2::{actor_runtime_password, build_logged_transport};
use super::prep_ref::write_prep_ref;

pub(crate) const CLAIM_POST_VIEW_DIR: &str = "claim_post";

#[derive(Debug, Clone)]
pub(super) struct FeeParty {
    pub(super) actor: String,
    pub(super) wallet_id: String,
    pub(super) password: Option<String>,
    pub(super) card: ReceiverCard,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PrepRow {
    pub(crate) definition_id_hex: String,
    pub(crate) asset_id_hex: String,
    pub(crate) serial_id: u32,
    pub(crate) leaf: TerminalLeaf,
    pub(crate) member_wit_hex: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PrepFile {
    pub(crate) prev_root_hex: String,
    pub(crate) rows: Vec<PrepRow>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct KeysFile {
    #[serde(default)]
    pub(crate) card_compact: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct TxInputRef {
    pub(super) asset_id: [u8; 32],
    pub(super) serial_id: u32,
    pub(super) amount: u64,
}

pub(super) const PREP_FILE: &str = "checkpoint_prep.json";

pub fn run_tx_prepare(ctx: &mut SimContext, stage: &DesignStage) -> StageResult {
    let rt = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(err) => {
            return StageResult::Fail(format!("stage4: tokio runtime: {err}"));
        }
    };

    match rt.block_on(run_core(ctx, stage)) {
        Ok(()) => StageResult::Ok,
        Err(err) => StageResult::Fail(format!(
            "stage {} ({}) failed: {}",
            stage.stage, stage.name, err
        )),
    }
}

// Seam map: core_build_output_bundle(...), core_verify_self_decrypt(...),
// and core_plain_balance(...) are owned by tx_lane_runtime* helpers.
// Snapshot store seam: PrepFsStore::save_snapshot(...) is owned by tx_lane_runtime_flow.rs.

pub(crate) use super::tx_lane_runtime::{validate_fee_sink, validate_tx_mode, Stage4ResolvedPaths};

async fn run_core(ctx: &mut SimContext, stage: &DesignStage) -> Result<(), String> {
    super::tx_lane_runtime::run_core(ctx, stage).await
}
