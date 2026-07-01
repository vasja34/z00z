<!-- markdownlint-disable MD003 MD022 MD036 MD041 MD047 MD056 MD060 -->
---
phase: 034
slug: mix1-fixes
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-10
updated: 2026-04-10
---

# Phase 034 — Validation Strategy

> Per-phase Nyquist validation contract audited against the executed Phase 034
> plans, summaries, closeout artifacts, and the live repository-backed state.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` integration and library tests plus repository `rg` and source-shape assertion checks |
| **Config file** | [Cargo.toml](/home/vadim/Projects/z00z/Cargo.toml) |
| **Quick run command** | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` |
| **Full suite command** | `cargo test --release --features test-fast --features wallet_debug_dump` |
| **Estimated runtime** | repository-dependent long-running release suite |

---

## Sampling Rate

- **After every task commit:** Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- **After every plan wave:** Run the exact phase-local targeted binaries recorded in the executed Phase 034 summaries, then corroborate with `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump` when the touched seam is simulator-visible
- **Before `/gsd-verify-work`:** The targeted closure matrix for claim continuity, spend-nullifier semantics, checkpoint backend acceptance, and wording guards must stay green, with broader release reruns used as corroborating evidence instead of replacing the seam-local proof
- **Max feedback latency:** bounded by bootstrap-first gates and targeted release binaries before any broader rerun

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 034-01-01 | 01 | 1 | PH34-CLAIM-CONTINUITY | — | Persisted membership remains the only authoritative claim-source seam, synthetic one-item fallback stays non-authoritative, and empty bundles reject fail closed | integration | `cargo test -p z00z_storage --release --test test_claim_source_proof -- --nocapture && cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage4_claim_gate -- --nocapture` | ✅ | ✅ green |
| 034-02-01 | 02 | 2 | PH34-SPEND-NULLIFIER | — | Regular spend nullifier semantics remain chain-bound, malformed and drifted `nullifier_hex` values reject fail closed, and Stage 4 package drift is rejected on the verified-package path | integration | `cargo test -p z00z_wallets --release --test test_spend_witness_gate -- --nocapture && cargo test -p z00z_wallets --release --test test_scenario1_semantics -- --nocapture && cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_spend_gate -- --nocapture` | ✅ | ✅ green |
| 034-03-01 | 03 | 3 | PH34-SENDER-AUTHORITY | — | Sender construction stays owned by `core::stealth`, public tx-facade construction authority remains retired, and legacy builder entrypoints stay fail-closed compatibility shims instead of becoming live authority again | integration + assertion | `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --test test_s5_spec6_bridge --test test_spend_witness_gate --test test_s7_examples && cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage6_checkpoint_final_gate --test test_stage6_checkpoint_storage_bridge && rg -n "build_stealth_sender_leaf|build_tx_stealth_output|build_tx_stealth_output_validated" crates/z00z_wallets/src/core/stealth/mod.rs && ! rg -n "pub mod builder|pub mod output_flow|pub use .*sender_create_output_for" crates/z00z_wallets/src/core/tx/mod.rs` | ✅ | ✅ green |
| 034-04-01 | 04 | 4 | PH34-CHECKPOINT-BACKEND | — | Checkpoint finalize, persisted reload, and simulator promotion all remain bound to backend-owned payload truth, while compat-only proof objects reject fail closed | integration | `cargo test -p z00z_storage --release --test test_checkpoint_finalization -- --nocapture && cargo test -p z00z_storage --release --test test_checkpoint_store_api -- --nocapture && cargo test -p z00z_storage --release --test test_redb_rehydrate -- --nocapture && cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_checkpoint_acceptance -- --nocapture` | ✅ | ✅ green |
| 034-05-01 | 06 | 5 | PH34-SPEND-NULLIFIER | — | The explicit spend validation wave keeps missing-nullifier rejection and public-spend drift rejection executable on exact named integration binaries instead of false-green filter runs | integration | `cargo test -p z00z_wallets --release --test test_spend_witness_gate -- --nocapture && cargo test -p z00z_wallets --release --test test_scenario1_semantics -- --nocapture && cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_spend_gate -- --nocapture` | ✅ | ✅ green |
| 034-06-01 | 06 | 6 | PH34-CHECKPOINT-BACKEND | — | The explicit checkpoint validation wave keeps unsupported proof-system bytes, reload drift, and finalize mismatch rejection executable on exact named binaries instead of plan drift or filter-only runs | integration | `cargo test -p z00z_storage --release --test test_checkpoint_finalization -- --nocapture && cargo test -p z00z_storage --release --test test_checkpoint_store_api -- --nocapture && cargo test -p z00z_storage --release --test test_redb_rehydrate -- --nocapture && cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_checkpoint_acceptance -- --nocapture` | ✅ | ✅ green |
| 034-07-01 | 07 | 7 | PH34-DOC-ALLOWLIST / Q47 | — | Active wording guards track the implemented closure truth, tolerate wrap and layout changes, and do not silently regress to stale blocker wording or stronger-than-shipped authority claims | integration + assertion | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_stage_surface -- --nocapture && cargo test -p z00z_storage --release --features test-fast --test test_redb_rehydrate -- --nocapture` | ✅ | ✅ green |
| 034-08-01 | 08 | 8 | PH34-CLOSURE-PROOF | T-034-20 | The phase-local closure package proves Q63, Q64, Q65, and Q47 are closed on the live tree, reconciles the main semantic chain, and keeps optional sidecars outside semantic completion evidence | integration + assertion | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh && cargo test -p z00z_storage --release --test test_claim_source_proof && cargo test -p z00z_wallets --release --test test_spend_witness_gate && cargo test -p z00z_storage --release --test test_checkpoint_finalization && cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_stage_surface && cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump` | ✅ | ✅ green |

