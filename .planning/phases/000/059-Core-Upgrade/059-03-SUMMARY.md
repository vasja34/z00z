---
phase: 059-Core-Upgrade
plan: 059-03
status: complete
completed: 2026-06-17
owner: Z00Z Planning
---

# 059-03 Summary: Core Genesis Policies, Vouchers, And Publication

## Scope Delivered

- Extended `GenesisConfig` with additive `policies` and `vouchers` sections
  plus validation and schema coverage so existing configs remain compatible
  while Phase 059 fixtures can declare explicit bootstrap policy and voucher
  records.
- Added deterministic `genesis_policies` and `genesis_vouchers` generation
  under the existing `z00z_core::genesis` boundary, keeping separate typed
  sections for assets, rights, policies, and vouchers instead of introducing a
  second genesis owner path.
- Kept the object split honest at genesis: assets stay finite-supply bootstrap
  definitions, rights stay zero-value authority objects, and genesis vouchers
  stay explicit bootstrap exceptions rather than the ordinary runtime issuance
  path.
- Extended Stage 1 publication to export `genesis_policies.json` and
  `genesis_vouchers.json`, bind replay and roundtrip digests for both artifact
  families, and lift `genesis_settlement_manifest.json` to canonical manifest
  version `2`.
- Rebound genesis rights, policies, and vouchers to the canonical
  `z00z_core::{actions,policies,rights,vauchers}` roots from `059-02` instead
  of leaving descriptor truth stranded in genesis-local duplicates or
  `assets/*` ownership.
- Updated downstream Stage 1 consumers in storage and simulator to ingest the
  widened genesis packet, and hardened `scenario_1` verification so policies,
  vouchers, rights, assets, and manifest fields are checked against the
  config-derived expected corpus rather than only against self-consistent
  artifacts.

## Boundary Kept

- This slice did not yet land `z00z_storage` voucher leaf families or proof
  semantics, typed object deltas or runtime conservation, wallet persistence or
  RPC surfaces, or Alice/Bob/Charlie object-transfer lanes in the simulator;
  those remain in `059-04` through `059-10`.
- No parallel genesis module, second manifest owner, or second descriptor
  authority was introduced; all publication widening stayed under the existing
  `z00z_core::genesis` export path.
- Existing configs without explicit vouchers remained additive-compatible under
  the compatibility and defaulting rules captured in `059-03-PLAN.md`.
- Runtime voucher issuance, watcher or validator verdict logic, and wallet
  spendability semantics were not claimed complete here.

## Validation

- Mandatory bootstrap gate passed on the final code:
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- Targeted Phase 059 validations passed:
  `cargo test -p z00z_core --release --features deterministic-rng test_genesis_manifest_phase059_fixture -- --nocapture`
  `cargo test -p z00z_storage --release test_ingestion_creates_rights -- --nocapture`
  `cargo test -p z00z_simulator --release runner_verify -- --nocapture`
- Prompt-required release validations passed:
  `cargo test -p z00z_storage --release --features test-params-fast`
  `cargo test -p z00z_wallets --release --features test-params-fast`
  `cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools`
- Broad workspace validation passed on the final code:
  `cargo test --release`
- `git diff --check` on the touched simulator and planning files was clean.
- Manual review against `.github/prompts/gsd-review-tasks-execution.prompt.md`
  was run in three passes:
  pass 1 found a significant Stage 1 verifier gap where
  `genesis_policies.json` and `genesis_vouchers.json` were loaded but not
  checked against a config-derived expected packet; that was fixed in
  `crates/z00z_simulator/src/scenario_1/runner_verify.rs`.
  pass 2 found no further significant code issues after the verifier fix and
  release-feature reruns.
  pass 3 found no significant planning, state, or roadmap sync issues after
  closeout.
- During release validation, a stale unrelated `cargo-nextest` tree from an
  older verification-orchestrator run was terminated because it was competing
  with the active simulator validation and did not belong to the current
  `059-03` closeout.

## Next Plan

Execution moves to `059-04-PLAN.md` for storage voucher leaf family and proof
semantics.
