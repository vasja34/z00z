---
phase: 040
slug: spend-proof
status: in_progress
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-25
updated: 2026-04-28
last_nyquist_audit: 2026-04-28
gap_count: 0
manual_open_boundaries: public proof-of-knowledge, checkpoint theorem finality, rollup settlement proof closure
---

# Phase 040 - Validation Strategy

## Internal Theorem-Relation Validation Reset

`040-09-SUMMARY.md` remains the last completed implementation checkpoint, but
the active validation authority now follows `040-10-PLAN.md`. This file must
track the internal theorem-relation closure gates without claiming public or
trustless proof-of-knowledge closure.

## Test Infrastructure

| Property | Value |
| --- | --- |
| Framework | `cargo test` across the Rust workspace |
| Config file | `Cargo.toml` workspace root |
| Quick run command | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` |
| Full suite command | `cargo test --release --features test-fast --features wallet_debug_dump` |

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Secure Behavior | Automated Command | Status |
| --- | --- | --- | --- | --- | --- | --- |
| 040-01 | 01 | T0 | Proof Carrier Contract | Versioned proof wire rejects unknown versions and placeholder drift once non-empty mode is active | `cargo test -p z00z_wallets --release --features test-fast --test test_spend_proof_wire -- --nocapture` | green |
| 040-02 | 01 | T1 | Canonical Spend Statement | Canonical statement is recomputed from tx facts and rejects fee, root, output, chain, and digest drift | `cargo test -p z00z_wallets --release --features test-fast --test test_spend_statement -- --nocapture` | green |
| 040-03 | 02 | T2 | Producer Path | Producer emits the canonical public spend carrier and fails closed on witness or package mismatch | `cargo test -p z00z_wallets --release --features test-fast --test test_spend_prover_contract -- --nocapture` | green |
| 040-04 | 02 | T2 | Verifier Path | Public verifier rejects malformed proof bytes, statement drift, root drift, and input binding drift | `cargo test -p z00z_wallets --release --features test-fast --test test_tx_proof_verifier -- --nocapture` | green |
| 040-05 | 03 | T3 | Nullifier Semantics | Regular-spend nullifiers stay deterministic, scoped, separate from claim nullifiers, and replay-safe at the checkpoint/state boundary | `cargo test -p z00z_wallets --release --features test-fast --test test_spend_nullifier_semantics -- --nocapture` | green |
| 040-06 | 03 | T3 | Full Regular Package Verification Entry Point | The composed full verifier rejects local-wire-only and proofless acceptance shortcuts | `cargo test -p z00z_wallets --release --features test-fast --lib core::tx::tx_verifier::tests -- --nocapture` | green |
| 040-07 | 04 | T4 | End-to-End Roundtrip And Surface Locks | Stage 4 to Stage 6 to Stage 11 keeps statement/root continuity and fail-closes before authoritative checkpoint mutation on drift | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_tx_proof_roundtrip -- --nocapture` | green |
| 040-08 | 04 | T5 | Optional Output-Constructor Follow-Up | The optional builder/output cleanup is closed as no-op while existing output-facing tests preserve `leaf_ad`, `tag16`, commitment, range-proof, and self-decrypt behavior | `cargo test -p z00z_wallets --release --features test-fast --test test_s5_sender_examples --test test_stealth_output --test test_e2e_send_scan -- --nocapture` | green |
| 040-09 | 10 | T1 | Final Authority Reset | Current phase authorities freeze one canonical internal theorem suite, one theorem contract `T(S, W)`, and one grounded witness table without live statement-bound aliases | bootstrap gate plus legacy semantic-branch grep over live code and test surfaces | green; legacy scan has no live-code hits |
| 040-10 | 10 | T2 | Canonical Internal Theorem Carrier And Backend | The live wallet proof-generation path uses one canonical suite and backend, rejects legacy artifacts, and validates statement shape, membership witnesses, nullifier, balance, and range relation before producing the deterministic artifact. | `cargo test -p z00z_wallets --release --features test-fast --test test_spend_proof_backend -- --nocapture && cargo test -p z00z_wallets --release --features test-fast --test test_tx_proof_verifier -- --nocapture` | green |
| 040-11 | 10 | T3 | Public Input, Membership, And Digest Discipline | The internal relation path carries explicit membership witnesses against `prev_root`, deterministic replay-safe nullifier semantics, and keeps `build_tx_package_digest(...)` as the only public proof-binding root; public proof-of-knowledge remains open. | `cargo test -p z00z_wallets --release --features test-fast --test test_spend_witness_gate -- --nocapture && cargo test -p z00z_wallets --release --features test-fast --test test_tx_tamper --test test_tx_wrong_root -- --nocapture` | green |
| 040-12 | 10 | T4 | Checkpoint And Rollup Boundary | Scenario runtime consumes the canonical internal proof package path, but checkpoint theorem finality and rollup settlement proof closure are not claimed by this internal wallet relation. | `cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_dump && cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump` | green for simulator; public checkpoint/rollup theorem closure open |
| 040-13 | 10 | T6 | Missing-Code Closure Tasks | The non-empty carrier, producer, verifier, checkpoint hook reuse, and bounded output follow-up are each mapped to landed owners or explicit non-goals | closeout and TODO source-shape grep | green |
| 040-14 | 10 | T6 | Prohibited Shortcut Checklist | Shortcut guards reject STARK overclaim, `receiver_cards`, separate `C_fee`, mixed leaf-ad runtime paths, and parallel checkpoint proof lanes | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_stage_surface -- --nocapture` | green |
| 040-RG | 10 | T5 | Rollup Public-Artifact Binding Guard | Rollup settlement admission binds the wallet public spend-theorem contract evidence to checkpoint artifact, link, exec ID, spend and checkpoint roots, and tx inclusion without claiming a public proof-of-knowledge backend | `cargo test -p z00z_rollup_node --release --test test_settlement_theorem -- --nocapture` | green for public-artifact binding; full settlement proof closure remains open |
| 040-CG | 10 | T6 | Completion Gate Alignment | Current phase authorities, runtime artifacts, and tests converge on one internal theorem-relation suite while keeping public/trustless proof-of-knowledge, checkpoint theorem finality, and full rollup settlement proof closure open. | `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh` | green for internal relation; public proof/checkpoint/rollup closure open |

## Manual-Only Verifications

All current Phase 040 implementation requirements have an owning automated
anchor. The remaining manual-only items are explicit open boundaries, not
missing Nyquist tests for the internal relation:

| Behavior | Requirement | Why Manual | Test Instructions |
| --- | --- | --- | --- |
| Public/trustless proof-of-knowledge | Stronger theorem verification beyond internal proof-generation relation | The current verifier checks deterministic canonical artifacts and public binding, not a cryptographic proof of witness knowledge | Keep this out of completed wording until a verifier-side public proof path lands with focused proof-of-knowledge tests |
| Checkpoint theorem finality | Stronger checkpoint/state-transition theorem closure | The current simulator path proves package-coupled continuity and authoritative fail-closed behavior, not a standalone trustless checkpoint theorem | Keep checkpoint finality as open follow-up unless Stage 11 consumes a public theorem artifact and checkpoint tests prove it |
| Full rollup settlement proof closure | Stronger settlement proof beyond public-artifact binding | `test_settlement_theorem.rs` covers public-artifact binding; a final rollup proof system is still out of scope for this internal wallet relation | Keep rollup proof closure open until settlement admission verifies the stronger proof path end to end |

## Validation Sign-Off

- [x] Input state detected as active final-closure execution
- [x] Test infrastructure detected
- [x] State A audit completed against existing `040-VALIDATION.md`
- [x] Requirement-to-test map covers `040-01` through `040-14`, the focused rollup public-artifact binding guard, and the completion gate
- [x] No generated test files were required; existing anchors cover the current internal theorem-relation scope
- [x] Sampling continuity is preserved through the internal theorem-relation migration gates
- [x] `nyquist_compliant: true` set in frontmatter

Approval: passed for internal theorem-relation closure on 2026-04-28; public/trustless proof-of-knowledge, checkpoint theorem finality, and rollup settlement closure remain open.

## Nyquist Audit Trail

### 2026-04-28 State A Audit

- Input state: State A (`040-VALIDATION.md` existed before this audit).
- Gap classification: no MISSING implementation-test rows for the internal
   theorem-relation scope.
- Repaired validation-map drift by adding explicit rows for `040-05`, `040-06`,
   `040-07`, `040-08`, `040-13`, `040-14`, and the focused rollup
   public-artifact binding guard.
- No new tests were generated because the owning Rust test files already exist
   and map to the current Phase 040 requirements.
- The stronger public/trustless proof-of-knowledge, checkpoint theorem
   finality, and full rollup settlement proof closure stay manual/open follow-up
   boundaries rather than hidden validation gaps.
