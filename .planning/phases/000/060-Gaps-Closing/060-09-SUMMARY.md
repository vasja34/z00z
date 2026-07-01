---
phase: 060-Gaps-Closing
plan: 060-09
status: complete
completed_at: 2026-06-20
next_plan: 060-10
summary_artifact_for: .planning/phases/060-Gaps-Closing/060-09-PLAN.md
---

# 060-09 Summary: Adversarial High-Finding Closure And Count Reconciliation

## Completed Scope

`060-09` is complete for `C3`.

This slice turned the current adversarial high-finding set from prose-only
hypotheses into one explicit closure packet with exact owners, exact closure
modes, and one honest rerun of the generator outputs. The live high set stays
normalized exactly as required by `060-TODO.md`: `7` project-owned highs and
`4` protected-vendor highs. The final rerun on
`reports/z00z-verification-orchestrator-20260620-123133/` reports `392` total
findings with `11` highs, and the summary count now matches the rendered high
section instead of silently drifting.

The only live code change in the adversarial generator is in
`.github/skills/z00z-verification-orchestrator/scripts/run-security-brainstorm.py`.
Fresh verification roots now fall back to the latest canonical
`prompt_corpus.json` and `attack_surface_registry.json` artifact instead of
dropping to an under-specified built-in family list, and `top_findings` now
serializes the full high-severity set instead of truncating to ten rows. The
checkpoint-lineage, compact-request rebinding, stealth request-binding, and
timing or scheduler nondeterminism seams did not require protocol semantic
changes; Phase 060 instead promoted already fail-closed behavior into explicit
negative tests and closure memos tied to exact code and test anchors.

The review loop also exposed a real generator gap. The first fresh rerun at
`20260620-122959` degraded to `389` total findings with only `8` highs because
the new verification root did not yet carry the family registry or prompt
corpus, and the fallback path silently lost the cross-crate high families. The
landed fix restores the canonical `11`-high packet on the next rerun without
inventing a second authority source.

## Closure Ledger

### Project-Owned Highs

- `family:checkpoint-lineage` (`project-owned`): disproved. Threat statement:
  a checkpoint exec input or branch could be rebound across lineage while local
  checks still pass. Evidence: existing
  `test_prev_root_bind_rejects` and `test_root_bind_rejects_branch`, plus new
  `test_exec_codec_rejects_prev_root_rebinding`, prove `prev_root` tampering is
  rejected before acceptance and JSON exec-input rebinding fails with
  `CheckpointError::RootMix`.
- `family:payment-request-replay` (`project-owned`): disproved. Threat
  statement: a compact request could be rebound across `chain_id` or `req_id`
  while remaining signature-valid. Evidence:
  `test_compact_request_chain_rebinding_breaks_signature` and
  `test_compact_request_req_id_rebinding_breaks_signature` prove structural
  decode still fails signature verification after rebinding, so the compact
  envelope is not replay-portable across those domains.
- `family:stealth-inbox-delivery` (`project-owned`): disproved. Threat
  statement: request-bound stealth output delivery could be misclassified under
  a different request context. Evidence: existing `test_req_binding`,
  `test_wrong_scan`, and `test_wrong_kdh`, plus new
  `test_req_binding_rejects_unrelated_request_context`, prove the scanner stays
  `NotMine` when the registered request does not match the bound request id.
- `crate:crates/z00z_storage` (`project-owned`): disproved by closure memo.
  Threat statement: crate-level concentration across checkpoint or settlement
  seams could hide an unowned bypass path. Closure result: after the explicit
  checkpoint lineage tests and settlement nondeterminism proofs, no separate
  crate-wide bypass remains beyond the already named and closed concrete seams.
- `module:crates/z00z_storage/src/settlement` (`project-owned`): disproved by
  closure memo. Threat statement: module-level concentration could preserve a
  hidden invariant split across scheduler, timing, proof, or object-contract
  code. Closure result: the remaining suspicious sites collapse into the
  already closed scheduler and timing hypotheses and the previously landed
  fail-closed settlement or object-package guards; there is no extra live
  high-risk path left unowned.
- `nondeterministic-validation-source` at
  `crates/z00z_storage/src/settlement/hjmt_scheduler.rs` (`project-owned`):
  disproved. Threat statement: `Instant::now()` or scheduling skew could change
  accept or reject semantics. Evidence:
  `test_terminal_commits_stable_skew` and `test_batch_verifies_input_order`
  prove stable roots and proof behavior across scheduler interleavings; the
  timing source feeds wait metrics, not validity decisions.
- `nondeterministic-validation-source` at
  `crates/z00z_storage/src/settlement/timing.rs` (`project-owned`): disproved.
  Threat statement: timing instrumentation could influence settlement outputs or
  proof bytes. Evidence:
  `test_timing_instrumentation_is_observability_only` proves timed and untimed
  runs produce the same settlement root and proof blob while the TSV output is
  observability-only.

### Protected-Vendor Highs

- `crate:crates/z00z_crypto/tari/crypto` (`protected-vendor`): accepted-risk
  protected-vendor bucket. No edits are allowed under the Tari vendor root, so
  the crate-level concentration remains explicit and tracked instead of being
  falsely marked fixed.
