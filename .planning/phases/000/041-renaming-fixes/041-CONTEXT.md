# Phase 041: Renaming Fixes - Context

**Gathered:** 2026-04-29
**Status:** Ready for planning

## Phase Boundary

Phase 041 standardizes naming on security-sensitive verifier and test/helper surfaces without changing behavior. Scope includes suffix cleanup and signature/test-name normalization where semantics are currently ambiguous, while preserving trait-locked and contract-bound names.

Out of scope: introducing new runtime capabilities, changing transaction/claim acceptance policy, or widening proof/public-input contracts.

## Implementation Decisions

### Threat-Critical Verifier Boundaries

- **D-01:** Keep full admission anchored on full-package verification entry points; local verifier helpers must not be treated as final tx admission.
- **D-02:** Keep claim verification two-step semantics: internal claim consistency plus persisted-store rebinding.
- **D-03:** Keep tx digest semantics explicit: tx_digest_hex is routing/storage identity, never a substitute for full payload verification.
- **D-04:** Keep public spend contract helper positioned as statement-envelope validation only, not as a complete tx verifier.

### Rename Scope and Priority

- **D-05:** Execute low-risk rename tranche first (tests/helpers and semantically stale suffix names), then medium-risk internal helper renames with local caller updates.
- **D-06:** Preserve trait-locked/format-locked names as-is; do not mechanically rename locked signatures.
- **D-07:** Apply synchronized renames for known coupled pairs/groups identified by the audit (for internal consistency).
- **D-08:** Keep vendored path excluded: crates/z00z_crypto/tari is read-only and out of rename scope.

### Naming Semantics Contract

- **D-09:** Replace suffix-only ambiguity (_with/_from/_at/_not/... without semantic payload) by intent-explicit names that encode operation meaning.
- **D-10:** Preserve Z00Z naming constraints while renaming: test_ prefix for tests and maximum identifier word-count policy.
- **D-11:** Do not widen rename intent into behavior refactors; each rename must stay a semantics-preserving symbol update.

### Validation and Safety Gates

- **D-12:** For each renamed symbol, update all local call sites and tests in the same change slice to avoid partial drift.
- **D-13:** Maintain negative-path security tests that guard tamper/replay/digest drift boundaries referenced by the threat audit.
- **D-14:** Use workspace verification gates after rename slices (format/lint/tests) before planning closure.

### the agent's Discretion

- Grouping concrete rename patches into batches.
- Choosing exact per-batch verification command sequence.
- Selecting whether to stage low-risk renames by crate or by semantic topic.

## Specific Ideas

- The threat audit marks API seams where caller misuse can convert helper-level checks into admission bypasses; naming updates must reduce this ambiguity rather than hide it.
- The suffix audit is semantic, not mechanical: keep names that are trait/format constrained, and rename only where the new name improves boundary clarity.
- The signature audit provides a large candidate inventory; planning should prioritize high-signal security and verifier-adjacent names before broad cosmetic cleanup.

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Inputs

- .planning/ROADMAP.md — Phase 041 scope anchor, dependency, and current phase boundary.
- .planning/phases/041-renaming-fixes/041-full-threats-audit.md — Threat-ranked verifier boundary seams and misuse hazards that naming must not regress.
- .planning/phases/041-renaming-fixes/041-rename-signature-audit.md — Workspace-wide signature/test-name candidate inventory for normalization.
- .planning/phases/041-renaming-fixes/041-rename-suffix-audit.md — Suffix-focused semantic rename plan with keep/rename/trait-locked split.

### Repository Rules

- .github/copilot-instructions.md — Z00Z naming constraints, read-only vendor rule, and verification expectations.
- .github/requirements/Z00Z_DESIGN_FOUNDATION.md — Architectural boundary and correctness constraints to preserve during renaming.

## Existing Code Insights

### Reusable Assets

- Threat-audit evidence links point directly to verifier and simulator gate files; these form the source-of-truth list for security-sensitive rename review.
- Signature and suffix audit tables already provide candidate old/new pairs and rationale, reducing discovery effort for planning.

### Established Patterns

- Admission paths are layered: local structure checks, public spend envelope checks, and full-package verification in canonical runtime/simulator seams.
- Claim validation pattern requires persisted-store contract rebinding after raw verifier consistency checks.
- Naming policy in this repository is strict for test prefixes and identifier size limits.

### Integration Points

- Verifier-related naming and helper boundaries in wallets and simulator gates.
- Test/helper naming normalization across core, crypto, simulator, storage, and network crates.
- Internal utility/helper renames where call sites are local and can be updated atomically.

## Deferred Ideas

- Broad non-security stylistic renames outside the audited candidate set.
- Any API surface redesign that changes behavior contracts rather than clarifying symbol semantics.
- New feature scope not tied to Phase 041 rename-fix goals.

---

## TODO Tasks

Five tasks implement the threat-anchor safety gate for Phase 041.
T-01 through T-04 must all complete before T-05 is started.

### T-01 — Confirm and anchor digest-auth-drift test coverage

**Threat:** T-3 | **Decisions:** D-03, D-13

**Files:**

- `crates/z00z_wallets/tests/test_spend_statement.rs`

**Work:** Add the following anchor comment above each of the three existing tests (do NOT change test logic):

```rust
// Threat T-3 anchor: digest is routing identity; auth drift must not change it.
```

Tests to annotate:

