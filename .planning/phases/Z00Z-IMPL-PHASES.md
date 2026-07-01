# Z00Z Local Implementation Plans

[TOC]



Date: 2026-05-27

> Phase 062 note (2026-06-25): for `.planning/phases/062-Gaps-Closing-2/`,
> live execution authority is the Phase 062 packet
> (`062-TODO.md`, `062-CONTEXT.md`, `062-COVERAGE.md`, and
> `062-01-PLAN.md` through `062-27-PLAN.md`). This document is a
> source-corpus input only; it must not become a second planning authority or
> reclassify Phase 062 live-scope rows back into future-only design status.
>
> Detailed gap closure execution plan:
> `.planning/phases/062-Gaps-Closing-2/062-TODO.md`
> This is the active execution plan for `👍` sections during Phase 062.

## Locality Filter

**Included work:**

- Deterministic data models, typed IDs, registries, policy objects, and validation rules.
- Canonical codecs, digest builders, bounded parsers, and fail-closed import paths.
- Local replay ledgers, nullifier registries, lock registries, and state-transition checks.
- Wallet-local receive, export, import, reconciliation, scan, and selective disclosure flows.
- Simulator scenarios with local fixtures, local mock adapters, local mock DA, and deterministic fault injection.
- Unit tests, crate integration tests, simulator stage tests, golden vectors, property-style tests, and local benchmark harness metadata.

**Excluded work:**

- Live network services, real OnionNet transport, live DA publication, live bridge lockers, real testnet/mainnet adapters, live oracle/attestation services, external custody, production slashing, and legal or corporate disclosure workflows.
- Claims that need production throughput, external liquidity, external finality, public network diversity, or real operator behavior.

**Every implementation package below uses this contract:**

- Goal: what the phase must build, prove, demonstrate, or make unambiguous for developers.
- Source: source whitepaper or planning sections that describe the task.
- Implementation-relevant fragments: the exact parts of the source block that developers should use for implementation, plus explicit limits where a source is context or boundary-only.
- Locality gate: why this can be implemented and verified locally.
- Implementation boundary: what is in scope and what must stay out of scope.
- Implementation tasks: concrete work items for the target crates.
- Tests and simulation: scenarios that must pass before the package is considered implemented.
- Done when: the observable local completion criteria.
- Doublecheck: explicit confirmation that the package satisfies the local-compute condition and is clear enough for developers.

## Codebase Rails

**Use these rails to avoid concept drift:**

- Put canonical protocol semantics in `z00z_core`; do not make the simulator own business rules.
- Put crypto primitives, domain separation, AEAD, KDF, commitment, and proof-facing wrappers in `z00z_crypto`; do not import or fork vendor internals outside the approved facade.
- Put authoritative asset-state paths, roots, checkpoint artifacts, replay records, and durable local indexes in `z00z_storage`.
- Put wallet-local UX state, receiver-card/payment-request validation, scan/reveal policy, tx package import/export/reconcile, and service boundaries in `z00z_wallets`.
- Put cross-crate demonstrations, local fault injection, and end-to-end evidence flows in `z00z_simulator`.
- Use `z00z_utils` for codec, IO, time, randomness, and trait-based infrastructure seams where the repository already uses them.
- Keep stable facades stable. Prefer `z00z_core::{Asset, AssetDefinition, AssetLeaf, AssetPkgWire, AssetWire, AssetError, ObjectFamily, ObjectRoleV1, ChainType}`, `z00z_crypto` public APIs, `z00z_storage::assets`, and `z00z_wallets::core::*` surfaces over reaching into implementation internals.
- Treat `SettlementStateRoot` as the live public storage root and `SettlementPath { definition_id, serial_id, terminal_id }` plus settlement proof paths as the storage-owned semantic path vocabulary. Do not commit derived metadata as authoritative state.
- `wallet sees public proofs/API only`; it must not read raw RedB, `hjmt_commit`, or backend tables directly.
- `storage-created scopes` remain storage-owned semantic truth; runtime and aggregator layers may route or orchestrate but must not own scope creation.
- `config/hjmt_runtime` stays the runtime-orchestration fixture home; only storage backend schema/default fragments are eligible to move under storage ownership.
- `deterministic local-network simulator` is the live distributed HJMT proof lane; it must drive multiple aggregator runtimes over real planner/storage/journal/proof primitives.
- `adapter-only exclusions` are limited to real transport and chain-network bindings; OpenRaft, discovery overlays, and external replicated-log wiring must not replace local simulator consensus or recovery evidence.
- Keep HJMT as the live backend; keep `backend_root`, tree ids, and old asset-only or forest wording historical or proof-local only, never public semantic authority.
- Treat `TxVerifierImpl` as a pre-broadcast verifier, not a canonical admission gate. Checkpoint/storage replay remains the local authority boundary.

## Absorbed Local Packet Contract

**Reader and post-read action:**

