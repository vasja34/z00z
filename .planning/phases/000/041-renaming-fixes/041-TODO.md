# 041-threats-audit-TODO

Canonical design source:

- [041-threats-audit](./041-threats-audit.md)
- [041-CONTEXT.md](./041-CONTEXT.md) — decisions D-01 through D-14

Execution rules:

- execute this file in order unless a dependency note says otherwise;
- treat `041-threats-audit.md` as normative for threat meaning and this file as
  normative for execution order;
- do not introduce new runtime capabilities, admission policies, or second
  verifier layers; every task is test coverage, documentation, or naming guard
  only;
- do not rename any symbol that appears in a threat-boundary test without
  updating the test name and its corresponding audit evidence link in the same
  change slice;
- when a rename lands for a threat-seam symbol, re-run the full test suite for
  `z00z_wallets` and `z00z_simulator` before marking the rename done;
- `041-CONTEXT.md` decisions D-01 through D-04 are the acceptance policy for
  all tasks in this file; a task that violates any of D-01 through D-04 is
  not done;
- before starting any numbered task, complete its `MANDATORY pre-read` block.

## 🎯 Decision Summary

Threat-audit execution baseline for Phase 041:

1. Keep `verify_full_tx_package` as the single canonical admission entry point
   for regular tx packages; no caller may substitute `TxVerifierImpl::verify`
   as final admission (D-01, Threat T-1).
2. Keep claim verification two-step: raw `ClaimTxVerifierImpl` consistency
   check plus `AssetStore::claim_source_contract_for_item` rebinding; raw
   verifier result alone is not final admission (D-02, Threat T-2).
3. `tx_digest_hex` is routing and storage identity; auth drift does not change
   it; no consumer may use digest equality as a tamper-detection substitute for
   re-running the full verifier (D-03, Threat T-3).
4. `verify_tx_public_spend_contract` validates statement-envelope scope only;
   it cannot replace local structure, balance, signature, or range-proof
   validation (D-04, Threat T-4).
5. All four threat seam tests must remain passing after every rename slice in
   phase 041; they are the regression lock for this audit (D-13).

## 🔗 Dependency Chain

Execution dependency chain:

1. `041-T-01` confirm and lock digest-auth-drift coverage (Threat T-3)
2. `041-T-02` add local-verifier bypass boundary test (Threat T-1)
3. `041-T-03` add raw claim verifier rebind-gap test (Threat T-2)
4. `041-T-04` add partial spend contract insufficiency test (Threat T-4)
5. `041-T-05` threat-seam rename safety gate (all four seams)

Hard dependencies:

- `041-T-02`, `041-T-03`, `041-T-04` are independent of each other but each
  depends on `041-T-01` being done first (establishes the harness pattern).
- `041-T-05` depends on `041-T-01` through `041-T-04`.

## 🗂️ File-First Implementation Order

Edit order by file cluster:

1. `crates/z00z_wallets/tests/test_spend_statement.rs` — T-01 coverage
   confirmation and doc marker
2. `crates/z00z_wallets/src/core/tx/test_tx_verifier_suite.rs` — T-02 bypass
   boundary test
3. `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs` — T-03
   rebind-gap test hook
4. `crates/z00z_wallets/src/core/tx/claim_tx.rs` — T-03 supporting context
5. `crates/z00z_simulator/src/claim_pkg_consumer.rs` — T-03 rebind-gap
   reference
6. `crates/z00z_wallets/src/core/tx/test_tx_verifier_suite.rs` — T-04 partial
   spend contract insufficiency test (second edit cluster)
7. `crates/z00z_wallets/src/core/tx/spend_verification.rs` — T-04 doc marker
8. `crates/z00z_wallets/src/core/tx/tx_verifier.rs` — T-04 boundary comment
9. `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs`
   — T-05 rename safety gate audit
10. `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_validation_gates.rs`
    — T-05 rename safety gate audit

## ✅ Validation Matrix

