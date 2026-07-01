---
phase: 059-Core-Upgrade
plan: 059-08
status: complete
completed: 2026-06-17
owner: Z00Z Planning
---

# 059-08 Summary: Wallet Scan, Package Builder, RPC, And Backup

## Scope Delivered

- Added the typed `wallet.object.*` RPC surface on the existing wallet RPC
  stack for object inventory, voucher/right listings, package preview/build,
  and explicit voucher/right lifecycle wrappers without overloading
  `wallet.asset.*`.
- Added one shared wallet object-package path around
  `RuntimeObjectPackageV1`/`ObjectValidatorVerdict` so preview/build checks bind
  policy descriptors, action pools, required rights, roots, typed deltas, and
  fee-support boundaries through the same storage/runtime vocabulary.
- Kept asset RPC cash-only by rejecting voucher/right inventory ids on asset
  receive/send/build surfaces and by routing typed object actions through the
  new object namespace instead of asset-history or asset-transfer overloads.
- Extended wallet backup/export/import validation so typed voucher/right owned
  objects roundtrip through the existing backup packet, reject wallet-id drift,
  and fail closed on tampered owned-object checksums or tampered archives.
- Added direct wallet tests for typed inventory listing, voucher/right object
  package wrappers, and the reject paths that keep vouchers out of cash inputs
  and rights out of value inputs.

## Boundary Kept

- No parallel wallet package or object authority was introduced: the wallet
  object package surface reuses the shared storage/runtime contract instead of
  inventing wallet-local DTOs or verdict semantics.
- `wallet.asset.*` remains the only cash-facing RPC lane. Voucher/right flows
  now live under `wallet.object.*` and remain non-cash until an explicit,
  validator-checkable redeem path succeeds.
- Backup/import support stayed additive. Legacy asset-only backups remain
  readable while typed voucher/right payloads use the same existing backup
  packet rather than a second export format.

## Validation

- Mandatory bootstrap gate passed on the final code:
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- Targeted wallet release validation passed:
  `cargo test -p z00z_wallets --release test_wallet_service -- --nocapture`
  `cargo test -p z00z_wallets --release --lib test_object_rpc_lists_typed_inventory -- --nocapture`
  `cargo test -p z00z_wallets --release --lib test_asset_rpc_rejects_voucher_and_right_ids -- --nocapture`
  `cargo test -p z00z_wallets --release --lib test_tx_build_rejects_voucher_inventory_id -- --nocapture`
  `cargo test -p z00z_wallets --release --lib test_tx_send_rejects_right_inventory_id -- --nocapture`
  `cargo test -p z00z_wallets --release test_verify_backup_detects_tamper -- --nocapture`
- Broad workspace validation passed on the final code:
  `cargo test --release`
- `git diff --check` stayed clean on the touched wallet, storage/runtime, and
  planning files for this slice.
- Manual review against `.github/prompts/gsd-review-tasks-execution.prompt.md`
  was run in three passes:
  pass 1 fixed three real issues in the new object RPC slice: the
  implementation now uses the shared public session-verification helper,
  voucher reject wrappers now honor the canonical refund lifecycle mapping
  instead of a non-existent reject effect, and the RPC method registry comment
  now includes the new `object.*` namespace.
  pass 2 rechecked object namespace registration, shared package/verdict reuse,
  cash-only asset guards, and typed-object backup/import validation and found
  no significant issues.
  pass 3 rechecked the same wallet object flows plus summary/doc sync and found
  no significant issues.
- Workspace-first doublecheck of the material review claims verified the object
  namespace registration, shared session-verifier use, asset-only reject tests,
  and typed-object backup/import tamper tests against repository files and the
  release-mode command outputs above; no disputed claims remained.

## Next Plan

Execution moves to `059-09-PLAN.md` for simulator object lanes and full
Alice/Bob/Charlie end-to-end evidence across Assets, Vouchers, Rights, and
cross-object interactions.
