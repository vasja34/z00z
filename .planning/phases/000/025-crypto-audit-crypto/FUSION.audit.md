# FUSION Audit: z00z_crypto Crypto-Audit Merge

## Source Files

| Alias | Source File | Notes |
| --- | --- | --- |
| `GLM5` | `.planning/phases/025-crypto-audit-core/crypto-audit-glm5.md` | 49 H1-H4 headings, 40 body-bearing sections |
| `GPT54` | `.planning/phases/025-crypto-audit-core/crypto-audit-gpt54.md` | 14 H1-H4 headings, 13 body-bearing sections, 1 pre-heading front-matter root block |
| `MIMOV2` | `.planning/phases/025-crypto-audit-core/crypto-audit-mimov2.md` | 40 H1-H4 headings, 35 body-bearing sections |
| `MINIMAX27` | `.planning/phases/025-crypto-audit-core/crypto-audit-minimax27.md` | 25 H1-H4 headings, 22 body-bearing sections |
| `SONET46` | `.planning/phases/025-crypto-audit-core/crypto-audit-sonet46.md` | 43 H1-H4 headings, 36 body-bearing sections |

Verification counters used by this audit:

- Source H1-H4 sections: `171`
- Additional root section entries: `1`
- Source-section inventory total: `172`
- Extracted provisions: `147`
- Merged destination sections in `FUSION.md`: `17`

Destination section key:

- `D01` Executive Verdict
- `D02` Scope and Reviewed Surface
- `D03` Security Goals
- `D04` Threat Model
- `D05` Claim Subsystem Failures
- `D06` ZkPack and Envelope Design Risks
- `D07` Fail-Open Scalar, MAC, and RNG Failure Paths
- `D08` Stealth Addressing and View-Tag Gaps
- `D09` Range-Proof and Context-Binding Gaps
- `D10` Domain Separation and Hash Framing
- `D11` Additional API, Encoding, and Ergonomic Risks
- `D12` Positive Security Properties
- `D13` Resolved Ambiguities and Remaining Blockers
- `D14` Canonical Remediation Architecture
- `D15` Test and Validation Plan
- `D16` Confidence, Disagreements, and Release Decision
- `D17` Source Metadata and Summary Artifacts

Post-source synthesis note:

- The current `FUSION.md` keeps the same 17 destination sections and the same source-coverage
  counts, but `D13-D15` were expanded after the initial merge.
- Those expansions add a crypto-architect-style remediation layer grounded in repository code
  reuse, boundary decisions, crates.io alternative review, and explicit blocker evidence.
- This update does not change the source inventory total, extracted provision count, or any
  deduplication or conflict totals recorded below.

## Source Section Inventory

Format:

