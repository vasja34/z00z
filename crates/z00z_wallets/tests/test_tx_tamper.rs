#[path = "test_inc/test_proof_blob_case.inc"]
mod proof_blob_case;
#[path = "test_inc/test_proof_blob_fix.inc"]
mod proof_blob_fix;

use z00z_storage::settlement::TerminalId;
use z00z_storage::settlement::TerminalLeaf;
use z00z_wallets::tx::{
    prepare_tx_sum, InputResolver, MemberWit, ResolvedInput, StateError, TxInputWire,
};

use self::proof_blob_case::wit_case;
use self::proof_blob_fix::WitCase;

fn expect_bad_member(err: StateError) {
    assert_eq!(err, StateError::BadMember);
}

fn test_out() -> [TerminalLeaf; 1] {
    [TerminalLeaf::default()]
}

fn same_input(
    case: &WitCase,
    prev_root: z00z_storage::settlement::CheckRoot,
    terminal_id: TerminalId,
    serial_id: u32,
) -> Result<(), StateError> {
    if prev_root != case.root
        || terminal_id != case.input.terminal_id()
        || serial_id != case.input.serial_id()
    {
        return Err(StateError::LeafMatch);
    }

    Ok(())
}

fn flip_proof(mut proof: Vec<u8>) -> Vec<u8> {
    let idx = proof.len().saturating_sub(1);
    proof[idx] ^= 1;
    proof
}

struct TamperResolver {
    case: WitCase,
}

impl InputResolver for TamperResolver {
    fn resolve(
        &self,
        prev_root: z00z_storage::settlement::CheckRoot,
        terminal_id: TerminalId,
        serial_id: u32,
    ) -> Result<ResolvedInput, StateError> {
        same_input(&self.case, prev_root, terminal_id, serial_id)?;

        let wit = MemberWit::new(
            flip_proof(self.case.proof.clone()),
            self.case.proof_item.clone(),
        )?;
        ResolvedInput::new(self.case.path, self.case.leaf.clone(), wit)
    }
}

fn one_ref(case: &WitCase) -> Vec<TxInputWire> {
    vec![TxInputWire {
        asset_id_hex: hex::encode(case.input.terminal_id().into_bytes()),
        serial_id: case.input.serial_id(),
    }]
}

#[test]
fn test_stage4_rejects_witness_bytes() {
    let case = wit_case([0x51; 32], 9, [3u8; 32]);
    let refs = one_ref(&case);

    let err = prepare_tx_sum(
        case.root,
        &TamperResolver { case },
        &refs,
        &test_out(),
        &[1u8],
    )
    .expect_err("tampered witness bytes must fail at typed wallet boundary");

    expect_bad_member(err);
}

#[test]
fn test_stage4_rejects_bad_blob() {
    let case = wit_case([0x61; 32], 9, [7u8; 32]);
    let err = MemberWit::new(vec![1u8, 2u8, 3u8], case.proof_item)
        .expect_err("non-empty opaque blob must not pass as witness");

    expect_bad_member(err);
}
