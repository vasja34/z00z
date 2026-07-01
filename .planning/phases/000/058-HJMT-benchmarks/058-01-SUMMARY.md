---
phase: 058-HJMT-benchmarks
plan: 058-01
status: complete
completed_at: 2026-06-14
next_plan: 058-02
requirements-completed:
  - 058-G1
summary_artifact_for: .planning/phases/058-HJMT-benchmarks/058-01-PLAN.md
---

<!-- markdownlint-disable MD060 -->

# 058-01 Summary: Evidence Ledger And Source-Honest Acceptance Map

## Completed Scope

`058-01` is complete for the live Phase 058 evidence-ledger slice.

The phase now has one canonical claim ledger:
`058-EVIDENCE-LEDGER.md` freezes gates `058-G1` through `058-G13`, the
Appendix C artifact map, the fixture-family coverage matrix, the `12.1`
evidence-gap rows, and the final verdict vocabulary on one exact
command/artifact/result/verdict row shape. `058-SOURCE-AUDIT.md` now points to
that ledger as the source-honest acceptance authority instead of leaving gate
ownership implied by TODO or design prose.

This slice also makes archive-home honesty explicit on the live bench seam.
`crates/z00z_storage/benches/settlement_benches.md` now states that the
measured bench evidence home is still
`crates/z00z_storage/outputs/settlement/`; alternate archive-home wording
remains a Phase 058 planning requirement until a later bridge or restated
final home lands.

Broad release validation exposed and closed one real test-environment leak
outside the planning packet: `setup_wallet_and_session` in the wallet RPC key
tests now isolates `Z00Z_WALLET_CONFIG_PATH` during setup so release-mode
tests cannot inherit a stale missing config path from concurrent suites. The
same release reruns also closed the last raw async env-lock bypass in
`test_derive_receiver_ignores_post_startup_env_drift`, so the module now uses
the helper-owned wallet-config env path consistently instead of mixing helper
and direct lock entrypoints.

## Files Changed

- `.planning/phases/058-HJMT-benchmarks/058-01-SUMMARY.md`
- `.planning/phases/058-HJMT-benchmarks/058-EVIDENCE-LEDGER.md`
- `.planning/phases/058-HJMT-benchmarks/058-SOURCE-AUDIT.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `crates/z00z_storage/benches/settlement_benches.md`
- `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/test_mod.rs`

## Boundary Kept Intact

- Phase 058 still consumes the completed Phase 056 runtime lineage and the
  completed Phase 057 publication lineage; it did not create a second
  benchmark, fixture, or evidence authority.
- `058-TODO.md` and the referenced HJMT design packet remained live scope
  authority, but TODO-only names were not promoted into false repository
  facts.
- `crates/z00z_storage/outputs/settlement/` remains the only live measured
  bench home today; the planned alternate archive home stayed planning-only.
- The wallet test fix was validation hygiene only. It did not change wallet
  runtime behavior, publication semantics, routing semantics, or Phase 058
  product scope.

## Review Loop

Manual fallback for `.github/prompts/gsd-review-tasks-execution.prompt.md`
was used because the slash prompt is not a callable tool in this environment.

- Pass 1 found a significant issue: `058-EVIDENCE-LEDGER.md` did not exist, so
  Phase 058 had no single canonical gate ledger, Appendix C artifact map,
  fixture-family map, or explicit unsupported-claim vocabulary. The ledger and
  source-audit linkage were added.
- Pass 2 found a significant issue: `test_bench_lanes` failed because the live
  bench note still used legacy archive-home wording that the lane guards reject
  on the measured evidence path. `settlement_benches.md` was rewritten to keep
  archive-home honesty without reintroducing the forbidden live wording.
- Pass 3 found a significant issue in the broad release gate:
  `test_master_rejects_bad_password` failed before reaching the password-path
  assertion because test setup inherited a stale
  `Z00Z_WALLET_CONFIG_PATH` value from concurrent suites. Async env isolation
  was added around `setup_wallet_and_session`.
- Pass 4 found a second significant issue on the first rerun:
  `setup_wallet_and_session` now always took the async wallet-config env lock,
  but several `key_impl` tests already run inside `with_wallet_env(...)` with
  that same lock held. The broad `z00z_wallets` suite deadlocked on the nested
  lock attempt. A task-local re-entrant guard now lets nested helpers reuse
  the active env scope instead of blocking.
- Pass 5 found a significant issue: `058-01-SUMMARY.md` was still missing even
  though `058-01-PLAN.md` requires a closeout artifact before the phase can
  advance. This summary was added and the state/roadmap closeout claims were
  rechecked against the landed files.
- Pass 6 found one more significant issue during the fresh broad
  `cargo test --release` rerun: `test_derive_receiver_ignores_post_startup_env_drift`
  still took `__lock_wallet_config_env_async()` directly and then called
  `setup_wallet_and_session(...)`, bypassing the canonical helper path and
  deadlocking the release suite after the earlier hermeticity fix. The test
  was rewritten to use `with_default_wallet_env(...)`.
- Pass 7 re-audited the final ledger, source-audit sync, bench-home wording,
  state/roadmap claims, and wallet test helper usage against `058-TODO.md`,
  `058-CONTEXT.md`, and `058-01-PLAN.md`. No significant issues remained.
- Pass 8 repeated the same audit after the final validation wave. No
  significant issues remained.

Two consecutive clean review passes were achieved on passes 7 and 8.

## Validation

All validation for this slice is green on the final code tree.

- `cargo test -p z00z_wallets --release test_derive_receiver_ignores_post_startup_env_drift -- --nocapture`
  passed after the raw async env-lock bypass was removed.
- `cargo test -p z00z_wallets --release test_master_rejects_bad_password -- --nocapture`
  passed after the env-isolation fix.
- `cargo test -p z00z_wallets --release test_get_receiver_card_ok -- --nocapture`
  passed after the re-entrant env-lock fix and confirms the
  `with_wallet_env(...)` path no longer deadlocks.
- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed
  as the mandatory fail-fast gate and was rerun green after the validation
  fix.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof -- --nocapture`
  passed.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof_negative -- --nocapture`
  passed.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_root_generation -- --nocapture`
  passed.
- `cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_shard_routing -- --nocapture`
  passed.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_bench_lanes -- --nocapture`
  passed after the bench-note wording fix.
- `cargo fmt --all --check` passed. The repository rustfmt config emitted the
  usual nightly-only option warnings on stable, but no formatting violations
  remained.
- `cargo test --release` passed for the workspace on the final code tree after
  the helper-path cleanup for the last `key_impl` raw-lock bypass.
- `cargo doc --no-deps` was not run because this slice did not change public
  Rust API or rustdoc-owned public surface; it changed phase planning
  artifacts, one live bench note, and one internal test helper.
- `git diff --check` is clean.

## Result

`058-01` is complete. Phase 058 advances to `058-02-PLAN.md` for the
release-mode simulator observability and trace-home closure slice.

This summary does not claim simulator trace packet closure, alternate-topology
closure, startup or import-export readiness, heavy benchmark score packets, or
the final integrated or release-ready verdict; those remain owned by `058-02`
through `058-07`.
