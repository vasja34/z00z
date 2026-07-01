use std::collections::BTreeMap;

use super::{
    apply_batch_checkpoint, prepare_tx_sum, resolve_inputs, CheckpointPubIn, InputResolver,
    MemberWit, ResolvedInput, SettlementState, SpentEnt, SpentIndex, SpentIndexError, StateError,
    TxPkgSum, TxProofError, TxProofVerifier,
};
use z00z_core::assets::AssetLeaf;
use z00z_storage::settlement::{
    CheckRoot, DefinitionId, ProofItem, SerialId as StorSerialId, SettlementPath,
    SettlementStateRoot, SettlementStore, StoreItem, TerminalId, TerminalLeaf,
};
use z00z_utils::codec::{Codec, JsonCodec};

use crate::tx::TxInputWire;

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

struct EmptySpent;

impl SpentIndex for EmptySpent {
    fn is_spent(
        &self,
        _prev: CheckRoot,
        _curr: CheckRoot,
        _id: &TerminalId,
    ) -> Result<bool, SpentIndexError> {
        Ok(false)
    }
}

struct TestResolver {
    root: CheckRoot,
    map: BTreeMap<TerminalId, ResolvedInput>,
}

impl InputResolver for TestResolver {
    fn resolve(
        &self,
        prev_root: CheckRoot,
        terminal_id: TerminalId,
        serial_id: u32,
    ) -> Result<ResolvedInput, StateError> {
        if prev_root != self.root {
            return Err(StateError::PrevRoot);
        }

        let resolved = self
            .map
            .get(&terminal_id)
            .cloned()
            .ok_or(StateError::MissingInput)?;
        if resolved.serial_id() != serial_id {
            return Err(StateError::LeafMatch);
        }

        Ok(resolved)
    }
}

struct NoMember;

impl InputResolver for NoMember {
    fn resolve(
        &self,
        _prev_root: CheckRoot,
        _terminal_id: TerminalId,
        _serial_id: u32,
    ) -> Result<ResolvedInput, StateError> {
        Err(StateError::BadMember)
    }
}

struct BadProof;

impl TxProofVerifier for BadProof {
    fn verify_tx(&self, _tx: &TxPkgSum) -> Result<(), TxProofError> {
        Err(TxProofError::Invalid)
    }
}

struct EmptyMember;

impl InputResolver for EmptyMember {
    fn resolve(
        &self,
        _prev_root: CheckRoot,
        terminal_id: TerminalId,
        serial_id: u32,
    ) -> Result<ResolvedInput, StateError> {
        let case = wit_case([0x11; 32], terminal_id.into_bytes(), serial_id);
        Ok(ResolvedInput {
            path: case.path,
            leaf: case.leaf,
            member_wit: raw_wit(Vec::new(), case.proof_item),
        })
    }
}

struct BadBlob;

impl InputResolver for BadBlob {
    fn resolve(
        &self,
        _prev_root: CheckRoot,
        terminal_id: TerminalId,
        serial_id: u32,
    ) -> Result<ResolvedInput, StateError> {
        let case = wit_case([0x12; 32], terminal_id.into_bytes(), serial_id);
        Ok(ResolvedInput {
            path: case.path,
            leaf: case.leaf,
            member_wit: raw_wit(vec![1u8, 2u8, 3u8], case.proof_item),
        })
    }
}

struct RootMixMember;

impl InputResolver for RootMixMember {
    fn resolve(
        &self,
        _prev_root: CheckRoot,
        terminal_id: TerminalId,
        serial_id: u32,
    ) -> Result<ResolvedInput, StateError> {
        let case = wit_case([0x13; 32], terminal_id.into_bytes(), serial_id);
        let wrong_item = ProofItem::new_settlement(
            SettlementStateRoot::settlement_v1([0x91; 32]),
            case.path,
            case.proof_item.def_leaf(),
            case.proof_item.ser_leaf(),
            case.leaf.clone(),
        )
        .expect("proof item");
        Ok(ResolvedInput {
            path: case.path,
            leaf: case.leaf,
            member_wit: raw_wit(case.proof, wrong_item),
        })
    }
}

