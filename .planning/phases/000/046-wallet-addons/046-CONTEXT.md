# Phase 046: Wallet Addons - Context

**Gathered:** 2026-05-13
**Status:** Ready for planning
**Source:** PRD Express Path (`.planning/phases/046-wallet-addons/046-wallet-addon-spec.md`)

## Phase Boundary

Phase 046 closes the wallet demonstration gaps by proving the live `wallet.tx.*` lifecycle end to end, proving where claimed assets persist, and aligning simulator evidence with the production wallet boundaries instead of relying on simulator-specific transaction helpers.

## Implementation Decisions

### Claimed Asset Persistence

- WHEN a wallet scan detects an owned leaf and the flow reaches `ReceiveNext::PersistClaim`, THE SYSTEM SHALL persist the asset through `wallet_claimed_assets` and `.wlt` snapshot `WalletPersistenceState.claimed_assets`.
- WHERE wallet persistence is described or restored, THE SYSTEM SHALL treat the encrypted snapshot object inside `.wlt` as the canonical claimed-asset persistence plane and SHALL NOT require a separate `wallet_*.bin.enc` file or dedicated asset table.

### Canonical Transaction Lifecycle

- WHEN a wallet builds a transaction through `wallet.tx.build_transaction`, THE SYSTEM SHALL select inputs only from claimed assets not reserved by pending tx history.
- WHEN Phase 046 proves the live wallet transaction lane, THE SYSTEM SHALL drive `wallet.session.unlock_wallet`, `wallet.key.get_receiver_card`, `wallet.tx.build_transaction`, `wallet.tx.list_pending_transactions`, `wallet.tx.cancel_transaction`, `wallet.tx.verify_transaction_package`, `wallet.tx.broadcast_transaction`, `wallet.tx.reconcile_transaction`, `wallet.tx.get_transaction_details`, `wallet.tx.get_transaction_history`, `wallet.tx.export_transaction`, and `wallet.tx.import_transaction` through the logged wallet RPC transport instead of simulator-only tx-history helpers.
- WHEN a tx package is prepared and later verified against settlement/storage, THE SYSTEM SHALL use the live `asset_wire_to_leaf(...)` conversion, bind `prev_root` to the spend proof, and prove output inclusion against checkpoint execution rows without introducing alternate root math.
- WHEN storage replay or reporting is emitted, THE SYSTEM SHALL distinguish semantic `state_root` from persisted `flat_root`, SHALL source `flat_root` from persisted storage metadata or artifacts, and SHALL NOT collapse them into one field or explanation.
- WHEN a pending tx is cancelled through `wallet.tx.cancel_transaction`, THE SYSTEM SHALL expose `cancelled` history status and allow later builds to use unspent claimed assets.
- WHEN a tx is broadcast and reconciled through `wallet.tx.broadcast_transaction` and `wallet.tx.reconcile_transaction`, THE SYSTEM SHALL validate package/evidence, remove spent claimed inputs, append wallet-owned outputs, and persist the changed claimed asset set.
- IF tx evidence has mismatched tx id, tx hash, chain id, checkpoint root, spent ids, or created ids, THEN THE SYSTEM SHALL reject fail-closed and leave claimed assets unchanged.
- WHEN tx history is requested through `wallet.tx.get_transaction_history`, THE SYSTEM SHALL support pending, cancelled, and confirmed status checks with cursor/filter/sort behavior, imported lifecycle evidence SHALL remain provable through canonical JSONL rows or persisted imported records, and exported lifecycle evidence SHALL remain provable through canonical JSONL exported rows or `record_exported(...)` persistence.

### Receive and Scan Authority

- WHEN a wallet resumes receive scanning after restart, THE SYSTEM SHALL resume from persisted `ScanStatePayload` through the canonical `recv_range(...)` lane and SHALL only persist hits via `ReceiveNext::PersistClaim`.
- WHERE receive scanning is documented or tested, THE SYSTEM SHALL treat `StealthOutputScanner` as the live wallet-side detector, `read_scan_state` as the live wallet resume-cursor persistence path, `ScanStorageImpl` as separate local scan-state persistence, and `ScanEngineImpl` as a deferred seam.
- IF a future receive-scanning integration consumes remote aggregator/JMT data, THEN THE SYSTEM SHALL treat that remote surface as a chunk/proof read adapter only and SHALL keep ownership detection in wallet-side `recv_range(...)` / `StealthOutputScanner`.
- IF docs or simulator output claim a JWT-based or JMT-side wallet scanner in this path, THEN THE SYSTEM SHALL reject that description and use the live wallet-side boundary instead.
- IF docs or simulator output describe auth for the receive or claim path, THEN THE SYSTEM SHALL reference `ClaimSourceProof` plus `verify_claim_authority(...)` / `claim_auth_pk()` and SHALL NOT introduce JWT wording.

