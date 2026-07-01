# Phase 032 Full Audit

## 🔔 Audit Run — 2026-04-05 00:00:00

### 📌 Audit Setup

- Phase directory: `.planning/phases/032-crypto-audit-scenario-1`
- Derived FULL-AUDIT path: `.planning/phases/032-crypto-audit-scenario-1/032-FULL-AUDIT.md`
- Mandatory context read:
  - `.github/requirements/Z00Z_DESIGN_FOUNDATION.md`
  - `.github/copilot-instructions.md`
  - `.planning/phases/032-crypto-audit-scenario-1/032-CONTEXT.md`
  - `.planning/phases/032-crypto-audit-scenario-1/032-SEMANTIC-FREEZE.md`
  - `.planning/phases/032-crypto-audit-scenario-1/032-HONEST-CLOSEOUT.md`
  - `.planning/phases/032-crypto-audit-scenario-1/032-VALIDATION.md`
  - `.planning/phases/032-crypto-audit-scenario-1/032-VERIFICATION.md`
  - `.planning/phases/032-crypto-audit-scenario-1/032-03-SUMMARY.md`
  - `.planning/phases/032-crypto-audit-scenario-1/032-04-SUMMARY.md`
  - `.planning/phases/032-crypto-audit-scenario-1/032-07-SUMMARY.md`
- Execution mode: all four mandatory audit passes executed via manual fallback because the named skills were not directly invokable as tools in the original audit environment.
- Historical normalization note: the original artifact stored day-only headings. This reformatted report preserves the same chronology and conclusions while normalizing section timestamps to canonical `YYYY-MM-DD HH:MM:SS` form with `00:00:00` as the explicit day-level time anchor.

> [!IMPORTANT]
> Final in-scope crate list: `z00z_crypto`, `z00z_core`, `z00z_storage`, `z00z_wallets`, `z00z_simulator`.

- Explicitly excluded from direct audit scope because Phase 032 artifacts do not make them primary remediation targets: `z00z_networks_rpc`

### 🎯 Scope And Source Of Truth

- `032-CONTEXT.md` defines the phase threat model, trust boundaries, waves, and the active blockers around claim trust, spend truthfulness, and checkpoint truthfulness.
- `032-SEMANTIC-FREEZE.md` is the canonical semantic contract for `leaf_ad_id`, `s_out`, request/card binding, `tag16`, and forbidden overclaims.
- `032-03-SUMMARY.md` records that the original `PH32-CLAIM-TRUST` wording remains open until claim-source proofs are anchored in persisted storage-backed membership state or the requirement is narrowed.
- `032-04-SUMMARY.md` records that current-stack spend hardening landed, but the original `PH32-SPEND` wording remains open because the regular spend contract still lacks nullifier semantics.
- `032-07-SUMMARY.md` records the honest checkpoint truthfulness outcome and explicitly states that `PH32-SPEND` and `PH32-CLAIM-TRUST` were previously closed more broadly than the code proved.
- `032-HONEST-CLOSEOUT.md`, `032-VALIDATION.md`, and `032-VERIFICATION.md` provide the phase-local truth surface for what is actually closed versus still blocked.
- `032-TODO.md`, `032-TEST-SPEC.md`, and `032-UAT.md` remain supporting phase-local planning and verification artifacts, but they do not override the semantic-freeze and honest-closeout surfaces.

### 🧪 Verification Model

#### 👣 Critical User Journeys

- Claim package publication and consumption in Scenario 1.
  - Why it matters: claim paths must stay deterministic, fail closed on malformed inputs, and not overclaim persisted trust.
  - Evidence path: `crates/z00z_simulator/src/claim_pkg_store.rs`, `crates/z00z_simulator/src/claim_pkg_consumer.rs`, `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs`, `032-03-SUMMARY.md`.
