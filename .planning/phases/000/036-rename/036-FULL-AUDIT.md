# Phase 036 Full Audit

## 🔔 Audit Run — 2026-04-21 23:16:43

### 📌 Audit Setup — 2026-04-21 23:28:06

> [!IMPORTANT]
> Final in-scope crate list before any audit pass begins: `z00z_wallets`, `z00z_simulator`, `z00z_core`, `z00z_crypto`, `z00z_storage`, `z00z_utils`.

- Phase directory: `.planning/phases/036-rename`
- Derived FULL-AUDIT path: `.planning/phases/036-rename/036-FULL-AUDIT.md`
- Mandatory context files read:
  - `.planning/phases/036-rename/036-CONTEXT.md`
  - `.planning/phases/036-rename/036-20-PLAN.md`
  - `.planning/phases/036-rename/036-23-PLAN.md`
  - `.planning/phases/036-rename/036-24-PLAN.md`
  - `.planning/phases/036-rename/036-20-SUMMARY.md`
  - `.planning/phases/036-rename/036-23-SUMMARY.md`
  - `.planning/phases/036-rename/036-24-SUMMARY.md`
  - `.github/prompts/references/gsd-audit-4-full-audit-report-format.md`
  - `.planning/phases/036-rename/036-a4-shims-spec.md`
  - `.planning/phases/036-rename/036-a6_claim-spec.md`
  - `.planning/phases/036-rename/036-a7_crypto-spec.md`
- Execution mode: manual fallback with targeted repository-backed validation
- Explicitly excluded crates or modules:
  - `crates/z00z_crypto/tari/`
  - unrelated workspace crates that were not named by the phase artifacts

### 🎯 Scope And Source Of Truth — 2026-04-21 23:28:06

- The scope is derived from `.planning/phases/036-rename/036-CONTEXT.md` and the phase summaries for `036-20`, `036-23`, and `036-24`.
- The code paths named by the phase artifacts and inspected for this audit were:
  - `crates/z00z_crypto/src/claim/claim_contract.rs`
  - `crates/z00z_crypto/src/hash/mod.rs`
  - `crates/z00z_crypto/src/kdf/mod.rs`
  - `crates/z00z_crypto/src/lib.rs`
  - `crates/z00z_crypto/src/lib_api.rs`
  - `crates/z00z_crypto/src/protocol/commitments.rs`
  - `crates/z00z_crypto/src/README.md`
  - `crates/z00z_crypto/tests/test_claim_contract.rs`
  - `crates/z00z_crypto/tests/test_pedersen.rs`
  - `crates/z00z_crypto/tests/test_bulletproofs.rs`
  - `crates/z00z_wallets/src/core/tx/claim_errors.rs`
  - `crates/z00z_wallets/src/core/tx/claim_tx.rs`
  - `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs`
  - `crates/z00z_wallets/src/core/tx/test_claim_tx.rs`
  - `crates/z00z_storage/src/assets/store_internal/store_query.rs`
  - `crates/z00z_storage/src/assets/types_identity.rs`
  - `crates/z00z_storage/tests/test_claim_source_proof.rs`
  - `crates/z00z_simulator/src/claim_pkg_consumer.rs`
  - `crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs`
  - `crates/z00z_core/src/assets/leaf.rs`
  - `crates/z00z_utils/src/lib.rs`
  - `crates/z00z_utils/src/codec/mod.rs`
  - `crates/z00z_utils/src/rng/mod.rs`
  - `crates/z00z_utils/src/time/traits.rs`

### 🧪 Verification Model — 2026-04-21 23:28:06

#### Critical User Journeys — 2026-04-21 23:28:06

- Claim contract root-version migration.
  - Why it matters: the claim contract now uses a raw `CLAIM_ROOT_VERSION` export rather than the retired wrapper type, and the wallet/storage/simulator paths must agree on that contract.
  - Evidence path: `crates/z00z_crypto/src/claim/claim_contract.rs`, `crates/z00z_crypto/src/lib.rs`, `crates/z00z_storage/src/assets/store_internal/store_query.rs`, `crates/z00z_simulator/src/claim_pkg_consumer.rs`, `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs`, `crates/z00z_crypto/tests/test_claim_contract.rs`, `crates/z00z_storage/tests/test_claim_source_proof.rs`, `crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs`.