struct PathMixMember;

impl InputResolver for PathMixMember {
    fn resolve(
        &self,
        _prev_root: CheckRoot,
        terminal_id: TerminalId,
        serial_id: u32,
    ) -> Result<ResolvedInput, StateError> {
        let left = wit_case([0x14; 32], terminal_id.into_bytes(), serial_id);
        let right = wit_case([0x15; 32], terminal_id.into_bytes(), serial_id);
        Ok(ResolvedInput {
            path: right.path,
            leaf: right.leaf,
            member_wit: raw_wit(left.proof, right.proof_item),
        })
    }
}

struct BindProof {
    want_id: [u8; 32],
}

impl TxProofVerifier for BindProof {
    fn verify_tx(&self, tx: &TxPkgSum) -> Result<(), TxProofError> {
        if tx.resolved_inputs.len() != 1 {
            return Err(TxProofError::Invalid);
        }
        if tx.resolved_inputs[0].terminal_id().into_bytes() != self.want_id {
            return Err(TxProofError::Invalid);
        }
        if tx.resolved_inputs[0].member_wit().proof().is_empty() {
            return Err(TxProofError::Invalid);
        }
        Ok(())
    }
}

fn test_path(definition_id: [u8; 32], asset_id: [u8; 32], serial_id: u32) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new(definition_id),
        StorSerialId::new(serial_id),
        TerminalId::new(asset_id),
    )
}

fn leaf_with(asset_id: [u8; 32], serial_id: u32) -> TerminalLeaf {
    TerminalLeaf::from(AssetLeaf {
        asset_id,
        serial_id,
        ..AssetLeaf::default()
    })
}

fn out_leaf(asset_id: [u8; 32]) -> TerminalLeaf {
    leaf_with(asset_id, 0)
}

struct WitCase {
    path: SettlementPath,
    leaf: TerminalLeaf,
    proof_item: ProofItem,
    proof: Vec<u8>,
    input: ResolvedInput,
    root: CheckRoot,
}

fn raw_wit(proof: Vec<u8>, proof_item: ProofItem) -> MemberWit {
    MemberWit { proof, proof_item }
}

fn wit_case_leaf(definition_id: [u8; 32], leaf: TerminalLeaf) -> WitCase {
    let path = test_path(definition_id, leaf.asset_id, leaf.serial_id);
    let mut store = SettlementStore::new();
    let item = StoreItem::new(path, leaf.clone()).expect("store item");
    let _ = store.put_settlement_item(item).expect("put item");
    let proof_item = store.settlement_proof_item(&path).expect("proof item");
    let proof = store
        .settlement_proof_blob(&path)
        .expect("proof blob")
        .encode()
        .expect("proof bytes");
    let input = ResolvedInput::new(
        path,
        leaf.clone(),
        MemberWit::new(proof.clone(), proof_item.clone()).expect("member wit"),
    )
    .expect("resolved input");

    WitCase {
        path,
        leaf,
        proof_item,
        proof,
        input,
        root: CheckRoot::from(store.settlement_root().expect("check root")),
    }
}

fn wit_case(definition_id: [u8; 32], asset_id: [u8; 32], serial_id: u32) -> WitCase {
    wit_case_leaf(definition_id, leaf_with(asset_id, serial_id))
}

fn resolved_input(definition_id: [u8; 32], asset_id: [u8; 32], serial_id: u32) -> ResolvedInput {
    wit_case(definition_id, asset_id, serial_id).input
}

