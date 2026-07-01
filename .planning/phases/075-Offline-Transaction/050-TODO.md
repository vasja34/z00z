# 050-TODO

Canonical design source:

- [050-Offline-Tx-Spec](./050-Offline-Tx-Spec.md)

Execution rules:

- execute this file in order unless a dependency note says otherwise;
- treat the spec as normative for requirement meaning and this file as
  normative for execution order;
- keep the implementation scoped to the live wallet, RPC, and simulator seams
  named in this backlog;
- extend an existing wallet, RPC, or simulator seam instead of creating a new
  facade;
- keep `TxPackage`, `TxWire`, `TxVerifierImpl`,
  `verify_tx_public_spend_contract`, `verify_transaction_package_impl`, and
  `is_import_ready(status)` as the live authority surface unless the spec is
  updated first;
- when execution discovers a security or crypto constraint that changes
  current truth, update the spec first, then this backlog, then the affected
  tests;
- treat `owned_outputs`, `all_owned_spendable`, and local package validity as
  report surfaces, not implicit import authority;
- keep lifecycle and import-gating vocabulary centralized; do not introduce
  ad hoc status spellings in simulator or wallet tests;
- the implementation tasks `050-01` through `050-07` in this file are
  mandatory and derive from the current `create-tests` planning pass for this
  phase;
- before starting any numbered task, complete its `MANDATORY pre-read` block.

## 🎯 Decision Summary

The execution baseline for Phase 050 is:

### Decision 1: Keep One Live Package Model

Choice:

- keep the current single-package `TxPackage` path as the only live offline-
  capable transaction contract.

Chosen direction:

- extend the existing `TxPackage` path only.

### Decision 2: Distinguish Raw Receiver Routing From Published Receiver Trust

Choice:

- keep raw compact `ReceiverCard` transport for direct routing use;
- use `ReceiverCardRecord` as the canonical published receiver trust contract
  whenever publication, revocation, stale-epoch, or relabel semantics matter.

Chosen direction:

- do not collapse raw card and published record into one interchangeable blob.

### Decision 3: Reuse The Existing Public Spend Contract Conditionally

Choice:

- if `proof.spend` and `auth.spend` are both absent, stay on the local package
  verifier path;
- if either spend container is present, require the full
  `verify_tx_public_spend_contract` path and reject half-populated spend data.

Chosen direction:

- one verifier pipeline with conditional public spend enforcement and the
  existing auth layer.

### Decision 4: Harden The Existing Status Gate Instead Of Adding New Status Families

Choice:

- keep `is_import_ready(status)` as the single import-readiness gate;
- harden normalization and test coverage before adding any new producer-side
  statuses.

Chosen direction:

- one normalized gate first; tighter schemas can be a later hardening phase.

### Decision 5: Keep Verify Or Report Separate From Import Or Persist

Choice:

- preserve report-only verification and explicit import or claim routing as
  distinct phases.

Chosen direction:

- verify or report stays separate from explicit import or claim handling.

## 🔗 Dependency Chain

Execution dependency chain:

1. `050-01` canonical verifier and package authority path
2. `050-02` receiver publication-trust boundary
3. `050-03` sender output-construction invariant lock-in
4. `050-04` runtime verify and public-spend reuse boundary
5. `050-05` import-readiness vocabulary and import-boundary semantics
6. `050-06` Stage 4 or Stage 5 parity and report-only receive closure
7. `050-07` harness and seam-reuse lock-in

Hard dependencies:

- `050-02` depends on `050-01`
- `050-03` depends on `050-01`
- `050-04` depends on `050-01` and `050-03`
- `050-05` depends on `050-01` and `050-04`
- `050-06` depends on `050-02`, `050-04`, and `050-05`
- `050-07` depends on `050-01` through `050-06`

## 🗂️ File-First Implementation Order

Edit order by file cluster:

