---
phase: 027-crypto-audit-utils
plan: "05"
subsystem: utils
tags: [rust, utils, logger, io, simulator, validation]
requires:
  - phase: 027-04
    provides: deterministic-RNG guardrails and the frozen Phase 027 utility contract baseline
provides:
  - persisted logger sanitization that strips ANSI and control-byte injection without losing trailing content
  - rotating persisted log lines with restored severity prefixes
  - explicit structured logging failure payloads pinned to the stable `logger.serialize_error` sentinel
  - generic write helpers that propagate permission-copy failures instead of swallowing them
  - simulator-side RPC log parsing that accepts the persisted prefixed log format used by the hardened logger path
affects: [027-06, z00z_utils, z00z_simulator]
tech-stack:
  added: []
  patterns: [persisted-log-sanitization, explicit-logger-failure-sentinel, permission-copy-propagation, prefixed-rpc-log-parsing]
key-files:
  created:
    - .planning/phases/027-crypto-audit-utils/027-05-SUMMARY.md
  modified:
    - crates/z00z_utils/src/logger/mod.rs
    - crates/z00z_utils/src/logger/file_logger.rs
    - crates/z00z_utils/src/logger/rotating_file_logger.rs
    - crates/z00z_utils/src/logger/structured.rs
    - crates/z00z_utils/src/io/fs.rs
    - crates/z00z_utils/tests/test_logger_integration.rs
    - crates/z00z_utils/tests/test_io_integration.rs
    - crates/z00z_simulator/src/scenario_1/stage_2_utils/artifacts.rs
    - crates/z00z_simulator/tests/support/test_stage4_support.rs
    - crates/z00z_simulator/tests/test_stage4_bob_flow.rs
    - crates/z00z_simulator/tests/test_stage4_selection.rs
key-decisions:
  - "Keep the logger serialization failure contract pinned to the stable `logger.serialize_error` sentinel instead of introducing a softer fallback event shape."
  - "Treat persisted RPC logs as the new source of truth for simulator validation and parse the prefixed logger format explicitly instead of assuming raw JSONL."
  - "Close `027-05` on targeted in-scope validation plus documented external blocker capture rather than holding the plan open for unrelated `z00z_wallets` release-test failures."
patterns-established:
  - "Persisted log readers must accept the shared on-disk logger prefix format before decoding embedded JSON payloads."
  - "Generic write helpers surface permission-copy failures explicitly; secret durability remains owned by the private atomic path."
requirements-completed: [PH27-LOGGER, PH27-IO]
duration: multi-session
completed: 2026-03-29
---

# Phase 027 Plan 05 Summary

📌 **Closed the logger and file-I/O hardening wave with stronger persisted-log sanitization, explicit permission-copy failure propagation, and simulator validation updated to the prefixed RPC-log format now written on disk.**

## Performance

- **Duration:** multi-session
- **Completed:** 2026-03-29T17:06:36Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments

- ✅ Hardened persisted log sanitization so ANSI and broader control-byte injection are neutralized before file writes.
- ✅ Restored severity prefixes in rotating log output and kept structured serialization failures pinned to the explicit `logger.serialize_error` sentinel.
- ✅ Made generic `write_file` permission-copy failures observable instead of silently ignored and kept the private atomic write path as the stronger secret-bearing contract.
- ✅ Fixed simulator Stage 2 and Stage 4 RPC-log readers so they parse the persisted prefixed logger format emitted by the hardened logger pipeline.

## Task Commits

📌 This execution closed the plan from validated working-tree state.

1. **Task 1: Extend persisted log sanitization and restore rotating severity prefixes** - not separately committed in this execution
2. **Task 2: Make generic write semantics explicit and stop swallowing permission-copy failures** - not separately committed in this execution

**Plan metadata:** not committed in this execution; repo-owned git or versioning checkpoint remains deferred.

## Files Created/Modified

- `crates/z00z_utils/src/logger/mod.rs` - extended persisted message sanitization and fixed malformed escape handling so incomplete ANSI sequences no longer truncate trailing content.
- `crates/z00z_utils/src/logger/file_logger.rs` - documented the current final-component symlink and trusted-parent boundary explicitly.
- `crates/z00z_utils/src/logger/rotating_file_logger.rs` - restored `[timestamp] [LEVEL] ...` persisted severity formatting.
- `crates/z00z_utils/src/logger/structured.rs` - preserved the stable `logger.serialize_error` failure contract on structured serialization failure.
- `crates/z00z_utils/src/io/fs.rs` - surfaced permission-copy failures and clarified generic versus private write-path durability semantics.
- `crates/z00z_utils/tests/test_logger_integration.rs` - pinned persisted sanitizer, rotating severity, and explicit structured-failure regressions.
- `crates/z00z_utils/tests/test_io_integration.rs` - pinned deterministic permission-copy failure and durability-contract regressions.
- `crates/z00z_simulator/src/scenario_1/stage_2_utils/artifacts.rs` - added dual-format RPC log parsing so Stage 2 privacy validation accepts prefixed persisted logger lines.
- `crates/z00z_simulator/tests/support/test_stage4_support.rs` - added shared Stage 4 RPC request-row parsing for prefixed persisted logs.
- `crates/z00z_simulator/tests/test_stage4_bob_flow.rs` - switched Bob-flow RPC assertions to the shared prefixed-log reader.
- `crates/z00z_simulator/tests/test_stage4_selection.rs` - switched selection-path RPC assertions to the shared prefixed-log reader.

## Decisions Made