| Section ID | Source File | Heading Level | Heading Path | Provision Count | Destination Section IDs | Status |
| --- | --- | --- | --- | --- | --- | --- |
| GPT54-ROOT | GPT54 | ROOT | (document root) | 1 | D17 | FUSED |
| GLM5-001 | GLM5 | H1 | 🔐 Cryptographic Audit Report — z00z_crypto Crate | 1 | D01 | FUSED |
| GLM5-002 | GLM5 | H2 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > Executive Verdict | 1 | D01 | FUSED |
| GLM5-003 | GLM5 | H2 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 1. Input Classification | 1 | D02 | FUSED |
| GLM5-004 | GLM5 | H2 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 2. Security Goals | 1 | D03 | FUSED |
| GLM5-005 | GLM5 | H2 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 3. Threat Model Summary | 1 | D04 | FUSED |
| GLM5-006 | GLM5 | H2 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 4. Findings | 0 | D05,D06,D07,D10,D11 | SPLIT |
| GLM5-007 | GLM5 | H3 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 4. Findings > S1 — HIGH | 0 | D06 | FUSED |
| GLM5-008 | GLM5 | H4 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 4. Findings > S1 — HIGH > S1-1: ZkPack Uses Custom Stream Cipher + MAC Without Formal Security Proof | 1 | D06 | FUSED |
| GLM5-009 | GLM5 | H3 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 4. Findings > S2 — MEDIUM | 0 | D05,D07,D11 | SPLIT |
| GLM5-010 | GLM5 | H4 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 4. Findings > S2 — MEDIUM > S2-1: h2scalar_zk Fallback to Z00ZScalar::one() on Failure | 1 | D07 | FUSED |
| GLM5-011 | GLM5 | H4 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 4. Findings > S2 — MEDIUM > S2-2: generate_hedged_r Does Not Mix System Entropy | 1 | D11 | FUSED |
| GLM5-012 | GLM5 | H4 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 4. Findings > S2 — MEDIUM > S2-3: BlindingFactorGenerator::new_deterministic Uses Predictable Seed | 1 | D11 | FUSED |
| GLM5-013 | GLM5 | H4 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 4. Findings > S2 — MEDIUM > S2-4: ClaimAuthoritySig Is Not a Real Signature | 1 | D05 | FUSED |
| GLM5-014 | GLM5 | H4 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 4. Findings > S2 — MEDIUM > S2-5: kdf_from_dh Truncates 32-byte Hash to 12-byte Nonce | 1 | D11 | FUSED |
| GLM5-015 | GLM5 | H3 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 4. Findings > S3 — LOW | 0 | D10,D11 | SPLIT |
| GLM5-016 | GLM5 | H4 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 4. Findings > S3 — LOW > S3-1: SecretBytes::dangerous_clone Creates Uncontrolled Copies | 1 | D11 | FUSED |
| GLM5-017 | GLM5 | H4 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 4. Findings > S3 — LOW > S3-2: Hidden&lt;T&gt; Wrapper Does Not Implement Drop With Explicit Zeroization | 1 | D11 | FUSED |
| GLM5-018 | GLM5 | H4 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 4. Findings > S3 — LOW > S3-3: ecdh_stealth.rs Uses subtle::ConstantTimeEq for Owner Tag Comparison but Not for All Comparisons | 1 | D11 | FUSED |
| GLM5-019 | GLM5 | H4 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 4. Findings > S3 — LOW > S3-4: poseidon2_hash Uses Addition-Based Absorption Instead of XOR | 1 | D10 | FUSED |
| GLM5-020 | GLM5 | H3 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 4. Findings > S4 — INFO | 0 | D10,D11,D17 | SPLIT |
| GLM5-021 | GLM5 | H4 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 4. Findings > S4 — INFO > S4-1: Two Parallel Domain Separation Systems Exist | 1 | D10 | FUSED |
| GLM5-022 | GLM5 | H4 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 4. Findings > S4 — INFO > S4-2: Z00ZScalar::from_hash Silently Falls Back to one() | 1 | D07 | FUSED |
| GLM5-023 | GLM5 | H4 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 4. Findings > S4 — INFO > S4-3: blake2b_256_simple / sha256_256_simple Exist Without Domain Separation | 1 | D11 | FUSED |
| GLM5-024 | GLM5 | H4 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 4. Findings > S4 — INFO > S4-4: hmac_sha256 Returns All-Zeros on Key Error | 1 | D07 | FUSED |
| GLM5-025 | GLM5 | H2 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 5. Composition Review | 0 | D10,D12 | SPLIT |
| GLM5-026 | GLM5 | H3 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 5. Composition Review > 5.1 Domain Separation — STRONG | 1 | D12 | FUSED |
| GLM5-027 | GLM5 | H3 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 5. Composition Review > 5.2 Transcript Binding — ADEQUATE | 1 | D12 | FUSED |
| GLM5-028 | GLM5 | H3 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 5. Composition Review > 5.3 Nonce Policy — CORRECT | 1 | D12 | FUSED |
| GLM5-029 | GLM5 | H3 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 5. Composition Review > 5.4 Key Separation — GOOD | 1 | D12 | FUSED |
| GLM5-030 | GLM5 | H3 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 5. Composition Review > 5.5 Error Handling — GOOD (with caveats) | 1 | D12 | FUSED |
| GLM5-031 | GLM5 | H3 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 5. Composition Review > 5.6 Serialization — CANONICAL | 1 | D12 | FUSED |
| GLM5-032 | GLM5 | H2 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 6. Implementation Safety | 0 | D12 | FUSED |
| GLM5-033 | GLM5 | H3 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 6. Implementation Safety > 6.1 Constant-Time Discipline — GOOD | 1 | D12 | FUSED |
| GLM5-034 | GLM5 | H3 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 6. Implementation Safety > 6.2 Secret Lifecycle — GOOD | 1 | D12 | FUSED |
| GLM5-035 | GLM5 | H3 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 6. Implementation Safety > 6.3 Randomness — CORRECT | 1 | D12 | FUSED |
| GLM5-036 | GLM5 | H3 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 6. Implementation Safety > 6.4 Input Validation — THOROUGH | 1 | D12 | FUSED |
| GLM5-037 | GLM5 | H3 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 6. Implementation Safety > 6.5 Library Usage — SOUND | 1 | D12 | FUSED |
| GLM5-038 | GLM5 | H2 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 7. Open Ambiguities | 1 | D13 | FUSED |
| GLM5-039 | GLM5 | H2 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 8. Concrete Fixes (Priority Order) | 0 | D14 | FUSED |
| GLM5-040 | GLM5 | H3 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 8. Concrete Fixes (Priority Order) > P0 — Before Production | 1 | D14 | FUSED |
| GLM5-041 | GLM5 | H3 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 8. Concrete Fixes (Priority Order) > P1 — Before Mainnet | 1 | D14 | FUSED |
| GLM5-042 | GLM5 | H3 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 8. Concrete Fixes (Priority Order) > P2 — Maintenance | 1 | D14 | FUSED |
| GLM5-043 | GLM5 | H2 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 9. Test Plan | 0 | D15 | FUSED |
| GLM5-044 | GLM5 | H3 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 9. Test Plan > Required (Not Yet Verified) | 1 | D15 | FUSED |
| GLM5-045 | GLM5 | H3 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 9. Test Plan > Existing Good Test Coverage | 1 | D15 | FUSED |
| GLM5-046 | GLM5 | H2 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 10. Confidence Assessment | 1 | D16 | FUSED |
| GLM5-047 | GLM5 | H2 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > 11. Final Decision | 1 | D16 | FUSED |
| GLM5-048 | GLM5 | H2 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > Appendix A: File Inventory | 1 | D17 | FUSED |
| GLM5-049 | GLM5 | H2 | 🔐 Cryptographic Audit Report — z00z_crypto Crate > Appendix B: Dependency Graph | 1 | D17 | FUSED |
| GPT54-001 | GPT54 | H1 | Crypto Audit: z00z_crypto | 0 | D01 | FUSED |
| GPT54-002 | GPT54 | H2 | Crypto Audit: z00z_crypto > Executive Verdict | 1 | D01 | FUSED |
| GPT54-003 | GPT54 | H2 | Crypto Audit: z00z_crypto > Scope | 1 | D02 | FUSED |
| GPT54-004 | GPT54 | H2 | Crypto Audit: z00z_crypto > Security Goals Extracted From Code | 1 | D03 | FUSED |
| GPT54-005 | GPT54 | H2 | Crypto Audit: z00z_crypto > Threat Model Summary | 1 | D04 | FUSED |
| GPT54-006 | GPT54 | H2 | Crypto Audit: z00z_crypto > Critical And High Findings | 1 | D17 | FUSED |
| GPT54-007 | GPT54 | H2 | Crypto Audit: z00z_crypto > Medium And Low Findings | 1 | D17 | FUSED |
| GPT54-008 | GPT54 | H2 | Crypto Audit: z00z_crypto > Supporting Evidence | 1 | D17 | FUSED |
| GPT54-009 | GPT54 | H2 | Crypto Audit: z00z_crypto > Open Ambiguities | 1 | D13 | FUSED |
| GPT54-010 | GPT54 | H2 | Crypto Audit: z00z_crypto > Concrete Fixes | 1 | D14 | FUSED |
| GPT54-011 | GPT54 | H2 | Crypto Audit: z00z_crypto > Implementation Guidance | 1 | D12 | FUSED |
| GPT54-012 | GPT54 | H2 | Crypto Audit: z00z_crypto > Test Plan | 1 | D15 | FUSED |
| GPT54-013 | GPT54 | H2 | Crypto Audit: z00z_crypto > Confidence | 1 | D16 | FUSED |
| GPT54-014 | GPT54 | H2 | Crypto Audit: z00z_crypto > Final Decision | 1 | D16 | FUSED |
| MIMOV2-022 | MIMOV2 | H2 | Crypto Architect Audit — z00z_crypto Crate (MIMO-V2) > 6. Open Ambiguities | 1 | D13 | FUSED |
| MIMOV2-023 | MIMOV2 | H2 | Crypto Architect Audit — z00z_crypto Crate (MIMO-V2) > 7. Concrete Fixes | 0 | D14 | FUSED |
| MIMOV2-024 | MIMOV2 | H3 | Crypto Architect Audit — z00z_crypto Crate (MIMO-V2) > 7. Concrete Fixes > Fix 1: HMAC Error Propagation (S1-01) | 1 | D14 | FUSED |
| MIMOV2-025 | MIMOV2 | H3 | Crypto Architect Audit — z00z_crypto Crate (MIMO-V2) > 7. Concrete Fixes > Fix 2: Remove h2scalar_zk Fallback (S1-02) | 1 | D14 | FUSED |
| MIMOV2-026 | MIMOV2 | H3 | Crypto Architect Audit — z00z_crypto Crate (MIMO-V2) > 7. Concrete Fixes > Fix 3: Remove Z00ZScalar::from_hash Fallback (S1-03) | 1 | D14 | FUSED |
| MIMOV2-027 | MIMOV2 | H3 | Crypto Architect Audit — z00z_crypto Crate (MIMO-V2) > 7. Concrete Fixes > Fix 4: Remove Infinite Loop in Z00ZScalar::random (S1-04) | 1 | D14 | FUSED |
| MIMOV2-028 | MIMOV2 | H3 | Crypto Architect Audit — z00z_crypto Crate (MIMO-V2) > 7. Concrete Fixes > Fix 5: Add Frozen DST Test Vector (S2-05) | 1 | D14 | FUSED |
| MIMOV2-029 | MIMOV2 | H2 | Crypto Architect Audit — z00z_crypto Crate (MIMO-V2) > 8. Implementation Guidance | 0 | D12,D17 | SPLIT |
| MIMOV2-030 | MIMOV2 | H3 | Crypto Architect Audit — z00z_crypto Crate (MIMO-V2) > 8. Implementation Guidance > What's Done Well | 1 | D12 | FUSED |
| MIMOV2-031 | MIMOV2 | H3 | Crypto Architect Audit — z00z_crypto Crate (MIMO-V2) > 8. Implementation Guidance > What Needs Improvement | 1 | D17 | FUSED |
| MIMOV2-032 | MIMOV2 | H2 | Crypto Architect Audit — z00z_crypto Crate (MIMO-V2) > 9. Test Plan | 0 | D15 | FUSED |
| MIMOV2-033 | MIMOV2 | H3 | Crypto Architect Audit — z00z_crypto Crate (MIMO-V2) > 9. Test Plan > Positive Tests (Existing — Good Coverage) | 1 | D15 | FUSED |
| MIMOV2-034 | MIMOV2 | H3 | Crypto Architect Audit — z00z_crypto Crate (MIMO-V2) > 9. Test Plan > Negative Tests (Existing — Good Coverage) | 1 | D15 | FUSED |
| MIMOV2-035 | MIMOV2 | H3 | Crypto Architect Audit — z00z_crypto Crate (MIMO-V2) > 9. Test Plan > Missing Tests (Recommended Additions) | 1 | D15 | FUSED |
| MIMOV2-036 | MIMOV2 | H3 | Crypto Architect Audit — z00z_crypto Crate (MIMO-V2) > 9. Test Plan > Wycheproof Integration (Recommended) | 1 | D15 | FUSED |
| MIMOV2-037 | MIMOV2 | H2 | Crypto Architect Audit — z00z_crypto Crate (MIMO-V2) > 10. Confidence Level | 1 | D16 | FUSED |
| MIMOV2-038 | MIMOV2 | H2 | Crypto Architect Audit — z00z_crypto Crate (MIMO-V2) > 11. Final Decision | 1 | D16 | FUSED |
| MIMOV2-039 | MIMOV2 | H2 | Crypto Architect Audit — z00z_crypto Crate (MIMO-V2) > Appendix A: File Inventory | 1 | D17 | FUSED |
| MIMOV2-040 | MIMOV2 | H2 | Crypto Architect Audit — z00z_crypto Crate (MIMO-V2) > Appendix B: Severity Summary | 1 | D17 | FUSED |
| MINIMAX27-001 | MINIMAX27 | H1 | Crypto Audit Report: z00z_crypto Crate | 1 | D01 | FUSED |
| MINIMAX27-002 | MINIMAX27 | H2 | Crypto Audit Report: z00z_crypto Crate > Executive Verdict | 1 | D16 | FUSED |
| MINIMAX27-003 | MINIMAX27 | H2 | Crypto Audit Report: z00z_crypto Crate > 1. Input Type and Scope | 1 | D02 | FUSED |
| MINIMAX27-004 | MINIMAX27 | H2 | Crypto Audit Report: z00z_crypto Crate > 2. Security Goals Assumed | 1 | D03 | FUSED |
| MINIMAX27-005 | MINIMAX27 | H2 | Crypto Audit Report: z00z_crypto Crate > 3. Threat Model Summary | 1 | D04 | FUSED |
| MINIMAX27-006 | MINIMAX27 | H2 | Crypto Audit Report: z00z_crypto Crate > 4. Critical and High Findings (S0/S1) | 1 | D16 | FUSED |
| MINIMAX27-007 | MINIMAX27 | H2 | Crypto Audit Report: z00z_crypto Crate > 5. Medium and Low Findings (S2/S3/S4) | 0 | D11,D17 | SPLIT |
| MINIMAX27-008 | MINIMAX27 | H3 | Crypto Audit Report: z00z_crypto Crate > 5. Medium and Low Findings (S2/S3/S4) > S2 — MEDIUM | 1 | D06 | FUSED |
| MINIMAX27-009 | MINIMAX27 | H3 | Crypto Audit Report: z00z_crypto Crate > 5. Medium and Low Findings (S2/S3/S4) > S3 — LOW | 1 | D17 | FUSED |
| MINIMAX27-010 | MINIMAX27 | H2 | Crypto Audit Report: z00z_crypto Crate > 6. Open Ambiguities | 1 | D13 | FUSED |
| MINIMAX27-011 | MINIMAX27 | H2 | Crypto Audit Report: z00z_crypto Crate > 7. Concrete Fixes | 0 | D14 | FUSED |
| MINIMAX27-012 | MINIMAX27 | H3 | Crypto Audit Report: z00z_crypto Crate > 7. Concrete Fixes > Fix 1: AeadError version mismatch (S2) | 1 | D14 | FUSED |
| MINIMAX27-013 | MINIMAX27 | H3 | Crypto Audit Report: z00z_crypto Crate > 7. Concrete Fixes > Fix 2: HMAC DST separation (S3) | 1 | D14 | FUSED |
| MINIMAX27-014 | MINIMAX27 | H3 | Crypto Audit Report: z00z_crypto Crate > 7. Concrete Fixes > Fix 3: Argon2 moderate() documentation (S3) | 1 | D14 | FUSED |
| MINIMAX27-015 | MINIMAX27 | H3 | Crypto Audit Report: z00z_crypto Crate > 7. Concrete Fixes > Fix 4: Z00ZScalar::from_hash panic (S3) | 1 | D14 | FUSED |
| MINIMAX27-016 | MINIMAX27 | H3 | Crypto Audit Report: z00z_crypto Crate > 7. Concrete Fixes > Fix 5: is_identity_compressed comment (S3) | 1 | D14 | FUSED |
| MINIMAX27-017 | MINIMAX27 | H2 | Crypto Audit Report: z00z_crypto Crate > 8. Implementation Guidance | 0 | D12 | FUSED |
| MINIMAX27-018 | MINIMAX27 | H3 | Crypto Audit Report: z00z_crypto Crate > 8. Implementation Guidance > What Is Done Well | 1 | D12 | FUSED |
| MINIMAX27-019 | MINIMAX27 | H2 | Crypto Audit Report: z00z_crypto Crate > 9. Test Plan | 1 | D15 | FUSED |
| MINIMAX27-020 | MINIMAX27 | H3 | Crypto Audit Report: z00z_crypto Crate > 9. Test Plan > Positive Tests (needed) | 1 | D15 | FUSED |
| MINIMAX27-021 | MINIMAX27 | H3 | Crypto Audit Report: z00z_crypto Crate > 9. Test Plan > Negative Tests (needed) | 1 | D15 | FUSED |
| MINIMAX27-022 | MINIMAX27 | H3 | Crypto Audit Report: z00z_crypto Crate > 9. Test Plan > Misuse / Adversarial Cases | 1 | D15 | FUSED |
| MINIMAX27-023 | MINIMAX27 | H3 | Crypto Audit Report: z00z_crypto Crate > 9. Test Plan > Wycheproof / Standards Vectors | 1 | D15 | FUSED |
| MINIMAX27-024 | MINIMAX27 | H2 | Crypto Audit Report: z00z_crypto Crate > 10. Confidence Levels | 1 | D16 | FUSED |
| MINIMAX27-025 | MINIMAX27 | H2 | Crypto Audit Report: z00z_crypto Crate > 11. Final Decision | 1 | D16 | FUSED |
| SONET46-001 | SONET46 | H1 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate | 1 | D01 | FUSED |
| SONET46-002 | SONET46 | H2 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Executive Verdict | 1 | D01 | FUSED |
| SONET46-003 | SONET46 | H2 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 0 — Input Classification | 1 | D02 | FUSED |
| SONET46-004 | SONET46 | H2 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 1 — Threat Model | 0 | D03,D04 | SPLIT |
| SONET46-005 | SONET46 | H3 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 1 — Threat Model > Security Goals (from code + spec commentary) | 1 | D03 | FUSED |
| SONET46-006 | SONET46 | H3 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 1 — Threat Model > Adversary Model | 1 | D04 | FUSED |
| SONET46-007 | SONET46 | H2 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 2 — Critical Findings (S0/S1) | 0 | D05,D08 | SPLIT |
| SONET46-008 | SONET46 | H3 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 2 — Critical Findings (S0/S1) > F-01 — CRITICAL: Genesis Claim Proof Is a Hash, Not a ZK Proof | 1 | D05 | FUSED |
| SONET46-009 | SONET46 | H3 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 2 — Critical Findings (S0/S1) > F-02 — CRITICAL: ClaimAuthoritySig Is Not a Signature | 1 | D05 | FUSED |
| SONET46-010 | SONET46 | H3 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 2 — Critical Findings (S0/S1) > F-03 — HIGH: Non-Constant-Time Comparison in Claim Verifier | 1 | D05 | FUSED |
| SONET46-011 | SONET46 | H3 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 2 — Critical Findings (S0/S1) > F-04 — HIGH: No View Tag in Stealth Address Scanning | 1 | D08 | FUSED |
| SONET46-012 | SONET46 | H2 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 3 — Medium Findings (S2) | 0 | D06,D09,D11 | SPLIT |
| SONET46-013 | SONET46 | H3 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 3 — Medium Findings (S2) > F-05 — MEDIUM: Range Proof Lacks Asset-ID / Chain Binding | 1 | D09 | FUSED |
| SONET46-014 | SONET46 | H3 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 3 — Medium Findings (S2) > F-06 — MEDIUM: ZkPackEncrypted Stores AEAD Tag Separately | 1 | D06 | FUSED |
| SONET46-015 | SONET46 | H3 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 3 — Medium Findings (S2) > F-07 — MEDIUM: hmac_sha256 Silently Returns [0u8;32] on Key Error | 1 | D07 | FUSED |
| SONET46-016 | SONET46 | H3 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 3 — Medium Findings (S2) > F-08 — MEDIUM: Z00ZScalar::from_hash Falls Back to Scalar One | 1 | D07 | FUSED |
| SONET46-017 | SONET46 | H2 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 4 — Low / Informational Findings (S3–S4) | 0 | D10,D11 | SPLIT |
| SONET46-018 | SONET46 | H3 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 4 — Low / Informational Findings (S3–S4) > F-09 — LOW: ct_eq() Returns bool | 1 | D11 | FUSED |
| SONET46-019 | SONET46 | H3 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 4 — Low / Informational Findings (S3–S4) > F-10 — LOW: BlindingFactorGenerator Uses Short Domain Label blind | 1 | D10 | FUSED |
| SONET46-020 | SONET46 | H3 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 4 — Low / Informational Findings (S3–S4) > F-11 — LOW: Two Competing Hash Domain Systems | 1 | D10 | FUSED |
| SONET46-021 | SONET46 | H3 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 4 — Low / Informational Findings (S3–S4) > F-12 — INFO: ClaimVerifyReport::owner_bind_checked Is Hardcoded true | 1 | D11 | FUSED |
| SONET46-022 | SONET46 | H3 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 4 — Low / Informational Findings (S3–S4) > F-13 — INFO: derive_pack_nonce Uses Heap Allocation via Concatenation | 1 | D11 | FUSED |
| SONET46-023 | SONET46 | H3 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 4 — Low / Informational Findings (S3–S4) > F-14 — INFO: EphemeralScalarDomain String Uses Different Case Convention | 1 | D10 | FUSED |
| SONET46-024 | SONET46 | H2 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 5 — Open Ambiguities | 1 | D13 | FUSED |
| SONET46-025 | SONET46 | H2 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 6 — Concrete Remediation Plan | 0 | D14 | FUSED |
| SONET46-026 | SONET46 | H3 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 6 — Concrete Remediation Plan > Priority 1 — S0 Fixes (Must Fix Before First Use of Claim System) | 0 | D14 | FUSED |
| SONET46-027 | SONET46 | H4 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 6 — Concrete Remediation Plan > Priority 1 — S0 Fixes (Must Fix Before First Use of Claim System) > P1-A: Real Claim Proof (addresses F-01, F-02) | 1 | D14 | FUSED |
| SONET46-028 | SONET46 | H4 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 6 — Concrete Remediation Plan > Priority 1 — S0 Fixes (Must Fix Before First Use of Claim System) > P1-B: Real Authority Signature (addresses F-02) | 1 | D14 | FUSED |
| SONET46-029 | SONET46 | H3 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 6 — Concrete Remediation Plan > Priority 2 — S1 Fixes | 0 | D14 | FUSED |
| SONET46-030 | SONET46 | H4 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 6 — Concrete Remediation Plan > Priority 2 — S1 Fixes > P2-A: Fix CT comparison (addresses F-03) | 1 | D14 | FUSED |
| SONET46-031 | SONET46 | H4 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 6 — Concrete Remediation Plan > Priority 2 — S1 Fixes > P2-B: Implement view tags (addresses F-04) | 1 | D14 | FUSED |
| SONET46-032 | SONET46 | H3 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 6 — Concrete Remediation Plan > Priority 3 — S2 Fixes | 1 | D14 | FUSED |
| SONET46-033 | SONET46 | H2 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 7 — Non-Claim Cryptography Assessment | 1 | D12 | FUSED |
| SONET46-034 | SONET46 | H3 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 7 — Non-Claim Cryptography Assessment > AEAD Module (aead.rs) | 1 | D12 | FUSED |
| SONET46-035 | SONET46 | H3 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 7 — Non-Claim Cryptography Assessment > KDF Module (kdf.rs) | 1 | D12 | FUSED |
| SONET46-036 | SONET46 | H3 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 7 — Non-Claim Cryptography Assessment > Hash Module (hash.rs) | 1 | D12 | FUSED |
| SONET46-037 | SONET46 | H3 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 7 — Non-Claim Cryptography Assessment > Commitment & Range Proof Modules | 1 | D12 | FUSED |
| SONET46-038 | SONET46 | H3 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 7 — Non-Claim Cryptography Assessment > ECDH / Stealth Address Module | 1 | D12 | FUSED |
| SONET46-039 | SONET46 | H3 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 7 — Non-Claim Cryptography Assessment > Types, Secret, Hidden Modules | 1 | D12 | FUSED |
| SONET46-040 | SONET46 | H3 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Phase 7 — Non-Claim Cryptography Assessment > Validation Module | 1 | D12 | FUSED |
| SONET46-041 | SONET46 | H2 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Consolidated Finding Table | 1 | D17 | FUSED |
| SONET46-042 | SONET46 | H2 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Test Plan Additions | 1 | D15 | FUSED |
| SONET46-043 | SONET46 | H2 | 🔐 Z00Z Crypto Audit — Deep Review of z00z_crypto Crate > Final Decision | 1 | D16 | FUSED |

