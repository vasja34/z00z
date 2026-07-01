# Phase 059 Attack Surface Report

**Scope:** `.planning/phases/059-Core-Upgrade/059-TODO.md`
**Resolved Scope Paths:** `.planning/phases/059-Core-Upgrade/059-TODO.md`, `.planning/phases/059-Core-Upgrade/059-CONTEXT.md`, `.planning/phases/059-Core-Upgrade/059-SECURITY.md`, `.planning/phases/059-Core-Upgrade/059-VALIDATION.md`, `crates/z00z_core/src`, `crates/z00z_storage/src`, `crates/z00z_wallets/src`, `crates/z00z_simulator/src`, `crates/z00z_runtime`, `crates/z00z_rollup_node/src`
**Run Date:** 2026-06-18
**Variants Executed:** 20
**Admission Result:** no admitted candidate

## Boundary Slice Map

- **external input and parser slice:** present via genesis config, simulator YAML, runtime trace packet paths, and report artifact loading
- **authn, authz, and capability boundary slice:** present via rights, policy descriptors, validator verdicts, and wallet object RPC
- **secret handling, storage, and logging slice:** present via wallet-owned payloads, backup/import, and simulator/report emission
- **cryptographic and proof-verification slice:** present via descriptor hashing, HJMT proofs, object packages, replay tags, and publication binding
- **replay, nonce, uniqueness, and state-consumption slice:** present via voucher lifecycle, right consumption, replay nonces, and double-redeem rejects
- **configuration, feature-flag, and deployment-default slice:** present via genesis sections, settlement env wiring, simulator profiles, and route/runtime config
- **dependency, build, CI, and supply-chain slice:** present only indirectly through closeout gates and not as a strong Phase 059 implementation finding source
- **operator, admin, and debug-only surface slice:** present via simulator artifact emission, report generation, and runtime observability packets

## Scan Result

No candidate passed the pro-con audit and verification gate.

### Rejection Summary

- **Simulator trace path escape:** rejected. `runtime_observability` blocks absolute trace paths and parent segments before read/write on the packet contract path, so the strongest file-path hypothesis failed closed at the real sink. Evidence: [runtime_observability.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/runtime_observability.rs:3250), [runtime_observability.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/runtime_observability.rs:3277).
- **Raw `serde_json::Value` in simulator observability:** rejected. The surface is design-foundation drift, but the concrete security path remained fail-closed and public-only; the code decodes through `JsonCodec`, compares canonical payloads, and returns `Scenario1Err::Evidence` on mismatch instead of admitting corrupted state. Evidence: [runtime_observability.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/runtime_observability.rs:1266), [runtime_observability.rs](/home/vadim/Projects/z00z/crates/z00z_simulator/src/scenario_1/runtime_observability.rs:4043).
- **`SettlementStore::new()` panic-driven availability break:** rejected. The panic seam is real, but within Phase 059 evidence it remained mostly test/simulator/helper-facing, while the live env parsing path itself is already fail-closed through `EnvConfig` and typed `SettlementStoreError`. I did not find a strong attacker-controlled production startup path that crossed a meaningful boundary beyond operator misconfiguration. Evidence: [store.rs](/home/vadim/Projects/z00z/crates/z00z_storage/src/settlement/store.rs:501), [hjmt_config.rs](/home/vadim/Projects/z00z/crates/z00z_storage/src/settlement/hjmt_config.rs:32).
- **`WalletReveal::expect` panic boundary:** rejected. The helper is a bad panic-shaped API, but this scan did not confirm a concrete attacker-controlled production call chain in the Phase 059 object flows. Without a live boundary-crossing caller, it stayed a near-miss rather than an admitted attack surface. Evidence: [types_receive.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/src/receiver/scan/types_receive.rs:185).

### Strongest Near-Miss

The closest candidate was the simulator-side operator/debug artifact surface around runtime observability and Stage 13 report emission. It stayed below admission threshold because:

- trace packet paths are validated before file I/O;
- corrupted trace payloads fail closed with explicit evidence errors;
- the remaining raw JSON/file-writing issues sit in simulator and reporting lanes rather than validator or wallet acceptance paths.

### DB Status

- No verified finding was admitted.
- No JSONL row was appended.
- The phase-local DB file exists and remains empty for future reruns.

## Rerun Audit Trail

- **2026-06-18 20:00:21 +0300** - reran the same `scope=059-TODO.md` attack-surface workflow with `max_variants=20`; verdict stayed `no admitted candidate`.
- **Rerun confirmation points:** trace-path guards still reject absolute and parent-segment paths before runtime packet I/O; canonical trace payload mismatches still fail closed with `Scenario1Err::Evidence`; `SettlementStore::new()` still contains a panic seam but without a stronger attacker-controlled production path than operator misconfiguration; `WalletReveal::expect` still lacks a confirmed boundary-crossing caller in Phase 059 object flows.
- **DB outcome:** JSONL remained unchanged and empty because no candidate passed the admission gate.
