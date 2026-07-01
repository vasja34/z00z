---
phase: 053-04
reviewed: 2026-05-30T07:11:36Z
depth: deep
files_reviewed: 20
files_reviewed_list:
  - crates/z00z_storage/src/settlement/fee_envelope.rs
  - crates/z00z_storage/src/settlement/mod.rs
  - crates/z00z_storage/src/settlement/store.rs
  - crates/z00z_storage/src/settlement/store_types.rs
  - crates/z00z_storage/src/settlement/store_rows.rs
  - crates/z00z_storage/src/settlement/store_query.rs
  - crates/z00z_storage/src/settlement/store_codec.rs
  - legacy dual-verify runtime lane, now removed from the live tree
  - crates/z00z_storage/src/settlement/tx_plan_types.rs
  - legacy tx-plan batch helpers, now removed from the live tree
  - crates/z00z_storage/src/settlement/redb_backend.rs
  - crates/z00z_storage/src/settlement/redb_backend_state.rs
  - crates/z00z_storage/src/settlement/redb_backend_helpers.rs
  - crates/z00z_storage/src/settlement/redb_backend_hjmt.rs
  - crates/z00z_storage/src/settlement/hjmt_journal.rs
  - crates/z00z_storage/src/settlement/hjmt_commit.rs
  - legacy whitebox mutation helper, now removed from the live tree
  - crates/z00z_storage/tests/test_fee_envelope.rs
  - crates/z00z_storage/tests/test_fee_replay.rs
findings:
  critical: 2
  warning: 2
  info: 0
  total: 4
status: issues_found
---

# Phase 053-04: Code Review Report

**Reviewed:** 2026-05-30T07:11:36Z  
**Depth:** deep  
**Files Reviewed:** 20  
**Status:** issues_found

## Summary

Reviewed the requested Phase 053-04 FeeEnvelope slice across validator logic,
store entrypoints, commit paths, RedB durability, forest journal checks,
reload, and the dedicated fee tests. I also consulted the adjacent
`crates/z00z_storage/src/settlement/types_record.rs` definition to verify the
public `FeeEnvelope` contract re-exported through
`crates/z00z_storage/src/settlement/mod.rs`.

Focused validation passed:

```bash
cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_fee_envelope --test test_fee_replay
```

The earlier out-of-band fee acceptance issue has been fixed: fee support now flows through the same commit path as the transition ops. The remaining defects are narrower but still material: replay uniqueness is not enforced durably across stale or concurrent store handles, and the checklist-mandated fee budget or reserve semantics are not actually modeled by the current `FeeEnvelope` shape.

## Narrative Findings (AI reviewer)

### CR-01 [BLOCKER] Replay protection is still only checked against in-memory state, so stale or concurrent store handles can accept the same fee envelope twice

**File:** `crates/z00z_storage/src/settlement/store_query.rs`
**File:** `crates/z00z_storage/src/settlement/redb_backend.rs`
**File:** `crates/z00z_storage/src/settlement/redb_backend_hjmt.rs`

**Issue:** `check_fee_support()` rejects replay only when the current process-local `self.fee_replays` map already contains the replay key, then allocates the next sequence from the current in-memory `fee_replay_seq`. The durable write paths for both compatibility and forest backends then open `asset_fee_replays` and insert the row without first checking whether the persisted table already contains the same replay key or whether another writer has already advanced the active version. Two `AssetStore` handles loaded from the same on-disk version can therefore both accept the same `FeeEnvelope`; the second commit simply rewrites the same version-scoped replay row. That violates the Phase 053 durable replay-protection contract in `.planning/phases/053-HJMT-Backend/053-TODO.md:584`.

**Fix:** Enforce replay uniqueness in the same durable write transaction that publishes the fee row. At minimum, fail closed when the persisted replay table already contains the replay key or when the active version has changed since the store instance was loaded. Sequence allocation should come from durable state, not only from `self.fee_replay_seq`.

### CR-02 [BLOCKER] The required fee budget and reserve contract is not implemented, so “insufficient fee support rejects” cannot be satisfied

**File:** `crates/z00z_storage/src/settlement/mod.rs`
**File:** `crates/z00z_storage/src/settlement/store_query.rs`
**File:** `crates/z00z_storage/tests/test_fee_envelope.rs`
**File:** `.planning/phases/053-HJMT-Backend/053-TODO.md:578`  
**File:** `.planning/phases/053-HJMT-Backend/053-TODO.md:580`  
**File:** `.planning/phases/053-HJMT-Backend/053-TODO.md:604`

**Issue:** The checklist requires `FeeEnvelope` to carry budget plus an optional fee-credit or reserve reference, and to reject insufficient fee support. The current exported `FeeEnvelope` contract does not model that behavior. The store derives `budget_commitment` entirely from serialized ops or exec rows in `fee_support_ctx()`, and the tests construct a “valid” envelope by copying that derived hash straight into the envelope. That means the validator can only check whether the envelope mirrors a transition-derived digest; it has no amount, reserve reference, or other funding datum it could use to distinguish sufficient from insufficient support. The current slice therefore cannot satisfy the mandatory insufficient-budget behavior or the optional reserve-reference contract.

**Fix:** Add explicit fee-funding semantics to `FeeEnvelope`, such as a committed budget value plus an optional fee-credit or reserve reference, and validate that funding witness against the transition’s required support before commit. Keep the reserve or fee-credit reference separate from right ownership semantics.

### WR-01 [WARNING] `validate_support()` still accepts partially unverified actor bindings when the caller omits one non-zero party commitment

**File:** `crates/z00z_storage/src/settlement/fee_envelope.rs`

**Issue:** The store now rejects a fully unbound actor context, but it still only verifies the payer or sponsor commitments that the caller chose to provide. If an envelope carries two non-zero party commitments and the caller passes only one of them, the other binding is never checked and still gets persisted into the replay row as if it were validated. That weakens the “who pays” contract and leaves room for accepted fee rows to overstate sponsor or payer attribution.

**Fix:** Require every non-zero binding carried by the envelope to be matched by caller context, or replace the ad hoc optional fields with an explicit binding-mode enum that states whether payer, sponsor, or both must verify.

### WR-02 [WARNING] Test coverage still misses the live duplicate-handle replay case, and the ownership-separation check is only a source-text grep

**File:** `crates/z00z_storage/tests/test_fee_envelope.rs`
**File:** `crates/z00z_storage/tests/test_fee_replay.rs`

**Issue:** The focused tests prove same-handle replay rejection after reload and detect row-deletion tamper on reload, but they do not cover the stale-handle duplicate acceptance path described above. The ownership-separation test also asserts only that `proof.rs` source text does not contain the string `FeeEnvelope`, which is not behavioral evidence that public proof-verifier APIs cannot be misused as fee-ownership evidence. There is also still no executable test for the mandatory insufficient-budget rejection path.

**Fix:** Add a two-handle replay test that loads the same RedB state twice before the first fee commit, then asserts that the second handle fails closed. Replace the source-shape proof test with a behavior-level test through the public verifier surface, and add an insufficient-funding test once the budget contract exists.

---

_Reviewed: 2026-05-30T07:11:36Z_  
_Reviewer: GitHub Copilot (gsd-code-reviewer)_  
_Depth: deep_
