# Audit Metadata

📌 Fusion target: `FUSION.md`

📌 Audit target: `FUSION.audit.md`

📌 Normalization rule: repeated findings from different reports were fused into
canonical topic statements, and single-source claims were preserved with
explicit notes instead of being silently dropped.

📌 Coverage basis:

- Source section coverage is measured against every H1-H4 heading from the five
  source reports.
- Provision coverage is measured against normalized substantive propositions used
  to build the fused document.

## Source Files

| Source File | Auditor Label | Lines | Notes |
| --- | --- | ---: | --- |
| `core-audit-glm5.md` | GLM-5 | 443 | Broad source review with detailed positives and test plan |
| `core-audit-gpt54.md` | GPT-5.4 | 290 | Integrity and semantic-binding focused |
| `core-audit-m27.md` | MiniMax M2 | 485 | Conservative severity, good encoding notes |
| `core-audit-mimov2.md` | MiMo-V2-Pro | 301 | Fee-binding and stealth-gap focused |
| `core-audit-sonet46.md` | Sonnet 4.6 | 725 | Widest finding inventory and remediation roadmap |

## Source Section Inventory

📌 Inventory size: `177` source H1-H4 sections.

📌 Container headings are intentionally preserved as meta-sections even when the
underlying substantive meaning is represented by nested findings elsewhere.