- The local packet reader is an internal engineer or implementation agent working on wallet, storage, and simulator closure.
- After reading this document, the reader should be able to implement the storage migration gate in phase `0`, then the local settlement closure packet in phases `9` and `13` through `21`, without consulting any bridge roadmap note; phase `27` is optional measurement only.
- The packet may be materialized as phase-local context, TODO, plan, summary, and verification artifacts, but those artifacts must point back to this document and the source specs listed in each phase.

**Authority and reuse:**

- Use live code, crate-local tests, and fresh simulator evidence before planning prose when they disagree.
- Use the moved `.planning/phases/050-state-mgmt` files as the current state-management source. They are the live workspace copy of the old `049-state-mgmt` content, so no task should depend on a missing `049` path.
- Reuse the current state-management backlog for storage-owned claim-root, checkpoint proof, validator-facing verdict, receive taxonomy, scan orchestration, nullifier transition, and simulator-closure work.
- Reuse the offline transaction backlog for the current `TxPackage` verify, report, import, receiver-publication, sender-invariant, and simulator-parity work.
- Treat the shipped Phase `047` wallet redesign, Phase `047-TODO`, and `047-11` remote-scan evidence as a baseline to preserve. Do not reopen those slices unless a new regression or missing exit criterion is reproduced.
- Create fresh local work only where no truthful standalone backlog exists yet: storage migration boundary and forest rollout, tx-history and `wallet.asset.*` convergence, wallet/storage simulator evidence, checkpoint/tamper/restart evidence, and optional proof-size measurement.

**Local closure slices:**

- Phase `0` defines the storage migration boundary through three ordered internal gates: `0.1` boundary and compatibility facade, `0.2` authority-slice closure on that facade, and `0.3` forest backend rollout as early infrastructure instead of a late measurement-only lane.
- Phase `9` closes storage-owned claim-root and checkpoint authority truth.
- Phase `13` makes unsupported receive versions explicit and fail-closed.
- Phase `14` finishes wallet scan orchestration and runtime scan status while preserving wallet-owned scan authority.
- Phase `15` hardens the live delayed-connectivity `TxPackage` verify, report, and import path.
- Phase `16` converges tx-history, owned-asset state, accepted-status evidence, and `wallet.asset.*` views.
- Phase `17` locks package hygiene, redaction, envelopes, logs, exports, backups, and report boundaries before evidence artifacts expose package material.
- Phase `18` adds request-bound inbox helper behavior only after receive, scan, and package-hygiene authority is clear; it remains advisory.
- Phase `19` defines local publication, evidence, restart, and tamper vocabulary for downstream simulator packs.
- Phase `20` proves checkpoint, theorem-style, tamper, and restart behavior locally.
- Phase `21` proves receive, import, and history behavior in simulator evidence.
- Phase `27` is optional measurement only; it may compare active storage backends, but it must not own migration work or change authority contracts.

**Execution rules:**

- Run phase `0.1` before any new storage-facing correctness or wallet work binds to current backend internals; use it to freeze one storage facade and one migration contract first.
- Run phase `0.2` immediately after `0.1` so phases `8`, `9`, `10`, `13`, `14`, `15`, `16`, `19`, `20`, and `21` consume the stable facade instead of raw backend internals.
- Start phase `0.3` as soon as `0.1` is stable and `0.2` is actively closing authority slices; it is early infrastructure work, but it must not block every remaining correctness slice.
- Run phase `9` before wallet receive/import/history evidence depends on checkpoint or claim-root truth.
- Run phase `13` before phase `14`, because scan status must be able to report unsupported versions without ambiguity.
- Run phase `14` before phase `15`, because package import and report hardening should consume one wallet scan/status vocabulary.
- Run phase `15` before phase `16`, because tx-history convergence needs a stable verify/report/import gate.
- Run phase `17` before simulator reports expose package bytes, logs, exports, backups, or forwarding bundles.
- Run phase `19` before simulator evidence phases `20` and `21`, because both evidence packs need one local publication and restart/tamper vocabulary.
- Run phase `20` after phase `9` and after the package authority in phase `15`; it may run in parallel with phase `21` once those prerequisites are stable.
- Run phase `21` after phase `16`, because simulator evidence should prove the final wallet authority model, not discover it.
- Run the forest-backend lane from phase `0` after the phase `0` boundary and after the first authority slices are consuming the stable facade; it is early infrastructure, but it is not a blocker for every later correctness slice.
- Run phase `27` after phase `0` and phase `22` when measurement work is useful; keep it optional and sidecar-only.

**Verification contract:**

- Start with targeted crate tests in the owning crate.
- Run focused simulator or local theorem tests after crate seams are stable.
- Run broad workspace verification only after the local slice has targeted evidence.
- Update planning, summary, and evidence artifacts only after the relevant local tests or simulator runs have deterministic pass/fail output.
- Recommended broad verification commands remain `cargo fmt`, `cargo clippy --all-targets --all-features`, `cargo test --all`, and `cargo doc --no-deps`.

**Do not schedule from this packet:**