fn pair_case(
    definition_id: [u8; 32],
    left_id: [u8; 32],
    right_id: [u8; 32],
    serial_id: u32,
) -> (CheckRoot, ResolvedInput, ResolvedInput) {
    let left_leaf = leaf_with(left_id, serial_id);
    let right_leaf = leaf_with(right_id, serial_id);
    let left_path = test_path(definition_id, left_id, serial_id);
    let right_path = test_path(definition_id, right_id, serial_id);
    let left_settlement = left_path;
    let right_settlement = right_path;
    let mut store = SettlementStore::new();
    let _ = store
        .apply_settlement_ops(vec![
            z00z_storage::settlement::StoreOp::Put(Box::new(
                StoreItem::new(left_path, left_leaf.clone()).expect("left item"),
            )),
            z00z_storage::settlement::StoreOp::Put(Box::new(
                StoreItem::new(right_path, right_leaf.clone()).expect("right item"),
            )),
        ])
        .expect("apply ops");

    let left_input = ResolvedInput::new(
        left_path,
        left_leaf,
        MemberWit::new(
            store
                .settlement_proof_blob(&left_settlement)
                .expect("left blob")
                .encode()
                .expect("left bytes"),
            store
                .settlement_proof_item(&left_settlement)
                .expect("left proof"),
        )
        .expect("left wit"),
    )
    .expect("left input");
    let right_input = ResolvedInput::new(
        right_path,
        right_leaf,
        MemberWit::new(
            store
                .settlement_proof_blob(&right_settlement)
                .expect("right blob")
                .encode()
                .expect("right bytes"),
            store
                .settlement_proof_item(&right_settlement)
                .expect("right proof"),
        )
        .expect("right wit"),
    )
    .expect("right input");

    (
        CheckRoot::from(store.settlement_root().expect("check root")),
        left_input,
        right_input,
    )
}

fn one_resolver(root: CheckRoot, resolved: ResolvedInput) -> TestResolver {
    let mut map = BTreeMap::new();
    map.insert(resolved.terminal_id(), resolved);
    TestResolver { root, map }
}

fn one_ref(input_id: [u8; 32], serial_id: u32) -> Vec<TxInputWire> {
    vec![TxInputWire {
        asset_id_hex: hex::encode(input_id),
        serial_id,
    }]
}

fn one_pkg(prev_root: [u8; 32], input: ResolvedInput) -> TxPkgSum {
    let out = TerminalLeaf::default();
    TxPkgSum {
        prev_root: prev_root.into(),
        resolved_inputs: vec![input],
        outputs: vec![out],
        tx_proof: vec![1u8],
    }
}

#[test]
fn test_cp_apply_ok() {
    let id = [3u8; 32];
    let input = resolved_input([0x11; 32], id, 9);
    let root = input.member_wit().proof_root().into_bytes();
    let mut state = TestState {
        root,
        map: BTreeMap::new(),
    };
    let leaf = leaf_with(id, 9);
    state.map.insert(id, leaf);

    let tx = one_pkg(root, input);
    let cp = apply_batch_checkpoint(7, &mut state, &[tx], &OkProof, &EmptySpent).expect("cp");

    assert_eq!(cp.height, 7);
    assert_eq!(cp.prev_root, root.into());
    assert_eq!(
        cp.spent_delta,
        vec![SpentEnt {
            terminal_id: id.into()
        }]
    );
    assert_eq!(cp.created_delta.len(), 1);
}

#[test]
fn test_cp_dup_in() {
    let id = [3u8; 32];
    let input = resolved_input([0x11; 32], id, 9);
    let root = input.member_wit().proof_root().into_bytes();
    let mut state = TestState {
        root,
        map: BTreeMap::new(),
    };
    let leaf = leaf_with(id, 9);
    state.map.insert(id, leaf);

    let mut tx = one_pkg(root, input);
    tx.resolved_inputs.push(resolved_input([0x11; 32], id, 9));
    let err = apply_batch_checkpoint(7, &mut state, &[tx], &OkProof, &EmptySpent).expect_err("err");
    assert_eq!(err, StateError::DupInput);
}