| Section ID | Source File | Heading Level | Heading Path | Provision Count | Destination Section IDs | Status |
| --- | --- | --- | --- | --- | --- | --- |
| GLM-001 | core-audit-glm5.md | H1 | Z00Z Core Cryptographic Audit Report | 1 | F-12 | FUSED |
| GLM-002 | core-audit-glm5.md | H2 | 1. Executive Verdict | 1 | F-01,F-11 | FUSED |
| GLM-003 | core-audit-glm5.md | H2 | 2. Scope and Security Goals | 1 | F-02 | FUSED |
| GLM-004 | core-audit-glm5.md | H3 | 2.1 Security Goals Assumed | 1 | F-03 | FUSED |
| GLM-005 | core-audit-glm5.md | H3 | 2.2 Threat Model Summary | 1 | F-03 | FUSED |
| GLM-006 | core-audit-glm5.md | H3 | 2.3 Trust Boundaries | 1 | F-03 | FUSED |
| GLM-007 | core-audit-glm5.md | H2 | 3. Findings | 1 | F-12 | FUSED |
| GLM-008 | core-audit-glm5.md | H3 | 3.1 S1 — HIGH Severity | 1 | F-05 | FUSED |
| GLM-009 | core-audit-glm5.md | H4 | S1-1: `AssetWire::to_asset()` accepts `secret` field from network | 1 | F-05 | FUSED |
| GLM-010 | core-audit-glm5.md | H4 | S1-2: `derive_nonce_simple` and `derive_nonce_minimal` silently fall back to timestamp=0 | 1 | F-06 | FUSED |
| GLM-011 | core-audit-glm5.md | H3 | 3.2 S2 — MEDIUM Severity | 1 | F-06 | FUSED |
| GLM-012 | core-audit-glm5.md | H4 | S2-1: `Asset::asset_id()` does not include `amount` in the hash | 1 | F-07 | FUSED |
| GLM-013 | core-audit-glm5.md | H4 | S2-2: `Asset::to_owner_message()` encodes `lock_height: None` as `0u64` | 1 | F-06 | FUSED |
| GLM-014 | core-audit-glm5.md | H4 | S2-3: `generate_blinding()` has a 64-iteration rejection loop | 1 | F-06 | FUSED |
| GLM-015 | core-audit-glm5.md | H4 | S2-4: `verify_genesis_consensus()` has placeholder `None` hashes | 1 | F-05 | FUSED |
| GLM-016 | core-audit-glm5.md | H4 | S2-5: `AssetMetadata::compute_hash()` does not include `metadata_hash` field itself | 1 | F-07 | FUSED |
| GLM-017 | core-audit-glm5.md | H3 | 3.3 S3 — LOW Severity | 1 | F-07 | FUSED |
| GLM-018 | core-audit-glm5.md | H4 | S3-1: `AssetDebug` implementation shows `nonce` in hex | 1 | F-07 | FUSED |
| GLM-019 | core-audit-glm5.md | H4 | S3-2: `Asset::new()` uses blinding factor as both commitment key and owner key | 1 | F-07 | FUSED |
| GLM-020 | core-audit-glm5.md | H4 | S3-3: `NonceCounter` uses `eprintln!` for safety warning | 1 | F-07 | FUSED |
| GLM-021 | core-audit-glm5.md | H4 | S3-4: `AssetPackPlain::from_bytes()` uses `Option` return instead of `Result` | 1 | F-06 | FUSED |
| GLM-022 | core-audit-glm5.md | H3 | 3.4 S4 — INFO | 1 | F-07 | FUSED |
| GLM-023 | core-audit-glm5.md | H4 | S4-1: `MAX_AMOUNT` is `u64::MAX` | 1 | F-06 | FUSED |
| GLM-024 | core-audit-glm5.md | H4 | S4-2: `AssetDefinition::from_decimal()` uses `f64` arithmetic | 1 | F-07 | FUSED |
| GLM-025 | core-audit-glm5.md | H4 | S4-3: Genesis blinding derivation uses `from_uniform_bytes` with 64-byte hash | 1 | F-07 | FUSED |
| GLM-026 | core-audit-glm5.md | H2 | 4. Open Ambiguities | 1 | F-08 | FUSED |
| GLM-027 | core-audit-glm5.md | H2 | 5. Positive Observations | 1 | F-04 | FUSED |
| GLM-028 | core-audit-glm5.md | H3 | 5.1 Domain Separation (EXCELLENT) | 1 | F-04 | FUSED |
| GLM-029 | core-audit-glm5.md | H3 | 5.2 Pedersen Commitment + Bulletproofs+ (CORRECT) | 1 | F-04 | FUSED |
| GLM-030 | core-audit-glm5.md | H3 | 5.3 Owner Signature (WELL-DESIGNED) | 1 | F-04,F-08 | FUSED |
| GLM-031 | core-audit-glm5.md | H3 | 5.4 Genesis Security (STRONG) | 1 | F-04,F-05 | FUSED |
| GLM-032 | core-audit-glm5.md | H3 | 5.5 Stealth Address Validation (CORRECT) | 1 | F-04,F-08 | FUSED |
| GLM-033 | core-audit-glm5.md | H3 | 5.6 Error Handling (GOOD) | 1 | F-04 | FUSED |
| GLM-034 | core-audit-glm5.md | H3 | 5.7 Secret Lifecycle (GOOD) | 1 | F-04,F-07 | FUSED |
| GLM-035 | core-audit-glm5.md | H3 | 5.8 Gas Calculation (CORRECT) | 1 | F-04,F-08 | FUSED |
| GLM-036 | core-audit-glm5.md | H2 | 6. Concrete Fixes Summary | 1 | F-09 | FUSED |
| GLM-037 | core-audit-glm5.md | H2 | 7. Test Plan | 1 | F-10 | FUSED |
| GLM-038 | core-audit-glm5.md | H3 | 7.1 Required Tests (Missing) | 1 | F-10 | FUSED |
| GLM-039 | core-audit-glm5.md | H3 | 7.2 Existing Test Coverage (Adequate) | 1 | F-10 | FUSED |
| GLM-040 | core-audit-glm5.md | H3 | 7.3 Recommended Additions | 1 | F-10 | FUSED |
| GLM-041 | core-audit-glm5.md | H2 | 8. Confidence Levels | 1 | F-11 | FUSED |
| GLM-042 | core-audit-glm5.md | H2 | 9. Final Decision | 1 | F-01,F-11 | FUSED |
| GLM-043 | core-audit-glm5.md | H3 | Conditions Before Production: | 1 | F-09 | FUSED |
| GLM-044 | core-audit-glm5.md | H3 | Recommended Next Steps: | 1 | F-09 | FUSED |
| GLM-045 | core-audit-glm5.md | H2 | Appendix A: Files Reviewed | 1 | F-02 | FUSED |
| GLM-046 | core-audit-glm5.md | H2 | Appendix B: Dependency Chain | 1 | F-04 | FUSED |
| GLM-047 | core-audit-glm5.md | H2 | Appendix C: Standards Compliance | 1 | F-04 | FUSED |
| GPT-001 | core-audit-gpt54.md | H1 | Crypto Audit Report — z00z_core | 1 | F-12 | FUSED |
| GPT-002 | core-audit-gpt54.md | H2 | Scope | 1 | F-02 | FUSED |
| GPT-003 | core-audit-gpt54.md | H2 | Executive Verdict | 1 | F-01,F-11 | FUSED |
| GPT-004 | core-audit-gpt54.md | H2 | Input Classification | 1 | F-02 | FUSED |
| GPT-005 | core-audit-gpt54.md | H2 | Security Goals Extracted | 1 | F-03 | FUSED |
| GPT-006 | core-audit-gpt54.md | H2 | Threat Model Summary | 1 | F-03 | FUSED |
| GPT-007 | core-audit-gpt54.md | H2 | Critical And High Findings | 1 | F-05 | FUSED |
| GPT-008 | core-audit-gpt54.md | H3 | S1-1 Registry Snapshot Integrity Hash Covers Only Definition IDs | 1 | F-05 | FUSED |
| GPT-009 | core-audit-gpt54.md | H3 | S1-2 Owner Signature Is Self-Asserted And Not Bound To Commitment Ownership | 1 | F-05 | FUSED |
| GPT-010 | core-audit-gpt54.md | H3 | S1-3 Genesis Consensus Hash Verification Is Effectively Disabled | 1 | F-05 | FUSED |
| GPT-011 | core-audit-gpt54.md | H2 | Medium Findings | 1 | F-06 | FUSED |
| GPT-012 | core-audit-gpt54.md | H3 | S2-1 Public Asset DTO Drops Frozen And Slashed State On Rehydration | 1 | F-05 | FUSED |
| GPT-013 | core-audit-gpt54.md | H3 | S2-2 Genesis Seed Entropy Threshold Is Impossible For A 32-Byte Sample | 1 | F-06 | FUSED |
| GPT-014 | core-audit-gpt54.md | H3 | S2-3 Production Registry Config Uses A Test-Only Asset-ID Domain | 1 | F-05,F-06 | FUSED |
| GPT-015 | core-audit-gpt54.md | H2 | Low Findings | 1 | F-07 | FUSED |
| GPT-016 | core-audit-gpt54.md | H3 | S3-1 Silent Time Fallbacks Hide Nonce-Derivation Failures | 1 | F-06 | FUSED |
| GPT-017 | core-audit-gpt54.md | H2 | Additional Observations | 1 | F-07 | FUSED |
| GPT-018 | core-audit-gpt54.md | H3 | S4-1 Internal `AssetWire` Still Carries Secret Material | 1 | F-05 | FUSED |
| GPT-019 | core-audit-gpt54.md | H2 | Open Ambiguities | 1 | F-08 | FUSED |
| GPT-020 | core-audit-gpt54.md | H2 | Concrete Remediation Plan | 1 | F-09 | FUSED |
| GPT-021 | core-audit-gpt54.md | H2 | Test Plan | 1 | F-10 | FUSED |
| GPT-022 | core-audit-gpt54.md | H2 | Confidence By Claim | 1 | F-11 | FUSED |
| GPT-023 | core-audit-gpt54.md | H2 | Final Decision | 1 | F-01,F-11 | FUSED |
| M27-001 | core-audit-m27.md | H1 | Z00Z Core Cryptographic Security Audit Report | 1 | F-12 | FUSED |
| M27-002 | core-audit-m27.md | H2 | 1. Executive Verdict | 1 | F-01,F-11 | FUSED |
| M27-003 | core-audit-m27.md | H2 | 2. Scope & Files Reviewed | 1 | F-02 | FUSED |
| M27-004 | core-audit-m27.md | H2 | 3. Security Goals Assumed | 1 | F-03 | FUSED |
| M27-005 | core-audit-m27.md | H2 | 4. Threat Model Summary | 1 | F-03 | FUSED |
| M27-006 | core-audit-m27.md | H2 | 5. S0/S1 Findings | 1 | F-08 | FUSED |
| M27-007 | core-audit-m27.md | H2 | 6. S2/S3/S4 Findings | 1 | F-06,F-07 | FUSED |
| M27-008 | core-audit-m27.md | H3 | S2 — Commitment Verification Not Constant-Time | 1 | F-06 | FUSED |
| M27-009 | core-audit-m27.md | H3 | S2 — Unvalidated Asset ID in Definition Constructor | 1 | F-05 | FUSED |
| M27-010 | core-audit-m27.md | H3 | S2 — Signature Scalar Parsing Without Canonicality Check | 1 | F-06 | FUSED |
| M27-011 | core-audit-m27.md | H3 | S3 — AssetPackPlain::decode_strict Doesn't Validate Blinding Canonicality | 1 | F-06 | FUSED |
| M27-012 | core-audit-m27.md | H3 | S3 — LE Serial ID Encoding Convention Not Prominently Documented | 1 | F-07 | FUSED |
| M27-013 | core-audit-m27.md | H3 | S3 — Lock Ordering Assertions Only in Debug Builds | 1 | F-07 | FUSED |
| M27-014 | core-audit-m27.md | H3 | S4 — Range Proof RNG Entropy Not Documented | 1 | F-07 | FUSED |
| M27-015 | core-audit-m27.md | H3 | S4 — Deterministic RNG for Genesis Not Audited for Cryptographic Suitability | 1 | F-07 | FUSED |
| M27-016 | core-audit-m27.md | H3 | S4 — Gas Calculation Implicitly Assumes Fee Asset is Coin | 1 | F-06,F-08 | FUSED |
| M27-017 | core-audit-m27.md | H2 | 7. Open Ambiguities | 1 | F-08 | FUSED |
| M27-018 | core-audit-m27.md | H2 | 8. Concrete Fixes | 1 | F-09 | FUSED |
| M27-019 | core-audit-m27.md | H3 | Fix 1: Constant-Time Commitment Verification | 1 | F-09 | FUSED |
| M27-020 | core-audit-m27.md | H3 | Fix 2: Validate Asset ID Derivation | 1 | F-09 | FUSED |
| M27-021 | core-audit-m27.md | H3 | Fix 3: Canonical Scalar Validation for Signatures | 1 | F-09 | FUSED |
| M27-022 | core-audit-m27.md | H3 | Fix 4: Make decode_checked the Default | 1 | F-09 | FUSED |
| M27-023 | core-audit-m27.md | H3 | Fix 5: Document Serial ID Byte Order | 1 | F-09 | FUSED |
| M27-024 | core-audit-m27.md | H2 | 9. Test Plan | 1 | F-10 | FUSED |
| M27-025 | core-audit-m27.md | H3 | Positive Tests (Should Pass) | 1 | F-10 | FUSED |
| M27-026 | core-audit-m27.md | H3 | Negative Tests (Should Fail) | 1 | F-10 | FUSED |
| M27-027 | core-audit-m27.md | H3 | Misuse Tests (Should Handle Gracefully) | 1 | F-10 | FUSED |
| M27-028 | core-audit-m27.md | H3 | Wycheproof-equivalent Tests | 1 | F-10 | FUSED |
| M27-029 | core-audit-m27.md | H2 | 10. Confidence Levels | 1 | F-11 | FUSED |
| M27-030 | core-audit-m27.md | H2 | 11. Final Decision | 1 | F-01,F-11 | FUSED |
| M27-031 | core-audit-m27.md | H2 | 12. Summary Table | 1 | F-01,F-11 | FUSED |
| MIM-001 | core-audit-mimov2.md | H1 | Crypto Audit Report: `z00z_core` Implementation | 1 | F-12 | FUSED |
| MIM-002 | core-audit-mimov2.md | H2 | Executive Verdict | 1 | F-01,F-11 | FUSED |
| MIM-003 | core-audit-mimov2.md | H2 | Input Type and Scope | 1 | F-02 | FUSED |
| MIM-004 | core-audit-mimov2.md | H2 | Security Goals Assumed | 1 | F-03 | FUSED |
| MIM-005 | core-audit-mimov2.md | H2 | Threat Model Summary | 1 | F-03 | FUSED |
| MIM-006 | core-audit-mimov2.md | H2 | Critical and High Findings (S0/S1) | 1 | F-05 | FUSED |
| MIM-007 | core-audit-mimov2.md | H3 | S1-01: Fee Amount Not Included in Proof Statement | 1 | F-05,F-08 | FUSED |
| MIM-008 | core-audit-mimov2.md | H3 | S1-02: Nonce Zero Check Only in Production Builds | 1 | F-06 | FUSED |
| MIM-009 | core-audit-mimov2.md | H3 | S1-03: Stealth Address — Missing View Tag Validation | 1 | F-05 | FUSED |
| MIM-010 | core-audit-mimov2.md | H3 | S1-04: `derive_asset_secret` Uses Poseidon2 Without Domain Tag Verification | 1 | F-06 | FUSED |
| MIM-011 | core-audit-mimov2.md | H2 | Medium and Low Findings (S2/S3/S4) | 1 | F-06,F-07 | FUSED |
| MIM-012 | core-audit-mimov2.md | H3 | S2-01: `NonceCounter::increment_unsafe` Debug Warning Only | 1 | F-06 | FUSED |
| MIM-013 | core-audit-mimov2.md | H3 | S2-02: `AssetMetadata::compute_hash` Uses `DomainHasher` Without Length Prefixing | 1 | F-07 | FUSED |
| MIM-014 | core-audit-mimov2.md | H3 | S2-03: `MAX_AMOUNT` Set to `u64::MAX` | 1 | F-06 | FUSED |
| MIM-015 | core-audit-mimov2.md | H3 | S2-04: `Asset::new_confidential` Returns `Hidden<Z00ZScalar>` Without Usage Guidance | 1 | F-06 | FUSED |
| MIM-016 | core-audit-mimov2.md | H3 | S3-01: `state/mod.rs` Is Empty | 1 | F-07 | FUSED |
| MIM-017 | core-audit-mimov2.md | H3 | S3-02: `AssetError` Has Both String-Based and Structured Variants | 1 | F-07 | FUSED |
| MIM-018 | core-audit-mimov2.md | H2 | Open Ambiguities | 1 | F-08 | FUSED |
| MIM-019 | core-audit-mimov2.md | H2 | Concrete Fixes | 1 | F-09 | FUSED |
| MIM-020 | core-audit-mimov2.md | H3 | Fix S1-01: Fee Binding | 1 | F-09 | FUSED |
| MIM-021 | core-audit-mimov2.md | H3 | Fix S1-02: Unconditional Nonce Check | 1 | F-09 | FUSED |
| MIM-022 | core-audit-mimov2.md | H3 | Fix S1-03: View Tag Verification | 1 | F-09 | FUSED |
| MIM-023 | core-audit-mimov2.md | H3 | Fix S1-04: Domain-Separated Poseidon2 | 1 | F-09 | FUSED |
| MIM-024 | core-audit-mimov2.md | H2 | Implementation Guidance | 1 | F-09 | FUSED |
| MIM-025 | core-audit-mimov2.md | H3 | Safe Architecture | 1 | F-09 | FUSED |
| MIM-026 | core-audit-mimov2.md | H3 | Test Plan | 1 | F-10 | FUSED |
| MIM-027 | core-audit-mimov2.md | H2 | Confidence Level | 1 | F-11 | FUSED |
| MIM-028 | core-audit-mimov2.md | H2 | Final Decision | 1 | F-01,F-11 | FUSED |
| SON-001 | core-audit-sonet46.md | H1 | 🔐 Crypto Audit — `z00z_core` | 1 | F-12 | FUSED |
| SON-002 | core-audit-sonet46.md | H2 | 🎯 Executive Verdict | 1 | F-01,F-11 | FUSED |
| SON-003 | core-audit-sonet46.md | H2 | 📌 Scope & Methodology | 1 | F-02 | FUSED |
| SON-004 | core-audit-sonet46.md | H3 | 📋 Files Audited | 1 | F-02 | FUSED |
| SON-005 | core-audit-sonet46.md | H2 | 🚨 S1 — Critical Blockers | 1 | F-05 | FUSED |
| SON-006 | core-audit-sonet46.md | H3 | `[S1-001]` Genesis consensus verification disabled | 1 | F-05 | FUSED |
| SON-007 | core-audit-sonet46.md | H3 | `[S1-002]` Stealth fields excluded from owner signature coverage | 1 | F-05 | FUSED |
| SON-008 | core-audit-sonet46.md | H2 | ⚠️ S2 — Serious Weaknesses | 1 | F-06 | FUSED |
| SON-009 | core-audit-sonet46.md | H3 | `[S2-001]` Plaintext amount in `AssetWire` breaks confidentiality | 1 | F-05 | FUSED |
| SON-010 | core-audit-sonet46.md | H3 | `[S2-002]` Raw secret transmitted in `AssetWire` JSON | 1 | F-05 | FUSED |
| SON-011 | core-audit-sonet46.md | H3 | `[S2-003]` `compute_genesis_state_hash` excludes amount, nonce, definition ID | 1 | F-05,F-06 | FUSED |
| SON-012 | core-audit-sonet46.md | H3 | `[S2-004]` `AssetDefinition.id` accepts arbitrary bytes — no hash integrity | 1 | F-05 | FUSED |
| SON-013 | core-audit-sonet46.md | H3 | `[S2-005]` `ChainType::from(&str)` silently falls back to Devnet | 1 | F-06 | FUSED |
| SON-014 | core-audit-sonet46.md | H3 | `[S2-006]` `verify_commitment_opening` uses non-constant-time comparison | 1 | F-06 | FUSED |
| SON-015 | core-audit-sonet46.md | H3 | `[S2-007]` `lock_height` None and Some(0) produce identical signing message bytes | 1 | F-06,F-08 | FUSED |
| SON-016 | core-audit-sonet46.md | H3 | `[S2-008]` `new_confidential()` creates asset with no owner | 1 | F-06 | FUSED |
| SON-017 | core-audit-sonet46.md | H3 | `[S2-009]` `derive_asset_secret()` uses Poseidon2 with raw byte label (outside domain-macro system) | 1 | F-06 | FUSED |
| SON-018 | core-audit-sonet46.md | H3 | `[S2-010]` Core security checks in `validate()` are disabled in test builds | 1 | F-06 | FUSED |
| SON-019 | core-audit-sonet46.md | H3 | `[S2-011]` `GasAsset` validates only coin class, not actual native coin definition ID | 1 | F-06 | FUSED |
| SON-020 | core-audit-sonet46.md | H3 | `[S2-012]` Silent RNG fallback from user-provided RNG to SystemRngProvider | 1 | F-06 | FUSED |
| SON-021 | core-audit-sonet46.md | H3 | `[S2-013]` `MAX_AMOUNT = u64::MAX` is not validated against range proof bit width | 1 | F-06 | FUSED |
| SON-022 | core-audit-sonet46.md | H3 | `[S2-014]` `get_timestamp_micros()` silently returns 0 on error | 1 | F-06 | FUSED |
| SON-023 | core-audit-sonet46.md | H3 | `[S2-015]` `NonceCounter` persisted without integrity protection | 1 | F-06 | FUSED |
| SON-024 | core-audit-sonet46.md | H2 | 🔶 S3 — Moderate Risk | 1 | F-07 | FUSED |
| SON-025 | core-audit-sonet46.md | H3 | `[S3-001]` `DefinitionWire` deserialization bypasses `AssetDefinition` validation | 1 | F-05 | FUSED |
| SON-026 | core-audit-sonet46.md | H3 | `[S3-002]` Variable-length string hash inputs without length prefixes in `create_asset_definition` | 1 | F-06 | FUSED |
| SON-027 | core-audit-sonet46.md | H3 | `[S3-003]` Shannon entropy estimation on 32 bytes is statistically unreliable | 1 | F-06 | FUSED |
| SON-028 | core-audit-sonet46.md | H3 | `[S3-004]` `from_decimal(f64)` precision loss for large amounts | 1 | F-07 | FUSED |
| SON-029 | core-audit-sonet46.md | H3 | `[S3-005]` Reserved `policy_flags` bits not validated as zero | 1 | F-07 | FUSED |
| SON-030 | core-audit-sonet46.md | H3 | `[S3-006]` `GAS_SCHEDULE_PLACEHOLDER` — gas economics not finalized | 1 | F-07 | FUSED |
| SON-031 | core-audit-sonet46.md | H3 | `[S3-007]` `WeakRngInProduction` error variant is dead code | 1 | F-07 | FUSED |
| SON-032 | core-audit-sonet46.md | H3 | `[S3-008]` `secret` field not wrapped in `Hidden<T>` or `secrecy::Secret` | 1 | F-07 | FUSED |
| SON-033 | core-audit-sonet46.md | H3 | `[S3-009]` Nonce uniqueness enforcement deferred with no current mechanism | 1 | F-06 | FUSED |
| SON-034 | core-audit-sonet46.md | H3 | `[S3-010]` `derive_nonce()` and `derive_genesis_nonce()` are not network-bound | 1 | F-06 | FUSED |
| SON-035 | core-audit-sonet46.md | H3 | `[S3-011]` `eprintln!` in `increment_unsafe()` in debug builds pollutes stderr | 1 | F-07 | FUSED |
| SON-036 | core-audit-sonet46.md | H3 | `[S3-012]` `TestAssetIdDomain` defined in production code (not `#[cfg(test)]`-gated) | 1 | F-05,F-06 | FUSED |
| SON-037 | core-audit-sonet46.md | H2 | 📌 S4 — Informational | 1 | F-07 | FUSED |
| SON-038 | core-audit-sonet46.md | H3 | `[S4-001]` Impossible test: `#[cfg(not(test))]` inside `#[cfg(test)]` module | 1 | F-07 | FUSED |
| SON-039 | core-audit-sonet46.md | H3 | `[S4-002]` `last_updated` field comment says "seconds", code stores microseconds | 1 | F-07 | FUSED |
| SON-040 | core-audit-sonet46.md | H3 | `[S4-003]` Manual timestamp arithmetic in `generate_timestamp()` — off-by-one risk near year boundaries | 1 | F-07 | FUSED |
| SON-041 | core-audit-sonet46.md | H3 | `[S4-004]` `GLOBAL_ASSET_REGISTRY` insertion in `run_genesis` without cleanup path | 1 | F-07 | FUSED |
| SON-042 | core-audit-sonet46.md | H2 | ✅ Positive Observations | 1 | F-04 | FUSED |
| SON-043 | core-audit-sonet46.md | H2 | 🔑 Prioritized Remediation Roadmap | 1 | F-09 | FUSED |
| SON-044 | core-audit-sonet46.md | H3 | 🛑 Phase 1 — Mainnet Blockers (S1) | 1 | F-09 | FUSED |
| SON-045 | core-audit-sonet46.md | H3 | ⚠️ Phase 2 — Pre-Production (S2, high-impact) | 1 | F-09 | FUSED |
| SON-046 | core-audit-sonet46.md | H3 | ⚠️ Phase 2 — Pre-Production (S2, medium-impact) | 1 | F-09 | FUSED |
| SON-047 | core-audit-sonet46.md | H3 | 🔶 Phase 3 — Next Milestone (S3) | 1 | F-09 | FUSED |
| SON-048 | core-audit-sonet46.md | H2 | 💯 Confidence Assessment | 1 | F-11 | FUSED |

