---
phase: 055
slug: 055-hjmt-boundary
status: verified
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-11
---

# Phase 055 — Validation Strategy

> Per-phase Nyquist validation contract reconstructed from the executed Phase
> 055 plans, summaries, live test homes, and current release-mode reruns.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` + `cargo bench --no-run` |
| **Config file** | `Cargo.toml` |
| **Quick run command** | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` |
| **Full suite command** | see `Current Evidence Commands` |
| **Estimated runtime** | ~240 seconds |

---

## Sampling Rate

- **After every task commit:** Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- **After every plan wave:** Run the phase-local release matrix in `Current Evidence Commands`
- **Before `/gsd-verify-work`:** Phase-local release matrix must be green, and the recorded full-workspace `cargo test --release` evidence must still be valid
- **Max feedback latency:** ~240 seconds

---

## Evidence Basis

- **Current rerun on 2026-06-11:** this validation pass reran the bootstrap
  gate, all current Phase 055 storage owner-home release tests, all current
  Phase 055 simulator owner-home release tests, and both canonical bench
  `--no-run` compile gates.
- **Execution-backed closeout evidence from 2026-06-10:** the numbered phase
  summaries and `055-SUMMARY.md` already record a green full-workspace
  `cargo test --release`, the canonical filtered bench evidence run, and the
  standalone release `scenario_1` acceptance run.
- **Truthfulness rule:** `✅ green` below means either a current rerun in this
  pass or an execution-backed release artifact explicitly named in the phase
  summaries. No row is marked green from file presence alone.

---

## Current Evidence Commands

```bash
./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh

cargo test -p z00z_storage --release --features test-params-fast \
  --test test_live_guardrails \
  --test test_hjmt_batch_proof \
  --test test_hjmt_batch_proof_negative \
  --test test_hjmt_live_proof_families \
  --test test_hjmt_proofs \
  --test test_bench_lanes \
  -- --nocapture

cargo test -p z00z_simulator --release --features test-params-fast \
  --features wallet_debug_tools \
  --test test_fixture_cache_contract \
  --test test_scenario_settlement \
  --test test_scenario1_stage_surface \
  -- --nocapture

cargo bench -p z00z_storage --bench settlement_proofs --no-run
cargo bench -p z00z_storage --bench settlement_hjmt --no-run
```

Execution-backed release evidence retained from the closeout summaries:

```bash
cargo test --release
cargo run --release -p z00z_simulator --bin scenario_1 \
  --features test-params-fast --features wallet_debug_tools
./crates/z00z_storage/scripts/run_storage_settlement_bench.py \
  --bench settlement_proofs --log-base settlement_proofs_batch \
  -- hjmt_batch_ --quick --noplot --warm-up-time 0.01 --measurement-time 0.02
```

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| `055-01-01` | `01` | `1` | `PH55-01` | `T-055-01`, `T-055-02`, `T-055-03` | `BatchProofBlobV1` stays deterministic, additive to `ProofBlob`, and fail-closed for future-generation or shard-context drift. | unit + integration | `cargo test -p z00z_storage --release --features test-params-fast --test test_live_guardrails --test test_hjmt_batch_proof -- --nocapture` | ✅ | ✅ green |
| `055-02-01` | `02` | `2` | `PH55-02` | `T-055-04`, `T-055-05`, `T-055-06` | The batch verifier accepts or rejects atomically, enforces limits and canonical ordering, and never exposes partial acceptance. | integration | `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof -- --nocapture` | ✅ | ✅ green |
| `055-02-02` | `02` | `2` | `PH55-02` | `T-055-04`, `T-055-08` | The `BPB-T-*` corpus binds canonical source bytes, exact mutation points, reject stages, and expected typed errors to the live verifier. | fixture + integration | `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof_negative -- --nocapture` | ✅ | ✅ green |
| `055-03-01` | `03` | `3` | `PH55-03` | `T-055-07` | The additive builder derives from live single-proof truth and does not replace `settlement_proof_blobs`. | integration | `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof -- --nocapture` | ✅ | ✅ green |
| `055-03-02` | `03` | `3` | `PH55-03` | `T-055-08`, `T-055-09` | Positive fixtures, migration vectors, and baseline parity remain deterministic and root-equivalent across `ProofBlob`, `Vec<ProofBlob>`, and `BatchProofBlobV1`. | fixture + integration | `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof --test test_hjmt_live_proof_families --test test_hjmt_proofs -- --nocapture` | ✅ | ✅ green |
| `055-04-01` | `04` | `4` | `PH55-04` | `T-055-11`, `T-055-12` | Canonical batch bench lanes stay truthful, bounded to the intended live scope, and attached to the existing settlement bench homes. | integration + bench-guard | `cargo test -p z00z_storage --release --features test-params-fast --test test_bench_lanes -- --nocapture && cargo bench -p z00z_storage --bench settlement_proofs --no-run && cargo bench -p z00z_storage --bench settlement_hjmt --no-run` | ✅ | ✅ green |
| `055-04-02` | `04` | `4` | `PH55-04` | `T-055-05`, `T-055-09`, `T-055-10`, `T-055-12` | Stage 13 artifacts must carry batch comparison, proof-size, replay, and tamper evidence, and runner verification must fail when that evidence drifts. | e2e + integration | `cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools --test test_fixture_cache_contract --test test_scenario_settlement --test test_scenario1_stage_surface -- --nocapture` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements.

---

## Manual-Only Verifications

All phase behaviors have automated verification.

---

## Validation Audit 2026-06-11

| Metric | Count |
|--------|-------|
| Plan auto-tasks audited | 7 |
| Phase requirements covered | 4 |
| Gaps found | 0 |
| Resolved in this pass | 0 |
| Escalated manual-only | 0 |

---

## Validation Sign-Off

- [x] All tasks have automated verification or execution-backed release evidence
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all missing references because no missing references remain
- [x] No watch-mode flags are used
- [x] Feedback latency stays within the documented phase-local budget
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-06-11
