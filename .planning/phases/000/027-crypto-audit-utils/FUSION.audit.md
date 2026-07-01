---
post_title: "027 Utils Audit Fusion Audit"
author1: "GitHub Copilot"
post_slug: "027-utils-audit-fusion-audit"
microsoft_alias: "copilot"
featured_image: "none"
categories:
  - "engineering"
tags:
  - "crypto"
  - "audit"
  - "rust"
  - "z00z_utils"
  - "fusion-audit"
ai_note: "AI-assisted coverage and deduplication audit for the fused z00z_utils audit document."
summary: "Section inventory, provision coverage, deduplication, conflicts, and verification summary for the z00z_utils audit fusion artifact."
post_date: "2026-03-26"
---

<!-- markdownlint-disable MD041 -->

## Audit Metadata

📌 Fusion target: `.planning/phases/027-crypto-audit-utils/FUSION.md`

📌 Audit target: `.planning/phases/027-crypto-audit-utils/FUSION.audit.md`

📌 Inventory basis: all H1-H4 headings from the five source markdown files.

📌 Section coverage basis: exact heading inventory from a regex pass over the
five source files.

📌 Provision coverage basis: normalized substantive propositions extracted from
the five source reports and merged by topic.

## Source Files

| Source File | Auditor Label | H1-H4 Sections | Notes |
| --- | --- | ---: | --- |
| `.planning/phases/027-crypto-audit-utils/utils-audit-glm5.md` | GLM-5 | 34 | Broadest positive-control and composition review coverage |
| `.planning/phases/027-crypto-audit-utils/utils-audit-gpt546.md` | GPT-5.4.6 | 26 | Strongest `LockedBytes` soundness analysis and config fail-open analysis |
| `.planning/phases/027-crypto-audit-utils/utils-audit-m27.md` | MiniMax M2.7 | 25 | Most optimistic verdict, strong non-Unix durability notes |
| `.planning/phases/027-crypto-audit-utils/utils-audit-mimov2.md` | MiMo-V2-Pro | 28 | Deterministic RNG guard and abstraction-boundary emphasis |
| `.planning/phases/027-crypto-audit-utils/utils-audit-sonet46.md` | Sonnet 4.6 | 29 | Detailed finding map for time fallback, file permissions, logger hardening |

📌 Total inventoried H1-H4 sections: `142`.

## Source Section Inventory

📌 Inventory rows below account for every H1-H4 heading found in the five source
files.

