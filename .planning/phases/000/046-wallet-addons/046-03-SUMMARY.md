---
phase: 046-wallet-addons
plan: 3
type: summary
status: completed
updated: 2026-05-15
---

# 046-03 Summary

## ✅ Outcome

046-03 is complete. `WalletPlusHistory` restore now fails closed on the wallet-owned boundary, preserves the encrypted `.wlt` claim set plus canonical tx-history JSONL atomically, and keeps scan-resume authority anchored to the live wallet-side `recv_range(...)` cursor path.

## ✅ Landed Changes

- Added an atomic `WalletPlusHistory` restore path that validates the snapshot and JSONL payload before live mutation, stages both `.wlt` and history writes, commits them in one fail-closed sequence, and rolls back durable state if the final `.wlt` publish or in-memory restore step fails.
- Tightened restore prevalidation so malformed claimed assets and duplicate claimed-asset ids reject before any live wallet or history mutation.
- Kept `restore_snapshot(...)` internal to the crate while exposing it to the restore action seam, so the atomic restore flow can reuse the canonical in-memory publish path instead of introducing a second restore authority.
- Extended restore regression coverage to prove restored claimed-asset retention, canonical JSONL replay, wrong-password failure closure, blocked history-commit failure closure, duplicate-claim rejection, and preservation of existing durable state when `.wlt` staging fails.
- Strengthened the Stage 13 scan helper notes so simulator evidence now states explicitly that `read_scan_state(...)` and `upsert_scan_state(...)` remain the live resume-cursor path, while `ScanStorageImpl` stays a separate local scan-state store and `ReceiveNext::PersistClaim` remains the claim persistence seam.

## ✅ Validation

- `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump restore_backup_with_wallet_plus_history` completed with exit code `0`.
- `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump test_recv_range_restart` completed with exit code `0`.
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump stage13` completed with exit code `0`.
- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` completed with exit code `0`.
- `cargo test --release --features test-fast --features wallet_debug_dump` completed with exit code `0`.
- Three review passes following `.github/prompts/gsd-review-tasks-execution.prompt.md` ran in YOLO mode: pass 1 found and fixed the duplicate-claim prevalidation gap, and passes 2 plus 3 were consecutive clean passes with no further material 046-03 findings.

## ⚠️ Boundary Kept Intact

- The encrypted `.wlt` snapshot remains the canonical claimed-asset persistence plane; the restore path does not introduce a second asset store or a second full-restore authority.
- Wallet-side receive restart remains anchored to the existing `recv_range(...)` flow and `test_recv_range_restart`; the Stage 13 helper only clarifies the boundary in simulator evidence and does not create a scanner-owned resume lane.

## 🔜 Next Phase

046-04 can now focus on payment-request / TOFU hardening, session limits, and rotate-master-key audit boundaries while reusing the now-closed restore and scan-resume baseline.
