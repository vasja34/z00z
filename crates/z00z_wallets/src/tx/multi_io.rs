//! Multi-input and multi-output planning helpers.

use rand::{rngs::StdRng, Rng, SeedableRng};
use std::collections::BTreeSet;
use thiserror::Error;
use z00z_core::{assets::AssetClass, AssetWire};
use z00z_utils::rng::SystemRngProvider;

/// Asset input-selection configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssetSelCfg {
    /// Minimum allowed distinct serial ids.
    pub distinct_serial_ids_min: u32,
    /// Desired distinct serial ids; falls back to `distinct_serial_ids_min` when zero.
    pub distinct_serial_ids_target: u32,
    /// Hard upper bound for distinct serial ids.
    pub distinct_serial_ids_max: u32,
}

/// Bob output planning configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BobOutCfg {
    /// Number of Bob outputs to build.
    pub count: u32,
}

/// Errors returned by multi-input/output planning helpers.
#[derive(Debug, Error)]
pub enum MultiIoErr {
    /// Distinct serial target is zero.
    #[error("distinct serial target must be > 0")]
    BadTarget,
    /// Distinct serial max is outside allowed bounds.
    #[error("distinct_serial_ids_max must be in range 1..=10")]
    BadMax,
    /// Distinct serial target is above configured max.
    #[error("target distinct serials {target} exceeds configured max {max}")]
    TargetAboveMax {
        /// Configured target.
        target: u32,
        /// Configured max.
        max: u32,
    },
    /// No spendable rows matched class/symbol filters.
    #[error("no spendable rows for class '{class}' symbol '{symbol}'")]
    NoRows {
        /// Asset class filter.
        class: String,
        /// Asset symbol filter.
        symbol: String,
    },
    /// Not enough distinct serials after filtering.
    #[error("distinct serial_id requirement not satisfied: got {got}, need {need}")]
    NotEnoughSerials {
        /// Observed distinct serial count.
        got: usize,
        /// Required distinct serial count.
        need: usize,
    },
    /// Selection output is empty after filtering.
    #[error("no selected inputs after distinct-serial filtering")]
    NoSelection,
    /// Bob output count is zero.
    #[error("bob output count must be > 0")]
    BadOutCount,
    /// Bob output count is below required distinct input serial count.
    #[error("bob output count {count} is below required distinct serial count {need}")]
    OutCountBelowSerials {
        /// Requested number of Bob outputs.
        count: usize,
        /// Required count equal to distinct input serial ids.
        need: usize,
    },
    /// Send amount cannot be split into requested output count.
    #[error("send amount {amount} is too small for output count {count}")]
    BadOutSplit {
        /// Total amount to split.
        amount: u64,
        /// Requested number of outputs.
        count: usize,
    },
    /// Source input serial list is empty.
    #[error("input serial ids cannot be empty")]
    NoInputSerials,
}

fn target_count(cfg: &AssetSelCfg) -> Result<usize, MultiIoErr> {
    let target = if cfg.distinct_serial_ids_target == 0 {
        cfg.distinct_serial_ids_min
    } else {
        cfg.distinct_serial_ids_target
    };
    if target == 0 {
        return Err(MultiIoErr::BadTarget);
    }
    if cfg.distinct_serial_ids_max == 0 || cfg.distinct_serial_ids_max > 10 {
        return Err(MultiIoErr::BadMax);
    }
    if target > cfg.distinct_serial_ids_max {
        return Err(MultiIoErr::TargetAboveMax {
            target,
            max: cfg.distinct_serial_ids_max,
        });
    }
    Ok(target as usize)
}

/// Select spendable inputs with distinct serial ids.
pub fn pick_input_rows(
    all_rows: Vec<AssetWire>,
    class: AssetClass,
    symbol: &str,
    cfg: AssetSelCfg,
) -> Result<Vec<AssetWire>, MultiIoErr> {
    let mut spendable: Vec<AssetWire> = all_rows
        .into_iter()
        .filter(|w| w.amount > 0)
        .filter(|w| w.definition.class == class)
        .filter(|w| w.definition.symbol.eq_ignore_ascii_case(symbol))
        .collect();

    if spendable.is_empty() {
        return Err(MultiIoErr::NoRows {
            class: format!("{class:?}"),
            symbol: symbol.to_string(),
        });
    }

    // Deterministic order for reproducible selection.
    spendable.sort_by(|a, b| {
        b.amount
            .cmp(&a.amount)
            .then_with(|| a.serial_id.cmp(&b.serial_id))
            .then_with(|| a.definition.id.cmp(&b.definition.id))
    });

    let target = target_count(&cfg)?;
    let mut picked = Vec::<AssetWire>::new();
    let mut serials = BTreeSet::<u32>::new();

    for row in spendable {
        if serials.insert(row.serial_id) {
            picked.push(row);
        }
        if serials.len() >= target {
            break;
        }
    }

    if serials.len() < target {
        return Err(MultiIoErr::NotEnoughSerials {
            got: serials.len(),
            need: target,
        });
    }
    if picked.is_empty() {
        return Err(MultiIoErr::NoSelection);
    }

    Ok(picked)
}

fn same_vals(vals: &[u64]) -> bool {
    if vals.is_empty() {
        return true;
    }
    vals.iter().all(|v| *v == vals[0])
}