## Provision Coverage Matrix

Format:

`Provision ID | Source File | Source Section ID | Source Heading Path | Provision Summary | Destination Section ID | Coverage Status | Notes`

`PROV-001 | GPT54 | GPT54-ROOT | (document root) | Front matter records author, AI-assisted note, date, and post metadata for the GPT-5.4 source artifact. | D17 | FUSED | Preserved as source metadata, not treated as a new crypto finding.`
`PROV-002 | GLM5 | GLM5-001 | 🔐 Cryptographic Audit Report — z00z_crypto Crate | Source title frames the audit as a cryptographic audit of the z00z_crypto crate. | D01 | FUSED | Retained in fused title and verdict context.`
`PROV-003 | GLM5 | GLM5-002 | Executive Verdict | Overall posture is risky but salvageable with ZkPack as primary risk. | D16 | FUSED | Folded into conditional blocked synthesis.`
`PROV-004 | GLM5 | GLM5-003 | 1. Input Classification | Reviewed crate provides commitments, range proofs, stealth ECDH, AEAD, KDF, hashing, placeholder claim proof, and custom ZkPack. | D02 | FUSED | Scope normalized and preserved.`
`PROV-005 | GLM5 | GLM5-004 | 2. Security Goals | Security goals include confidentiality, binding, unlinkability, zeroization, domain separation, AEAD integrity, replay resistance, and forward secrecy notes. | D03 | FUSED | Preserved as consolidated goals.`
`PROV-006 | GLM5 | GLM5-005 | 3. Threat Model Summary | Threat model includes passive observers, active attackers, malicious provers/verifiers, side-channel risk, and trust boundary to Tari vendor code. | D04 | FUSED | Preserved in fused threat model.`
`PROV-007 | GLM5 | GLM5-008 | S1-1 ZkPack Uses Custom Stream Cipher + MAC Without Formal Security Proof | ZkPack is a hand-rolled authenticated-encryption construction lacking formal proof and should be replaced or formally justified. | D06 | FUSED | One of the highest-confidence shared findings.`
`PROV-008 | GLM5 | GLM5-010 | S2-1 h2scalar_zk fallback | Hash-to-scalar failure must not silently collapse to scalar one. | D07 | FUSED | Shared with multiple sources.`
`PROV-009 | GLM5 | GLM5-011 | S2-2 generate_hedged_r entropy mixing | Hedged-r derivation may be deterministic if caller entropy is weak or predictable. | D11 | FUSED | Preserved as derivation hygiene concern.`
`PROV-010 | GLM5 | GLM5-012 | S2-3 deterministic blinding seed | Deterministic blinding generation is dangerous if the seed is predictable. | D11 | FUSED | Preserved as usage-boundary warning.`
`PROV-011 | GLM5 | GLM5-013 | S2-4 ClaimAuthoritySig placeholder | Claim authority artifact is a hash-derived placeholder rather than a real signature. | D05 | FUSED | Core production blocker when reachable.`
`PROV-012 | GLM5 | GLM5-014 | S2-5 nonce truncation | Pack-nonce derivation truncates a longer hash and should prefer explicit-length derivation. | D11 | FUSED | Retained as lower-severity derivation concern.`
`PROV-013 | GLM5 | GLM5-016 | S3-1 dangerous_clone | Secret cloning expands memory exposure and should be auditable. | D11 | FUSED | Preserved as secret-lifecycle hardening item.`
`PROV-014 | GLM5 | GLM5-017 | S3-2 Hidden drop semantics | Hidden wrapper depends on downstream Zeroize behavior and could use clearer guarantees. | D11 | FUSED | Preserved as defense-in-depth note.`
`PROV-015 | GLM5 | GLM5-018 | S3-3 owner-tag comparison note | Constant-time discipline is selective and should be clearly scoped to secret-derived values. | D11 | FUSED | Preserved as comparison-discipline nuance.`
`PROV-016 | GLM5 | GLM5-019 | S3-4 Poseidon absorption mode | Native and circuit Poseidon2 absorption modes must match to avoid cross-implementation divergence. | D10 | FUSED | Preserved as compatibility ambiguity.`
`PROV-017 | GLM5 | GLM5-021 | S4-1 parallel domain systems | Typed domains and legacy domain constants coexist and require migration discipline. | D10 | FUSED | Preserved as domain-management caveat.`
`PROV-018 | GLM5 | GLM5-022 | S4-2 from_hash fallback | Scalar-from-hash fallback to one should not remain silent. | D07 | FUSED | Reinforces fail-closed requirement.`
`PROV-019 | GLM5 | GLM5-023 | S4-3 simple hash helpers | No-domain-separation helper hashes are too easy to misuse from the public surface. | D11 | FUSED | Preserved as API-surface risk.`
`PROV-020 | GLM5 | GLM5-024 | S4-4 HMAC zero return | HMAC must not return all-zero output on constructor failure. | D07 | FUSED | Shared fail-open concern.`
`PROV-021 | GLM5 | GLM5-026 | 5.1 Domain Separation | Domain separation design is strong and versioned. | D12 | FUSED | Positive overlap across audits.`
`PROV-022 | GLM5 | GLM5-027 | 5.2 Transcript Binding | Transcript binding is broadly adequate in statement hashing and ECDH derivation. | D12 | FUSED | Preserved as positive structural property.`
`PROV-023 | GLM5 | GLM5-028 | 5.3 Nonce Policy | Standard AEAD nonce handling is correct and failure is fail-fast. | D12 | FUSED | Preserved as positive overlap.`
`PROV-024 | GLM5 | GLM5-029 | 5.4 Key Separation | KDF salt and info separation is strong. | D12 | FUSED | Preserved as positive overlap.`
`PROV-025 | GLM5 | GLM5-030 | 5.5 Error Handling | Error strings generally avoid sensitive data, with noted fallback exceptions. | D12 | FUSED | Preserved with caveat already normalized.`
`PROV-026 | GLM5 | GLM5-031 | 5.6 Serialization | Canonical encoding discipline is broadly sound. | D12 | FUSED | Preserved in positive posture.`
`PROV-027 | GLM5 | GLM5-033 | 6.1 Constant-Time Discipline | Constant-time helpers are used in multiple critical locations. | D12 | FUSED | Preserved as positive overlap.`
`PROV-028 | GLM5 | GLM5-034 | 6.2 Secret Lifecycle | Secret wrappers zeroize and redact correctly in core paths. | D12 | FUSED | Preserved as positive overlap.`
`PROV-029 | GLM5 | GLM5-035 | 6.3 Randomness | RNG paths are explicit and failure-aware. | D12 | FUSED | Preserved as positive overlap.`
`PROV-030 | GLM5 | GLM5-036 | 6.4 Input Validation | Point, scalar, size, and overflow validation are comprehensive. | D12 | FUSED | Preserved as positive overlap.`
`PROV-031 | GLM5 | GLM5-037 | 6.5 Library Usage | Standard RustCrypto and Tari primitives are used rather than bespoke low-level cryptography. | D12 | FUSED | Preserved as positive overlap.`
`PROV-032 | GLM5 | GLM5-038 | 7. Open Ambiguities | ZkPack role, Poseidon mode, deterministic blinding use, hedged-r entropy, and claim-signature migration remain unresolved. | D13 | FUSED | Preserved in ambiguity register.`
`PROV-033 | GLM5 | GLM5-040 | P0 Before Production | Immediate priorities are ZkPack replacement/proof, fail-closed scalar derivation, and real claim signatures. | D14 | FUSED | Preserved in remediation priorities.`
`PROV-034 | GLM5 | GLM5-041 | P1 Before Mainnet | Mainnet priorities include view-tag and entropy clarifications plus nonce derivation cleanup. | D14 | FUSED | Preserved in remediation priorities.`
`PROV-035 | GLM5 | GLM5-042 | P2 Maintenance | Lower-priority cleanup includes domain migration and helper-surface reduction. | D14 | FUSED | Preserved in remediation priorities.`
`PROV-036 | GLM5 | GLM5-044 | Required Tests | Wycheproof, fuzzing, negative, and cross-implementation tests are still needed. | D15 | FUSED | Preserved in test plan.`
`PROV-037 | GLM5 | GLM5-045 | Existing Good Test Coverage | There is already meaningful coverage for roundtrip, tamper, uniqueness, and domain checks. | D15 | FUSED | Preserved as baseline coverage note.`
`PROV-038 | GLM5 | GLM5-046 | Confidence Assessment | Confidence is high in standard primitives and lower in ZkPack and unverified assumptions. | D16 | FUSED | Preserved in confidence section.`
`PROV-039 | GLM5 | GLM5-047 | Final Decision | GLM5 blocks production on ZkPack while judging the crate otherwise salvageable. | D16 | FUSED | Preserved as a source verdict in disagreement set.`
`PROV-040 | GLM5 | GLM5-048 | Appendix A File Inventory | GLM5 supplies a file inventory of reviewed modules. | D17 | FUSED | Preserved as source artifact.`
`PROV-041 | GLM5 | GLM5-049 | Appendix B Dependency Graph | GLM5 supplies a dependency graph for the crypto crate. | D17 | FUSED | Preserved as source artifact.`
`PROV-042 | GPT54 | GPT54-002 | Executive Verdict | GPT-5.4 judges the crate fundamentally broken and blocked for production. | D16 | FUSED | Preserved as one extreme source verdict.`
`PROV-043 | GPT54 | GPT54-003 | Scope | GPT-5.4 enumerates the same 23-file implementation surface and vendor exclusion. | D02 | FUSED | Preserved in normalized scope.`
`PROV-044 | GPT54 | GPT54-004 | Security Goals Extracted From Code | Goals include confidentiality, integrity, stealth privacy, canonical encoding, and secret hygiene. | D03 | FUSED | Preserved in security-goal synthesis.`
`PROV-045 | GPT54 | GPT54-005 | Threat Model Summary | GPT-5.4 frames downstream callers as trusting the public facade as production-safe. | D04 | FUSED | Important for fused boundary assessment.`
`PROV-046 | GPT54 | GPT54-006 | Critical And High Findings | GPT-5.4 groups claim-proof forgery, authority forgery, and custom-ZkPack risk into one high-severity table. | D17 | FUSED | Preserved as a source-specific severity rollup; substantive issues are normalized into D05 and D06.`
`PROV-047 | GPT54 | GPT54-007 | Medium And Low Findings | GPT-5.4 groups fail-open scalar derivation, placeholder exports, helper misuse, and dual pack surfaces into one summary table. | D17 | FUSED | Preserved as source rollup while underlying topics are normalized elsewhere.`
`PROV-048 | GPT54 | GPT54-008 | Supporting Evidence | GPT-5.4 includes direct code-path evidence tying placeholder artifacts to downstream consumers. | D17 | FUSED | Preserved as source-specific evidence summary.`
`PROV-049 | GPT54 | GPT54-009 | Open Ambiguities | GPT-5.4 highlights production reachability of claims, ZkPack purpose, and missing authority-key ownership model. | D13 | FUSED | Preserved in ambiguity register.`
`PROV-050 | GPT54 | GPT54-010 | Concrete Fixes | GPT-5.4 prioritizes gating claim placeholders, adding real signatures, deleting or internalizing ZkPack, and removing fail-open scalar helpers. | D14 | FUSED | Preserved in remediation priorities.`
`PROV-051 | GPT54 | GPT54-011 | Implementation Guidance | GPT-5.4 treats the backend and standard AEAD path as sound while demanding a fail-closed public facade. | D12 | FUSED | Preserved in positive posture with caution.`
`PROV-052 | GPT54 | GPT54-012 | Test Plan | GPT-5.4 emphasizes claim-forgery, replay, fail-open, fuzz, and cross-envelope tests. | D15 | FUSED | Preserved in fused test plan.`
`PROV-053 | GPT54 | GPT54-013 | Confidence | GPT-5.4 has high confidence in the claim-placeholder and fallback findings. | D16 | FUSED | Preserved as source confidence note.`
`PROV-054 | GPT54 | GPT54-014 | Final Decision | GPT-5.4 treats the crate as not production-ready until core blockers are removed. | D16 | FUSED | Preserved as one extreme source verdict.`
`PROV-055 | MIMOV2 | MIMOV2-001 | Title | MIMO-V2 frames the work as a crypto architect audit. | D01 | FUSED | Preserved in title context.`
`PROV-056 | MIMOV2 | MIMOV2-002 | Executive Verdict | MIMO-V2 judges the crate risky but salvageable. | D16 | FUSED | Preserved as midpoint source verdict.`
`PROV-057 | MIMOV2 | MIMOV2-003 | Input Type and Scope | MIMO-V2 confirms the same crate-wide implementation-review scope. | D02 | FUSED | Scope preserved.`
`PROV-058 | MIMOV2 | MIMOV2-004 | Security Goals Assumed | MIMO-V2 emphasizes confidentiality, binding, unlinkability, replay resistance, and domain separation. | D03 | FUSED | Goals preserved.`
`PROV-059 | MIMOV2 | MIMOV2-005 | Threat Model Summary | MIMO-V2 highlights passive observers, malicious provers, replay, side-channel, and DoS attackers. | D04 | FUSED | Threat model preserved.`
`PROV-060 | MIMOV2 | MIMOV2-007 | HMAC zero-return risk | HMAC must not silently return a valid-looking all-zero MAC. | D07 | FUSED | Shared fail-open concern.`
`PROV-061 | MIMOV2 | MIMOV2-008 | h2scalar_zk fallback | Hash-to-scalar helpers must fail closed instead of collapsing to one. | D07 | FUSED | Shared fail-open concern.`
`PROV-062 | MIMOV2 | MIMOV2-009 | from_hash fallback | Scalar-from-hash must fail closed instead of collapsing to one. | D07 | FUSED | Shared fail-open concern.`
`PROV-063 | MIMOV2 | MIMOV2-010 | random infinite loop | Broken RNG must not trigger infinite loops in scalar generation. | D07 | FUSED | Unique liveness concern preserved.`
`PROV-064 | MIMOV2 | MIMOV2-012 | ecdh_stealth explicit validation | Compatibility wrappers should validate points explicitly as defense in depth. | D08 | FUSED | Preserved in stealth-gap section.`
`PROV-065 | MIMOV2 | MIMOV2-013 | ClaimAuthoritySig placeholder | The authority signature is only a deterministic hash placeholder. | D05 | FUSED | Shared claim blocker.`
`PROV-066 | MIMOV2 | MIMOV2-014 | GenesisClaimProof placeholder | The claim proof is deterministic and not a real ZK proof. | D05 | FUSED | Shared claim blocker.`
`PROV-067 | MIMOV2 | MIMOV2-015 | ZkPack serde risk | ZkPackEncrypted should not expose non-canonical serde paths for consensus-critical use. | D06 | FUSED | Preserved as encoding risk.`
`PROV-068 | MIMOV2 | MIMOV2-016 | manual DST framing | DST framing should be frozen by test vector to prevent silent divergence. | D10 | FUSED | Preserved in domain/framing section.`
`PROV-069 | MIMOV2 | MIMOV2-017 | into_inner responsibility transfer | Extracting secret bytes shifts zeroization responsibility to the caller. | D11 | FUSED | Preserved as secret-lifecycle hardening note.`
`PROV-070 | MIMOV2 | MIMOV2-018 | reveal() backend leak | Public reveal methods weaken the abstraction boundary. | D11 | FUSED | Preserved as API-boundary risk.`
`PROV-071 | MIMOV2 | MIMOV2-019 | Debug leak | Debug output for public points can still increase traceability and should be considered deliberately. | D11 | FUSED | Preserved as debug-surface note.`
`PROV-072 | MIMOV2 | MIMOV2-020 | duplicate from_hash warning | The duplicate S4 note reinforces the same fail-open scalar concern. | D07 | FUSED | Preserved as reinforcement, not a new issue.`
`PROV-073 | MIMOV2 | MIMOV2-021 | unknown ZkPack logic location | The actual encryption logic behind ZkPack may live elsewhere and remains ambiguous. | D13 | FUSED | Preserved in ambiguity register.`
`PROV-074 | MIMOV2 | MIMOV2-022 | Open Ambiguities | MIMO-V2 explicitly records missing visibility into sponge location, view tags, nullifier derivation, and transcript binding. | D13 | FUSED | Preserved in ambiguity register.`
`PROV-075 | MIMOV2 | MIMOV2-024 | Fix 1 HMAC error propagation | HMAC APIs should become explicit Result-returning surfaces. | D14 | FUSED | Preserved in remediation priorities.`
`PROV-076 | MIMOV2 | MIMOV2-025 | Fix 2 remove h2scalar fallback | Scalar-derivation helpers must return errors instead of silent constants. | D14 | FUSED | Preserved in remediation priorities.`
`PROV-077 | MIMOV2 | MIMOV2-026 | Fix 3 remove from_hash fallback | Scalar-from-hash should return Result rather than a constant. | D14 | FUSED | Preserved in remediation priorities.`
`PROV-078 | MIMOV2 | MIMOV2-027 | Fix 4 remove random infinite loop | Random scalar generation must fail closed, not hang. | D14 | FUSED | Preserved in remediation priorities.`
`PROV-079 | MIMOV2 | MIMOV2-028 | Fix 5 frozen DST vector | Framing-sensitive code needs a frozen vector. | D14 | FUSED | Preserved in remediation priorities.`
`PROV-080 | MIMOV2 | MIMOV2-030 | What's Done Well | MIMO-V2 highlights domain separation, error hygiene, zeroization, constant-time use, and backend abstraction as strengths. | D12 | FUSED | Preserved as positive overlap.`
`PROV-081 | MIMOV2 | MIMOV2-031 | What Needs Improvement | MIMO-V2 summarizes silent fallbacks, placeholder proofs, serde exposure, and over-broad reveal surfaces as primary weaknesses. | D17 | FUSED | Preserved as source rollup artifact.`
`PROV-082 | MIMOV2 | MIMOV2-033 | Positive Tests Existing | Existing positive tests cover ECDH symmetry, domain uniqueness, commitments, range proofs, Argon2id, HKDF, HMAC, and claim roundtrip. | D15 | FUSED | Preserved as baseline coverage note.`
`PROV-083 | MIMOV2 | MIMOV2-034 | Negative Tests Existing | Existing negative tests cover identity rejection, zero scalar rejection, tampered proofs, empty info, low-entropy IKM without salt, and bad Argon2 parameters. | D15 | FUSED | Preserved as baseline coverage note.`
`PROV-084 | MIMOV2 | MIMOV2-035 | Missing Tests | MIMO-V2 requests explicit regression tests for HMAC, scalar fallbacks, random liveness, DST freeze, ZkPack serde, and explicit wrapper validation. | D15 | FUSED | Preserved in fused test plan.`
`PROV-085 | MIMOV2 | MIMOV2-036 | Wycheproof Integration | Wycheproof-style coverage is recommended for HMAC, signatures, and AEAD-adjacent surfaces. | D15 | FUSED | Preserved in fused test plan.`
`PROV-086 | MIMOV2 | MIMOV2-037 | Confidence Level | MIMO-V2 reports high confidence in domain separation and weaker confidence in ZkPack soundness. | D16 | FUSED | Preserved as source confidence note.`
`PROV-087 | MIMOV2 | MIMOV2-038 | Final Decision | MIMO-V2 blocks production on four open decisions despite calling the crate salvageable. | D16 | FUSED | Preserved in disagreement set.`
`PROV-088 | MIMOV2 | MIMOV2-039 | Appendix A File Inventory | MIMO-V2 includes a structured file inventory. | D17 | FUSED | Preserved as source artifact.`
`PROV-089 | MIMOV2 | MIMOV2-040 | Appendix B Severity Summary | MIMO-V2 includes a severity-count summary table. | D17 | FUSED | Preserved as source artifact.`
`PROV-090 | MINIMAX27 | MINIMAX27-001 | Title | MiniMax frames the document as a crypto audit report. | D01 | FUSED | Preserved in title context.`
`PROV-091 | MINIMAX27 | MINIMAX27-002 | Executive Verdict | MiniMax is the outlier source: it rates the crate safe enough and execution-ready with conditions. | D16 | FUSED | Preserved as key disagreement.`
`PROV-092 | MINIMAX27 | MINIMAX27-003 | Input Type and Scope | MiniMax confirms the same vendor-excluded implementation scope. | D02 | FUSED | Scope preserved.`
`PROV-093 | MINIMAX27 | MINIMAX27-004 | Security Goals Assumed | MiniMax emphasizes confidentiality, range proof soundness, stealth unlinkability, nullifier uniqueness, and domain separation. | D03 | FUSED | Goals preserved.`
`PROV-094 | MINIMAX27 | MINIMAX27-005 | Threat Model Summary | MiniMax lists passive observers, replay, malformed input, and side-channel attackers. | D04 | FUSED | Threat model preserved.`
`PROV-095 | MINIMAX27 | MINIMAX27-006 | Critical and High Findings | MiniMax explicitly reports no S0 or S1 findings. | D16 | FUSED | Preserved as source disagreement.`
`PROV-096 | MINIMAX27 | MINIMAX27-008 | S2 Medium | MiniMax focuses its medium finding on opaque envelope version mismatch handling. | D06 | FUSED | Preserved as envelope-diagnostics issue.`
`PROV-097 | MINIMAX27 | MINIMAX27-009 | S3 Low | MiniMax groups HMAC DST separation, Argon2 parameter framing, generic ECDH wrapper errors, from_hash fallback, identity-compressed clarity, Poseidon parameter exposure, and panic-message hygiene into one low-severity cluster. | D17 | FUSED | Preserved as source rollup artifact.`
`PROV-098 | MINIMAX27 | MINIMAX27-010 | Open Ambiguities | MiniMax records unresolved ZkPack length/context, proof-invalid semantics, and claim-proof replacement timing. | D13 | FUSED | Preserved in ambiguity register.`
`PROV-099 | MINIMAX27 | MINIMAX27-012 | Fix 1 AeadError version mismatch | Version mismatch should be explicit and typed. | D14 | FUSED | Preserved in remediation priorities.`
`PROV-100 | MINIMAX27 | MINIMAX27-013 | Fix 2 HMAC DST separation | HMAC-specific framing should be more explicit. | D14 | FUSED | Preserved in remediation priorities.`
`PROV-101 | MINIMAX27 | MINIMAX27-014 | Fix 3 Argon2 documentation | Moderate Argon2 settings need clearer documentation about intended use. | D14 | FUSED | Preserved in remediation priorities.`
`PROV-102 | MINIMAX27 | MINIMAX27-015 | Fix 4 from_hash panic or explicit propagation | The impossible scalar-reduction fallback should not silently return one. | D14 | FUSED | Preserved in remediation priorities.`
`PROV-103 | MINIMAX27 | MINIMAX27-016 | Fix 5 identity comment | Identity-compressed helper should document canonicalization assumptions. | D14 | FUSED | Preserved in remediation priorities.`
`PROV-104 | MINIMAX27 | MINIMAX27-018 | What Is Done Well | MiniMax strongly endorses domain separation, constant-time handling, identity rejection, zero-scalar rejection, lazy initialization, Poseidon2 use, error hygiene, and HKDF separation. | D12 | FUSED | Preserved as positive overlap and source outlier context.`
`PROV-105 | MINIMAX27 | MINIMAX27-019 | Test Plan | MiniMax requests property, negative, fuzz, and misuse tests across commitments, scalars, points, proofs, and envelopes. | D15 | FUSED | Preserved in fused test plan.`
`PROV-106 | MINIMAX27 | MINIMAX27-020 | Positive Tests Needed | Additional positive-property coverage is requested for commitments, randomness, domain collisions, HMAC vectors, Poseidon, decompression, and DH uniqueness. | D15 | FUSED | Preserved in fused test plan.`
`PROV-107 | MINIMAX27 | MINIMAX27-021 | Negative Tests Needed | Additional negative tests are requested for identity points, non-canonical encodings, zero scalars, bad sizes, and version mismatches. | D15 | FUSED | Preserved in fused test plan.`
`PROV-108 | MINIMAX27 | MINIMAX27-022 | Misuse / Adversarial Cases | MiniMax asks for fuzzing and timing-oriented misuse cases. | D15 | FUSED | Preserved in fused test plan.`
`PROV-109 | MINIMAX27 | MINIMAX27-023 | Wycheproof / Standards Vectors | MiniMax recommends Wycheproof or equivalent standards-vector validation. | D15 | FUSED | Preserved in fused test plan.`
`PROV-110 | MINIMAX27 | MINIMAX27-024 | Confidence Levels | MiniMax reports high confidence in domain separation and identity rejection, medium elsewhere. | D16 | FUSED | Preserved in source confidence comparison.`
`PROV-111 | MINIMAX27 | MINIMAX27-025 | Final Decision | MiniMax deems the crate execution-ready if lower-severity items are addressed before later phases. | D16 | FUSED | Preserved as core verdict conflict.`
`PROV-112 | SONET46 | SONET46-001 | Title | Sonnet 4.6 frames the document as a deep review of the crypto crate. | D01 | FUSED | Preserved in title context.`
`PROV-113 | SONET46 | SONET46-002 | Executive Verdict | Sonnet blocks production primarily on the claim subsystem, with non-claim crypto judged structurally sound. | D16 | FUSED | Preserved as a strong blocked verdict.`
`PROV-114 | SONET46 | SONET46-003 | Phase 0 Input Classification | Sonnet confirms the same implementation surface and primitive families. | D02 | FUSED | Scope preserved.`
`PROV-115 | SONET46 | SONET46-005 | Security Goals | Sonnet details amount hiding, range validity, stealth addressing, AEAD, claim integrity, and secret lifecycle. | D03 | FUSED | Goals preserved.`
`PROV-116 | SONET46 | SONET46-006 | Adversary Model | Sonnet emphasizes network adversaries, malicious nodes, forgers, scanners, and DoS attackers. | D04 | FUSED | Threat model preserved.`
`PROV-117 | SONET46 | SONET46-008 | F-01 claim proof placeholder | Current genesis claim proof is a deterministic hash, not a proof-of-knowledge. | D05 | FUSED | Shared claim blocker.`
`PROV-118 | SONET46 | SONET46-009 | F-02 authority signature placeholder | Current authority artifact is a deterministic hash, not a signature. | D05 | FUSED | Shared claim blocker.`
`PROV-119 | SONET46 | SONET46-010 | F-03 non-constant-time claim comparison | Claim verifier equality should be constant-time before any real signature/proof path ships. | D05 | FUSED | Preserved as secondary claim issue.`
`PROV-120 | SONET46 | SONET46-011 | F-04 no view tag | Wallet scanning lacks the short-tag optimization expected for stealth addresses at scale. | D08 | FUSED | Key stealth-gap finding.`
`PROV-121 | SONET46 | SONET46-013 | F-05 range proof context gap | Range proofs are not obviously bound to asset or chain context inside the proof object. | D09 | FUSED | Preserved as medium-confidence context gap.`
`PROV-122 | SONET46 | SONET46-014 | F-06 separate AEAD tag storage | ZkPackEncrypted splits tag and ciphertext in a way that diverges from the standard envelope model. | D06 | FUSED | Preserved as ZkPack boundary issue.`
`PROV-123 | SONET46 | SONET46-015 | F-07 HMAC zero return | HMAC should not silently produce all-zero output on constructor failure. | D07 | FUSED | Shared fail-open concern.`
`PROV-124 | SONET46 | SONET46-016 | F-08 from_hash fallback | from_hash should not silently fall back to scalar one. | D07 | FUSED | Shared fail-open concern.`
`PROV-125 | SONET46 | SONET46-018 | F-09 ct_eq returns bool | Constant-time APIs should prefer returning Choice rather than plain bools. | D11 | FUSED | Preserved as API-hardening note.`
`PROV-126 | SONET46 | SONET46-019 | F-10 short blind label | The blinding-generator label should be better namespaced. | D10 | FUSED | Preserved as domain-framing caveat.`
`PROV-127 | SONET46 | SONET46-020 | F-11 two hash domain systems | The coexistence of two domain systems is a maintenance hazard. | D10 | FUSED | Shared domain-management caveat.`
`PROV-128 | SONET46 | SONET46-021 | F-12 owner_bind_checked hardcoded | One report field overstates what is actually verified. | D11 | FUSED | Preserved as report/API hygiene note.`
`PROV-129 | SONET46 | SONET46-022 | F-13 heap allocation in nonce derivation | Small hot-path allocations are unnecessary and should be simplified. | D11 | FUSED | Preserved as low-severity engineering note.`
`PROV-130 | SONET46 | SONET46-023 | F-14 mixed case conventions | Mixed domain-string conventions need stronger governance. | D10 | FUSED | Preserved as domain-management note.`
`PROV-131 | SONET46 | SONET46-024 | Open Ambiguities | Sonnet records unresolved questions about abstraction boundaries, ZkPack structure, unused tag domains, wrapper reachability, and entropy sources. | D13 | FUSED | Preserved in ambiguity register.`
`PROV-132 | SONET46 | SONET46-027 | P1-A real claim proof | Claim placeholder must be replaced with a witness-bound proof mechanism. | D14 | FUSED | Preserved in remediation priorities.`
`PROV-133 | SONET46 | SONET46-028 | P1-B real authority signature | Claim authority placeholder must be replaced with a real asymmetric signature. | D14 | FUSED | Preserved in remediation priorities.`
`PROV-134 | SONET46 | SONET46-030 | P2-A fix constant-time comparison | Claim verification equality must become constant-time. | D14 | FUSED | Preserved in remediation priorities.`
`PROV-135 | SONET46 | SONET46-031 | P2-B implement view tags | View tags should be wired into stealth scanning and leaf associated data. | D14 | FUSED | Preserved in remediation priorities.`
`PROV-136 | SONET46 | SONET46-032 | Priority 3 S2 fixes | Remaining ZkPack, HMAC, and scalar fallback issues should be cleaned up after P1/P2. | D14 | FUSED | Preserved in remediation priorities.`
`PROV-137 | SONET46 | SONET46-033 | Non-Claim Cryptography Assessment | Sonnet explicitly separates claim blockers from a stronger non-claim crypto surface. | D12 | FUSED | Critical to the fused conditional verdict.`
`PROV-138 | SONET46 | SONET46-034 | AEAD module assessment | Standard AEAD path is production-ready in structure. | D12 | FUSED | Preserved as positive overlap.`
`PROV-139 | SONET46 | SONET46-035 | KDF module assessment | KDF path is structurally strong with some cleanup needs. | D12 | FUSED | Preserved as positive overlap.`
`PROV-140 | SONET46 | SONET46-036 | Hash module assessment | Hash module is strong overall with a few policy gaps. | D12 | FUSED | Preserved as positive overlap.`
`PROV-141 | SONET46 | SONET46-037 | Commitment and range proof assessment | Commitment and proof integration are structurally sound aside from context-binding caveats. | D12 | FUSED | Preserved as positive overlap.`
`PROV-142 | SONET46 | SONET46-038 | ECDH and stealth assessment | ECDH base is strong but still misses DLEQ-style proofing and view tags. | D12 | FUSED | Preserved as positive overlap with caveat.`
`PROV-143 | SONET46 | SONET46-039 | Types, secret, hidden assessment | Secret handling and redaction are structurally solid. | D12 | FUSED | Preserved as positive overlap.`
`PROV-144 | SONET46 | SONET46-040 | Validation assessment | Validation logic is strong for identity, canonicality, and rejection paths. | D12 | FUSED | Preserved as positive overlap.`
`PROV-145 | SONET46 | SONET46-041 | Consolidated Finding Table | Sonnet includes a source-local consolidated finding matrix and priority ordering. | D17 | FUSED | Preserved as source artifact.`
`PROV-146 | SONET46 | SONET46-042 | Test Plan Additions | Sonnet asks for claim, view-tag, ZkPack, blinding, and HMAC regression tests. | D15 | FUSED | Preserved in fused test plan.`
`PROV-147 | SONET46 | SONET46-043 | Final Decision | Sonnet blocks the claim subsystem and permits only non-claim crypto as structurally sound. | D16 | FUSED | Preserved as strong blocked verdict.`

