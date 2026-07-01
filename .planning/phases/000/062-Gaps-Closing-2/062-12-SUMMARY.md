---
phase: 062-Gaps-Closing-2
plan: 062-12
status: complete
completed_at: 2026-06-25
next_plan: 062-13
summary_artifact_for: .planning/phases/062-Gaps-Closing-2/062-12-PLAN.md
---

# 062-12 Summary: Cash/Object Separation And Voucher Baseline

## ✅ Outcome

`062-12` is complete. The grouped plan contract `PLAN-062-G12` now closes
through the renamed `062-12-PLAN.md` packet with one canonical wallet path for
cash/object separation, typed object inventory, fee-envelope and right
projection, bounded voucher lifecycle evidence, and phase-local object-wallet
docs.

The direct cash/object boundary now has explicit store-level and RPC-level
proof. `test_asset_import_security.rs` adds a locked-wallet reopen check that
proves cash assets cannot be inserted through voucher/right object storage, and
an RPC-boundary check that proves a cash import remains an `asset` projection
on `wallet.object.*` while staying available only through `wallet.asset.*`.

The existing object, voucher, right, and fee-envelope authority lane is now
explicitly bounded in the live wallet guide. `WALLET-GUIDE.md` states that
Phase 062 closes internal object families only, includes `RightLeaf`,
`VoucherLeaf`, `RightClass`, `FeeEnvelope`, the object policy registry, wallet
object inventory, validator fail-closed checks, deterministic local
voucher/right scenarios, and cash/object separation proofs, and excludes
external chain trust tiers, linked liability, live cross-chain settlement, and
any claim that vouchers or rights are ordinary wallet cash.

The execution packet itself was also corrected to stay phase-local. The final
`062-12-PLAN.md` now points to the actual wallet, validator, and simulator
anchors for this slice, removes stale `TASK-075` placeholder drift, and keeps
the closeout evidence inside the Phase 062 packet instead of reopening a
second planning authority.

## 📂 Files Changed

- `.planning/phases/062-Gaps-Closing-2/062-12-PLAN.md`
- `.planning/phases/062-Gaps-Closing-2/062-12-SUMMARY.md`
- `.planning/STATE.md`
- `.planning/ROADMAP.md`
- `crates/z00z_wallets/docs/WALLET-GUIDE.md`
- `crates/z00z_wallets/tests/test_asset_import_security.rs`

## 🧪 Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_wallets --test test_asset_import_security`
- `cargo test --release -p z00z_validators --test test_object_policy_verdicts`
- `cargo test --release -p z00z_wallets test_asset_impl -- --nocapture`
- `cargo test --release -p z00z_simulator --test scenario_1 test_scenario1_object_flows -- --nocapture`
- `cargo test --release -p z00z_storage --test test_live_guardrails test_phase0_source_promotes_live_settlement_authority -- --nocapture`
- `rg -n "Phase 062 Bounded Closeout|external chain trust tiers|linked liability" crates/z00z_wallets/docs/WALLET-GUIDE.md`
- `git diff --check -- .planning/phases/062-Gaps-Closing-2/062-12-PLAN.md crates/z00z_wallets/docs/WALLET-GUIDE.md crates/z00z_wallets/tests/test_asset_import_security.rs`

Result:

- The direct `062-12` acceptance packet is green on wallet, validator, and
  simulator release tests.
- The broad workspace release gate is still blocked outside the `062-12` slice
  by `crates/z00z_storage/tests/test_live_guardrails.rs::test_phase0_source_promotes_live_settlement_authority`,
  which expects a heading inside the phase-external
  `.planning/phases/Z00Z-IMPL-PHASES.md` file.
- That phase-external file was left untouched here because the current
  execution constraint is to work only with the Phase 062 packet.

## 🔎 Manual Review Passes

Because `/GSD-Review-Tasks-Execution` is not callable as a tool here, the
required review loop was executed manually against the same scope.

- Pass 1
  - Read `062-12-PLAN.md`, `062-TODO.md`, and the `GAPS.md` rows for
    `TASK-023`, `TASK-049`, `TASK-050`, `TASK-051`, `TASK-057`, and
    `TASK-060`.
  - Result: found stale `TASK-075` and phase-external closeout drift in
    `062-12-PLAN.md`; rewrote the plan to use the actual wallet guide,
    validator, simulator, and `test_asset_import_security.rs` anchors.
- Pass 2
  - Re-read `test_asset_import_security.rs`, `test_asset_impl.rs`,
    `test_object_policy_verdicts.rs`,
    `test_scenario1_object_flows.rs`, and `WALLET-GUIDE.md`.
  - Result: found missing direct cash/object proof on the live wallet path;
    added store-level and RPC-level no-leak tests plus a bounded Phase 062
    closeout section in `WALLET-GUIDE.md`.
  - Workspace-first doublecheck of the slice claims against these files found
    direct support for the cash/object, fee-boundary, missing-right,
    voucher-as-cash, right-as-value, and bounded-scope assertions with no
    internal contradictions in the `062-12` evidence packet.
- Pass 3
  - `cargo test --release -p z00z_wallets --test test_asset_import_security`
  - `cargo test --release -p z00z_validators --test test_object_policy_verdicts`
  - `cargo test --release -p z00z_wallets test_asset_impl -- --nocapture`
  - `cargo test --release -p z00z_simulator --test scenario_1 test_scenario1_object_flows -- --nocapture`
  - `git diff --check -- .planning/phases/062-Gaps-Closing-2/062-12-PLAN.md crates/z00z_wallets/docs/WALLET-GUIDE.md crates/z00z_wallets/tests/test_asset_import_security.rs`
  - Result: clean
- Pass 4
  - `cargo test --release -p z00z_storage --test test_live_guardrails test_phase0_source_promotes_live_settlement_authority -- --nocapture`
  - Result: reproduced the unrelated broad blocker on a phase-external doc
    anchor; no `062-12` code or doc failure surfaced.

Passes 2 and 3 were consecutive clean passes for the `062-12` slice.

## 📌 Task Closeout

- `TASK-023`
  - Closed by the new `test_object_inventory_rejects_cash_asset_payload` and
    `test_asset_import_cannot_write_object_inventory_as_cash` proofs. Cash
    assets now have explicit live evidence that they cannot be written through
    the voucher/right object path.
- `TASK-049`
  - Closed by the existing validator and simulator reject matrix plus the new
    cash/object separation proofs. Unknown policy, known policy, missing right,
    fee-boundary, voucher-as-cash, and right-as-value cases remain fail-closed
    on the current local object-family lane.
- `TASK-050`
  - Closed by the existing object RPC inventory/build tests together with the
    validator fee-boundary tests. Rights and vouchers are visible through
    `wallet.object.*`, and `wallet.asset.*` still rejects object ids as cash.
- `TASK-051`
  - Closed by the existing deterministic voucher lifecycle packet in
    `test_asset_impl.rs` and `test_scenario1_object_flows.rs`, including
    issue, accept, redeem, refund/reject, expiry, replay rejection, and
    unknown-policy rejection.
- `TASK-057`
  - Closed on the phase-local wallet guide and this summary packet under the
    current `062`-only execution constraint. The live docs now state the
    bounded internal object-family scope and explicitly exclude external trust
    tiers, linked liability, and live cross-chain settlement.
- `TASK-060`
  - Closed by the combined wallet guide, cash/object boundary tests, and
    object RPC proofs. The live documentation and tests now keep
    `wallet.object.*` as the typed object namespace and `wallet.asset.*` as the
    cash-only authority with no second wallet truth plane.