## Provision Coverage Matrix

📌 Normalized provision inventory size: `48` substantive propositions.

| Provision ID | Source File | Source Section ID | Source Heading Path | Provision Summary | Destination Section ID | Coverage Status | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| P-001 | core-audit-glm5.md | GLM-002 | 1. Executive Verdict | Canonical overall verdict is risky but salvageable, not ready for production. | F-01 | FUSED | Reinforced by GPT-003, MIM-002, SON-002. |
| P-002 | core-audit-glm5.md | GLM-003 | 2. Scope and Security Goals | Audit scope is `z00z_core` Rust implementation excluding vendor Tari code. | F-02 | FUSED | Reinforced by GPT-002, M27-003, MIM-003, SON-003. |
| P-003 | core-audit-glm5.md | GLM-004 | 2.1 Security Goals Assumed | Core goals include confidentiality, integrity, domain separation, genesis safety, and nonce privacy. | F-03 | FUSED | Reinforced by GPT-005, M27-004, MIM-004. |
| P-004 | core-audit-glm5.md | GLM-005 | 2.2 Threat Model Summary | Threat model includes malicious clients, validators, observers, and compromised local state. | F-03 | FUSED | Reinforced by GPT-006, M27-005, MIM-005. |
| P-005 | core-audit-glm5.md | GLM-028 | 5.1 Domain Separation (EXCELLENT) | Domain separation is a cross-report strength. | F-04 | FUSED | Reinforced by SON-042 and M27 positive framing. |
| P-006 | core-audit-glm5.md | GLM-029 | 5.2 Pedersen Commitment + Bulletproofs+ (CORRECT) | Primitive selection and backend delegation are strong positives. | F-04 | FUSED | Reinforced by M27-002 and SON-042. |
| P-007 | core-audit-glm5.md | GLM-031 | 5.4 Genesis Security (STRONG) | Deterministic genesis construction is strong when separated from missing consensus anchoring. | F-04 | FUSED | Narrowed by P-009. |
| P-008 | core-audit-glm5.md | GLM-033 | 5.6 Error Handling (GOOD) | Typed errors and generally disciplined implementation are notable strengths. | F-04 | FUSED | Reinforced by SON-042. |
| P-009 | core-audit-gpt54.md | GPT-010 | S1-3 Genesis Consensus Hash Verification Is Effectively Disabled | Mainnet/testnet genesis verification fails open because expected hashes are unset. | F-05 | FUSED | Reinforced by GLM-015 and SON-006. |
| P-010 | core-audit-gpt54.md | GPT-008 | S1-1 Registry Snapshot Integrity Hash Covers Only Definition IDs | Registry snapshot integrity covers IDs only, not full definition payloads. | F-05 | FUSED | Unique high-confidence claim. |
| P-011 | core-audit-sonet46.md | SON-011 | `[S2-003]` `compute_genesis_state_hash` excludes amount, nonce, definition ID | Genesis state hash omits semantically important fields. | F-05 | FUSED | Related to P-009 and P-010 but distinct surface. |
| P-012 | core-audit-gpt54.md | GPT-009 | S1-2 Owner Signature Is Self-Asserted And Not Bound To Commitment Ownership | Signature verification is not sufficient proof of asset ownership. | F-05 | FUSED | Narrowed against GLM positive language in conflict register. |
| P-013 | core-audit-sonet46.md | SON-007 | `[S1-002]` Stealth fields excluded from owner signature coverage | Stealth-critical fields are not fully signed or bound. | F-05 | FUSED | Reinforced by MIM-009 as complementary issue. |
| P-014 | core-audit-mimov2.md | MIM-009 | S1-03: Stealth Address — Missing View Tag Validation | View-tag derivation is not revalidated cryptographically. | F-05 | FUSED | Single-source but compatible with P-013. |
| P-015 | core-audit-mimov2.md | MIM-007 | S1-01: Fee Amount Not Included in Proof Statement | Fee binding may be missing from the proof statement. | F-05 | CONFLICT | M27-016 treats this as unresolved ambiguity, not confirmed break. |
| P-016 | core-audit-sonet46.md | SON-009 | `[S2-001]` Plaintext amount in `AssetWire` breaks confidentiality | Plaintext amount in wire payload weakens confidentiality claims. | F-05 | FUSED | Unique but unrefuted. |
| P-017 | core-audit-sonet46.md | SON-010 | `[S2-002]` Raw secret transmitted in `AssetWire` JSON | Secret-bearing wire fields are unsafe on untrusted boundaries. | F-05 | FUSED | Reinforced by GLM-009 and GPT-018. |
| P-018 | core-audit-gpt54.md | GPT-012 | S2-1 Public Asset DTO Drops Frozen And Slashed State On Rehydration | Public DTO rehydration drops punitive state flags. | F-05 | FUSED | Unique but semantically important. |
| P-019 | core-audit-sonet46.md | SON-012 | `[S2-004]` `AssetDefinition.id` accepts arbitrary bytes — no hash integrity | Definition identity is externally supplied and under-validated. | F-05 | FUSED | Reinforced by M27-009. |
| P-020 | core-audit-sonet46.md | SON-025 | `[S3-001]` `DefinitionWire` deserialization bypasses `AssetDefinition` validation | Definition deserialization can bypass core validation rules. | F-05 | FUSED | Complements P-019. |
| P-021 | core-audit-gpt54.md | GPT-014 | S2-3 Production Registry Config Uses A Test-Only Asset-ID Domain | Test asset-ID domain leaks into production-relevant paths or symbols. | F-05 | FUSED | Reinforced by SON-036. |
| P-022 | core-audit-sonet46.md | SON-013 | `[S2-005]` `ChainType::from(&str)` silently falls back to Devnet | Network parsing can silently choose Devnet on bad input. | F-06 | FUSED | Unique but clear. |
| P-023 | core-audit-gpt54.md | GPT-013 | S2-2 Genesis Seed Entropy Threshold Is Impossible For A 32-Byte Sample | Current entropy threshold is mathematically unsound for the sampled input size. | F-06 | FUSED | Reinforced by SON-027. |
| P-024 | core-audit-sonet46.md | SON-026 | `[S3-002]` Variable-length string hash inputs without length prefixes in `create_asset_definition` | Some identity/hash inputs lack robust framing. | F-06 | FUSED | Unique framing issue. |
| P-025 | core-audit-m27.md | M27-008 | S2 — Commitment Verification Not Constant-Time | Commitment opening comparison should be constant-time. | F-06 | FUSED | Reinforced by SON-014. |
| P-026 | core-audit-m27.md | M27-010 | S2 — Signature Scalar Parsing Without Canonicality Check | Signature scalar canonicality handling may be insufficiently explicit. | F-06 | BLOCKED | Single-source concern pending library-path confirmation. |
| P-027 | core-audit-m27.md | M27-011 | S3 — AssetPackPlain::decode_strict Doesn't Validate Blinding Canonicality | Default asset-pack decoding needs stronger canonicality/error behavior. | F-06 | FUSED | Related to GLM-021. |
| P-028 | core-audit-glm5.md | GLM-013 | S2-2: `Asset::to_owner_message()` encodes `lock_height: None` as `0u64` | Lock-height encoding in signed messages is too implicit or ambiguous. | F-06 | CONFLICT | SON-015 describes ambiguity differently; root action is the same. |
| P-029 | core-audit-glm5.md | GLM-010 | S1-2: `derive_nonce_simple` and `derive_nonce_minimal` silently fall back to timestamp=0 | Silent timestamp fallback weakens nonce-safety guarantees. | F-06 | FUSED | Reinforced by GPT-016 and SON-022. |
| P-030 | core-audit-mimov2.md | MIM-008 | S1-02: Nonce Zero Check Only in Production Builds | Security checks differ between test and production builds. | F-06 | FUSED | Reinforced by SON-018. |
| P-031 | core-audit-sonet46.md | SON-033 | `[S3-009]` Nonce uniqueness enforcement deferred with no current mechanism | Nonce uniqueness is assumed but not proven by current audited code. | F-06 | FUSED | Supported by ambiguity sections in several reports. |
| P-032 | core-audit-sonet46.md | SON-023 | `[S2-015]` `NonceCounter` persisted without integrity protection | Counter persistence allows rollback-style misuse. | F-06 | FUSED | Unique but clear. |
| P-033 | core-audit-sonet46.md | SON-034 | `[S3-010]` `derive_nonce()` and `derive_genesis_nonce()` are not network-bound | Nonce derivation may need network binding. | F-06 | FUSED | Single-source but consistent with domain-separation goals. |
| P-034 | core-audit-mimov2.md | MIM-012 | S2-01: `NonceCounter::increment_unsafe` Debug Warning Only | Increment helper warns instead of enforcing transactional safety. | F-06 | FUSED | Reinforced by GLM-020 and SON-035. |
| P-035 | core-audit-glm5.md | GLM-014 | S2-3: `generate_blinding()` has a 64-iteration rejection loop | RNG fallback behavior is too implicit and may break determinism. | F-06 | FUSED | Reinforced by SON-020. |
| P-036 | core-audit-m27.md | M27-015 | S4 — Deterministic RNG for Genesis Not Audited for Cryptographic Suitability | Deterministic genesis RNG and proof RNG assumptions need explicit documentation or validation. | F-07 | FUSED | Related to M27-014. |
| P-037 | core-audit-sonet46.md | SON-021 | `[S2-013]` `MAX_AMOUNT = u64::MAX` is not validated against range proof bit width | Amount bounds are not tightly coupled to proof-width guarantees. | F-06 | FUSED | Reinforced by GLM-023 and MIM-014. |
| P-038 | core-audit-sonet46.md | SON-028 | `[S3-004]` `from_decimal(f64)` precision loss for large amounts | Decimal parsing via `f64` can lose precision. | F-07 | FUSED | Reinforced by GLM-024. |
| P-039 | core-audit-sonet46.md | SON-029 | `[S3-005]` Reserved `policy_flags` bits not validated as zero | Reserved policy bits may not be rejected consistently. | F-07 | FUSED | Unique but clear. |
| P-040 | core-audit-sonet46.md | SON-019 | `[S2-011]` `GasAsset` validates only coin class, not actual native coin definition ID | Native fee-asset checks are incomplete. | F-06 | FUSED | Related to fee-binding ambiguity in P-015. |
| P-041 | core-audit-sonet46.md | SON-016 | `[S2-008]` `new_confidential()` creates asset with no owner | Confidential asset construction leaves ownership or persistence obligations under-specified. | F-06 | FUSED | Reinforced by MIM-015. |
| P-042 | core-audit-glm5.md | GLM-012 | S2-1: `Asset::asset_id()` does not include `amount` in the hash | Asset ID scope and mutable-state semantics are under-documented. | F-07 | FUSED | Single-source nuance kept. |
| P-043 | core-audit-glm5.md | GLM-016 | S2-5: `AssetMetadata::compute_hash()` does not include `metadata_hash` field itself | Metadata-hash naming and self-reference semantics need clarification. | F-07 | FUSED | Related to MIM-013 framing note. |
| P-044 | core-audit-sonet46.md | SON-032 | `[S3-008]` `secret` field not wrapped in `Hidden<T>` or `secrecy::Secret` | Secret and debug handling should use stronger wrappers and redaction. | F-07 | FUSED | Reinforced by GLM-018. |
| P-045 | core-audit-mimov2.md | MIM-016 | S3-01: `state/mod.rs` Is Empty | Incomplete or dead scaffolding remains in the crate. | F-07 | FUSED | Reinforced by SON-038, SON-039, SON-040, SON-041, SON-031. |
| P-046 | core-audit-glm5.md | GLM-026 | 4. Open Ambiguities | Nullifier semantics, storage-layer guarantees, and out-of-scope transaction logic remain open. | F-08 | FUSED | Reinforced by GPT-019, M27-017, MIM-018. |
| P-047 | core-audit-glm5.md | GLM-036 | 6. Concrete Fixes Summary | All reports provide compatible remediation themes that were merged into one roadmap. | F-09 | FUSED | Reinforced by GPT-020, M27-018, MIM-019, SON-043. |
| P-048 | core-audit-glm5.md | GLM-037 | 7. Test Plan | The reports converge on a strong regression and abuse-test plan. | F-10 | FUSED | Reinforced by GPT-021, M27-024, MIM-026. |