*Status: ✅ green · ❌ red · ⚠️ partial*

---

## Wave 0 Requirements

Existing infrastructure covers all executed Phase 034 requirements.

No Wave 0 harness installation, fixture bootstrapping, or new test-framework
creation is needed for the main Phase 034 semantic chain.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Optional keep-path complexity cleanup, if later executed | PH34-KEEP-PATH-SIDECAR | Deferred post-closure sidecar; not executed on the current live tree and explicitly excluded from semantic closure evidence | If `034-09` executes the sidecar, rerun `cargo test -p z00z_storage --release test_search_api -- --nocapture` and confirm the Phase 034 closeout wording still treats it as non-semantic |

---

## Validation Audit 2026-04-10

| Metric | Count |
|--------|-------|
| Gaps found | 0 |
| Resolved | 0 |
| Escalated | 0 |

### Audit Notes

- Input state: **State A** — an existing `034-VALIDATION.md` already existed beside the executed `034-*-SUMMARY.md` artifacts.
- The existing file already proved the Phase 034 closure story by evidence, but it did not yet expose the standard Nyquist validation-strategy structure used by the repository templates and prior validated phases.
- This audit found no missing automated verification for the executed main semantic chain. Instead, it normalized the validation artifact into an explicit infrastructure-plus-per-task matrix so coverage can be audited without re-deriving the phase from narrative closeout text.
- The executed live semantic chain for Phase 034 is the summary-backed sequence through `034-08`; the completed `034-09` hygiene chain is recorded separately in `034-09-SUMMARY.md` and is therefore not required for `nyquist_compliant: true` on the closed Q63/Q64/Q65/Q47 chain.
- No new Nyquist test file was generated in this audit because the executed summaries and the live tree already map Phase 034 behavior to canonical existing test homes in `z00z_storage`, `z00z_wallets`, and `z00z_simulator`.
- The optional keep-path sidecar `034-15` remains explicitly outside the semantic closure story and is tracked as a deferred/manual-only follow-up rather than an uncovered gap.
- The executed post-closure hygiene chain `034-16`, `034-17`, and `034-18` is now recorded in `034-09-SUMMARY.md` and should no longer be read as deferred validation work.

---

