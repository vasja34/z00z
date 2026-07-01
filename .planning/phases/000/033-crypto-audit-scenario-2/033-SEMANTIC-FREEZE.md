# Phase 032 Semantic Freeze

## 🔒 Purpose

This file is the canonical Wave 0 and Wave 1 semantic contract for Scenario 1.

All later Phase 032 claim, spend, checkpoint, hygiene, summary, and review work
must cite this file instead of restating or strengthening the same guarantees
from memory.

## 🎯 Canonical Meanings

### `leaf_ad_id`

- `leaf_ad_id` is the canonical decrypt-associated-data asset identifier.
- The accepted-flow scanner and decrypt path treat `leaf_ad_id` as the asset-id
  input for `leaf_ad` derivation.
- Any drift between the stored leaf asset identifier and the decrypt-associated
  identifier is a semantic failure, not a compatibility detail.

### `s_out`

- `s_out` is part of the output secret material carried inside the encrypted
  asset pack.
- Current Scenario 1 output construction derives `s_out` from sender-side
  available material during output build.
- Therefore Phase 032 must not claim sender ignorance of `s_out`.
- The truthful anti-theft statement is narrower: wallet-local spend ownership
  still requires receiver-secret-gated verification in addition to the output
  secret material.

### Receiver Card And Payment Request Binding

- A signed receiver card is the canonical routing surface for the receiver view
  and identity keys.
- A signed payment request is the accepted-flow privacy path when request-bound
  behavior is available and approved.
- A request and a receiver card must agree on owner handle, view key, and
  identity key before the validated build path accepts them together.
- A request is not automatically trusted on first sight. The current accepted
  flow is explicit TOFU or pinning policy through `PinnedReceiverCards`.

### `tag16`

- Card-bound mode derives `tag16` from `k_dh` and canonical `leaf_ad`.
- Request-bound mode derives `tag16` from `k_dh` and `req_id`.
- These modes are intentionally distinct and must stay test-covered as distinct
  semantics.

## ⚖️ Wallet-Local Versus Public Guarantees

- Wallet-local ownership checks may use receiver-secret-gated verification and
  decrypted `s_out` semantics.
- Public trustless verification is not implied by the current wallet-local
  checks.
- Phase 032 must not describe the present spend path as a completed public ZK
  verifier boundary unless a later plan explicitly delivers that verifier.
- Phase 032 must not describe the current checkpoint path as stronger than the
  implemented proof and spent-set boundary.

## 🧭 Accepted-Flow Rules

- `output_build.rs` is the accepted-flow constructor seam for request/card
  binding and request-bound tag selection.
- `output_validator.rs` is the accepted-flow lightweight sender-side validation
  seam for the built output contract.
- `stealth_trust.rs` owns the explicit TOFU, pinning, rotation, and revoke
  policy surface.
- `stealth_request.rs` owns request signature, expiry, chain-id, and TOFU-driven
  approval checks.

## ⛔ Forbidden Overclaims

The following claims are forbidden until a later Phase 032 plan proves them in
code and tests:

- "The sender cannot know `s_out`"
- "Scenario 1 already has trustless validator verification"
- "Checkpoint verification is already authoritative end to end"
- "STARK or FRI enforcement is live in the current Scenario 1 stack"

## ✅ Regression Expectations

Phase 032 semantic regression tests must fail when any of the following drift:

- `leaf_ad_id` stops being the canonical decrypt boundary
- wallet-local ownership checks stop depending on receiver secret plus `s_out`
- validated request/card binding accepts foreign or mismatched routes
- TOFU or pinning behavior becomes implicit or silent
- request-bound and card-bound `tag16` behavior collapse into one ambiguous path
