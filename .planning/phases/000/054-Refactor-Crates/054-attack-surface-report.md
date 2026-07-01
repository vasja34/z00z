# Phase 054 Attack Surface Report

**Rechecked:** 2026-06-09
**Doublecheck Mode:** workspace-first live-code verification
**Result:** fixed after closing the runtime planner digest-routing follow-up

## ✅ Attack Surface: Runtime planner trusted caller-controlled package digest for shard routing

**Status:** fixed
**Severity:** medium
**Confidence:** medium
**Exploitability:** medium
**Category Domain:** validation
**Category CWE:** CWE-345
**Attack Class:** fail-open-validation
**Scope Level:** crate
**Scope Paths:** `crates/z00z_runtime/aggregators`, `crates/z00z_runtime/validators`
**Boundary Slice:** external input and parser slice
**Protected Asset:** shard-routing integrity and batch-admission correctness
**Trust Boundary:** untrusted package metadata -> runtime planner route selection
**Attacker Capability Model:** an external caller or compromised upstream integrator could previously submit package metadata whose `tx_digest_hex` diverged from the actual `TxPackage` or `ClaimTxPackage` payload
**Existing Control State:** complete
**Main Vulnerability:** the old runtime planner path derived `route_key`, ordered `intake_ids`, and `plan_digest` from caller-controlled `tx_digest_hex` metadata without recomputing or enforcing a canonical payload-bound digest at the planner boundary. The live code no longer does that.

### Threat Model Snapshot

- **Attacker Class:** external caller or compromised upstream service
- **Entry Point:** `IngressBoundary::normalize(...)` into the public
  `z00z_aggregators` planner lane
- **Sink:** `BatchPlanner::canonical_entries(...)` route lookup and
  `plan_digest(...)`
- **Why This Path Was Realistic:** Phase 054 moved planner authority into
  `z00z_aggregators`, exported the planner lane as a live runtime surface, and
  the original closeout still left no-op ingress normalization plus
  caller-controlled digest routing there

### ✅ Live Code Evidence

- `crates/z00z_runtime/aggregators/src/ingress.rs` now recomputes tx and claim
  digests with `build_tx_package_digest(...)` and
  `build_claim_tx_digest(...)`, rejects mismatches before planning, and emits
  the canonical planner-ready `WorkItem`.
- `crates/z00z_runtime/aggregators/src/types.rs` now keeps `IntakeId` fields
  private, stores verified digest bytes inside private `CanonicalDigest`, and
  exposes no public `WorkItem` constructor.
- `crates/z00z_runtime/aggregators/src/batch_planner.rs` now routes from
  `item.route_key()` instead of decoding a raw caller string on the planner
  boundary.
- `crates/z00z_runtime/aggregators/src/service.rs` now starts public ingress
  from `WorkPayload`, so planner-ready items come from runtime normalization
  rather than caller-filled digest DTOs.
- `crates/z00z_runtime/aggregators/tests/test_live_guardrails.rs` now guards
  the public lane against reintroducing a public `WorkItem` constructor,
  public digest fields, or raw planner digest decoding.
- `crates/z00z_runtime/aggregators/README.md` now documents one canonical
  payload-bound intake path and explicitly rejects caller-supplied digest
  strings as planner authority.

### 🧭 Doublecheck Correction

The original report is now stale on all material evidence points: `IntakeId`
no longer copies caller strings through public constructors, ingress
normalization is no longer a no-op, `BatchPlanner` no longer decodes raw
`digest_hex()` into its route key, and the public lane now has a release-tested
source guard proving the canonical path remains ingress-owned.

### ✅ Defensive Implementation Contract

- Runtime ingress recomputes the canonical digest from `TxPackage` and
  `ClaimTxPackage` payload bytes before planning.
- Mismatched `tx_digest_hex` metadata is rejected fail closed with
  `RejectClass::ShapeInvalid`.
- `BatchPlanner`, `OrderingBoundary`, `SchedulerBoundary`, and the service lane
  route or schedule only verified `WorkItem` values that come from ingress.
- Regression coverage proves forged digest metadata is rejected for both tx and
  claim paths and that the live public lane does not expose a direct
  planner-ready `WorkItem` constructor.

### ✅ Validation

```bash
./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh
cargo test -p z00z_aggregators --release -q
cargo test --release
```

Result: passed.

### ⚠️ Residual Risk

No live safe-API bypass remains in `z00z_aggregators`. An external caller would
need to step outside the safe public Rust API contract to forge planner-ready
runtime digest metadata, which is out of scope for this report.