1. `crates/z00z_wallets/src/tx/verify/tx_wire_types.rs`
2. `crates/z00z_wallets/src/tx/verify/tx_digest.rs`
3. `crates/z00z_wallets/src/tx/verify/tx_verifier.rs`
4. `crates/z00z_wallets/src/tx/spend/spend_verification.rs`
5. `crates/z00z_wallets/src/stealth/output/output.rs`
6. `crates/z00z_wallets/src/stealth/mod.rs`
7. `crates/z00z_wallets/src/tx/output/output_flow.rs`
8. `crates/z00z_wallets/src/tx/mod.rs`
9. `crates/z00z_wallets/src/chain/publication/receiver_card_record.rs`
10. `crates/z00z_wallets/src/chain/mod.rs`
11. `crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_impl.rs`
12. `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs`
13. `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_catalog.rs`
14. `crates/z00z_wallets/src/adapters/rpc/types/tx.rs`
15. `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_validation_gates.rs`
16. `crates/z00z_simulator/src/scenario_1/stage_4_utils/output_construction.rs`
17. `crates/z00z_simulator/src/scenario_1/stage_4_utils/output_construction_balance.rs`
18. `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_impl.rs`
19. `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_flow.rs`
20. `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_support.rs`
21. `crates/z00z_simulator/src/scenario_1/stage_5_utils/transfer_lane_support.rs`
22. `crates/z00z_simulator/src/scenario_1/stage_5_utils/transfer_lane_runtime_support.rs`
23. wallet-side tests
24. simulator-side tests
25. harness and parity locks

## ✅ Validation Matrix

This table proves that the implementation-driving instructions from
`050-Offline-Tx-Spec.md` have been migrated into this backlog and remain
traceable section by section.

| 050-Offline-Tx-Spec section | Required theme | TODO coverage | Status |
| --- | --- | --- | --- |
| `Purpose And Authority` and `Scope` | current truth is the live delayed-connectivity package flow anchored in wallet, RPC, and simulator seams | execution rules; `Implemented Authority Surface`; `Current Offline-Capable Receive Cycle` | Validated mapped |
| `Implemented Authority Surface` | keep one live wallet/RPC/offline authority surface and forbid parallel layers | execution rules; `050-01`; `050-04`; `050-07` | Validated mapped |
| `Current Transaction Package Model` | keep `TxPackage` and `build_tx_package_digest` canonical; reject digest drift and malformed structure | `050-01`; `050-04` | Validated mapped |
| `Receiver Routing Model` | preserve signed routing card versus published record distinction | `050-02` | Validated mapped |
| `Output Construction Model` | reuse sender-side output builders and self-check helpers as canonical invariants | `050-03` | Validated mapped |
| `Local Verification Guarantees` | keep local validity package-scoped and fail closed on malformed or inconsistent payloads | `050-01`; `050-04` | Validated mapped |
| `Public Spend Authorization Path` | reuse live spend proof and auth contract; keep nullifier replay caveat explicit | `050-04` | Validated mapped |
| `Current Offline-Capable Receive Cycle` | keep verify, owned-output scan, import-readiness, and import phases distinct | `050-04`; `050-05`; `050-06`; `050-07` | Validated mapped |
| `Import Readiness Gate` | keep one normalized status gate and prove valid is not automatically importable | `050-05` | Validated mapped |
| `Lifecycle Projection` | preserve pending and confirmed vocabulary expected by reporting paths | `050-05`; `050-06` | Validated mapped |
| `Current Implementation Guarantees` | keep live guarantees honest and fail closed across parse, verify, report, and import seams | `050-01` through `050-07` | Validated mapped |

## 🧩 Implementation Boundary

The Phase 050 backlog is bounded by the live implementation seams already present in the repository:

- one `TxPackage` and `TxWire` path for offline-capable package exchange;
- one signed receiver-routing artifact and one published receiver-card record;
- one sender-output construction flow built from the live wallet helpers;
- one runtime verify/report/import path with explicit import-readiness gating;
- one lifecycle projection vocabulary for pending and confirmed states;
- one simulator parity surface for the same live wallet behavior.

## ⚙️ Concrete Execution Tasks

### 050-01 Canonical Verifier And Package Authority Path

Spec references:

- `Implemented Authority Surface`
- `Current Transaction Package Model`
- `Local Verification Guarantees`
- `Current Implementation Guarantees`

MANDATORY pre-read in `050-Offline-Tx-Spec.md`:

- `Implemented Authority Surface`
- `Current Transaction Package Model`
- `Local Verification Guarantees`

- [ ] Route delayed-connectivity package verification through one canonical
  sequence: parse package, run `TxVerifierImpl`, optionally run public spend
  verification when spend containers are present, only then expose report
  surfaces.
- [ ] Reject half-populated spend containers where proof and auth do not arrive
  together.
- [ ] Keep `tx_digest_hex` bound only to `build_tx_package_digest`; do not add
  a second digest scheme for the same payload.
- [ ] Keep runtime response terminology explicit about package-local validity.

Files:

- `crates/z00z_wallets/src/tx/verify/tx_wire_types.rs`
- `crates/z00z_wallets/src/tx/verify/tx_digest.rs`
- `crates/z00z_wallets/src/tx/verify/tx_verifier.rs`
- `crates/z00z_wallets/src/tx/spend/spend_verification.rs`
- `crates/z00z_wallets/src/tx/mod.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs`
- `crates/z00z_wallets/src/adapters/rpc/types/tx.rs`

Tests:

- [ ] extend `crates/z00z_wallets/tests/test_tx_tamper.rs`
  - digest drift rejects before owned-output reporting
  - partial spend-proof or spend-auth presence rejects
- [ ] extend `crates/z00z_wallets/tests/test_tx_digest_framing.rs`
  - package digest remains bound only to canonical digest fields
- [ ] extend `crates/z00z_simulator/tests/test_stage4_verifier_support.rs`
  - simulator parity stays aligned with the live verifier contract

Exit condition:

- callers reuse the canonical package verification path.

### 050-02 Receiver Publication-Trust Boundary

Spec references:

- `Receiver Routing Model`
- `Current Implementation Guarantees`

MANDATORY pre-read in `050-Offline-Tx-Spec.md`:

- `Receiver Routing Model`
- `Current Implementation Guarantees`

- [ ] Keep raw compact `ReceiverCard` as the direct routing artifact and keep
  `ReceiverCardRecord` as the canonical published receiver trust contract.
- [ ] Keep raw compact `ReceiverCard` decode and verification fail closed for
  unsupported version, malformed canonical bytes, bad point decode, bad
  signature, and expired-card paths.
- [ ] Reuse the existing record decode and verify path for RPC or delayed-
  connectivity publication surfaces instead of introducing a second record
  wrapper.
- [ ] Preserve revocation, stale-epoch, and relabel semantics at the published
  receiver boundary.
- [ ] Do not let any path claim publication trust from a raw compact card alone.

Files:

- `crates/z00z_wallets/src/receiver/card/stealth_card.rs`
- `crates/z00z_wallets/src/receiver/card/stealth_card_codec.rs`
- `crates/z00z_wallets/src/chain/publication/receiver_card_record.rs`
- `crates/z00z_wallets/src/chain/mod.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_validation_gates.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_support.rs`

Tests:

- [ ] extend `crates/z00z_wallets/tests/test_receiver_card_record.rs`
  - revoked, stale, and relabel paths stay fail closed
- [ ] extend `crates/z00z_wallets/tests/test_tx_roundtrip.rs`
  - raw compact receiver cards reject unsupported version, malformed canonical
    bytes, bad point decode, bad signature, and expired-card inputs
- [ ] extend `crates/z00z_wallets/tests/test_s5_record_gate.rs`
  - published record roundtrip remains the canonical trust path
- [ ] extend `crates/z00z_wallets/tests/test_tx_interop.rs`
  - raw-card transport and record-based publication semantics remain distinct
- [ ] extend `crates/z00z_simulator/tests/test_stage4_card_gate.rs`
  - simulator fee-wallet and publication-card paths reject invalid record
    compact input

Exit condition:

- live paths keep raw receiver cards and published receiver records as distinct
  trust contracts.

### 050-03 Sender Output-Construction Invariant Lock-In

Spec references:

- `Output Construction Model`
- `Current Implementation Guarantees`

MANDATORY pre-read in `050-Offline-Tx-Spec.md`:

- `Output Construction Model`
- `Current Implementation Guarantees`