#[test]
fn test_cp_empty_batch() {
    let mut state = TestState {
        root: [1u8; 32],
        map: BTreeMap::new(),
    };
    let err = apply_batch_checkpoint(7, &mut state, &[], &OkProof, &EmptySpent).expect_err("err");
    assert_eq!(err, StateError::EmptyBatch);
}

#[test]
fn test_cp_dup_out() {
    let id = [3u8; 32];
    let (root_chk, input_a, input_b) = pair_case([0x11; 32], id, [4u8; 32], 9);
    let root = root_chk.into_bytes();
    let mut state = TestState {
        root,
        map: BTreeMap::new(),
    };
    let leaf = leaf_with(id, 9);
    state.map.insert(id, leaf);

    let mut tx1 = one_pkg(root, input_a);
    tx1.outputs[0].asset_id = [9u8; 32];

    let mut tx2 = one_pkg(root, input_b);
    tx2.outputs[0].asset_id = [9u8; 32];
    state.map.insert([4u8; 32], leaf_with([4u8; 32], 9));

    let err =
        apply_batch_checkpoint(7, &mut state, &[tx1, tx2], &OkProof, &EmptySpent).expect_err("err");
    assert_eq!(err, StateError::DupOut);
}

#[test]
fn test_resolve_uses_leaf() {
    let id = [8u8; 32];
    let mut leaf = leaf_with(id, 9);
    leaf.enc_pack.ciphertext = vec![1u8, 2u8, 3u8];
    leaf.tag16 = 77;
    let case = wit_case_leaf([0x22; 32], leaf.clone());
    let resolved = resolve_inputs(
        case.root,
        &one_ref(id, 9),
        &one_resolver(case.root, case.input),
    )
    .expect("resolved");

    assert_eq!(resolved.len(), 1);
    assert_eq!(resolved[0].terminal_id().into_bytes(), id);
    assert_eq!(resolved[0].serial_id(), 9);
    assert_eq!(resolved[0].path().definition_id.into_bytes(), [0x22; 32]);
    assert_eq!(resolved[0].leaf().enc_pack.ciphertext, vec![1u8, 2u8, 3u8]);
    assert_eq!(resolved[0].leaf().tag16, 77);
    assert_eq!(resolved[0].member_wit().proof(), case.proof.as_slice());
}

#[test]
fn test_prep_bad_member() {
    let id = [8u8; 32];
    let err = prepare_tx_sum(
        [1u8; 32].into(),
        &NoMember,
        &one_ref(id, 9),
        &[TerminalLeaf::default()],
        &[1u8],
    )
    .expect_err("bad member");

    assert_eq!(err, StateError::BadMember);
}

#[test]
fn test_prep_empty_member() {
    let id = [8u8; 32];
    let err = prepare_tx_sum(
        [1u8; 32].into(),
        &EmptyMember,
        &one_ref(id, 9),
        &[TerminalLeaf::default()],
        &[1u8],
    )
    .expect_err("empty member");

    assert_eq!(err, StateError::BadMember);
}

#[test]
fn test_prep_bad_blob() {
    let id = [8u8; 32];
    let err = prepare_tx_sum(
        [1u8; 32].into(),
        &BadBlob,
        &one_ref(id, 9),
        &[TerminalLeaf::default()],
        &[1u8],
    )
    .expect_err("bad blob");

    assert_eq!(err, StateError::BadMember);
}

#[test]
fn test_prep_root_mix_member() {
    let id = [8u8; 32];
    let err = prepare_tx_sum(
        [1u8; 32].into(),
        &RootMixMember,
        &one_ref(id, 9),
        &[TerminalLeaf::default()],
        &[1u8],
    )
    .expect_err("root mix");

    assert_eq!(err, StateError::BadMember);
}

#[test]
fn test_prep_path_mix_member() {
    let id = [8u8; 32];
    let err = prepare_tx_sum(
        [1u8; 32].into(),
        &PathMixMember,
        &one_ref(id, 9),
        &[TerminalLeaf::default()],
        &[1u8],
    )
    .expect_err("path mix");

    assert_eq!(err, StateError::BadMember);
}

