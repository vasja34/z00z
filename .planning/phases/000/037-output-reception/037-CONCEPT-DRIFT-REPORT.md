# Concept Drift Report

## Scope

- Baseline reference: `main`
- Current reference: live workspace on `z00z-dev`
- Scope: `.planning/phases/037-output-reception/` plus the live Phase 037 wallet receive and tx-package admission seams in `crates/z00z_wallets`
- Focus dimensions: security and trust boundaries, public API and behavioral contract, duplication and source-of-truth, architecture ownership
- Report date: `2026-04-23`

## Executive Verdict

- Overall conclusion: No confirmed suspicious concept drift was found in the inspected Phase 037 receive surface relative to `main`.
- Confirmed suspicious findings: `0`
- Critical regressions: `0`
- Cleared healthy evolution items: `3`
- Ambiguous items: `0`
- Notable semantic shift: the tx-package admission boundary is stricter than `main` because current code fail-closes on the live public spend contract instead of structural package checks alone; repository evidence supports this as a justified security tightening rather than undocumented drift.

## Baseline Concept Inventory

| ID | Dimension | Historical concept | Historical evidence |
| --- | --- | --- | --- |
| B01 | architecture | `WalletService::recv_range(...)` is the canonical Phase 037 receive lane; compatibility helpers must not silently replace it | `main:.planning/phases/037-output-reception/037-ARCHITECTURE.md`, current `037-TODO.md` Task 1 |
| B02 | security | Single-asset receive remains compatibility-only and reconstructs ownership without becoming the canonical privacy lane | `main:.planning/phases/037-output-reception/037-ARCHITECTURE.md`, `main:crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_transfer.rs` |
| B03 | source-of-truth | Receive logging is part of the adapter seam and should align with project-wide logging abstractions instead of growing a parallel direct-macro style | `main:crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_transfer.rs`, current Phase 037 audit notes |
| B04 | security | Request-aware candidate selection must stay bounded to the canonical receive lane and should not let unordered or expired request metadata change ownership results nondeterministically | `main:.planning/phases/037-output-reception/037-TODO.md`, `main:.planning/phases/037-output-reception/037-ARCHITECTURE.md` |
| B05 | tx admission | Transaction-package verification existed on `main`, but `verify_transaction_package_impl(...)` only used structural verifier checks | `main:crates/z00z_wallets/src/core/tx/tx_verifier.rs`, `main:crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs` |

## Candidate Classification Table

| ID | Dimension | Candidate summary | Initial class | Doublecheck result | Final class | Severity | Confidence |
| --- | --- | --- | --- | --- | --- | --- | --- |
| C01 | source-of-truth | Receive/send adapter logging moved from direct `tracing` macros to `z00z_utils::logger::Logger` helpers | expected_evolution | cleared | expected_evolution | — | high |
| C02 | security | Request registration in `recv_range(...)` now skips expired requests and matches deterministic ordering notes | expected_evolution | cleared | expected_evolution | — | high |
| C03 | security | Tx-package admission now composes structural verification with the live public spend contract via `verify_full_tx_package(...)` | justified_change | cleared | justified_change | medium | high |
| C04 | architecture | Phase 037 docs still mark single-asset RPC receive as compatibility-only while `recv_range(...)` remains canonical | expected_evolution | cleared | expected_evolution | — | high |

## Findings First

### Confirmed Drift Findings

No `suspicious_drift` or `critical_regression` items were confirmed in the inspected Phase 037 surface.

### Ambiguous Or Blocked Items

No ambiguous items remain after current-code review, object-level `main` comparison, and independent doublecheck.

## Cleared Healthy Evolution

### C01 source-of-truth cleared as expected_evolution

- Initial concern: the compatibility receive adapter may have forked observability behavior away from the project logging boundary.
- Clearing evidence: `main` used direct `warn!` and `info!` macros in `asset_impl_server_transfer.rs`, while the current file routes receive and send events through `log_receive_reject(...)`, `log_receive_info(...)`, and `log_send_info(...)`, all backed by `z00z_utils::logger::Logger` plus `TracingLogger`.
- Why the concept is still intact: the adapter still owns the same receive/send events, but the implementation now converges on the repository-wide logging abstraction instead of widening into a second logging contract.
- Optional note: this is architecture alignment with the Design Foundation one-source-of-truth rule, not a semantic behavior regression.

### C02 security cleared as expected_evolution

