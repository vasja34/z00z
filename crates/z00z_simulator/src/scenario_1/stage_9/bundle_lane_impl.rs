use serde::{Deserialize, Serialize};
#[cfg(test)]
use std::collections::BTreeMap;
use std::collections::{BTreeSet, HashSet};
use std::path::Path;
use z00z_core::AssetWire;
use z00z_crypto::expert::encoding::to_hex;
#[cfg(test)]
use z00z_crypto::frame_str;
#[cfg(test)]
use z00z_crypto::{domains::TxDigestDomain, frame_u32_le, hash_zk::hash_zk};
use z00z_storage::checkpoint::{
    CheckpointExecInput, CheckpointInRef, SpentIndex, SpentIndexError, TxPkgSum, TxProofError,
    TxProofVerifier,
};
use z00z_storage::settlement::{CheckRoot, SerialId, TerminalId, TerminalLeaf as StorAssetLeaf};
#[cfg(test)]
use z00z_storage::snapshot::PrepReplayEntry;
use z00z_storage::snapshot::{PrepSnapshot, PrepSnapshotId};
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::{create_dir_all, read_file, read_to_string, save_json},
};
#[cfg(test)]
use z00z_wallets::tx::{InputResolver, MemberWit, ResolvedInput, StateError};
use z00z_wallets::{
    receiver::ReceiverCard,
    tx::{asset_wire_to_leaf, verify_tx_public_spend_contract, TxOutputWire, TxPackage},
};

use super::bridge_output_router::build_bridge_out;
use super::fragment_construction::{build_target_frag, decode_hex32};
use super::prep_snapshot_loader::{load_prep, resolve_input_path};

