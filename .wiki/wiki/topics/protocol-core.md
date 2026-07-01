---
title: "Protocol Core"
category: topic
sources:
  - raw/papers/2026-06-26-main.md
  - raw/papers/2026-06-26-smart-cash.md
  - raw/papers/2026-06-26-uniqueness.md
  - raw/papers/2026-06-26-litepaper.md
  - raw/notes/2026-06-26-z00z-rollup-architecture.md
created: 2026-06-26
updated: 2026-06-26
tags: [protocol, settlement, cash]
aliases: [core protocol boundary]
confidence: medium
volatility: warm
verified: 2026-06-26
compiled-from: sources
summary: "The core corpus defines Z00Z as a privacy-first settlement system with wallet-local possession and a narrow checkpointed verification surface."
---

# Protocol Core

> Z00Z’s strongest claim is structural: private objects are prepared locally, while the chain verifies only the evidence needed for safe settlement.

## What The Core Tries To Protect

The main and smart-cash papers emphasize a narrow public surface. The chain is
not supposed to become a global balance table or a general operator-managed
service. It is meant to carry just enough proof to preserve authorization,
finality, and replay safety.

## What Makes The Boundary Distinct

The uniqueness paper reinforces that privacy alone is not the selling point.
The differentiator is the way money, rights, and claims stay wallet-local until
they have to cross the checkpoint boundary.

## Settlement Notary Framing

The litepaper compresses the same boundary for first-pass readers: Z00Z is best
read as a settlement notary over replay-safe public evidence, not as a public
balance table or as a chain that mirrors the full wallet state. Wallets prepare
objects locally; authoritative settlement starts only at checkpoint
verification. The rollup-architecture image stub preserves provenance for the
visual explanation of that checkpoint-facing architecture.

## See Also

- [[private-object-settlement|Private Object Settlement]] ([Private Object Settlement](../concepts/private-object-settlement.md)) — recurring primitive behind the protocol
- [[expansion-paths|Expansion Paths]] ([Expansion Paths](expansion-paths.md)) — where the same core boundary is extended outward
- [[market-positioning|Market Positioning]] ([Market Positioning](market-positioning.md)) — public category language derived from this boundary
- [[whitepaper-corpus|Whitepaper Corpus]] ([Whitepaper Corpus](../references/whitepaper-corpus.md)) — corpus map and reading order

## Sources

- [Z00Z Main Whitepaper](../../raw/papers/2026-06-26-main.md) — protocol thesis
- [Z00Z Smart Cash](../../raw/papers/2026-06-26-smart-cash.md) — live verifier boundary
- [Z00Z Uniqueness Whitepaper](../../raw/papers/2026-06-26-uniqueness.md) — architectural differentiation
- [Z00Z Litepaper](../../raw/papers/2026-06-26-litepaper.md) — short public compression and maturity boundary
- [Z00Z Rollup Architecture asset](../../raw/notes/2026-06-26-z00z-rollup-architecture.md) — rollup and checkpoint visual provenance
