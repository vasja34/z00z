---
phase: 059-Core-Upgrade
plan: 059-07
status: complete
completed: 2026-06-17
owner: Z00Z Planning
---

# 059-07 Summary: Wallet Typed Object Inventory And Persistence

## Scope Delivered

- Added a typed wallet owned-object inventory on the existing `z00z_wallets`
  RedB persistence path, with explicit `OwnedAssetPayload`,
  `OwnedVoucherPayload`, and `OwnedRightPayload` instead of forcing vouchers
  or rights through the asset payload shape.
- Added the canonical typed inventory seam
  `object_inventory_store()` / `ObjectInventoryStore`, plus unified
  `WalletOwnedObject` projection and family-aware inventory filtering for
  assets, vouchers, and rights.
- Kept the asset cash path canonical and asset-only through
  `wallet_asset_store()`, while adding separate voucher-claim and right
  inventory listing paths that do not inflate spendable balance.
- Added durable non-asset indexes by family, lifecycle status, policy
  availability, holder commitment, voucher terminal id, and right terminal id,
  and extended rotation-time index rebuild logic to vouchers and rights.
- Added fail-closed checksum and lifecycle validation for voucher/right rows,
  durable policy-aware quarantine, and unified inventory object-id projection
  for assets as well as non-asset objects.
- Synced schema, codec, debug-export, and test surfaces so the new wallet
  object kinds and index tables are visible through one storage vocabulary.

## Boundary Kept

- No parallel wallet persistence layer was introduced: asset cash authority
  stays on the existing `wallet_asset_store()` path, and vouchers/rights use
  the new typed inventory seam without duplicating asset semantics.
- `OwnedAssetPayload::VERSION = 1` and `PAYLOAD_VERSION_OWNED_ASSET = 1`
  remain readable; this slice added object families additively instead of
  rewriting legacy asset rows into a new storage truth.
- Vouchers and rights remain non-cash objects. Unknown-policy objects stay in
  durable quarantine and do not become spendable balance through restart,
  restore, or inventory projection.

## Validation

- Mandatory bootstrap gate passed on the final code:
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- Targeted wallet release validation passed:
  `cargo test -p z00z_wallets --release redb_wallet_store -- --nocapture`
  `cargo test -p z00z_wallets --release test_object_inventory -- --nocapture`
  `cargo test -p z00z_wallets --release test_owned_object_tags_roundtrip -- --nocapture`
- Broad workspace validation passed on the final code:
  `cargo test --release`
- `git diff --check` must stay clean on touched wallet and planning files for
  this slice.
- Manual review against `.github/prompts/gsd-review-tasks-execution.prompt.md`
  was run in three passes:
  pass 1 found two real issues and both were fixed: the typed inventory public
  seam exposed private fields, and unified asset rows were missing `object_id`.
  pass 2 rechecked typed inventory API, checksum gates, quarantine rules, and
  asset-only cash projection and found no significant issues.
  pass 3 rechecked schema/codecs/debug-export/test sync for the new wallet
  object kinds and index tables and found no significant issues.
- During the broad workspace gate, stale background verification trees from
  earlier YOLO review runs were holding `.cache/scenario_1/.scenario_run.lock`
  and blocking simulator release tests. Those stale external lock holders were
  terminated, after which the same `cargo test --release` gate completed green
  without code changes.

## Next Plan

Execution moves to `059-08-PLAN.md` for wallet scan, RPC, export/import, and
backup flows over the new typed object inventory.