- Claim transaction verification and reject-class ordering.
  - Why it matters: semantic claim failures must remain distinguishable from digest mismatches.
  - Evidence path: `crates/z00z_wallets/src/core/tx/claim_tx.rs`, `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs`, `crates/z00z_wallets/src/core/tx/test_claim_tx.rs`.
- Storage-backed claim source proofs.
  - Why it matters: storage remains the authority for the claim-source root and proof blob that the wallet and simulator verify.
  - Evidence path: `crates/z00z_storage/src/assets/store_internal/store_query.rs`, `crates/z00z_storage/src/assets/types_identity.rs`, `crates/z00z_storage/tests/test_claim_source_proof.rs`, `crates/z00z_simulator/src/claim_pkg_consumer.rs`.
- Public crypto helper compatibility surfaces.
  - Why it matters: public helper wrappers such as `hash_to_scalar_domain` and `commit_value` remain part of the facade, so their behavior must stay deliberate and bounded.
  - Evidence path: `crates/z00z_crypto/src/hash/mod.rs`, `crates/z00z_crypto/src/kdf/mod.rs`, `crates/z00z_crypto/src/lib.rs`, `crates/z00z_crypto/src/lib_api.rs`, `crates/z00z_crypto/src/protocol/commitments.rs`, `crates/z00z_crypto/tests/test_pedersen.rs`, `crates/z00z_crypto/tests/test_bulletproofs.rs`.

#### State Transitions — 2026-04-21 23:28:06

- `ClaimRootVer` wrapper to `CLAIM_ROOT_VERSION` raw byte.
  - Preconditions and postconditions: the deleted wrapper must not survive on the public facade, and the new raw-byte contract must be enforced at the claim statement and claim-source proof boundaries.
  - Evidence path: `crates/z00z_crypto/src/claim/claim_contract.rs`, `crates/z00z_crypto/src/lib.rs`, `crates/z00z_crypto/tests/test_claim_contract.rs`.
- Storage claim root to claim-source proof.
  - Preconditions and postconditions: storage emits a versioned claim-source root and proof blob; the simulator and wallet compare the same root-version contract, source root, proof version, and proof blob.
  - Evidence path: `crates/z00z_storage/src/assets/store_internal/store_query.rs`, `crates/z00z_storage/src/assets/types_identity.rs`, `crates/z00z_simulator/src/claim_pkg_consumer.rs`.
- Semantic claim validation to digest verification.
  - Preconditions and postconditions: structure, nullifier, proof, authority, and owner-attestation checks occur before digest mismatch classification.
  - Evidence path: `crates/z00z_wallets/src/core/tx/claim_tx.rs`, `crates/z00z_wallets/src/core/tx/test_claim_tx.rs`.
- README claim export wording to current facade wording.
  - Preconditions and postconditions: the public README should not advertise retired claim exports that no longer exist on the facade.
  - Evidence path: `crates/z00z_crypto/src/README.md`.

#### Proof Paths — 2026-04-21 23:28:06

- Claim statement hashing.
  - Statement: `claim_stmt_hash` must bind the `claim_contract` domain label rather than the retired legacy label.
  - Evidence: `crates/z00z_crypto/src/claim/claim_contract.rs`, `crates/z00z_crypto/tests/test_claim_contract.rs`.
- Claim source proof equality.
  - Statement: storage and simulator must agree on `root_version`, `source_root`, `proof_ver`, and `proof_blob` for the same claim-source item.
  - Evidence: `crates/z00z_storage/src/assets/store_internal/store_query.rs`, `crates/z00z_simulator/src/claim_pkg_consumer.rs`, `crates/z00z_storage/tests/test_claim_source_proof.rs`, `crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs`.
- Claim verifier classification.
  - Statement: proof-version and root-version failures must still reject before digest mismatches are considered.
  - Evidence: `crates/z00z_wallets/src/core/tx/claim_tx.rs`, `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs`, `crates/z00z_wallets/src/core/tx/test_claim_tx.rs`.

#### Failure Paths — 2026-04-21 23:28:06

- Zero root version.
  - Expected behavior: `ClaimStmt` and `ClaimSourceProof` reject zero root versions.
  - Evidence: `crates/z00z_crypto/src/claim/claim_contract.rs`, `crates/z00z_crypto/tests/test_claim_contract.rs`.