## Deduplication Decisions

| Decision ID | Duplicate Source Provision IDs | Kept In Destination | Removal Rationale | Why No Meaning Was Lost |
| --- | --- | --- | --- | --- |
| `DD-01` | `PROV-011`, `PROV-045`, `PROV-065`, `PROV-118` | `D05` | All four provisions describe the current claim authority artifact as a hash placeholder rather than a real signature. | The fused section retains the strongest formulation and the production-reachability caveat. |
| `DD-02` | `PROV-045`, `PROV-066`, `PROV-117` | `D05` | All three provisions describe the current claim proof artifact as a deterministic hash placeholder rather than a witness-bound proof. | The fused section preserves both the generic placeholder claim and the stronger forgery consequence. |
| `DD-03` | `PROV-007`, `PROV-046`, `PROV-067`, `PROV-122` | `D06` | All four provisions are different views of the same `ZkPack` risk cluster: custom construction, encoding exposure, and boundary handling. | `D06` keeps construction, encoding, tag handling, and version concerns together without duplicating prose. |
| `DD-04` | `PROV-008`, `PROV-018`, `PROV-061`, `PROV-062`, `PROV-123`, `PROV-124` | `D07` | These all describe fail-open derivation or MAC behavior collapsing to known constants. | `D07` keeps the shared fail-closed principle and preserves each concrete example. |
| `DD-05` | `PROV-064`, `PROV-120` | `D08` | Both provisions describe stealth-flow incompleteness: explicit wrapper validation and missing view tags. | `D08` preserves both the base ECDH strength and the missing fast-filter layer. |
| `DD-06` | `PROV-021`, `PROV-024`, `PROV-080`, `PROV-104`, `PROV-138`, `PROV-139`, `PROV-140`, `PROV-141`, `PROV-143`, `PROV-144` | `D12` | These positive provisions overlap strongly on domain separation, AEAD/KDF soundness, validation, and secret hygiene. | `D12` keeps all positive themes once, with the strongest wording preserved. |
| `DD-07` | `PROV-016`, `PROV-068`, `PROV-127` | `D10` | These provisions all warn that hash-framing or domain-separation drift can silently break compatibility. | `D10` preserves both the strength of current discipline and the maintenance hazard. |
| `DD-08` | `PROV-033`, `PROV-050`, `PROV-075`, `PROV-076`, `PROV-077`, `PROV-078`, `PROV-099`, `PROV-100`, `PROV-101`, `PROV-102`, `PROV-103`, `PROV-132`, `PROV-133`, `PROV-134`, `PROV-135`, `PROV-136` | `D14` | The sources repeatedly restate remediation steps with overlapping priorities. | `D14` keeps one consolidated remediation sequence without dropping any major required action. |
| `DD-09` | `PROV-036`, `PROV-052`, `PROV-084`, `PROV-085`, `PROV-105`, `PROV-106`, `PROV-107`, `PROV-108`, `PROV-109`, `PROV-146` | `D15` | The audits repeatedly request negative, fuzz, vector, replay, and fail-open regression coverage. | `D15` preserves all recurring testing asks as one canonical validation plan. |
| `DD-10` | `PROV-003`, `PROV-042`, `PROV-056`, `PROV-091`, `PROV-113` | `D16` | The source verdicts overlap in structure but diverge in severity. | `D16` retains the disagreement explicitly instead of flattening it into false consensus. |

