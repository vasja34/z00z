#[path = "test_inc/test_proof_blob_input_case.inc"]
mod proof_blob_input_case;
#[path = "test_inc/test_mod.rs"]
mod test_common;

use std::{collections::BTreeMap, path::PathBuf, sync::Mutex};

use test_common::managed_test_output_root;
use z00z_core::assets::AssetLeaf;
use z00z_storage::settlement::{CheckRoot, TerminalId, TerminalLeaf};
use z00z_utils::io::{create_dir_all, write_file};
use z00z_wallets::tx::{
    apply_batch_checkpoint, SettlementState, SpentIndex, SpentIndexError, StateError, TxPkgSum,
    TxProofError, TxProofVerifier,
};

use self::proof_blob_input_case::wit_input_case;

struct TestState {
    root: [u8; 32],
    map: BTreeMap<[u8; 32], TerminalLeaf>,
}

impl SettlementState for TestState {
    fn root(&self) -> CheckRoot {
        self.root.into()
    }

    fn get_leaf(&self, id: &TerminalId) -> Result<Option<TerminalLeaf>, StateError> {
        Ok(self.map.get(id.as_bytes()).cloned())
    }

    fn del_leaf(&mut self, id: &TerminalId) -> Result<(), StateError> {
        self.map.remove(id.as_bytes());
        Ok(())
    }

    fn put_leaf(&mut self, leaf: TerminalLeaf) -> Result<(), StateError> {
        self.map.insert(leaf.asset_id, leaf);
        Ok(())
    }

    fn leaf_hash(&self, leaf: &TerminalLeaf) -> Result<[u8; 32], StateError> {
        Ok(leaf.asset_id)
    }
}

fn out_leaf(asset_id: [u8; 32], serial_id: u32) -> TerminalLeaf {
    TerminalLeaf::from(AssetLeaf {
        asset_id,
        serial_id,
        ..AssetLeaf::default()
    })
}

struct OkProof;

impl TxProofVerifier for OkProof {
    fn verify_tx(&self, _tx: &TxPkgSum) -> Result<(), TxProofError> {
        Ok(())
    }
}

struct NoSpent;

impl SpentIndex for NoSpent {
    fn is_spent(
        &self,
        _prev: CheckRoot,
        _curr: CheckRoot,
        _id: &TerminalId,
    ) -> Result<bool, SpentIndexError> {
        Ok(false)
    }
}

struct SpentHit {
    id: [u8; 32],
    log: Mutex<Vec<String>>,
}

impl SpentHit {
    fn new(id: [u8; 32]) -> Self {
        Self {
            id,
            log: Mutex::new(Vec::new()),
        }
    }

    fn dump(&self) -> String {
        let rows = self.log.lock().expect("lock log");
        rows.join("\n")
    }
}

impl SpentIndex for SpentHit {
    fn is_spent(
        &self,
        prev: CheckRoot,
        curr: CheckRoot,
        id: &TerminalId,
    ) -> Result<bool, SpentIndexError> {
        let hit = id.into_bytes() == self.id;
        let mut rows = self.log.lock().expect("lock log");
        rows.push(format!(
            "prev={} curr={} id={} spent={}",
            hex::encode(prev.into_bytes()),
            hex::encode(curr.into_bytes()),
            hex::encode(id.into_bytes()),
            hit
        ));
        Ok(hit)
    }
}

fn out_dir() -> PathBuf {
    managed_test_output_root("e2e12")
}

fn snap_state(state: &TestState) -> serde_json::Value {
    let ids: Vec<String> = state.map.keys().map(hex::encode).collect();
    serde_json::json!({
        "root": hex::encode(state.root),
        "leaf_ids": ids,
        "leaf_count": state.map.len()
    })
}

fn snap_batch(tx: &TxPkgSum) -> serde_json::Value {
    let ins: Vec<String> = tx
        .input_terminal_ids()
        .iter()
        .map(|item| hex::encode(item.into_bytes()))
        .collect();
    let outs: Vec<String> = tx
        .outputs
        .iter()
        .map(|leaf| hex::encode(leaf.asset_id))
        .collect();
    serde_json::json!({
        "prev_root": hex::encode(tx.prev_root.into_bytes()),
        "inputs": ins,
        "input_paths": tx.resolved_inputs.iter().map(|item| serde_json::json!({
            "definition_id": hex::encode(item.path().definition_id.into_bytes()),
            "serial_id": item.path().serial_id.get(),
            "terminal_id": hex::encode(item.path().terminal_id().into_bytes())
        })).collect::<Vec<_>>(),
        "outputs": outs,
        "tx_proof_len": tx.tx_proof.len()
    })
}