- Do not create a new node-process rollout, DA-provider adapter rollout, devnet, testnet, or multi-process deployment sequence.
- Do not schedule `z00z_runtime` service-loop rollout or `z00z_rollup_node` process wiring from this packet, except for local theorem or status tests that already exist as verification surfaces.
- Do not schedule live OnionNet, governance, treasury, extension, locker, or broad ecosystem work as part of phase `0`, phase `9`, or phases `13` through `21`. Local deterministic OnionNet modeling remains separate phase `35`.
- Do not create a second wallet history plane, second scan cursor model, second checkpoint verifier, second offline package family, or second wallet-owned asset authority.
- Do not let simulator code become the owner of business rules; simulator scenarios prove crate-owned behavior after the crate seams are clear.

## Ordering Rationale And Tradeoffs

**Chosen dependency strategy:**

- Use a source-freeze and guardrail front matter first, because developers need the current state-management, offline transaction, privacy, and spec-gap truth before changing code.
- Insert a storage migration boundary before the authority slices, because the semantic storage contract is mostly correct already, while the shared physical backend is the part expected to change.
- Implement canonical authority before demos: core objects, storage-owned claim roots, checkpoint proof verification, wallet receive/import authority, package hygiene, and history convergence come before simulator evidence.
- Promote real storage-backend migration ahead of the measurement-only lane: phase `0` owns the backend trait, compatibility backend, proof-envelope contract, equivalence corpus, and early forest rollout; phase `27` stays measurement-only.
- Treat simulator phases as evidence, not rule definition. Simulator work starts only after the owning crate behavior is stable enough to prove.
- Keep measurement and recursive-proof work after checkpoint and simulator evidence, because proof-size numbers are misleading if the underlying authority contract is still moving.
- Move cross-chain, linked-liability, agentic, machine, and OnionNet expansions after settlement and wallet closure, so expansion work builds on stable local semantics instead of creating parallel authority planes.
- Keep the residual spec-gap hardening gate last, because it audits the final local plan for live-vs-future wording, version drift, secret-boundary risk, and research-only terms.

**Pros of this order:**

- It matches dependency direction: source truth -> storage migration boundary -> crate authority -> wallet/package closure -> simulator evidence -> measurement -> expansion.
- It reduces concept drift by preventing broad whitepaper domains from being implemented before current wallet/storage truth is stable.
- It avoids writing new wallet and checkpoint work directly against the current shared-JMT internals when the repository already expects a storage backend transition.
- It makes local verification incremental: every simulator phase can point to lower-level crate tests instead of discovering rules inside simulator code.
- It gives parallel teams clear split points after the phase `0` facade and after the core authority path is stable.

**Cons of this order:**

- It is less aligned with the original whitepaper narrative order, so readers must use this table as the implementation order rather than the document-source order.
- It front-loads storage abstraction work that does not immediately produce user-visible simulator demos.
- It delays visible cross-chain, agentic, machine, and OnionNet demos until after lower-level closure work.
- It front-loads several source/spec phases that are not large code phases, but they prevent later rework and ambiguous implementation ownership.

**Rejected ordering alternatives:**

- Whitepaper-domain order first: easier to read against the source papers, but it mixes future architecture with current local implementation and leaves concrete wallet/storage closure too late.
- Demo-first expansion order: produces cross-chain, voucher, machine, or agent scenarios earlier, but risks building them on unstable package, history, replay, and checkpoint semantics.
- Recursive/measurement-first order: can produce interesting proof-size data early, but those numbers are low-value until checkpoint authority, artifact codecs, simulator evidence, and the storage migration facade are stable.
- Full forest rewrite before authority closure: it attacks the right bottleneck, but it risks freezing correctness work until the whole backend move lands; phase `0` therefore stages boundary first, authority slices second, and forest rollout third.

**Parallelism rule:**

- The table below is the recommended single-team serial order. Parallel teams may split after phase `0` if they preserve the dependency gates: authority phases must consume the stable storage facade, forest-backend work must stay behind that boundary, wallet phases must not outrun storage truth, simulator phases must not define crate behavior, and measurement or expansion phases must not change canonical authority contracts.

## Phase Order

**Goal:**

- Provide the execution order for all local phases and point each row to a detailed phase section whose first block states what that phase must build, prove, demonstrate, and make unambiguous.
- Keep this table as the dependency map; use each numbered section's `**Goal:**` block as the implementation intent before reading `Source`, boundaries, tasks, and tests.

