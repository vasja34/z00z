---
phase: 027-crypto-audit-utils
plan: "03"
subsystem: utils
tags: [rust, utils, time, fail-closed, wallets, simulator]
requires:
  - phase: 027-01
    provides: lifetime-safe secret handling for later wallet-session hardening work
  - phase: 027-02
    provides: fail-closed config policy baseline for the remaining utils audit waves
provides:
  - explicit fail-closed time-provider contract with `try_unix_timestamp*` as the blessed production path
  - repository scan closure proving the production perimeter no longer uses ambiguous direct `.unix_timestamp*()` helpers
  - downstream classification of compatibility-only operational timestamps versus security-sensitive fail-closed consumers
  - wallet-session hardening so expiry, seed-view throttling, and auto-lock state stay aligned with current time-provider truth
affects: [027-04, 027-05, 027-06, z00z_utils, z00z_core, z00z_storage, z00z_wallets, z00z_simulator]
tech-stack:
  added: []
  patterns: [explicit-compat-time-helpers, fail-closed-wallet-expiry, post-lock-time-sampling, production-perimeter-scan]
key-files:
  created:
    - .planning/phases/027-crypto-audit-utils/027-03-SUMMARY.md
  modified:
    - crates/z00z_utils/src/time/traits.rs
    - crates/z00z_utils/src/time/mod.rs
    - crates/z00z_utils/src/time/system.rs
    - crates/z00z_utils/src/time/test_time.rs
    - crates/z00z_utils/tests/test_time_policy_micros.rs
    - crates/z00z_core/src/genesis/genesis.rs
    - crates/z00z_core/src/assets/registry.rs
    - crates/z00z_storage/src/assets/store_internal/redb_backend.rs
    - crates/z00z_wallets/src/services/session_service.rs
    - crates/z00z_wallets/src/services/wallet_service.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/wallet_impl.rs
key-decisions:
  - "Keep `try_unix_timestamp*` as the only blessed production contract and make lossy helpers explicit through `compat_*` naming instead of ambiguous default wrappers."
  - "Close the wave only after a reproducible first-party production scan shows no direct `.unix_timestamp*()` callers left in the declared perimeter."
  - "Treat wallet session expiry as fail closed by sampling current time inside `WalletSessionManager` after the session lock is acquired, not from caller-provided pre-await timestamps."
patterns-established:
  - "Production time paths must choose explicitly between fail-closed `try_*` and documented compatibility-only `compat_*` helpers."
  - "Async expiry checks sample time at the owner of the protected state after lock acquisition to avoid TOCTOU drift."
  - "Wallet activity refresh happens after successful validation, not from stale pre-check timestamps."
requirements-completed: [PH27-TIME]
duration: multi-session
completed: 2026-03-29
---

# Phase 027 Plan 03 Summary

📌 **Frozen the `z00z_utils` time contract around explicit fail-closed `try_unix_timestamp*` helpers, removed ambiguous direct wrapper usage from the first-party production perimeter, and closed the final wallet-session expiry drift uncovered during downstream review.**

## Performance

- **Duration:** multi-session
- **Completed:** 2026-03-29T12:00:00Z
- **Tasks:** 2
- **Files modified:** 20+

## Accomplishments

- ✅ Promoted `try_unix_timestamp*` to the explicit production contract and renamed lossy helpers to `compat_*` so compatibility paths cannot be mistaken for the blessed security surface.
- ✅ Re-scanned the first-party production perimeter and closed it with no direct `.unix_timestamp*()` or raw epoch-micros callers left outside `z00z_utils/src/time/**`.
- ✅ Migrated or reclassified downstream consumers across `z00z_core`, `z00z_storage`, `z00z_wallets`, and `z00z_simulator`, then widened the closeout to fix review-discovered wallet/session timing bugs instead of treating the original planning inventory as exhaustive.
- ✅ Hardened wallet seed-view, session verification, cached-session reuse, and state-sync logic so expiry checks fail closed and auto-lock state stays aligned with real session truth.

## Task Commits

📌 This execution closed the plan from validated working-tree state.

