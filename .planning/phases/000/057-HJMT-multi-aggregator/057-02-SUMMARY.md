---
phase: 057-HJMT-multi-aggregator
plan: 057-02
status: complete
completed_at: 2026-06-13
next_plan: 057-03
requirements-completed:
  - 057-G4
summary_artifact_for: .planning/phases/057-HJMT-multi-aggregator/057-02-PLAN.md
---

<!-- markdownlint-disable MD060 -->

# 057-02 Summary: Two-Layer Proof Composition And Historical Compatibility

## Completed Scope

`057-02` is complete for the live Phase 057 layered-proof slice.

The repository now ships one explicit public proof wrapper on the existing
storage-owned HJMT seam: `CheckpointPublicationProofV1` binds a verified
shard-local `ProofBlob` to one ordered `CheckpointPublicationV1` plus the
matching `PolicySetCommitmentV1` and `ShardProofContextV1`. Public
publication stays an additive outer layer only. The shard-local HJMT proof
remains the semantic truth component, while the publication wrapper proves
that the shard root appears in one canonical public checkpoint story.

This slice also closes the main correctness risk discovered during execution:
the first review pass found that the wrapper could be self-consistent without
carrying an explicit public-root anchor. The final contract now stores
`public_root`, recomputes `publication.public_root_v1()`, rejects drift, and
forces route-generation, shard-id, journal-checkpoint, and policy-set binding
to line up across both layers before the inner proof is accepted.

Historical compatibility now has executable proof paths instead of paper-only
claims. The live tests prove that an older public proof remains valid against
its original publication root after a later lawful publication appears, and
that route-drift or cross-shard rebinding stays fail-closed. Future-only
wording in the referenced HJMT packet was treated as live scope authority for
this slice, but the implementation stayed on the current storage proof seam
instead of introducing a second semantic proof system.

## Files Changed

- `.planning/phases/057-HJMT-multi-aggregator/057-02-SUMMARY.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `crates/z00z_storage/src/settlement/mod.rs`
- `crates/z00z_storage/src/settlement/proof.rs`
- `crates/z00z_storage/src/settlement/proof_batch.rs`
- `crates/z00z_storage/src/settlement/proof_batch_verify.rs`
- `crates/z00z_storage/src/snapshot/store.rs`
- `crates/z00z_storage/tests/test_hjmt_historical_proofs.rs`

## Boundary Kept Intact

- Phase 057 still publishes Phase 056 lineage; it did not reopen runtime
  routing truth, planner truth, or storage semantic truth.
- The public checkpoint proof stays layered. It does not replace or compress
  the storage-owned HJMT proof family into a second semantic truth path.
- Historical compatibility is enforced by explicit root, route, shard, and
  policy continuity checks instead of permissive replay or implicit rebinding.
- The `6.8.3` cross-shard counterexample remains a reject case rather than a
  normalized success path.

## Review Loop

Manual fallback for `.github/prompts/gsd-review-tasks-execution.prompt.md`
was used because the slash prompt is not a callable tool in this environment.

- Pass 1 found one significant issue: the first wrapper draft did not carry an
  explicit `public_root`, so a self-consistent publication object could be
  verified without an external public-root anchor. The wrapper was patched to
  store `public_root`, recompute `publication.public_root_v1()`, and reject
  anchor drift before the inner proof is accepted.
- Pass 2 re-audited the final wrapper contract, the route/shard/checkpoint/
  policy bindings, and the new historical-proof tests after the targeted
  release reruns. No significant issues remained.
- Pass 3 repeated the same audit after the full workspace `cargo test
  --release`, `cargo doc --no-deps`, and `git diff --check` gates completed.
  No significant issues remained.

Two consecutive clean review passes were achieved on passes 2 and 3.

## Validation

All Rust validation for this slice is green on the final code path.

- `cargo fmt --all` completed.
- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed
  as the mandatory fail-fast gate and was rerun green after the final
  public-root anchor fix.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_historical_proofs -- --nocapture`
  passed.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_live_proof_families -- --nocapture`
  passed.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof -- --nocapture`
  passed.
- `cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test test_stage8_proof_path -- --nocapture`
  passed.
- `cargo test --release` passed for the workspace on the final code tree.
- `cargo doc --no-deps` passed. It reported pre-existing rustdoc warnings in
  untouched crates (`z00z_crypto`, `z00z_core`, `z00z_wallets`,
  `z00z_simulator`) outside the `057-02` slice.
- `git diff --check` is clean.

## Result

`057-02` is complete. Phase 057 now advances to `057-03-PLAN.md` for the
`SIM-5A7S-PUB` publication-integration and trace-packet slice.

This summary does not claim `SIM-5A7S-PUB` runtime wiring, topology-driven
publication activation, join/transfer/carry-forward execution, validator or
watcher binding, or phase closeout evidence; those remain owned by `057-03`
through `057-06`.