| # | Order | Primary crates | Dependency reason |
| ---: | --- | --- | --- |
| 0 | Storage Migration Boundary, Authority Facade, And Forest Backend Rollout | `z00z_core`, `z00z_crypto`, `z00z_storage`, `z00z_wallets`, `z00z_simulator` | Execute three ordered gates: `0.1` freeze one storage facade and compatibility contract, `0.2` force authority slices onto that facade, and `0.3` land the forest backend early without turning it into a blocker for every correctness slice. |
| 1 | State Management Current Spec | `z00z_storage`, `z00z_wallets`, `z00z_simulator` | Freeze the current moved state-management source before any storage, wallet, or simulator work uses old phase paths or stale assumptions. |
| 2 | State Management Execution Backlog | `z00z_storage`, `z00z_wallets`, `z00z_simulator` | Convert the state-management source into the concrete local checklist for claim roots, checkpoint verification, scan orchestration, nullifier transitions, and simulator closure. |
| 3 | Offline Transaction Current Spec | `z00z_wallets`, `z00z_storage`, `z00z_simulator` | Freeze delayed-connectivity package semantics before wallet package verification, reporting, import readiness, or history work changes. |
| 4 | Offline Transaction Execution Backlog | `z00z_wallets`, `z00z_simulator` | Convert offline transaction semantics into the executable checklist for the package verifier, receiver boundary, sender invariants, import gate, and simulator parity. |
| 5 | Privacy And Anonymity Boundary | `z00z_crypto`, `z00z_wallets`, `z00z_simulator` | Establish current privacy truth before receive, pack, package hygiene, inbox helper, OnionNet, or disclosure work can make privacy claims. |
| 6 | Spec-Gap Normalization Register | `z00z_core`, `z00z_crypto`, `z00z_storage`, `z00z_wallets`, `z00z_simulator` | Inventory future terms, live version drift, secret APIs, and metadata caveats early so implementation does not accidentally promote research notes into protocol truth. |
| 7 | Cross-Crate Test Matrix | `z00z_core`, `z00z_crypto`, `z00z_storage`, `z00z_wallets`, `z00z_simulator` | Define test ownership before coding so simulator evidence cannot become the first place business rules are specified. |
| 8 | Core Object, Claim-Root, And Checkpoint Authority Hardening | `z00z_core`, `z00z_storage`, `z00z_wallets`, `z00z_simulator` | All local implementation depends on canonical object boundaries, digest stability, replay safety, honest roots, and checkpoint authority. |
| 9 | Storage Claim-Root And Checkpoint Authority Closure | `z00z_storage`, `z00z_simulator` | Land the concrete storage authority slice on top of the phase `0` facade before wallet, publication, simulator, or recursive phases depend on checkpoint truth. |
| 10 | Wallet Receive, Scan, Import, And History Authority Closure | `z00z_wallets`, `z00z_storage`, `z00z_simulator` | Establish the wallet authority model against the stable storage facade before individual receive, scan, package, history, and reconciliation slices are tightened. |
| 11 | Field-Native Pack Migration Plan | `z00z_crypto`, `z00z_wallets`, `z00z_simulator` | Freeze current `ZkPack_v1` behavior before receive, scan, package, and privacy work can accidentally imply field-native parity. |
| 12 | Privacy, Stealth, And Selective Disclosure Primitives | `z00z_crypto`, `z00z_wallets`, `z00z_simulator` | Local receive and package behavior depends on honest owner-tag, prefilter, AEAD/AAD, reveal, redaction, and scoped disclosure semantics. |
| 13 | Unsupported Receive-Version Taxonomy | `z00z_wallets` | Make unsupported input explicit before scan status, package report/import, and simulator evidence publish ambiguous ownership classes. |
| 14 | Wallet Scan Orchestration And Runtime Scan Status | `z00z_wallets`, `z00z_simulator` | Finish one scan lane and one cursor/status model before package import and history convergence consume scan results. |
| 15 | Offline `TxPackage` Verify, Report, And Import Hardening | `z00z_wallets`, `z00z_simulator` | Harden one package verifier and one import-readiness vocabulary before tx-history, simulator evidence, and publication traces depend on package outcomes. |
| 16 | Tx-History And `wallet.asset.*` Authority Convergence | `z00z_wallets` | Close wallet history and asset-view authority before simulator evidence proves receive, import, conflict, and restart behavior. |
| 17 | Package Hygiene And Transport Privacy Plan | `z00z_crypto`, `z00z_wallets`, `z00z_simulator` | Redaction, envelope, export, backup, log, and report hygiene must be in place before simulator reports or transport-adjacent helpers handle package bytes. |
| 18 | Request-Bound Inbox Helper Plan | `z00z_wallets`, `z00z_crypto`, `z00z_simulator` | Helper routing can be added only after request-bound receive, scan authority, and package hygiene are clear; it remains advisory and off-consensus. |
| 19 | Local Publication, Simulator Evidence, And Restart/Tamper Harness | `z00z_storage`, `z00z_wallets`, `z00z_simulator` | Define the local evidence vocabulary before checkpoint and wallet simulator packs emit publication, verdict, restart, or tamper artifacts. |
| 20 | Simulator Checkpoint, Theorem, Tamper, And Restart Evidence Pack | `z00z_storage`, `z00z_simulator` | Prove storage/checkpoint authority under tamper and restart after storage truth and package authority are stable. |
| 21 | Simulator Receive, Import, And History Evidence Pack | `z00z_simulator`, `z00z_wallets` | Prove wallet receive/import/history behavior after package hardening, scan status, and asset/history convergence are stable. |
| 22 | Benchmark, Proof-Size, And Evidence Guardrails | `z00z_core`, `z00z_crypto`, `z00z_storage`, `z00z_wallets`, `z00z_simulator` | Add local evidence metadata and non-claim rules after phase `0` owns storage migration, so measurement and benchmark reports cannot masquerade as migration or production evidence. |
| 23 | Recursive-Proof Spike Boundary | `z00z_core`, `z00z_crypto`, `z00z_storage`, `z00z_simulator` | Bound recursive work to one local checkpoint/state-transition statement before selecting or modeling any backend. |
| 24 | Nova Or SuperNova State-Transition Note | `z00z_core`, `z00z_storage`, `z00z_simulator` | Translate recursive backend notes into backend-neutral transition semantics before implementation. |
| 25 | Recursive State Proof Research Register | `z00z_core`, `z00z_crypto`, `z00z_storage`, `z00z_simulator` | Mine research material for local failure cases and measurement questions before the recursive spike consumes it. |
| 26 | Recursive-Proof Statement Spike And Proof-Size Guardrails | `z00z_core`, `z00z_crypto`, `z00z_storage`, `z00z_simulator` | Implement the narrow local recursive statement only after checkpoint authority, simulator evidence, recursive boundaries, and the phase `0` storage facade are clear. |
| 27 | Optional Proof-Size And Storage Measurement Sidecar | `z00z_core`, `z00z_storage`, `z00z_wallets`, `z00z_simulator` | Measure proof and storage behavior after stable authority and benchmark guardrails exist; keep it optional and non-authoritative. |
| 28 | Multi-Asset Families, Trust Tiers, And Internal Asset Phase | `z00z_core`, `z00z_storage`, `z00z_wallets` | Expansion work needs stable core assets, wallet authority, privacy, package, and evidence boundaries before new asset-family semantics matter. |
| 29 | Local Adapter Model For Cross-Chain Inputs Without Live Chains | `z00z_core`, `z00z_storage`, `z00z_simulator` | Mock adapter semantics depend on asset-family identity and local publication/replay vocabulary. |
| 30 | Voucher, Payroll, B2B, And Useful-Work Claim Scenarios | `z00z_core`, `z00z_wallets`, `z00z_simulator` | Use-case scenarios depend on multi-asset policy, claim package authority, wallet import semantics, and local simulator evidence patterns. |
| 31 | Linked Liability MVP: Domain, Commitment, Fraud Proof, Lock Registry | `z00z_core`, `z00z_storage`, `z00z_wallets`, `z00z_simulator` | Liability locks depend on stable package, replay, conflict, wallet authority, and local evidence semantics. |
| 32 | Fee Envelope And Rights Wallet Extensions | `z00z_core`, `z00z_wallets`, `z00z_simulator` | Agentic and machine rights need deterministic fee accounting and rights-wallet inventory before scenario expansion. |
| 33 | Agentic Rights Local Simulations | `z00z_core`, `z00z_wallets`, `z00z_simulator` | Pure software agent-right flows can validate fee envelopes, escrow, payout, and audit before physical-machine scenarios add domain complexity. |
| 34 | Machine Capability Local Simulations | `z00z_core`, `z00z_wallets`, `z00z_simulator` | Machine capability scenarios build on rights, fees, liability, receipts, and simulator actors established earlier. |
| 35 | OnionNet Deterministic Control Plane And Packet Discipline | `z00z_core`, `z00z_crypto`, `z00z_storage`, `z00z_wallets`, `z00z_simulator` | Local transport-control work depends on privacy truth and package hygiene but stays separate from settlement, wallet, and expansion authority. |
| 36 | Spec-Gap Normalization And Residual Hardening Gate | `z00z_core`, `z00z_crypto`, `z00z_storage`, `z00z_wallets`, `z00z_simulator` | Final pass checks that all implemented, deferred, research, and future-only terms remain correctly classified after the local lanes have evidence. |