- Accepted spend flow from persisted package to wallet-visible public verifier.
  - Why it matters: Phase 032 must prove honest current-stack spend authorization without silently promoting it to a stronger public-proof claim than the code supports.
  - Evidence path: `crates/z00z_wallets/src/core/tx/spend_verification.rs`, `crates/z00z_wallets/src/core/tx/witness_gate.rs`, `crates/z00z_simulator/tests/test_scenario1_spend_gate.rs`, `032-04-SUMMARY.md`.
- Checkpoint promotion and final artifact acceptance.
  - Why it matters: checkpoint artifacts must not claim stronger proof semantics than the live verifier and compatibility payload path provide.
  - Evidence path: `crates/z00z_storage/src/checkpoint/build.rs`, `crates/z00z_storage/src/checkpoint/codec.rs`, `crates/z00z_storage/src/checkpoint/artifact_proof_draft.rs`, `crates/z00z_storage/src/checkpoint/artifact_final.rs`, `032-07-SUMMARY.md`.

#### 🔁 State Transitions

- Store item to claim-source root/proof derivation.
  - Required postcondition: helper outputs stay deterministic and coherent.
  - Limitation: proof remains helper-owned unless derived from persisted membership state.
- Persisted tx package to accepted spend verification.
  - Required postcondition: proof/auth bytes, range proof checks, balance checks, receiver card verification, and statement framing must all pass.
  - Limitation: regular spend contract still does not bind nullifier semantics.
- Checkpoint draft to promoted checkpoint proof artifact.
  - Required postcondition: draft/public-input statement remains bound to the tx package and rejects empty/tampered compatibility payloads.
  - Limitation: this is still package-coupled compatibility data, not a standalone proof backend.

#### 🔐 Proof Paths

- Claim-source contract helper path.
  - Statement that must hold: the returned root/proof pair matches the item inserted into the helper-owned store.
  - Evidence path: `crates/z00z_storage/src/assets/store_internal/store_query.rs`, claim-source tests in `z00z_storage`.
- Public spend contract verification path.
  - Statement that must hold: persisted `proof.spend` and `auth.spend` verify against the canonical public spend statement.
  - Evidence path: `crates/z00z_wallets/src/core/tx/spend_verification.rs`, `crates/z00z_wallets/tests/test_spend_witness_gate.rs`, `crates/z00z_simulator/tests/test_scenario1_spend_gate.rs`.
- Checkpoint package verification path.
  - Statement that must hold: tx proof bytes, input refs, and outputs remain bound to the accepted package contract.
  - Evidence path: `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs`, `crates/z00z_simulator/src/scenario_1/stage_6_utils/exec_input_builder.rs`, checkpoint acceptance tests in `z00z_simulator`.

#### 🚫 Failure Paths

- Malformed or mismatched claim nullifier data must reject.
  - Expected behavior: fail closed with `claim_nullifier_invalid`-class rejection.
  - Evidence path: `crates/z00z_wallets/src/core/tx/claim_tx.rs`, `crates/z00z_wallets/src/core/tx/test_claim_tx.rs`.
- Missing or tampered spend proof/auth data must reject.
  - Expected behavior: `verify_tx_public_spend_contract(...)` fails on missing proof/auth, bad versions, bad prev root, tampered leaf/ad hash, or duplicate output leaf ids.
  - Evidence path: `crates/z00z_wallets/src/core/tx/spend_verification.rs`, wallet and simulator spend-gate tests.
- Tampered or replayed checkpoint package data must reject.
  - Expected behavior: checkpoint acceptance rejects tampered exec rows, tampered tx-proof bytes, replayed link rows, and empty checkpoint proof bytes.
  - Evidence path: `crates/z00z_simulator/tests/test_checkpoint_acceptance.rs`, `crates/z00z_storage/src/checkpoint/codec.rs`, `crates/z00z_storage/src/checkpoint/artifact_final.rs`.

### 📊 Findings Summary

