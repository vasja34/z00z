# Z00Z HJMT Acceptance Thresholds

Version: 2026-06-16

## 🎯 Purpose

This document closes Appendix C row `C-16` from
[Z00Z-HJMT-Upgrade.md](Z00Z-HJMT-Upgrade.md).

It freezes the exact readiness thresholds used by the HJMT upgrade packet.
These thresholds classify repository evidence; they are not protocol
constants.

## 📐 Verdict Thresholds

| Verdict | Minimum threshold |
| --- | --- |
| `contract only` | design and planning contracts exist, but live tests or evidence packets are not yet closed |
| `prototype` | some live tests or artifacts exist, but required gates, fixture classes, or evidence-gap rows remain open or partial |
| `verified slice` | bounded live slices pass, but one or more required gate rows, fixture rows, or `12.1` evidence-gap rows remain open or partial |
| `integrated upgrade` | all gates `058-G1` through `058-G13` are closed, every required fixture family is closed, every required `12.1` evidence-gap class is closed, benchmark archive-home wording is canonical, and the final evidence ledger maps each live claim to one concrete owner home |
| `release-ready` | `integrated upgrade` is already satisfied, final release packets are current, broad release validation is green, and the final review loop finishes clean without reopening evidence rows |

## ✅ Current Accepted Baseline

Phase 058 accepts the following as the current HJMT readiness baseline:

| Baseline rule | Accepted condition |
| --- | --- |
| archive-home honesty | `crates/z00z_storage/outputs/settlement/` is the only canonical measured-report home; `outputs/assets/` is retired wording |
| shared-proof closure | `BPB-G-001..005` plus `BPB-T-001..008` and the final shared-proof closeout report exist together |
| bucket-commit closure | `BCM-G-001` and `BCM-G-002` are frozen as deterministic checked conformance artifacts |
| compatibility-equivalence closure | `CEQ-G-001..008` are frozen as deterministic oracle-backed conformance cases |
| threat-model closure | parser, replay, authority, and transition misuse are mapped to exact owner tests |
| broad validation closure | bootstrap-first validation plus targeted release suites plus `cargo test --release` are green on the accepted tree |

## 🔒 Interpretation Rules

1. A stronger verdict may not be claimed from adjacent evidence.
2. Unsupported compression claims remain disallowed until a versioned measured
   compression lane exists.
3. Deterministic conformance artifacts may close semantic-equivalence rows when
   the row is about correctness rather than throughput.
4. Historical status notes do not override the current accepted baseline in
   this file.

## ✅ Verification Commands

```bash
./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_commit -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_compat_equivalence -- --nocapture
cargo test -p z00z_storage --release --features test-params-fast --test test_bench_lanes -- --nocapture
cargo test --release
```