| Section ID | Source File | Heading Level | Heading Path | Provision Count | Destination Section IDs | Status |
| --- | --- | --- | --- | ---: | --- | --- |
| GLM-001 | utils-audit-glm5.md | H1 | 027 — Cryptographic Audit: `z00z_utils` Crate | 1 | UF-01,UF-02 | FUSED |
| GLM-002 | utils-audit-glm5.md | H2 | Executive Verdict | 1 | UF-01,UF-08 | FUSED |
| GLM-003 | utils-audit-glm5.md | H2 | 1. Input Classification | 1 | UF-02 | FUSED |
| GLM-004 | utils-audit-glm5.md | H2 | 2. Scope and Threat Model | 1 | UF-02 | FUSED |
| GLM-005 | utils-audit-glm5.md | H3 | 2. Scope and Threat Model > Security Goals | 1 | UF-02 | FUSED |
| GLM-006 | utils-audit-glm5.md | H3 | 2. Scope and Threat Model > Assets Protected | 1 | UF-02 | FUSED |
| GLM-007 | utils-audit-glm5.md | H3 | 2. Scope and Threat Model > Adversaries Considered | 1 | UF-02 | FUSED |
| GLM-008 | utils-audit-glm5.md | H3 | 2. Scope and Threat Model > Trust Boundaries | 1 | UF-02 | FUSED |
| GLM-009 | utils-audit-glm5.md | H2 | 3. Findings | 1 | UF-03 | FUSED |
| GLM-010 | utils-audit-glm5.md | H3 | 3. Findings > S0 — CRITICAL | 1 | UF-01 | FUSED |
| GLM-011 | utils-audit-glm5.md | H3 | 3. Findings > S1 — HIGH | 1 | UF-01 | FUSED |
| GLM-012 | utils-audit-glm5.md | H3 | 3. Findings > S2 — MEDIUM | 1 | UF-03.1,UF-03.2,UF-03.6 | FUSED |
| GLM-013 | utils-audit-glm5.md | H3 | 3. Findings > S3 — LOW | 1 | UF-03.3,UF-03.5 | FUSED |
| GLM-014 | utils-audit-glm5.md | H3 | 3. Findings > S4 — INFO | 1 | UF-03.8 | FUSED |
| GLM-015 | utils-audit-glm5.md | H2 | 4. Composition Review | 1 | UF-04 | FUSED |
| GLM-016 | utils-audit-glm5.md | H3 | 4. Composition Review > RNG → Crypto Pipeline | 1 | UF-04,UF-03.6 | FUSED |
| GLM-017 | utils-audit-glm5.md | H3 | 4. Composition Review > Serialization → Canonical Encoding | 1 | UF-04 | FUSED |
| GLM-018 | utils-audit-glm5.md | H3 | 4. Composition Review > File I/O → Atomicity | 1 | UF-04,UF-03.4 | FUSED |
| GLM-019 | utils-audit-glm5.md | H3 | 4. Composition Review > OS Hardening → Secret Lifecycle | 1 | UF-04,UF-03.1 | FUSED |
| GLM-020 | utils-audit-glm5.md | H2 | 5. Dependency Audit | 1 | UF-03.8,UF-08 | FUSED |
| GLM-021 | utils-audit-glm5.md | H2 | 6. Test Coverage Assessment | 1 | UF-07 | FUSED |
| GLM-022 | utils-audit-glm5.md | H3 | 6. Test Coverage Assessment > Positive Tests (Verified Present) | 1 | UF-07 | FUSED |
| GLM-023 | utils-audit-glm5.md | H3 | 6. Test Coverage Assessment > Missing Tests (Recommended) | 1 | UF-07 | FUSED |
| GLM-024 | utils-audit-glm5.md | H3 | 6. Test Coverage Assessment > Fuzzing Targets (Recommended) | 1 | UF-07 | FUSED |
| GLM-025 | utils-audit-glm5.md | H2 | 7. Confidence Assessment | 1 | UF-08 | FUSED |
| GLM-026 | utils-audit-glm5.md | H2 | 8. Open Ambiguities | 1 | UF-05 | FUSED |
| GLM-027 | utils-audit-glm5.md | H2 | 9. Concrete Fixes | 1 | UF-06 | FUSED |
| GLM-028 | utils-audit-glm5.md | H3 | 9. Concrete Fixes > Fix S2-1: Document `with_u64_seed` as test-only | 1 | UF-03.6,UF-06 | FUSED |
| GLM-029 | utils-audit-glm5.md | H3 | 9. Concrete Fixes > Fix S2-3: Add lifetime to `LockedBytes` | 1 | UF-03.1,UF-06 | FUSED |
| GLM-030 | utils-audit-glm5.md | H3 | 9. Concrete Fixes > Fix S2-4: Add bounded YAML deserialization | 1 | UF-03.2,UF-06 | FUSED |
| GLM-031 | utils-audit-glm5.md | H3 | 9. Concrete Fixes > Fix S2-5: Use bounded I/O in `YamlConfig::from_file` | 1 | UF-03.2,UF-06 | FUSED |
| GLM-032 | utils-audit-glm5.md | H2 | 10. Final Decision | 1 | UF-01,UF-08 | FUSED |
| GLM-033 | utils-audit-glm5.md | H2 | Appendix A: Files Reviewed | 1 | UF-02,UF-08 | FUSED |
| GLM-034 | utils-audit-glm5.md | H2 | Appendix B: Severity Distribution | 1 | UF-08 | FUSED |
| GPT-001 | utils-audit-gpt546.md | H1 | Crypto Audit: z00z_utils | 1 | UF-01,UF-02 | FUSED |
| GPT-002 | utils-audit-gpt546.md | H2 | Executive Verdict | 1 | UF-01,UF-08 | FUSED |
| GPT-003 | utils-audit-gpt546.md | H2 | Input Type And Scope | 1 | UF-02 | FUSED |
| GPT-004 | utils-audit-gpt546.md | H2 | Security Goals Extracted From Code | 1 | UF-02 | FUSED |
| GPT-005 | utils-audit-gpt546.md | H2 | Threat Model Summary | 1 | UF-02 | FUSED |
| GPT-006 | utils-audit-gpt546.md | H2 | Critical And High Findings | 1 | UF-03.1 | FUSED |
| GPT-007 | utils-audit-gpt546.md | H3 | Critical And High Findings > S1-01: `LockedBytes` Is Lifetime-Unsound And Can Cause UB From Safe Callers | 1 | UF-03.1 | FUSED |
| GPT-008 | utils-audit-gpt546.md | H2 | Medium Findings | 1 | UF-03.2,UF-03.5 | FUSED |
| GPT-009 | utils-audit-gpt546.md | H3 | Medium Findings > S2-01: `LayeredConfig::new()` Fails Open On YAML Errors | 1 | UF-03.2 | FUSED |
| GPT-010 | utils-audit-gpt546.md | H3 | Medium Findings > S2-02: `YamlConfig::from_file()` Bypasses The Crate’s Bounded I/O Boundary | 1 | UF-03.2 | FUSED |
| GPT-011 | utils-audit-gpt546.md | H3 | Medium Findings > S2-03: Log File Symlink Protection Is Partial, Not Race-Hardened | 1 | UF-03.5 | FUSED |
| GPT-012 | utils-audit-gpt546.md | H2 | Low And Informational Findings | 1 | UF-03.3,UF-03.1,UF-03.5 | FUSED |
| GPT-013 | utils-audit-gpt546.md | H3 | Low And Informational Findings > S3-01: Infallible `TimeProvider` Helpers Collapse Clock Errors To Zero | 1 | UF-03.3 | FUSED |
| GPT-014 | utils-audit-gpt546.md | H3 | Low And Informational Findings > S3-02: `LockedBytes` Debug Output Leaks Raw Addresses | 1 | UF-03.1 | FUSED |
| GPT-015 | utils-audit-gpt546.md | H3 | Low And Informational Findings > S4-01: Structured Logging Falls Back To A Generic Event On Serialization Failure | 1 | UF-03.5 | FUSED |
| GPT-016 | utils-audit-gpt546.md | H2 | Positive Security Properties | 1 | UF-04 | FUSED |
| GPT-017 | utils-audit-gpt546.md | H2 | Open Ambiguities | 1 | UF-05 | FUSED |
| GPT-018 | utils-audit-gpt546.md | H2 | Concrete Fixes | 1 | UF-06 | FUSED |
| GPT-019 | utils-audit-gpt546.md | H3 | Concrete Fixes > Fix 1: Make Memory Lock Guard Lifetime-Safe | 1 | UF-03.1,UF-06 | FUSED |
| GPT-020 | utils-audit-gpt546.md | H3 | Concrete Fixes > Fix 2: Make Layered Config Loading Fail Closed Except For Missing Files | 1 | UF-03.2,UF-06 | FUSED |
| GPT-021 | utils-audit-gpt546.md | H3 | Concrete Fixes > Fix 3: Route YAML File Reads Through Bounded I/O | 1 | UF-03.2,UF-06 | FUSED |
| GPT-022 | utils-audit-gpt546.md | H3 | Concrete Fixes > Fix 4: Harden File Logger Path Opening | 1 | UF-03.5,UF-06 | FUSED |
| GPT-023 | utils-audit-gpt546.md | H3 | Concrete Fixes > Fix 5: Demote Or Rename Zero-On-Error Time Helpers | 1 | UF-03.3,UF-06 | FUSED |
| GPT-024 | utils-audit-gpt546.md | H2 | Test Plan | 1 | UF-07 | FUSED |
| GPT-025 | utils-audit-gpt546.md | H2 | Confidence | 1 | UF-08 | FUSED |
| GPT-026 | utils-audit-gpt546.md | H2 | Final Decision | 1 | UF-01,UF-08 | FUSED |
| M27-001 | utils-audit-m27.md | H1 | 027-Crypto-Audit-Utils: z00z_utils Deep Cryptographic Audit | 1 | UF-01,UF-02 | FUSED |
| M27-002 | utils-audit-m27.md | H2 | Executive Verdict | 1 | UF-01,UF-08 | FUSED |
| M27-003 | utils-audit-m27.md | H2 | 1. Input Classification | 1 | UF-02 | FUSED |
| M27-004 | utils-audit-m27.md | H2 | 2. Security Goals Extracted | 1 | UF-02 | FUSED |
| M27-005 | utils-audit-m27.md | H2 | 3. Critical and High Findings (S0/S1) | 1 | UF-01 | FUSED |
| M27-006 | utils-audit-m27.md | H2 | 4. Medium and Low Findings (S2/S3) | 1 | UF-03.4,UF-03.8 | FUSED |
| M27-007 | utils-audit-m27.md | H3 | 4. Medium and Low Findings (S2/S3) > Finding M1 — `io/fs.rs`: `atomic_write_file_private` lacks fsync on non-Unix paths | 1 | UF-03.4 | FUSED |
| M27-008 | utils-audit-m27.md | H3 | 4. Medium and Low Findings (S2/S3) > Finding M2 — `io/fs.rs`: `atomic_write_file_streaming` — identical non-Unix gap | 1 | UF-03.4 | FUSED |
| M27-009 | utils-audit-m27.md | H3 | 4. Medium and Low Findings (S2/S3) > Finding L1 — `compression.rs`: LZ4 frame magic constant is little-endian only | 1 | UF-03.8 | FUSED |
| M27-010 | utils-audit-m27.md | H3 | 4. Medium and Low Findings (S2/S3) > Finding L2 (RESCINDED) — `os_hardening.rs`: `LockedBytes::Drop` was incorrectly suspected | 1 | UF-05 | FUSED |
| M27-011 | utils-audit-m27.md | H2 | 5. INFO Findings (S4) | 1 | UF-03.8 | FUSED |
| M27-012 | utils-audit-m27.md | H3 | 5. INFO Findings (S4) > INFO-1 — `compression.rs`: `zstd_decompress_bounded` re-checks after `take(max_output)` | 1 | UF-04 | FUSED |
| M27-013 | utils-audit-m27.md | H3 | 5. INFO Findings (S4) > INFO-2 — `rng/mock.rs`: `MockRngProvider` compile-error gate is technically correct but fragile | 1 | UF-03.6,UF-05 | FUSED |
| M27-014 | utils-audit-m27.md | H3 | 5. INFO Findings (S4) > INFO-3 — `codec/yaml.rs`: Multi-document YAML detection | 1 | UF-04 | FUSED |
| M27-015 | utils-audit-m27.md | H3 | 5. INFO Findings (S4) > INFO-4 — `codec/bincode.rs`: Three fixed size limits (1MB/10MB/100MB) | 1 | UF-03.8 | FUSED |
| M27-016 | utils-audit-m27.md | H3 | 5. INFO Findings (S4) > INFO-5 — `time/traits.rs`: `unix_timestamp_micros()` overflow returns `u64::MAX` | 1 | UF-03.3,UF-03.8 | FUSED |
| M27-017 | utils-audit-m27.md | H3 | 5. INFO Findings (S4) > INFO-6 — `io/fs.rs`: `atomic_write_with_context` permission inheritance | 1 | UF-03.4 | FUSED |
| M27-018 | utils-audit-m27.md | H3 | 5. INFO Findings (S4) > INFO-7 — `os_hardening.rs`: `apply_best_effort` requires explicit consumer call | 1 | UF-03.8,UF-05 | FUSED |
| M27-019 | utils-audit-m27.md | H2 | 6. Open Ambiguities | 1 | UF-05 | FUSED |
| M27-020 | utils-audit-m27.md | H2 | 7. Concrete Fixes | 1 | UF-06 | FUSED |
| M27-021 | utils-audit-m27.md | H3 | 7. Concrete Fixes > Fix for M1 + M2 (non-Unix fsync) | 1 | UF-03.4,UF-06 | FUSED |
| M27-022 | utils-audit-m27.md | H2 | 8. Implementation Guidance | 1 | UF-04,UF-06 | FUSED |
| M27-023 | utils-audit-m27.md | H2 | 9. Test Plan | 1 | UF-07 | FUSED |
| M27-024 | utils-audit-m27.md | H2 | 10. Confidence Level | 1 | UF-08 | FUSED |
| M27-025 | utils-audit-m27.md | H2 | 11. Final Decision | 1 | UF-01,UF-08 | FUSED |
| MIM-001 | utils-audit-mimov2.md | H1 | Crypto Architect Audit Report — `z00z_utils` Crate | 1 | UF-01,UF-02 | FUSED |
| MIM-002 | utils-audit-mimov2.md | H2 | Executive Verdict | 1 | UF-01,UF-08 | FUSED |
| MIM-003 | utils-audit-mimov2.md | H2 | 1. Input Type & Scope | 1 | UF-02 | FUSED |
| MIM-004 | utils-audit-mimov2.md | H2 | 2. Security Goals | 1 | UF-02 | FUSED |
| MIM-005 | utils-audit-mimov2.md | H2 | 3. Threat Model Summary | 1 | UF-02 | FUSED |
| MIM-006 | utils-audit-mimov2.md | H2 | 4. Critical & High Findings (S0/S1) | 1 | UF-03.6,UF-03.7,UF-05 | FUSED |
| MIM-007 | utils-audit-mimov2.md | H3 | 4. Critical & High Findings (S0/S1) > ~~S1: `LockedBytes::drop` Does Not Call `munlock`~~ — RETRACTED | 1 | UF-05 | FUSED |
| MIM-008 | utils-audit-mimov2.md | H3 | 4. Critical & High Findings (S0/S1) > S1: `DeterministicRngProvider` Has No Compile-Time Production Guard | 1 | UF-03.6,UF-05 | FUSED |
| MIM-009 | utils-audit-mimov2.md | H3 | 4. Critical & High Findings (S0/S1) > S1: `logger/macros.rs` Uses `serde_json` Directly — Violates ONE SOURCE OF TRUTH | 1 | UF-03.7,UF-05 | FUSED |
| MIM-010 | utils-audit-mimov2.md | H2 | 5. Medium & Low Findings (S2/S3/S4) | 1 | UF-03.2,UF-03.5,UF-03.6,UF-03.7,UF-03.8 | FUSED |
| MIM-011 | utils-audit-mimov2.md | H3 | 5. Medium & Low Findings (S2/S3/S4) > S2: `codec/mod.rs` Re-exports `serde_json::{json, Value}` Directly | 1 | UF-03.7 | FUSED |
| MIM-012 | utils-audit-mimov2.md | H3 | 5. Medium & Low Findings (S2/S3/S4) > S2: `YamlConfig::from_file` Uses `std::fs::read_to_string` Directly | 1 | UF-03.2 | FUSED |
| MIM-013 | utils-audit-mimov2.md | H3 | 5. Medium & Low Findings (S2/S3/S4) > S2: `RotatingFileLogger::write_log` Discards Level Information | 1 | UF-03.5 | FUSED |
| MIM-014 | utils-audit-mimov2.md | H3 | 5. Medium & Low Findings (S2/S3/S4) > S3: `BincodeCodec::deserialize_bounded` Only Supports Three Fixed Sizes | 1 | UF-03.8 | FUSED |
| MIM-015 | utils-audit-mimov2.md | H3 | 5. Medium & Low Findings (S2/S3/S4) > S3: `MockRngProvider::rng()` Returns Same Sequence Per Call | 1 | UF-03.6 | FUSED |
| MIM-016 | utils-audit-mimov2.md | H3 | 5. Medium & Low Findings (S2/S3/S4) > S4: `logger/structured.rs` Uses `erased_serde` for Dynamic Serialization | 1 | UF-03.8 | FUSED |
| MIM-017 | utils-audit-mimov2.md | H3 | 5. Medium & Low Findings (S2/S3/S4) > S4: `time/format.rs` Uses `chrono` for Timestamp Formatting | 1 | UF-03.8 | FUSED |
| MIM-018 | utils-audit-mimov2.md | H2 | 6. Open Ambiguities | 1 | UF-05 | FUSED |
| MIM-019 | utils-audit-mimov2.md | H2 | 7. Concrete Fixes (Summary) | 1 | UF-06 | FUSED |
| MIM-020 | utils-audit-mimov2.md | H2 | 8. Implementation Guidance | 1 | UF-04,UF-06 | FUSED |
| MIM-021 | utils-audit-mimov2.md | H3 | 8. Implementation Guidance > What Is Done Well | 1 | UF-04 | FUSED |
| MIM-022 | utils-audit-mimov2.md | H3 | 8. Implementation Guidance > What Needs Improvement | 1 | UF-06 | FUSED |
| MIM-023 | utils-audit-mimov2.md | H2 | 9. Test Plan | 1 | UF-07 | FUSED |
| MIM-024 | utils-audit-mimov2.md | H3 | 9. Test Plan > Positive Tests (Already Present) | 1 | UF-07 | FUSED |
| MIM-025 | utils-audit-mimov2.md | H3 | 9. Test Plan > Missing Tests (Recommended) | 1 | UF-07 | FUSED |
| MIM-026 | utils-audit-mimov2.md | H3 | 9. Test Plan > Adversarial Tests (Recommended) | 1 | UF-07 | FUSED |
| MIM-027 | utils-audit-mimov2.md | H2 | 10. Confidence Level | 1 | UF-08 | FUSED |
| MIM-028 | utils-audit-mimov2.md | H2 | 11. Final Decision | 1 | UF-01,UF-08 | FUSED |
| SON-001 | utils-audit-sonet46.md | H1 | 🔐 Crypto-Security Audit — `z00z_utils` Crate | 1 | UF-01,UF-02 | FUSED |
| SON-002 | utils-audit-sonet46.md | H2 | 📌 Executive Verdict | 1 | UF-01,UF-08 | FUSED |
| SON-003 | utils-audit-sonet46.md | H2 | ⚙️ Scope & Input Classification | 1 | UF-02 | FUSED |
| SON-004 | utils-audit-sonet46.md | H2 | 🎯 Security Goals & Threat Model | 1 | UF-02 | FUSED |
| SON-005 | utils-audit-sonet46.md | H2 | ⚠️ Findings Summary | 1 | UF-03 | FUSED |
| SON-006 | utils-audit-sonet46.md | H2 | 🔑 Detailed Findings | 1 | UF-03 | FUSED |
| SON-007 | utils-audit-sonet46.md | H3 | 🔑 Detailed Findings > ⭐ F-01 · S2 — `unix_timestamp()` Silent 0 on Clock Error | 1 | UF-03.3 | FUSED |
| SON-008 | utils-audit-sonet46.md | H3 | 🔑 Detailed Findings > ⭐ F-02 · S2 — `write_file()` Permission Preservation Silently Discarded | 1 | UF-03.4 | FUSED |
| SON-009 | utils-audit-sonet46.md | H3 | 🔑 Detailed Findings > 💥 F-03 · S3 — `LockedBytes` Exposes Raw Address in fmt output | 1 | UF-03.1 | FUSED |
| SON-010 | utils-audit-sonet46.md | H3 | 🔑 Detailed Findings > 🚨 F-04 · S3 — `sanitize_message` omits ANSI escape sequences | 1 | UF-03.5 | FUSED |
| SON-011 | utils-audit-sonet46.md | H3 | 🔑 Detailed Findings > ⚠️ F-05 · S3 — `EnvConfig` Has No Allowlist | 1 | UF-03.5,UF-05 | FUSED |
| SON-012 | utils-audit-sonet46.md | H3 | 🔑 Detailed Findings > ⚠️ F-06 · S3 — `DeterministicRngProvider` Requires `CryptoRng` Bound | 1 | UF-03.6 | FUSED |
| SON-013 | utils-audit-sonet46.md | H3 | 🔑 Detailed Findings > 🐞 F-07 · S4 — `with_u64_seed` Effective Entropy Limited to 2^64 | 1 | UF-03.6 | FUSED |
| SON-014 | utils-audit-sonet46.md | H3 | 🔑 Detailed Findings > 🐞 F-08 · S4 — MockRngProvider (StdRng) vs DeterministicRngProvider (ChaCha20) Inconsistency | 1 | UF-03.6 | FUSED |
| SON-015 | utils-audit-sonet46.md | H3 | 🔑 Detailed Findings > 🐞 F-09 · S4 — YAML `TrailingBytes` Error Reports `consumed: 0` | 1 | UF-03.8 | FUSED |
| SON-016 | utils-audit-sonet46.md | H3 | 🔑 Detailed Findings > 🐞 F-10 · S4 — Local-Timezone Format Function Mixes Into UTC Log Stream | 1 | UF-03.8 | FUSED |
| SON-017 | utils-audit-sonet46.md | H3 | 🔑 Detailed Findings > 🐞 F-11 · S4 — Structured Logger Silently Substitutes Missing Events | 1 | UF-03.5 | FUSED |
| SON-018 | utils-audit-sonet46.md | H3 | 🔑 Detailed Findings > 🐞 F-12 · S4 — `write_file()` Has No `sync_all`/`fsync` | 1 | UF-03.4 | FUSED |
| SON-019 | utils-audit-sonet46.md | H3 | 🔑 Detailed Findings > 🐞 F-13 · S4 — `munlock` Failure Silently Dropped in `LockedBytes::Drop` | 1 | UF-03.8 | FUSED |
| SON-020 | utils-audit-sonet46.md | H2 | ✅ Positive Controls (What Is Done Well) | 1 | UF-04 | FUSED |
| SON-021 | utils-audit-sonet46.md | H2 | 🔑 Implementation Guidance | 1 | UF-06 | FUSED |
| SON-022 | utils-audit-sonet46.md | H3 | 🔑 Implementation Guidance > Priority 1 (Required — S2 fixes) | 1 | UF-06 | FUSED |
| SON-023 | utils-audit-sonet46.md | H3 | 🔑 Implementation Guidance > Priority 2 (Recommended — S3 fixes) | 1 | UF-06 | FUSED |
| SON-024 | utils-audit-sonet46.md | H3 | 🔑 Implementation Guidance > Priority 3 (Optional — S4 improvements) | 1 | UF-06 | FUSED |
| SON-025 | utils-audit-sonet46.md | H2 | 🧪 Test Plan | 1 | UF-07 | FUSED |
| SON-026 | utils-audit-sonet46.md | H2 | ❓ Open Ambiguities | 1 | UF-05 | FUSED |
| SON-027 | utils-audit-sonet46.md | H2 | 💯 Confidence Assessment | 1 | UF-08 | FUSED |
| SON-028 | utils-audit-sonet46.md | H2 | 📌 Dependency Versions (Cargo.toml Snapshot) | 1 | UF-03.8,UF-08 | FUSED |
| SON-029 | utils-audit-sonet46.md | H2 | 🎯 Conclusion | 1 | UF-01,UF-08 | FUSED |

