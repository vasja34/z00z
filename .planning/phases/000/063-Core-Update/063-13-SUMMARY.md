---
phase: 063-Core-Update
plan: 063-13
status: complete
completed_at: 2026-06-29
next_plan: none
summary_artifact_for: .planning/phases/063-Core-Update/063-13-PLAN.md
---

# 063-13 Summary: Benches, Binaries, Examples, And Crate Responsibility

## Outcome

`063-13` is complete. `PLAN-063-G13` closes `REC-063-P2-03` by flattening the
`z00z_core` support surface onto one canonical owner path per behavior:
`crates/z00z_core/benches/*`, `crates/z00z_core/bin/*`, and
`crates/z00z_core/examples/*` are now direct-entry roots with no surviving
nested files or nested support-surface `README.md` residue.

The closeout landed four material fixes on the live tree:

- every manifest-owned bench, bin, and example path in
  `crates/z00z_core/Cargo.toml` now points at the flat support tree;
- `autobins = false`, `autoexamples = false`, and `autobenches = false` are
  now explicit, which removed the accidental `bench_helpers.rs` autodiscovery
  lane and kept one manifest-owned target path per support entrypoint;
- the operator-facing CLI boundary is now explicit through a dedicated
  `cli = ["dep:clap"]` feature, while the current export/ZIP surface remains
  intentionally crate-owned and justified by the live genesis output contract;
- the examples and support scripts/docs no longer treat example-local YAML as
  live authority and instead point at the canonical `z00z_config` root.

The mandatory bootstrap gate was rerun first. Its first pass exposed an
environmental race on the shared release target directory during concurrent
bench/build work, so the gate was rerun with an isolated `CARGO_TARGET_DIR`
and then completed green before broader validation continued. The final tree
has green focused release validation, `063-13-PLAN.md` now carries the honest
`cargo bench -p z00z_core --no-run` verify string, `063-13-SUMMARY.md` closes
the last active Phase 063 slice, and no active Phase 063 execution lane
remains.

## Files Changed

- `.planning/phases/063-Core-Update/063-13-PLAN.md`
- `.planning/phases/063-Core-Update/063-13-SUMMARY.md`
- `.planning/STATE.md`
- `.planning/ROADMAP.md`
- `crates/z00z_core/Cargo.toml`
- `crates/z00z_core/benches/{README.md,bench_helpers.rs,commitment_properties_bench.rs,gas_calculation_bench.rs,genesis_bench.rs,metadata_ops_bench.rs,metadata_validation_bench.rs,registry_bench.rs}`
- `crates/z00z_core/bin/{README.md,ASSETS_GENERATION_CLI_SUMMARY.md,assets_allocator_cli.rs,assets_analyzer_cli.rs,assets_analyzer_cli_args.rs,assets_analyzer_cli_ops.rs,assets_extractor_cli.rs,assets_extractor_cli_args.rs,assets_extractor_cli_ops.rs,assets_generation_cli.rs,assets_generation_cli_phase.rs,assets_generation_cli_report.rs,genesis_cli.rs}`
- `crates/z00z_core/examples/{README.md,asset_config_loading.rs,asset_registry_basic.rs,asset_registry_with_metrics.rs,asset_snapshot.rs,genesis_example.rs}`
- `crates/z00z_core/scripts/generate_assets.sh`

## Validation

- `BOOTSTRAP_THREADS=4 CARGO_TARGET_DIR=target/bootstrap-tests-063-13-post ./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo bench -p z00z_core --no-run`
- `cargo build --release -p z00z_core --bins`
- `cargo build --release -p z00z_core --bin assets_generation_cli --features 'cli deterministic-rng'`
- `cargo test --release -p z00z_core --examples`
- `cargo test --release -p z00z_core`
- `cargo metadata --format-version 1 --no-deps`
- `find crates/z00z_core/benches -mindepth 2 -type f`
- `find crates/z00z_core/bin -mindepth 2 -type f`
- `find crates/z00z_core/examples -mindepth 2 -type f`
- `find crates/z00z_core/benches crates/z00z_core/bin crates/z00z_core/examples -mindepth 2 -name 'README.md'`
- `rg -n "path = \"(benches|bin|examples)/.*/" crates/z00z_core/Cargo.toml`
- `rg -n "default =|clap|zip|export" crates/z00z_core/Cargo.toml`
- `git diff --check -- .planning/phases/063-Core-Update/063-13-PLAN.md .planning/phases/063-Core-Update/063-13-SUMMARY.md .planning/STATE.md .planning/ROADMAP.md crates/z00z_core/Cargo.toml crates/z00z_core/benches crates/z00z_core/bin crates/z00z_core/examples crates/z00z_core/scripts/generate_assets.sh`

Result:

- the mandatory bootstrap gate completed green after rerunning on an isolated
  target directory;
- the bench no-run build, all bins build, the feature-gated
  `assets_generation_cli` build, the examples suite, and the full
  `z00z_core` release test suite completed green;
- `cargo metadata` confirmed that the accidental `bench_helpers` bench target
  disappeared once `autobenches = false` became explicit;
- the nested-file and nested-README scans returned no output;
- the manifest nested-path scan returned no matches;
- the feature scan showed the explicit `cli` boundary plus the documented live
  export/ZIP rationale;
- the scoped diff hygiene check stayed clean.

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times in
YOLO mode against this slice, but the available runtime again produced no
review output:

- Attempt 1
  - `timeout 120s gsd --no-session --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/063-Core-Update/063-13-PLAN.md current_task="Flatten benches, bins, and examples to direct owner paths" --yolo'`
  - Result: timed out with exit `124` and no output
- Attempt 2
  - `timeout 120s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/063-Core-Update/063-13-PLAN.md current_task="Flatten benches, bins, and examples to direct owner paths" --yolo'`
  - Result: timed out with exit `124` and no output
- Attempt 3
  - `timeout 120s gsd --no-session -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/063-Core-Update/063-13-PLAN.md current_task="Flatten benches, bins, and examples to direct owner paths" --yolo'`
  - Result: timed out with exit `124` and no output

Equivalent review passes were executed manually under the prompt contract and
the repository `doublecheck` expectations.

- Pass 1
  - Re-read `063-13-PLAN.md`, `063-TODO.md`, the live support-surface tree,
    and `Cargo.toml` against the prompt contract.
  - Result: found one real manifest-ownership issue. `cargo metadata` still
    exposed `bench_helpers.rs` as an unintended bench target until
    `autobenches = false` was added and the CLI boundary was made explicit.
- Pass 2
  - Re-ran the mandatory bootstrap gate on an isolated target directory plus
    the focused bench/bin/example and full `z00z_core` release validations.
  - Result: clean.
- Pass 3
  - Re-ran the nested-path scans, manifest/feature scans, metadata check, and
    scoped `git diff --check` after the plan-doc verify-string fix and summary
    write-up.
  - Result: clean.

Passes 2 and 3 were consecutive clean review runs for the final `063-13`
closeout state.

## Task Status

- `REC-063-P2-03`
  - Closed by the flat manifest-owned `z00z_core` support tree, the canonical
    `z00z_config` authority path, and the explicit `cli` feature boundary with
    no surviving nested support paths, no nested support `README.md`, and no
    example-local YAML authority remaining.

## Closeout

- `063-13-SUMMARY.md` closes `PLAN-063-G13` and completes the full Phase 063
  packet on the existing `.planning/phases/063-Core-Update/` directory only.
- Phase 063 now has no active execution lane remaining.
