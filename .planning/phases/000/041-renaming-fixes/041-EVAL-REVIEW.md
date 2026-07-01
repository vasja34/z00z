# 041-EVAL-REVIEW — Evaluation Coverage Audit

**Phase:** 041-renaming-fixes  
**Audit date:** 2026-01-16  
**Auditor:** gsd-eval-auditor  
**Scope:** 041-CONTEXT.md, 041-TODO.md, 041-01-PLAN.md, 041-02-PLAN.md, 041-03-PLAN.md, plus signature-audit, suffix-audit, threats-audit artifacts  
**Output:** Scored per-plan verdict + gap remediation table

---

## 1. Evaluation Dimensions

| # | Dimension | Weight | Description |
| --- | --- | --- | --- |
| E-1 | Threat coverage | HIGH | All threats in threats-audit must be addressed or explicitly deferred |
| E-2 | TODO task mapping | HIGH | Every TODO.md task (T-01→T-05) must have a corresponding plan task |
| E-3 | Audit finding closure | HIGH | All CRITICAL findings in signature-audit and suffix-audit must be tracked |
| E-4 | Cross-plan ordering | MEDIUM | Wave dependencies must be explicit; no plan can assume another wave is complete without a gate |
| E-5 | Regression guard | MEDIUM | Cargo test bootstrap gate must appear in every wave's verify block |
| E-6 | Shared test strategy | LOW | Waves that touch the same modules must coordinate on test coverage |
| E-7 | No-behavior-change invariant | HIGH | Plans must document that no runtime logic changes are introduced |

---

## 2. Per-Plan Scores

### 2.1  041-01-PLAN.md — Wave 1 (Signature Renames)

**Goal:** Rename identifiers from signature-audit findings across 10 files.

| Dimension | Verdict | Evidence |
| --- | --- | --- |
| E-1 Threat coverage | PARTIAL | Plan acknowledges naming-consistency threats but does not address T-4 (stale-ref confusion) or T-5 (test-stub collision) from threats-audit |
| E-2 TODO task mapping | PARTIAL | Covers T-01 renames; no explicit linkage to T-03 (suffix) or T-05 (final gate) |
| E-3 Audit finding closure | PARTIAL | 3 of 5 signature-audit CRITICAL findings tracked in tranche manifests; `KeyRotationService` propagation (finding S-04) not listed in any task |
| E-4 Cross-plan ordering | MISSING | No gate or prerequisite clause stating Wave 1 must complete before Wave 2 begins |
| E-5 Regression guard | COVERED | Bootstrap `cargo test` gate present in task verify blocks |
| E-6 Shared test strategy | MISSING | No note about which test files Wave 2 touches to prevent double-patch conflicts |
| E-7 No-behavior-change | COVERED | Stated explicitly in plan preamble |

**Wave 1 overall: PARTIAL** — Two critical gaps: missing `KeyRotationService` propagation task and absent cross-wave ordering gate.

---

### 2.2  041-02-PLAN.md — Wave 2 (Wallet Suffix Renames)

**Goal:** Apply suffix renames to 19 wallet-domain files; synchronized rename pairs documented.

| Dimension | Verdict | Evidence |
| --- | --- | --- |
| E-1 Threat coverage | PARTIAL | Addresses T-4 implicitly via synchronized pairs but does not reference threats-audit T-4 or T-5 by ID |
| E-2 TODO task mapping | COVERED | Maps to T-02 (suffix changes) and T-03 (cross-file consistency); 19-file manifest is explicit |
| E-3 Audit finding closure | COVERED | All 4 suffix-audit findings appear in the rename manifest |
| E-4 Cross-plan ordering | MISSING | No prerequisite clause: Wave 2 reads Wave 1 output but this is not gated anywhere |
| E-5 Regression guard | COVERED | `cargo test` bootstrap gate in every task verify block |
| E-6 Shared test strategy | PARTIAL | Synchronized pairs (rows 8+19, 11+20, 26+29) noted but no explicit test coordination note for shared test fixtures |
| E-7 No-behavior-change | COVERED | Explicitly stated |

**Wave 2 overall: PARTIAL** — Missing cross-wave ordering gate; threats T-4/T-5 not referenced by ID.

---

### 2.3  041-03-PLAN.md — Wave 3 (Threat Anchor Tests)

**Goal:** Add explicit negative-path tests anchoring Threats T-1, T-2, T-3 (critical); partial-spend boundary for T-4 (evaluate).

| Dimension | Verdict | Evidence |
| --- | --- | --- |
| E-1 Threat coverage | PARTIAL | T-1 (041-T-02), T-2 (041-T-03), T-3 (041-T-01), T-4-flavor (041-T-04) are covered; Threats T-5 (test-stub collision, moderate) is absent |
| E-2 TODO task mapping | PARTIAL | T-01→T-04 land in this wave; T-05 (final gate/integration check) is explicitly deferred to a "next plan artifact" but no plan artifact exists |
| E-3 Audit finding closure | COVERED | Threat references T-041-03-01 through T-041-03-04 map to all CRITICAL threats |
| E-4 Cross-plan ordering | PARTIAL | Sequential order 041-T-01→04 stated inside wave; wave's dependency on Waves 1+2 not formally gated |
| E-5 Regression guard | COVERED | Mandatory order: bootstrap gate, broader release cargo test, YOLO review loop |
| E-6 Shared test strategy | COVERED | Each task identifies exact test file and test function name; no ambiguity |
| E-7 No-behavior-change | COVERED | Explicitly stated: "no runtime behavior or admission-policy change" |