#[test]
fn test_prep_miss_input() {
    let resolver = TestResolver {
        root: [1u8; 32].into(),
        map: BTreeMap::new(),
    };

    let err = prepare_tx_sum(
        [1u8; 32].into(),
        &resolver,
        &one_ref([8u8; 32], 9),
        &[TerminalLeaf::default()],
        &[1u8],
    )
    .expect_err("missing input");

    assert_eq!(err, StateError::MissingInput);
}

#[test]
fn test_prep_dup_id() {
    let id = [8u8; 32];
    let case = wit_case([0x11; 32], id, 9);
    let inputs = vec![
        TxInputWire {
            asset_id_hex: hex::encode(id),
            serial_id: 9,
        },
        TxInputWire {
            asset_id_hex: hex::encode(id),
            serial_id: 10,
        },
    ];

    let err = prepare_tx_sum(
        case.root,
        &one_resolver(case.root, case.input),
        &inputs,
        &[TerminalLeaf::default()],
        &[1u8],
    )
    .expect_err("dup input");

    assert_eq!(err, StateError::DupInput);
}

#[test]
fn test_prep_leaf_match() {
    let id = [8u8; 32];
    let case = wit_case([0x11; 32], id, 9);
    let err = prepare_tx_sum(
        case.root,
        &one_resolver(case.root, case.input),
        &one_ref(id, 10),
        &[TerminalLeaf::default()],
        &[1u8],
    )
    .expect_err("leaf match");

    assert_eq!(err, StateError::LeafMatch);
}

#[test]
fn test_prep_prev_root() {
    let id = [8u8; 32];
    let case = wit_case([0x11; 32], id, 9);
    let err = prepare_tx_sum(
        [1u8; 32].into(),
        &one_resolver(case.root, case.input),
        &one_ref(id, 9),
        &[TerminalLeaf::default()],
        &[1u8],
    )
    .expect_err("prev root");

    assert_eq!(err, StateError::PrevRoot);
}

#[test]
fn test_prep_then_apply() {
    let id = [8u8; 32];
    let out_id = [9u8; 32];
    let case = wit_case([0x33; 32], id, 9);
    let root = case.root.into_bytes();
    let mut state = TestState {
        root,
        map: BTreeMap::new(),
    };
    state.map.insert(id, case.leaf.clone());

    let tx = prepare_tx_sum(
        case.root,
        &one_resolver(case.root, case.input),
        &one_ref(id, 9),
        &[out_leaf(out_id)],
        &[1u8],
    )
    .expect("prepared");

    let cp = apply_batch_checkpoint(3, &mut state, &[tx], &OkProof, &EmptySpent).expect("cp");
    assert_eq!(
        cp.spent_delta,
        vec![SpentEnt {
            terminal_id: id.into()
        }]
    );
    assert_eq!(cp.created_delta.len(), 1);
    assert_eq!(cp.created_delta[0].terminal_id, TerminalId::new(out_id));
}

#[test]
fn test_cp_bind_proof() {
    let id = [8u8; 32];
    let case = wit_case([0x33; 32], id, 9);
    let root = case.root.into_bytes();
    let mut state = TestState {
        root,
        map: BTreeMap::new(),
    };
    state.map.insert(id, case.leaf.clone());

    let tx = prepare_tx_sum(
        case.root,
        &one_resolver(case.root, case.input),
        &one_ref(id, 9),
        &[out_leaf([9u8; 32])],
        &[1u8],
    )
    .expect("prepared");

    let err = apply_batch_checkpoint(
        3,
        &mut state,
        &[tx],
        &BindProof { want_id: [7u8; 32] },
        &EmptySpent,
    )
    .expect_err("bind err");
    assert_eq!(err, StateError::TxProof(TxProofError::Invalid));
}