| Severity | Count | Meaning |
| --- | ---: | --- |
| 🔴 CRITICAL | 0 | Immediate correctness, security, or truthfulness failure |
| 🟠 HIGH | 3 | Material closure gap on required phase behavior |
| 🟡 MEDIUM | 0 | No distinct medium-severity issue remained after consolidating overlap into the three high blockers |
| 🔵 LOW | 2 | Narrow clarity or honesty-fence issue resolved locally |
| ⚪ INFO | 8 | Positive confirmations and no-finding pass results across in-scope crates |

The audit proved that Phase 032 contains real current-stack hardening for claim-path determinism, spend contract checks, and fail-closed checkpoint handling. It also proved that three broader closure claims remain open: persisted claim continuity is still synthetic-helper scoped, the regular spend contract still lacks nullifier semantics, and checkpoint acceptance remains package-coupled compatibility proof handling rather than a standalone authoritative backend. All code changes applied during the audit were comment-only honesty fences that clarified these boundaries without changing runtime behavior.

### 🔍 Audit Pass Results

#### 🧬 `z00z_crypto`

- **Audit Pass:** `crypto-architect`
  - **Status:** `manual fallback`
  - **Files inspected:**
    - `crates/z00z_crypto/src/claim/statement.rs`
    - `crates/z00z_crypto/src/claim/v2.rs`
  - **Positive confirmations:** `GenesisClaimStatement` and `ClaimStmtV2` both carry `source_root` and `nullifier`, so the crypto-layer statement format already exposes the fields needed for claim anti-replay and root binding.
  - **Exact fixes required:** none at the crypto wrapper layer.
- **Audit Pass:** `security-audit`
  - **Status:** `manual fallback`
  - **Files inspected:** same claim statement and proof surfaces.
  - **Positive confirmations:** no Phase 032-specific secret dump, unsafe-code, or silent parsing fallback was found in the inspected claim statement or proof files.
  - **Exact fixes required:** none.
- **Audit Pass:** `spec-to-code-compliance`
  - **Status:** `manual fallback`
  - **Positive confirmations:** the crate supports the claim root/nullifier statement shape required by Phase 032.
  - **Limit:** the crate does not itself prove persisted storage-backed continuity for `PH32-CLAIM-TRUST`.
  - **Exact fixes required:** none within `z00z_crypto`.
- **Audit Pass:** `z00z-design-foundation-compliance`
  - **Status:** `manual fallback`
  - **Positive confirmations:** no direct design-foundation boundary violation was found in the inspected claim files.
  - **Exact fixes required:** none.

#### 🧱 `z00z_core`

- **Audit Pass:** `crypto-architect`
  - **Status:** `manual fallback`
  - **Files inspected:** `crates/z00z_core/src/assets/wire_pkg.rs`
  - **Positive confirmations:** `AssetPkgWire` remains the frozen public DTO and explicitly rejects trusted-only secret material at the human-readable JSON boundary.
  - **Exact fixes required:** none.
- **Audit Pass:** `security-audit`
  - **Status:** `manual fallback`
  - **Positive confirmations:** the DTO boundary enforces a payload size ceiling and documents that secret fields are rejected before JSON parsing.
  - **Exact fixes required:** none.
- **Audit Pass:** `spec-to-code-compliance`
  - **Status:** `manual fallback`
  - **Positive confirmations:** `z00z_core` remains a transport surface for Phase 032 and does not itself own the claim/spend/checkpoint acceptance logic under review.
  - **Exact fixes required:** none.
- **Audit Pass:** `z00z-design-foundation-compliance`
  - **Status:** `manual fallback`
  - **Positive confirmations:** no direct design-foundation issue was found in the inspected DTO file.
  - **Exact fixes required:** none.

#### 🗃️ `z00z_storage`

