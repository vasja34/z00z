---
phase: 055-HJMT-boundary
plan: 055-01
status: complete
completed_at: 2026-06-10
next_plan: 055-02
requirements-completed:
  - PH55-01
summary_artifact_for: .planning/phases/055-HJMT-boundary/055-01-PLAN.md
---

<!-- markdownlint-disable MD060 -->

# 055-01 Summary: Batch Proof Wire Contract And Deterministic Codec

## Completed Scope

`055-01` is complete for the live Phase 055 contract-freeze slice.

Storage now exposes one canonical additive batch-proof wire surface through
`BatchProofBlobV1`, `BatchProofHeaderV1`, `BatchPathEntryV1`,
`BatchProofLimits`, `WitnessNodeV1`, `OpeningEntryV1`, the family-specific
opening payload structs, and `PathWitnessRefV1`. The new surface is
deterministic, positional, storage-owned, and explicitly versioned. The live
repository keeps internal `SettlementLeafFamily::{Terminal, Right}` naming,
while the public batch wire contract uses explicit V1 wire-tag mappings
instead of renaming the live semantic model.

`ProofBlob` remains unchanged and `Vec<ProofBlob>` remains the independent
batch baseline. `BatchProofBlobV1` is additive only; it does not replace the
existing single-proof authority path.

To keep the frozen contract unambiguous, the landed contract checks also reject
unsupported root generation, partial or non-live shard context, noncanonical
path ordering, duplicate `SettlementPath` entries, out-of-bounds opening or
reference indexes, out-of-bounds witness indexes, opening/path mismatches,
noncanonical default commitments, and shard witness-domain use in the current
non-sharded V1 mode.

## Files Changed

- `.planning/STATE.md`
- `.planning/ROADMAP.md`
- `.planning/phases/055-HJMT-boundary/055-CONTEXT.md`
- `.planning/phases/055-HJMT-boundary/055-SOURCE-AUDIT.md`
- `.planning/phases/055-HJMT-boundary/055-TEST-SPEC.md`
- `.planning/phases/055-HJMT-boundary/055-TESTS-TASKS.md`
- `crates/z00z_storage/src/settlement/mod.rs`
- `crates/z00z_storage/src/settlement/proof.rs`
- `crates/z00z_storage/src/settlement/proof_batch.rs`
- `crates/z00z_storage/src/settlement/proof_batch_verify.rs`
- `crates/z00z_storage/src/snapshot/store/mod.rs`
- `crates/z00z_storage/tests/test_hjmt_batch_proof.rs`
- `crates/z00z_storage/tests/test_live_guardrails.rs`

## Boundary Kept Intact

- The packet's previously future-only design wording is now treated as live
  Phase 055 authority, but it is discharged through the existing storage-owned
  seams instead of a second backlog or parallel implementation.
- `ProofBlob` stayed unchanged; no replacement, alias, or widened public proof
  authority path was introduced.
- `BatchProofBlobV1` is exposed through one canonical
  `z00z_storage::settlement` path only.
- The live repository still uses `SettlementLeafFamily::{Terminal, Right}`
  internally; V1 wire tags bridge naming without a repo-wide rename wave.
- Version 1 remains fail-closed on non-live shard and generation claims.

## Review Loop

Manual fallback for `.github/prompts/gsd-review-tasks-execution.prompt.md`
was used because the slash prompt is not a callable tool in this environment.

- Pass 1 found significant verifier-contract gaps against
  `docs/tech-papers/Z00Z-HJMT-Upgrade.md`: missing bounds checks for
  `opening_index`, `reference_index`, and witness indexes; missing canonical
  path-order enforcement; missing duplicate-path rejection; missing binding
  between `BatchPathEntryV1` and the referenced opening leaf; missing
  canonical default-commitment validation; and missing shard witness-domain
  rejection in live non-sharded V1 mode. All were fixed in the storage-owned
  batch contract path and covered by tests.
- Pass 2 found one remaining error-taxonomy issue: tampered default
  commitments were reported as `UnsupportedBatchProofVersion`. Added the
  typed `ProofChkErr::BatchDefaultCommitmentMix`, mapped it through the
  snapshot-facing error surface, and updated tests.
- Pass 3 reran the contract-truth, canonical-path, error-taxonomy, and
  whitepaper-alignment audit across the live code and tests. No significant
  issues remained.
- Pass 4 repeated the same audit and rechecked diff hygiene. No significant
  phase-local issues remained. `git diff --check` still reports only
  pre-existing unrelated trailing whitespace in
  `.github/agents/gsd-intel-updater.agent.md`.

Two consecutive clean review passes were achieved on passes 3 and 4 after the
Pass 1 and Pass 2 fixes.

## Validation

All Rust validation for this plan was run after the final Rust code change.

- `cargo fmt` completed.
- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed
  as the mandatory fail-fast gate.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof -- --nocapture`
  passed: 12 passed.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_live_guardrails -- --nocapture`
  passed: 13 passed.
- `cargo test --release` passed for the workspace.
- `git diff --check` reports only pre-existing unrelated trailing whitespace in
  `.github/agents/gsd-intel-updater.agent.md`; no new Phase 055 whitespace
  issue was introduced by this slice.

## Result

`055-01` is complete. Phase 055 now advances to `055-02-PLAN.md` for the
explicit negative fixture corpus and the remaining dedicated fail-closed
verifier evidence. This summary does not claim the `BPB-T-*` tamper matrix,
builder fixtures, Stage 13 evidence, or benchmark closeout are complete; they
remain owned by the later numbered Phase 055 plans.
