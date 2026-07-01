# Attack Surfaces Resolve Spec

## Scope

- Target finding: `AS-20260615-001`
- Source DB: `.planning/phases/058-HJMT-benchmarks/058-attack-surface-db.jsonl`
- Source report: `.planning/phases/058-HJMT-benchmarks/058-attack-surface-report.md`
- Phase authority: `.planning/phases/058-HJMT-benchmarks/058-TODO.md`

## Finding Summary

`start_cmd` and `restart_cmd` could previously satisfy validation by containing filename markers while actually launching a shadow config tree. At the same time, config digests and runtime evidence remained bound only to the canonical files under `hjmt.home`. That created a second truth lane and violated the phase requirement for one canonical path.

## Acceptance Contract

- Runtime launch commands must be bound to the exact canonical config files already loaded by the node.
- Runtime observability must reject process-topology evidence when launch commands diverge from the hashed canonical config trio.
- No new parallel config-evidence layer may be introduced.
- The fix must fail closed and preserve the existing module structure.

## Ranked Candidates

| Rank | Candidate | Status | Why |
| --- | --- | --- | --- |
| 1 | Exact canonical lifecycle path binding in `z00z_rollup_node`, reused by `z00z_simulator` | selected | Closes the shadow-config lane without adding a second authority path |
| 2 | Emit additional runtime path fields into the packet and cross-check them against digests | rejected | Creates a parallel evidence lane and increases concept-drift risk |
| 3 | Replace free-form lifecycle commands with a new structured lifecycle schema | rejected | Too invasive for Phase 058 and introduces a new abstraction layer |

## Candidate Review

### Candidate 1

**Design**

- Parse `start_cmd` and `restart_cmd` with `shell_words`.
- Require exactly one non-empty `--aggregator-config`, `--planner-config`, and `--storage-config`.
- Normalize and compare those paths against the already-loaded canonical config trio.
- Reuse the same check in runtime observability so the packet path and the launch path cannot diverge.

**Validation Pass A: structural completeness**

- Pass: closes validation gap in node config ingestion.
- Pass: closes observability gap in simulator packet generation.
- Pass: keeps one authority path by reusing loaded config objects.
- Pass: preserves current command-string model instead of adding a second schema.

**Validation Pass B: adversarial robustness**

- Pass: rejects shadow directories that preserve the same filenames.
- Pass: rejects missing config flags.
- Pass: accepts normalized canonical paths, including the repo-root-relative path form used by existing Phase 058 configs.
- Pass: forces fail-closed behavior before launch evidence is accepted.

**Doublecheck status**

- `pass`

**Pros**

- Minimal behavioral surface.
- No parallel logic layer.
- Directly aligned with `058-TODO.md` "one canonical path" requirement.
- Testable with precise positive and negative cases.

**Cons**

- Still depends on shell-style lifecycle command strings.
- Only validates the supported config flags, not arbitrary extra command arguments.

### Candidate 2

**Design**

- Add explicit runtime-resolved config path fields to emitted process evidence and compare them with config digests later.

**Validation Pass A: structural completeness**

- `pass-with-risk`: could detect drift, but only by adding a second evidence channel.

**Validation Pass B: adversarial robustness**

- `fail`: the extra packet lane can itself drift from launch truth and violates the phase rule against parallel authority paths.

**Doublecheck status**

- `fail`

**Why rejected**

- Duplicates authority instead of removing ambiguity.
- Conflicts with the user requirement to avoid codebase duplication and prevent concept drift.

### Candidate 3

**Design**

- Replace free-form lifecycle command strings with a new structured lifecycle schema and render commands from that schema.

**Validation Pass A: structural completeness**

- `pass-with-risk`: would eventually remove ambiguity, but requires larger refactors across config, runtime, and docs.

**Validation Pass B: adversarial robustness**

- `pass-with-risk`: technically strong, but disproportionate to the finding and phase scope.

**Doublecheck status**

- `blocked`

**Why rejected**

- Introduces a new model layer not required by the Phase 058 authority docs.
- Too much scope for a targeted remediation and increases migration risk.

## Final Selection

Candidate 1 is the only acceptable remediation. It removes the shadow-config lane at the point of truth, reuses existing canonical config loading, and keeps the codebase on a single authority path.

## Implemented Delta

- `crates/z00z_rollup_node/src/config.rs`
  - added canonical lifecycle validator `HjmtCfg::check_life_cmd(...)`
  - replaced marker-only lifecycle validation with exact path binding
- `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`
  - reused the same validator before building process-topology evidence
- `crates/z00z_rollup_node/tests/test_hjmt_process.rs`
  - added rejection coverage for shadow config paths
  - added acceptance coverage for normalized canonical paths
- `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`
  - added unit coverage proving process-view rejection on shadow config paths

## Non-Behavioral Cleanup Required To Pass Final Gates

- `crates/z00z_core/src/genesis/genesis_derivation.rs`
  - removed redundant `.into_iter()` calls reported by `clippy`
- `crates/z00z_core/tests/assets/test_integration_assets_test24.rs`
  - replaced two `vec![]` literals with arrays for `clippy::useless_vec`
- `crates/z00z_storage/src/settlement/proof.rs`
  - derived enum defaults instead of manual `impl Default`
- `crates/z00z_storage/src/settlement/hjmt_config.rs`
  - derived enum default instead of manual `impl Default`
- `crates/z00z_wallets/src/tx/asset_selector/mod.rs`
  - replaced `sort_by` with `sort_by_key(Reverse(...))`

These changes do not alter the Phase 058 lifecycle-path fix; they were required to satisfy the repo's `-D warnings` validation path reached by the selected crates.

## Verification

- Pass: `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- Pass: `cargo test -p z00z_rollup_node --release --features test-params-fast --test test_hjmt_process`
- Pass: `cargo test -p z00z_simulator --release --features test-params-fast process_view_rejects_shadow_cfg_paths`
- Pass: `cargo clippy -p z00z_core --release --all-targets --features test-params-fast -- -D warnings`
- Pass: `cargo clippy -p z00z_storage --release --all-targets --features test-params-fast -- -D warnings`
- Pass: `cargo clippy -p z00z_rollup_node --release --all-targets --features test-params-fast -- -D warnings`
- Pass: `cargo clippy -p z00z_simulator --release --all-targets --features test-params-fast -- -D warnings`

## Residual Risk

This remediation removes the hidden shadow-config lane. It does not prevent an authorized operator from intentionally changing the canonical config files themselves; that remains an explicit operational trust boundary rather than an untracked execution bypass.