- **Audit Pass:** `crypto-architect`
  - **Status:** `manual fallback`
  - **Files inspected:**
    - `crates/z00z_storage/src/assets/store_internal/store_query.rs`
    - `crates/z00z_storage/src/assets/store_internal/store_rows.rs`
    - `crates/z00z_storage/src/checkpoint/artifact_proof_draft.rs`
    - `crates/z00z_storage/src/checkpoint/artifact_final.rs`
    - `crates/z00z_storage/src/checkpoint/build.rs`
  - **Disposition:** material issues found.

#### 🟠 Claim Source Continuity Remains Synthetic 🟠

**Location:** `crates/z00z_storage/src/assets/store_internal/store_query.rs:23`

**Issue:**

```rust
let mut store = Self::build(super::RedbBackend::off());
store.put_item(item.clone())?;
let claim_root = store.claim_source_root()?;
let claim_proof = store.claim_source_proof(&item.path())?;
```

**Why This is Critical:**
The helper deterministically proves a one-item off-store tree, not persisted storage-backed membership continuity. That keeps `PH32-CLAIM-TRUST` truthful only as a helper-owned contract and blocks broader closure language.

**Recommendation:**

```rust
// Replace the off-store helper reconstruction with a persisted-membership
// witness source, or formally narrow PH32-CLAIM-TRUST to the helper-owned
// boundary and keep the phase marked partial.
```

**Severity:** 🟠 High
**Category:** Functionality
**Proof Status:** Partial Evidence
**Verification:** VERIFIED

#### 🟠 Checkpoint Proof Acceptance Is Compatibility-Payload Only 🟠

**Location:** `crates/z00z_storage/src/checkpoint/build.rs:220`, `crates/z00z_storage/src/checkpoint/codec.rs:64`, `crates/z00z_storage/src/checkpoint/artifact_final.rs:44`

**Issue:**

```rust
pub trait TxProofVerifier {
    fn verify_tx(&self, tx: &TxPkgSum) -> Result<(), TxProofError>;
}

if artifact.cp_proof().is_empty() {
    return Err(CheckpointError::EmptyProof);
}
```

**Why This is Critical:**
The storage path truthfully rejects empty proof payloads and requires an injected verifier, but it does not itself validate a stronger standalone checkpoint proof backend. The closure claim therefore stays package-coupled and compatibility-payload based.

**Recommendation:**

```rust
// Introduce an authoritative checkpoint-proof backend and bind finalized
// artifact acceptance to that backend instead of non-empty compatibility bytes
// plus an externally supplied verifier contract.
```

**Severity:** 🟠 High
**Category:** Security
**Proof Status:** Partial Evidence
**Verification:** VERIFIED

- **Audit Pass:** `security-audit`
  - **Status:** `manual fallback`
  - **Files inspected:**
    - `crates/z00z_storage/src/checkpoint/artifact_proof_draft.rs`
    - `crates/z00z_storage/src/checkpoint/artifact_final.rs`
    - `crates/z00z_storage/src/checkpoint/codec.rs`
    - `crates/z00z_storage/src/checkpoint/build.rs`
  - **Positive confirmations:** finalize/load paths reject empty `cp_proof` payloads.
  - **Residual truth:** `build.rs` explicitly documents `TxProofVerifier` and `SpentIndex` as external trust boundaries; that is honest, but it confirms the final artifact path still depends on caller-supplied trust rather than a self-contained verifier.
  - **Exact fixes required:** no safe scope-local cryptographic upgrade was implemented during this audit.
- **Audit Pass:** `spec-to-code-compliance`
  - **Status:** `manual fallback`
  - **Positive confirmations:** live code matches the reopened Phase 032 truth. Claim helper consistency exists, but persisted continuity is not closed; checkpoint artifact integrity is stronger than placeholder acceptance but still not authoritative proof-backend validation.
  - **Exact fixes required:** none beyond blocked-gap tracking.
- **Audit Pass:** `z00z-design-foundation-compliance`
  - **Status:** `manual fallback`
  - **Positive confirmations:** compatibility payloads and canonical statement-owned bindings are honestly separated in the inspected storage files.
  - **Exact fixes required:** none.