fn split_with_rng<R: Rng + ?Sized>(amount: u64, count: usize, rng: &mut R) -> Vec<u64> {
    if count == 1 {
        return vec![amount];
    }
    let mut cuts = BTreeSet::<u64>::new();
    while cuts.len() < (count - 1) {
        let cut = rng.gen_range(1..amount);
        cuts.insert(cut);
    }

    let mut out = Vec::<u64>::with_capacity(count);
    let mut prev = 0u64;
    for cut in cuts {
        out.push(cut - prev);
        prev = cut;
    }
    out.push(amount - prev);
    out
}

/// Split sender amount into Bob outputs.
///
/// When `count > 1` this creates randomized, non-uniform chunks where possible.
pub fn split_output_amounts(
    amount: u64,
    cfg: BobOutCfg,
    seed: Option<u64>,
) -> Result<Vec<u64>, MultiIoErr> {
    let count = cfg.count as usize;
    if count == 0 {
        return Err(MultiIoErr::BadOutCount);
    }
    if count == 1 {
        return Ok(vec![amount]);
    }
    if amount < count as u64 {
        return Err(MultiIoErr::BadOutSplit { amount, count });
    }

    let mut best = Vec::<u64>::new();
    match seed {
        Some(seed_val) => {
            let mut rng = StdRng::seed_from_u64(seed_val);
            for _ in 0..32 {
                let vals = split_with_rng(amount, count, &mut rng);
                if !same_vals(&vals) {
                    return Ok(vals);
                }
                best = vals;
            }
        }
        None => {
            let mut rng = SystemRngProvider.rng();
            for _ in 0..32 {
                let vals = split_with_rng(amount, count, &mut rng);
                if !same_vals(&vals) {
                    return Ok(vals);
                }
                best = vals;
            }
        }
    }

    Ok(best)
}

/// Assign serial ids from selected Alice inputs to Bob outputs.
pub fn pick_output_serials(
    input_serials: &[u32],
    out_count: usize,
    seed: Option<u64>,
) -> Result<Vec<u32>, MultiIoErr> {
    if input_serials.is_empty() {
        return Err(MultiIoErr::NoInputSerials);
    }
    if out_count == 0 {
        return Ok(Vec::new());
    }

    let uniq: Vec<u32> = {
        let mut set = BTreeSet::<u32>::new();
        for serial in input_serials {
            set.insert(*serial);
        }
        set.into_iter().collect()
    };

    if out_count < uniq.len() {
        return Err(MultiIoErr::OutCountBelowSerials {
            count: out_count,
            need: uniq.len(),
        });
    }

    let mut out = Vec::with_capacity(out_count);
    match seed {
        Some(seed_val) => {
            let mut rng = StdRng::seed_from_u64(seed_val);
            // Coverage phase: each distinct input serial appears at least once.
            for serial in &uniq {
                out.push(*serial);
            }
            // Fill phase: remaining outputs can repeat serials randomly.
            while out.len() < out_count {
                let idx = rng.gen_range(0..uniq.len());
                out.push(uniq[idx]);
            }
        }
        None => {
            let mut rng = SystemRngProvider.rng();
            for serial in &uniq {
                out.push(*serial);
            }
            while out.len() < out_count {
                let idx = rng.gen_range(0..uniq.len());
                out.push(uniq[idx]);
            }
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::{pick_output_serials, split_output_amounts, AssetSelCfg, BobOutCfg, MultiIoErr};
    use std::collections::BTreeSet;

    #[test]
    fn test_split_single_out() {
        let vals = split_output_amounts(100, BobOutCfg { count: 1 }, Some(7)).expect("single");
        assert_eq!(vals, vec![100]);
    }

    #[test]
    fn test_split_multi_sum_ok() {
        let vals = split_output_amounts(100, BobOutCfg { count: 3 }, Some(7)).expect("multi");
        assert_eq!(vals.len(), 3);
        assert_eq!(vals.iter().sum::<u64>(), 100);
        assert!(vals.iter().all(|v| *v > 0));
    }

    #[test]
    fn test_split_reject_bad_count() {
        let err = split_output_amounts(100, BobOutCfg { count: 0 }, None).expect_err("count");
        assert!(matches!(err, MultiIoErr::BadOutCount));
    }

    #[test]
    fn test_split_reject_too_small() {
        let err = split_output_amounts(2, BobOutCfg { count: 3 }, None).expect_err("size");
        assert!(matches!(err, MultiIoErr::BadOutSplit { .. }));
    }

    #[test]
    fn test_serial_pick_len_ok() {
        let out = pick_output_serials(&[5, 9, 11], 4, Some(42)).expect("serials");
        assert_eq!(out.len(), 4);
    }

    #[test]
    fn test_serial_covers_input_serial() {
        let out = pick_output_serials(&[5, 9, 11], 5, Some(42)).expect("serials");
        let out_set: BTreeSet<u32> = out.into_iter().collect();
        assert!(out_set.contains(&5));
        assert!(out_set.contains(&9));
        assert!(out_set.contains(&11));
    }

    #[test]
    fn test_serial_rejects_below_serials() {
        let err = pick_output_serials(&[5, 9, 11], 2, Some(42)).expect_err("floor");
        assert!(matches!(err, MultiIoErr::OutCountBelowSerials { .. }));
    }

    #[test]
    fn test_serial_pick_reject_empty() {
        let err = pick_output_serials(&[], 2, None).expect_err("empty");
        assert!(matches!(err, MultiIoErr::NoInputSerials));
    }

    #[test]
    fn test_target_fallback_uses_min() {
        let cfg = AssetSelCfg {
            distinct_serial_ids_min: 2,
            distinct_serial_ids_target: 0,
            distinct_serial_ids_max: 5,
        };
        let target = super::target_count(&cfg).expect("target");
        assert_eq!(target, 2);
    }
}