#[test]
fn test_cp_stable_order() {
    let in_id = [8u8; 32];
    let out_id = [9u8; 32];
    let case = wit_case([0x44; 32], in_id, 9);
    let root = case.root.into_bytes();

    let mut state_a = TestState {
        root,
        map: BTreeMap::new(),
    };
    state_a.map.insert(in_id, case.leaf.clone());
    let mut state_b = TestState {
        root,
        map: state_a.map.clone(),
    };

    let tx_a = prepare_tx_sum(
        case.root,
        &one_resolver(case.root, case.input.clone()),
        &one_ref(in_id, 9),
        &[out_leaf(out_id)],
        &[1u8],
    )
    .expect("tx a");
    let tx_b = prepare_tx_sum(
        case.root,
        &one_resolver(case.root, case.input),
        &one_ref(in_id, 9),
        &[out_leaf(out_id)],
        &[1u8],
    )
    .expect("tx b");

    let cp_a = apply_batch_checkpoint(
        3,
        &mut state_a,
        &[tx_a],
        &BindProof { want_id: in_id },
        &EmptySpent,
    )
    .expect("cp a");
    let cp_b = apply_batch_checkpoint(
        3,
        &mut state_b,
        &[tx_b],
        &BindProof { want_id: in_id },
        &EmptySpent,
    )
    .expect("cp b");

    assert_eq!(cp_a.as_pub_in(), cp_b.as_pub_in());
}

#[test]
fn test_split_member_proof() {
    let id = [8u8; 32];
    let case = wit_case([0x55; 32], id, 9);
    let root = case.root.into_bytes();
    let mut state = TestState {
        root,
        map: BTreeMap::new(),
    };
    state.map.insert(id, case.leaf.clone());

    let tx = prepare_tx_sum(
        case.root,
        &one_resolver(case.root, case.input),
        &one_ref(id, 9),
        &[out_leaf([9u8; 32])],
        &[1u8],
    )
    .expect("prepared");

    let proof_err = apply_batch_checkpoint(3, &mut state, &[tx], &BadProof, &EmptySpent)
        .expect_err("proof err");
    assert_eq!(proof_err, StateError::TxProof(TxProofError::Invalid));

    let mut state_bad = TestState {
        root,
        map: BTreeMap::new(),
    };
    state_bad.map.insert(id, leaf_with(id, 9));
    let member_err = prepare_tx_sum(
        root.into(),
        &NoMember,
        &one_ref(id, 9),
        &[TerminalLeaf::default()],
        &[1u8],
    )
    .expect_err("member err");
    assert_eq!(member_err, StateError::BadMember);
}

#[test]
fn test_prep_keeps_path() {
    let id = [8u8; 32];
    let case = wit_case([0x66; 32], id, 9);
    let tx = prepare_tx_sum(
        case.root,
        &one_resolver(case.root, case.input),
        &one_ref(id, 9),
        &[TerminalLeaf::default()],
        &[1u8],
    )
    .expect("prepared");

    assert_eq!(tx.resolved_inputs.len(), 1);
    assert_eq!(
        tx.resolved_inputs[0].path().definition_id.into_bytes(),
        [0x66; 32]
    );
}

#[test]
fn test_pub_in_roundtrip() {
    let cp = super::Checkpoint {
        height: 7,
        prev_root: [1u8; 32].into(),
        new_root: [2u8; 32].into(),
        spent_delta: vec![SpentEnt {
            terminal_id: [3u8; 32].into(),
        }],
        created_delta: vec![super::CreatedEnt {
            terminal_id: [4u8; 32].into(),
            leaf_hash: [5u8; 32],
        }],
        cp_proof: vec![9u8],
    };

    let pub_in = cp.as_pub_in();
    let json = JsonCodec.serialize(&pub_in).expect("json");
    let back: CheckpointPubIn = JsonCodec.deserialize(&json).expect("back");

    assert_eq!(json, JsonCodec.serialize(&pub_in).expect("json same"));
    assert_eq!(
        back.spent_delta,
        vec![SpentEnt {
            terminal_id: [3u8; 32].into()
        }]
    );
    assert_eq!(back.created_delta.len(), 1);
    assert_eq!(
        back.created_delta[0].terminal_id,
        TerminalId::new([4u8; 32])
    );
    assert_eq!(back.created_delta[0].leaf_hash, [5u8; 32]);
}

