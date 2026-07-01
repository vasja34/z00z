use std::path::PathBuf;

#[path = "test_inc/test_mod.rs"]
mod test_common;

use test_common::managed_test_output_root;
use z00z_crypto::{create_commitment, Z00ZCommitment, Z00ZScalar};
use z00z_utils::io::{create_dir_all, write_file};
use z00z_wallets::tx::AssetClassAuditTarget;

fn scalar(seed: u64) -> Z00ZScalar {
    let mut bytes = [0u8; 32];
    bytes[..8].copy_from_slice(&seed.to_le_bytes());
    Z00ZScalar::try_from_bytes(bytes).expect("valid scalar")
}

fn sum_scalars(vals: &[Z00ZScalar]) -> Z00ZScalar {
    assert!(!vals.is_empty(), "sum_scalars requires non-empty input");
    vals.iter()
        .skip(1)
        .fold(vals[0].dangerous_clone(), |acc, val| &acc + val)
}

fn sum_comms(vals: &[Z00ZCommitment]) -> Z00ZCommitment {
    assert!(!vals.is_empty(), "sum_comms requires non-empty input");
    vals.iter()
        .skip(1)
        .fold(vals[0].clone(), |acc, val| &acc + val)
}

fn out_dir() -> PathBuf {
    managed_test_output_root("e2e07")
}

fn run_set(set_id: u64, count: usize, log: &mut String) {
    let mut vals = Vec::with_capacity(count);
    let mut blinds = Vec::with_capacity(count);
    let mut comms = Vec::with_capacity(count);

    let mut sum_val = 0u64;

    for idx in 0..count {
        let val = 1_000 + set_id * 10 + idx as u64;
        sum_val = sum_val.checked_add(val).expect("value sum overflow");
        let blind = scalar(10_000 + set_id * 1_000 + idx as u64);
        let com = create_commitment(val, &blind).expect("commitment");

        vals.push(val);
        blinds.push(blind);
        comms.push(com);
    }

    let sum_blind = sum_scalars(&blinds);
    let lhs = sum_comms(&comms);
    let rhs = create_commitment(sum_val, &sum_blind).expect("sum commitment");

    assert_eq!(lhs, rhs, "homomorphism must hold for set {set_id}");

    log.push_str(&format!(
        "set={} n={} pass=1 sum_val={} sum_blind={} lhs={} rhs={}\n",
        set_id,
        count,
        sum_val,
        hex::encode(sum_blind.to_bytes()),
        hex::encode(lhs.as_bytes()),
        hex::encode(rhs.as_bytes())
    ));
}

fn run_tamper(log: &mut String) {
    let vals = [700u64, 701, 702, 703];
    let blinds = [
        scalar(70_001),
        scalar(70_002),
        scalar(70_003),
        scalar(70_004),
    ];

    let comms = vals
        .iter()
        .zip(blinds.iter())
        .map(|(val, blind)| create_commitment(*val, blind).expect("commitment"))
        .collect::<Vec<_>>();

    let lhs = sum_comms(&comms);
    let sum_val = vals.iter().copied().fold(0u64, |acc, val| {
        acc.checked_add(val).expect("value sum overflow")
    });
    let sum_blind = sum_scalars(&blinds);

    let rhs_ok = create_commitment(sum_val, &sum_blind).expect("sum ok");
    let tam_com = create_commitment(vals[0] + 1, &blinds[0]).expect("tam commitment");
    let mut tam_set = vec![tam_com];
    tam_set.extend(comms.iter().skip(1).cloned());
    let lhs_bad = sum_comms(&tam_set);
    let rhs_bad = create_commitment(sum_val + 1, &sum_blind).expect("sum bad");

    assert_eq!(lhs, rhs_ok, "baseline equality must hold in tamper set");
    assert_ne!(
        lhs_bad, rhs_ok,
        "tampered commitment with same blind must fail"
    );
    assert_ne!(lhs, rhs_bad, "tampered value with same blind must fail");

    log.push_str(&format!(
        "tamper n=4 pass=1 fail=1 lhs={} lhs_bad={} rhs_ok={} rhs_bad={}\n",
        hex::encode(lhs.as_bytes()),
        hex::encode(lhs_bad.as_bytes()),
        hex::encode(rhs_ok.as_bytes()),
        hex::encode(rhs_bad.as_bytes())
    ));
}

#[test]
fn test_target_resolves_commitment_shapes() {
    let issued_blind = scalar(80_001);
    let burned_blind = scalar(80_002);
    let delta_blind = &issued_blind - &burned_blind;
    let expected_total = create_commitment(40, &delta_blind).expect("delta commitment");
    let issued_total = create_commitment(100, &issued_blind).expect("issued commitment");
    let burned_total = create_commitment(60, &burned_blind).expect("burned commitment");

    let direct = AssetClassAuditTarget::ExpectedTotalCommitment {
        expected_total: expected_total.clone(),
    };
    let checkpoint = AssetClassAuditTarget::CheckpointEquation {
        expected_total: expected_total.clone(),
        checkpoint_id: "checkpoint-043-11".to_string(),
    };
    let issuance_delta = AssetClassAuditTarget::IssuanceBurnDeltaTarget {
        issued_total,
        burned_total,
    };

    assert_eq!(direct.expected_total(), expected_total);
    assert_eq!(checkpoint.expected_total(), expected_total);
    assert_eq!(issuance_delta.expected_total(), expected_total);
}

#[test]
fn test_stage4_pedersen() {
    if cfg!(debug_assertions) {
        return;
    }

    let mut log = String::from("E2E-07 matrix\n");

    for (set_id, count) in [1usize, 2, 3, 4, 5, 6, 7, 8, 9, 10].iter().enumerate() {
        run_set(set_id as u64 + 1, *count, &mut log);
    }

    run_tamper(&mut log);
    run_set(99, 128, &mut log);

    create_dir_all(out_dir()).expect("mkdir outputs/tests/e2e07");
    write_file(out_dir().join("e2e07_matrix.txt"), log.as_bytes()).expect("write matrix");
}
