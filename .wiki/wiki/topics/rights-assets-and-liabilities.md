---
title: "Rights, Assets, And Liabilities"
category: topic
sources:
  - raw/papers/2026-06-26-assets-rights-vouchers.md
  - raw/papers/2026-06-26-linked-liability.md
  - raw/papers/2026-06-26-cross-chain-integration.md
  - raw/papers/2026-06-26-use-cases.md
  - raw/notes/2026-06-26-z00z-cross-chain-composition.md
created: 2026-06-26
updated: 2026-06-27
tags: [rights, assets, liabilities]
aliases: [object families]
confidence: medium
volatility: warm
verified: 2026-06-27
compiled-from: sources
summary: "Several whitepapers describe value not as one coin type, but as a family of privately transferable assets, vouchers, rights, and liabilities."
---

# Rights, Assets, And Liabilities

> The corpus repeatedly turns policy and economic meaning into bounded objects instead of collapsing everything into one public-account balance model.

## Object Families

The assets-rights-vouchers paper separates fully valuable assets, conditional
vouchers, and authority-only rights. The linked-liability paper extends that
logic into delayed-connectivity fraud enforcement, while the cross-chain paper
shows how external custody can still map into private internal rights.

The cross-chain composition image stub preserves provenance for the visual
asset that explains how external custody boundaries compose with private
internal rights.

## Why This Matters

This object view is what lets Z00Z support selective disclosure, external-asset
ownership transfer, and policy-shaped money without abandoning the same
settlement model.

## See Also

- [[private-object-settlement|Private Object Settlement]] ([Private Object Settlement](../concepts/private-object-settlement.md)) — shared primitive behind these object families
- [[cross-chain-private-rights|Cross-Chain Private Rights On Transparent Chains]] ([Cross-Chain Private Rights On Transparent Chains](../concepts/cross-chain-private-rights.md)) — detailed mechanism for private reassignment of rights over public external custody
- [[expansion-paths|Expansion Paths]] ([Expansion Paths](expansion-paths.md)) — how these objects power later ecosystem growth
- [[whitepaper-corpus|Whitepaper Corpus]] ([Whitepaper Corpus](../references/whitepaper-corpus.md)) — corpus map and source grouping

## Sources

- [Z00Z Assets, Rights, And Vouchers Whitepaper](../../raw/papers/2026-06-26-assets-rights-vouchers.md) — object taxonomy
- [Z00Z Linked Liability Whitepaper](../../raw/papers/2026-06-26-linked-liability.md) — liability adaptation
- [Z00Z Cross-Chain Integration Whitepaper](../../raw/papers/2026-06-26-cross-chain-integration.md) — external asset rights
- [Z00Z Use Cases Whitepaper](../../raw/papers/2026-06-26-use-cases.md) — applied object families
- [Z00Z Cross Chain Composition asset](../../raw/notes/2026-06-26-z00z-cross-chain-composition.md) — external-right composition visual provenance
