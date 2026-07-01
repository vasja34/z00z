## Attack Surface: Lifecycle commands can launch shadow config trees while canonical config digests still certify the phase packet

**Status:** verified
**Severity:** medium
**Confidence:** high
**Exploitability:** medium
**Category Domain:** validation
**Category CWE:** CWE-15
**Attack Class:** config-or-deployment-fail-open
**Scope Level:** repo
**Scope Paths:** `.planning/phases/058-HJMT-benchmarks/058-TODO.md`, `crates/z00z_rollup_node/src/config.rs`, `crates/z00z_simulator/src/scenario_1/runtime_observability.rs`
**Boundary Slice:** configuration, feature-flag, and deployment-default
**Protected Asset:** canonical HJMT runtime topology integrity and release-evidence authenticity
**Trust Boundary:** operator-authored lifecycle YAML and manifest fields -> aggregator OS process launch and public release packet
**Attacker Capability Model:** a malicious operator or compromised deployment artifact can rewrite `start_cmd` and `restart_cmd` while keeping the canonical config tree present on disk
**Existing Control State:** partial
**Main Vulnerability:** lifecycle command validation is marker-based rather than path-bound, so `start_cmd` and `restart_cmd` can point to shadow `aggregator-config.yaml`, `planner-config.yaml`, and `storage-config.yaml` files while preflight and release evidence continue hashing and certifying the canonical config files under `hjmt.home`.

### Threat Model Snapshot

- **Attacker Class:** malicious operator
- **Entry Point:** `lifecycle.start_cmd` and `lifecycle.restart_cmd` in the HJMT runtime config pack
- **Sink:** launched aggregator OS processes and the emitted runtime packet that claims canonical config coverage
- **Why This Path Is Realistic:** Phase 058 explicitly treats five independent aggregator OS processes plus config-digest evidence as an acceptance gate, but the live code never binds the command targets to the exact files whose digests are recorded.

### Implementation Nuance

`NodeConfig::config_digests()` hashes the planner, storage, route, and aggregator config files resolved from the canonical `hjmt.home`. Separately, `NodeConfig` accepts lifecycle commands if they merely contain filename markers and the `agg-N` substring. The simulator then reuses those canonical digests in the public runtime packet and only checks that the command strings still mention `--planner-config` and `--storage-config`. This leaves a shadow-config lane: a deployment can preserve the canonical files for hashing and evidence while the actual launch commands target a different config tree with the same filenames.

### Evidence

- `.planning/phases/058-HJMT-benchmarks/058-TODO.md:221` - the integrated-upgrade gate requires five independent aggregator OS processes and explicit config-digest evidence, so launch-path integrity is phase-authoritative rather than optional metadata.
- `crates/z00z_rollup_node/src/config.rs:358` - `config_digests()` hashes only the canonical planner, storage, route, manifest, and aggregator files resolved from `hjmt.home`.
- `crates/z00z_rollup_node/src/config.rs:805` - lifecycle validation delegates to `has_cfg_refs()` instead of checking that the command flags resolve to the loaded canonical paths.
- `crates/z00z_rollup_node/src/config.rs:1343` - `has_cfg_refs()` accepts any command containing `aggregator-config.yaml`, `planner-config.yaml`, `storage-config.yaml`, and `agg-N`, which allows alternate directories with the same filenames.
- `crates/z00z_simulator/src/scenario_1/runtime_observability.rs:849` - the release packet reuses `node_cfg.config_digests()`, so public evidence stays anchored to the canonical home hashes.
- `crates/z00z_simulator/src/scenario_1/runtime_observability.rs:3081` - runtime process-topology validation only requires `--planner-config` and `--storage-config` markers, not exact path equality with the hashed config trio.

### Security Control Review

- **Controls Checked:** config digests, startup preflight, duplicate-path checks, process-topology validation
- **Why Existing Controls Are Insufficient:** the current controls prove internal consistency of the canonical config home, but they never prove that the runtime commands actually launch those same files. The deployment can therefore present one config set for hashing and a second config set for execution.

### Pro-Con Audit

**Pros**
- The candidate is backed by production code in two live crates, not by tests or planning prose alone.
- The weak checks are exact and narrow: simple substring tests on lifecycle commands.
- The phase authority explicitly forbids second truth lanes and makes config-digest evidence release-gating.

**Cons**
- Exploitation requires operator or deployment-artifact control rather than an unauthenticated remote caller.
- The launched shadow config pack still passes whatever checks it contains, so this is an integrity and evidence-authenticity break, not a direct proof-verification bypass.

**Decision:** accepted

### Verification

**Gate:** passed
**Reason:** the candidate has concrete production-code evidence, crosses a real configuration/deployment trust boundary, and the present controls do not bind launch commands to the exact canonical files that Phase 058 treats as authority.

### Defensive Implementation Contract

- Parse `start_cmd` and `restart_cmd` into argument vectors and require exact normalized equality between the `--aggregator-config`, `--planner-config`, and `--storage-config` targets and the already-loaded canonical config paths.
- Extend runtime observability to reject process-topology rows unless the emitted lifecycle commands are path-bound to the same canonical config trio whose digests are recorded in the packet.
- Add regression coverage proving that commands targeting `/shadow/.../aggregator-config.yaml`, `/shadow/.../planner-config.yaml`, and `/shadow/.../storage-config.yaml` are rejected even when filename markers and `agg-N` substrings still appear.

### Residual Risk

Operators who are allowed to rewrite the canonical config files can still change runtime behavior deliberately. This defense removes the untracked shadow-config lane, but it does not eliminate all insider or deployment-pipeline risk around intentionally authorized configuration changes.