#[test]
fn test_stage4_spent_gate() {
    if cfg!(debug_assertions) {
        return;
    }

    let in_id = [5u8; 32];
    let out_id = [6u8; 32];
    let case = wit_input_case([0x21; 32], 0, in_id);
    let root_ok = case.root.into_bytes();

    let mut state = TestState {
        root: root_ok,
        map: BTreeMap::new(),
    };
    state.map.insert(in_id, case.input.leaf().clone());

    let tx = TxPkgSum {
        prev_root: case.root,
        resolved_inputs: vec![case.input.clone()],
        outputs: vec![out_leaf(out_id, 0)],
        tx_proof: vec![1u8],
    };
    let batch = snap_batch(&tx);
    assert_eq!(
        batch["input_paths"][0]["definition_id"],
        serde_json::Value::String(hex::encode([0x21; 32])),
        "batch snapshot must retain definition_id instead of reconstructing it later"
    );
    assert_eq!(
        batch["input_paths"][0]["terminal_id"],
        serde_json::Value::String(hex::encode(in_id)),
        "batch snapshot must retain the canonical terminal id"
    );

    let mut state_ok = TestState {
        root: root_ok,
        map: BTreeMap::new(),
    };
    state_ok.map.insert(in_id, case.input.leaf().clone());
    let cp = apply_batch_checkpoint(12, &mut state_ok, &[tx.clone()], &OkProof, &NoSpent)
        .expect("non-spent path must pass");
    assert_eq!(cp.prev_root, root_ok.into(), "checkpoint must keep root");
    assert_eq!(cp.spent_delta.len(), 1, "one input must be consumed");
    assert_eq!(
        cp.spent_delta[0].terminal_id,
        TerminalId::new(in_id),
        "spent delta must keep canonical terminal id"
    );
    assert_eq!(cp.created_delta.len(), 1, "one output must be created");

    let spent = SpentHit::new(in_id);
    let before = snap_state(&state);
    let err = apply_batch_checkpoint(12, &mut state, &[tx.clone()], &OkProof, &spent)
        .expect_err("spent candidate must reject");
    assert_eq!(
        err,
        StateError::SpentAfter,
        "must reject with spent-path class"
    );

    let after = snap_state(&state);
    assert_eq!(before, after, "state must not mutate on spent-path reject");
    assert!(state.map.contains_key(&in_id), "input must stay in state");
    assert!(
        !state.map.contains_key(&out_id),
        "output must not be inserted on reject"
    );

    create_dir_all(out_dir()).expect("mkdir outputs/tests/e2e12");
    let tx_bytes = serde_json::to_vec_pretty(&snap_batch(&tx)).expect("json batch");
    write_file(out_dir().join("batch_spent.json"), &tx_bytes).expect("write batch");

    let before_bytes = serde_json::to_vec_pretty(&before).expect("json before");
    write_file(out_dir().join("state_before.json"), &before_bytes).expect("write before");
    let after_bytes = serde_json::to_vec_pretty(&after).expect("json after");
    write_file(out_dir().join("state_after.json"), &after_bytes).expect("write after");

    let spent_log = spent.dump();
    assert!(
        spent_log.contains(&format!("prev={}", hex::encode(root_ok))),
        "spent log must include expected prev root"
    );
    assert!(
        spent_log.contains(&format!("curr={}", hex::encode(root_ok))),
        "spent log must include expected current root"
    );
    assert!(
        spent_log.contains(&format!("id={}", hex::encode(in_id))),
        "spent log must include consumed id"
    );
    assert!(
        spent_log.contains("spent=true"),
        "spent log must record hit=true"
    );
    write_file(out_dir().join("spent_index_log.txt"), spent_log.as_bytes())
        .expect("write spent log");

    let mut out = String::from("E2E-12 state mutation\n");
    out.push_str(&format!("error={err}\n"));
    out.push_str(&format!(
        "input_present={}\n",
        state.map.contains_key(&in_id)
    ));
    out.push_str(&format!(
        "output_present={}\n",
        state.map.contains_key(&out_id)
    ));
    write_file(out_dir().join("state_mutation_log.txt"), out.as_bytes()).expect("write state log");
}