- Initial concern: filtering expired requests before scanner registration could have changed ownership semantics in a way that diverges from the Phase 037 baseline.
- Clearing evidence: the current `recv_range(...)` loop explicitly filters `!request.is_expired()`, and the Phase 037 planning and validation artifacts record deterministic ordered non-expired candidate selection as the intended closeout for Task 15, including `037-08-SUMMARY.md`, `037-VALIDATION.md`, `037-TEST-SPEC.md`, and `037-TODO.md`.
- Why the concept is still intact: the canonical receive lane did not move; it was hardened so the same lane becomes deterministic and expiry-aware instead of depending on unordered request registration.
- Optional note: this is bounded receive-lane tightening, not silent concept replacement.

### C03 security cleared as justified_change

- Initial concern: current tx-package verification is materially stricter than `main`, where `verify_transaction_package_impl(...)` called `TxVerifierImpl::verify(...)` and accepted structurally valid packages without composing the live public spend contract.
- Clearing evidence: current `tx_verifier.rs` adds `verify_full_tx_package(...)` and `verify_package_public_spend_contract(...)`; current `tx_impl_server_lifecycle.rs` delegates verification through that full wrapper; current `test_direct_tx_receive.rs` now asserts fail-closed behavior for packages missing spend proof; and `037-FULL-AUDIT.md` records that the old integration-test expectation had drifted from the live verifier contract and required correction.
- Why the concept is still intact: this changes admission semantics at a security boundary, but the direction is stronger and explicitly documented. The repository now treats status labels as insufficient unless the package also satisfies the live public spend contract.
- Optional note: this should stay visible in drift history because it is a real semantic shift from `main`, even though it is justified rather than suspicious.

### C04 architecture cleared as expected_evolution

- Initial concern: current docs might still claim `recv_range(...)` is canonical while the live code silently lets the single-asset RPC receive path become a second receive authority.
- Clearing evidence: current `037-ARCHITECTURE.md` and `037-TODO.md` still mark `scan_asset_report(...)`, `receive_asset(...)`, and outward `wallet.asset.receive_asset` as compatibility-only; current `asset_impl_server_transfer.rs` reconstructs ownership through `scan_asset_report(...)` and `receiver_keys(...)` without routing through the placeholder service `receive_asset(...)`; current `wallet_service_actions_receive.rs` still documents the compatibility-only status of `receive_asset(...)`; and no inspected code path auto-promotes the compatibility lane into the canonical persistence or range-receive lane.
- Why the concept is still intact: the compatibility lane remains explicitly non-parity, and the canonical Phase 037 ownership/persistence authority still lives in `recv_range(...)` plus `recv_route(...)`.
- Optional note: the current tree keeps the boundary honest by documenting the non-parity seam instead of pretending it was unified.

## Doublecheck Ledger

| ID | What was challenged | Alternative explanation tested | Outcome | Notes |
| --- | --- | --- | --- | --- |
| C01 | logger-seam change | refactor-only normalization versus behavioral drift | cleared | Independent review agreed this is logging-boundary convergence, not semantic drift |
| C02 | expiry-aware receive registration | hidden receive fork versus documented deterministic hardening | cleared | Independent review matched the code to the Task 15 planning and validation chain |
| C03 | stricter tx-package verifier | suspicious contract break versus intentional fail-closed tightening | cleared | Independent review recommended keeping it as a visible context note because it is a real semantic shift from `main` |
| C04 | canonical versus compatibility receive boundary | stale docs masking a silent code fork | cleared | Independent review found no evidence that current code violated the documented non-parity boundary |

## Recommendations

- Immediate actions:
  - Treat this report as a cleared comparison for the inspected Phase 037 receive surface rather than an escalation artifact.
  - Keep `verify_full_tx_package(...)` as the explicit admission boundary for tx-package verification and do not let tests regress to status-only expectations.
- Follow-up validation:
  - Re-run the same concept-drift comparison if future work promotes single-asset RPC receive into the canonical lane or introduces a real inbox-assisted receive source.
  - Re-check tx-package verifier semantics whenever spend-proof or import-readiness behavior changes.
- Documentation updates:
  - Preserve the explicit compatibility-only wording in `037-ARCHITECTURE.md` and `037-TODO.md` until real code promotion happens.
  - Keep Phase 037 audits and tests aligned on the fail-closed public spend contract language.
- Optional cleanup:
  - If a future phase unifies the compatibility receive path with the canonical lane, record that as a new justified architectural change instead of silently editing the Phase 037 baseline.
