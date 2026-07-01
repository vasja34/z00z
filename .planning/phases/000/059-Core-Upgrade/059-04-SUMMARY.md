---
phase: 059-Core-Upgrade
plan: 059-04
status: complete
completed: 2026-06-17
owner: Z00Z Planning
---

# 059-04 Summary: Storage Voucher Leaf Family And Proof Semantics

## Scope Delivered

- Extended the existing storage settlement leaf-family model in place with
  `VoucherLeaf`, `VoucherBackingRef`, `SettlementLeaf::Voucher`, and
  `SettlementLeafFamily::Voucher` under the same public
  `SettlementStateRoot`/`SettlementPath` contract that already carries Asset
  and Right families.
- Added voucher-aware proof, nonexistence, deletion, batch-proof, marker-leaf,
  and HJMT cache coverage across
  `record.rs`/`leaf.rs`/`proof.rs`/`proof_batch.rs`/`hjmt_cache.rs`, plus the
  downstream Stage 13 simulator surface so storage reports voucher proof lanes
  as a first-class family instead of an unknown payload.
- Kept the storage boundary honest by committing storage-owned voucher backing
  references plus raw policy or action-pool hashes instead of importing wallet-
  or runtime-owned descriptor semantics into the storage record plane.
- Added storage compatibility and regression coverage for voucher codec,
  path-family mismatch rejection, voucher nonexistence and deletion proofs,
  voucher batch-proof family tags, durable reload/listing/absence recovery, and
  voucher fuzz-seed dispatch or README coverage.

## Boundary Kept

- This slice did not introduce a parallel voucher tree, second settlement root,
  family-agnostic proof path, or wallet-secret authority inside storage.
- This slice did not yet land typed mixed-object deltas, voucher conservation
  semantics, validator or watcher runtime verdict logic, wallet typed-object
  persistence, or Alice/Bob/Charlie simulator transfer lanes; those remain in
  `059-05` through `059-10`.
- Terminal and Right family compatibility stayed additive: old family tags
  still decode, the one settlement-root contract stays intact, and voucher
  support is an in-place extension rather than a parallel layer.

## Validation

- Mandatory bootstrap gate passed on the final code:
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- Targeted storage release validation passed:
  `cargo test -p z00z_storage --release --test test_settlement_leaf --test test_hjmt_live_proof_families --test test_hjmt_batch_proof --test test_hjmt_batch_proof_negative --test test_fuzz_seeds -- --nocapture`
- Prompt-required release validations passed:
  `cargo test -p z00z_storage --release --features test-params-fast`
  `cargo test -p z00z_wallets --release --features test-params-fast`
  `cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools`
- Broad workspace validation passed on the final code:
  `cargo test --release`
- `git diff --check` must stay clean on the touched storage, simulator, and
  planning files for this slice.
- Manual review against `.github/prompts/gsd-review-tasks-execution.prompt.md`
  was run in three passes:
  pass 1 found two significant gaps: voucher reload/listing/absence recovery
  was not covered in durable HJMT reload tests, and voucher-family fuzz-seed
  coverage was not explicitly exercised or documented; both were fixed.
  pass 2 found no further significant code issues after the storage-owned
  voucher record/proof/cache review plus targeted release reruns.
  pass 3 found no significant closeout drift after the final broad release gate
  and summary/state/roadmap sync.

## Next Plan

Execution moves to `059-05-PLAN.md` for typed object deltas, conservation,
voucher lifecycle transitions, and the fee-support boundary.