1. **Task 1: Make the lossy time helpers an explicit compatibility surface in `z00z_utils`** - not separately committed in this execution
2. **Task 2: Classify and migrate the verified downstream time consumers** - not separately committed in this execution

**Plan metadata:** not committed in this execution; repo-owned git or versioning checkpoint remains deferred.

## Files Created/Modified

- `crates/z00z_utils/src/time/traits.rs` - split the public contract into explicit fail-closed `try_*` helpers and compatibility-only `compat_*` helpers, while preserving deprecation-guided migration shims.
- `crates/z00z_utils/tests/test_time_policy_micros.rs` - turned the repository guard into a production-perimeter policy check that rejects direct `.unix_timestamp*()` use and raw `duration_since(...).as_micros()` patterns.
- `crates/z00z_core/src/genesis/genesis.rs`, `crates/z00z_core/src/assets/registry.rs`, `crates/z00z_core/src/assets/snapshot.rs`, `crates/z00z_storage/src/assets/store_internal/redb_backend.rs` - reclassified operational timestamps onto explicit compatibility helpers.
- `crates/z00z_wallets/src/services/session_service.rs` - moved expiry-sensitive current-time sampling inside the session manager after lock acquisition and made time failure fail closed for cached-session reuse and liveness checks.
- `crates/z00z_wallets/src/services/wallet_service.rs` - migrated security-sensitive checks to `try_*`, added expired-session and clock-failure state-sync coverage, and refreshed activity only after successful verification.
- `crates/z00z_wallets/src/adapters/rpc/methods/wallet_impl.rs` - aligned RPC seed-view behavior with the new service ordering and late error mapping.
- `crates/z00z_wallets/tests/test_addr_rate_limit_integration.rs`, `crates/z00z_wallets/tests/test_key_manager.rs`, `crates/z00z_wallets/tests/test_stealth_request.rs` - kept wallet anchors green after the fail-closed rollout and the wallet/session hardening pass.

## Decisions Made

- 📌 The workspace now treats `compat_*` naming itself as part of the contract: compatibility use is allowed only when the call site is explicitly non-security and documented as such.
- 📌 The closure gate for this wave is source-shape plus behavioral: the perimeter scan must be clean, and real wallet/session anchors must still pass after the migration.
- 📌 Review findings discovered in the downstream wallet/session seam were treated as part of honest `PH27-TIME` closure because they were caused by the timing contract rollout and affected fail-closed expiry semantics.

## Final Consumer Classification

| Consumer class | Representative files | Final state | Rationale |
| --- | --- | --- | --- |
| Canonical fail-closed production path | `crates/z00z_core/src/assets/nonce.rs` | `try_unix_timestamp_micros()` retained | Nonce derivation is security-sensitive and remains the model pattern for this phase. |
| Fail-closed wallet expiry and throttling | `crates/z00z_wallets/src/core/wallet/policy.rs`, `crates/z00z_wallets/src/services/session_service.rs`, `crates/z00z_wallets/src/services/wallet_service.rs`, `crates/z00z_wallets/src/core/address/rate_limiter.rs`, `crates/z00z_wallets/src/core/address/stealth_request.rs` | migrated to `try_*` or explicit fail-closed handling | These paths gate expiry, rate limits, seed access, session validity, or anti-replay behavior. |
| Fail-closed sentinel on clock failure | `crates/z00z_wallets/src/core/address/stealth_trust.rs`, `crates/z00z_wallets/src/core/address/stealth_card.rs` | explicit `try_*` with fail-closed sentinel | Clock failure now marks trust or validity windows as expired instead of silently extending them. |
| Compatibility-only operational metadata | `crates/z00z_core/src/genesis/genesis.rs`, `crates/z00z_core/src/assets/registry.rs`, `crates/z00z_core/src/assets/snapshot.rs`, `crates/z00z_storage/src/assets/store_internal/redb_backend.rs` | retained on `compat_*` | These timestamps label artifacts, durations, or persistence metadata and do not gate security policy. |
| Compatibility-only wallet or RPC metadata | `crates/z00z_wallets/src/services/app_service.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl.rs`, `crates/z00z_wallets/src/db/redb_wallet_store.rs`, `crates/z00z_wallets/src/core/storage/tx_storage_impl.rs` | retained on `compat_*` after classification | These sites stamp logs, cache entries, UI-visible events, or persisted metadata rather than enforcing security-critical deadlines. |
| Mixed wallet subsystems | `crates/z00z_wallets/src/core/address/address_manager.rs`, `crates/z00z_wallets/src/core/storage/secret_store_impl.rs`, `crates/z00z_wallets/src/core/key/key_manager.rs` | split into `try_*` for security decisions and `compat_*` for bookkeeping | The wave separated expiry or backoff logic from operational age, cache, or report timestamps instead of flattening both onto one helper. |
| Compatibility-only simulator or CLI artifacts | `crates/z00z_simulator/src/scenario_1/stage_2_utils/flow.rs`, `crates/z00z_simulator/src/scenario_1/stage_3.rs`, `crates/z00z_simulator/src/scenario_1/stage_3_utils/wallet_flow.rs`, `crates/z00z_core/bin/assets/assets_generation_cli.rs` | retained on `compat_*` | These surfaces produce artifact timestamps or operator-facing output and are intentionally outside the security-critical set for this wave. |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] The mandatory production-perimeter scan found additional live callers beyond the planning-time inventory**

