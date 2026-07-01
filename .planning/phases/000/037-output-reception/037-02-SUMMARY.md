# 037-02 Summary

## Scope

This summary records the completion state for `037-02-PLAN.md`, covering task
`Task 2. Materialize request-bound Tag16Context explicitly` and task
`Task 3. Keep proof verification downstream of ownership detection`.

## Outcome

Plan 02 is closed for the detector-boundary clarification slice.

Phase 037 now states explicitly that strict tag-only receive requires concrete
`Tag16Context` registration, that `add_request(...)` remains liveness metadata
only, and that receive detection and classification stay separate from any
downstream proof-verification boundary. The outward `InvalidProof` status and
reject labels remain frozen compatibility vocabulary, but the live contract now
 states clearly that they do not assert verifier execution inside the scanner.

## Repository Changes

- `037-ARCHITECTURE.md` now states that future strict tag-only paths must
  materialize `add_tag_context(...)` entries and that receive detection stays
  separate from downstream import, tx validation, and proof-verification
  boundaries.
- `crates/z00z_wallets/src/core/address/stealth_scanner.rs` now documents
  `add_request(...)` as liveness metadata only, `scan_report(...)` as an
  ownership-detection and receive-classification surface only, and
  `scan_leaf_tag_only(...)` as requiring concrete tag-context registration.
- `crates/z00z_wallets/src/core/address/stealth_scanner/types_receive.rs` now
  documents `ReceiveStatus::InvalidProof` and `ReceiveReject::InvalidProof` as
  frozen compatibility labels for detector-side candidate failures rather than
  proof-verifier execution claims.
- `crates/z00z_wallets/src/services/wallet_service_actions_receive.rs` now
  keeps the canonical `recv_range(...)` commentary aligned to the same
  wallet-local accepted-path boundary used by the Phase 037 source-shape
  regression guards.
- `037-02-SUMMARY.md` now exists as the required plan-closeout artifact.

## Validation

- Diagnostics for `037-ARCHITECTURE.md`, `stealth_scanner.rs`,
  `types_receive.rs`, and `wallet_service_actions_receive.rs`: clean after the
  wording sync.
- Mandatory bootstrap gate after the Rust comment and contract updates:
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed
  clean.
- Focused strict-tag contract regression:
  `cargo test -p z00z_wallets --test test_stealth_scanner_prefilter --release --features test-fast --features wallet_debug_dump`
  passed clean.
- Focused wallet-local receive-boundary regression:
  `cargo test -p z00z_wallets --test test_scenario1_semantics --release --features test-fast --features wallet_debug_dump`
  passed clean after the exact source-shape wording repair.
- Focused receive-report compatibility regression after the final
  `InvalidProof` wording clarification:
  `cargo test -p z00z_wallets --test test_s5_spec6_bridge --release --features test-fast --features wallet_debug_dump test_s5_spec6_bridge_rejects -- --exact`
  passed clean.
- Required broader release suite rerun:
  `cargo test --release --features test-fast --features wallet_debug_dump`
  reached the pre-existing unrelated read-only vendor doctest blocker in
  `crates/z00z_crypto/tari/crypto/` (`-p tari_crypto --doc`), where multiple
  `tari_utilities` versions break doctest imports. No Phase 037 slice failure
  was reported before that blocker.
- Historical note: later Phase 037 validation artifacts and later reruns moved
  past this interim workspace blocker, so this paragraph must not be read as
  the latest closeout truth for the phase.

## Review Loop

The bounded review passes for Plan 02 converged on one real contract gap and
then closed cleanly.

1. Task 2 review converged on the implemented truth that `add_request(...)`
   alone cannot authorize strict tag-only ownership, and later clean passes
   found no remaining ambiguity in the scanner or planning surfaces.
2. Task 3 review initially flagged that the outward `InvalidProof` vocabulary
   could still sound like inline proof verification. The fix kept the frozen
   compatibility names but made their detector-only meaning explicit at the
   status, reject, RPC-code, and `recv_report()` mapping boundary.
3. After that wording repair, the final Task 3 review passes were consecutive
   clean runs with no further significant issues in the detector/report scope.

## Current Boundary

This summary closes only the Plan 02 detector-boundary clarification slice for
Task 2 and Task 3. It does not claim closure of later Phase 037 persistence,
API, observability, or RPC-alignment work, and it does not claim resolution of
the unrelated read-only `tari_crypto` doctest blocker outside the Phase 037
scope.