## Provision Coverage Matrix

📌 Normalized substantive proposition count: `26`.

| Provision ID | Source File | Source Section ID | Source Heading Path | Provision Summary | Destination Section ID | Coverage Status | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| P-001 | utils-audit-gpt546.md | GPT-007 | Critical And High Findings > S1-01 | `LockedBytes` is lifetime-unsound because the guard is not tied to the borrowed slice lifetime. | UF-03.1 | FUSED | Reinforced by GLM-012 and SON-009. |
| P-002 | utils-audit-sonet46.md | SON-009 | Detailed Findings > F-03 | `LockedBytes` debug output should not leak raw memory addresses. | UF-03.1 | FUSED | Reinforced by GPT-014. |
| P-003 | utils-audit-glm5.md | GLM-029 | Concrete Fixes > Fix S2-3 | The correct structural fix for `LockedBytes` is a lifetime-bound guard. | UF-03.1 | FUSED | Reinforced by GPT-019. |
| P-004 | utils-audit-gpt546.md | GPT-009 | Medium Findings > S2-01 | `LayeredConfig::new()` fails open on YAML errors by using `.ok()`. | UF-03.2 | FUSED | Unique but high-confidence. |
| P-005 | utils-audit-gpt546.md | GPT-010 | Medium Findings > S2-02 | `YamlConfig::from_file()` bypasses the bounded I/O boundary. | UF-03.2 | FUSED | Reinforced by GLM-012 and MIM-012. |
| P-006 | utils-audit-glm5.md | GLM-012 | 3. Findings > S2 — MEDIUM | YAML deserialization should have its own bounded path, not just bounded reads. | UF-03.2 | FUSED | Reinforced by GLM-030. |
| P-007 | utils-audit-sonet46.md | SON-007 | Detailed Findings > F-01 | `unix_timestamp*()` lossy zero fallback is dangerous for nonce, expiry, and anti-replay paths. | UF-03.3 | FUSED | Reinforced by GPT-013. |
| P-008 | utils-audit-m27.md | M27-016 | INFO-5 | `unix_timestamp_micros()` overflow sentinel should be documented. | UF-03.3 | FUSED | Lower-priority companion note. |
| P-009 | utils-audit-sonet46.md | SON-008 | Detailed Findings > F-02 | `write_file()` silently discards permission-preservation failures on overwrite. | UF-03.4 | FUSED | Unique but concrete. |
| P-010 | utils-audit-sonet46.md | SON-018 | Detailed Findings > F-12 | `write_file()` lacks `sync_all()` durability parity with the private helper. | UF-03.4 | FUSED | Complements P-009. |
| P-011 | utils-audit-m27.md | M27-007 | Finding M1 | `atomic_write_file_private` has a non-Unix durability gap because the fallback path does not fsync. | UF-03.4 | FUSED | Reinforced by M27-008. |
| P-012 | utils-audit-glm5.md | GLM-013 | 3. Findings > S3 — LOW | `sanitize_message` omits ANSI and broader control-character stripping. | UF-03.5 | FUSED | Reinforced by SON-010. |
| P-013 | utils-audit-mimov2.md | MIM-013 | Medium & Low Findings > S2 | `RotatingFileLogger::write_log` discards the log level. | UF-03.5 | FUSED | Unique but concrete. |
| P-014 | utils-audit-gpt546.md | GPT-011 | Medium Findings > S2-03 | Log symlink protection is partial and not race-hardened across parent directories. | UF-03.5 | FUSED | Unique but preserved as medium-confidence. |
| P-015 | utils-audit-gpt546.md | GPT-015 | Low And Informational Findings > S4-01 | Structured logger serialization fallback is safe but too generic for good audit precision. | UF-03.5 | FUSED | Reinforced by SON-017. |
| P-016 | utils-audit-sonet46.md | SON-011 | Detailed Findings > F-05 | `EnvConfig` has no allowlist or namespace restriction. | UF-03.5 | FUSED | Single-source concern retained as ambiguity-backed hardening. |
| P-017 | utils-audit-mimov2.md | MIM-008 | Critical & High Findings > S1 | `DeterministicRngProvider` lacks a compile-time production guard. | UF-03.6 | CONFLICT | Not corroborated by other reports. |
| P-018 | utils-audit-sonet46.md | SON-012 | Detailed Findings > F-06 | `DeterministicRngProvider` using a `CryptoRng` bound is semantically misleading for deterministic output. | UF-03.6 | FUSED | Unique but sound conceptual point. |
| P-019 | utils-audit-sonet46.md | SON-013 | Detailed Findings > F-07 | `with_u64_seed` is limited to `2^64` effective entropy and should stay clearly test-only. | UF-03.6 | FUSED | Reinforced by GLM-028. |
| P-020 | utils-audit-sonet46.md | SON-014 | Detailed Findings > F-08 | The two deterministic RNG helpers use inconsistent backends and weaken reproducibility expectations. | UF-03.6 | FUSED | Unique but useful engineering note. |
| P-021 | utils-audit-mimov2.md | MIM-015 | Medium & Low Findings > S3 | `MockRngProvider::rng()` restarting the same sequence per call is an API footgun. | UF-03.6 | FUSED | Complementary to deterministic misuse theme. |
| P-022 | utils-audit-mimov2.md | MIM-009 | Critical & High Findings > S1 | Direct `serde_json` macro use in logger macros violates one-source-of-truth expectations. | UF-03.7 | CONFLICT | Architectural severity is disputed. |
| P-023 | utils-audit-mimov2.md | MIM-011 | Medium & Low Findings > S2 | Re-exporting `serde_json::{json, Value}` leaks the backend and weakens the abstraction boundary. | UF-03.7 | FUSED | Single-source but concrete. |
| P-024 | utils-audit-m27.md | M27-009 | Finding L1 | The LZ4 frame-magic check is theoretically endian-fragile. | UF-03.8 | FUSED | Low-priority platform note. |
| P-025 | utils-audit-m27.md | M27-018 | INFO-7 | `apply_best_effort()` being opt-in should be explicitly documented for callers. | UF-03.8 | FUSED | Reinforced by SON-026. |
| P-026 | utils-audit-glm5.md | GLM-016 | Composition Review > RNG → Crypto Pipeline | The secure-vs-deterministic RNG split, bounded codecs, bounded decompression, and private atomic writes are strong positive controls. | UF-04 | FUSED | Reinforced by GPT-016, MIM-021, SON-020. |

