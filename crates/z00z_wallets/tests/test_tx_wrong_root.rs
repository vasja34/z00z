#[path = "test_inc/test_proof_blob_case.inc"]
mod proof_blob_case;
#[path = "test_inc/test_proof_blob_fix.inc"]
mod proof_blob_fix;
#[path = "test_inc/test_proof_blob_pair.inc"]
mod proof_blob_pair;

use z00z_storage::settlement::TerminalId;
use z00z_storage::settlement::TerminalLeaf;
use z00z_wallets::tx::{prepare_tx_sum, InputResolver, ResolvedInput, StateError, TxInputWire};

use self::proof_blob_case::wit_case;
use self::proof_blob_fix::WitCase;
use self::proof_blob_pair::wit_pair;

fn expect_bad_member(err: StateError) {
    assert_eq!(err, StateError::BadMember);
}

fn test_out() -> [TerminalLeaf; 1] {
    [TerminalLeaf::default()]
}

struct FixedResolver {
    case: WitCase,
}

impl InputResolver for FixedResolver {
    fn resolve(
        &self,
        _prev_root: z00z_storage::settlement::CheckRoot,
        terminal_id: TerminalId,
        serial_id: u32,
    ) -> Result<ResolvedInput, StateError> {
        if terminal_id != self.case.input.terminal_id() || serial_id != self.case.input.serial_id()
        {
            return Err(StateError::LeafMatch);
        }
        Ok(self.case.input.clone())
    }
}

struct PathMixResolver {
    left: WitCase,
    right: WitCase,
}

impl InputResolver for PathMixResolver {
    fn resolve(
        &self,
        prev_root: z00z_storage::settlement::CheckRoot,
        terminal_id: TerminalId,
        serial_id: u32,
    ) -> Result<ResolvedInput, StateError> {
        if prev_root != self.left.root
            || terminal_id != self.left.input.terminal_id()
            || serial_id != self.left.input.serial_id()
        {
            return Err(StateError::LeafMatch);
        }

        ResolvedInput::new(
            self.left.path,
            self.left.leaf.clone(),
            self.right.input.member_wit().clone(),
        )
    }
}

fn one_ref(case: &WitCase) -> Vec<TxInputWire> {
    vec![TxInputWire {
        asset_id_hex: hex::encode(case.input.terminal_id().into_bytes()),
        serial_id: case.input.serial_id(),
    }]
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

#[test]
fn test_stage4_rejects_bad_root() {
    let case = wit_case([0x31; 32], 9, [3u8; 32]);
    let refs = one_ref(&case);
    let bad_root = [0xAA; 32].into();

    let err = prepare_tx_sum(
        bad_root,
        &FixedResolver {
            case: clone_case(&case),
        },
        &refs,
        &test_out(),
        &[1u8],
    )
    .expect_err("wrong-root witness must fail before state apply");

    expect_bad_member(err);
}

#[test]
fn test_stage4_rejects_bad_path() {
    let pair = wit_pair([0x41; 32], 9, [3u8; 32], [4u8; 32]);
    let refs = one_ref(&pair.left);

    let err = prepare_tx_sum(
        pair.root,
        &PathMixResolver {
            left: pair.left,
            right: pair.right,
        },
        &refs,
        &test_out(),
        &[1u8],
    )
    .expect_err("wrong-path witness must fail before state apply");

    expect_bad_member(err);
}
