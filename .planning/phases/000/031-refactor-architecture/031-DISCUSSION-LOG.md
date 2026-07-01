# Phase 31: Refactor Architecture - Discussion Log

**Date:** 2026-04-04
**Mode:** discuss
**Outcome:** Context normalized from existing review and code evidence; ready for
planning.

## 🎯 Inputs Reviewed

- `.github/get-shit-done/workflows/discuss-phase.md`
- `.github/get-shit-done/workflows/discuss-phase-assumptions.md`
- `.github/get-shit-done/templates/context.md`
- `.planning/PROJECT.md`
- `.planning/REQUIREMENTS.md`
- `.planning/ROADMAP.md`
- `.planning/phases/031-refactor-architecture/031-TODO.md`
- `.planning/phases/000/030-refactor-long-files/030-CONTEXT.md`
- `.planning/phases/000/029-crypto-audit-wallets/029-CONTEXT.md`
- `.planning/phases/000/027-crypto-audit-utils/027-CONTEXT.md`
- `.planning/codebase/ARCHITECTURE.md`
- `.planning/codebase/CONVENTIONS.md`
- `.planning/codebase/STRUCTURE.md`

## 🔎 Code Evidence Collected

- `crates/z00z_core/src/lib.rs` still exposes a broad root contract via
  `pub use assets::*`.
- `crates/z00z_crypto/src/lib.rs` still mixes Z00Z aliases and direct Tari
  concrete-type re-exports at the root.
- `crates/z00z_wallets/src/core/mod.rs` already introduces ownership facades
  (`app_owned`, `wallet_owned`) but still contains broad facade-style re-export
  modules.
- `crates/z00z_wallets/src/services/wallet_service.rs` is still assembled via
  `include!` fragments.
- `crates/z00z_wallets/src/adapters/rpc/types/mod.rs` still exposes DTO modules
  through broad public re-exports.
- `crates/z00z_networks/rpc/src/lib.rs` defines only a request/response
  transport seam.
- `crates/z00z_networks/onionnet/` is empty, confirming the namespace exists
  before its ownership rules are fully encoded in code.

## 🧭 Key Observations

- The Phase 031 source document already contains embedded answers for most
  crate-level open questions. The discussion phase therefore did not need to
  invent new product direction; it needed to normalize those answers into one
  planning-ready contract.
- The biggest mismatch is between intended architecture and stable-looking
  public surfaces. The code compiles, but several root facades are broader than
  the maturity of the contracts they expose.
- Compatibility handling is no longer a scattered deep-import problem, but it
  is still not under strict criteria. Public re-exports, legacy names, and
  migration helpers remain visible in places where the architecture says they
  should be hidden or narrowly scoped.
- Security findings in the Phase 031 source mostly reinforce the same
  architectural direction: fail closed, reduce ambiguous surfaces, and keep
  persisted identity or canonical proof-binding as the single source of truth.

## 🔐 Decisions Locked During Discussion

### Export Governance

- Stable root facades must use curated exports only.
- Public wildcard re-exports are not acceptable under strict criteria.
- Version-suffixed names survive only until the canonical implementation is
  isolated; suffix removal is a second wave, not the first.

### Compatibility Governance

- Compatibility paths are allowed only for persisted-data migration or explicit
  feature-gated transition support.
- Compatibility helpers must not be default public import surfaces.

### Crate Ownership

- `z00z_networks` remains in the current layout; ownership is clarified before
  new OnionNet code lands.
- `z00z_wallets` stays one crate for now, but its internal boundaries must be
  hardened before further product growth.
- `z00z_storage` keeps checkpoint, snapshot, and asset-tree ownership together,
  but backend mechanics stay behind stricter internal seams.

### Security-Carryover Direction

- `z00z_crypto`: remove or confine non-test exposure of dangerous test-only
  helpers.
- `z00z_core`: bound JSON import surfaces or document upstream size ceilings.
- `z00z_storage`: tighten seal/finalization semantics toward canonical replay
  evidence and proof binding.
- `z00z_wallets`: persisted identity becomes the authoritative per-wallet
  source; lock transitions require authorization.
- `z00z_simulator`: debug secret artifacts must not stay in the default output
  contract.

## ❓ Remaining Flex Points

- Exact wave ordering across crates after import-graph mapping.
- Whether policy checks land first as tests, structural guards, or both.
- Whether wallet and storage security hardening becomes one integrated Phase 031
  plan or a set of sub-plans.
- The strict final shape of the long-term wallet facade remains open at the
  sequencing level, but not at the direction level: provisional surfaces must
  stop looking stable.

## ✅ Ready State

- Phase context can be written without further user clarification because the
  Phase 031 source already encodes the dominant answers.
- The next workflow step should be planning, with import-graph audits and wave
  sequencing as the first execution-planning inputs.

---

*Phase: 031-refactor-architecture*
*Discussion logged: 2026-04-04*