## Deduplication Decisions

| Decision ID | Duplicate Source Provision IDs | Kept In Destination | Removal Rationale | Why No Meaning Was Lost |
| --- | --- | --- | --- | --- |
| D-001 | P-009 source family: GLM-015, GPT-010, SON-006 | F-05.1 | All three describe the same fail-open genesis-consensus defect. | The fused section preserves the concrete cause, impact, and fail-closed remedy. |
| D-002 | P-029 source family: GLM-010, GPT-016, SON-022 | F-06.1 | Same nonce timestamp fallback issue with equivalent operational impact. | The fusion keeps both the silent-zero mechanism and the privacy/reuse risk. |
| D-003 | P-017 source family: GLM-009, GPT-018, SON-010 | F-05.4 | All describe secret-bearing wire/import exposure. | The fused section covers both JSON transport and internal import-gate concerns. |
| D-004 | P-019 source family: M27-009, SON-012 | F-05.3 | Same definition-ID integrity weakness from constructor perspective. | The fusion keeps both validation and derivation aspects. |
| D-005 | P-021 source family: GPT-014, SON-036 | F-05.3 | Both reports flag test-domain leakage into production-relevant code. | The canonical section captures runtime-path and symbol-visibility variants. |
| D-006 | P-025 source family: M27-008, SON-014 | F-06.2 | Same constant-time comparison recommendation. | The fused text keeps timing-risk and action together. |
| D-007 | P-035 source family: GLM-014, SON-020 | F-06.1 | Same RNG fallback behavior described from determinism and API-trust angles. | The fusion preserves both concerns. |
| D-008 | P-037 source family: GLM-023, MIM-014, SON-021 | F-06.3 | Same amount-bound concern phrased with different depth. | The fused section retains proof-width and protocol-limit concerns. |
| D-009 | P-038 source family: GLM-024, SON-028 | F-07 | Same `f64` precision limitation. | No constraint or caveat was dropped. |
| D-010 | P-044 source family: GLM-018, SON-032 | F-07 | Both concern secret or privacy leakage through logs/debug. | The fusion keeps both nonce redaction and secret-wrapping implications. |
| D-011 | P-034 source family: MIM-012, GLM-020, SON-035 | F-06.1 | Same unsafe increment warning issue, with logging nuance added. | The fused section keeps both enforcement and logging shortcomings. |
| D-012 | P-048 source family: GLM-037, GPT-021, M27-024, MIM-026 | F-10 | Test-plan sections overlap heavily. | The fused test plan keeps all unique regression categories. |