### Backup, Restore, and Import Parity

- WHEN a backup is restored with `WalletPlusHistory`, THE SYSTEM SHALL restore both `.wlt` claimed assets and canonical tx-history JSONL with all-or-nothing commit semantics.
- IF backup restore fails at any stage, including wrong password, snapshot decode mismatch, tx-history decode mismatch, JSONL replay failure, or staged write failure, THEN THE SYSTEM SHALL reject without mutating existing wallet state and SHALL discard staged restore outputs for both `.wlt` and tx-history artifacts.
- WHEN a receiver imports a portable tx package, THE SYSTEM SHALL preserve receiver-owned-output detection and continue through the receiver tx history path without inventing a separate receiver-only lane.
- WHEN a receiver imports a portable tx package, THE SYSTEM SHALL surface the imported `tx_id` in receiver pending history, preserve canonical imported/exported evidence, and continue through receiver-side broadcast/reconcile without creating a second receiver-specific lifecycle.

### Payment Requests, TOFU, and Session Hardening

- WHEN a payment request is used as the tx recipient, THE SYSTEM SHALL validate signature, expiry, chain binding, and TOFU status before building tx outputs.
- IF payment request validation fails in Phase 046 coverage, THEN THE SYSTEM SHALL keep the live error boundary aligned with `REQUEST_EXPIRED`, `REQUEST_CHAIN_MISMATCH`, `REQUEST_INVALID_SIGNATURE`, and TOFU confirmation-required outcomes rather than simulator-only aliases.
- IF a receiver card is revoked, stale, relabeled, or has changed view/identity material, THEN THE SYSTEM SHALL reject or require confirmation using existing TOFU/payment request errors.
- WHEN session limits are exceeded or a lifecycle lock event occurs, THE SYSTEM SHALL reject sensitive wallet operations without leaking secrets in logs.
- WHEN session hardening is proved, THE SYSTEM SHALL cover wrong unlock, show-seed rate limiting, lifecycle lock events, stale-session rejection for build, cancel, broadcast, reconcile, import, and export wallet.tx paths, and RPC log redaction with no password, seed phrase, raw session token, or key material leakage.
- WHEN `wallet.key.rotate_master_key` is called, THE SYSTEM SHALL enforce session auth, password confirmation, secondary `ROTATE` confirmation, one-per-hour rate limiting, audit logging, and secret-clean RPC logs.
- WHEN `wallet.key.rotate_master_key` succeeds in Phase 046, THE SYSTEM SHALL report only the live in-memory rederive result and SHALL NOT claim persisted seed rotation or durable master-key rewrite.
- WHEN `wallet.key.rotate_master_key` succeeds in Phase 046, THE SYSTEM SHALL return non-empty fingerprint and timestamp evidence plus `keys_rederived` sourced from the live in-memory rederive path only.

### Wallet UX and Documentation Boundaries

- WHERE `wallet.asset.*` methods remain compatibility or UX operations, THE SYSTEM SHALL not label them as confirmed ledger mutation authority.
- WHERE wallet service comments, live RPC module headers, or simulator labels mention stale `stub`, `placeholder`, `Phase 1`, or `residue` wording, THE SYSTEM SHALL replace them with exact compatibility, report-only, live-state, or canonical-lifecycle wording.

### the agent's Discretion

- The stage 13 simulator helper split may follow existing Scenario 1 module style as long as the live `wallet.tx.*` boundary remains canonical.
- The exact report JSON shape may follow existing scenario-report patterns provided it preserves separate `prev_root`, `state_root`, and `flat_root` fields.
- Helper function names for tests may be adapted to current module exports if the behavioral contracts remain identical.

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 046 source notes

