use std::collections::BTreeSet;

use super::{
    build_selection_fixture, check_batch, check_statement, derive_output_id, is_low_link, InRef,
    MultiErr, MultiStmt, OutRef, SelCase,
};
use z00z_crypto::{Hidden, Z00ZScalar};

fn spent() -> BTreeSet<[u8; 32]> {
    BTreeSet::new()
}

fn state() -> BTreeSet<[u8; 32]> {
    BTreeSet::new()
}

fn one_blind(n: usize) -> Vec<Hidden<Z00ZScalar>> {
    (0..n).map(|_| Hidden::hide(super::one_scalar())).collect()
}

#[test]
fn test_fix_2_2() {
    let fix = build_selection_fixture(SelCase::In2Out2);
    assert_eq!(fix.stmt.inputs.len(), 2);
    assert_eq!(fix.stmt.outputs.len(), 3);
    assert_eq!(fix.stmt.fee, 5);
    assert_eq!(fix.change, 15);
    assert_eq!(check_statement(&fix.stmt, &spent(), &state()), Ok(()));
}

#[test]
fn test_fix_3_2() {
    let fix = build_selection_fixture(SelCase::In3Out2);
    assert_eq!(fix.stmt.inputs.len(), 3);
    assert_eq!(fix.stmt.outputs.len(), 3);
    assert_eq!(fix.stmt.fee, 10);
    assert_eq!(fix.change, 10);
    assert_eq!(check_statement(&fix.stmt, &spent(), &state()), Ok(()));
}

#[test]
fn test_fix_5_3() {
    let fix = build_selection_fixture(SelCase::In5Out3);
    assert_eq!(fix.stmt.inputs.len(), 5);
    assert_eq!(fix.stmt.outputs.len(), 4);
    assert_eq!(fix.stmt.fee, 5);
    assert_eq!(fix.change, 20);
    assert_eq!(check_statement(&fix.stmt, &spent(), &state()), Ok(()));
}

#[test]
fn test_overflow_sum() {
    let stmt = MultiStmt {
        inputs: vec![
            InRef {
                asset_id: [1u8; 32],
                serial_id: 1,
                amount: u64::MAX,
            },
            InRef {
                asset_id: [2u8; 32],
                serial_id: 2,
                amount: 1,
            },
        ],
        outputs: vec![OutRef {
            asset_id: [3u8; 32],
            serial_id: 3,
            amount: 1,
        }],
        in_blind: one_blind(2),
        fee: 0,
    };
    assert_eq!(
        check_statement(&stmt, &spent(), &state()),
        Err(MultiErr::InOver)
    );
}

#[test]
fn test_fee_over() {
    let stmt = MultiStmt {
        inputs: vec![InRef {
            asset_id: [1u8; 32],
            serial_id: 1,
            amount: 1,
        }],
        outputs: vec![OutRef {
            asset_id: [3u8; 32],
            serial_id: 3,
            amount: u64::MAX,
        }],
        in_blind: one_blind(1),
        fee: 1,
    };
    assert_eq!(
        check_statement(&stmt, &spent(), &state()),
        Err(MultiErr::FeeOver)
    );
}

#[test]
fn test_batch_ser_unique() {
    let a = build_selection_fixture(SelCase::In2Out2).stmt;
    let mut b = build_selection_fixture(SelCase::In3Out2).stmt;
    b.outputs[0].serial_id = a.outputs[0].serial_id;
    assert_eq!(
        check_batch(&[a, b], &spent(), &state()),
        Err(MultiErr::DupSer)
    );
}

#[test]
fn test_blind_len_guard() {
    let mut stmt = build_selection_fixture(SelCase::In2Out2).stmt;
    stmt.in_blind.pop();
    assert_eq!(
        check_statement(&stmt, &spent(), &state()),
        Err(MultiErr::BlindLen)
    );
}

#[test]
fn test_zero_output_guard() {
    let mut stmt = build_selection_fixture(SelCase::In2Out2).stmt;
    stmt.outputs[0].amount = 0;
    assert_eq!(
        check_statement(&stmt, &spent(), &state()),
        Err(MultiErr::OutZero)
    );
}

#[test]
fn test_io_reuse_guard() {
    let mut stmt = build_selection_fixture(SelCase::In2Out2).stmt;
    stmt.outputs[0].asset_id = stmt.inputs[0].asset_id;
    assert_eq!(
        check_statement(&stmt, &spent(), &state()),
        Err(MultiErr::IoReuse)
    );
}

#[test]
fn test_out_id_unique() {
    let seed = [9u8; 32];
    let a = derive_output_id(seed, 10, 0);
    let b = derive_output_id(seed, 11, 0);
    let c = derive_output_id(seed, 10, 1);
    assert_ne!(a, b);
    assert_ne!(a, c);
    assert_ne!(b, c);
}

#[test]
fn test_priv_low_for_change() {
    let stmt = build_selection_fixture(SelCase::In2Out2).stmt;
    assert!(is_low_link(&stmt));
}

#[test]
fn test_priv_high_single_out() {
    let stmt = MultiStmt {
        inputs: vec![InRef {
            asset_id: [1u8; 32],
            serial_id: 1,
            amount: 100,
        }],
        outputs: vec![OutRef {
            asset_id: [2u8; 32],
            serial_id: 2,
            amount: 100,
        }],
        in_blind: one_blind(1),
        fee: 1,
    };
    assert!(!is_low_link(&stmt));
}

#[test]
fn test_priv_high_mirror() {
    let stmt = MultiStmt {
        inputs: vec![
            InRef {
                asset_id: [1u8; 32],
                serial_id: 1,
                amount: 40,
            },
            InRef {
                asset_id: [3u8; 32],
                serial_id: 2,
                amount: 60,
            },
        ],
        outputs: vec![
            OutRef {
                asset_id: [2u8; 32],
                serial_id: 3,
                amount: 40,
            },
            OutRef {
                asset_id: [4u8; 32],
                serial_id: 4,
                amount: 59,
            },
        ],
        in_blind: one_blind(2),
        fee: 1,
    };
    assert!(!is_low_link(&stmt));
}

#[test]
fn test_no_change_reject() {
    let stmt = MultiStmt {
        inputs: vec![
            InRef {
                asset_id: [31u8; 32],
                serial_id: 70,
                amount: 70,
            },
            InRef {
                asset_id: [32u8; 32],
                serial_id: 71,
                amount: 50,
            },
        ],
        outputs: vec![
            OutRef {
                asset_id: [33u8; 32],
                serial_id: 80,
                amount: 100,
            },
            OutRef {
                asset_id: [34u8; 32],
                serial_id: 81,
                amount: 5,
            },
        ],
        in_blind: one_blind(2),
        fee: 5,
    };

    assert_eq!(
        check_statement(&stmt, &spent(), &state()),
        Err(MultiErr::BadBal)
    );
}
