# Phase 033 Full Audit

## 🔔 Audit Run — 2026-04-09 02:26:20

### 📌 Audit Setup

- Phase directory: `.planning/phases/033-crypto-audit-scenario-2`
- Derived FULL-AUDIT path: `.planning/phases/033-crypto-audit-scenario-2/033-FULL-AUDIT.md`
- Mandatory context read:
  - `.github/requirements/Z00Z_DESIGN_FOUNDATION.md`
  - `.github/copilot-instructions.md`
  - `.planning/phases/033-crypto-audit-scenario-2/033-CONTEXT.md`
  - `.planning/phases/033-crypto-audit-scenario-2/033-SEMANTIC-FREEZE.md`
  - `.planning/phases/033-crypto-audit-scenario-2/033-TODO.md`
  - `.planning/phases/033-crypto-audit-scenario-2/033-UAT.md`
  - `.planning/phases/033-crypto-audit-scenario-2/033-VALIDATION.md`
  - `.planning/phases/033-crypto-audit-scenario-2/033-23-SUMMARY.md`
  - `.planning/phases/033-crypto-audit-scenario-2/033-32FULL-AUDIT.md`
- Audit methodology note: the pass labels below are documentary audit categories. The repository-backed conclusions in this file come from the cited phase artifacts and code seams, not from any claim about tool availability.

> [!IMPORTANT]
> Final in-scope crate list: `z00z_crypto`, `z00z_core`, `z00z_storage`, `z00z_wallets`, `z00z_simulator`.

- Explicitly excluded from direct audit scope because the phase artifacts do not make them primary remediation targets:
  - `z00z_networks_rpc`
  - vendored code under `z00z_crypto/tari/`

### 🎯 Scope And Source Of Truth

- `033-CONTEXT.md` defines the phase boundary and states that Phase 033 is an execution-ready planning and closeout surface for the existing audit backlog rather than a new rescoping pass.
- `033-TODO.md` is the canonical residual ledger and keeps the still-open `Pending` row plus the crossed high-severity rows 63, 64, and 65.
- `033-SEMANTIC-FREEZE.md` remains the canonical semantic contract for `leaf_ad_id`, `s_out`, request or card routing, and forbidden overclaims.
- `033-UAT.md` and `033-VALIDATION.md` prove the executed narrowed matrix, but they do not override the open semantic blockers preserved in `033-TODO.md` and `.planning/REQUIREMENTS.md`.
- `.planning/REQUIREMENTS.md` keeps `PH32-CLAIM-TRUST` and `PH32-SPEND` marked complete only under formal narrowing while the honest reclassification gate remains blocked until the broader original gaps are implemented and re-verified or formally narrowed and re-approved.
- `033-23-SUMMARY.md` proves the latest Phase 033 checkpoint-authority wording guard landed and that Task 65 must stay scoped to the standalone proof-backend gap.
- `033-32FULL-AUDIT.md` is the authoritative source body for the crossed Task 64 and 65 blocker text and must win over title-only inference.

### 🧪 Verification Model

#### 👣 Critical User Journeys

- Claim package emission and claim-source verification continuity.
  - Why it matters: the phase must not overclaim persisted authority when the live seam is still helper-owned.
  - Evidence path: `crates/z00z_storage/src/assets/store_internal/store_query.rs`, `033-TODO.md`, `.planning/REQUIREMENTS.md`.
- Public spend acceptance for Scenario 1 and validator-style flows.
  - Why it matters: the delivered persisted public spend contract is real, but it must not be widened into a finished nullifier-complete theorem.
  - Evidence path: `crates/z00z_wallets/src/core/tx/spend_verification.rs`, `crates/z00z_wallets/tests/test_s5_closure_gate.rs`, `033-VALIDATION.md`.
- Checkpoint finalize and reload acceptance.
  - Why it matters: the code must stay package-coupled and fail closed without being described as a standalone authoritative proof backend.
  - Evidence path: `crates/z00z_wallets/src/core/tx/state_checkpoint.rs`, `crates/z00z_storage/src/assets/store_internal/redb_backend_validate.rs`, `033-23-SUMMARY.md`.
- Stage-surface wording guards that freeze the narrow truth.
  - Why it matters: the phase relies on behavior-plus-wording guards, not only runtime semantics.
  - Evidence path: `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`, `033-UAT.md`, `033-VALIDATION.md`.