**Wave 3 overall: PARTIAL** — Threat T-5 not addressed; T-05 deferred with no follow-up plan artifact; cross-wave gate missing.

---

## 3. Cross-Plan Gap Matrix

| Gap ID | Type | Affected Plans | Severity | Description |
| --- | --- | --- | --- | --- |
| G-01 | MISSING | 041-01, 041-02, 041-03 | HIGH | No cross-wave ordering gate. Waves 2 and 3 depend on Wave 1 completing cleanly, but no task or prerequisite clause enforces this. |
| G-02 | MISSING | 041-01 | HIGH | `KeyRotationService` propagation (signature-audit finding S-04) not assigned to any plan task. |
| G-03 | MISSING | All | MEDIUM | Threats T-4 and T-5 (moderate) from threats-audit are not explicitly addressed or formally deferred in any plan. |
| G-04 | MISSING | 041-03 | MEDIUM | T-05 from TODO.md (final gate / integration check) is deferred to "next plan artifact" but no such artifact is created or referenced. |
| G-05 | MISSING | 041-01 | MEDIUM | No audit log task: changes to 10 files do not produce a structured artifact recording rename decisions for future drift detection. |
| G-06 | PARTIAL | 041-02 | LOW | Synchronized pair coordination (rows 8+19, 11+20, 26+29) lacks an explicit test-fixture conflict check to prevent double-patch when both pairs touch the same test module. |

---

## 4. Overall Phase Score

| Dimension | Score | Note |
| --- | --- | --- |
| E-1 Threat coverage | 3 / 5 | T-1/T-2/T-3 covered; T-4 partial; T-5 absent |
| E-2 TODO task mapping | 4 / 5 | T-01→T-04 mapped; T-05 deferred without follow-up |
| E-3 Audit finding closure | 4 / 5 | All suffix-audit findings closed; S-04 from signature-audit open |
| E-4 Cross-plan ordering | 1 / 5 | No inter-wave gates documented in any plan |
| E-5 Regression guard | 5 / 5 | All three waves have bootstrap + broader test gates |
| E-6 Shared test strategy | 3 / 5 | Wave 3 explicit; Waves 1 and 2 partial |
| E-7 No-behavior-change | 5 / 5 | All plans state invariant explicitly |

**Weighted Phase Score: 3.6 / 5 — PARTIAL**

---

## 5. Remediation Recommendations

### R-01 — Add cross-wave ordering gates (G-01) · **HIGH priority**

In each wave's prerequisite block, add:

```
Prerequisites: <previous_wave>-SUMMARY.md must exist and all tasks must be DONE before this wave begins.
```

Alternatively, create a single `041-WAVE-GATE.md` artifact that records per-wave completion status and is checked at the start of each subsequent wave.

---

### R-02 — Add `KeyRotationService` propagation task (G-02) · **HIGH priority**

Append to 041-01-PLAN.md or create `041-04-PLAN.md`:

```
Task: Propagate KeyRotationService rename (signature-audit S-04) to all call sites.
Files: grep for old symbol, update, confirm zero old-symbol occurrences after cargo build.
```

---

### R-03 — Formally address Threats T-4 and T-5 (G-03) · **MEDIUM priority**

Either:
- Add threat anchor test for T-5 (test-stub collision) in Wave 3 or a new wave-4 plan.
- Or document explicit deferral with rationale in `041-CONTEXT.md` under a "Deferred Threats" section.

Current silence is ambiguous: an auditor cannot tell if T-4/T-5 are intentionally deferred or forgotten.

---

### R-04 — Create T-05 follow-up plan artifact (G-04) · **MEDIUM priority**

041-03-PLAN.md defers T-05 to "next plan artifact." Create `041-04-PLAN.md` (or `041-04-GATE.md`) that:
- Runs full integration smoke test across all three waves.
- Confirms zero old-symbol occurrences in the entire workspace.
- Produces `041-FINAL-SUMMARY.md` as the phase close artifact.

---

### R-05 — Add audit log task to Wave 1 (G-05) · **MEDIUM priority**

Add a task to 041-01-PLAN.md:

```
Task: After all renames land, produce rename-log.md listing old → new mapping, file path, and rationale for each change. Archive to .planning/phases/041-renaming-fixes/.
```

This prevents future concept-drift by providing a ground-truth reference for reviewers.

---

### R-06 — Add test-fixture conflict check for synchronized pairs (G-06) · **LOW priority**

In 041-02-PLAN.md, add a pre-task check:

```
Before executing synchronized pairs (rows 8+19, 11+20, 26+29), grep test modules for both symbols to confirm no double-patch conflict will result from applying both renames in the same task.
```

---

## 6. Verdict Summary

| Plan | Verdict | Blocker? |
| --- | --- | --- |
| 041-01-PLAN.md | PARTIAL | Yes — G-01, G-02 are HIGH-severity gaps |
| 041-02-PLAN.md | PARTIAL | No — gaps are MEDIUM or lower |
| 041-03-PLAN.md | PARTIAL | No — T-05 deferral is noted; proceed with caution |
| Phase 041 overall | **PARTIAL** | G-01 and G-02 must be resolved before merge |

**Minimum actions before phase can be closed:**

1. Resolve G-01: Document cross-wave ordering gates.
2. Resolve G-02: Add or explicitly defer `KeyRotationService` task.
3. Resolve G-04: Create T-05 follow-up plan or gate artifact.

Remaining gaps (G-03, G-05, G-06) are recommended but not blocking.