- `test_package_digest_ignores_auth` (approx. line 211)
- `test_digest_ignores_auth_hex` (approx. line 295)
- `test_digest_ignores_spend_blobs` (approx. line 326)

**Exit:** All three tests pass; no logic changes.

---

### T-02 — Add local-verifier bypass boundary test

**Threat:** T-1 (CRITICAL) | **Decisions:** D-01, D-13

**Files:**

- `crates/z00z_wallets/src/core/tx/test_tx_verifier_suite.rs`

**Work:** Add test `test_local_verifier_alone_is_not_final_admission`:

- Build a tx that passes `TxVerifierImpl::verify` but fails `verify_full_tx_package` (absent/stale spend proof).
- Assert `TxVerifierImpl::verify` → `Ok`; `verify_full_tx_package` → error/non-admissible.
- Add anchor comment:

```rust
// Threat T-1 anchor: local verifier is not final admission; verify_full_tx_package is the canonical gate.
```

**Exit:** New test passes; all pre-existing tests remain green.

---

### T-03 — Add raw claim verifier rebind-gap test

**Threat:** T-2 (HIGH) | **Decisions:** D-02, D-13

**Files:**

- `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs` (+ optionally `claim_tx.rs`)

**Work:** Add test `test_raw_claim_verifier_alone_is_not_final_admission`:

- Construct claim package where internal proof/statement is self-consistent but root is synthetic (not from live store).
- Assert `ClaimTxVerifierImpl::verify` → `Ok` for the synthetic package.
- Document that without `claim_source_contract_for_item` rebinding no persisted-store validation occurs.
- Add anchor comment:

```rust
// Threat T-2 anchor: raw verifier passes internal consistency only; store rebinding via claim_source_contract_for_item is the mandatory second step for final admission.
```

**Exit:** New test passes; pre-existing tests remain green.

---

### T-04 — Add partial spend contract insufficiency test

**Threat:** T-4 (MEDIUM) | **Decisions:** D-04, D-13

**Files:**

- `crates/z00z_wallets/src/core/tx/test_tx_verifier_suite.rs`
- `crates/z00z_wallets/src/core/tx/spend_verification.rs`
- `crates/z00z_wallets/src/core/tx/tx_verifier.rs`

**Work:** Add test `test_partial_spend_contract_is_not_full_admission`:

- Build tx where the public spend envelope is self-consistent but local structure fails `TxVerifierImpl::verify`.
- Assert `verify_tx_public_spend_contract` → passing; `verify_full_tx_package` → error.
- Add anchor comment:

```rust
// Threat T-4 anchor: verify_tx_public_spend_contract checks statement-envelope scope only; verify_full_tx_package is required for complete admission.
```

- Add doc comment on `verify_tx_public_spend_contract` in `spend_verification.rs`:
  *"Validates the spend statement envelope only. Caller must also run `TxVerifierImpl::verify` for full admission; see `verify_full_tx_package`."*
- Add doc comment on call site / re-export in `tx_verifier.rs` confirming it is a sub-check.

**Exit:** New test + `test_public_spend_valid_package` pass; pre-existing suite remains green.
`cargo clippy -p z00z_wallets` emits zero new warnings from touched files.

---

### T-05 — Threat-seam rename safety gate

**Threats:** All (T-1 through T-4) | **Decisions:** D-12, D-13, D-14

**Pre-condition:** T-01 through T-04 must all be complete.

**Canonical call sites to verify:**

- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs` line 187 — `verify_full_tx_package` RPC entry
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_validation_gates.rs` line 186 — `verify_full_tx_package` simulator entry

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

**Implementation checklist:**

- For each symbol below, confirm it appears in the threat-anchor tests added by T-01 through T-04, and that no Phase 041 rename changed it at the admission boundary without also updating the test:
  - `verify_full_tx_package`
  - `TxVerifierImpl::verify`
  - `ClaimTxVerifierImpl::verify` (or current raw verify surface)
  - `verify_tx_public_spend_contract`
  - `build_tx_package_digest`
  - `claim_source_contract_for_item`
- For each symbol: run `grep` to confirm threat-anchor tests reference the live symbol name (not a stale pre-rename name).
- Run `cargo test -p z00z_wallets && cargo test -p z00z_simulator` — full suites green.
- Run `cargo clippy -p z00z_wallets -p z00z_simulator` — zero new warnings from threat-seam files.
- Run `cargo fmt --check -p z00z_wallets -p z00z_simulator` — clean output.

**Exit:** Full wallet and simulator test suites green. All four threat-anchor tests exist with anchoring comments referencing live symbol names. `cargo clippy` and `cargo fmt --check` emit no new errors on touched files. Phase 041 rename slices may proceed after this gate passes.

---

## Validation Matrix

| Task | Threat | Mechanism | Status |
| --- | --- | --- | --- |
| T-01 | T-3 — digest-auth drift | Anchor comments on 3 existing tests | To do |
| T-02 | T-1 — local-verifier bypass (CRITICAL) | New negative-path test | To do |
| T-03 | T-2 — claim rebind gap (HIGH) | New negative-path test | To do |
| T-04 | T-4 — partial spend contract (MEDIUM) | New negative-path test + doc comments | To do |
| T-05 | All | Symbol grep + full suite verification | Blocked on T-01..T-04 |

---

*Phase: 041-renaming-fixes*
*Context gathered: 2026-04-29*