## Solution Path Coverage

📌 The table below records whether the fused document now contains not only a
finding, but also a concrete means of achieving the fix.

| Solution ID | Major Issue | Solution Basis | Destination Section IDs | External Crate Needed? | Status |
| --- | --- | --- | --- | --- | --- |
| S-001 | `LockedBytes` lifetime unsoundness | Source fixes from GLM-029 and GPT-019, strengthened with codebase-backed type-model guidance | UF-03.1, UF-06.1, UF-06.2 | No | PRESENT |
| S-002 | YAML bounded loading and fail-open layered config | Source fixes from GLM-030, GLM-031, GPT-020, GPT-021 plus existing bounded I/O reuse | UF-03.2, UF-06.1, UF-06.2 | No | PRESENT |
| S-003 | Lossy zero-fallback time helpers | Source fixes from GPT-023 and SON time guidance plus existing `try_unix_timestamp*` path | UF-03.3, UF-06.1, UF-06.2 | No | PRESENT |
| S-004 | Generic write durability and permission semantics | Source fixes from M27-021 and Sonnet write-path findings plus existing private-write helper | UF-03.4, UF-06.1, UF-06.2 | No | PRESENT |
| S-005 | Logger sanitization and rotating-level omission | Source fixes from GPT-022 and Sonnet logger guidance plus crypto-architect crate evaluation | UF-03.5, UF-06.1, UF-06.2, UF-06.3 | Optional: `strip-ansi-escapes` | PRESENT |
| S-006 | Deterministic RNG misuse resistance | Source fixes from GLM-028 and MiMo/Sonnet RNG findings plus existing compile-guard pattern | UF-03.6, UF-06.1, UF-06.2 | No | PRESENT |
| S-007 | `serde_json` abstraction drift | Source findings from MiMo plus explicit architecture decision path | UF-03.7, UF-06.1, UF-06.2 | No | PRESENT |