#[test]
fn test_pub_in_sep() {
    let cp_a = super::Checkpoint {
        height: 7,
        prev_root: [1u8; 32].into(),
        new_root: [2u8; 32].into(),
        spent_delta: vec![SpentEnt {
            terminal_id: [3u8; 32].into(),
        }],
        created_delta: vec![super::CreatedEnt {
            terminal_id: [4u8; 32].into(),
            leaf_hash: [5u8; 32],
        }],
        cp_proof: vec![9u8],
    };
    let cp_b = super::Checkpoint {
        cp_proof: vec![8u8],
        ..cp_a.clone()
    };

    assert_eq!(cp_a.as_pub_in(), cp_b.as_pub_in());
}

#[test]
fn test_cp_bad_resolve() {
    let id = [8u8; 32];
    let mut state = TestState {
        root: [1u8; 32],
        map: BTreeMap::new(),
    };
    state.map.insert(id, leaf_with(id, 9));

    let tx = TxPkgSum {
        prev_root: [1u8; 32].into(),
        resolved_inputs: vec![ResolvedInput {
            path: test_path([0x11; 32], id, 9),
            leaf: leaf_with(id, 10),
            member_wit: raw_wit(vec![7u8], wit_case([0x11; 32], id, 9).proof_item),
        }],
        outputs: vec![out_leaf([9u8; 32])],
        tx_proof: vec![1u8],
    };

    let err = apply_batch_checkpoint(3, &mut state, &[tx], &OkProof, &EmptySpent)
        .expect_err("bad resolve");
    assert_eq!(err, StateError::BadResolve);
}

#[test]
fn test_cp_bad_leaf_bytes() {
    let id = [8u8; 32];
    let mut state = TestState {
        root: [1u8; 32],
        map: BTreeMap::new(),
    };
    state.map.insert(id, {
        let mut leaf = leaf_with(id, 9);
        leaf.owner_tag = [1u8; 32];
        leaf
    });

    let tx = TxPkgSum {
        prev_root: [1u8; 32].into(),
        resolved_inputs: vec![ResolvedInput {
            path: test_path([0x11; 32], id, 9),
            leaf: {
                let mut leaf = leaf_with(id, 9);
                leaf.owner_tag = [2u8; 32];
                leaf
            },
            member_wit: raw_wit(vec![7u8], wit_case([0x11; 32], id, 9).proof_item),
        }],
        outputs: vec![out_leaf([9u8; 32])],
        tx_proof: vec![1u8],
    };

    let err = apply_batch_checkpoint(3, &mut state, &[tx], &OkProof, &EmptySpent)
        .expect_err("bad leaf bytes");
    assert_eq!(err, StateError::BadResolve);
}

#[test]
fn test_cp_spent_batch() {
    let id = [8u8; 32];
    let input = resolved_input([0x11; 32], id, 9);
    let root = input.member_wit().proof_root().into_bytes();
    let mut state = TestState {
        root,
        map: BTreeMap::new(),
    };
    state.map.insert(id, leaf_with(id, 9));

    let mk_tx = |out_id: [u8; 32]| TxPkgSum {
        prev_root: root.into(),
        resolved_inputs: vec![input.clone()],
        outputs: vec![out_leaf(out_id)],
        tx_proof: vec![1u8],
    };

    let err = apply_batch_checkpoint(
        3,
        &mut state,
        &[mk_tx([9u8; 32]), mk_tx([10u8; 32])],
        &OkProof,
        &EmptySpent,
    )
    .expect_err("spent batch");
    assert_eq!(err, StateError::SpentBatch);
}