- Proof version mismatch.
  - Expected behavior: a mismatched proof version fails source validation before any digest mismatch classification.
  - Evidence: `crates/z00z_crypto/src/claim/claim_contract.rs`, `crates/z00z_crypto/tests/test_claim_contract.rs`, `crates/z00z_wallets/src/core/tx/test_claim_tx.rs`.
- Source root drift.
  - Expected behavior: storage-backed source roots and simulator-consumed proofs must reject drifted or stale roots.
  - Evidence: `crates/z00z_storage/src/assets/store_internal/store_query.rs`, `crates/z00z_simulator/src/claim_pkg_consumer.rs`, `crates/z00z_storage/tests/test_claim_source_proof.rs`, `crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs`.
- README facade drift.
  - Expected behavior: the public README should not expose a retired claim export name.
  - Evidence: `crates/z00z_crypto/src/README.md`.

### 📊 Findings Summary — 2026-04-21 23:28:06

| Severity | Count | Meaning |
| --- | ---: | --- |
| 🔴 CRITICAL | 0 | Immediate correctness, security, or truthfulness failure |
| 🟠 HIGH | 0 | Material gap or blocker on required phase behavior |
| 🟡 MEDIUM | 0 | Non-trivial weakness, drift, or incomplete evidence |
| 🔵 LOW | 1 | Minor documentation drift that was corrected during the audit |
| ⚪ INFO | 0 | Confirmed observation with no immediate remediation |

No actionable runtime defect was proven in the audited claim path. The only concrete mismatch was a stale README export name, and it was corrected during this audit. The remaining open truth is the separate `036-20` partial boundary, which is intentionally kept distinct from the completed `036-23` and `036-24` slices.

### 🔍 Audit Pass Results — 2026-04-21 23:28:06

#### z00z_crypto — 2026-04-21 23:28:06

- status: executed
- files inspected: `src/claim/claim_contract.rs`, `src/hash/mod.rs`, `src/kdf/mod.rs`, `src/lib.rs`, `src/lib_api.rs`, `src/protocol/commitments.rs`, `src/README.md`, `tests/test_claim_contract.rs`, `tests/test_pedersen.rs`, `tests/test_bulletproofs.rs`
- findings grouped by severity:
  - 🔵 Low: stale README wording on the public claim export list referenced `ClaimRootVer`; the doc now says `CLAIM_ROOT_VERSION`.
- exact issues found:
  - The README advertised a retired claim export name that no longer exists on the facade.
- exact fixes required:
  - None remaining after the README update.

#### z00z_wallets — 2026-04-21 23:28:06

- status: executed
- files inspected: `src/core/tx/claim_errors.rs`, `src/core/tx/claim_tx.rs`, `src/core/tx/claim_tx_verifier_impl_proof.rs`, `src/core/tx/test_claim_tx.rs`
- findings grouped by severity:
  - None actionable.
- exact issues found:
  - No semantic drift was proven in the claim verifier ordering, root-version handling, or reject-class classification.
- exact fixes required:
  - None.

#### z00z_storage — 2026-04-21 23:28:06

- status: executed
- files inspected: `src/assets/store_internal/store_query.rs`, `src/assets/types_identity.rs`, `tests/test_claim_source_proof.rs`
- findings grouped by severity:
  - None actionable.
- exact issues found:
  - Storage continues to emit and validate the claim-source root and proof contract using the current root-version constant.
- exact fixes required:
  - None.

#### z00z_simulator — 2026-04-21 23:28:06

- status: executed
- files inspected: `src/claim_pkg_consumer.rs`, `tests/test_claim_pkg_crypto_support.rs`
- findings grouped by severity:
  - None actionable.
- exact issues found:
  - The simulator still compares the storage-backed claim-source root-version contract, source root, proof version, and proof blob.
- exact fixes required:
  - None.

#### z00z_core — 2026-04-21 23:28:06

- status: executed
- files inspected: `src/assets/leaf.rs`
- findings grouped by severity:
  - None actionable.
- exact issues found:
  - The V2 memo boundary remains explicit and bounded through the asset-pack decode contract.
- exact fixes required:
  - None.

#### z00z_utils — 2026-04-21 23:28:06

- status: executed
- files inspected: `src/lib.rs`, `src/codec/mod.rs`, `src/rng/mod.rs`, `src/time/traits.rs`
- findings grouped by severity:
  - None actionable.
- exact issues found:
  - The helper facades already expose explicit codec, RNG, and time contracts, including the try-versus-compat time split used by the audited claim path.
- exact fixes required:
  - None.