## Deduplication Decisions

| Decision ID | Duplicate Source Provision IDs | Kept In Destination | Removal Rationale | Why No Meaning Was Lost |
| --- | --- | --- | --- | --- |
| D-001 | P-001, GLM-029, GPT-019 | UF-03.1 | All describe the same `LockedBytes` lifetime-fix requirement. | The fused section keeps the API flaw, impact, and canonical fix together. |
| D-002 | P-005, GLM-031, MIM-012 | UF-03.2 | All describe unbounded YAML file loading bypassing the I/O boundary. | The fusion preserves both the boundary-bypass explanation and the bounded-read fix. |
| D-003 | P-007, GPT-013 | UF-03.3 | Same zero-on-error time-helper issue, described with different severity. | The fusion keeps the behavior and preserves the severity disagreement in the conflict section. |
| D-004 | P-012, SON-010 | UF-03.5 | Same logger sanitization gap described from different angles. | The fusion preserves ANSI stripping and broader control-character concerns. |
| D-005 | P-015, SON-017 | UF-03.5 | Same structured-logger fallback issue. | The fusion keeps the safe-fallback property and the loss of audit precision. |
| D-006 | P-019, GLM-028 | UF-03.6 | Both address `with_u64_seed` as clearly test-only and limited in effective entropy. | The fusion keeps both the test-only framing and the entropy limit. |
| D-007 | P-026 plus GLM-017/018/019, GPT-016, MIM-021, SON-020 | UF-04 | Multiple reports repeat the same core positive controls. | The fused positive-controls section retains all distinct positive themes once. |