## 6. Spec-Gap Normalization Register

**Goal:**

- Turn the spec-gap register into a concrete local normalization and hardening checklist for future-backend terms, live versions, secret APIs, wallet ID caveats, metadata privacy, salt compatibility, and mnemonic boundaries.
- Prove every register item is implemented locally, mapped to another phase, explicitly deferred, or explicitly out of scope.

**Source:**

- [Spec-gap normalization register](../.planning/phases/087-spec-gaps/087-TODO.md)

**Implementation-relevant fragments:**

- Use the register paragraphs that name `Poseidon2`, Halo2 as spelled `holo2` in the register, STARK/FRI as spelled `STAR/FRI` in the register, abbreviation inventory, `V1`/`V2` cleanup, `reveal_receiver_secret`, `wallet_id`, backup metadata privacy, salt compatibility, mnemonic boundary, `CipherSeedContainer`, and `derive_s_out`.
- Treat every register item as one of: implement locally, map to another phase, explicitly defer, or explicitly out of scope.
- Do not treat mentioned research systems as current protocol merely because they appear in the register.

**Locality gate:**

- The register can be closed through local source audits, API visibility changes, docs normalization, version cleanup, and targeted tests.
- No external security audit, testnet, network run, or production migration is required.

**Implementation boundary:**

