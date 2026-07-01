# 037-08 Summary

## ✅ Scope

This summary records the completion state for `037-08-PLAN.md`, covering the request-candidate ordering and expiry-aware receive slice.

## ✅ Outcome

Plan 08 is closed for the deterministic request-candidate policy slice.

The Phase 037 receive lane now evaluates request-bound candidates in a stable order, prunes expired requests before registration, and keeps the `req_id = None` fallback explicit and last. The scanner path and service registration path now agree on that policy, and the live architecture ledger describes the same behavior.

## ✅ Repository Changes

- `crates/z00z_wallets/src/core/address/stealth_scanner/types_tag_cache.rs` now stores active requests in deterministic order and skips expired requests during registration.
- `crates/z00z_wallets/src/core/address/stealth_scan_support.rs` now builds ordered request candidates and preserves the fallback-last policy at the scan boundary.
- `crates/z00z_wallets/src/services/wallet_service_actions_receive.rs` now prunes expired requests before registering them with the scanner.
- `crates/z00z_wallets/src/core/address/test_stealth_scan_support_suite.rs` now covers the ordered candidate policy and a request-bound scan path.
- `.planning/phases/037-output-reception/037-ARCHITECTURE.md` was rebased to describe the implemented deterministic request-candidate policy.

## ✅ Validation

- Targeted regression tests passed:
  - `ordered_request_candidates_puts_fallback_last`
  - `test_active_requests_are_sorted_and_skip_expired`
  - `scan_owned_matches_request_bound_output`
- Mandatory bootstrap gate passed:
  - `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- Full release test command passed:
  - `cargo test --release --features test-fast --features wallet_debug_dump`

## ✅ Review Loop

The validation loop stayed narrow and aligned to the live scanner and receive seams.

1. The first pass confirmed the ordered request-candidate helper preserved the fallback-last policy.
2. The second pass confirmed the cache-side request set stays deterministic and prunes expired requests before registration.
3. The third pass confirmed a request-bound scan still resolves to the live owned path under the new ordering policy.
4. The broader release test command confirmed the change stayed compatible with the rest of the workspace.

## ✅ Current Boundary

This summary closes only the Plan 08 deterministic request-candidate slice. Future Phase 037 work may still improve hint plumbing and detector observability, but it should treat the ordered request-candidate policy as already implemented.

<!-- End of 037-08 Summary -->