#### 🔁 State Transitions

- Store item to claim-source contract tuple.
  - Required postcondition: helper root and proof stay deterministic for the one-item helper-owned contract.
  - Limitation: this does not prove persisted membership continuity.
- Persisted tx package to public spend verification.
  - Required postcondition: proof, auth, root, input, output, and range-proof checks pass on the shipped contract.
  - Limitation: nullifier semantics are still not bound into the regular public spend statement.
- Finalized checkpoint metadata to accepted reload path.
  - Required postcondition: reload stays fail closed on tuple drift and compatibility mismatches.
  - Limitation: acceptance still relies on externally supplied verifier trust and compatibility payload bytes instead of a standalone proof backend.

#### 🔐 Proof Paths

- Helper-owned claim-source reconstruction path.
  - Statement that must hold: the returned root and proof pair matches the off-store helper reconstruction for the inserted item.
  - Evidence path: `crates/z00z_storage/src/assets/store_internal/store_query.rs`.
- Current persisted public spend contract.
  - Statement that must hold: the verifier rejects structurally plausible but semantically incomplete spend artifacts before state mutation.
  - Evidence path: `crates/z00z_wallets/src/core/tx/spend_verification.rs`, `crates/z00z_wallets/tests/test_s5_closure_gate.rs`.
- Package-coupled checkpoint acceptance path.
  - Statement that must hold: finalized checkpoint metadata is rebound to statement, exec identity, and state root rather than accepted as a free-standing opaque carrier.
  - Evidence path: `crates/z00z_wallets/src/core/tx/state_checkpoint.rs`, `crates/z00z_storage/src/assets/store_internal/redb_backend_validate.rs`.

#### 🚫 Failure Paths

- Claim-source continuity must not be described as persisted storage authority when it is produced by an off-store helper.
  - Expected behavior: the audit keeps the seam classified as helper-owned continuity only.
  - Evidence path: `crates/z00z_storage/src/assets/store_internal/store_query.rs`, `033-TODO.md`.
- Spend verification must not imply nullifier-complete closure.
  - Expected behavior: public spend wording stays narrow and the high-severity blocker remains explicit.
  - Evidence path: `crates/z00z_wallets/src/core/tx/spend_verification.rs`, `.planning/REQUIREMENTS.md`, `033-TODO.md`.
- Checkpoint acceptance must not be described as standalone backend authority.
  - Expected behavior: stage-surface and reload guards preserve compatibility-payload and externally supplied verifier wording.
  - Evidence path: `crates/z00z_wallets/src/core/tx/state_checkpoint.rs`, `crates/z00z_storage/src/assets/store_internal/redb_backend_validate.rs`, `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`.

### 📊 Findings Summary

| Severity | Count | Meaning |
| --- | ---: | --- |
| 🔴 CRITICAL | 0 | Immediate correctness, security, or truthfulness failure |
| 🟠 HIGH | 3 | Material closure gaps remain on required phase behavior |
| 🟡 MEDIUM | 0 | No additional medium-severity divergence remained after consolidating evidence into the three high blockers |
| 🔵 LOW | 0 | No narrow crate-local issue justified separate remediation in this run |
| ⚪ INFO | 7 | Positive confirmations and no-finding pass results across in-scope crates |

The audit confirms that Phase 033 has complete artifact coverage for the executed narrowed surface, but it does not have complete implementation closure. The still-open blockers remain the same three semantic gaps already recorded in the phase backlog: helper-owned claim continuity, spend-side nullifier semantics, and standalone checkpoint proof authority. No direct crate-local fix in this execution could close those gaps without widening scope beyond what the phase artifacts support.

### 🔍 Audit Pass Results

#### 🧬 `z00z_crypto`

- **Audit Pass:** `crypto-architect`
  - **Status:** `manual fallback`
  - **Files inspected:**
    - `crates/z00z_crypto/src/claim/statement.rs`
  - **Positive confirmations:** `GenesisClaimStatement` still binds `genesis_root`, `claim_source_asset_id`, `recipient_binding`, and `nullifier` in canonical field order, so the crypto statement layer still exposes the fields needed for the narrowed claim contract.
  - **Exact fixes required:** none at the crypto crate boundary.
