# Phase 027: Crypto Audit Utils - Research

**Researched:** 2026-03-29
**Domain:** `z00z_utils` audit rollout evidence, downstream consumers, and
source provenance
**Confidence:** HIGH

## Summary

Phase 027 is not blocked by lack of remediation ideas. It is blocked by missing
execution evidence in the context document.

The strongest confirmed blocker remains the `LockedBytes` lifetime-unsound guard
in `crates/z00z_utils/src/os_hardening.rs`, followed by the fail-open YAML and
layered-config boundary in `crates/z00z_utils/src/config/yaml.rs` and
`crates/z00z_utils/src/config/layered.rs`.

The most important rollout finding is asymmetrical.

- `LayeredConfig::new()` currently has verified doc or example usage only.
- Lossy time helpers already have real downstream reach across `z00z_core`,
  `z00z_storage`, `z00z_wallets`, and `z00z_simulator`.
- `DeterministicRngProvider` is already live in genesis and simulator
  reproducibility flows, so stronger build guards must preserve those allowed
  domains.

The correct planning conclusion is therefore: config policy can harden quickly,
but time-policy and deterministic-RNG work need explicit downstream consumer
classification before any breaking default or build-gating change lands.

## User Constraints

- Review the phase context critically and fix execution ambiguity.
- Base all rollout and blocker claims on verified codebase evidence.
- Keep planning artifacts in English.
- Preserve the current architecture and crate boundaries.

## Verified Codebase Findings

### 1. Secret-Memory Blocker

- `crates/z00z_utils/src/os_hardening.rs` exposes `lock_bytes_best_effort()` as
  `Option<LockedBytes>` with an address and length, not a lifetime-bound guard.
- The current `Drop` reconstructs a mutable slice from the stored address and
  length, which confirms the context and fusion concern about safe-call-site
  lifetime soundness.
- Current tests exercise API shape and best-effort behavior, but they do not
  prove the absence of a dangling-drop path.

### 2. YAML and Layered Config Boundary

- `crates/z00z_utils/src/config/yaml.rs` currently reads configuration via
  `std::fs::read_to_string()` instead of the crate's bounded I/O surface.
- `crates/z00z_utils/src/config/layered.rs` currently constructs YAML state with
  `YamlConfig::from_file("config.yaml").ok()`, which silently downgrades all
  read and parse failures to `None`.
- Workspace search found `LayeredConfig::new()` only in
  `crates/z00z_utils/src/config/mod.rs` documentation. No broader production use
  was verified during review.

### 3. Time Helper Rollout Footprint

The lossy `unix_timestamp*()` helpers are not local-only utility surfaces.

| Crate | Verified location | Current use | Research conclusion |
| --- | --- | --- | --- |
| `z00z_core` | `crates/z00z_core/src/assets/registry.rs`, `crates/z00z_core/src/genesis/genesis.rs` | registry and genesis timing | must classify each call site before deprecating lossy wrappers |
| `z00z_storage` | `crates/z00z_storage/src/assets/store_internal/redb_backend.rs` | storage timestamp metadata | decide whether lossy fallback is acceptable for this state marker |
| `z00z_wallets` | `crates/z00z_wallets/src/core/address/stealth_trust.rs`, `crates/z00z_wallets/src/core/key/key_manager.rs`, `crates/z00z_wallets/src/services/wallet_service.rs` | trust expiry, runtime state, timing, RPC, cache | highest rollout-risk surface; requires explicit classification |
| `z00z_simulator` | `crates/z00z_simulator/src/scenario_1/stage_3.rs`, `crates/z00z_simulator/src/scenario_1/stage_3_utils/wallet_flow.rs` | scenario artifacts and timing fields | may keep compatibility behavior if confirmed non-security only |

- `crates/z00z_core/src/assets/nonce.rs` already provides the stronger pattern:
  `try_get_timestamp_micros()` calls `try_unix_timestamp_micros()` directly.
- The correct context implication is that Phase 027 must include a consumer
  classification gate before changing public defaults.

### 4. Deterministic RNG Reachability

- `crates/z00z_utils/src/rng/mock.rs` already uses a compile-time production
  guard pattern via `compile_error!` unless test or explicit test features are
  enabled.
- `crates/z00z_utils/src/rng/traits.rs` defines the deterministic provider trait
  with `type Rng: RngCore + CryptoRng + Send;`, which confirms that the
  semantics confusion around deterministic providers is real and code-backed.
- `crates/z00z_utils/src/rng/deterministic.rs` does not currently carry the same
  hard guard.
- Verified live deterministic-RNG consumers exist in:
  - `crates/z00z_core/src/genesis/asset_std.rs`
  - `crates/z00z_core/src/genesis/genesis.rs`
  - `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_impl.rs`
- The correct planning conclusion is not "ban deterministic RNG everywhere".
  It is "preserve explicit genesis and simulator reproducibility while blocking
  new unapproved production domains".

### 5. Logger and File-I/O Surface

- `crates/z00z_utils/src/logger/mod.rs` sanitizes newline, carriage return, and
  NUL, but not full ANSI or broader control-byte sequences.
- `crates/z00z_utils/src/logger/macros.rs` uses direct `::serde_json::json!()`
  in all structured logger macros, confirming that the JSON boundary drift is a
  live helper-path issue rather than a purely theoretical architecture note.
- `crates/z00z_utils/src/logger/rotating_file_logger.rs` drops the level string
  when writing the final line to disk.
- `crates/z00z_utils/src/io/fs.rs` already contains the strongest private write
  path in `atomic_write_file_private()`, while generic `write_file()` still
  swallows permission-copy failures.

## Planning Implications

### What the context can state as verified fact

- `LockedBytes` lifetime soundness is a real blocker.
- YAML bounded-load bypass and layered-config fail-open behavior are real.
- `LayeredConfig::new()` does not currently show broad production reach.
- Lossy time wrappers do have broad downstream reach.
- Deterministic RNG has verified allowed consumers in genesis and simulator.
- Deterministic RNG also has a verified `CryptoRng`-bound trait contract that
  can over-signal production suitability if planning leaves the semantics
  untouched.
- Direct `serde_json` use is present on the logger macro path, so the boundary
  drift finding is verified rather than speculative.

### What remains an open planning decision

- Whether any wallet or storage time-wrapper use is security-critical enough to
  force same-phase migration to `try_*`.
- Whether `DeterministicRngProvider` should use a compile-time production guard,
  a feature gate, or both.
- Whether `serde_json` exposure is a tolerated narrow exception or an
  architecture issue to remove.

## Recommended Context Corrections

- Make downstream consumer audit mandatory for time, config, and RNG contract
  work.
- Add an execution-order section so release blockers land before secondary
  hardening.
- Add a validation-gate section with Miri, YAML matrix, consumer audit,
  deterministic-RNG allowlist, logger sanitization, and file-durability gates.
- Add a provenance ledger so the context distinguishes verified code facts from
  fusion-derived recommendations.
- Add one dependency-and-rollout Mermaid diagram; more diagrams are not needed.

## Acceptance Use

This research artifact exists to support `027-CONTEXT.md` during planning
review. It does not replace `027-FUSION.md`; it narrows the fused findings down
to verified codebase evidence and rollout constraints.

---

*Phase: 027-crypto-audit-utils*
*Research gathered: 2026-03-29 during context review*
