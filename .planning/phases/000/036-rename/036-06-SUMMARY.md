# 036-06 Summary

## Scope

This summary records the completion state for `036-06-PLAN.md`, covering task
`036-03 Rename Internal Wiring And Diagnostic Noise`.

## Outcome

Plan 06 is closed for the active Phase 036 slice.

The Step 2 rename-ready wave is now complete only on the three raw rows owned
by the canonical spec: internal wiring now uses `KeyExportPublicParams`, the
internal diagnostic enum uses `IntegrityMismatch`, and the internal helper name
is now `export_public_material_impl`. The outward RPC method name
`wallet.key.export_public_material_v2` and the outward diagnostic literal
`INTEGRITY_MISMATCH_V1` remain unchanged.

## Repository Changes

- `crates/z00z_wallets/src/adapters/rpc/wallet_dispatcher_wiring.rs` now uses
  `KeyExportPublicParams` as the internal typed export-params struct.
- `crates/z00z_wallets/src/adapters/rpc/wallet_dispatcher_wiring_register.rs`
  now routes the unchanged public method string
  `wallet.key.export_public_material_v2` through `KeyExportPublicParams`.
- `crates/z00z_wallets/src/core/wallet/errors_types.rs` now renames the
  internal enum variant to `IntegrityMismatch` while preserving the outward
  display value `INTEGRITY_MISMATCH_V1`.
- `crates/z00z_wallets/src/db/wallet_validate.rs` now maps
  `WalletDiagCode::IntegrityMismatch` to the same diagnostic rank as before.
- `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/server_requests.rs`
  and `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/server.rs` now
  use the internal helper name `export_public_material_impl` while leaving the
  public v2 RPC surface unchanged.
- `.planning/phases/036-rename/036-TODO-2.md` now truthfully marks the Step 2
  checklist and required test reruns complete after review pass 1 reconciled
  the checklist state with the implemented code.

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`: passed
- `cargo test --release --features test-fast --features wallet_debug_dump`:
  failed outside Plan 06 scope in read-only vendor `crates/z00z_crypto/tari/`
  doctests because multiple `tari_utilities` versions break `tari_crypto --doc`
- `cargo test -p z00z_wallets --release --features test-fast --test test_rpc_wiring_spec_a`:
  passed
- `cargo test -p z00z_wallets --release --features test-fast --lib export_public_material_v2`:
  passed
- `rg -n "KeyExportPublicV2Params|IntegrityMismatchV1|export_public_material_v2_impl|wallet\.key\.export_public_material_v2" crates/z00z_wallets/src crates/z00z_wallets/tests`:
  passed, with old internal spellings absent and only the intended outward
  public RPC string retained
- editor diagnostics on all Plan 06 modified Rust files and the updated TODO
  section: clean
- `.github/prompts/**/*.tar.gz` residue check: no prompt-archive artifact
  present at closeout

## Review Loop

The review loop closed truthfully in three passes:

1. review pass 1 found no substantive code defect but did correct checklist
   drift in `036-TODO-2.md`
2. review pass 2 found no significant issues after the checklist sync
3. review pass 3 found no significant issues, making passes 2 and 3 the two
   consecutive clean review runs required by the plan verify gate

The exact runtime commands for `/crypto-architect`, `/security-audit`, and
`/doublecheck` were not directly available in this environment, so the review
evidence used the repo-local best-effort path: canonical spec rereads,
symbol-usage inspection, residue scans, editor diagnostics, and targeted test
reruns.

## Current Boundary

This summary closes only Plan 06 of Phase 036. It does not claim execution of
the later Step 3 through Step 7 rename or validation waves now queued under
`036-07-PLAN.md` through `036-10-PLAN.md`.