- **Audit Pass:** `security-audit`
  - **Status:** `manual fallback`
  - **Files inspected:** same claim statement surface.
  - **Positive confirmations:** no Phase 033-specific secret leak, silent decode fallback, or weakened anti-replay field loss was found in the inspected claim statement path.
  - **Exact fixes required:** none.
- **Audit Pass:** `spec-to-code-compliance`
  - **Status:** `manual fallback`
  - **Positive confirmations:** the crypto statement still supports the narrowed Phase 033 claim-binding contract.
  - **Limit:** the crate does not itself prove persisted storage-backed continuity.
  - **Exact fixes required:** none.
- **Audit Pass:** `z00z-design-foundation-compliance`
  - **Status:** `manual fallback`
  - **Positive confirmations:** no design-foundation violation was found in the inspected claim statement file.
  - **Exact fixes required:** none.

#### 🧱 `z00z_core`

- **Audit Pass:** `crypto-architect`
  - **Status:** `manual fallback`
  - **Files inspected:**
    - `crates/z00z_core/src/assets/wire_pkg.rs`
  - **Positive confirmations:** `AssetPkgWire` remains the frozen external DTO and still rejects trusted-only secret fields at the public JSON boundary.
  - **Exact fixes required:** none.
- **Audit Pass:** `security-audit`
  - **Status:** `manual fallback`
  - **Files inspected:** same DTO surface.
  - **Positive confirmations:** the JSON seam still enforces a bounded payload ceiling and preserves the explicit non-confidential DTO contract.
  - **Exact fixes required:** none.
- **Audit Pass:** `spec-to-code-compliance`
  - **Status:** `manual fallback`
  - **Positive confirmations:** `z00z_core` remains a transport boundary only and does not itself overclaim the helper-owned claim or public spend closure.
  - **Exact fixes required:** none.
- **Audit Pass:** `z00z-design-foundation-compliance`
  - **Status:** `manual fallback`
  - **Positive confirmations:** no design-foundation violation was found in the inspected DTO file.
  - **Exact fixes required:** none.

#### 🗃️ `z00z_storage`

- **Audit Pass:** `crypto-architect`
  - **Status:** `manual fallback`
  - **Files inspected:**
    - `crates/z00z_storage/src/assets/store_internal/store_query.rs`
    - `crates/z00z_storage/src/assets/store_internal/redb_backend_validate.rs`
  - **Disposition:** material issues found.

#### 🟠 Claim Source Continuity Remains Synthetic

**Location:** `crates/z00z_storage/src/assets/store_internal/store_query.rs:28`

**Issue:**

```rust
let mut store = Self::build(super::RedbBackend::off());
store.put_item(item.clone())?;
let claim_root = store.claim_source_root()?;
let claim_proof = store.claim_source_proof(&item.path())?;
```

**Why This is Critical:**
The live claim-source contract is still a helper-owned off-store one-item reconstruction. That is compatible with the narrowed Phase 033 requirement, but it does not close persisted storage-backed membership continuity and therefore keeps the broader original claim-trust theorem open.

**Recommendation:**

```rust
// Replace helper-owned reconstruction with persisted membership evidence,
// or keep the requirement formally narrowed and the closure state partial.
```

**Severity:** 🟠 High
**Category:** Functionality
**Proof Status:** Partial Evidence
**Verification:** VERIFIED

#### 🟠 Regular Spend Contract Still Lacks Nullifier Semantics

**Location:** `crates/z00z_storage/src/assets/store_internal/redb_backend_validate.rs:14`

**Issue:**

```rust
// Finalized checkpoint acceptance still relies on externally supplied verifier trust
// and compatibility payload bytes instead of a standalone proof backend.
```

**Why This is Critical:**
The persisted reload path is fail closed and stronger than raw artifact replay, but it still depends on externally supplied verifier trust and compatibility payload bytes. This keeps the final checkpoint theorem below authoritative standalone backend closure.

**Recommendation:**

```rust
// Introduce an authoritative checkpoint proof backend and bind finalize/load
// acceptance to that backend instead of compatibility payload bytes.
```

**Severity:** 🟠 High
**Category:** Security
**Proof Status:** Partial Evidence
**Verification:** VERIFIED