#[derive(Serialize, Deserialize)]
pub(crate) struct Stage9Bridge {
    pub(crate) stage: u32,
    pub(crate) prev_root_hex: String,
    pub(crate) exec_input_id_hex: String,
    pub(crate) fragment_ids: Vec<String>,
    pub(crate) bridge_outputs: Vec<TxOutputWire>,
    pub(crate) status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FragIn {
    // Wire encoding of the canonical state_key for the consumed pre-state leaf.
    pub(crate) asset_id_hex: String,
    // Input_ref consistency field. This is not part of the spent state_key.
    pub(crate) serial_id: u32,
    pub(crate) prev_root_hex: String,
    // Resolved pre-state leaf hash used only for audit and proof/state linkage.
    pub(crate) leaf_hash_hex: String,
    pub(crate) member_ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FragOut {
    pub(crate) asset_id_hex: String,
    pub(crate) leaf_hash_hex: String,
    pub(crate) amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FragTx {
    pub(crate) id: String,
    pub(crate) prev_root_hex: String,
    pub(crate) inputs: Vec<FragIn>,
    pub(crate) outputs: Vec<FragOut>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MadeEnt {
    pub(crate) asset_id_hex: String,
    pub(crate) leaf_hash_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Checkpoint {
    pub(crate) prev_root_hex: String,
    pub(crate) new_root_hex: String,
    pub(crate) spent_delta: Vec<String>,
    // Typed audit artifacts derived from the executed state transition.
    pub(crate) created_delta: Vec<MadeEnt>,
    pub(crate) fragment_ids: Vec<String>,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DemoCheckpoint {
    pub(crate) prev_root_hex: String,
    // Demo digest only. This is not a state root and must not be used as one.
    pub(crate) demo_digest_hex: String,
    pub(crate) spent_delta: Vec<String>,
    pub(crate) created_delta: Vec<MadeEnt>,
    pub(crate) fragment_ids: Vec<String>,
}

#[cfg(test)]
#[derive(Debug)]
pub(crate) struct PrepIdx {
    map: BTreeMap<RootKey, ResolvedInput>,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct RootKey {
    root: CheckRoot,
    terminal_id: TerminalId,
}

pub(crate) struct CheckpointReplaySpentIndex {
    prev_root: CheckRoot,
    input_terminal_ids: BTreeSet<TerminalId>,
}

pub(crate) struct CheckpointPackageProofVerifier {
    expected_prev_root: CheckRoot,
    tx_proof: Vec<u8>,
    input_refs: Vec<CheckpointInRef>,
    outputs: Vec<StorAssetLeaf>,
}

impl CheckpointReplaySpentIndex {
    pub(crate) fn from_exec(exec: &CheckpointExecInput) -> Result<Self, String> {
        let [tx] = exec.txs() else {
            return Err("stage7: expected exactly one checkpoint exec tx".to_string());
        };
        let input_terminal_ids = tx
            .input_refs()
            .iter()
            .map(|item| item.terminal_id())
            .collect::<BTreeSet<_>>();
        if input_terminal_ids.is_empty() {
            return Err("stage7: checkpoint exec tx must keep at least one input ref".to_string());
        }

        Ok(Self {
            prev_root: exec.prev_root(),
            input_terminal_ids,
        })
    }
}

impl CheckpointPackageProofVerifier {
    pub(crate) fn expected_prev_root(pkg: &TxPackage) -> Result<CheckRoot, String> {
        let spend = pkg
            .tx
            .proof
            .spend
            .as_ref()
            .ok_or_else(|| "stage7: missing stage4 spend proof".to_string())?;
        Ok(CheckRoot::new(decode_hex32(&spend.prev_root_hex)?))
    }

    pub(crate) fn from_stage11(pkg: &TxPackage, outputs: &[TxOutputWire]) -> Result<Self, String> {
        let expected_prev_root = Self::expected_prev_root(pkg)?;
        let tx_proof = JsonCodec
            .serialize(&pkg.tx.proof)
            .map_err(|e| format!("stage7: tx proof encode failed: {e}"))?;
        let input_refs = pkg
            .tx
            .inputs
            .iter()
            .map(|input| {
                let asset_id = decode_hex32(&input.asset_id_hex)?;
                Ok(CheckpointInRef::new(
                    asset_id,
                    SerialId::new(input.serial_id),
                ))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let outputs = outputs
            .iter()
            .map(|output| {
                let asset = output
                    .asset_wire
                    .clone()
                    .to_asset()
                    .map_err(|e| format!("stage7: output asset decode failed: {e}"))?;
                asset_wire_to_leaf(&AssetWire::from_asset(&asset))
                    .map_err(|e| format!("stage7: output leaf build failed: {e}"))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            expected_prev_root,
            tx_proof,
            input_refs,
            outputs,
        })
    }

    pub(crate) fn verify_pkg_contract(pkg: &TxPackage) -> Result<(), String> {
        verify_tx_public_spend_contract(
            pkg.chain_id,
            pkg.version,
            &pkg.chain_type,
            &pkg.chain_name,
            &pkg.tx,
        )
        .map_err(|e| format!("stage7: current-stack tx public spend verifier failed: {e}"))
    }
}

#[cfg(test)]
pub(crate) fn demo_spent_key(input: &FragIn) -> String {
    // Placeholder only for the current demo flow. The spent_delta already uses
    // the canonical state_key, while serial_id stays on the input_ref only for
    // leaf-match consistency.
    input.asset_id_hex.clone()
}

pub(crate) struct Stage9Load {
    pub(crate) pkg: TxPackage,
    pub(crate) snap_id: PrepSnapshotId,
    pub(crate) prep: PrepSnapshot,
    pub(crate) prev_root_hex: String,
    pub(crate) frag_a: FragTx,
    pub(crate) frag_b: FragTx,
    pub(crate) bridge_outputs: Vec<TxOutputWire>,
}

pub(crate) fn prep_dirs(out: &Path, logs_dir: &Path, tx_dir: &Path) -> Result<(), String> {
    create_dir_all(out).map_err(|e| e.to_string())?;
    create_dir_all(logs_dir).map_err(|e| e.to_string())?;
    create_dir_all(tx_dir).map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) fn load_stage9(
    out: &Path,
    tx_dir: &Path,
    s4: &crate::config::Stage4TxPrepareCfg,
    charlie: &ReceiverCard,
) -> Result<Stage9Load, String> {
    let s4_tx_path = resolve_input_path(out, &s4.paths.tx_pkg_file)?;
    let s4_prep_path = s4_tx_path
        .parent()
        .unwrap_or(tx_dir)
        .join("checkpoint_prep.json");
    let pkg = load_tx_pkg(&s4_tx_path)?;
    ensure_tx_parts(&pkg)?;
    let (snap_id, prep, _) = load_prep(&s4_prep_path)?;
    let prev_root_hex = to_hex(prep.prev_root.as_bytes());
    let bridge_a = build_bridge_out(0, &pkg.tx.outputs[0], charlie)?;
    let bridge_b = build_bridge_out(1, &pkg.tx.outputs[1], charlie)?;
    let frag_a = build_target_frag(
        1,
        &prev_root_hex,
        &prep.entries,
        &pkg.tx.inputs[0],
        &bridge_a,
    )?;
    let frag_b = build_target_frag(
        2,
        &prev_root_hex,
        &prep.entries,
        &pkg.tx.inputs[1],
        &bridge_b,
    )?;

    Ok(Stage9Load {
        pkg,
        snap_id,
        prep,
        prev_root_hex,
        frag_a,
        frag_b,
        bridge_outputs: vec![bridge_a, bridge_b],
    })
}

pub(crate) fn load_tx_pkg(path: &Path) -> Result<TxPackage, String> {
    let raw = read_file(path)
        .map_err(|e| format!("failed reading stage4 tx package {}: {e}", path.display()))?;
    let verdict = z00z_wallets::tx::verify_full_tx_package(raw.as_slice())
        .map_err(|e| format!("stage4 tx package verification failed: {e}"))?;
    if !verdict.valid {
        return Err(format!(
            "stage4 tx package verification failed: {}",
            verdict.errors.join("; ")
        ));
    }
    let pkg: TxPackage = JsonCodec
        .deserialize(raw.as_slice())
        .map_err(|e| format!("invalid stage4 tx package decode: {e}"))?;
    Ok(pkg)
}

pub(crate) fn load_stage9_bridge(path: &Path) -> Result<Stage9Bridge, String> {
    let text = read_to_string(path)
        .map_err(|e| format!("failed reading stage6 bridge {}: {e}", path.display()))?;
    JsonCodec
        .deserialize(text.as_bytes())
        .map_err(|e| format!("invalid stage6 bridge decode: {e}"))
}

pub(crate) fn load_frag(path: &Path, name: &str) -> Result<FragTx, String> {
    JsonCodec
        .deserialize(read_file(path).map_err(|e| e.to_string())?.as_slice())
        .map_err(|e| format!("invalid {name} decode: {e}"))
}

pub(crate) fn frag_amount_sum(frag_a: &FragTx, frag_b: &FragTx) -> u64 {
    frag_a
        .outputs
        .iter()
        .chain(frag_b.outputs.iter())
        .map(|output| output.amount)
        .sum()
}

pub(crate) fn write_step_fallbacks(
    stage: &crate::DesignStage,
    covered: &HashSet<String>,
    lines: &mut Vec<String>,
) -> Result<(), String> {
    let mut missing = stage
        .steps
        .iter()
        .filter(|step| !covered.contains(&step.id))
        .map(|step| step.id.clone())
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        missing.sort();
        return Err(format!(
            "stage {} missing canonical coverage for steps: {}",
            stage.stage,
            missing.join(", ")
        ));
    }
    let _ = lines;
    Ok(())
}

pub(crate) fn ensure_tx_parts(pkg: &TxPackage) -> Result<(), String> {
    if pkg.tx.inputs.len() < 2 {
        return Err("stage4 tx package must contain at least 2 inputs".to_string());
    }
    if pkg.tx.outputs.len() < 2 {
        return Err("stage4 tx package must contain at least 2 outputs".to_string());
    }
    Ok(())
}

pub(crate) fn save_frags(
    tx_dir: &Path,
    p: &crate::config::Stage6PathsCfg,
    data: &Stage9Load,
) -> Result<(), String> {
    save_json(tx_dir.join(&p.frag1_file), &data.frag_a).map_err(|e| e.to_string())?;
    save_json(tx_dir.join(&p.frag2_file), &data.frag_b).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
pub(crate) fn calc_prev_root(stage: u32) -> [u8; 32] {
    hash_zk::<TxDigestDomain>(
        "S6/PREV_ROOT",
        &[&frame_u32_le(stage), &frame_str("scenario_1")],
    )
}

#[cfg(test)]
impl PrepIdx {
    pub(crate) fn new(prev_root: CheckRoot, entries: &[PrepReplayEntry]) -> Result<Self, String> {
        let mut map = BTreeMap::new();
        for entry in entries {
            let item = entry.item();
            let path = item.path();
            let resolved = ResolvedInput::new(
                path,
                item.leaf()
                    .as_terminal()
                    .ok_or_else(|| "prep replay entry carried non-asset leaf".to_string())?
                    .clone(),
                MemberWit::new(item.wit().to_vec(), entry.proof_item().clone())
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            map.insert(
                RootKey {
                    root: prev_root,
                    terminal_id: path.terminal_id(),
                },
                resolved,
            );
        }
        Ok(Self { map })
    }
}

#[cfg(test)]
impl InputResolver for PrepIdx {
    fn resolve(
        &self,
        prev_root: CheckRoot,
        terminal_id: TerminalId,
        serial_id: u32,
    ) -> Result<ResolvedInput, StateError> {
        let key = RootKey {
            root: prev_root,
            terminal_id,
        };
        let resolved = match self.map.get(&key).cloned() {
            Some(resolved) => resolved,
            None if self.map.keys().any(|item| item.terminal_id == terminal_id) => {
                return Err(StateError::PrevRoot)
            }
            None => return Err(StateError::MissingInput),
        };
        if resolved.serial_id() != serial_id {
            return Err(StateError::LeafMatch);
        }

        Ok(resolved)
    }
}

impl SpentIndex for CheckpointReplaySpentIndex {
    fn is_spent(
        &self,
        prev: CheckRoot,
        curr: CheckRoot,
        id: &TerminalId,
    ) -> Result<bool, SpentIndexError> {
        if prev != self.prev_root || curr != self.prev_root {
            return Err(SpentIndexError::Lookup);
        }
        if !self.input_terminal_ids.contains(id) {
            return Err(SpentIndexError::Lookup);
        }
        Ok(false)
    }
}

impl TxProofVerifier for CheckpointPackageProofVerifier {
    fn verify_tx(&self, tx: &TxPkgSum) -> Result<(), TxProofError> {
        // This is a current-stack package-coupled verifier: it proves that the
        // checkpoint draft is bound to the persisted tx package contract, not
        // that a stronger standalone checkpoint proof backend was validated.
        // Later exec artifacts must inherit this accepted package path rather
        // than become standalone authorization carriers.
        if tx.prev_root != self.expected_prev_root {
            return Err(TxProofError::Invalid);
        }
        if tx.tx_proof != self.tx_proof {
            return Err(TxProofError::Invalid);
        }
        if tx.resolved_inputs.len() != self.input_refs.len() || tx.outputs != self.outputs {
            return Err(TxProofError::Invalid);
        }
        for (resolved, expected) in tx.resolved_inputs.iter().zip(self.input_refs.iter()) {
            if resolved.terminal_id().into_bytes() != expected.terminal_id().into_bytes()
                || resolved.serial_id() != expected.serial_id().get()
            {
                return Err(TxProofError::Invalid);
            }
        }
        Ok(())
    }
}
