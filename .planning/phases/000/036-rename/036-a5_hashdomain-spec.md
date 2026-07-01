---
title: Hash Domain Canonicalization
version: 1.0
date_created: 2026-04-21
last_updated: 2026-04-21
tags: [phase-036, rename, crypto, domain-separation]
---

## 1. Purpose & Scope

This spec normalizes non-Tari `hash_domain!` declarations and adjacent KDF salt constants to one canonical style.

The target form is:

```rust
hash_domain!(ConsensusHashDomain, "z00z.consensus.hash.v1", 1);
```

The same dotted, lower-case namespace style applies to every domain label and salt byte sequence covered by this spec. Tari vendor code under `crates/z00z_crypto/tari/` is excluded.

## 2. Definitions

- Canonical domain form: a `hash_domain!` declaration with a descriptive `PascalCase` type name and a lower-case dotted namespace string that starts with `z00z.` and ends with a version suffix such as `.v1`.
- Legacy domain form: an uppercase, slash-separated, or otherwise non-canonical literal currently used as a domain separator.
- Salt constant: a `&[u8]` value used as an HKDF or domain-separation input and treated as a namespace-bearing contract.

## 3. Requirements, Constraints & Guidelines

- REQ-001: All non-Tari `hash_domain!` literals shall use the dotted lower-case `z00z.<scope>.<purpose>.v<version>` format.
- REQ-002: Domain identifiers shall remain descriptive `PascalCase` names; opaque abbreviations shall be expanded when they do not add audit value.
- REQ-003: KDF salts participating in the same contract family shall use the same dotted lower-case namespace style and full-word constant names.
- REQ-004: The version number and cryptographic family shall remain unchanged unless a separate migration spec explicitly approves a protocol change.
- CON-001: Do not touch files under `crates/z00z_crypto/tari/`.
- CON-002: Do not alter the `hash_domain!(TypeName, "domain", version)` call shape; only the identifier and literal should change when a row is normalized.
- GUD-001: Prefer one owner per domain family so future additions reuse the same naming lane instead of creating a second dialect.

## 4. Normalization Ledger

The table below lists every noncanonical consensus-domain declaration in source order. `ConsensusHashDomain` and `HashToScalarDomain` are intentionally omitted because they already satisfy the canonical dotted-lowercase format.

The `Comments` column records `036-22` execution verification state. It remains blank until a no-drift continuation is verified against workspace artifacts.