- 📌 The persisted logger format with visible severity stays the canonical on-disk contract, so simulator and test readers must adapt to it instead of forcing the logger back to raw JSON lines.
- 📌 The permission-copy failure proof remains deterministic through an explicit test seam rather than relying on unstable host-permission behavior.
- 📌 Full-phase honesty in this wave means recording unrelated wider-gate failures, not reopening the logger or I/O work that already validated green inside scope.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Unterminated ANSI sequences could strip the tail of persisted log messages**

- **Found during:** Task 1 rereview
- **Issue:** the first sanitizer hardening pass could consume a malformed escape sequence and drop the remaining tail of the message.
- **Fix:** treated incomplete escape sequences as escaped control-byte content so persisted logs keep the full message tail while still neutralizing injection.
- **Files modified:** `crates/z00z_utils/src/logger/mod.rs`, `crates/z00z_utils/tests/test_logger_integration.rs`
- **Verification:** `cargo test -p z00z_utils --release --test test_logger_integration`
- **Committed in:** not committed in this execution

**2. [Rule 1 - Bug] The first permission-copy failure proof leaked a magic-filename seam into production behavior**

- **Found during:** Task 2 rereview
- **Issue:** the initial deterministic proof strategy depended on a production-visible filename convention and could affect real writes.
- **Fix:** replaced it with a path-local crate-private seam used only by tests and kept the public API unchanged.
- **Files modified:** `crates/z00z_utils/src/io/fs.rs`, `crates/z00z_utils/tests/test_io_integration.rs`
- **Verification:** `cargo test -p z00z_utils --release --test test_io_integration`; `cargo test -p z00z_utils --release`
- **Committed in:** not committed in this execution

**3. [Rule 3 - Blocking] The required full release gate first failed outside scope in simulator acceptance because Stage 2 parsed persisted RPC logs as raw JSON lines**

- **Found during:** plan-level release validation
- **Issue:** `validate_rpc_log_privacy` in Scenario 1 Stage 2 assumed `rpc_logger.json` contained raw JSONL, but the hardened logger now persists `[timestamp] [LEVEL] {json}` lines.
- **Fix:** added a dual-format parser in `crates/z00z_simulator/src/scenario_1/stage_2_utils/artifacts.rs` and pinned both raw and prefixed cases with tests.
- **Files modified:** `crates/z00z_simulator/src/scenario_1/stage_2_utils/artifacts.rs`
- **Verification:** `cargo test -p z00z_simulator --release --features test-fast --test test_claim_acceptance -- --nocapture`
- **Committed in:** not committed in this execution

**4. [Rule 3 - Blocking] Stage 4 integration tests also assumed raw JSONL RPC logs after the Stage 2 blocker was cleared**

- **Found during:** full release-gate retry after the Stage 2 fix
- **Issue:** Stage 4 integration tests filtered `rpc_logger.json` through direct JSON deserialization and missed all prefixed persisted rows.
- **Fix:** introduced a shared Stage 4 test-support parser and moved the affected tests onto that reader.
- **Files modified:** `crates/z00z_simulator/tests/support/test_stage4_support.rs`, `crates/z00z_simulator/tests/test_stage4_bob_flow.rs`, `crates/z00z_simulator/tests/test_stage4_selection.rs`
- **Verification:** `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage4_bob_flow --test test_stage4_selection -- --nocapture`
- **Committed in:** not committed in this execution

---

**Total deviations:** 4 auto-fixed (2 bug, 2 blocking)
**Impact on plan:** the logger and I/O changes stayed in scope, but honest closure required adapting simulator validation to the new persisted log contract that this wave intentionally hardened.

## Issues Encountered

- ⚠️ The first wider-gate blocker turned out to be real downstream fallout from the new persisted log format, not a false negative in the logger wave itself.
- ⚠️ After the simulator blockers were cleared, the full workspace release command still failed outside `027-05` scope in `z00z_wallets --lib` with three unrelated tests: `core::address::stealth_trust::tests::test_tofu_reject_expired_card`, `services::wallet_service::tests::test_get_wallet_state`, and `services::wallet_service::tests::test_verify_session_fails_closed_when_clock_unavailable`.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- ✅ `PH27-LOGGER` and `PH27-IO` are now closed with in-scope code and regression coverage.
- ✅ Phase `027-06` can start; the remaining wider-gate failure is outside the logger and I/O boundary and is recorded as an external blocker.
- ⚠️ The full workspace release command remains non-green until the unrelated `z00z_wallets` release tests above are triaged separately.

## Validation Evidence

- ✅ `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` -> passed earlier in the 027-05 closeout cycle
- ✅ `cargo test -p z00z_utils --release --test test_logger_integration` -> passed after the final sanitizer fix
- ✅ `cargo test -p z00z_utils --release --test test_io_integration` -> passed after the final permission-copy seam fix
- ✅ `cargo test -p z00z_utils --release` -> passed
- ✅ `cargo test -p z00z_simulator --release --features test-fast --test test_claim_acceptance -- --nocapture` -> passed (`5 passed; 0 failed`)
- ✅ `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage4_bob_flow --test test_stage4_selection -- --nocapture` -> passed (`3 passed; 0 failed`)
- ⚠️ `cargo test --release --features test-fast --features wallet_debug_dump` -> still blocked outside `027-05` scope by unrelated `z00z_wallets --lib` failures listed above

## Self-Check: PASSED

- ✅ Summary artifact created at `.planning/phases/027-crypto-audit-utils/027-05-SUMMARY.md`
- ✅ Validation evidence recorded against the final logger, I/O, and simulator log-reader tree
- ✅ No commit hashes were claimed because git checkpointing was not performed in this execution

---

*Phase: 027-crypto-audit-utils*
*Completed: 2026-03-29*