### ⚙️ Fixes Applied — 2026-04-21 23:16:43

- Updated `crates/z00z_crypto/src/README.md` so the stable claim export list now uses `CLAIM_ROOT_VERSION` instead of the retired `ClaimRootVer` wrapper name.
- Revalidated the README with a focused grep check; `ClaimRootVer` no longer appears in the public claim export line, and `CLAIM_ROOT_VERSION` is present.
- The targeted claim tests remained green during this audit:
  - `cargo test -p z00z_crypto --release --features test-fast --test test_claim_contract -- --nocapture`
  - `cargo test -p z00z_wallets --release --features test-fast core::tx::claim_tx::claim_tx_tests -- --nocapture`
- No code changes were required for the claim verifier ordering, storage-backed proof wiring, or simulator consumer contract.

### ♻️ Re-Audit Results — 2026-04-21 23:16:43

- The claim contract still hashes with the `claim_contract` label and still enforces raw-byte root-version semantics.
- The wallet verifier still validates structure and semantic claim fields before digest mismatch classification.
- Storage and simulator still agree on the same `ClaimSourceRoot` / `ClaimSourceProof` contract.
- The only contradictory evidence discovered during the audit was the stale README wording, and that text was corrected.
- No new actionable issue appeared in the re-audit pass.

### ✅ Doublecheck Results — 2026-04-21 23:16:43

- Doublecheck ran via the `Doublecheck` subagent.
- Re-verified surfaces: claim contract wiring, claim verification ordering, storage/simulator root-version wiring, and README claim export wording.
- New actionable issues: none.
- Contradictory evidence: the stale README reference to `ClaimRootVer`; fixed during the audit.
- Report truthfulness: supported by the inspected files, the targeted claim tests, and the focused README grep validation.

### 🧾 Exact Fixes Required Summary — 2026-04-21 23:28:06

| Q | Title | Proof Status | Verification | Severity | Missing Evidence Or Blocker | Gap Closure Path |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Preserve the separate `036-20` partial boundary | Full Evidence | VERIFIED | 🟡 MEDIUM | `036-20-SUMMARY.md` still records the constructor, storage, and simulator tail as partial, and that truth remains authoritative | Continue `036-20` under its own contract and do not collapse it into the already-complete `036-23` and `036-24` slices |

### 🚩 Final Status — 2026-04-21 23:28:06

Phase 036 remains partial because `036-20` is still open. The audited claim-contract, storage, simulator, and wallet verifier surfaces are internally consistent, and the only concrete mismatch found during this audit was a stale README export name that has now been corrected. The remaining gap-closure action is to keep `036-20` separate and finish that boundary on its own terms.

## 🔔 Audit Run — 2026-04-21 23:28:06

### 📌 Audit Setup

> [!IMPORTANT]
> Final in-scope crate list before any audit pass begins: `z00z_wallets`, `z00z_simulator`, `z00z_core`, `z00z_crypto`, `z00z_storage`, `z00z_utils`.

- Phase directory: `.planning/phases/036-rename`
- Derived FULL-AUDIT path: `.planning/phases/036-rename/036-FULL-AUDIT.md`
- Mandatory context files read in this rerun:
  - `.planning/phases/036-rename/036-CONTEXT.md`
  - `.planning/phases/036-rename/036-20-PLAN.md`
  - `.planning/phases/036-rename/036-23-PLAN.md`
  - `.planning/phases/036-rename/036-24-PLAN.md`
  - `.planning/phases/036-rename/036-20-SUMMARY.md`
  - `.planning/phases/036-rename/036-23-SUMMARY.md`
  - `.planning/phases/036-rename/036-24-SUMMARY.md`
  - `.planning/phases/036-rename/036-UAT.md`
  - `.planning/phases/036-rename/036-VALIDATION.md`
  - `.planning/phases/036-rename/036-EVAL-REVIEW.md`
  - `.github/prompts/references/gsd-audit-4-full-audit-report-format.md`
- Execution mode: manual fallback with targeted repository-backed validation
- Explicitly excluded crates or modules:
  - `crates/z00z_crypto/tari/`
  - unrelated workspace crates that were not named by the phase artifacts
- Current worktree delta relevant to the audit rerun:
  - `crates/z00z_crypto/src/README.md`
  - `.planning/phases/036-rename/036-FULL-AUDIT.md`

### 🎯 Scope And Source Of Truth