- `.planning/phases/046-wallet-addons/046-wallet-addon-spec.md` — primary spec and locked decisions
- `.planning/phases/046-wallet-addons/046-wallet-misses.md` — gap source notes
- `.planning/phases/046-wallet-addons/046-storage-explain.md` — storage boundary explanation

### Wallet tx RPC and history

- `crates/z00z_wallets/src/adapters/rpc/methods/tx.rs` — live `wallet.tx.*` API surface
- `crates/z00z_wallets/src/adapters/rpc/wallet_dispatcher_wiring.rs` — RPC param shapes
- `crates/z00z_wallets/src/persistence/tx/tx_storage.rs` — canonical history model
- `crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs` — file-backed JSONL history store

### Receive, scan, and backup

- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs` — `recv_range(...)` authority
- `crates/z00z_wallets/src/receiver/scan/stealth_scanner.rs` — wallet-side detector
- `crates/z00z_wallets/src/persistence/scans/scan_storage_impl.rs` — local scan-state persistence
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_backup.rs` — backup/restore boundaries

### Simulator and storage

- `crates/z00z_simulator/src/scenario_1/stage_2_utils/transport.rs` — logged RPC transport builder
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_runtime_flow.rs` — existing tx lane baseline
- `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs` — package proof bridge
- `crates/z00z_simulator/src/scenario_1/storage_view.rs` — storage replay helpers
- `crates/z00z_storage/src/assets/model.rs` — semantic root authority
- `crates/z00z_storage/src/assets/store_internal/tx_plan_engine.rs` — apply authority
- `crates/z00z_storage/src/assets/store_internal/redb_backend.rs` — persisted roots

### Security and session boundaries

- `crates/z00z_wallets/src/adapters/rpc/methods/key.rs` — payment request APIs
- `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/server_admin.rs` — master key rotation RPC boundary
- `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/support.rs` — `finish_rotate(...)` support path
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_rpc.rs` — rotate_master_key compatibility shim
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_derivation_recovery.rs` — in-memory rederive path
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_reachability.rs` — claimed asset mutation helpers
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_assets.rs` — asset UX docs/comments target
- `crates/z00z_wallets/src/chain/scan_engine_impl.rs` — deferred seam wording

## Specific Ideas

- Stage 13 is a new Scenario 1 stage after the current checkpoint stages.
- Use `WalletPlusHistory` for restore validation.
- Keep `state_root` and `flat_root` distinct in reports.
- Keep `ScanEngineImpl` future-only and scanner authority wallet-side.
- Correct stale stub/placeholder/Phase 1 language in wallet action and RPC comments.

## Deferred Ideas

- Persisted seed rotation or durable master-key rewrite is out of scope.
- JWT-based scanner wording is out of scope.
- A separate wallet asset RedB table is out of scope.
- A second root/digest engine is out of scope.
- Full Phase 044 closure is out of scope as a prerequisite; Phase 046 must prove the wallet.tx lifecycle on current canonical surfaces without blocking on a separate Phase 044 finish line.

## Spec Coverage Routing

- Plan 01 covers the Stage 13 config model, `ScenarioCfg` wiring, exact `S13-1` through `S13-15` contract ids, runner registration, and the stage module skeleton.
- Plan 02 covers the logged wallet RPC lifecycle, canonical root-binding order, claimed-asset reconcile mutation, receiver export/import continuation, and imported/exported history evidence.
- Plan 03 covers `.wlt` claimed-asset restore, `WalletPlusHistory`, wrong-password fail-closed behavior, `recv_range(...)` restart authority, and `read_scan_state(...)` / `upsert_scan_state(...)` boundaries.
- Plan 04 covers payment-request negative paths, TOFU card drift, session prechecks and lifecycle locking, rotate-master-key auth/rate-limit/audit behavior, and secret-clean RPC logging.
- Plan 05 covers wording-only cleanup for wallet.asset compatibility surfaces, receive and claim-auth wording, backup and scanner labels, and stage-facing compatibility terminology.
- Plan 06 covers the required wallet, wallet-service, and simulator regression inventory plus the exact focused and release-style validation commands.

---

*Phase: 046-wallet-addons*
*Context gathered: 2026-05-13 via PRD Express Path*