- `module:crates/z00z_crypto/tari/crypto/src/ristretto`
  (`protected-vendor`): accepted-risk protected-vendor bucket. Phase 060 keeps
  the module concentration visible and relies on wrapper isolation and explicit
  tracking rather than source edits.
- `secret-or-proof-logging` at
  `crates/z00z_crypto/tari/crypto/src/ristretto/ristretto_keys.rs:895`
  (`protected-vendor`): accepted-risk protected-vendor bucket. Inspection
  confirmed the `println!` lives inside `#[cfg(feature = "serde")] mod
  test_serialize`, so the flagged sink is test-only vendor code, not a project
  runtime logging path.
- `unsafe-in-critical-surface` at
  `crates/z00z_crypto/tari/crypto/src/ristretto/ristretto_keys.rs:847`
  (`protected-vendor`): accepted-risk protected-vendor bucket. Inspection
  confirmed the `unsafe` block lives inside `#[test] fn
  secret_keys_are_cleared_after_drop()` and executes only in the test harness;
  the vendor root remains protected, so the finding stays explicit instead of
  being patched locally.

## Files Changed

- `.planning/phases/060-Gaps-Closing/060-09-SUMMARY.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.github/skills/z00z-verification-orchestrator/scripts/run-security-brainstorm.py`
- `crates/z00z_storage/tests/test_async_scheduler.rs`
- `crates/z00z_storage/tests/test_checkpoint_root_binding.rs`
- `crates/z00z_wallets/tests/test_stealth_request.rs`
- `crates/z00z_wallets/tests/test_stealth_scanner_flow.rs`

## Boundary Kept

- No Tari vendor source file was modified.
- No second authority source was introduced for adversarial prompt or family
  data; the rerun falls back to the latest canonical verification artifact
  instead of inventing a new registry.
- No checkpoint, request, or stealth protocol semantics were widened; the slice
  only converted existing fail-closed behavior into explicit closure evidence.
- No parallel module or function path was added; the packet preserves one live
  code path per behavior and one explicit closure ledger per high finding.

## Review Loop

Manual fallback for `/GSD-Review-Tasks-Execution` was used because the slash
prompt is not a callable tool in this environment.

- Pass 1 reviewed the owned high surfaces and landed the first negative tests.
  It exposed one real issue in the new checkpoint tamper harness
  (`serde_json::json!([0xAAu8; 32])`), which was corrected to a canonical
  vector encoding.
- Pass 2 reran the adversarial generator on a fresh verification root and
  exposed the real report bug: the run degraded from the baseline `392 / 11`
  packet to `389 / 8` because the missing registry or prompt-corpus artifacts
  forced an under-specified fallback path. The generator fix landed in
  `run-security-brainstorm.py`.
- Pass 3 reran the generator after the fix, restored the canonical
  `392 total / 11 high` packet, and proved summary or markdown consistency with
  `11 / 11 / 11`.
- Pass 4 reran the broad workspace `cargo test --release`, reran scoped diff
  hygiene, and rechecked the closure ledger against the landed test names and
  vendor anchors. No significant issues remained.

Two consecutive clean review passes were achieved on passes 3 and 4 after the
final fixes landed.

## Validation

- Mandatory bootstrap gate passed:
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- Targeted release validation passed:
  `cargo test -p z00z_storage --release --test test_checkpoint_root_binding -- --nocapture`
  `cargo test -p z00z_storage --release --test test_async_scheduler -- --nocapture`
  `cargo test -p z00z_wallets --release --test test_stealth_request -- --nocapture`
  `cargo test -p z00z_wallets --release --test test_stealth_scanner_flow -- --nocapture`
- Adversarial rerun passed on the fixed generator:
  `python3 ./.github/skills/z00z-verification-orchestrator/scripts/run-security-brainstorm.py --root . --scope-kind project --target-root-rel "" --verification-root reports/z00z-verification-orchestrator-20260620-123133/verification20260620-123133 --summary-out reports/z00z-verification-orchestrator-20260620-123133/security/adversarial-summary.json --report-out reports/z00z-verification-orchestrator-20260620-123133/security/adversarial-review.md`
- Report-consistency check passed with one explicit proof of no hidden unowned
  high: `high_risk_count = 11`, `len(top_findings) = 11`, and markdown
  `## High Findings` entries = `11`.
- Broad workspace release validation passed:
  `cargo test --release`
- Final scoped whitespace check is clean:
  `git diff --check -- .github/skills/z00z-verification-orchestrator/scripts/run-security-brainstorm.py crates/z00z_storage/tests/test_checkpoint_root_binding.rs crates/z00z_storage/tests/test_async_scheduler.rs crates/z00z_wallets/tests/test_stealth_request.rs crates/z00z_wallets/tests/test_stealth_scanner_flow.rs .planning/phases/060-Gaps-Closing/060-09-SUMMARY.md .planning/STATE.md .planning/ROADMAP.md`

## Result

`060-09` is complete. Phase 060 advances to `060-10-PLAN.md` for the HJMT
measurement lanes and A/B rerun packet.
