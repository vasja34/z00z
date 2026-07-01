use std::collections::BTreeSet;

use thiserror::Error;
use z00z_core::assets::registry::AssetId;
use z00z_crypto::{domains::TxDigestDomain, hash_zk::hash_zk, Hidden, Z00ZScalar};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InRef {
    pub asset_id: AssetId,
    pub serial_id: u32,
    pub amount: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutRef {
    pub asset_id: AssetId,
    pub serial_id: u32,
    pub amount: u64,
}

#[derive(Debug)]
pub struct MultiStmt {
    pub inputs: Vec<InRef>,
    pub outputs: Vec<OutRef>,
    pub in_blind: Vec<Hidden<Z00ZScalar>>,
    pub fee: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelCase {
    In2Out2,
    In3Out2,
    In5Out3,
}

#[derive(Debug)]
pub struct SelFix {
    pub stmt: MultiStmt,
    pub target: u64,
    pub change: u64,
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum MultiErr {
    #[error("inputs must be non-empty")]
    EmptyIn,
    #[error("outputs must be non-empty")]
    EmptyOut,
    #[error("duplicate input asset")]
    DupIn,
    #[error("duplicate output asset")]
    DupOut,
    #[error("duplicate output serial")]
    DupSer,
    #[error("input blinding count mismatch")]
    BlindLen,
    #[error("input amount must be positive")]
    InZero,
    #[error("output amount must be positive")]
    OutZero,
    #[error("output reuses input asset")]
    IoReuse,
    #[error("input sum overflow")]
    InOver,
    #[error("output sum overflow")]
    OutOver,
    #[error("output plus fee overflow")]
    FeeOver,
    #[error("input and output balance mismatch")]
    BadBal,
    #[error("output collides with spent index")]
    SpentHit,
    #[error("output collides with state")]
    StateHit,
}

fn sum_amt(vals: &[u64], err: MultiErr) -> Result<u64, MultiErr> {
    vals.iter()
        .try_fold(0u64, |acc, item| acc.checked_add(*item).ok_or(err))
}

fn one_scalar() -> Z00ZScalar {
    let mut bytes = [0u8; 32];
    bytes[0] = 1;
    Z00ZScalar::try_from_bytes(bytes).expect("valid scalar")
}

pub fn check_statement(
    stmt: &MultiStmt,
    spent: &BTreeSet<AssetId>,
    state: &BTreeSet<AssetId>,
) -> Result<(), MultiErr> {
    if stmt.inputs.is_empty() {
        return Err(MultiErr::EmptyIn);
    }
    if stmt.outputs.is_empty() {
        return Err(MultiErr::EmptyOut);
    }
    if stmt.in_blind.len() != stmt.inputs.len() {
        return Err(MultiErr::BlindLen);
    }

    let mut in_ids = BTreeSet::new();
    for input in &stmt.inputs {
        if input.amount == 0 {
            return Err(MultiErr::InZero);
        }
        if !in_ids.insert(input.asset_id) {
            return Err(MultiErr::DupIn);
        }
    }

    let mut out_ids = BTreeSet::new();
    let mut out_ser = BTreeSet::new();
    for output in &stmt.outputs {
        if output.amount == 0 {
            return Err(MultiErr::OutZero);
        }
        if !out_ids.insert(output.asset_id) {
            return Err(MultiErr::DupOut);
        }
        if in_ids.contains(&output.asset_id) {
            return Err(MultiErr::IoReuse);
        }
        if !out_ser.insert(output.serial_id) {
            return Err(MultiErr::DupSer);
        }
        if spent.contains(&output.asset_id) {
            return Err(MultiErr::SpentHit);
        }
        if state.contains(&output.asset_id) {
            return Err(MultiErr::StateHit);
        }
    }

    let in_sum = sum_amt(
        &stmt.inputs.iter().map(|x| x.amount).collect::<Vec<_>>(),
        MultiErr::InOver,
    )?;
    let out_sum = sum_amt(
        &stmt.outputs.iter().map(|x| x.amount).collect::<Vec<_>>(),
        MultiErr::OutOver,
    )?;
    let _ = out_sum.checked_add(stmt.fee).ok_or(MultiErr::FeeOver)?;
    if in_sum != out_sum {
        return Err(MultiErr::BadBal);
    }

    Ok(())
}

pub fn check_batch(
    batch: &[MultiStmt],
    spent: &BTreeSet<AssetId>,
    state: &BTreeSet<AssetId>,
) -> Result<(), MultiErr> {
    let mut out_ids = BTreeSet::new();
    let mut out_ser = BTreeSet::new();

    for stmt in batch {
        check_statement(stmt, spent, state)?;
        for output in &stmt.outputs {
            if !out_ids.insert(output.asset_id) {
                return Err(MultiErr::DupOut);
            }
            if !out_ser.insert(output.serial_id) {
                return Err(MultiErr::DupSer);
            }
        }
    }

    Ok(())
}

pub fn derive_output_id(seed: [u8; 32], ser: u32, idx: u32) -> AssetId {
    hash_zk::<TxDigestDomain>("M10/OUT", &[&seed, &ser.to_le_bytes(), &idx.to_le_bytes()])
}

pub fn build_selection_fixture(case: SelCase) -> SelFix {
    let in_blind = |n: usize| {
        (0..n)
            .map(|_| Hidden::hide(one_scalar()))
            .collect::<Vec<_>>()
    };

    match case {
        SelCase::In2Out2 => {
            let target = 100;
            let fee = 5;
            let change = 15;
            SelFix {
                stmt: MultiStmt {
                    inputs: vec![
                        InRef {
                            asset_id: [1u8; 32],
                            serial_id: 10,
                            amount: 70,
                        },
                        InRef {
                            asset_id: [2u8; 32],
                            serial_id: 11,
                            amount: 50,
                        },
                    ],
                    outputs: vec![
                        OutRef {
                            asset_id: [3u8; 32],
                            serial_id: 20,
                            amount: target,
                        },
                        OutRef {
                            asset_id: [4u8; 32],
                            serial_id: 21,
                            amount: change,
                        },
                        OutRef {
                            asset_id: [5u8; 32],
                            serial_id: 22,
                            amount: fee,
                        },
                    ],
                    in_blind: in_blind(2),
                    fee,
                },
                target,
                change,
            }
        }
        SelCase::In3Out2 => {
            let target = 100;
            let fee = 10;
            let change = 10;
            SelFix {
                stmt: MultiStmt {
                    inputs: vec![
                        InRef {
                            asset_id: [11u8; 32],
                            serial_id: 30,
                            amount: 60,
                        },
                        InRef {
                            asset_id: [12u8; 32],
                            serial_id: 31,
                            amount: 40,
                        },
                        InRef {
                            asset_id: [13u8; 32],
                            serial_id: 32,
                            amount: 20,
                        },
                    ],
                    outputs: vec![
                        OutRef {
                            asset_id: [14u8; 32],
                            serial_id: 40,
                            amount: target,
                        },
                        OutRef {
                            asset_id: [15u8; 32],
                            serial_id: 41,
                            amount: change,
                        },
                        OutRef {
                            asset_id: [16u8; 32],
                            serial_id: 42,
                            amount: fee,
                        },
                    ],
                    in_blind: in_blind(3),
                    fee,
                },
                target,
                change,
            }
        }
        SelCase::In5Out3 => {
            let target = 90;
            let fee = 5;
            let change = 20;
            SelFix {
                stmt: MultiStmt {
                    inputs: vec![
                        InRef {
                            asset_id: [21u8; 32],
                            serial_id: 50,
                            amount: 40,
                        },
                        InRef {
                            asset_id: [22u8; 32],
                            serial_id: 51,
                            amount: 30,
                        },
                        InRef {
                            asset_id: [23u8; 32],
                            serial_id: 52,
                            amount: 20,
                        },
                        InRef {
                            asset_id: [24u8; 32],
                            serial_id: 53,
                            amount: 15,
                        },
                        InRef {
                            asset_id: [25u8; 32],
                            serial_id: 54,
                            amount: 10,
                        },
                    ],
                    outputs: vec![
                        OutRef {
                            asset_id: [26u8; 32],
                            serial_id: 60,
                            amount: 50,
                        },
                        OutRef {
                            asset_id: [27u8; 32],
                            serial_id: 61,
                            amount: 40,
                        },
                        OutRef {
                            asset_id: [28u8; 32],
                            serial_id: 62,
                            amount: change,
                        },
                        OutRef {
                            asset_id: [29u8; 32],
                            serial_id: 63,
                            amount: fee,
                        },
                    ],
                    in_blind: in_blind(5),
                    fee,
                },
                target,
                change,
            }
        }
    }
}

pub fn link_score(stmt: &MultiStmt) -> u8 {
    let mut risk: u16 = 0;

    if stmt.outputs.len() < 2 {
        risk += 40;
    }

    let mut out_vals = BTreeSet::new();
    for output in &stmt.outputs {
        if !out_vals.insert(output.amount) {
            risk += 25;
            break;
        }
    }

    let in_sum = stmt
        .inputs
        .iter()
        .fold(0u64, |acc, item| acc.saturating_add(item.amount));
    let out_sum = stmt
        .outputs
        .iter()
        .fold(0u64, |acc, item| acc.saturating_add(item.amount));
    if in_sum == out_sum && stmt.outputs.len() == 1 {
        risk += 35;
    }

    let mut mirror_hits = 0u16;
    for input in &stmt.inputs {
        if stmt
            .outputs
            .iter()
            .any(|output| output.amount == input.amount)
        {
            mirror_hits += 1;
        }
    }
    if mirror_hits > 0 {
        risk += 45;
        if mirror_hits > 1 {
            risk += 10;
        }
    }

    risk.min(100) as u8
}

pub fn is_low_link(stmt: &MultiStmt) -> bool {
    link_score(stmt) <= 40
}

#[cfg(test)]
#[path = "test_asset_selector_multi.rs"]
mod tests;
