---
phase: 053-HJMT-Backend
plan: 053-10
status: complete
completed_at: 2026-06-02
next_plan: 053-11
requirements:
  - PH53-10
summary_artifact_for: .planning/phases/053-HJMT-Backend/053-10-PLAN.md
---

<!-- markdownlint-disable MD060 -->

# 053-10 Summary: Forest Cache Plane

## Completed Scope

`053-10` is complete for the private forest-cache plane on the live settlement
HJMT surface.

The cache is now wired into the real `AssetStore` seam through
`hjmt/hjmt_cache.rs` and covers subtree roots, parent leaves, terminal leaf
encodings, bucket derivation, proof segments, non-existence proofs, adaptive
policy proofs, journal digests, and the terminal path index. All entries stay
private performance or diagnostics state and are recomputed from settlement
authority before acceptance.

Dirty-set invalidation now runs on commit, rollback restores the prior cache
snapshot, proof-version drift clears versioned layers, and RedB reload paths
clear or warm the cache from durable root state before sampled verification.
The verification contract is fail-closed: sampled cache entries must match
authoritative recomputation for values and key metadata, including
root-generation, proof-version, policy-id, epoch, and journal backend-root
bindings.

Measured reuse evidence is now explicit in the phase-owned tests. Hot proofs
increase `subtree_root`, `parent_leaf`, and `proof_segment` hit counters versus
cold recomputation, unrelated updates preserve unchanged bucket or serial or
definition roots, bounded eviction produces real evictions without changing
proof bytes, and rollback or reload keeps hot and cold proof results equal.

## Scoped Boundary

This summary closes the forest-cache slice only. It does not claim async
scheduler parallelism, downstream generalized settlement integration, final doc
closeout beyond cache-slice evidence, or legacy-storage purge work.

## Review Loop

The required `GSD-Review-Tasks-Execution` loop completed for `053-10`.

- Review pass 1 found a correctness gap: `verify_forest_cache()` could accept
  key-only drift in several cache layers because it recomputed values without
  validating all cache-key metadata. The verification path now checks
  root-generation, proof-version, policy-id, and journal backend-root fields.
- Review pass 2 found a second correctness gap: journal-key verification still
  derived the current backend root through cached state. The verification path
  now uses authoritative `hjmt_roots`, and a journal-key drift regression test
  was added.
- Review pass 3 reran the task on the post-fix tree and found no significant
  remaining issues.
- Review pass 4 reran the same task after the full validation wave and found no
  significant remaining issues.

Two consecutive post-fix review passes were clean.

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed on
  the final tree.
- `cargo test -p z00z_storage --release --features test-fast --test test_forest_cache --test test_cache_recompute`
  passed.
- `cargo test -p z00z_storage --release --features test-fast` passed on the
  final tree.
- `cargo test --release --features test-fast --features wallet_debug_dump`
  passed on the final tree.

## Result

`053-10` is complete. Phase 053 advances to `053-11-PLAN.md` for the async
forest-scheduler and parallel-commit slice.