- [ ] Reuse `build_tx_stealth_output`, `build_tx_stealth_output_serial`,
  `build_card_stealth_output_validated`, `build_tx_stealth_output_validated`,
  `build_output_bundle`, `build_output_bundle_with_rng`, `bind_output_wire`,
  `decode_output_pack`, and `verify_self_decrypt` as the only canonical
  sender-side output construction and self-check path.
- [ ] Remove or prevent simulator-side reimplementation drift where a caller can
  package outputs without the same self-decrypt, commitment, and range-proof
  checks.
- [ ] Keep output nonce, tag, pack, and commitment checks anchored to the core
  wallet helpers instead of stage-local ad hoc logic.
- [ ] Land simulator-side changes in the existing Stage 4 output-construction
  split seams before touching wider lane orchestration files.

Files:

- `crates/z00z_wallets/src/stealth/output/output.rs`
- `crates/z00z_wallets/src/stealth/mod.rs`
- `crates/z00z_wallets/src/tx/output/output_flow.rs`
- `crates/z00z_wallets/src/tx/mod.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/output_construction.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/output_construction_balance.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_impl.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_flow.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_validation_gates.rs`

Tests:

- [ ] extend `crates/z00z_simulator/tests/test_stage4_output_crypto.rs`
  - self-decrypt, tag, commitment, and range-proof tamper paths reject
- [ ] extend `crates/z00z_simulator/tests/test_stage4_tamper.rs`
  - stage-level tamper hooks remain aligned with core output checks
- [ ] extend `crates/z00z_wallets/tests/test_tx_roundtrip.rs`
  - core output wire roundtrip stays compatible with the package path

Exit condition:

- sender output construction has one canonical self-check contract and simulator
  paths only mirror that contract.

### 050-04 Runtime Verify, Public Spend Reuse, And Report Contract

Spec references:

- `Public Spend Authorization Path`
- `Current Offline-Capable Receive Cycle`
- `Runtime Verification Response`

MANDATORY pre-read in `050-Offline-Tx-Spec.md`:

- `Public Spend Authorization Path`
- `Current Offline-Capable Receive Cycle`

- [ ] Reuse `verify_tx_public_spend_contract` from the runtime package verify
  entry point whenever spend proof and auth are present.
- [ ] Keep `owned_outputs` empty on any package that fails local verification or
  public spend verification.
- [ ] Keep `all_owned_spendable` as a report surface derived only after local
  validity and owned-output reconstruction succeed.
- [ ] Preserve the explicit nullifier replay caveat in response semantics and
  tests; do not widen public spend wording beyond the current public boundary,
  which remains narrower than wallet-local ownership closure.

Files:

- `crates/z00z_wallets/src/tx/spend/spend_verification.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs`
- `crates/z00z_wallets/src/adapters/rpc/types/tx.rs`

Tests:

- [ ] extend `crates/z00z_wallets/tests/test_spend_witness_gate.rs`
  - public spend proof and auth drift rejects from the runtime path
- [ ] extend `crates/z00z_wallets/tests/test_direct_tx_receive.rs`
  - malformed or partially populated public spend material stays out of owned
    output reporting and import readiness
- [ ] extend `crates/z00z_simulator/tests/test_stage4_verifier_support.rs`
  - runtime and simulator stay in parity for spend-proof presence and absence

Exit condition:

- runtime verification reuses the live public spend contract and the existing
  auth layer.

### 050-05 Import-Readiness Vocabulary And Import-Boundary Semantics

Spec references:

- `Import Readiness Gate`
- `Lifecycle Projection`
- `Current Implementation Guarantees`

MANDATORY pre-read in `050-Offline-Tx-Spec.md`:

- `Import Readiness Gate`
- `Lifecycle Projection`

- [ ] Keep `is_import_ready(status)` as the single import-readiness helper.
- [ ] Harden accepted and rejected spellings before any producer writes new
  status values.
- [ ] Lock the exact lifecycle vocabulary expected by reporting and handoff
  paths to `pending_spend`, `pending_receive`, `pending_change`,
  `pending_fee`, `confirmed_spend`, `confirmed_receive`,
  `confirmed_change`, and `confirmed_fee`.
- [ ] Keep pending-to-confirmed transitions constrained to the existing helper-
  approved state pairs instead of treating lifecycle status as free-form text.