- **Audit Pass:** `security-audit`
  - **Status:** `manual fallback`
  - **Files inspected:** same storage surfaces.
  - **Positive confirmations:** the persisted RedB path remains fail closed on tuple drift, mixed-id era mismatch, and proof-byte mismatch.
  - **Residual truth:** fail-closed reload does not upgrade the path into standalone backend authority.
  - **Exact fixes required:** none beyond the blocked high-severity gap above.
- **Audit Pass:** `spec-to-code-compliance`
  - **Status:** `manual fallback`
  - **Positive confirmations:** storage code matches the narrowed helper-owned and package-coupled wording frozen by the phase artifacts.
  - **Exact fixes required:** none beyond the blocked high-severity gaps above.
- **Audit Pass:** `z00z-design-foundation-compliance`
  - **Status:** `manual fallback`
  - **Positive confirmations:** the inspected storage files use project codec abstractions and showed no vendor-boundary or one-source-of-truth violation.
  - **Exact fixes required:** none.

#### 👛 `z00z_wallets`

- **Audit Pass:** `crypto-architect`
  - **Status:** `manual fallback`
  - **Files inspected:**
    - `crates/z00z_wallets/src/core/tx/spend_verification.rs`
    - `crates/z00z_wallets/src/core/tx/state_checkpoint.rs`
  - **Disposition:** material issues found.

#### 🟠 Checkpoint Proof Acceptance Is Compatibility-Payload Only

**Location:** `crates/z00z_wallets/src/core/tx/spend_verification.rs:334`

**Issue:**

```rust
/// Current-stack spend verification is real and fail closed, but the regular public spend contract still carries no nullifier semantics.
/// The exact still-open spend-statement element is the missing nullifier field / nullifier semantics surface.
```

**Why This is Critical:**
The public spend verifier is live and fail closed for the shipped statement, but the exact still-open spend element remains nullifier semantics. That blocks honest full closure of the broader original spend theorem.

**Recommendation:**

```rust
// Extend the spend statement, persisted wire contract, and verifier to bind
// nullifier semantics, or keep the requirement formally narrowed.
```

**Severity:** 🟠 High
**Category:** Functionality
**Proof Status:** Partial Evidence
**Verification:** VERIFIED

- **Audit Pass:** `security-audit`
  - **Status:** `manual fallback`
  - **Files inspected:** same wallet surfaces.
  - **Positive confirmations:** spend verification still fails closed on missing proof or auth, version drift, bad previous root, and malformed input/output bindings.
  - **Residual truth:** those guards do not yet imply nullifier-complete public spend closure.
  - **Exact fixes required:** none beyond the blocked high-severity gap above.
- **Audit Pass:** `spec-to-code-compliance`
  - **Status:** `manual fallback`
  - **Positive confirmations:** wallet code matches the narrowed wording preserved in the phase artifacts and requirements.
  - **Exact fixes required:** none beyond the blocked high-severity gap above.
- **Audit Pass:** `z00z-design-foundation-compliance`
  - **Status:** `manual fallback`
  - **Positive confirmations:** no design-foundation violation was found in the inspected wallet files.
  - **Exact fixes required:** none.

#### 🧪 `z00z_simulator`

