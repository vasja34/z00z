# Z00Z Verification Orchestrator Attestation

**Date:** 2026-06-15
**Source reviewed:** `docs/tech-papers/z00z-verification-orchestrator.md`
**Result:** Adopt the pipeline, but as a staged continuous bug-finding and evidence orchestration system, not as a promise of full formal proof.

## ✅ Recommendation

Use one orchestrator skill plus five level skills:

| Level | Skill | Decision |
| --- | --- | --- |
| Orchestrator | `z00z-verification-orchestrator` | Adopt now |
| L0 | `z00z-l0-spec-gate` | Adopt now |
| L1 | `z00z-l1-protocol-model-gate` | Adopt now, but model coverage starts as UNKNOWN until `specs/tla` and `specs/alloy` exist |
| L2 | `z00z-l2-crypto-protocol-gate` | Adopt now for domain/transcript checks; add Tamarin/ProVerif models incrementally |
| L3 | `z00z-l3-rust-implementation-gate` | Adopt now |
| L4 | `z00z-l4-security-engineering-gate` | Adopt now |

The main correction is scope discipline: L4 should combine fuzzing, dependency audit, unsafe inventory, semver, and constant-time harnesses instead of splitting into competing skills. If it grows too large later, split it by execution cost, not by tool name.

## ⚠️ Current Repo Fit

The source document uses conceptual crate names such as `z00z-state`, `z00z-validator`, and `z00z-aggregator`. The current repository uses these real surfaces instead:

- `crates/z00z_storage/` for checkpoint, settlement, HJMT, and proof storage logic.
- `crates/z00z_crypto/` for project-owned crypto wrappers and domain-separated crypto glue.
- `crates/z00z_core/` for assets, genesis, metadata, and core domain behavior.
- `crates/z00z_wallets/` for wallet, delivery, timing, and key-flow surfaces.
- `crates/z00z_rollup_node/`, `crates/z00z_runtime/`, and `crates/z00z_simulator/` for runtime, node, validator, watcher, and simulation surfaces.

There is no `specs/` tree in the current workspace yet, so L1 and parts of L2 must initially report UNKNOWN/SKIPPED instead of PASS. That is correct behavior.

## 🎯 What Is Worth Doing Now

- Keep the existing Rust delivery gate and add L3 wrappers for targeted Kani, Miri, Loom, and Verus.
- Add L0 invariant/traceability extraction before new security-critical code.
- Add L2 static domain and transcript checks immediately because the repository already contains `hash_domain!` and proof binding code.
- Add L4 supply-chain and unsafe reporting now; cargo-audit, cargo-deny, cargo-vet, cargo-geiger, cargo-fuzz, and cargo-semver-checks are practical local gates.
- Start L1 with the first TLA+ model for checkpoint/settlement and the first Alloy model for asset/right/voucher/policy constraints.

## 🛑 What Not To Make Mandatory Yet

- Do not require Coq, Lean, or Isabelle for the whole system. They are high-value only for selected stable mathematical constructions.
- Do not make both Verus and Creusot hard gates at the same time. Use Verus first for pure critical kernels; keep Creusot experimental until a concrete target exists.
- Do not require EasyCrypt or hax/hacspec extraction before the cryptographic construction stabilizes.
- Do not claim Alloy CI coverage from the stock GUI jar alone. Add a headless runner before marking Alloy as machine-enforced in CI.
- Do not let an LLM be the verifier. The LLM should generate attack hypotheses, route tools, and summarize evidence.

## 🔑 Evidence Semantics

Use these statuses consistently:

- `PASS`: the configured machine check ran and passed.
- `FAIL`: the configured machine check ran and failed.
- `SKIPPED`: no matching files/specs/tool configuration existed for this run.
- `UNKNOWN`: the security claim needs a model, invariant, harness, or human review that does not exist yet.
- `NEEDS_HUMAN_CRYPTO_REVIEW`: a new cryptographic construction, proof, or leakage assumption was introduced.

## 📌 Rollout Order

1. Install tools with `scripts/verification-tools/install-verification-tools.sh --profile all`.
2. Run `scripts/verification-tools/install-verification-tools.sh --self-test`.
3. Use L0/L3/L4 as the default pre-commit/pre-PR path.
4. Add `specs/invariants/*.yaml` and require `ZINV:` references for security-critical code in strict mode.
5. Add first L1 models for checkpoint/settlement and rights/voucher/policy.
6. Add first L2 Tamarin/ProVerif models for stealth/inbox/payment request flows.
7. Promote only stable, non-flaky checks into CI required status.

## ✅ Created Artifacts

- `.github/skills/z00z-verification-orchestrator/`
- `.github/skills/z00z-l0-spec-gate/`
- `.github/skills/z00z-l1-protocol-model-gate/`
- `.github/skills/z00z-l2-crypto-protocol-gate/`
- `.github/skills/z00z-l3-rust-implementation-gate/`
- `.github/skills/z00z-l4-security-engineering-gate/`
- `scripts/verification-tools/install-verification-tools.sh`

## 🔍 External Tool Facts Checked

Critical installer choices were checked against upstream sources:

- Kani uses `cargo install --locked kani-verifier` followed by `cargo kani setup`: <https://model-checking.github.io/kani/install-guide.html>
- hax is installed with `nix profile install github:hacspec/hax` when Nix is available: <https://hax.cryspen.com/manual/>
- Tamarin supports Homebrew, Arch, Nix/NixOS, binaries, or source builds: <https://tamarin-prover.com/install.html>
- Apalache supports prebuilt packages, Docker, or source builds: <https://apalache-mc.org/docs/apalache/installation/index.html>
- Prusti provides precompiled release binaries or source builds exposing `cargo-prusti` and `prusti-rustc`: <https://github.com/viperproject/prusti-dev>

The remaining tool inventory is encoded as recommended installer coverage and local status checks, not as a claim that every upstream installation path is fully normalized across Linux distributions.