- [ ] Make the relation between verify-time `import_ready` and the actual asset
  import path explicit and testable.
- [ ] Keep import readiness dependent on both local validity and accepted
  status.

Files:

- `crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_impl.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_catalog.rs`
- `crates/z00z_simulator/src/scenario_1/stage_5_utils/transfer_lane_support.rs`

Tests:

- [ ] extend `crates/z00z_wallets/tests/test_direct_tx_receive.rs`
  - `prepared` packages stay inspectable but not import-ready
  - only accepted ready statuses unlock the runtime ready flag
- [ ] extend `crates/z00z_wallets/tests/test_tx_store_integration.rs`
  - import path stays distinct from verify-time report semantics
- [ ] extend `crates/z00z_simulator/tests/test_stage5_receive_bridge.rs`
  - Stage 5 handoff preserves lifecycle vocabulary expected by downstream
    reporting
  - only helper-approved pending-to-confirmed transitions are accepted

Exit condition:

- import-readiness semantics stay centralized in the live helper.
- lifecycle vocabulary and pending-to-confirmed transitions stay exact.

### 050-06 Stage 4 Or Stage 5 Parity And Report-Only Receive Closure

Spec references:

- `Current Offline-Capable Receive Cycle`
- `Lifecycle Projection`
- `Current Implementation Guarantees`

MANDATORY pre-read in `050-Offline-Tx-Spec.md`:

- `Current Offline-Capable Receive Cycle`
- `Lifecycle Projection`

- [ ] Preserve Stage 4 and Stage 5 parity with the wallet runtime: verify or
  report remains report-only, claim or import remains explicit, and repeated
  claim paths stay idempotent or fail closed.
- [ ] Keep simulator handoff artifacts aligned with the runtime verify response
  vocabulary.
- [ ] Keep simulator lifecycle projection aligned with the exact pending and
  confirmed status names accepted by the wallet helper.
- [ ] Reject any shortcut that converts report-only receive into implicit state
  mutation.

Files:

- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_support.rs`
- `crates/z00z_simulator/src/scenario_1/stage_5_utils/transfer_lane_support.rs`
- `crates/z00z_simulator/src/scenario_1/stage_5_utils/transfer_lane_runtime_support.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs`

Tests:

- [ ] extend `crates/z00z_simulator/tests/test_stage5_receive_bridge.rs`
  - report-only receive stays separate from explicit claim and import actions
  - explicit claim remains idempotent
- [ ] extend `crates/z00z_wallets/tests/test_direct_tx_receive.rs`
  - runtime verify path never persists assets by itself

Exit condition:

- verify or report and import or claim are provably separate lifecycle phases.

### 050-07 Harness And Seam-Reuse Lock-In

Spec references:

- `Implemented Authority Surface`
- `Current Offline-Capable Receive Cycle`

MANDATORY pre-read in `050-Offline-Tx-Spec.md`:

- `Implemented Authority Surface`
- `Current Offline-Capable Receive Cycle`

- [ ] Audit test homes and simulator support files so Phase 050 extensions land
  in the current wallet and simulator seams instead of new one-off harnesses.
- [ ] Remove any temporary helper duplication introduced while landing
  `050-01` through `050-06`.
- [ ] Add seam-map comments only where they prevent a parallel offline layer.

Files:

- `crates/z00z_wallets/src/tx/mod.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/mod.rs`
- existing Phase 050 test homes

Tests:

- [ ] extend existing wallet or simulator test homes instead of adding a
  standalone Phase 050 harness file

Exit condition:

- the phase reuses existing seams and keeps a single test/runtime layer.

## ✅ Completion Gate

Phase 050 stage 1 is complete only when all of the following hold:

- every task in this backlog through `050-07` is implemented and validated;
- the implementation stays on the existing `TxPackage` path, the existing
  public spend contract, and the existing runtime verify/import path;
- runtime verification, owned-output reporting, import-readiness, and import
  boundaries are all covered by wallet and simulator tests;
- receiver trust semantics remain split correctly between raw routing cards and
  published receiver-card records;
- every claim in the code, tests, and docs remains inside the current-state
  delayed-connectivity package model.
