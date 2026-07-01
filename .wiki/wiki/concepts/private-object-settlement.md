---
title: "Private Object Settlement"
category: concept
sources:
  - raw/papers/2026-06-26-main.md
  - raw/papers/2026-06-26-smart-cash.md
  - raw/papers/2026-06-26-uniqueness.md
  - raw/papers/2026-06-26-use-cases.md
created: 2026-06-26
updated: 2026-06-27
tags: [settlement, privacy, rights]
aliases: [wallet-local possession, private rights settlement]
confidence: medium
volatility: warm
verified: 2026-06-27
compiled-from: sources
summary: "Z00Z repeatedly models value and rights as private wallet-local objects that later become checkpointed settlement evidence."
---

# Private Object Settlement

> The corpus converges on one architectural move: ownership meaning stays local to the wallet until a narrow public settlement boundary needs proof.

## Core Idea

The main, smart-cash, uniqueness, and use-case papers all reject a public
account ledger as the primary ownership model. They instead describe spendable
objects, rights, or packages that move privately and only expose enough public
evidence to make settlement replay-safe.

## Why It Recurs

This concept explains why offline flows, policy-shaped money, rights over
external assets, and agent or machine economies can share one protocol family.
They all depend on the same split between local possession and later public
verification.

## See Also

- [[cross-chain-private-rights|Cross-Chain Private Rights On Transparent Chains]] ([Cross-Chain Private Rights On Transparent Chains](cross-chain-private-rights.md)) — shows how the same wallet-local pattern survives when custody remains public on another chain
- [[protocol-core|Protocol Core]] ([Protocol Core](../topics/protocol-core.md)) — the live protocol boundary around packages and settlement
- [[rights-assets-and-liabilities|Rights, Assets, And Liabilities]] ([Rights, Assets, And Liabilities](../topics/rights-assets-and-liabilities.md)) — object families built on the same pattern
- [[privacy-and-network-boundary|Privacy And Network Boundary]] ([Privacy And Network Boundary](../topics/privacy-and-network-boundary.md)) — privacy consequences of keeping meaning wallet-local
- [[whitepaper-corpus|Whitepaper Corpus]] ([Whitepaper Corpus](../references/whitepaper-corpus.md)) — reading map for the full source set

## Sources

- [Z00Z Main Whitepaper](../../raw/papers/2026-06-26-main.md) — protocol thesis and wallet-local possession
- [Z00Z Smart Cash](../../raw/papers/2026-06-26-smart-cash.md) — live boundary and verifier surface
- [Z00Z Uniqueness Whitepaper](../../raw/papers/2026-06-26-uniqueness.md) — positioning and rights-centric differentiation
- [Z00Z Use Cases Whitepaper](../../raw/papers/2026-06-26-use-cases.md) — application families that rely on the same primitive