- In scope: audit of mentioned-but-unfinished topics, future proof backend names, live version cleanup, `reveal_receiver_secret` hardening, wallet ID caveat, export/backup metadata privacy, salt compatibility wording, mnemonic boundary wording, and code/doc terminology alignment.
- Out of scope: completing every research idea, moving old code without a concrete owner, proving wallet ID offline-impact mathematically resolved, or implementing new proof systems solely because they are mentioned.

**Implementation tasks:**

1. Build an abbreviation and future-backend inventory from docs for Poseidon2, Halo2, STARK/FRI, Nova, SuperNova, Plonky2, Plonky3, PQ, PCD, IVC, HJMT, JMT, DA, OWF, AAD, AEAD, KDF, and related terms.
2. Label each term as live, compatibility, spike, research, future, or not-in-use.
3. Move unused or historical version wording into compatibility/not-in-use documentation instead of leaving it as active implementation instruction.
4. Audit public secret-reveal helpers and narrow `reveal_receiver_secret` or equivalent surfaces when they are test/simulator-only.
5. Add tests proving export and backup public metadata do not leak mnemonic text, receiver secrets, plaintext packs, or private chain/network data beyond explicit policy.
6. Add threat-model wording for public `wallet_id`: mitigation through random new-write salt may exist, but material offline-impact remains an explicit question unless proven.
7. Keep legacy salt behavior as compatibility/export contract where required and prevent reuse for new wallet or backup policy.
8. Keep `CipherSeedContainer` wrapper decisions evidence-based: do not add a second wrapper unless a migration plan proves it changes safety.
9. Correct docs that imply Poseidon2, Halo2, STARK/FRI, Nova, SuperNova, Plonky2, Plonky3, field-native packs, or post-quantum recursion are current protocol truth.
10. Add a final drift check that every register item is linked to a local plan section, explicitly deferred, or explicitly out of scope.

**Tests and simulation:**

- Abbreviation inventory check proves every future/backend term has a status label.
- API visibility tests prove secret-reveal helpers are unavailable to production-facing modules unless explicitly capability-gated.
- Export/backup metadata tests for public header redaction, encrypted payload containment, wrong AAD, wrong purpose, wrong wallet ID, and wrong chain/network policy.
- Version cleanup tests or doc checks proving only working versions are described as live.
- Compatibility tests proving legacy salt/export behavior remains supported where required but is not reused as new-write policy.
- Drift check proving speculative proof and field-native terms are not used as active implementation instructions.

**Done when:**

- Every item in the spec-gap register is either implemented locally, mapped to a local plan section, explicitly deferred, or explicitly out of scope.
- Secret-boundary and metadata-risk items have tests.
- Future proof/backend terminology cannot be confused with live protocol.
- `Z00Z-LOCAL-PLANS.md` is self-contained enough that the old bridge note can be removed without losing these local tasks.

**Doublecheck:**

- Local condition: satisfied. The work is local audit, local tests, local docs normalization, and API hardening.
- Developer clarity: satisfied. Register topics, target crates, test categories, and completion criteria are explicit.

## 👍 36. Spec-Gap Normalization And Residual Hardening Gate

Closeout status: Bounded closed

**Canonical closeout register:**

| Term or lane | Classification | Current canonical note |
| --- | --- | --- |
| `SettlementStateRoot` / `SettlementPath` public storage authority | `Live` | Section `0` is closed on the settlement-root and settlement-path vocabulary only. |
| `AssetStateRoot` / `AssetPath` as live public authority | `Compatibility-only` | Archived or historical compatibility vocabulary only; do not revive as live authority claims. |
| local deterministic HJMT and wallet-node simulation | `Simulation-only` | Local simulator proof remains the truthful bounded path; real external transport is not implied. |
| real transport, real node process, and remote publication bindings | `Adapter-only` | External deployment adapters remain outside live local-closure claims. |
| live OnionNet transport anonymity and live field-native pack parity claims | `Removed-claim` | Wallet/docs keep these terms explicitly non-live unless later implementation lands with owner tests. |

**Residual gap register:**

| Residual | Classification | Current status note |
| --- | --- | --- |
| Recursive proof backend | `Research only` | Local checkpoint and simulator evidence are live; no truthful recursive-proof backend is implemented on the current tree. |
| Linked Liability | `Deferred future implementation` | Liability objects or schemas may be prototyped locally, but no end-to-end fraud-proof or lock-registry enforcement path is live. |
| OnionNet | `Deferred future implementation` | Wallet docs and app stubs keep OnionNet explicitly planned or placeholder-only; no live transport-anonymity claim is allowed. |
| live external DA | `Out of scope` | Phase 062 keeps external DA publication adapter-only and outside live bounded authority. |
| live cross-chain bridge | `Out of scope` | Cross-chain settlement remains excluded from the current bounded local-closeout packet. |
| field-native / Poseidon2 pack parity | `Deferred future implementation` | Section `11` closes the live `ZkPack_v1` AEAD path and rejects any present-tense parity claim. |
| useful-work scenario | `Deferred future implementation` | Voucher and rights local scenarios are live; useful-work attestation stays outside the current truthful authority path. |

