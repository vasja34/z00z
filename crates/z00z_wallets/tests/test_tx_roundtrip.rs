#[path = "test_inc/test_proof_blob_fix.inc"]
mod proof_blob_fix;
#[path = "test_inc/test_proof_blob_pair.inc"]
mod proof_blob_pair;

use std::collections::BTreeMap;

use z00z_core::assets::AssetLeaf;
use z00z_storage::settlement::{CheckRoot, TerminalId, TerminalLeaf};
use z00z_wallets::tx::{
    apply_batch_checkpoint, prepare_tx_sum, InputResolver, ResolvedInput, SettlementState,
    SpentIndex, SpentIndexError, StateError, TxInputWire, TxPkgSum, TxProofError, TxProofVerifier,
};

use self::proof_blob_fix::WitCase;
use self::proof_blob_pair::{wit_pair, WitPair};

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

struct MapResolver {
    root: CheckRoot,
    map: BTreeMap<TerminalId, ResolvedInput>,
}

impl InputResolver for MapResolver {
    fn resolve(
        &self,
        prev_root: CheckRoot,
        terminal_id: TerminalId,
        serial_id: u32,
    ) -> Result<ResolvedInput, StateError> {
        if prev_root != self.root {
            return Err(StateError::PrevRoot);
        }

        let input = self
            .map
            .get(&terminal_id)
            .cloned()
            .ok_or(StateError::MissingInput)?;
        if input.serial_id() != serial_id {
            return Err(StateError::LeafMatch);
        }

        Ok(input)
    }
}

struct SwapResolver {
    root: CheckRoot,
    left: WitCase,
    right: WitCase,
}

impl InputResolver for SwapResolver {
    fn resolve(
        &self,
        prev_root: CheckRoot,
        terminal_id: TerminalId,
        serial_id: u32,
    ) -> Result<ResolvedInput, StateError> {
        if prev_root != self.root || serial_id != self.left.input.serial_id() {
            return Err(StateError::LeafMatch);
        }

        if terminal_id == self.left.input.terminal_id() {
            return Ok(self.right.input.clone());
        }

        Err(StateError::MissingInput)
    }
}

fn out_leaf(asset_id: [u8; 32]) -> TerminalLeaf {
    TerminalLeaf::from(AssetLeaf {
        asset_id,
        ..AssetLeaf::default()
    })
}

fn clone_case(case: &WitCase) -> WitCase {
    WitCase {
        root: case.root,
        path: case.path,
        leaf: case.leaf.clone(),
        proof_item: case.proof_item.clone(),
        proof: case.proof.clone(),
        input: case.input.clone(),
    }
}

fn pair_refs(pair: &WitPair) -> Vec<TxInputWire> {
    vec![
        TxInputWire {
            asset_id_hex: hex::encode(pair.left.input.terminal_id().into_bytes()),
            serial_id: pair.left.input.serial_id(),
        },
        TxInputWire {
            asset_id_hex: hex::encode(pair.right.input.terminal_id().into_bytes()),
            serial_id: pair.right.input.serial_id(),
        },
    ]
}

fn one_ref(case: &WitCase) -> Vec<TxInputWire> {
    vec![TxInputWire {
        asset_id_hex: hex::encode(case.input.terminal_id().into_bytes()),
        serial_id: case.input.serial_id(),
    }]
}

fn pair_resolver(pair: &WitPair) -> MapResolver {
    let mut map = BTreeMap::new();
    map.insert(pair.left.input.terminal_id(), pair.left.input.clone());
    map.insert(pair.right.input.terminal_id(), pair.right.input.clone());
    MapResolver {
        root: pair.root,
        map,
    }
}

fn seed_state(pair: &WitPair) -> TestState {
    let mut map = BTreeMap::new();
    map.insert(
        pair.left.input.terminal_id().into_bytes(),
        pair.left.leaf.clone(),
    );
    map.insert(
        pair.right.input.terminal_id().into_bytes(),
        pair.right.leaf.clone(),
    );
    TestState {
        root: pair.root.into_bytes(),
        map,
    }
}

#[test]
fn test_stage4_roundtrip_uses_prestate() {
    let pair = wit_pair([0x71; 32], 9, [3u8; 32], [4u8; 32]);
    let refs = pair_refs(&pair);
    let tx = prepare_tx_sum(
        pair.root,
        &pair_resolver(&pair),
        &refs,
        &[out_leaf([8u8; 32]), out_leaf([9u8; 32])],
        &[1u8],
    )
    .expect("prepare tx sum");

    assert_eq!(tx.prev_root, pair.root);
    assert_eq!(tx.resolved_inputs.len(), 2);
    assert_eq!(tx.resolved_inputs[0].path(), pair.left.path);
    assert_eq!(tx.resolved_inputs[1].path(), pair.right.path);
    assert_eq!(tx.input_terminal_ids()[0], pair.left.input.terminal_id());
    assert_eq!(tx.input_terminal_ids()[1], pair.right.input.terminal_id());
    for input in &tx.resolved_inputs {
        assert_eq!(
            input.member_wit().proof_root().into_bytes(),
            pair.root.into_bytes()
        );
    }

    let mut state = seed_state(&pair);
    let cp = apply_batch_checkpoint(17, &mut state, &[tx], &OkProof, &NoSpent)
        .expect("apply checkpoint");
    assert_eq!(cp.prev_root, pair.root);
    assert_eq!(cp.spent_delta.len(), 2);
    assert_eq!(cp.created_delta.len(), 2);
}

#[test]
fn test_stage4_roundtrip_bad_compact() {
    let pair = wit_pair([0x72; 32], 9, [3u8; 32], [4u8; 32]);
    let left = clone_case(&pair.left);
    let right = clone_case(&pair.right);

    let err = prepare_tx_sum(
        pair.root,
        &SwapResolver {
            root: pair.root,
            left: clone_case(&left),
            right,
        },
        &one_ref(&left),
        &[out_leaf([8u8; 32])],
        &[1u8],
    )
    .expect_err("compact ref mismatch must not roundtrip through path-bound resolver");

    assert_eq!(err, StateError::LeafMatch);
}
