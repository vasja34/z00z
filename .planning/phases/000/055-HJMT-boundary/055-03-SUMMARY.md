---
phase: 055-HJMT-boundary
plan: 055-03
status: complete
completed_at: 2026-06-10
next_plan: 055-04
requirements-completed:
  - PH55-03
summary_artifact_for: .planning/phases/055-HJMT-boundary/055-03-PLAN.md
---

<!-- markdownlint-disable MD060 -->

# 055-03 Summary: Builder, Golden Fixtures, And Compatibility Preservation

## Completed Scope

`055-03` is complete for the live Phase 055 builder-and-fixture slice.

Storage now exposes one additive storage-owned batch builder surface through
`SettlementStore::settlement_inclusion_batch_v1(...)`,
`SettlementStore::settlement_nonexistence_batch_v1(...)`, and
`SettlementStore::settlement_deletion_batch_v1(...)`. The live builder stands
on current `ProofBlob` truth instead of introducing a second proof engine:
each batch path is sourced from the already-live single-path proof context,
the batch header binds semantic `SettlementStateRoot` to the live
`backend_root`, committed bucket-policy fields, journal digest, and checkpoint
bind, and the verifier reconstructs the atomic fold back to `backend_root`
under the same live HJMT model.

`ProofBlob` remains unchanged and `Vec<ProofBlob>` remains the independent
baseline. Witness reuse in V1 stays limited to exact-byte deduplication of
canonical witness nodes; no speculative subtree merge, structural rewrite, or
parallel compression layer was introduced.

The positive fixture corpus is now live and checked in as repository evidence:
`BPB-G-001` through `BPB-G-005` cover inclusion, non-existence, deletion,
clustered witness reuse, and scattered reference-index stability, while the
`root_generation_migration/` manifest records current-generation accept and
unsupported future-generation reject vectors. Compatibility tests now prove
that one `ProofBlob`, the current `Vec<ProofBlob>` baseline, and
`BatchProofBlobV1` reconstruct the same accepted live root for equivalent path
sets, including reload-mode coverage.

## Files Changed

- `.planning/STATE.md`
- `.planning/ROADMAP.md`
- `.planning/phases/055-HJMT-boundary/055-03-SUMMARY.md`
- `crates/z00z_storage/src/settlement/proof.rs`
- `crates/z00z_storage/src/settlement/proof_batch.rs`
- `crates/z00z_storage/src/settlement/proof_batch_verify.rs`
- `crates/z00z_storage/src/settlement/store.rs`
- `crates/z00z_storage/src/settlement/hjmt_proof.rs`
- `crates/z00z_storage/src/settlement/hjmt_batch_proof.rs`
- `crates/z00z_storage/src/snapshot/store/mod.rs`
- `crates/z00z_storage/tests/test_batch_proof_support.rs`
- `crates/z00z_storage/tests/test_hjmt_batch_proof.rs`
- `crates/z00z_storage/tests/test_hjmt_batch_proof_negative.rs`
- `crates/z00z_storage/tests/test_hjmt_live_proof_families.rs`
- `crates/z00z_storage/tests/test_hjmt_proofs.rs`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/batch_proof_v1_positive/manifest.json`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/batch_proof_v1_positive/README.md`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/batch_proof_v1_negative/canonical_sources.json`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/root_generation_migration/manifest.json`
- `crates/z00z_storage/tests/fixtures/hjmt_upgrade/root_generation_migration/README.md`

## Boundary Kept Intact

- No shadow proof engine or second semantic authority path was introduced; the
  batch builder is derived from the live single-path storage proof seam.
- `settlement_proof_blobs` stayed untouched as the independent baseline and
  still returns `Vec<ProofBlob>`.
- V1 witness reuse remains exact-byte canonical deduplication only.
- Unsupported future root generations and shard-context claims remain reject
  vectors in the current live boundary.

## Review Loop

Manual fallback for `.github/prompts/gsd-review-tasks-execution.prompt.md`
was used because the slash prompt is not a callable tool in this environment.

- Pass 1 found one real issue in the new test-support seam: the live deletion
  prior-context helper bound `prior_checkpoint_bind` to
  `hjmt_envelope_version()` instead of the live journal checkpoint. That bug
  could pass accidentally only when both versions matched. The helper was
  corrected to bind against `hjmt_journal_checkpoint()`, and the affected
  storage release tests were rerun.
- Pass 2 re-audited the batch builder, verifier, and fixture corpus against the
  live-proof invariants: root bind, checkpoint bind, policy reconstruction,
  backend-root fold target, exact-byte witness reuse, and baseline preservation.
  No significant issues remained.
- Pass 3 doublechecked coverage against `055-TODO.md`, `055-03-PLAN.md`, and
  `055-TEST-SPEC.md`. The additive builder APIs, `BPB-G-001` through
  `BPB-G-005`, root-generation migration vectors, fixture homes, and baseline
  compatibility anchors were all present in code or checked-in artifacts. No
  missing 055-03 scope remained.

Two consecutive clean review passes were achieved on passes 2 and 3 after the
Pass 1 fix.

## Validation

All Rust validation for this plan was rerun or rechecked after the final
Phase 055 edits.

- `cargo fmt` completed. Stable `rustfmt` emitted the expected warnings about
  ignored nightly-only configuration keys, but formatting succeeded.
- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed
  as the mandatory fail-fast gate.
- `cargo test --release` passed for the full workspace before the final
  helper-only review fix.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof -- --nocapture`
  passed after the final helper fix: 30 passed, 2 ignored helper targets.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof_negative -- --nocapture`
  passed after the final helper fix: 5 passed, 1 ignored helper target.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_live_proof_families -- --nocapture`
  passed after the final helper fix: 17 passed.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_proofs -- --nocapture`
  passed after the final helper fix: 3 passed.
- `git diff --check` is clean on the touched Phase 055 files.

## Result

`055-03` is complete. Phase 055 now advances to `055-04-PLAN.md` for the
benchmark lanes, Stage 13 batch-proof evidence, runner verification
strengthening, and final closeout guardrails. This summary does not claim the
Phase 055 benchmark or simulator evidence slice is complete; those requirements
remain owned by `055-04`.