**Final closeout summary:**

- Closed sections: `0`, `11`, `22`, and `27` now carry explicit live-closeout evidence on the current tree.
- Bounded-closed sections: `28`, `29`, `30`, `32`, `33`, `34`, and this `36` gate are closed with explicit local-only or adapter-bounded limits.
- Deferred sections: recursive-proof expansion, linked-liability enforcement, OnionNet transport rollout, field-native pack parity, and useful-work attestation remain explicitly deferred instead of implied as live.
- Validation commands run for this gate are the mandatory bootstrap gate, focused release guardrail suites, the broad `cargo test --release` rerun, scoped drift grep, and the Phase 062 manual review loop.
- Unresolved execution lanes are phase-tracked in `PLAN-062-G23` through `PLAN-062-G27`; they remain separate queued work after this residual-hardening closeout and do not reopen the documentation normalization gate.

## Production-grade bounded components 

Реалистичный ответ такой: ты можешь писать production-grade bounded components и гонять их на fixtures, golden corpus и simulator-сценариях, если держишься за живую authority-chain: wallet package → runtime publication → checkpoint → storage root. Полное понимание всей сети не нужно, но понимание этой оси обязательно, иначе почти гарантирован concept drift.

| Документ                                     | Вердикт                                       | Что можно честно делать локально                             | Почему                                                       |
| -------------------------------------------- | --------------------------------------------- | ------------------------------------------------------------ | ------------------------------------------------------------ |
| Z00Z-Compact-Delta-Log.md                    | `Частично да`                                 | spec/codec/receipt-normalization/golden fixtures             | Сам документ прямо говорит, что CDL, `normalized_tx`, `canonical_tx`, `tx_receipt` и strict archive пока target architecture, а не finished pipeline |
| Z00Z-Thin-Transaction-Mode.md                | `Да, лучший кандидат`                         | signed-index DTO, thin/thick fallback rules, helper-side expansion validator, equivocation/stale-index tests | Документ явно опирается на уже живые `TxPackage`, `ClaimTxPackage`, checkpoint/runtime surface; это хороший bounded transport slice |
| Z00Z-Smart-Cash-Whitepaper.md                | `Да, но узко`                                 | bounded-right objects, local policy evaluators, settlement predicates, fixtures на object transitions | Сам paper прямо отрезает “universal private VM” и оставляет только bounded predicates/right objects |
| Z00Z-Privacy-Threat-Model-Whitepaper.md      | `Да`                                          | metrics, QA harnesses, simulator audits, privacy regression fixtures | Это в основном measurement/QA/governance paper, а не новый consensus plane |
| Z00Z-Multi-DA-and-Checkpoint-Architecture.md | `Частично да`                                 | anchor/commitment/proof envelope models, verifier stubs, fixture corpus | Полный multi-DA topology и settlement semantics требуют большего понимания checkpoint/storage boundary |
| Z00Z-Linked-Liability-Whitepaper.md          | `Только частично`                             | liability objects, codecs, fraud-claim schemas, lock-state specs, test vectors | По live repo нет признаков, что полный `FraudProof`/`LockRegistry`/slashing enforcement path уже landed end-to-end |
| Z00Z-OnionNet-Whitepaper.md                  | `Нет как full impl, да как bounded prototype` | packet formats, route descriptors, replay filters, crypto harnesses | Live crate прямо помечен как placeholder в README.md         |

Живая опора в репо у тебя есть. Thin-mode и smart-cash не висят в воздухе: в коде уже есть `TxPackage` в `crates/z00z_wallets/src/tx/tx_wire.rs`, `ClaimTxPackage` в `crates/z00z_wallets/src/tx/claim_tx_wire.rs`, `SettlementTheorem` в `crates/z00z_rollup_node/src/lib.rs`, а runtime publication vocabulary живет в `crates/z00z_runtime/aggregators/src/types.rs`. Для fixture/scenario validation у тебя уже есть нормальный harness: z00z_simulator/README.md прямо говорит, что `z00z_simulator` предназначен для reproducible scenario flows, deterministic fixture wiring и release-style validation. Это и есть правильная база для production-grade bounded implementation.

Самая честная формула такая: не надо понимать “всю архитектуру testnet blockchain”, но надо понимать четыре вещи. Первое: кто владеет authority. Второе: что считается live settlement boundary. Третье: где helper/DA/onion/privacy layer только обслуживает, а не принимает решение. Четвертое: что в paper написано как target direction, а не как landed protocol. Без этого ты сможешь написать хороший код, но высок риск, что он будет красивый и тестируемый, но архитектурно ложный.

Если идти practically, я бы делил так:

1. Сразу делать: thin mode, privacy metrics, bounded smart-cash predicates, CDL codecs/receipts.
2. Делать только как constrained prototype: multi-DA verifier envelopes, OnionNet packet/session/replay pieces.
3. Пока не обещать как full production protocol: full OnionNet network plane, linked-liability enforcement loop, full generalized smart-contract/privacy VM widening.

