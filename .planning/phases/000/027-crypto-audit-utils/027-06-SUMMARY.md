---
phase: 027-crypto-audit-utils
plan: "06"
subsystem: utils
tags: [rust, utils, codec, logger, json-policy, validation]
requires:
  - phase: 027-05
    provides: persisted logger hardening and explicit logger-format groundwork for the final boundary wave
provides:
  - explicit narrow `z00z_utils` JSON compatibility policy for `Value` and `json!()`
  - logger macros routed through the owned codec boundary instead of raw `serde_json`
  - codec and logger regressions that pin the sanctioned JSON surface
affects: [z00z_utils, z00z_wallets, phase-027-closeout]
tech-stack:
  added: []
  patterns: [owned-json-compat-surface, macro-route-through-crate-boundary, prefixed-rpc-log-reader-tests]
key-files:
  created:
    - .planning/phases/027-crypto-audit-utils/027-06-SUMMARY.md
  modified:
    - crates/z00z_utils/src/codec/mod.rs
    - crates/z00z_utils/src/logger/macros.rs
    - crates/z00z_utils/src/lib.rs
    - crates/z00z_utils/tests/test_codec_integration.rs
    - crates/z00z_utils/tests/test_logger_integration.rs
    - crates/z00z_wallets/src/services/session_service.rs
    - crates/z00z_wallets/src/core/address/stealth_trust.rs
    - crates/z00z_wallets/src/services/wallet_service.rs
    - crates/z00z_wallets/tests/test_open_wallet_source_discovery.rs
    - crates/z00z_wallets/tests/test_rpc_logging_configured_path.rs
    - crates/z00z_wallets/tests/test_rpc_logging_file_sink.rs
    - crates/z00z_wallets/tests/test_rpc_logging_replay_audit.rs
key-decisions:
  - "Keep `Value` and `json!()` as an explicit narrow compatibility exception owned by `z00z_utils::codec` instead of claiming workspace-wide `serde_json` removal in this wave."
  - "Route all `z00z_utils` logger macros through `$crate::codec::json!` so internal code follows the same owned boundary it exposes publicly."
  - "Clear release-gate fallout in `z00z_wallets` before closing the phase so Phase 027 ends on a truthful green workspace release gate instead of a documented waiver."
requirements-completed: [PH27-JSON]
duration: multi-session
completed: 2026-03-29
---

# Phase 027 Plan 06 Summary

📌 **Closed the final `z00z_utils` audit wave by making the JSON exception explicit, routing logger macros through the owned codec boundary, and ending Phase 027 on a green release gate after clearing stale `z00z_wallets` blocker fallout.**

## Performance

- **Duration:** multi-session
- **Completed:** 2026-03-29
- **Tasks:** 2
- **Files modified:** 12

## Accomplishments

- ✅ Documented `z00z_utils::codec::{Value, json}` as an intentional compatibility surface rather than an undocumented `serde_json` leak.
- ✅ Rewired every `z00z_utils` logger macro to use `$crate::codec::json!` instead of `::serde_json::json!` directly.
- ✅ Added regressions that pin both the codec compatibility surface and macro-emitted structured payloads.
- ✅ Cleared the previously blocking `z00z_wallets` release fallout so the final `cargo test --release --features test-fast --features wallet_debug_dump` gate completed green.

## Files Created/Modified

- `crates/z00z_utils/src/codec/mod.rs` - records the owned JSON compatibility policy and keeps the sanctioned `Value` / `json!()` boundary explicit.
- `crates/z00z_utils/src/logger/macros.rs` - routes every structured logging macro through `$crate::codec::json!`.
- `crates/z00z_utils/src/lib.rs` - documents the narrow JSON exception in the crate-level utility overview.
- `crates/z00z_utils/tests/test_codec_integration.rs` - adds a compatibility-surface roundtrip regression for `Value` / `json!()`.
- `crates/z00z_utils/tests/test_logger_integration.rs` - adds a real logger-facing macro regression that parses emitted structured payloads.
- `crates/z00z_wallets/src/services/session_service.rs` - aligns fail-closed session clock messaging to the canonical `clock unavailable` wording.
- `crates/z00z_wallets/src/core/address/stealth_trust.rs` - fixes the expired-card fixture to remain correctly signed after metadata is set.
- `crates/z00z_wallets/src/services/wallet_service.rs` - aligns a stale unlocked-state test to real session-backed wallet behavior.
- `crates/z00z_wallets/tests/test_open_wallet_source_discovery.rs` - fixes the persisted wallet fixture to carry a valid password verifier.
- `crates/z00z_wallets/tests/test_rpc_logging_configured_path.rs` - accepts prefixed persisted logger lines by parsing from the first JSON object boundary.
- `crates/z00z_wallets/tests/test_rpc_logging_file_sink.rs` - accepts prefixed persisted logger lines in the file-sink suite.
- `crates/z00z_wallets/tests/test_rpc_logging_replay_audit.rs` - accepts prefixed persisted logger lines in audit-tail verification and removes one deprecated compatibility call.