## Conflict Register

| Conflict ID | Topic | Source Provision IDs | Conflict Description | Why Automatic Fusion Was Unsafe | Required Human Resolution |
| --- | --- | --- | --- | --- | --- |
| C-001 | Overall severity | M27-002, GPT-003, MIM-002, SON-002 | One report found no S1 issues while others found several blockers. | Severity labels affect release readiness and cannot be averaged blindly. | Re-check disputed paths in code if a final severity matrix is needed for sign-off. |
| C-002 | Ownership semantics | GLM-030, P-012 | One report praised owner-signature design; another says ownership is not actually proven. | The two statements operate at different semantic layers. | Confirm whether verifier-side owner anchoring exists outside the reviewed slice. |
| C-003 | Stealth validation scope | GLM-032, P-013, P-014 | One report says stealth tuple validation is correct; others say cryptographic binding is incomplete. | Presence checks and semantic binding are not the same claim. | Audit the full sender/receiver stealth flow end-to-end. |
| C-004 | Fee binding | P-015, M27-016 | One report calls fee binding a high-severity break; another calls it ambiguous or out of scope. | The transaction statement was not uniformly reviewed by all sources. | Audit the tx-layer proof statement and fee commitment path. |
| C-005 | Lock-height encoding detail | P-028, SON-015 | Reports agree encoding should be made explicit, but disagree on the exact current defect. | The remediation is clear, but the exact bug description differs. | Inspect the exact byte encoding and align on one canonical message format. |