#### 👛 `z00z_wallets`

- **Audit Pass:** `crypto-architect`
  - **Status:** `manual fallback`
  - **Files inspected:**
    - `crates/z00z_wallets/src/core/tx/spend_verification.rs`
    - `crates/z00z_wallets/src/core/tx/witness_gate.rs`
    - `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs`
    - `crates/z00z_wallets/src/core/claim/nullifier.rs`
    - `crates/z00z_wallets/src/core/tx/state_checkpoint.rs`
  - **Disposition:** material issues found.

#### 🟠 Regular Spend Contract Still Lacks Nullifier Semantics 🟠

**Location:** `crates/z00z_wallets/src/core/tx/spend_verification.rs:339`, `crates/z00z_wallets/src/core/tx/witness_gate.rs:267`, `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs:23`

**Issue:**

```rust
pub fn verify_tx_public_spend_contract(
    chain_id: u32,
    tx_version: u8,
    tx: &TxWire,
) -> Result<(), SpendPublicErr> {
    let proof = tx.proof.spend.as_ref().ok_or(SpendPublicErr::MissingProof)?;
    let auth = tx.auth.spend.as_ref().ok_or(SpendPublicErr::MissingAuth)?;
    // ... range, balance, card, and statement checks ...
}
```

**Why This is Critical:**
Current-stack spend verification is real and fail closed, but the regular spend statement still carries no nullifier field while the claim path does. That means the original `PH32-SPEND` wording remains broader than the delivered public spend contract.

**Recommendation:**

```rust
// Extend the regular spend statement, proof, and persisted wire contract to
// bind nullifier semantics explicitly, or formally narrow PH32-SPEND to the
// current public-spend contract without nullifier closure language.
```

**Severity:** 🟠 High
**Category:** Security
**Proof Status:** Partial Evidence
**Verification:** VERIFIED

#### 🔵 Wallet-Local Checkpoint Helper Could Be Misread As Finalized Artifact Semantics 🔵

**Location:** `crates/z00z_wallets/src/core/tx/state_checkpoint.rs:29`

**Issue:**

```rust
/// Opaque checkpoint proof bytes when later storage-backed finalization
/// attaches them. Local proofless wallet helpers may legitimately leave this
/// empty and must not be treated as finalized checkpoint artifacts.
pub cp_proof: Vec<u8>,
```

**Why This is Critical:**
This was a clarity-only issue, but it was easy to misread the wallet-local helper as if it represented the finalized checkpoint artifact boundary.

**Recommendation:**

```rust
// Keep an explicit honesty-fence comment that local wallet checkpoint helpers
// may carry empty cp_proof bytes and are not finalized storage artifacts.
```

**Severity:** 🔵 Low
**Category:** Code Quality
**Proof Status:** Full Evidence
**Verification:** VERIFIED

- **Audit Pass:** `security-audit`
  - **Status:** `manual fallback`
  - **Files inspected:**
    - `crates/z00z_wallets/src/core/tx/spend_verification.rs`
    - `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs`
    - `crates/z00z_wallets/src/core/claim/nullifier.rs`
  - **Positive confirmations:** claim nullifier derivation is correctly chain-scoped and deterministic.
  - **Residual truth:** the claim verifier is internally self-consistent, yet it still proves claim root coherence against synthetic helper state rather than persisted store continuity.
  - **Exact fixes required:** none safely implementable here without changing the public spend and claim proof contracts.
- **Audit Pass:** `spec-to-code-compliance`
  - **Status:** `manual fallback`
  - **Positive confirmations:** wallet code matches the current truthful Phase 032 status. Strong current-stack spend proof/auth enforcement exists, but `PH32-SPEND` and `PH32-CLAIM-TRUST` are not fully closed.
  - **Exact fixes required:** no semantic widening.