- **Audit Pass:** `crypto-architect`
  - **Status:** `manual fallback`
  - **Files inspected:**
    - `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
    - phase summaries referencing `stage_11.rs` and `stage_12.rs`
  - **Positive confirmations:** the stage-surface suite still keeps claim, request-bound privacy, validator-boundary, Task 64, Task 65, and receiver-identity wording scoped to the narrowed phase truth.
  - **Exact fixes required:** none.
- **Audit Pass:** `security-audit`
  - **Status:** `manual fallback`
  - **Files inspected:** same stage-surface guard suite.
  - **Positive confirmations:** the release-mode suite continues to act as the anti-overclaim and wording-regression fence for the live Scenario 1 surfaces.
  - **Exact fixes required:** none.
- **Audit Pass:** `spec-to-code-compliance`
  - **Status:** `manual fallback`
  - **Positive confirmations:** the simulator guards match the narrowed documentation contract rather than asserting broader trustlessness.
  - **Exact fixes required:** none.
- **Audit Pass:** `z00z-design-foundation-compliance`
  - **Status:** `manual fallback`
  - **Positive confirmations:** no phase-local design-foundation blocker was found in the inspected simulator test surface.
  - **Exact fixes required:** none.

## ⚙️ Fixes Applied — 2026-04-09 02:26:21

- This report is the canonical append-only audit artifact at `.planning/phases/033-crypto-audit-scenario-2/033-FULL-AUDIT.md`.
- No crate-local runtime or protocol fix was applied in this execution because every material finding still requires a wider-scope protocol or requirement change already preserved as an explicit blocker in the phase artifacts.
- No phase artifact was force-reclassified during this audit. The open blockers remain visible in `033-TODO.md`, `.planning/REQUIREMENTS.md`, and the final summary table below.

> [!IMPORTANT]
> The remaining findings are blocked wider-scope gaps, not skipped local chores. Closing them would require changing the helper-owned claim boundary, the regular public spend statement, or the checkpoint proof backend rather than only editing wording.

## ♻️ Re-Audit Results — 2026-04-09 02:26:22

- Repository-backed narrowed-surface conclusions remain anchored in the existing source and artifact set:
  - `033-UAT.md`
  - `033-VALIDATION.md`
  - `033-TODO.md`
  - `033-CONTEXT.md`
  - `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
- The same broader blockers still prevent honest full closure:
  - helper-owned claim continuity
  - regular public spend without nullifier semantics
  - checkpoint acceptance without standalone proof-backend authority

## ✅ Doublecheck Results — 2026-04-09 02:26:23

- Repository-backed cross-check outcome:
  - the Q63 helper-owned continuity boundary is preserved honestly;
  - the crossed Q64 and Q65 title/body pairings are preserved honestly;
  - `033-UAT.md` and `033-VALIDATION.md` prove the executed narrowed matrix rather than full implementation closure;
  - `033-TODO.md` and `.planning/REQUIREMENTS.md` still block truthful reclassification to `FULL`.
- No repository artifact in the cited source set supports upgrading the final audit verdict beyond `PARTIAL`.
- The unresolved items in the final summary table remain wider-scope blocked semantic gaps, not crate-local wording or formatting defects.

## 🧾 Exact Fixes Required Summary

| Q | Title | Proof Status | Verification | Severity | Missing Evidence Or Blocker | Gap Closure Path |
| --- | --- | --- | --- | --- | --- | --- |
| 63 | Claim Source Continuity Remains Synthetic | Partial Evidence | VERIFIED | 🟠 HIGH | `claim_source_contract_for_item(...)` still derives root and proof from an off-store helper-owned one-item tree instead of persisted membership state | Bind claim-source proofs to persisted storage-backed membership state, or keep the requirement formally narrowed |
| 64 | Checkpoint Proof Acceptance Is Compatibility-Payload Only | Partial Evidence | VERIFIED | 🟠 HIGH | The regular public spend contract is real and fail closed, but it still carries no nullifier semantics in the regular public spend statement | Extend the spend statement, persisted wire contract, and verifier to bind nullifier semantics, or narrow the requirement honestly |
| 65 | Regular Spend Contract Still Lacks Nullifier Semantics | Partial Evidence | VERIFIED | 🟠 HIGH | Finalized checkpoint acceptance still relies on externally supplied verifier trust and compatibility payload bytes instead of a standalone proof backend | Implement an authoritative checkpoint proof backend and bind finalize/load acceptance to that backend, or narrow the requirement honestly |
| 47 | What May Stay In Documentation | Missing Evidence | BLOCKED | 🟠 HIGH | The documentation allowlist gate is still blocked by the unresolved upstream closure gaps above | Close or formally narrow Q63, Q64, and Q65 first, then re-evaluate what may stay in active documentation |

## 🚩 Final Status

Phase 033 is **PARTIAL** under the `/gsd-audit-4` closure rule.

- The phase directory now has the canonical append-only `033-FULL-AUDIT.md` artifact required by the prompt.
- The executed narrowed matrix remains well covered by UAT, validation, and live stage-surface guards.
- The phase is not honestly fully closed because unresolved `🟠 HIGH` blockers remain in the canonical closure ledger above.
- No unsupported claim of full implementation or full cryptographic closure is made by this audit run.