| Source section | Requirement meaning | TODO coverage | Status |
| --- | --- | --- | --- |
| Threat T-1 verdict: "high-value seam" | No caller may use partial verifier as final admission | T-02 bypass boundary test | to add |
| Threat T-1 current defense | `verify_full_tx_package` at RPC and simulator | T-05 rename gate confirms seam names survive rename | to verify |
| Threat T-2 verdict: "high-risk API seam" | Raw claim verifier alone is not final admission; store rebind mandatory | T-03 rebind-gap test | to add |
| Threat T-2 current defense | Simulator consumer rebinds before accepting | T-03 references simulator consumer as model | to verify |
| Threat T-3 verdict: "second-order integrity risk" | Digest equality is not tamper detection | T-01 confirms existing tests cover this; adds doc marker | covered |
| Threat T-3 evidence L210, L232 | `test_package_digest_ignores_auth`, `test_digest_ignores_auth_hex` | T-01 locks coverage by adding `#[doc]` or comment anchor | covered |
| Threat T-4 verdict: "caller-misuse hazard" | Partial spend contract check is not full tx verification | T-04 partial gate insufficiency test | to add |
| Threat T-4 current defense | `verify_full_tx_package` composes both checks | T-04 doc marker in `spend_verification.rs` and `tx_verifier.rs` | to add |
| D-13 (CONTEXT.md) | Negative-path tests must guard tamper/replay/digest drift | T-01 through T-04 collectively | to verify after all tasks |
| D-01 (CONTEXT.md) | Full package verification is canonical admission | T-02 + T-05 | to add |
| D-02 (CONTEXT.md) | Claim verification is two-step | T-03 + T-05 | to add |
| D-03 (CONTEXT.md) | Digest is routing identity only | T-01 + T-05 | covered |
| D-04 (CONTEXT.md) | Spend contract is statement-envelope only | T-04 + T-05 | to add |

## 🚧 Explicit Phase Boundary

This TODO covers only test coverage and naming guard tasks derived from the
threat audit. Out of scope:

- Changing `TxVerifierImpl::verify`, `ClaimTxVerifierImpl::verify`, or
  `verify_tx_public_spend_contract` behavior.
- Adding a new verifier layer, facade, or pipeline.
- Changing admission policy in RPC or simulator paths.
- Renaming symbols (that belongs to the rename-suffix and rename-signature
  TODO artifacts for phase 041).
- Threat T-3 digest behavior change — the current design is intentional and
  the tests already lock it; no code change is needed beyond the doc anchor.

---

## Concrete Execution Tasks

---

### 041-T-01 — Confirm and anchor digest-auth-drift test coverage

**Spec references:** Threat T-3; `041-threats-audit.md` §3 verdict; D-03.

**MANDATORY pre-read:**

- `041-threats-audit.md` §3 — tx_digest_hex stability under auth drift
- `crates/z00z_wallets/tests/test_spend_statement.rs` lines 211–245 —
  `test_package_digest_ignores_auth`
- `crates/z00z_wallets/tests/test_spend_statement.rs` lines 295–325 —
  `test_digest_ignores_auth_hex`
- `crates/z00z_wallets/tests/test_spend_statement.rs` lines 326–358 —
  `test_digest_ignores_spend_blobs`

**Implementation checklist:**

- [ ] Read the three existing tests to confirm each one asserts that
  `tx_digest_hex` is unchanged after mutating `auth.spend_sig_hex`,
  `receiver_card_compact`, and spend blobs respectively.
- [ ] Add a single-line comment above each test anchoring it to Threat T-3:
  `// Threat T-3 anchor: digest is routing identity; auth drift must not
  change it.`
- [ ] Do not change test logic, assertions, or names.

**Files:**

- `crates/z00z_wallets/tests/test_spend_statement.rs`

**Tests:**

- `test_package_digest_ignores_auth` must still pass.
- `test_digest_ignores_auth_hex` must still pass.
- `test_digest_ignores_spend_blobs` must still pass.

**Exit condition:** All three tests pass with the anchor comment added.
`cargo test -p z00z_wallets test_package_digest_ignores_auth
test_digest_ignores_auth_hex test_digest_ignores_spend_blobs` returns green.

---

### 041-T-02 — Add local-verifier bypass boundary test

**Spec references:** Threat T-1 (CRITICAL); `041-threats-audit.md` §1;
D-01; D-13.

**MANDATORY pre-read:**

- `041-threats-audit.md` §1 — admission bypass via local verifier only
- `crates/z00z_wallets/src/core/tx/tx_verifier.rs` lines 103–130 —
  `TxVerifierImpl::verify` vs `verify_full_tx_package` split
