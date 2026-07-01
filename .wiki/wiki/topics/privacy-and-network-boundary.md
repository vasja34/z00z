---
title: "Privacy And Network Boundary"
category: topic
sources:
  - raw/papers/2026-06-26-privacy-threat-model.md
  - raw/papers/2026-06-26-onionnet.md
  - raw/papers/2026-06-26-pq-migration.md
  - raw/papers/2026-06-26-main.md
created: 2026-06-26
updated: 2026-06-27
tags: [privacy, network, pq]
aliases: [privacy boundary]
confidence: medium
volatility: warm
verified: 2026-06-27
compiled-from: sources
summary: "Privacy in the corpus is layered across settlement, wallet behavior, network assistance, and future cryptographic migration."
---

# Privacy And Network Boundary

> The privacy papers treat anonymity as a systems property, not as a single proof primitive or a one-number metric.

## Layered Privacy

The privacy threat model paper separates protocol guarantees from wallet,
network, ingress, egress, and operator leakage. This keeps the claims honest:
the core can preserve private settlement without pretending to solve every
transport or usage-pattern problem by itself.

## Network And Cryptography

OnionNet adds a transport layer that reduces route exposure and relay leakage,
while the PQ migration paper keeps the long-term cryptographic story explicit
instead of assuming current primitives stay safe forever.

## See Also

- [[private-object-settlement|Private Object Settlement]] ([Private Object Settlement](../concepts/private-object-settlement.md)) — wallet-local meaning as the base privacy move
- [[cross-chain-private-rights|Cross-Chain Private Rights On Transparent Chains]] ([Cross-Chain Private Rights On Transparent Chains](../concepts/cross-chain-private-rights.md)) — explains why private transfer can exist between two still-public route edges
- [[expansion-paths|Expansion Paths]] ([Expansion Paths](expansion-paths.md)) — where privacy constraints meet product growth
- [[whitepaper-corpus|Whitepaper Corpus]] ([Whitepaper Corpus](../references/whitepaper-corpus.md)) — corpus map and source grouping

## Sources

- [Z00Z Privacy Threat Model And Metrics](../../raw/papers/2026-06-26-privacy-threat-model.md) — formal threat model
- [Z00Z OnionNet Whitepaper](../../raw/papers/2026-06-26-onionnet.md) — transport boundary
- [Z00Z PQ Migration Whitepaper](../../raw/papers/2026-06-26-pq-migration.md) — migration discipline
- [Z00Z Main Whitepaper](../../raw/papers/2026-06-26-main.md) — protocol privacy thesis