Для проверки “production-grade + fixtures” у тебя уже есть нормальные поверхности. Минимум для simulator:

```bash
cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_dump
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump
```

И отдельно у storage есть golden/checkpoint suites, описанные в README.MD. Я их в этой сессии не запускал, так что это именно repo-backed assessment, а не claim о passing state.

Разложить все 7 whitepaper’ов на `можно сейчас / можно как prototype / нельзя честно claimить` в виде жёсткой implementation-matrix.

## Ниже жёсткая implementation-matrix без “может быть”. 

Критерий такой:  
“можно сейчас” = можно писать production-grade bounded code, который опирается на уже живые repo-authority surfaces и нормально тестируется на fixtures/golden/scenario.  
“можно как prototype” = можно делать сильный локальный прототип или spec-driven module, но нельзя выдавать за landed protocol capability.  
“нельзя честно claimить” = в текущем состоянии репо и corpus это будет архитектурным враньём.

| Whitepaper                                        | Можно сейчас                                                 | Можно как prototype                                          | Нельзя честно claimить                                       |
| ------------------------------------------------- | ------------------------------------------------------------ | ------------------------------------------------------------ | ------------------------------------------------------------ |
| docs/Z00Z-Compact-Delta-Log.md                    | Да, как receipt/codec/normalization/golden-corpus slice      | Да, как future CDL pipeline mock                             | Нельзя claimить finished canonical receipt pipeline, strict archive, full bridge-verifier path |
| docs/Z00Z-Linked-Liability-Whitepaper.md          | Нет                                                          | Да, как liability object model, fraud-claim schema, lock-state simulator | Нельзя claimить live full FraudProof, LockRegistry, slashing/quarantine enforcement loop |
| docs/Z00Z-Multi-DA-and-Checkpoint-Architecture.md | Нет                                                          | Да, как anchor/DA-commitment/ZTS-proof envelopes, verifier fixtures, compatibility corpus | Нельзя claimить full multi-DA production topology, real provider failover authority, complete settlement-grade DA orchestration |
| docs/Z00Z-OnionNet-Whitepaper.md                  | Нет                                                          | Да, как packet/path/session/replay-filter/privacy-harness prototype | Нельзя claimить full OnionNet implementation, mature overlay network, production anonymity plane |
| docs/Z00Z-Privacy-Threat-Model-Whitepaper.md      | Да, как metrics, audits, simulator privacy regressions, wallet QA | Да, как richer scoring and red-team harnesses                | Нельзя claimить that this paper itself delivers a new consensus/runtime capability |
| docs/Z00Z-Smart-Cash-Whitepaper.md                | Да, как bounded-rights / bounded-predicate settlement objects | Да, как wider object families around lockers/vouchers/agent rights | Нельзя claimить universal private VM, arbitrary hidden smart-contract execution, generalized private compute platform |
| docs/Z00Z-Thin-Transaction-Mode.md                | Да, лучший кандидат                                          | Да, как helper-index and expansion lane with strict fallback | Нельзя claimить that thin mode replaces thick mode or moves authority out of wallet/checkpoint boundary |

Жёсткая группировка по трём корзинам:

| Категория                                    | Whitepapers                                                  |
| -------------------------------------------- | ------------------------------------------------------------ |
| Можно сейчас                                 | docs/Z00Z-Compact-Delta-Log.md, docs/Z00Z-Privacy-Threat-Model-Whitepaper.md, docs/Z00Z-Smart-Cash-Whitepaper.md, docs/Z00Z-Thin-Transaction-Mode.md |
| Можно как prototype                          | docs/Z00Z-Linked-Liability-Whitepaper.md, docs/Z00Z-Multi-DA-and-Checkpoint-Architecture.md, docs/Z00Z-OnionNet-Whitepaper.md |
| Нельзя честно claimить как landed capability | Linked Liability end-to-end enforcement, Multi-DA full production plane, OnionNet full live network, Smart Cash as universal private VM, Thin Mode as authority replacement, CDL as already-finished canonical receipt/archive system |

Самые сильные “можно сейчас” опоры в live repo:

- Thin/Smart-Cash опираются на уже живые package и settlement surfaces: `crates/z00z_wallets/src/tx/tx_wire.rs`, `crates/z00z_wallets/src/tx/claim_tx_wire.rs`, `crates/z00z_rollup_node/src/lib.rs`, `crates/z00z_runtime/aggregators/src/types.rs`.
- Fixture/scenario база уже есть в crates/z00z_simulator/README.md и storage test surfaces из crates/z00z_storage/src/assets/README.MD.
- OnionNet прямо сейчас нельзя переоценивать, потому что boundary помечен как placeholder в crates/z00z_networks/onionnet/README.md.

Практический вывод:

- Если цель “сейчас писать production-grade код и тестировать на fixtures”, бери сначала Thin Transaction Mode и bounded slice из Smart Cash.
- Если цель “собрать сильный R&D prototype”, бери Multi-DA envelopes, Linked Liability schemas или OnionNet packet/session model.