## Conflict Register

| Conflict ID | Topic | Source Provision IDs | Conflict Description | Why Automatic Fusion Was Unsafe | Required Human Resolution |
| --- | --- | --- | --- | --- | --- |
| `CF-01` | Claim placeholder severity | `PROV-011`, `PROV-045`, `PROV-065`, `PROV-066`, `PROV-091`, `PROV-117`, `PROV-118` | Some audits treat claim placeholders as S0 production breakers; others treat them as lower-severity placeholders assuming non-production scope. | Severity depends on deployment reachability, which is not resolved inside the source set. | Confirm whether the claim subsystem is production-reachable and publicly trusted. |
| `CF-02` | Final release verdict | `PROV-003`, `PROV-042`, `PROV-056`, `PROV-091`, `PROV-113` | Final verdicts range from `execution-ready with conditions` to `fundamentally broken` and `blocked`. | The verdict depends on how unresolved placeholder and ZkPack paths are scoped. | Decide whether production policy treats those paths as active, gated, or experimental. |
| `CF-03` | Fail-open fallback severity | `PROV-008`, `PROV-018`, `PROV-060`, `PROV-061`, `PROV-062`, `PROV-123`, `PROV-124` | The same fallback behavior is graded from S1 to S3/S4 depending on whether reviewers treat the error path as reachable. | Reachability and caller behavior are not fully proven in the source audits. | Validate real call paths and decide whether to treat unreachable fallbacks as bugs or blockers. |
| `CF-04` | ZkPack risk framing | `PROV-007`, `PROV-046`, `PROV-067`, `PROV-096`, `PROV-122` | Sources disagree whether ZkPack is primarily a custom-AEAD blocker, an encoding/version issue, or an experimental side path. | The source set does not resolve whether ZkPack is required, experimental, or production-critical. | Decide whether to replace, gate, or formally justify ZkPack. |