| # | Kind | Current Surface | File | Line | Canonical Target | Change Type | Notes | Comments |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | hash_domain | `hash_domain!(EphemeralScalarDomain, "Z00Z/R", 1);` | `crates/z00z_crypto/src/domains.rs` | 158 | `hash_domain!(EphemeralScalarDomain, "z00z.consensus.ephemeral_scalar.v1", 1);` | literal normalization | Hedged ephemeral scalar. | ✅ |
| 2 | hash_domain | `hash_domain!(ReceiverIdDomain, "Z00Z/RID", 1);` | `crates/z00z_crypto/src/domains.rs` | 159 | `hash_domain!(ReceiverIdDomain, "z00z.consensus.receiver_id.v1", 1);` | literal normalization | Owner-handle derivation for the receiver identity lane. | ✅ |
| 3 | hash_domain | `hash_domain!(ViewKeyDomain, "Z00Z/VIEW", 1);` | `crates/z00z_crypto/src/domains.rs` | 160 | `hash_domain!(ViewKeyDomain, "z00z.consensus.view_key.v1", 1);` | literal normalization | View-secret derivation lane. | ✅ |
| 4 | hash_domain | `hash_domain!(KdhDomain, "Z00Z/DH", 1);` | `crates/z00z_crypto/src/domains.rs` | 163 | `hash_domain!(DhKeyDomain, "z00z.consensus.dh_key.v1", 1);` | identifier + literal normalization | ECDH-to-symmetric-key lane. | ✅ |
| 5 | hash_domain | `hash_domain!(OwnerTagDomain, "Z00Z/TAG", 1);` | `crates/z00z_crypto/src/domains.rs` | 166 | `hash_domain!(OwnerTagDomain, "z00z.consensus.owner_tag.v1", 1);` | literal normalization | Owner-tag derivation lane. | ✅ |
| 6 | hash_domain | `hash_domain!(Tag16Domain, "Z00Z/TAG16", 1);` | `crates/z00z_crypto/src/domains.rs` | 167 | `hash_domain!(Tag16Domain, "z00z.consensus.tag16.v1", 1);` | literal normalization | Scan-accelerator tag lane. | ✅ |
| 7 | hash_domain | `hash_domain!(AssetIdDomain, "Z00Z/ASSET", 1);` | `crates/z00z_crypto/src/domains.rs` | 170 | `hash_domain!(AssetIdDomain, "z00z.consensus.asset_id.v1", 1);` | literal normalization | Asset identifier derivation lane. | ✅ |
| 8 | hash_domain | `hash_domain!(LeafAdDomain, "Z00Z/LEAFAD", 1);` | `crates/z00z_crypto/src/domains.rs` | 171 | `hash_domain!(LeafAdDomain, "z00z.consensus.leaf_ad.v1", 1);` | literal normalization | Leaf associated-data lane. | ✅ |
| 9 | hash_domain | `hash_domain!(LeafHashDomain, "Z00Z/LEAFHASH", 1);` | `crates/z00z_crypto/src/domains.rs` | 172 | `hash_domain!(LeafHashDomain, "z00z.consensus.leaf_hash.v1", 1);` | literal normalization | Leaf hash lane for the JMT path. | ✅ |
| 10 | hash_domain | `hash_domain!(ZkPackDomain, "Z00Z/ZKPACK", 1);` | `crates/z00z_crypto/src/domains.rs` | 175 | `hash_domain!(ZkPackDomain, "z00z.consensus.zkpack.v1", 1);` | literal normalization | ZkPack sponge initialization lane. | ✅ |
| 11 | hash_domain | `hash_domain!(PackKeyDomain, "Z00Z/PACKKEY", 1);` | `crates/z00z_crypto/src/domains.rs` | 176 | `hash_domain!(PackKeyDomain, "z00z.consensus.pack_key.v1", 1);` | literal normalization | Pack encryption key derivation lane. | ✅ |
| 12 | hash_domain | `hash_domain!(PackNonceDomain, "Z00Z/PACKNONCE", 1);` | `crates/z00z_crypto/src/domains.rs` | 177 | `hash_domain!(PackNonceDomain, "z00z.consensus.pack_nonce.v1", 1);` | literal normalization | Pack nonce derivation lane. | ✅ |
| 13 | hash_domain | `hash_domain!(PackFlowDomain, "Z00Z/PKFLOW", 1);` | `crates/z00z_crypto/src/domains.rs` | 178 | `hash_domain!(PackFlowDomain, "z00z.consensus.pack_flow.v1", 1);` | literal normalization | Older pack flow lane. | ✅ |
| 14 | hash_domain | `hash_domain!(PackMacDomain, "Z00Z/PKMAC", 1);` | `crates/z00z_crypto/src/domains.rs` | 179 | `hash_domain!(PackMacDomain, "z00z.consensus.pack_mac.v1", 1);` | literal normalization | Older pack MAC lane. | ✅ |
| 15 | hash_domain | `hash_domain!(XofBlockDomain, "Z00Z/XOFBLK", 1);` | `crates/z00z_crypto/src/domains.rs` | 180 | `hash_domain!(XofBlockDomain, "z00z.consensus.xof_block.v1", 1);` | literal normalization | XOF block generation lane. | ✅ |
| 16 | hash_domain | `hash_domain!(TxDigestDomain, "Z00Z/TXDIGEST", 1);` | `crates/z00z_crypto/src/domains.rs` | 183 | `hash_domain!(TxDigestDomain, "z00z.consensus.tx_digest.v1", 1);` | literal normalization | Transaction digest lane. | ✅ |
| 17 | hash_domain | `hash_domain!(SpendNullifierDomain, "Z00Z/NULLIFIER", 1);` | `crates/z00z_crypto/src/domains.rs` | 184 | `hash_domain!(SpendNullifierDomain, "z00z.consensus.nullifier.v1", 1);` | literal normalization | Spend-nullifier lane. | ✅ |
| 18 | hash_domain | `hash_domain!(TxOutputNonceDomain, "Z00Z/TXOUTNONCE", 1);` | `crates/z00z_crypto/src/domains.rs` | 185 | `hash_domain!(TxOutputNonceDomain, "z00z.consensus.tx_output_nonce.v1", 1);` | literal normalization | Tx output nonce lane. | ✅ |
| 19 | hash_domain | `hash_domain!(RangeCtxDomain, "Z00Z/RANGECTX", 1);` | `crates/z00z_crypto/src/domains.rs` | 186 | `hash_domain!(RangeCtxDomain, "z00z.consensus.range_ctx.v1", 1);` | literal normalization | Range-proof context lane. | ✅ |
| 20 | hash_domain | `hash_domain!(Stage4OutSeedDomain, "Z00Z/STAGE4/OUTSEED", 1);` | `crates/z00z_crypto/src/domains.rs` | 187 | `hash_domain!(Stage4OutSeedDomain, "z00z.consensus.stage4_out_seed.v1", 1);` | literal normalization | Stage-4 deterministic output seed lane. | ✅ |
| 21 | hash_domain | `hash_domain!(TxProofDomain, "Z00Z/TXPROOF/V1", 1);` | `crates/z00z_crypto/src/domains.rs` | 188 | `hash_domain!(TxProofDomain, "z00z.consensus.tx_proof.v1", 1);` | literal normalization | TxProof Fiat-Shamir challenge lane. | ✅ |
| 22 | hash_domain | `hash_domain!(CheckpointDomain, "Z00Z/CHECKPOINT/V1", 1);` | `crates/z00z_crypto/src/domains.rs` | 189 | `hash_domain!(CheckpointDomain, "z00z.consensus.checkpoint.v1", 1);` | literal normalization | Checkpoint-proof lane. | ✅ |
| 23 | hash_domain | `hash_domain!(ReqDomain, "Z00Z/REQv1", 1);` | `crates/z00z_crypto/src/domains.rs` | 192 | `hash_domain!(PaymentRequestDomain, "z00z.payment.request.v1", 1);` | identifier + literal normalization | Expands the abbreviated request domain name. | ✅ |
| 24 | hash_domain | `hash_domain!(RcardDomain, "Z00Z/RCARD", 1);` | `crates/z00z_crypto/src/domains.rs` | 193 | `hash_domain!(ReceiverCardDomain, "z00z.receiver.card.v1", 1);` | identifier + literal normalization | Expands the abbreviated receiver-card domain name. | ✅ |
| 25 | const | `KDF_CONS_SALT: &[u8] = b"z00z/consensus/kdf/v1";` | `crates/z00z_crypto/src/kdf.rs` | 150 | `KDF_CONSENSUS_SALT: &[u8] = b"z00z.consensus.kdf.v1";` | identifier + literal normalization | Expands the consensus abbreviation and aligns the salt with the dotted namespace convention. | ✅ |
| 26 | const | `KDF_WLT_SALT: &[u8] = b"z00z/wallet/kdf/v1";` | `crates/z00z_crypto/src/kdf.rs` | 151 | `KDF_WALLET_SALT: &[u8] = b"z00z.wallet.kdf.v1";` | identifier + literal normalization | Expands the wallet abbreviation and removes the slash-separated label. | ✅ |
| 27 | const | `KDF_WLT_VAR_SALT: &[u8] = b"z00z/wallet/kdf_var/v1";` | `crates/z00z_crypto/src/kdf.rs` | 152 | `KDF_WALLET_VARIABLE_SALT: &[u8] = b"z00z.wallet.kdf.variable.v1";` | identifier + literal normalization | Replaces the shorthand `VAR` with the full word `VARIABLE` for clarity. | ✅ |

