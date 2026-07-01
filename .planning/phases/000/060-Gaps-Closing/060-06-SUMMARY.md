---
phase: 060-Gaps-Closing
plan: 060-06
status: complete
completed_at: 2026-06-20
next_plan: 060-07
summary_artifact_for: .planning/phases/060-Gaps-Closing/060-06-PLAN.md
---

# 060-06 Summary: Supply-Chain Review Records And Vet Trust Closure

## Completed Scope

`060-06` is complete for the Phase 060 supply-chain review-records and
vet-trust slice.

The repository now carries one canonical supply-chain decision path under the
repository-owned `.reviews/` directory. The L4 supply-chain gate no longer
treats a report-local snapshot as the default advisory-review source or as the
default cargo-vet trust store. Instead, the gate now reads
`.reviews/reviewed-advisories.toml` and `.reviews/` as the canonical
review inputs, while per-run `reports/.../supply-chain/*` files remain
generated evidence only.

This slice closes the project-owned advisory-review gap by inventorying the
four live findings called out by `060-TODO.md` and recording explicit
repository decisions for each one: `bincode 2.0.1 / RUSTSEC-2025-0141`,
`paste 1.0.15 / RUSTSEC-2024-0436`, `derivative 2.2.0 / RUSTSEC-2024-0388`,
and `instant 0.1.13 / RUSTSEC-2024-0384`. Each reviewed exception now carries
owner, reason, scope, sunset, ancestry, code-path criticality, replacement
cost, and temporary-exception conditions. Protected-vendor
`bincode 1.3.3 / RUSTSEC-2025-0141` was kept separate as a vendor exception
with no edits under `crates/z00z_crypto/tari/**`.

This slice also moves cargo-vet ownership onto the repository tree. The
workspace now has a repository-owned `.reviews/config.toml`,
`.reviews/audits.toml`, and `.reviews/imports.lock`, and the L4 gate
now resolves those paths as the canonical defaults instead of treating a
report-local cache as authority. Current live-tree re-audit confirms that the
repo-owned check passes with `Vetting Succeeded (776 exempted)`, but
`.reviews/audits.toml` is still empty and `.reviews/config.toml` still
carries explicit `[[exemptions.*]]` backlog. That closes the authority-path
gap, but it does not yet prove mature repo-owned vet trust.

## Files Changed

- `.planning/phases/060-Gaps-Closing/060-06-SUMMARY.md`
- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.github/skills/z00z-l4-security-engineering-gate/scripts/audit-supply-chain.sh`
- `.reviews/audits.toml`
- `.reviews/config.toml`
- `.reviews/imports.lock`
- `.reviews/reviewed-advisories.toml`

## Boundary Kept

- No second advisory-review ledger was introduced outside the repository-owned
  `.reviews/` home.
- Per-run `reports/.../supply-chain/*` artifacts remain snapshots only and are
  not treated as repository authority.
- No direct edits were made under `crates/z00z_crypto/tari/**`; vendor findings
  stayed on explicit exception or upstream-isolation tracks.
- No fake local cargo-vet audits were invented; the canonical repo store keeps
  its current state explicit, with `imports.lock` still empty and the remaining
  exemptions still called backlog rather than mature trust.
- The thin orchestrator wrapper remains a delegator to the L4 script rather
  than a second supply-chain policy implementation.

## Review Loop

Manual fallback for `.github/prompts/gsd-review-tasks-execution.prompt.md` was
used because the slash prompt is not a callable tool in this environment.

- Pass 1 audited authority paths and found that the L4 gate still defaulted to
  report-local review and vet stores. The script was rewired so the canonical
  defaults point to repository-owned `.reviews/`, while bootstrap init and
  bootstrap vet checks remain an explicit fallback only.
- Pass 2 audited live advisory ancestry and decision coverage. The four
  project-owned advisories plus the protected-vendor `bincode 1.3.3` finding
  were separated into explicit reviewed entries with the required metadata. The
  candidate `zcash` import URL was rejected because it returned `404`, so it
  was not added as a fake trust source.
- Pass 3 reran the strict L4 supply-chain gate, rechecked generated project and
  vendor reports, and confirmed that the canonical repo-owned review ledger is
  live: `project.reviewed = 4`, `project.unreviewed = 0`,
  `vendor.reviewed = 1`, and `vendor.unreviewed = 0`. The same direct repo
  check still reports `Vetting Succeeded (776 exempted)`, so the exemption
  backlog remains open and visible instead of being mislabeled as mature trust.
- Pass 4 reran the broad `cargo test --release` workspace gate, rechecked
  canonical-path strings across the changed slice, and reran scoped
  `git diff --check` on the phase files plus supply-chain files. No significant
  issues remained.

Two consecutive clean review passes were achieved on passes 3 and 4.

## Validation

- Mandatory bootstrap gate passed on the final tree:
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `Z00Z_L4_STRICT=1 ./.github/skills/z00z-l4-security-engineering-gate/scripts/audit-supply-chain.sh`
  now resolves review inputs from repository-owned `.reviews/` and the
  latest rerun wrote
  `reports/z00z-verification-orchestrator-20260623-075715/supply-chain/supply-chain-summary.json`
  with `project.reviewed = 4`, `project.unreviewed = 0`,
  `vendor.reviewed = 1`, and `vendor.unreviewed = 0`.
- `source .github/skills/z00z-verification-orchestrator/scripts/profile-lib.sh && z00z_profile_activate_tool_env "$PWD" && export CARGO=$(command -v cargo) && cargo-vet vet check --store-path .reviews`
  passed with `Vetting Succeeded (776 exempted)`, while
  `.reviews/audits.toml` remains empty and `.reviews/config.toml`
  remains exemption-heavy.
- `rg -n "RUSTSEC-2025-0141|RUSTSEC-2024-0436|RUSTSEC-2024-0388|RUSTSEC-2024-0384|owner|reason|scope|sunset|ancestry|criticality|replacement|temporary_exception_conditions" .reviews/reviewed-advisories.toml reports/z00z-verification-orchestrator-20260623-075715/supply-chain/supply-chain-project.md reports/z00z-verification-orchestrator-20260623-075715/supply-chain/supply-chain-vendor.md`
  confirms the reviewed records and generated reports stay aligned.
- `cargo test --release` passed on the final tree.
- `git diff --check -- .planning/phases/060-Gaps-Closing/060-06-SUMMARY.md .planning/STATE.md .planning/ROADMAP.md .github/skills/z00z-l4-security-engineering-gate/scripts/audit-supply-chain.sh .reviews/audits.toml .reviews/config.toml .reviews/imports.lock .reviews/reviewed-advisories.toml`
  is clean for this slice.

## Result

`060-06` remains complete for repository-owned advisory review ownership and
gate wiring. Later live-tree re-audit keeps one honest residual open at the
phase-closeout level: repo-owned cargo-vet maturity is still exemption-heavy
and is tracked as a current Phase 060 security/validation blocker rather than
being hidden behind a fake green trust claim.