## Decisions Made

- 📌 Phase 027 ends with a narrow, explicit JSON compatibility exception, not a broad rewrite of every `serde_json` consumer in the workspace.
- 📌 Internal `z00z_utils` helpers must obey the same sanctioned boundary they expose publicly; logger macros no longer bypass it.
- 📌 Clearing a release blocker is part of honest phase closeout when the blocker is stale downstream fallout from already-landed contracts.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] `z00z_wallets` release-gate tests still failed on stale session, expiry, and fixture assumptions before Phase 027 could close truthfully**

- **Found during:** pre-plan-06 release-gate triage
- **Issue:** the wider release gate still failed in `z00z_wallets` on a stale unlocked-state test, a divergent fail-closed clock error message, and an expired-card fixture that mutated signed metadata after signing.
- **Fix:** aligned the session error wording, updated the wallet-state test to use a real unlocked session, and rebuilt the expired-card test fixture so it remains validly signed.
- **Files modified:** `crates/z00z_wallets/src/services/session_service.rs`, `crates/z00z_wallets/src/services/wallet_service.rs`, `crates/z00z_wallets/src/core/address/stealth_trust.rs`
- **Verification:** targeted release tests for `test_tofu_reject_expired_card`, `test_get_wallet_state`, and `test_verify_session_fails_closed_when_clock_unavailable`

**2. [Rule 3 - Blocking] Wallet source-discovery fixtures persisted an invalid password verifier and broke post-import unlock checks**

- **Found during:** wider release-gate retry
- **Issue:** `test_open_wallet_source_discovery` wrote a snapshot with an all-zero verifier, so imported wallets failed unlock with `InvalidPassword` after restore.
- **Fix:** derived the fixture verifier with the same domain-separated password-verifier formula used by production.
- **Files modified:** `crates/z00z_wallets/tests/test_open_wallet_source_discovery.rs`
- **Verification:** `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --test test_open_wallet_source_discovery -- --nocapture`

**3. [Rule 3 - Blocking] File-backed wallet RPC logging tests still assumed pure JSONL after the persisted logger format hardened to prefixed lines**

- **Found during:** wider release-gate retry
- **Issue:** three wallet RPC logging tests treated each persisted line as a raw JSON object and failed on `[timestamp] [LEVEL] {json}` output.
- **Fix:** updated each file-backed consumer to parse from the first `{` boundary, matching the now-canonical persisted logger format.
- **Files modified:** `crates/z00z_wallets/tests/test_rpc_logging_configured_path.rs`, `crates/z00z_wallets/tests/test_rpc_logging_file_sink.rs`, `crates/z00z_wallets/tests/test_rpc_logging_replay_audit.rs`
- **Verification:** targeted release tests for the three wallet RPC logging suites plus the final workspace release gate

## Validation Evidence

- ✅ `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` -> passed
- ✅ `cargo test -p z00z_utils --release --test test_codec_integration` -> passed
- ✅ `cargo test -p z00z_utils --release --test test_logger_integration` -> passed
- ✅ `rg '(^|[^[:alnum:]_])(::)?serde_json::json!|use serde_json::json|use serde_json::\{[^}]*json' crates/z00z_utils/src/logger/macros.rs` -> no matches
- ✅ `cargo test --release --features test-fast --features wallet_debug_dump` -> passed

## Next Phase Readiness

- ✅ `PH27-JSON` is closed and Phase 027 now has all seven mapped requirements complete.
- ✅ The wider workspace release gate is green, so Phase 027 can close without an external-blocker waiver.
- ✅ No active deferred blocker remains under `.planning/phases/027-crypto-audit-utils/`.

## Self-Check: PASSED

- ✅ Summary artifact created at `.planning/phases/027-crypto-audit-utils/027-06-SUMMARY.md`
- ✅ Phase-owned `z00z_utils` boundary changes and prerequisite wallet-blocker cleanup are both documented
- ✅ Final workspace release gate recorded as green for the completed phase

---

*Phase: 027-crypto-audit-utils*
*Completed: 2026-03-29*
