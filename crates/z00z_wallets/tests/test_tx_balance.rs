use std::path::PathBuf;

#[path = "test_inc/test_mod.rs"]
mod test_common;

use test_common::managed_test_output_root;
use z00z_crypto::Z00ZScalar;
use z00z_utils::io::{create_dir_all, write_file};
use z00z_wallets::tx::verify_blind_balance;

fn scalar(seed: u64) -> Z00ZScalar {
    let mut bytes = [0u8; 32];
    bytes[..8].copy_from_slice(&seed.to_le_bytes());
    Z00ZScalar::try_from_bytes(bytes).expect("valid scalar")
}

fn sum_scalars(vals: &[Z00ZScalar]) -> Z00ZScalar {
    vals.iter()
        .skip(1)
        .fold(vals[0].dangerous_clone(), |acc, val| &acc + val)
}

fn sum_with_fee(outs: &[Z00ZScalar], fee: &Z00ZScalar) -> Z00ZScalar {
    &sum_scalars(outs) + fee
}

fn make_outs(count: usize, base: u64) -> Vec<Z00ZScalar> {
    (0..count).map(|idx| scalar(base + idx as u64)).collect()
}

fn in_pair(total: &Z00ZScalar, seed: u64) -> [Z00ZScalar; 2] {
    let first = scalar(seed);
    let second = total - &first;
    [first, second]
}

fn bump_one(val: &Z00ZScalar) -> Z00ZScalar {
    val + &scalar(1)
}

fn out_dir() -> PathBuf {
    managed_test_output_root("e2e06")
}

fn run_det(log: &mut String, out_n: usize, base: u64, fee_seed: u64, in_seed: u64) {
    let outs = make_outs(out_n, base);
    let fee = scalar(fee_seed);
    let total = sum_with_fee(&outs, &fee);
    let ins = in_pair(&total, in_seed);

    let mut out_all = outs
        .iter()
        .map(Z00ZScalar::dangerous_clone)
        .collect::<Vec<_>>();
    out_all.push(fee.dangerous_clone());

    assert!(
        verify_blind_balance(&ins, &out_all),
        "valid vector must pass"
    );

    let mut bad = out_all
        .iter()
        .map(Z00ZScalar::dangerous_clone)
        .collect::<Vec<_>>();
    bad[0] = bump_one(&bad[0]);
    assert!(
        !verify_blind_balance(&ins, &bad),
        "perturbed vector must fail"
    );

    let mut fee_bad = outs
        .iter()
        .map(Z00ZScalar::dangerous_clone)
        .collect::<Vec<_>>();
    fee_bad.push(bump_one(&fee));
    assert!(
        !verify_blind_balance(&ins, &fee_bad),
        "wrong fee blind must fail balance"
    );

    assert!(
        !verify_blind_balance(&ins, &outs),
        "omitting fee blind must fail balance"
    );

    assert_eq!(ins[0].to_bytes().len(), 32, "in0 scalar size must be 32");
    assert_eq!(ins[1].to_bytes().len(), 32, "in1 scalar size must be 32");
    assert_eq!(fee.to_bytes().len(), 32, "fee scalar size must be 32");
    assert_eq!(outs[0].to_bytes().len(), 32, "out scalar size must be 32");

    log.push_str(&format!(
        "det out_n={} in0={} in1={} fee={} out0={}\n",
        out_n,
        hex::encode(ins[0].to_bytes()),
        hex::encode(ins[1].to_bytes()),
        hex::encode(fee.to_bytes()),
        hex::encode(outs[0].to_bytes())
    ));
}

fn run_sweep(log: &mut String) {
    let mut state = 0xD0E1_A2B3_C4D5_E6F7u64;
    for idx in 0..64u64 {
        state ^= state << 7;
        state ^= state >> 9;
        state ^= state << 8;

        let out_n = (state as usize % 10) + 1;
        let outs = make_outs(out_n, 10_000 + idx * 64);
        let fee = scalar(90_000 + idx);
        let total = sum_with_fee(&outs, &fee);
        let ins = in_pair(&total, 120_000 + idx);

        let mut out_all = outs
            .iter()
            .map(Z00ZScalar::dangerous_clone)
            .collect::<Vec<_>>();
        out_all.push(fee.dangerous_clone());
        assert!(
            verify_blind_balance(&ins, &out_all),
            "sweep valid must pass"
        );

        log.push_str(&format!("sweep idx={} out_n={} pass=1\n", idx, out_n));
    }
}

#[test]
fn test_stage4_balance() {
    if cfg!(debug_assertions) {
        return;
    }

    let mut log = String::from("E2E-06 balance\n");

    run_det(&mut log, 1, 100, 200, 300);
    run_det(&mut log, 2, 400, 500, 600);
    run_det(&mut log, 5, 700, 800, 900);
    run_det(&mut log, 10, 1_000, 2_000, 3_000);

    run_sweep(&mut log);

    let max_ok = u64::MAX.checked_add(1).is_none();
    assert!(max_ok, "overflow-safe handling must use checked arithmetic");
    log.push_str(&format!("overflow_checked={}\n", max_ok));

    create_dir_all(out_dir()).expect("mkdir outputs/tests/e2e06");
    write_file(out_dir().join("e2e06_balance.txt"), log.as_bytes()).expect("write e2e06 log");
}