- The scope is still derived from `.planning/phases/036-rename/036-CONTEXT.md` and the phase summaries for `036-20`, `036-23`, and `036-24`.
- The code paths named by the phase artifacts and rechecked for this rerun were:
  - `crates/z00z_crypto/src/claim/claim_contract.rs`
  - `crates/z00z_crypto/src/README.md`
  - `crates/z00z_crypto/src/hash/mod.rs`
  - `crates/z00z_crypto/src/kdf/mod.rs`
  - `crates/z00z_crypto/src/lib.rs`
  - `crates/z00z_crypto/src/lib_api.rs`
  - `crates/z00z_crypto/src/protocol/commitments.rs`
  - `crates/z00z_crypto/tests/test_claim_contract.rs`
  - `crates/z00z_wallets/src/core/tx/claim_errors.rs`
  - `crates/z00z_wallets/src/core/tx/claim_tx.rs`
  - `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs`
  - `crates/z00z_wallets/src/core/tx/test_claim_tx.rs`
  - `crates/z00z_storage/src/assets/store_internal/store_query.rs`
  - `crates/z00z_storage/src/assets/types_identity.rs`
  - `crates/z00z_storage/tests/test_claim_source_proof.rs`
  - `crates/z00z_simulator/src/claim_pkg_consumer.rs`
  - `crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs`
  - `crates/z00z_core/src/assets/leaf.rs`
  - `crates/z00z_utils/src/lib.rs`
  - `crates/z00z_utils/src/codec/mod.rs`
  - `crates/z00z_utils/src/rng/mod.rs`
  - `crates/z00z_utils/src/time/traits.rs`

### 🧪 Verification Model

#### Critical User Journeys

- Claim contract root-version migration.
  - Why it matters: the claim contract still relies on a raw `CLAIM_ROOT_VERSION` export rather than a retired wrapper type, and the wallet/storage/simulator paths must agree on that contract.
  - Evidence path: `crates/z00z_crypto/src/claim/claim_contract.rs`, `crates/z00z_crypto/src/lib.rs`, `crates/z00z_storage/src/assets/store_internal/store_query.rs`, `crates/z00z_simulator/src/claim_pkg_consumer.rs`, `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs`, `crates/z00z_crypto/tests/test_claim_contract.rs`, `crates/z00z_storage/tests/test_claim_source_proof.rs`, `crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs`.
- Claim transaction verification and reject-class ordering.
  - Why it matters: semantic claim failures must remain distinguishable from digest mismatches.
  - Evidence path: `crates/z00z_wallets/src/core/tx/claim_tx.rs`, `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs`, `crates/z00z_wallets/src/core/tx/test_claim_tx.rs`.
- Storage-backed claim source proofs.
  - Why it matters: storage remains the authority for the claim-source root and proof blob that the wallet and simulator verify.
  - Evidence path: `crates/z00z_storage/src/assets/store_internal/store_query.rs`, `crates/z00z_storage/src/assets/types_identity.rs`, `crates/z00z_storage/tests/test_claim_source_proof.rs`, `crates/z00z_simulator/src/claim_pkg_consumer.rs`.
- Public crypto helper compatibility surfaces.
  - Why it matters: public helper wrappers such as `hash_to_scalar_domain` and `commit_value` remain part of the facade, so their behavior must stay deliberate and bounded.
  - Evidence path: `crates/z00z_crypto/src/hash/mod.rs`, `crates/z00z_crypto/src/kdf/mod.rs`, `crates/z00z_crypto/src/lib.rs`, `crates/z00z_crypto/src/lib_api.rs`, `crates/z00z_crypto/src/protocol/commitments.rs`, `crates/z00z_crypto/tests/test_pedersen.rs`, `crates/z00z_crypto/tests/test_bulletproofs.rs`.

#### State Transitions

- `ClaimRootVer` wrapper to `CLAIM_ROOT_VERSION` raw byte.
  - Preconditions and postconditions: the deleted wrapper must not survive on the public facade, and the raw-byte contract must stay enforced at the claim statement and claim-source proof boundaries.
  - Evidence path: `crates/z00z_crypto/src/claim/claim_contract.rs`, `crates/z00z_crypto/src/lib.rs`, `crates/z00z_crypto/src/README.md`, `crates/z00z_crypto/tests/test_claim_contract.rs`.
