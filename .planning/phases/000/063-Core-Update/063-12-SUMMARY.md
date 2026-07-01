---
phase: 063-Core-Update
plan: 063-12
status: complete
completed_at: 2026-06-29
next_plan: 063-13
summary_artifact_for: .planning/phases/063-Core-Update/063-12-PLAN.md
---

# 063-12 Summary: `z00z_core` Docs Truth Restoration

## Outcome

`063-12` is complete. `PLAN-063-G12` closes `REC-063-P2-02` by auditing
`crates/z00z_core/docs`, `crates/z00z_core/README.md`, and the adjacent
Markdown support surfaces under `bin`, `benches`, `examples`, and `tests`
against the live Phase 063 authority.

The live docs now describe only current code and current config authority:

- stale pre-`z00z_config` YAML references are removed from the targeted docs
- stale nested support-surface path references are removed from the targeted
  docs
- the non-ASCII live doc filename was replaced with the ASCII English
  `CLAIM_WITH_CRYPTOGRAPHIC_BALANCE_VALIDATION.md`
- support-surface notes were rewritten to the live `assets_generation_cli`
  binary and the current flattened genesis test layout
- archive backups of fully rewritten docs were moved out of the live tree into
  `.planning/phases/063-Core-Update/backups/063-12-docs/` so final drift scans
  evaluate only active documentation

The final cleanup pass also fixed the rustdoc warnings exposed by the required
`cargo doc --release` gate. The `Arc<...>` and `BTreeMap` type mentions in
`crates/z00z_core/src/assets/*` comments are now backticked, so the release doc
build closes without warning noise from invalid HTML tags.

## Files Changed

- `crates/z00z_core/README.md`
- `crates/z00z_core/docs/{ASSETS_ARCHITECTURE.md,ASSETS_DOCUMENTATION.md,ASSETS_EXAMPLES.md,GENESIS_DOCUMENTATION.md,OBJECT_FAMILY_SEMANTICS.md,CLAIM_WITH_CRYPTOGRAPHIC_BALANCE_VALIDATION.md}`
- `crates/z00z_core/bin/genesis/README.md`
- `crates/z00z_core/benches/genesis/README.md`
- `crates/z00z_core/examples/README.md`
- `crates/z00z_core/examples/assets/CLI_SUMMARY.md`
- `crates/z00z_core/tests/{test_genesis_readme.md,test_genesis_rights_manifest.md}`
- `crates/z00z_core/src/assets/{asset_construction.rs,assets.rs,definition.rs,registry.rs}`
- `.planning/phases/063-Core-Update/backups/063-12-docs/`

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo doc --release -p z00z_core --no-deps`
- `cargo test --release -p z00z_core --tests`
- `rg -n "src/assets/.*\\.ya?ml|src/genesis/.*\\.ya?ml|config/genesis|assets_config\\.yaml|examples/(assets|genesis)|benches/(assets|genesis)|bin/(assets|genesis)" crates/z00z_core/docs crates/z00z_core/README.md`
- `rg -n "src/assets/.*\\.ya?ml|src/genesis/.*\\.ya?ml|config/genesis|assets_config\\.yaml|examples/(assets|genesis)|benches/(assets|genesis)|bin/(assets|genesis)|[А-Яа-я]" crates/z00z_core/bin crates/z00z_core/benches crates/z00z_core/examples crates/z00z_core/tests -g '*.md'`
- `find crates/z00z_core/docs -maxdepth 1 -type f | LC_ALL=C grep '[^ -~]' || true`
- `git diff --check -- crates/z00z_core/README.md crates/z00z_core/docs crates/z00z_core/bin/genesis/README.md crates/z00z_core/benches/genesis/README.md crates/z00z_core/examples crates/z00z_core/tests crates/z00z_core/src/assets`

- Result:
  - the mandatory bootstrap gate passed before the slice work
  - the stale-path scan over docs and README finished with no matches
  - the stale-path plus Cyrillic scan over support-surface Markdown finished
    with no matches
  - the live docs root now contains only ASCII filenames
  - `cargo doc --release -p z00z_core --no-deps` finished green after the
    rustdoc-comment cleanup
  - `cargo test --release -p z00z_core --tests` finished green on the current
    tree
  - diff hygiene stayed clean across the touched docs and source-comment files

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times
against this slice in YOLO mode, but the available runtime again produced no
review output:

- Attempt 1
  - `timeout 120s gsd --no-session --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/063-Core-Update/063-12-PLAN.md current_task="PLAN-063-G12 docs truth restoration review pass 1" --yolo'`
  - Result: timed out with exit `124` and no output
- Attempt 2
  - `timeout 45s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/063-Core-Update/063-12-PLAN.md current_task="PLAN-063-G12 docs truth restoration review pass 2" --yolo'`
  - Result: timed out with exit `124` and no output
- Attempt 3
  - `timeout 45s gsd --no-session -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/063-Core-Update/063-12-PLAN.md current_task="PLAN-063-G12 docs truth restoration review pass 3" --yolo'`
  - Result: timed out with exit `124` and no output

Equivalent review passes were executed manually under the prompt contract and
the repository `doublecheck` expectations.

- Pass 1
  - Rechecked the stale-path, non-ASCII filename, and support-surface Cyrillic
    invariants across the targeted docs
  - Result: clean
- Pass 2
  - Reran `cargo doc --release -p z00z_core --no-deps` after fixing the
    rustdoc warning surface and reran `cargo test --release -p z00z_core --tests`
  - Result: clean
- Pass 3
  - Rechecked changed-file diffs plus `git diff --check` to ensure no stale
    commands, duplicate authority wording, or whitespace damage remained
  - Result: clean

Passes 1 and 2 were consecutive clean review passes after the final doc and
comment fixes, and Pass 3 kept the slice clean through closeout.

## Completion Notes

- `063-12-SUMMARY.md` closes `PLAN-063-G12` and advances the active execution
  lane to `063-13-PLAN.md`.
- The targeted `z00z_core` docs and support-surface Markdown now describe only
  live paths and live support targets.
- The active live doc tree is ASCII-normalized.