## Deletion-Safety Verdict

Verification summary:

- Source H1-H4 section coverage: `100% PASS`
- Source-section inventory coverage including root section: `100% PASS`
- Extracted provision coverage: `100% PASS`
- Unresolved duplicate propositions in `FUSION.md`: `0 PASS`
- Documented semantic conflicts: `4 PASS`
- Canonical merged document exists: `PASS`
- `/doublecheck` self-consistency review: `PASS`

Deletion-safety decision: `PASS`

Rationale:

- Every source section is represented in the inventory.
- Every extracted provision is mapped to a destination section.
- Duplicates were merged by topic and logged explicitly.
- Disagreements were not flattened; they remain in the conflict register and D16.
- Source-specific metadata, summary tables, dependency graphs, and severity rollups were
  preserved in D17 rather than discarded.

## Doublecheck Review

| Claim ID | Verification Claim | Doublecheck Rating | Disposition | Follow-Up Action |
| --- | --- | --- | --- | --- |
| `C1` | Source H1-H4 sections = `171` | `PASS` | Accepted | None |
| `C2` | Source-section inventory total = `172` | `PASS` | Accepted | None |
| `C3` | Extracted provisions = `147` | `PASS` | Accepted | None |
| `C4` | Merged topic sections = `17` | `PASS` | Accepted | None |
| `C5` | Section coverage = `100%` | `PASS` | Accepted as gate/result value | None |
| `C6` | Provision coverage = `100%` | `PASS` | Accepted as gate/result value | None |
| `C7` | Unresolved duplicates = `0` | `PASS` | Accepted as gate/result value | None |
| `C8` | Documented semantic conflicts = `4` | `PASS` | Accepted | None |
| `C9` | Canonical fused decision is internally consistent with the evidence bundle | `PASS` | Accepted | None |
| `C10` | Documented conflict statement is internally consistent with the evidence bundle | `PASS` | Accepted | None |

Doublecheck note:

- The first `Doublecheck` attempt was unusable because the agent lacked file access and did
  not receive enough inline evidence.
- The final `Doublecheck` pass was rerun with an explicit evidence bundle containing section
  counts, provision counts, source verdicts, and conflict framing.
- No final `FABRICATION RISK`, `DISPUTED`, or `UNVERIFIED` rating remains unresolved for the
  accepted verification-summary claims.