- Storage claim root to claim-source proof.
  - Preconditions and postconditions: storage emits a versioned claim-source root and proof blob; the simulator and wallet compare the same root-version contract, source root, proof version, and proof blob.
  - Evidence path: `crates/z00z_storage/src/assets/store_internal/store_query.rs`, `crates/z00z_storage/src/assets/types_identity.rs`, `crates/z00z_simulator/src/claim_pkg_consumer.rs`.
- Semantic claim validation to digest verification.
  - Preconditions and postconditions: structure, nullifier, proof, authority, and owner-attestation checks occur before digest mismatch classification.
  - Evidence path: `crates/z00z_wallets/src/core/tx/claim_tx.rs`, `crates/z00z_wallets/src/core/tx/test_claim_tx.rs`.
- README claim export wording to current facade wording.
  - Preconditions and postconditions: the public README should not advertise retired claim exports that no longer exist on the facade.
  - Evidence path: `crates/z00z_crypto/src/README.md`.

#### Proof Paths

- Claim statement hashing.
  - Statement: `claim_stmt_hash` must bind the `claim_contract` domain label rather than the retired legacy label.
  - Evidence: `crates/z00z_crypto/src/claim/claim_contract.rs`, `crates/z00z_crypto/tests/test_claim_contract.rs`.
- Claim source proof equality.
  - Statement: storage and simulator must agree on `root_version`, `source_root`, `proof_ver`, and `proof_blob` for the same claim-source item.
  - Evidence: `crates/z00z_storage/src/assets/store_internal/store_query.rs`, `crates/z00z_simulator/src/claim_pkg_consumer.rs`, `crates/z00z_storage/tests/test_claim_source_proof.rs`, `crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs`.
- Claim verifier classification.
  - Statement: proof-version and root-version failures must still reject before digest mismatches are considered.
  - Evidence: `crates/z00z_wallets/src/core/tx/claim_tx.rs`, `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs`, `crates/z00z_wallets/src/core/tx/test_claim_tx.rs`.

#### Failure Paths

- Zero root version.
  - Expected behavior: `ClaimStmt` and `ClaimSourceProof` reject zero root versions.
  - Evidence: `crates/z00z_crypto/src/claim/claim_contract.rs`, `crates/z00z_crypto/tests/test_claim_contract.rs`.
- Proof version mismatch.
  - Expected behavior: a mismatched proof version fails source validation before any digest mismatch classification.
  - Evidence: `crates/z00z_crypto/src/claim/claim_contract.rs`, `crates/z00z_crypto/tests/test_claim_contract.rs`, `crates/z00z_wallets/src/core/tx/test_claim_tx.rs`.
- Source root drift.
  - Expected behavior: storage-backed source roots and simulator-consumed proofs must reject drifted or stale roots.
  - Evidence: `crates/z00z_storage/src/assets/store_internal/store_query.rs`, `crates/z00z_simulator/src/claim_pkg_consumer.rs`, `crates/z00z_storage/tests/test_claim_source_proof.rs`, `crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs`.
- README facade drift.
  - Expected behavior: the public README should not expose a retired claim export name.
  - Evidence: `crates/z00z_crypto/src/README.md`.

### 📊 Findings Summary

| Severity | Count | Meaning |
| --- | ---: | --- |
| 🔴 CRITICAL | 0 | Immediate correctness, security, or truthfulness failure |
| 🟠 HIGH | 0 | Material gap or blocker on required phase behavior |
| 🟡 MEDIUM | 0 | Non-trivial weakness, drift, or incomplete evidence |
| 🔵 LOW | 0 | No residual documentation drift remained after the README correction |
| ⚪ INFO | 0 | Confirmed observation with no immediate remediation |

No actionable runtime defect was proven in the rerun. The earlier README mismatch remains corrected, and the fresh validation on the claim contract and wallet claim verifier stayed green.

### 🔍 Audit Pass Results

#### z00z_crypto

- status: executed
- files inspected: `src/claim/claim_contract.rs`, `src/hash/mod.rs`, `src/kdf/mod.rs`, `src/lib.rs`, `src/lib_api.rs`, `src/protocol/commitments.rs`, `src/README.md`, `tests/test_claim_contract.rs`, `tests/test_pedersen.rs`, `tests/test_bulletproofs.rs`
- findings grouped by severity:
  - None actionable.
- exact issues found:
  - The claim contract still uses the raw `CLAIM_ROOT_VERSION` contract and the README export line now matches it.
- exact fixes required:
  - None.