- **Audit Pass:** `z00z-design-foundation-compliance`
  - **Status:** `manual fallback`
  - **Positive confirmations:** the comment clarification restored honest boundary language without changing runtime or widening API surface.
  - **Exact fixes required:** honesty-fence comment only.

#### 🧪 `z00z_simulator`

- **Audit Pass:** `crypto-architect`
  - **Status:** `manual fallback`
  - **Files inspected:**
    - `crates/z00z_simulator/src/claim_pkg_consumer.rs`
    - `crates/z00z_simulator/src/claim_pkg_store.rs`
    - `crates/z00z_simulator/src/scenario_1/stage_3_utils/claim_pkg.rs`
    - `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_flow.rs`
    - `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs`
    - `crates/z00z_simulator/src/scenario_1/stage_6_utils/exec_input_builder.rs`
    - `crates/z00z_simulator/src/scenario_1/stage_12.rs`
  - **Disposition:** material issues found.

#### 🟠 Simulator Checkpoint Acceptance Remains Package-Coupled

**Location:** `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs:463`, `crates/z00z_simulator/src/scenario_1/stage_6_utils/exec_input_builder.rs:68`

**Issue:**

```rust
// This is a current-stack package-coupled verifier: it proves that the
// checkpoint draft is bound to the persisted tx package contract, not
// that a stronger standalone checkpoint proof backend was validated.
if tx.tx_proof != self.tx_proof {
    return Err(TxProofError::Invalid);
}
```

```rust
// The current stack persists the tx proof bytes as the compatibility payload
// for checkpoint promotion. This binds Stage 8 to the package contract, but
// it is not a standalone checkpoint-proof backend.
let bytes = JsonCodec.serialize(&pkg.tx.proof)?;
```

**Why This is Critical:**
The simulator proves package coherence and fail-closed acceptance, but it does not promote checkpoint truthfulness to an authoritative proof backend. That keeps the checkpoint closure claim partial.

**Recommendation:**

```rust
// Preserve the package-coupled verifier as an honesty-fenced compatibility
// seam until a standalone checkpoint-proof backend and authoritative verifier
// are implemented and wired into promotion/finalization.
```

**Severity:** 🟠 High
**Category:** Security
**Proof Status:** Partial Evidence
**Verification:** VERIFIED

#### 🔵 Simulator Checkpoint Bridge Needed Explicit Honesty Fences

**Location:** `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs:465`, `crates/z00z_simulator/src/scenario_1/stage_6_utils/exec_input_builder.rs:74`

**Issue:**

```rust
// This is a current-stack package-coupled verifier ...
// ... not a stronger standalone checkpoint proof backend.
```

**Why This is Critical:**
Without explicit wording, current-stack checkpoint success could be misread as proof-backend authoritative rather than package-coupled.

**Recommendation:**

```rust
// Keep comment-level honesty fences in the simulator checkpoint bridge so no
// future audit or report upgrades the semantics beyond package-coupled truth.
```

**Severity:** 🔵 Low
**Category:** Code Quality
**Proof Status:** Full Evidence
**Verification:** VERIFIED

- **Audit Pass:** `security-audit`
  - **Status:** `manual fallback`
  - **Files inspected:**
    - `crates/z00z_simulator/src/claim_pkg_consumer.rs`
    - `crates/z00z_simulator/src/claim_pkg_store.rs`
    - `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs`
    - `crates/z00z_simulator/src/scenario_1/stage_6_utils/exec_input_builder.rs`
  - **Positive confirmations:** claim nullifier replay rejection is present and fail closed in the simulator publish path.
  - **Residual truth:** checkpoint acceptance rejects tampered exec rows, tampered tx-proof bytes, and replayed link rows, but the accepted `cp_proof` payload is still only the serialized tx proof, not a stronger cryptographic checkpoint backend.
  - **Exact fixes required:** no safe scope-local cryptographic upgrade during this audit.