## 5. Acceptance Criteria

- AC-001: No non-Tari `hash_domain!` literal in `crates/**/*.rs` uses uppercase, slash-separated, or mixed-case namespace syntax.
- AC-002: The ledger rows above enumerate every noncanonical consensus-domain declaration in `crates/z00z_crypto/src/domains.rs` in source order.
- AC-003: `KDF_CONS_SALT`, `KDF_WLT_SALT`, and `KDF_WLT_VAR_SALT` no longer appear in live non-Tari Rust sources after the sweep.
- AC-004: Tari vendor files under `crates/z00z_crypto/tari/` remain unchanged.

## 6. Test Automation Strategy

- Primary scan: `rg -n 'hash_domain!\(|KDF_CONS_SALT|KDF_WLT_SALT|KDF_WLT_VAR_SALT|Z00Z/' crates --glob '*.rs' --glob '!crates/z00z_crypto/tari/**'`
- Expected result: only canonical dotted-lowercase domain strings and the renamed identifiers above.
- Follow-up: run the existing crate tests that cover `crates/z00z_crypto/src/domains.rs` and `crates/z00z_crypto/src/kdf.rs` to confirm the domain registry and KDF helpers still agree on the live surface.

## 7. Validation Criteria

- The consensus-domain block in `crates/z00z_crypto/src/domains.rs` uses the dotted lowercase namespace style everywhere.
- The KDF salt constants use the same namespace style and full-word identifiers.
- No Tari source file changes are required.
- The canonical example remains the single documented form for future additions.

## 8. Related Specifications / Further Reading

- `.planning/phases/036-rename/036-a1-versioning-spec.md`
- `crates/z00z_crypto/src/domains.rs`
- `crates/z00z_crypto/src/kdf.rs`
- `crates/z00z_crypto/src/kdf_domains.rs`