- validation:
  - `cargo test -p z00z_crypto --release --features test-fast --test test_claim_contract -- --nocapture` passed again with 5 tests.

#### z00z_wallets

- status: executed
- files inspected: `src/core/tx/claim_errors.rs`, `src/core/tx/claim_tx.rs`, `src/core/tx/claim_tx_verifier_impl_proof.rs`, `src/core/tx/test_claim_tx.rs`
- findings grouped by severity:
  - None actionable.
- exact issues found:
  - The verifier still rejects semantic claim failures before digest mismatches.
- exact fixes required:
  - None.
- validation:
  - `cargo test -p z00z_wallets --release --features test-fast core::tx::claim_tx::claim_tx_tests -- --nocapture` passed again with 37 tests.

#### z00z_storage

- status: executed
- files inspected: `src/assets/store_internal/store_query.rs`, `src/assets/types_identity.rs`, `tests/test_claim_source_proof.rs`
- findings grouped by severity:
  - None actionable.
- exact issues found:
  - Storage continues to emit and validate the claim-source root and proof contract using the current root-version constant.
- exact fixes required:
  - None.

#### z00z_simulator

- status: executed
- files inspected: `src/claim_pkg_consumer.rs`, `tests/test_claim_pkg_crypto_support.rs`
- findings grouped by severity:
  - None actionable.
- exact issues found:
  - The simulator still compares the storage-backed claim-source root-version contract, source root, proof version, and proof blob.
- exact fixes required:
  - None.

#### z00z_core

- status: executed
- files inspected: `src/assets/leaf.rs`
- findings grouped by severity:
  - None actionable.
- exact issues found:
  - The V2 memo boundary remains explicit and bounded through the asset-pack decode contract.
- exact fixes required:
  - None.

#### z00z_utils

- status: executed
- files inspected: `src/lib.rs`, `src/codec/mod.rs`, `src/rng/mod.rs`, `src/time/traits.rs`
- findings grouped by severity:
  - None actionable.
- exact issues found:
  - The helper facades still expose explicit codec, RNG, and time contracts, including the try-versus-compat time split used by the audited claim path.
- exact fixes required:
  - None.

### ⚙️ Fixes Applied — 2026-04-21 23:28:06

- No new code edits were required in this rerun.
- The earlier README correction remains in place: `crates/z00z_crypto/src/README.md` now lists `CLAIM_ROOT_VERSION` instead of `ClaimRootVer`.
- Fresh validation stayed green on the claim contract and wallet claim verifier slices.

### ♻️ Re-Audit Results — 2026-04-21 23:28:06

- The claim contract still hashes with the `claim_contract` label and still enforces raw-byte root-version semantics.
- The wallet verifier still validates structure and semantic claim fields before digest mismatch classification.
- Storage and simulator still agree on the same `ClaimSourceRoot` / `ClaimSourceProof` contract.
- The README wording now matches the public facade and no longer advertises the retired wrapper name.
- No new actionable issue appeared in the rerun.

### ✅ Doublecheck Results — 2026-04-21 23:28:06

- Doublecheck ran via the `Doublecheck` subagent.
- Re-verified surfaces: claim contract wiring, claim verification ordering, storage/simulator root-version wiring, README claim export wording, and the separate 036-20 partial boundary.
- New actionable issues: none.
- Contradictory evidence: none.
- Report truthfulness: supported by the inspected files, the fresh targeted tests, and the doublecheck verification.

### 🧾 Exact Fixes Required Summary

| Q | Title | Proof Status | Verification | Severity | Missing Evidence Or Blocker | Gap Closure Path |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Claim-contract, wallet, storage, simulator, core, and utils surfaces remain internally consistent | Full Evidence | VERIFIED | ⚪ INFO | None | None |
| 2 | Preserve the separate `036-20` partial boundary | Partial Evidence | PARTIAL | 🟡 MEDIUM | `036-20-SUMMARY.md` still records the constructor, storage, and simulator tail as partial, and that truth remains authoritative | Continue `036-20` under its own contract and do not collapse it into the already-complete `036-23` and `036-24` slices |

### 🚩 Final Status

Phase 036 remains partial because `036-20` is still open. The audited claim-contract, storage, simulator, wallet verifier, core, and utils surfaces are internally consistent, and the README drift is fixed and independently doublechecked. The remaining gap-closure action is to keep `036-20` separate and finish that boundary on its own terms.