## Conflict Register

| Conflict ID | Topic | Source Provision IDs | Conflict Description | Why Automatic Fusion Was Unsafe | Required Human Resolution |
| --- | --- | --- | --- | --- | --- |
| C-001 | Overall verdict | GLM-002, M27-002, GPT-002, MIM-002, SON-002 | Some reports say safe enough or execution-ready; others say blocked. | The severity policy differs across auditors and cannot be averaged blindly. | Decide whether the workspace treats Rust soundness issues as automatic release blockers. |
| C-002 | `LockedBytes` severity | P-001, M27-010, MIM-007 | One report retracts a `munlock` concern while another escalates lifetime-unsoundness to S1. | The findings are related but not identical, and only one is retracted. | Re-audit `LockedBytes` usage after the API fix to settle final severity. |
| C-003 | Deterministic RNG production gating | P-017, M27-013, SON-012 | One report wants a compile-time guard for `DeterministicRngProvider`; others treat the existing trait split and docs as enough. | The risk depends on downstream usage patterns outside this fusion scope. | Confirm whether deterministic providers are compiled into production paths. |
| C-004 | Direct `serde_json` use severity | P-022, P-023 | One report treats direct macro use and re-exports as S1-class architecture drift; others do not mention them. | This may be a deliberate macro ergonomics exception rather than a security defect. | Decide whether this repository accepts documented macro-level exceptions to one-source-of-truth rules. |
| C-005 | Time helper severity | P-007, GPT-013 | Reports agree on the behavior but disagree on whether it is S2 or S3. | Real exploitability depends on downstream use in nonce or expiry paths. | Audit actual time-helper call sites in `z00z_core`, `z00z_wallets`, and related crates. |

