---
title: "Cross-Chain Private Rights Explainer"
source: "/home/vadim/Projects/z00z/docs/Z00Z-Cross-Chain-Integration-Whitepaper.md"
type: notes
ingested: 2026-06-27
tags: [cross-chain, privacy, rights, ethereum, explainer]
summary: "Explainer note answering how a transparent chain can anchor a privately transferable right without claiming that the external asset itself became anonymous."
content_format: markdown
---

# Cross-Chain Private Rights Explainer

## Source

- Canonical file: `/home/vadim/Projects/z00z/docs/Z00Z-Cross-Chain-Integration-Whitepaper.md`
- Derived from a user question about whether Ethereum can carry anonymous
  rights when Ethereum transactions remain transparent.

## Imported Content

### Core claim

The cross-chain paper does not describe "anonymous ETH on Ethereum". It
describes a split architecture in which the external chain keeps public custody
or public source events, while Z00Z privately transfers the right that points
to that external anchor.

### What actually moves

What moves privately is not the external ERC-20 balance and not the Ethereum
account state. What moves is an internal private right tied to:

- an asset family identity
- one import route
- one replay key or external event identifier
- one redemption or release model

The paper names this family of integration nouns with `Locker`, `LockerID`,
`BridgeInTx`, `BridgeOutTx`, and `Attestation`.

### Minimal EVM locker example

1. A user deposits ETH, wrapped ETH, or an ERC-20 asset into an EVM locker.
2. The locker emits one public deposit event.
3. An adapter binds the asset family, external event reference, finality
   status, amount or object payload, and the intended Z00Z receiver.
4. Z00Z creates one private internal right from that event.
5. That right can move privately inside Z00Z many times.
6. A later exit consumes the right once and only then authorizes external
   release or redemption.

### Other supported route shapes

- Native issuer rail: an attested burn-and-mint route such as a CCTP-style
  import where the external source fact is a burn plus issuer attestation.
- Non-token fact import: a work result, merchant receipt, score, or other
  attested condition that creates a private claim right rather than a bridged
  token balance.

### What remains public

The public chain still sees the entry and exit edges. It can see that value
entered a route and later left a route. What it no longer sees is the full
intermediate reassignment graph while the internal right is circulating inside
Z00Z.

### Guarantees versus assumptions

Z00Z can guarantee that:

- the imported right was created under the chosen route rules
- the private right was not consumed twice inside Z00Z
- the exit path depends on prior internal consumption

External operators still have to guarantee:

- custody honesty
- reserve integrity
- release execution
- observation quality
- service availability

### Correlation limits

This architecture does not erase the visibility of public entry and public exit.
The paper flags correlation risks such as:

- exact-amount deposit followed by exact-amount redemption
- repeated use of one narrow route with deterministic timing
- immediate public redemption after a private internal transfer
- service logs that can be joined back to a route-visible custody event