## Validation Sign-Off

- [x] All executed main-chain tasks have automated verification commands or repository-backed assertion checks
- [x] Sampling continuity is present through bootstrap-first gates, targeted release binaries, and summary-backed reruns
- [x] Wave 0 infrastructure already covers the phase surface
- [x] No watch-mode flags were used in the validation contract
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-04-10

Reason: The executed Phase 034 main semantic chain already has full automated
coverage on the live tree, and the audit found no missing Nyquist tests. The
remaining work was to normalize the existing proof-oriented validation record
into the repository's standard strategy format while keeping only the deferred
`034-15` keep-path sidecar explicitly out of semantic completion evidence.

---

## Closure Evidence Appendix

### Purpose

This artifact is the repository-backed validation package for the mandatory
`034-14` closure sweep. It reruns the required targeted seam-specific
regression allowlist from `034-09` through `034-13`, then confirms the
release-style simulator and broader workspace gates on the current tree.

### Required Blocker Matrix

| Blocker | Old truth | Current proof | Repository-backed evidence |
| --- | --- | --- | --- |
| Q63 | claim-source continuity remained helper-owned and synthetic | persisted membership state is now the accepted authority path and synthetic one-item authority is rejected | `test_claim_source_contract_for_item_matches_store_roundtrip`, `test_claim_source_contract_for_item_rejects_synthetic_one_item_authority`, `test_authoritative_root_claim_emit` |
| Q64 | regular public spend lacked enforced nullifier semantics | deterministic nullifier behavior, malformed input rejection, duplicate rejection, and signed drift rejection are green on wallet and simulator paths | `test_public_rejects_bad_nullifier_hex`, `test_public_rejects_duplicate_nullifier`, `test_public_rejects_signed_nullifier_drift`, `scenario1_public_spend_gate_matches_wallet_verifier` |
| Q65 | checkpoint acceptance remained compatibility-payload or external-trust driven | finalize, seal, reload, and simulator acceptance now consume the backend-defined package-coupled proof path and reject drift | `test_attested_proof_requires_backend_payload`, `test_legacy_opaque_proof_rejects_on_seal`, `rehydrate_rejects_attested_proof_drift`, `checkpoint_roundtrip_matches_exec_proof_across_storage_surfaces` |
| Q47 | active documentation allowlist could not be reclassified honestly while Q63, Q64, and Q65 were unresolved | active wording guards now track the implemented closure truth while append-only audit artifacts remain historical | `test_phase034_doc_allowlist_tracks_active_closure_truth`, `test_public_spend_boundary_wording_stays_narrow`, `test_checkpoint_continuity_wording_stays_package_coupled` |

### Targeted Regression Sweep

#### 034-09 Claim Continuity Test Wave

- `cargo test -p z00z_storage --release --test test_claim_source_proof` passed with 6 tests.
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_pkg_crypto_support` passed with 12 tests.

#### 034-10 Spend Nullifier Test Wave

- `cargo test -p z00z_wallets --release --test test_spend_witness_gate` passed with 16 tests.
- `cargo test -p z00z_wallets --release --test test_scenario1_semantics` passed with 10 tests.
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_spend_gate` passed with 9 tests.

#### 034-11 Checkpoint Backend Test Wave

- `cargo test -p z00z_storage --release --test test_checkpoint_finalization` passed with 7 tests.
- `cargo test -p z00z_storage --release --test test_checkpoint_store_api` passed with 11 tests.
- `cargo test -p z00z_storage --release --test test_redb_rehydrate` passed with 15 tests.
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_checkpoint_acceptance` passed with 6 tests.

#### 034-12 And 034-13 Documentation And Wording Guards

- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_stage_surface` passed with 29 tests.

### Manual Documentation Sweep

