---
title: "Corpus Terminology"
category: reference
sources:
  - raw/notes/2026-06-26-corpus-terminology-reference.md
created: 2026-06-26
updated: 2026-06-26
tags: [terminology, glossary, editorial]
aliases: [shared vocabulary, term contract]
confidence: medium
volatility: warm
verified: 2026-06-26
compiled-from: sources
summary: "The terminology reference normalizes Z00Z corpus vocabulary, authority boundaries, aliases, abbreviations, and editorial guardrails."
---

# Corpus Terminology

> The terminology source is the vocabulary guardrail for the Z00Z paper family: it normalizes terms without moving authority away from the paper that owns each concept.

## Scope

The terminology reference covers explicit `Key Terms` sections, glossary
appendices, and abbreviation tables across the Z00Z paper corpus. It is not a
code-symbol index. It exists to prevent drift when the same idea appears across
protocol, cross-chain, agentic, legal, governance, and migration papers.

## Shared Term Contract

The source organizes the live vocabulary around core protocol terms such as
`AssetLeaf`, `RightLeaf`, `TxPackage`, `ClaimTxPackage`, `Checkpoint`,
`Settlement evidence`, `Wallet-local possession`, `Soft confirmation`,
`SettlementStateRoot`, and `SettlementPath`. It also separates semantic object
families: `Asset` carries final value, `Voucher` carries conditional value, and
`Right` carries bounded authority.

For machine, service, and agent flows, the preferred terms include `Spendable
right`, `Spendable capability object`, `Agent spending envelope`,
`MachineCapabilityObject`, `FeeEnvelope`, `FeeCredit`, `Offline receipt`, and
`Checkpointed reconciliation`.

## Authority Map

The reference does not flatten all papers into one source of truth. It maps
authority by domain: the main whitepaper owns core settlement vocabulary,
cross-chain owns lockers and external-asset terms, the agentic paper owns
machine and agent rights, the legal paper owns neutral-protocol language, the
tokenomics paper owns native-asset and fee-lane terms, and HJMT design owns
storage-root and proof-path names.

This matters because a wording dispute should be resolved by the owning paper,
with this reference acting as the reader-facing index.

## Editorial Guardrails

The source prefers live-core nouns over future-architecture nouns unless a text
is explicitly discussing target architecture. It also rejects loose synonyms
that change risk posture: `Selective disclosure`, `Selective audit`, and
`Selective Reveal` are different; `SettlementStateRoot` is the live root term,
while `AssetStateRoot` is compatibility language; `Neutral protocol`,
`independent issuer`, and `reference wallet` are not interchangeable.

## Abbreviation Discipline

Abbreviations are classified as corpus-default, corpus-acceptable after first
mention, or document-local. Short forms such as `FE`, `LL`, and `SCO` should
not become default corpus shorthand unless they are reintroduced in the local
document.

## See Also

- [[protocol-core|Protocol Core]] ([Protocol Core](../topics/protocol-core.md)) - settlement vocabulary in context
- [[rights-assets-and-liabilities|Rights, Assets, And Liabilities]] ([Rights, Assets, And Liabilities](../topics/rights-assets-and-liabilities.md)) - object-family vocabulary
- [[market-positioning|Market Positioning]] ([Market Positioning](../topics/market-positioning.md)) - public-language guardrails
- [[whitepaper-corpus|Whitepaper Corpus]] ([Whitepaper Corpus](whitepaper-corpus.md)) - corpus map and source grouping

## Sources

- [Z00Z Corpus Terminology And Abbreviations Reference](../../raw/notes/2026-06-26-corpus-terminology-reference.md) - shared term contract, authority map, alias rules, abbreviation list, and editorial guardrails
