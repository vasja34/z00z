---
phase: 040
slug: spend-proof
status: verified
threats_open: 0
asvs_level: 1
created: 2026-04-29
---

# Phase 040 - Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Security Scope Boundary

This security audit verifies only the declared Phase 040 threat mitigations for
the completed internal theorem-relation scope. The verified claim is limited to
the wallet, simulator, checkpoint-admission, and rollup public-artifact binding
evidence already recorded for Phase 040.

The following stronger claims remain outside this security sign-off and must not
be treated as completed by this file:

- public or trustless proof-of-knowledge closure;
- checkpoint theorem finality;
- full public or trustless rollup settlement proof closure.

No `## Threat Flags` sections were present in the Phase 040 summary artifacts at
the time of this audit.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
| -------- | ----------- | ------------- |
| tx wire to public spend statement | Untrusted package bytes cross into the signed and verifiable spend contract. | `TxPackage`, `TxWire`, `SpendProofWire`, public statement bytes |
| digest helper to public root contract | Internal digest helpers must not become public proof-binding roots. | package digest, helper digest material |
| Stage 4 producer to persisted tx package | Wallet or Stage 4 output becomes the reusable proof-bearing package consumed by later simulator and checkpoint hooks. | proof/auth carrier, receiver auth, tx package JSON |
| checkpoint apply to proof verifier | State mutation depends on the shared package proof verifier and existing checkpoint hooks. | `TxPkgSum`, checkpoint exec rows, `TxProofVerifier` result |
| spend nullifier to checkpoint/state mutation | Replay checks must stay explicit at the live asset-id spent model and not be overstated as stronger nullifier persistence. | spend nullifiers, spent asset ids, checkpoint roots |
| local wire checks to full admission | Structural package validation is not enough for admission without the public spend contract. | decoded package, verifier report, public spend contract |
| Stage 4 package to Stage 6 reload to Stage 11 apply | Proof-bearing package continuity must survive serialization and simulator handoff. | persisted package, bridge output rows, exec input rows |
| optional output cleanup to proof-facing behavior | Late output cleanup must not change already-validated proof or output semantics. | output roles, leaf fields, range proofs |
| theorem rules to canonical statement and proof path | Core spend relations must remain identical after proof integration. | `verify_spend_rules(...)`, statement facts, backend artifact |
| public input surface to verifier recomputation | Carried public inputs must match verifier-recomputed facts. | inputs, outputs, scope, `prev_root`, `proof_suite` |
| checkpoint pipeline to closeout evidence | Closeout must prove reuse of existing trust boundaries instead of silently widening them. | checkpoint draft/link/artifact evidence |
| phase artifacts to completion claims | Spec, backlog, validation, UAT, and closeout truth must stay aligned. | Phase 040 planning and validation Markdown artifacts |
| canonical statement to theorem proof | The verifier must bind the exact public statement and the deterministic theorem artifact together. | statement hash, public hash, theorem bytes |
| private witness inventory to public carrier | Witness-only relations must stay private and must not be promoted into persisted public fields. | receiver secret, ordered `s_in`, membership witnesses |
| proof bytes to signed auth | Authorization must stay bound to the canonical statement and suite/version, not to a detached envelope. | auth signature, receiver card, statement bytes |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
| --------- | -------- | --------- | ----------- | ---------- | ------ |
| T-040-01 | T | `TxProofWire` carrier | mitigate | `SpendProofWire` is versioned and explicit; `verify_tx_public_spend_contract(...)` rejects missing proof/auth, bad proof/auth versions, bad suite, and empty statement/proof payloads. Covered by `test_spend_proof_wire.rs`. | closed |
| T-040-02 | I | `encode_spend_statement(...)` | mitigate | `encode_spend_statement(...)` frames package digest, chain scope, tx version, `prev_root`, ordered inputs, proof inputs, and output facts into one canonical statement. Covered by `test_spend_statement.rs`. | closed |
| T-040-03 | R | verifier evidence surface | mitigate | Replayable contract evidence exists in `test_spend_proof_wire.rs` and `test_spend_statement.rs`, including carrier roundtrip, deterministic statement rebuild, and drift rejection. | closed |
| T-040-04 | E | helper digest path | mitigate | `build_tx_package_digest(...)` remains authoritative; a bare wire digest carried as `statement_hex` rejects with `StatementMismatch`. Covered by `test_spend_statement.rs`. | closed |
| T-040-05 | T | Stage 4 producer path | mitigate | `build_public_spend_contract(...)` validates receiver-bound inputs and witness shape before signing or persistence; forged receiver input and input-binding mismatch reject in `test_spend_prover_contract.rs`. | closed |
| T-040-06 | S | spend authorization | mitigate | Producer signs `SPEND_AUTH_CTX` plus canonical statement bytes, and verifier checks the same statement with `verify_spend_authorization(...)`; bad auth version, statement drift, and proof suite drift reject. | closed |
| T-040-07 | E | checkpoint verifier path | mitigate | `state_update.rs` adapts checkpoint proof checks through the `TxProofVerifier` trait, and Stage 11 constructs `CheckpointPackageProofVerifier` instead of a parallel verifier object. | closed |
| T-040-08 | D | Stage 6 adapter | mitigate | Stage 6 loads packages through `verify_full_tx_package(...)`; Stage 7/11 handoff verifies package contract, proof bytes, input refs, and bridge outputs before checkpoint draft construction. | closed |
| T-040-09 | R | regular-spend nullifier surface | mitigate | `derive_spend_nullifier(...)` is deterministic and chain-scoped, distinct from claim nullifiers, and duplicate or wrong nullifiers reject in `verify_spend_rules(...)`. Covered by `test_spend_nullifier_semantics.rs`. | closed |
| T-040-10 | T | state replay boundary | mitigate | The live boundary is explicitly limited to the asset-id spent model: `apply_batch_checkpoint(...)` rejects duplicate batch spends, missing inputs, bad resolved leaves, and spent-after checks through `SpentIndex`. Stronger nullifier persistence remains outside the completed claim. | closed |
| T-040-11 | I | full verifier callers | mitigate | `verify_full_tx_package(...)` is the composed package-admission entry point and wraps structure, balance, digest, signature, range proof, and public spend-contract verification. Stage 6 uses that entry point on reload. | closed |
| T-040-12 | D | range-proof backend integration | mitigate | Public verifier and backend both require output range proof validity; missing or tampered range proofs reject in `test_tx_proof_verifier.rs` and `test_spend_proof_backend.rs`. | closed |
| T-040-13 | T | simulator handoff path | mitigate | Scenario roundtrip preserves proof/auth and digest from Stage 4 through Stage 10, and chain-scope tamper fails before Stage 11 checkpoint summary/draft persistence. Covered by `test_scenario1_tx_proof_roundtrip.rs`. | closed |
| T-040-14 | I | stage-surface wording | mitigate | `test_scenario1_stage_surface.rs`, `040-INTEGRITY-GATES.md`, `040-CLOSEOUT-GATES.md`, `040-VALIDATION.md`, and `040-UAT.md` keep wording bounded to internal theorem-relation closure and open public/checkpoint/rollup boundaries. | closed |
| T-040-15 | T | output constructor follow-up | mitigate | `040-CLOSEOUT-GATES.md` keeps the output-constructor work as a bounded no-op/follow-up; existing output-facing validation remains tied to leaf fields, range proofs, and sender/receiver examples. | closed |
| T-040-16 | E | checkpoint draft persistence | mitigate | Stage 11 verifies package contract, proof bytes, input refs, outputs, and exec root before draft construction; scenario tests reject tampered handoff before final checkpoint artifacts or draft output are emitted. | closed |
| T-040-17 | T | theorem preservation | mitigate | `CanonicalSpendProofBackend::prove(...)` validates statement shape, membership, output ranges, nullifier, balance, and `verify_spend_rules(...)` before producing the deterministic artifact. Recorded in `040-INTEGRITY-GATES.md`. | closed |
| T-040-18 | I | public input surface | mitigate | Public verifier recomputes statement facts and rejects fee, root, output, chain, version, input count, input binding, output leaf-ad, range-proof, and overlap drift. Covered by `test_spend_statement.rs` and `test_tx_proof_verifier.rs`. | closed |
| T-040-19 | E | digest-root discipline | mitigate | `build_tx_package_digest(...)` remains the only public/persisted proof-binding root; bare wire-digest-only proof paths reject, and auth-only field drift does not alter the package digest. | closed |
| T-040-20 | R | closeout auditability | mitigate | `040-INTEGRITY-GATES.md`, `040-CLOSEOUT-GATES.md`, `040-VALIDATION.md`, and `040-UAT.md` map integrity gates to code, tests, and explicit open boundaries. | closed |
| T-040-21 | E | checkpoint proof boundary | mitigate | Checkpoint draft construction reuses `TxProofVerifier`, `SpentIndex`, package-coupled verifier adapters, and checkpoint hooks; `040-CLOSEOUT-GATES.md` forbids a separate regular-tx proof layer. | closed |
| T-040-22 | R | missing-code closure truth | mitigate | `040-VALIDATION.md` and `040-CLOSEOUT-GATES.md` keep a closure matrix tied to task IDs, code/test anchors, and the manual open-boundary table. | closed |
| T-040-23 | I | final shortcut checklist | mitigate | `040-CLOSEOUT-GATES.md` and stage-surface guards re-check the prohibited STARK lane, `receiver_cards`, separate `C_fee`, mixed leaf-ad runtime paths, and standalone checkpoint authority shortcuts. | closed |
| T-040-24 | T | phase closeout claims | mitigate | `040-Spend-Proof-Spec.md`, `040-TODO.md`, `040-CLOSEOUT-GATES.md`, `040-VALIDATION.md`, and `040-UAT.md` align on internal theorem-relation closure only. | closed |
| T-040-25 | T | theorem closure | mitigate | Envelope-only verification was replaced by backend verification over the canonical statement: `verify_tx_public_spend_contract(...)` decodes the artifact and calls `CanonicalSpendProofBackend::verify(...)`. | closed |
| T-040-26 | I | privacy boundary | mitigate | Witness-only fields remain in `SpendProofWitness` and backend generation; persisted `SpendProofWire` carries statement/proof/public input data, not receiver secret, ordered `s_in`, or membership witness internals. | closed |
| T-040-27 | E | checkpoint proof path | mitigate | Checkpoint verification reuses the package-coupled `TxProofVerifier` path in `state_update.rs` and simulator Stage 11; no shadow checkpoint proof object or bypass seam is introduced. | closed |
| T-040-28 | R | signing boundary | mitigate | Authorization is subordinate to canonical statement bytes plus suite/version compatibility; proof suite drift, statement drift, bad auth version, and malformed signature fields reject in the wallet verifier tests. | closed |
| T-040-29 | T | closeout wording | mitigate | Active Phase 040 artifacts preserve the real theorem boundary and explicitly keep public proof-of-knowledge, checkpoint theorem finality, and full rollup settlement proof closure outside completed language. | closed |

---

## Accepted Risks Log

No accepted risks.

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By         |
| ---------- | ------------- | ------ | ---- | -------------- |
| 2026-04-29 | 29            | 29     | 0    | GitHub Copilot |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-04-29
