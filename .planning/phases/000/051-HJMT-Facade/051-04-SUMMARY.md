---
phase: 051-HJMT-Facade
plan: 051-04
status: complete
completed_at: 2026-05-28T12:34:16Z
requirements:
  - PH51-EQUIVALENCE
  - PH51-COMPAT-BACKEND
  - PH51-PROOF-ENVELOPE
  - PH51-CHECKPOINT-RELOAD
summary_artifact_for: .planning/phases/051-HJMT-Facade/051-04-PLAN.md
---

<!-- markdownlint-disable MD060 -->

# 051-04 Summary: Compatibility Golden Corpus

## Objective

Build the Phase 051 compatibility and golden semantic corpus over the
storage-owned asset facade, while keeping future forest-backend work as a named
extension point rather than a copied compatibility implementation.

## Changes

- Added `crates/z00z_storage/tests/test_phase051_golden_corpus.rs`.
- Added a backend-case registry with one executable `compatibility` case and a
  named, non-executable `future-real-forest` slot so later work can join the
  same expectation matrix without Phase 051 shipping a fake forest backend.
- Added golden semantic workloads through `AssetTreeBackend` and `AssetStore`:
  insert-many, hot-serial, cross-definition, listing pagination, lookup/find,
  valid proof verification, delete-many, reorder-stable final roots, no-op
  roots, duplicate path rejection, and delete-missing rejection.
- Added proof rejection coverage for malformed and trailing detached payloads,
  wrong semantic root, wrong path context, sibling branch proof mismatch,
  root-bind mismatch, and unsupported deletion/non-existence proof families.
- Added RedB reload coverage proving persisted root recovery, path-index
  rebuild, item lookup, proof verification after reload, checkpoint metadata
  reload, and `AssetTreeBackend::validate_reload` behavior.
- Added validator source-shape coverage that keeps checkpoint verdicts on
  storage-owned `CheckpointArtifact` and `RejectClass` authority and prevents a
  duplicated proof formula or artifact decoder from appearing in validator
  checkpoint flow.

## Review Loop

Manual fallback for `.github/prompts/gsd-review-tasks-execution.prompt.md`
was run in YOLO review mode for both tasks because the slash command is not a
callable tool in this environment.

`Add facade compatibility golden tests`:

- Pass 1 found a compile gap in the new corpus: the facade root helper needed
  the public `RootApi` trait in scope. Fixed the import and reran the mandatory
  bootstrap gate before broader validation.
- Pass 2 found no significant issues after the focused compatibility corpus
  release test passed.
- Pass 3 found no significant issues after storage-package and workspace
  release validation passed.

`Add reload checkpoint and proof reject corpus`:

- Pass 1 found that one branch-proof negative case was byte-corrupting proof
  bytes and only proving codec rejection. Reworked the case to use a sibling
  branch proof so it exercises the intended `AssetProofMix` mismatch.
- Pass 2 found no significant issues after the corrected focused release test
  passed.
- Pass 3 found no significant issues after storage-package and workspace
  release validation passed.

## Validation

All Rust validation for this plan was run in release mode.

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed as
  the mandatory fail-fast gate before edits.
- After the `RootApi` import fix,
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed
  again before broader validation.
- After the branch-proof negative-case fix,
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed
  again before broader validation.
- `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_phase051_golden_corpus -- --nocapture`
  passed: 5 passed, 0 failed.
- `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump`
  passed, including the new golden corpus and `Doc-tests z00z_storage`.
- `cargo test --release --features test-fast --features wallet_debug_dump`
  passed for the workspace, including the new golden corpus and workspace
  doctests.
- `cargo fmt --check` exited 0. The command printed the repository's existing
  stable-channel rustfmt warnings for nightly-only options.
- `git diff --check` exited 0.
- Source-shape checks found no dummy/fake forest backend, no public physical
  tree or raw JMT leakage in the new corpus or validator guard targets, no
  duplicated validator checkpoint proof formula, and no stale claim-source
  proof spelling in `.planning`, `docs`, `crates`, or `.github`.

## Result

`051-04` is complete. The active Phase 051 execution lane moves to
`.planning/phases/051-HJMT-Facade/051-05-PLAN.md`.