- **Found during:** Task 2 rollout and review
- **Issue:** The narrower plan file list did not cover all first-party production direct time-helper consumers.
- **Fix:** widened the rollout and classification pass to every caller found by the required `rg` perimeter scan, then kept the exact scan command in this summary as the reproducible closure artifact.
- **Files modified:** cross-crate rollout including `crates/z00z_core/**`, `crates/z00z_storage/**`, `crates/z00z_wallets/**`, and `crates/z00z_simulator/**`
- **Verification:** final perimeter scan returned no direct `.unix_timestamp*()` or raw epoch-micros matches
- **Committed in:** not committed in this execution

**2. [Rule 1 - Bug] Reused wallet sessions could save or export successfully without refreshing the state-based auto-lock tracker**

- **Found during:** Task 2 downstream wallet validation
- **Issue:** cached-session save and export paths reused valid sessions but refreshed wallet state with stale outer timestamps, leaving auto-lock state drift.
- **Fix:** added focused regressions and switched reused-session success paths to refresh activity after successful verification rather than reusing stale sampled time.
- **Files modified:** `crates/z00z_wallets/src/services/wallet_service.rs`
- **Verification:** `cargo test -p z00z_wallets --release reused_session_refreshes_auto_lock_activity -- --nocapture`; wallet export and backup integration tests
- **Committed in:** not committed in this execution

**3. [Rule 2 - Missing Critical] Seed-view rate limiting and wallet state sync still had fail-open expiry windows**

- **Found during:** Task 2 security review
- **Issue:** `wallet.session.show_seed_phrase` could burn rate-limit budget after a late session invalidation, and wallet state reporting could remain unlocked after expiry.
- **Fix:** made precheck non-mutating, moved actual seed-view budget consumption after a second no-touch session validation, mapped late service failures at the RPC layer, and added best-effort expired-session synchronization for wallet state and list views.
- **Files modified:** `crates/z00z_wallets/src/services/wallet_service.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/wallet_impl.rs`
- **Verification:** `cargo test -p z00z_wallets --release wallet_show_seed_phrase -- --nocapture`; focused state-sync tests
- **Committed in:** not committed in this execution

**4. [Rule 1 - Bug] Session-manager expiry checks trusted caller-supplied time sampled before `await`**

- **Found during:** final manual review after the downstream fixes
- **Issue:** expiry-sensitive methods still accepted caller-provided `now_ms` captured before acquiring the session lock, leaving a TOCTOU expiry window and a fail-open state-sync path on time-provider failure.
- **Fix:** injected `Arc<dyn TimeProvider>` into `WalletSessionManager`, sampled time internally after taking the session lock, made liveness checks fail closed on time failure, and updated wallet activity refresh to happen after successful verification.
- **Files modified:** `crates/z00z_wallets/src/services/session_service.rs`, `crates/z00z_wallets/src/services/wallet_service.rs`
- **Verification:** focused wallet timing reruns plus wallet anchor tests
- **Committed in:** not committed in this execution