- **Audit Pass:** `spec-to-code-compliance`
  - **Status:** `manual fallback`
  - **Positive confirmations:** simulator code matches the reopened phase truth exactly: claim trust and spend nullifier closure stay partial, and checkpoint acceptance is package-coupled rather than final-proof authoritative.
  - **Exact fixes required:** none beyond honesty fencing.
- **Audit Pass:** `z00z-design-foundation-compliance`
  - **Status:** `manual fallback`
  - **Positive confirmations:** comment-only honesty fences improved clarity without widening semantics.
  - **Exact fixes required:** explicit comments only.

## ⚙️ Fixes Applied — 2026-04-05 00:00:00

- `crates/z00z_wallets/src/core/tx/state_checkpoint.rs`
  - Added an honesty-fence comment clarifying that wallet-local helpers may carry empty `cp_proof` bytes and must not be confused with finalized storage checkpoint artifacts.
- `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs`
  - Strengthened the verifier comment to state that current checkpoint acceptance is package-coupled and not a standalone checkpoint proof backend.
- `crates/z00z_simulator/src/scenario_1/stage_6_utils/exec_input_builder.rs`
  - Strengthened the `build_cp_proof(...)` comment to state that the persisted `cp_proof` payload is serialized tx-proof compatibility data, not an authoritative checkpoint-proof backend.
- Runtime and API impact:
  - All applied code edits are comment-only honesty fences.
  - No runtime behavior or public API changed during this audit wave.
- Remaining blocked findings:
  - persisted claim-membership continuity remains open;
  - regular spend nullifier semantics remain open;
  - authoritative checkpoint proof backend remains open.

## ♻️ Re-Audit Results — 2026-04-05 00:00:00

- Release-mode validation commands executed:
  - `cargo test -p z00z_storage --release claim_source_proof -- --nocapture`
  - `cargo test -p z00z_wallets --release spend_contract -- --nocapture`
  - `cargo test -p z00z_simulator --release claim_pkg_crypto -- --nocapture`
  - `cargo test -p z00z_simulator --release scenario_1_spend_gate -- --nocapture`
  - `cargo test -p z00z_simulator --release checkpoint_acceptance -- --nocapture`
- Validation summary:
  - `z00z_storage` claim-source proof checks passed, confirming helper-owned root/proof determinism without proving persisted continuity.
  - `z00z_wallets` spend public-contract tests passed, confirming current-stack proof/auth enforcement while leaving nullifier semantics outside the regular spend statement.
  - `z00z_simulator` claim package tests passed, confirming fail-closed authority-anchor, storage-proof, and signature rejection paths.
  - `z00z_simulator` Scenario 1 spend-gate tests passed, confirming that simulator admission follows the wallet public spend verifier.
  - `z00z_simulator` checkpoint acceptance tests passed, confirming tamper rejection and replay rejection on the current package-coupled path.

| Area | Re-Audit Status | Evidence | Remaining Disposition |
| --- | --- | --- | --- |
| Claim helper determinism | VERIFIED | `z00z_storage` claim-source proof tests | Still blocked on persisted continuity |
| Public spend contract | VERIFIED | wallet + simulator spend-gate tests | Still blocked on nullifier semantics |
| Checkpoint acceptance | VERIFIED | simulator checkpoint acceptance tests | Still blocked on standalone proof backend |
| Honesty-fence comments | VERIFIED | source inspection of touched files | Closed locally |

- Re-audit conclusion:
  - The codebase is now more explicit about checkpoint trust boundaries.
  - No honest basis was found to close `PH32-SPEND` or `PH32-CLAIM-TRUST`.
  - Remaining gaps stay blocked on real spend-nullifier contract widening, persisted claim-membership continuity, and an authoritative checkpoint proof backend.

## ✅ Doublecheck Results — 2026-04-05 00:00:00

- Direct audit-file review:
  - The audit records setup, crate-by-crate findings, concrete fixes applied, and post-fix validation results.