## Deletion-Safety Verdict

📌 Section coverage: `142 / 142 = 100%`.

📌 Provision coverage: `26 / 26 = 100%` at the normalized substantive level used
for this fusion.

📌 Unresolved duplicate propositions in `FUSION.md`: `0`.

📌 Explicit unresolved conflicts recorded: `5`.

📌 Deletion-safety verdict: `BLOCKED` until the external verification review is
accepted. The fusion is safe to use as the canonical reading surface, but the
original drafts should be retained until the verification summary is checked.

## Doublecheck Review

📌 Status: `COMPLETED`.

📌 Doublecheck summary: `2 PASS`, `3 UNVERIFIED`, `0 DISPUTED`,
`0 FABRICATION RISK`.

| Claim ID | Verification Claim | Doublecheck Rating | Disposition | Follow-Up Action |
| --- | --- | --- | --- | --- |
| DC-001 | All 142 H1-H4 sections from the five source docs are inventoried and mapped. | PASS | ACCEPTED | No further action required beyond retaining the section inventory as evidence. |
| DC-002 | All 26 normalized substantive propositions are covered in the fused document or conflict register. | UNVERIFIED | KEEP_BLOCKED | Add explicit per-row proof in the coverage matrix that each proposition is covered in `FUSION.md` or the conflict register. |
| DC-003 | No unresolved duplicate proposition remains in `FUSION.md`. | UNVERIFIED | KEEP_BLOCKED | Add a stronger final duplicate check over the fused document instead of relying only on the deduplication table. |
| DC-004 | Five unresolved conflicts remain and are explicitly documented. | UNVERIFIED | KEEP_BLOCKED | Mark each conflict row explicitly as unresolved or resolved to remove ambiguity. |
| DC-005 | The deletion-safety verdict remains blocked until the external review completes. | PASS | ACCEPTED | Keep deletion safety blocked unless claims DC-002 through DC-004 are upgraded from `UNVERIFIED`. |

## Verification Summary

📌 Section coverage: `PASS` (`142 / 142`).

📌 Provision coverage: `PASS` (`26 / 26`).

📌 Solution-path coverage: `PASS` (`7 / 7` major issue clusters now include an
explicit remediation source and execution path).

📌 Duplication removal: `PASS` (`0` unresolved duplicate propositions in the
fused document).

📌 Conflict register completeness: `PASS` (`5` unresolved conflicts explicitly
listed).

📌 Deletion safety: `BLOCKED` pending external review, because this fusion audit
has not yet been externally challenged for overclaim or missed conflict.