- `crates/z00z_wallets/src/core/tx/test_tx_verifier_suite.rs` lines 161–176 —
  `test_full_verifier_spend_contract` (the closest existing positive-path test)
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs`
  line 187 — canonical `verify_full_tx_package` call site

**Implementation checklist:**

- [ ] In `test_tx_verifier_suite.rs` add a test named
  `test_local_verifier_alone_is_not_final_admission`.
- [ ] The test must build a tx payload that passes
  `TxVerifierImpl::verify` (local structure, balance, signatures, range proofs)
  but whose spend proof or spend auth is absent or stale.
- [ ] Assert that `TxVerifierImpl::verify` returns `Ok` for that payload.
- [ ] Assert that `verify_full_tx_package` returns an error or a
  non-admissible result for the same payload when the spend layer is absent.
- [ ] Add a comment: `// Threat T-1 anchor: local verifier is not final
  admission; verify_full_tx_package is the canonical gate.`
- [ ] Do not change any existing test.

**Files:**

- `crates/z00z_wallets/src/core/tx/test_tx_verifier_suite.rs`

**Tests:**

- `test_local_verifier_alone_is_not_final_admission` must pass.
- All pre-existing tests in the suite must continue to pass.

**Exit condition:** `cargo test -p z00z_wallets test_local_verifier_alone_is_not_final_admission`
returns green. All pre-existing verifier suite tests remain green.

---

### 041-T-03 — Add raw claim verifier rebind-gap test

**Spec references:** Threat T-2 (HIGH); `041-threats-audit.md` §2;
D-02; D-13.

**MANDATORY pre-read:**

- `041-threats-audit.md` §2 — raw claim verify without persisted-store rebind
- `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs` lines
  142 and 277 — `ClaimTxVerifierImpl::verify` scope boundary
- `crates/z00z_wallets/src/core/tx/claim_tx.rs` line 227 — claim tx wire
  boundary
- `crates/z00z_simulator/src/claim_pkg_consumer.rs` lines 210 and 278 —
  the canonical two-step consumer pattern (consistency + store rebind)

**Implementation checklist:**

- [ ] Locate or create a test module in
  `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs` (or in a
  sibling test file if the module shape demands it).
- [ ] Add a test named `test_raw_claim_verifier_alone_is_not_final_admission`.
- [ ] The test must construct a claim package whose internal claim-source proof
  and statement are self-consistent but whose root is synthetic (not bound to
  the live persisted store).
- [ ] Assert that `ClaimTxVerifierImpl::verify` (or the equivalent raw verify
  surface) returns `Ok` / a passing report for the synthetic package.
- [ ] Assert that without `claim_source_contract_for_item` rebinding, no
  persisted-store validation occurs — i.e., document that the synthetic root
  passes the verifier unchallenged.
- [ ] Add a comment: `// Threat T-2 anchor: raw verifier passes internal
  consistency only; store rebinding via claim_source_contract_for_item
  is the mandatory second step for final admission.`
- [ ] Do not change any existing test or any verifier behavior.

**Files:**

- `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs`
- Optionally `crates/z00z_wallets/src/core/tx/claim_tx.rs` if a doc
  comment is added to the claim wire boundary.

**Tests:**

- `test_raw_claim_verifier_alone_is_not_final_admission` must pass.
- All pre-existing claim tx verifier tests must continue to pass.

**Exit condition:** `cargo test -p z00z_wallets test_raw_claim_verifier_alone_is_not_final_admission`
returns green. Pre-existing tests remain green.

---

### 041-T-04 — Add partial spend contract insufficiency test

**Spec references:** Threat T-4 (MEDIUM); `041-threats-audit.md` §4;
D-04; D-13.

**MANDATORY pre-read:**

- `041-threats-audit.md` §4 — public spend contract is not a full tx verifier
- `crates/z00z_wallets/src/core/tx/tx_verifier.rs` lines 103–130 —
  `verify_full_tx_package` composition of local checks plus spend contract
- `crates/z00z_wallets/src/core/tx/tx_verifier.rs` line 121 —
  `verify_tx_public_spend_contract` call site within the full verifier
- `crates/z00z_wallets/src/core/tx/test_tx_verifier_suite.rs` lines 178–195 —
  `test_public_spend_valid_package` (positive-path only)

**Implementation checklist:**

- [ ] In `test_tx_verifier_suite.rs` add a test named
  `test_partial_spend_contract_is_not_full_admission`.
- [ ] The test must construct a tx package whose public spend envelope is
  self-consistent but whose local structure validation (e.g., balance, role,
  output key) would be rejected by `TxVerifierImpl::verify`.
- [ ] Assert that `verify_tx_public_spend_contract` (called directly) returns
  a passing result for the envelope-only check.
- [ ] Assert that `verify_full_tx_package` (which composes both layers)
  returns an error for the same payload.
- [ ] Add a comment: `// Threat T-4 anchor: verify_tx_public_spend_contract
  checks statement-envelope scope only; verify_full_tx_package is required
  for complete admission.`
- [ ] In `crates/z00z_wallets/src/core/tx/spend_verification.rs` add a doc
  comment on `verify_tx_public_spend_contract` clarifying: "Validates the
  spend statement envelope only. Caller must also run `TxVerifierImpl::verify`
  for full admission; see `verify_full_tx_package`."
- [ ] In `crates/z00z_wallets/src/core/tx/tx_verifier.rs` add a doc comment
  on `verify_tx_public_spend_contract` re-export or call site confirming it is
  a sub-check, not a complete gate.

**Files:**

- `crates/z00z_wallets/src/core/tx/test_tx_verifier_suite.rs`
- `crates/z00z_wallets/src/core/tx/spend_verification.rs`
- `crates/z00z_wallets/src/core/tx/tx_verifier.rs`

**Tests:**

- `test_partial_spend_contract_is_not_full_admission` must pass.
- `test_public_spend_valid_package` must continue to pass.
- All pre-existing verifier suite tests must remain green.

**Exit condition:** `cargo test -p z00z_wallets test_partial_spend_contract_is_not_full_admission`
returns green. Pre-existing suite remains green.
`cargo clippy -p z00z_wallets` emits zero new warnings from touched files.

---

### 041-T-05 — Threat-seam rename safety gate

**Spec references:** D-12, D-13, D-14 (041-CONTEXT.md); all four threats.

**MANDATORY pre-read:**

- `041-CONTEXT.md` decisions D-12 through D-14
- All four `041-T-01` through `041-T-04` tasks must be complete before
  starting this task.
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs`
  line 187 — `verify_full_tx_package` canonical RPC call site
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_validation_gates.rs`
  line 186 — `verify_full_tx_package` canonical simulator call site

**Implementation checklist:**

- [ ] For each of the following symbol names, confirm it appears in the
  threat-anchor tests added by T-01 through T-04, and that no rename in
  phase 041 changed the name at the admission boundary without updating the
  test accordingly:
  - `verify_full_tx_package`
  - `TxVerifierImpl::verify`
  - `ClaimTxVerifierImpl::verify` (or the current raw verify surface)
  - `verify_tx_public_spend_contract`
  - `build_tx_package_digest`
  - `claim_source_contract_for_item`
- [ ] For each symbol: run a `grep` to confirm the threat-anchor test still
  references the live symbol name (not a stale pre-rename name).
- [ ] Run the full wallet and simulator test suite:
  `cargo test -p z00z_wallets && cargo test -p z00z_simulator`
- [ ] Run `cargo clippy -p z00z_wallets -p z00z_simulator` and confirm zero
  new warnings from threat-seam files.
- [ ] Run `cargo fmt --check -p z00z_wallets -p z00z_simulator` and confirm
  clean output.

**Files audited:**

- `crates/z00z_wallets/src/core/tx/tx_verifier.rs`
- `crates/z00z_wallets/src/core/tx/test_tx_verifier_suite.rs`
- `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs`
- `crates/z00z_wallets/src/core/tx/tx_digest.rs`
- `crates/z00z_wallets/src/core/tx/spend_verification.rs`
- `crates/z00z_wallets/tests/test_spend_statement.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_validation_gates.rs`
- `crates/z00z_simulator/src/claim_pkg_consumer.rs`

**Tests:**

- Full `z00z_wallets` test suite passes.
- Full `z00z_simulator` test suite passes.
- The four threat-anchor tests from T-01 through T-04 are all green.

**Exit condition:** `cargo test -p z00z_wallets && cargo test -p z00z_simulator`
returns green. All four threat-anchor tests exist with their anchoring comments
and reference live symbol names. `cargo clippy` and `cargo fmt --check` emit
no new errors on touched files. Phase 041 rename slices may proceed after this
gate passes.