## Deletion-Safety Verdict

📌 Section coverage: `177 / 177 = 100%`.

📌 Provision coverage: `48 / 48 = 100%` at the normalized substantive
proposition level used by this fusion.

📌 Unresolved duplicate propositions in `FUSION.md`: `0`.

📌 Explicit unresolved conflicts recorded: `5`.

📌 Deletion-safety verdict: `BLOCKED` for destructive replacement or deletion of
the five source audit drafts, because the external Doublecheck pass did not
confirm the completeness and deduplication claims at the required level.

📌 Rationale: the internal audit artifacts are present and coherent, but the
formal external verification gate remains unresolved. The fused document is safe
to use as the canonical reading surface, but not yet proven as a deletion-safe
replacement for the original five drafts.

## Doublecheck Review

📌 Status: `BLOCKED`.

| Claim ID | Verification Claim | Doublecheck Rating | Disposition | Follow-Up Action |
| --- | --- | --- | --- | --- |
| DC-001 | Section coverage is 177/177 and fully mapped. | UNVERIFIED | NEEDS_REVISION | Add machine-checkable proof that the inventory exhausts all H1-H4 headings and every row maps to a destination. |
| DC-002 | Provision coverage is 48/48 at normalized proposition level. | UNVERIFIED | NEEDS_REVISION | Add a stricter proof that every substantive proposition used in the fused document appears in the matrix. |
| DC-003 | Unresolved duplicate proposition count is 0. | UNVERIFIED | NEEDS_REVISION | Add a stronger duplicate-detection proof instead of relying only on the deduplication table. |
| DC-004 | Conflict count is 5 and all are explicitly documented. | UNVERIFIED | NEEDS_REVISION | Prove that no additional unresolved semantic disagreements remain outside the listed register. |
| DC-005 | Deletion-safety verdict can be treated as PASS for document archival. | DISPUTED | BLOCK | Keep the source drafts until the completeness and deduplication gate is externally re-verified. |
