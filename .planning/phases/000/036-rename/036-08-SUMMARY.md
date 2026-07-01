# 036-08 Summary

## Scope

This summary records the completion state for `036-08-PLAN.md`, covering task
`036-05 Hold Paired Legacy/Public Outward Contracts`.

## Outcome

Plan 08 is closed for the active Phase 036 slice.

The Step 4 hold-only wave is now complete only on the two raw rows owned by the
canonical spec: `19` and `44`. No rename or delete work was emitted for
`derive_key_v2_zero_padding` or `ReceiverCardRecordV1`. The paired KDF helper
remains explicit without reopening legacy acceptance semantics, and the
published receiver-card contract remains explicit and outward-facing.

## Repository Changes

- `crates/z00z_wallets/src/db/redb_wallet_crypto_kdf_helpers.rs` now carries an
  explicit hold note stating that `derive_key_v2_zero_padding` stays named and
  separate until a later paired-lane migration proof retires that distinction.
- `crates/z00z_wallets/src/db/redb_wallet_crypto_tests.rs` now includes a guard
  test proving the current KDF lane still routes through
  `derive_key_v2_zero_padding` while the legacy version-1 metadata lane remains
  fail-closed.
- `.planning/phases/036-rename/036-TODO-2.md` now truthfully marks the Step 4
  hold checklist and required test surfaces complete after the review loop
  reconciled checklist drift.

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`: passed
- `cargo test --release --features test-fast --features wallet_debug_dump`:
  failed outside Plan 08 scope in read-only vendor
  `crates/z00z_crypto/tari/crypto/` doctests because multiple
  `tari_utilities` versions break `tari_crypto --doc`
- `cargo test -p z00z_wallets --release --features test-fast --test test_receiver_card_record`:
  passed
- `cargo test -p z00z_wallets --release --features test-fast --test test_redb_wlt_open`:
  passed
- `cargo test -p z00z_wallets --release --features test-fast --test test_s5_record_gate`:
  passed
- `rg -n "derive_key_v2_zero_padding|ReceiverCardRecordV1" crates/z00z_wallets/src crates/z00z_wallets/tests crates/z00z_simulator/src`:
  passed, with both Step 4 hold symbols still explicitly present in the live
  wallet, test, and simulator surfaces
- editor diagnostics on all Plan 08 modified Rust files and the updated TODO
  section: clean

## Review Loop

The review loop closed truthfully in four passes:

1. review pass 1 found no significant in-scope issue in the bounded Step 4
   hold slice
2. review pass 2 confirmed the code and tests were clean but identified
   checklist drift in `036-TODO-2.md`
3. the checklist was synchronized, deterministic tests were rerun green, and
   review pass 3 found no significant issues after that fix cycle
4. review pass 4 found no significant issues, making passes 3 and 4 the two
   consecutive clean review runs required by the plan verify gate after the
   last fix cycle

The exact runtime commands for `/crypto-architect`, `/security-audit`, and
`/doublecheck` were not directly available in this environment, so the review
evidence used the repo-local best-effort path: canonical spec rereads,
path-specific source inspection, residue scans, editor diagnostics, and
targeted test reruns.

## Current Boundary

This summary closes only Plan 08 of Phase 036. It does not claim execution of
the later Step 5 cleanup or Step 6 closure-validation waves now queued under
`036-09-PLAN.md` and `036-10-PLAN.md`.