---

**Total deviations:** 4 auto-fixed (2 bug, 2 missing critical)
**Impact on plan:** the time-policy rollout stayed inside `PH27-TIME`, but honest closure required widening the downstream proof to the real wallet/session expiry seam exposed by the migration.

## Issues Encountered

- ⚠️ The original planning inventory understated the live production caller set; the required perimeter scan had to become the real source of truth for closure.
- ⚠️ Two focused RPC regressions initially failed because wrong-password mapping and confirmation-ordering behavior had changed during the seed-view hardening pass; both were corrected before closeout.
- ⚠️ A final external read-only review attempt hit agent rate limiting, so the last narrowed-scope pass was completed by direct manual review of the changed wallet/session code and then revalidated with focused tests.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- ✅ `PH27-TIME` is now closed with explicit helper naming, a reproducible perimeter scan, and green wallet anchor validation.
- ✅ Later Phase 027 waves can rely on a stable rule: security-sensitive code must choose `try_*`, and compatibility-only timestamps must remain explicit.
- ✅ The wallet/session seam no longer carries known stale-time or fail-open expiry drift from this rollout.

## Validation Evidence

- ✅ `cargo test -p z00z_utils --release --test test_time_policy_micros` -> passed earlier in the closeout cycle after the contract split
- ✅ `cargo check -p z00z_utils --release --example time_provider_demo` -> passed earlier in the closeout cycle
- ✅ `cargo test -p z00z_wallets --release reused_session_refreshes_auto_lock_activity -- --nocapture` -> passed
- ✅ `cargo test -p z00z_wallets --release wallet_show_seed_phrase -- --nocapture` -> final focused rerun passed, including `wallet_show_seed_phrase_rate_limit_does_not_extend_auto_lock` and `wallet_show_seed_phrase_wrong_password_does_not_extend_auto_lock`
- ✅ `cargo test -p z00z_wallets --release syncs_expired_session_to_locked -- --nocapture` -> passed in the final focused wallet timing sweep
- ✅ `cargo test -p z00z_wallets --release test_unlock_fast_path_still_requires_password -- --nocapture` -> passed in the final focused wallet timing sweep
- ✅ `cargo test -p z00z_wallets --release test_unlock_fast_path_wrong_password_applies_backoff -- --nocapture` -> passed in the final focused wallet timing sweep
- ✅ `cargo test -p z00z_wallets --release test_unlock_fast_path_success_resets_backoff -- --nocapture` -> passed in the final focused wallet timing sweep
- ✅ `cargo test -p z00z_wallets --release --test test_addr_rate_limit_integration -- --nocapture` -> passed (`3 passed` in the final rerun)
- ✅ `cargo test -p z00z_wallets --release --test test_key_manager -- --nocapture` -> passed (`9 passed` in the final rerun)
- ✅ `cargo test -p z00z_wallets --release --test test_stealth_request -- --nocapture` -> passed (`36 passed` in the final rerun)
- ✅ User-required simulator gates stayed green earlier in the same closeout cycle: `cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_dump` and `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump`
- ✅ Final perimeter scan command:

```bash
rg -n '\.unix_timestamp(_millis|_micros)?\(|duration_since\(SystemTime::UNIX_EPOCH\).*as_micros\(' crates --glob 'crates/**/src/**/*.rs' --glob 'crates/**/bin/**/*.rs' --glob '!crates/z00z_crypto/tari/**' --glob '!crates/z00z_utils/src/time/**' --glob '!**/tests/**' --glob '!**/examples/**' --glob '!**/benches/**' --glob '!**/fuzz/**'
```

- ✅ Final perimeter scan result: no matches, exit code `1`

## Self-Check: PASSED

- ✅ Summary artifact created at `.planning/phases/027-crypto-audit-utils/027-03-SUMMARY.md`
- ✅ Validation evidence recorded for the final post-review wallet/session tree and the clean production-perimeter scan
- ✅ No commit hashes were claimed because git checkpointing was not performed in this execution

---

*Phase: 027-crypto-audit-utils*
*Completed: 2026-03-29*