- `grep` over `.planning/REQUIREMENTS.md` found no active authority drift to `builder.rs`, `output_flow.rs`, or `core::tx::builder`.
- `grep` over `.planning/phases/040-spend-proof/040-Spend-Proof-Spec.md` found no active authority drift to `builder.rs`, `output_flow.rs`, or `core::tx::builder`.
- `grep` over `.planning/temp/Z00Z-ECC-IDEAS.md` found no active authority drift to `builder.rs`, `output_flow.rs`, or `core::tx::builder`.
- `.planning/temp/Z00Z-ECC-SPEC_part1.md` was manually swept so its active planning guidance no longer points future sender-construction work at `core::tx::builder`, `output_builder.rs`, or `output_flow.rs` as canonical construction owners; mixed-era conceptual material outside that owner claim remains non-authoritative and does not replace the live `034-05` summary plus current source surfaces.

### Sender-Authority Migration Evidence

- `.planning/phases/034-mix1-fixes/034-03-SUMMARY.md` is the summary-backed repository artifact for the sender-authority retirement slice and records the migrated wallet, simulator, example, and proof-planning surfaces.
- `crates/z00z_wallets/src/core/tx/mod.rs` now documents `crate::core::stealth` as the public sender-construction owner, keeps `builder` and `output_flow` private, and no longer re-exports legacy sender-construction entrypoints on the public tx facade.
- `crates/z00z_wallets/src/core/tx/builder.rs` keeps `sender_create_output_for(...)` only as a blocked legacy path that fails closed with `migrate to core::stealth::build_stealth_sender_leaf`.
- `crates/z00z_wallets/src/core/tx/output_flow.rs` keeps `create_output_bundle(...)` only as a blocked compatibility shim that fails closed and points callers at the canonical `core::stealth` construction owner.
- `crates/z00z_wallets/src/core/stealth/mod.rs` publicly exports the canonical sender-construction surface, including `build_stealth_sender_leaf`, `build_tx_stealth_output`, and `build_tx_stealth_output_validated`.
- `crates/z00z_wallets/tests/test_s5_sender_examples.rs` now exercises the stealth-owned sender surface directly, and `crates/z00z_simulator/tests/test_claim_acceptance.rs` keeps the migrated Scenario 1 lane wired through the narrowed stage helpers rather than through public `core::tx::builder::*` or `core::tx::output_flow::*` construction ownership.

### Release Gates

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed and ended with `=== BOOTSTRAP COMPLETE ===`.
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_stage_surface` passed with 29 tests and wrote `PHASE034_STAGE_SURFACE_GATE_OK` into its raw log.
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump` passed on the current tree.
- `cargo test --release --features test-fast --features wallet_debug_dump` was rerun on the current tree and passed; the fresh transcript is recorded at `.planning/phases/034-mix1-fixes/logs/034-14-workspace-release-rerun.log`.
- Raw stage-surface transcript: `.planning/phases/034-mix1-fixes/logs/034-14-stage-surface.log`
- Raw bootstrap transcript: `.planning/phases/034-mix1-fixes/logs/034-14-bootstrap.log`
- Raw simulator release transcript: `.planning/phases/034-mix1-fixes/logs/034-14-simulator-release.log`
- Raw workspace release transcript: `.planning/phases/034-mix1-fixes/logs/034-14-workspace-release-rerun.log`

The simulator and workspace release transcripts are external corroboration
artifacts. The stage-surface guard checks that the phase package points at
those exact raw logs, but it intentionally does not re-read their content from
inside `test_phase034_closeout_artifacts_reconcile_semantic_chain()` because
the same commands generate the logs and would create a circular proof surface.

### Conclusion

The required targeted `034-09` through `034-13` regression allowlist reran
green on the live repository, the exact simulator release gate also passed
afterward, and the fresh full workspace release rerun now passes on the same
tree. That broader rerun remains corroborating evidence rather than a
replacement for the seam-local proof package. On the current tree, the old
Q63, Q64, and Q65 blocker forms are no longer reproducible, the
sender-authority migration remains evidenced by the summary-backed `034-03`
artifact plus the current fail-closed tx and stealth surfaces, and the
active-documentation allowlist gate for Q47 tracks the implemented closure
truth across the required active documentation sweep without rewriting
append-only historical audit artifacts.
