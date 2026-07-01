#![allow(unused_imports)]

pub(super) use crate::scenario_1::claim_pkg_consumer::{
    build_claim_store_ops, load_claim_packages,
};
pub(super) use crate::ScenarioCfg;
pub(super) use crate::{
    config::{Stage4FeeSinkCfg, Stage4SelectionCfg, Stage4TxPrepareCfg},
    DesignStage, SimContext, StageResult,
};
pub(super) use rust_xlsxwriter::{Workbook, XlsxError};
pub(super) use serde::{Deserialize, Serialize};
pub(super) use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::Arc,
};
pub(super) use z00z_core::{
    assets::{AssetClass, AssetPackPlain, AssetPkgWire},
    AssetWire,
};
pub(super) use z00z_crypto::expert::encoding::to_hex;
pub(super) use z00z_crypto::{
    domains::Stage4OutSeedDomain, hash_zk::hash_zk, protocol::ecdh::derive_dh_key, Hidden,
    Z00ZCommitment, Z00ZScalar,
};
pub(super) use z00z_networks_rpc::RpcTransport;
pub(super) use z00z_storage::{
    settlement::{
        chk_blob_settlement, CheckRoot, DefinitionId, SerialId, SettlementPath,
        SettlementStateRoot, SettlementStore, SnapItem, StoreItem, StoreOp, TerminalId,
        TerminalLeaf,
    },
    snapshot::{
        build_snapshot, PrepFsStore, PrepSnapshot, PrepSnapshotError, PrepSnapshotId,
        PrepSnapshotStore,
    },
};
pub(super) use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::{create_dir_all, path_exists, read_file, read_to_string, save_json, write_file},
    rng::{DeterministicRngProvider, SystemRngProvider},
    time::{format_system_time_local, SystemTimeProvider, TimeProvider},
};
pub(super) use z00z_wallets::{
    chain::ReceiverCardRecord,
    domains::hashing::compute_wallet_file_id,
    key::{ReceiverKeys, ReceiverSecret},
    persistence::{TxRecord, TxStatus, TxStorage, TxStorageImpl},
    receiver::{check_stealth_own, decode_card_compact, encode_card_compact, ReceiverCard},
    rpc::types::common::PersistWalletId,
    stealth::{
        bind_stealth_output_wire, build_output_bundle as core_build_output_bundle,
        build_output_bundle_with_rng as core_build_output_bundle_with_rng,
        build_stealth_leaf_with_rng,
    },
    stealth::{
        ecdh::sender_derive_dh_with_r,
        kdf::{compute_leaf_ad, compute_tag16, derive_s_out},
        zkpack::ZkPack,
    },
    tx::{
        build_confirm_rows as core_build_confirm, build_pending_rows as core_build_pending,
        build_public_spend_contract, build_tx_package_digest,
        derive_tx_output_nonce as core_output_nonce, pick_input_rows, pick_output_serials,
        prepare_spend_public_inputs, split_output_amounts,
        validate_confirm_rows as core_validate_confirm,
        verify_plaintext_balance_with_fee as core_plain_balance,
        verify_self_decrypt as core_verify_self_decrypt,
        verify_spend_witness_gate as core_witness_gate, verify_tx_public_spend_contract,
        AssetSelCfg, BobOutCfg, ClaimTxPackage, ConfirmEnt as CoreConfirm, OutputBundle,
        PendingEnt as CorePending, TxAssemblerImpl, TxInputWire, TxOutputWire, TxPackage,
        TxVerifier, TxVerifierImpl, TxWire,
    },
};

pub(super) use super::stage_2::deterministic_seed_phrase_24;
pub(super) use super::stage_2::{actor_runtime_password, build_logged_transport};
pub(super) use super::*;

mod bob_flow;
mod input_selection_scan;
mod output_construction;
mod output_construction_balance;
pub(crate) mod paths;
mod persistence;
pub(super) mod prep_ref;
pub(crate) mod reporting;
mod reports;
mod reports_capture;
mod reports_diff;
mod reports_md;
mod reports_rows;
mod reports_xlsx;
pub mod shared_cases;
pub mod sim_pkg_support;
#[cfg(test)]
mod test_tx_lane_runtime_suite;
mod tx_lane_impl;
mod tx_lane_runtime;
mod tx_lane_runtime_flow;
#[cfg(test)]
mod tx_lane_runtime_suite_support;
mod tx_lane_runtime_support;
mod tx_preparation_core;
pub(crate) mod tx_validation_gates;
pub mod verifier_support;
mod wallet_state_capture;

pub(super) use bob_flow::{run_bob_checks, run_fee_checks};
#[cfg(test)]
pub(super) use input_selection_scan::pick_sender_rows;
pub(super) use input_selection_scan::{
    canonical_input_asset_id, distinct_serial_target, extract_next_cursor,
    list_sender_inputs_distinct_serials, parse_list_settlement_rows, unique_serial_count,
};
#[cfg(test)]
pub(crate) use output_construction::make_output_with_blind;
pub(super) use output_construction::{
    build_outputs_cfg, check_zero_send, has_change_hint, send_target_cfg, split_amount_cfg,
};
pub(crate) use paths::{find_actor, resolve_stage4_paths};
pub(super) use persistence::persist_sender_state;
pub(super) use reporting::{flush_logs, push_log, Stage4Snap};
pub(crate) use reports::{
    build_confirm_rows, build_pending_rows, build_pending_rows_for_assets, build_wallet_diff,
    capture_wallet_actor, capture_wallet_states, merge_wallet_diff_dump, merge_wallet_state_dump,
    validate_confirm_rows, wallet_amount_total, write_wallet_report_md, write_wallet_report_xlsx,
};
use reports::{build_tx_econ, TxEcon};
pub use tx_lane_impl::run_tx_prepare;
use tx_lane_impl::{FeeParty, TxInputRef, PREP_FILE};
pub(crate) use tx_lane_impl::{KeysFile, PrepFile, PrepRow, CLAIM_POST_VIEW_DIR};
pub(crate) use tx_lane_runtime::{validate_fee_sink, validate_tx_mode, Stage4ResolvedPaths};
use tx_lane_runtime_support::{
    apply_root_tamper, apply_test_tamper, apply_wit_tamper, fee_capture, resolve_fee_party,
};
pub(super) use tx_preparation_core::{
    build_canon_snapshot, build_prep_file, load_claim_post_store, prep_membership_witnesses,
};
#[cfg(test)]
pub(super) use tx_preparation_core::{prep_leaf, prep_root, prep_store};
#[cfg(test)]
pub(super) use tx_validation_gates::verify_spend_witness_gate;
pub(super) use tx_validation_gates::{
    calc_fee, decode_output_pack, load_stage4_verified_card, parse_asset_class, stage4_cfg,
    stage4_flags_summary, to_tx_output_wires, validate_stage4_cfg, verify_commitment_balance_gate,
    verify_fee_matches_formula, verify_spend_witness_gate_membership, verify_tx_package,
};
pub(super) use wallet_state_capture::SenderPersist;
pub(crate) use wallet_state_capture::{
    ConfirmRow, PendingRow, SelectedInputRow, SerialDistRow, WalletDiffDump, WalletDiffRow,
    WalletItemRow, WalletStateDump, WalletStateRow,
};