- Code-change review:
  - All applied code edits are comment-only honesty fences. No runtime behavior or public API changed during this audit wave.
- Residual risk review:
  - storage claim proof remains synthetic-helper scoped;
  - spend verification remains nullifier-incomplete at the regular public-contract boundary;
  - checkpoint acceptance remains package-coupled rather than proof-backend authoritative.
- Final disposition from the original doublecheck pass:
  - evidence gathering and safe remediation are complete for this phase;
  - Phase 032 must remain recorded as partially complete until the blocked semantic gaps are implemented and re-verified.

> [!CAUTION]
> Doublecheck validated both the code conclusions and the truthfulness of the audit narrative. It did not upgrade the blocked semantics beyond what repository-backed evidence proved.

- External addendum — 2026-04-06 00:00:00:
  - A separate Doublecheck pass found no unsupported or overstated claims in the four core audit conclusions.
  - Confirmed conclusions:
    - `PH32-SPEND` remains partial/open.
    - `PH32-CLAIM-TRUST` remains partial/open.
    - Current checkpoint acceptance remains package-coupled and compatibility-payload based rather than a standalone checkpoint proof backend.
    - The code edits in this audit wave are comment-only honesty fences with no runtime behavior change.
  - Caveat preserved:
    - The external pass did not independently replay the previously recorded test commands in that pass; it verified the audit conclusions against live repository evidence and the current worktree.

## 🧾 Exact Fixes Required Summary

| Q | Title | Proof Status | Verification | Severity | Missing Evidence Or Blocker | Gap Closure Path |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | `PH32-CLAIM-TRUST` closure versus helper-owned claim-source determinism | Partial Evidence | VERIFIED | 🟠 HIGH | `claim_source_contract_for_item(...)` still derives the root/proof from an off-store synthetic helper tree instead of persisted membership state | Bind claim-source proofs to persisted storage-backed membership state, or formally narrow the requirement to the helper-owned boundary |
| 2 | `PH32-SPEND` closure versus current public spend contract | Partial Evidence | VERIFIED | 🟠 HIGH | The regular public spend contract verifies real proof/auth data but still carries no nullifier semantics | Extend the spend statement, persisted wire contract, and verifier to bind nullifier semantics, or narrow the requirement honestly |
| 3 | Checkpoint truthfulness versus package-coupled compatibility proof handling | Partial Evidence | VERIFIED | 🟠 HIGH | Finalized checkpoint acceptance still relies on externally supplied verifier trust and compatibility payload bytes instead of a standalone proof backend | Implement an authoritative checkpoint proof backend and bind finalize/load acceptance to that backend |
| 4 | Wallet-local checkpoint helper wording | Full Evidence | VERIFIED | 🔵 LOW | No blocker remains after the honesty-fence comment landed | Preserve the explicit comment and keep wallet-local helpers distinct from finalized artifacts |
| 5 | Simulator checkpoint bridge wording | Full Evidence | VERIFIED | 🔵 LOW | No blocker remains after the honesty-fence comments landed | Preserve explicit package-coupled wording until the real checkpoint backend exists |
| 6 | Current-stack truth versus broader future-proof ambition | Full Evidence | VERIFIED | ⚪ INFO | None | None |

## 🚩 Final Status

> [!NOTE]
> Phase 032 current-stack hardening is real, but the broader original closure claim remains partial.

- Phase disposition: **partial / blocked on three high-severity closure gaps**.
- Exact findings that still prevent full closure:
  - persisted claim-membership continuity is not yet authoritative;
  - regular spend public-contract nullifier semantics are not yet implemented;
  - checkpoint proof handling is still package-coupled and compatibility-payload based rather than proof-backend authoritative.
- Next gap-closure action:
  - implement one of the three blocked semantics end to end and rerun the same audit, re-audit, and doublecheck workflow against the updated code and phase artifacts.
