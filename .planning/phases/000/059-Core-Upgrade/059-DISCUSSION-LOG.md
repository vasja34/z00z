# Phase 059: Core-Upgrade - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in `059-CONTEXT.md` -- this log preserves the alternatives considered.

**Date:** 2026-06-16
**Phase:** 059-Core-Upgrade
**Areas discussed:** scope, object semantics, genesis, policies, storage, wallet, simulator, tests, second-pass expansion

---

## User Direction

The user requested a comprehensive discussion pass over
`.planning/phases/059-Core-Upgrade/059-TODO.md`.

The user explicitly required:
- understand all problems and issues in `059-TODO.md`;
- account for the wide impact of new core object types across `z00z_storage`,
  `z00z_wallets`, and `z00z_simulator`;
- expand tests and simulations to all object types and all interactions;
- process the TODO at micro-level so no paragraph, idea, or requirement is
  missed;
- adapt wallet structures for all new object classes and interactions;
- adapt simulator scenarios so Alice, Bob, and Charlie transfer and validate
  different object classes end to end;
- design genesis for all object classes, recognizing not everything has
  asset-style limited supply;
- decide how policies, vouchers, and rights are generated;
- decide whether genesis remains unified or each object class has a separate
  birth/initiation point.

---

## Genesis Boundary

| Option | Description | Selected |
|--------|-------------|----------|
| One monolithic asset-style genesis | Force assets, vouchers, and rights into existing asset supply semantics. | |
| Unified genesis orchestration with typed generators | Keep one `z00z_core::genesis` boundary, with separate per-object generators and invariants. | yes |
| Separate genesis authorities per object class | Give each object class an independent initiation authority. | |

**User's choice:** The user asked to think through whether genesis should be
common or per-class. The selected solution is common orchestration with typed
per-class birth semantics.

**Notes:** This avoids duplicate chain authority while allowing assets,
vouchers, rights, and policies to have different lifecycle and supply models.

---

## Object Model

| Option | Description | Selected |
|--------|-------------|----------|
| Encode all behavior into assets | Treat vouchers/rights as asset metadata or encumbered cash. | |
| Keep Asset/Voucher/Right as sibling object classes | Preserve final value, conditional value, and authority as separate classes. | yes |
| Defer vouchers/rights to wallet-only metadata | Avoid protocol/storage changes. | |

**User's choice:** The user required full support for assets, rights, and
vouchers across core, storage, wallets, and simulator.

**Notes:** Wallet-only metadata is rejected because validators and storage must
verify committed state, proofs, lifecycle, and conservation.

---

## Storage Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Add separate voucher/right storage trees | Create parallel settlement authorities. | |
| Extend current settlement root/path/leaf family | Keep one settlement root and add typed leaf support. | yes |
| Store vouchers only in wallet payloads | Avoid committed voucher state. | |

**User's choice:** The user emphasized broad storage impact. The selected
solution extends the current generalized settlement contract in place.

**Notes:** This aligns with Phase 056-058 HJMT direction and avoids concept
drift.

---

## Wallet Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Extend asset balance rows to hold all objects | Risk treating vouchers/rights as spendable cash. | |
| Add typed object inventory with asset cash projection | Preserve asset balances while tracking vouchers and rights separately. | yes |
| Keep vouchers/rights outside wallet persistence | Prevents reliable scan, quarantine, and package building. | |

**User's choice:** The user required existing wallet structures to adapt to all
new object classes and interactions.

**Notes:** The selected approach keeps spendable cash clean and gives vouchers
and rights explicit lifecycle views.

---

## Simulator Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Add isolated unit examples only | Does not validate end-to-end transfer paths. | |
| Extend existing Alice/Bob/Charlie scenario stages | Reuse current simulator flow and add all object classes and interactions. | yes |
| Build a separate simulator lane | Risks drift from existing scenario evidence. | |

**User's choice:** The user explicitly required Alice, Bob, and Charlie to pass
different object classes and validate the full transfer path.

**Notes:** Existing `scenario_1` should be expanded rather than replaced.

---

## Second-Pass Expansion

| Option | Description | Selected |
|--------|-------------|----------|
| Leave the first context as-is | It covered TODO headings but left several implementation seams abstract. | |
| Expand the existing context in place | Preserve the current artifact and add deeper object, policy, genesis, wallet, simulator, and test matrices. | yes |
| Create a separate context document | Risks splitting downstream planning input. | |

**User's choice:** The user explicitly requested another pass to check,
improve, supplement, and expand the Phase 059 discussion context.

**Notes:** The second pass added D-38 through D-72, live-code seam notes,
target shape requirements, action interaction matrix, per-crate micro planning
drilldown, wallet adaptation matrix, simulator expansion matrix, test expansion
grid, and a planning order constraint.

---

## Source-Audit Requirement

| Option | Description | Selected |
|--------|-------------|----------|
| Plan directly from the whitepaper | Fast but risks confusing target architecture with live code. | |
| Require source audit before numbered plans | Forces live/target/migration separation before implementation planning. | yes |
| Start with wallet/simulator changes | Visible, but depends on core/storage semantics that are not yet defined. | |

**User's choice:** The user's repeated instruction emphasized micro-level
coverage and broad crate impact.

**Notes:** The selected path requires `059-SOURCE-AUDIT.md` before numbered
plans and orders core/storage semantics before wallet/simulator execution
flows.

---

## Detailed Object Support

| Option | Description | Selected |
|--------|-------------|----------|
| Generalized object payload only | Compact, but risks losing cash/voucher/right semantics. | |
| Typed object families with shared facades | Keeps Asset/Voucher/Right semantics distinct while enabling shared inventory/proof flows. | yes |
| Object classes only in wallet metadata | Rejected because storage/validators must verify them. | |

**User's choice:** The user required all object classes and interactions to be
covered across core, storage, wallets, simulator, and tests.

**Notes:** The context now locks typed target shapes and lifecycle/action
matrices for downstream planners.

---

## Deferred Ideas

The discussion deferred universal policy VM behavior, subjective/oracle-heavy
conditions, cross-chain semantics, and UI polish beyond object-class wallet/RPC
projections.
