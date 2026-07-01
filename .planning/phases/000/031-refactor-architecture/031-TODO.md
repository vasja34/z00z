---
post_title: "Z00Z Crate Architecture Review"
author1: "GitHub Copilot"
post_slug: "z00z-crate-architect-review"
microsoft_alias: "copilot"
featured_image: "N/A"
categories:
  - "Architecture"
tags:
  - "rust"
  - "architecture"
  - "blockchain"
  - "review"
ai_note: "AI-assisted architectural review artifact based on repository code and docs."
summary: "Architecture review for z00z_core, z00z_crypto, z00z_networks, z00z_simulator, z00z_storage, z00z_utils, and z00z_wallets with prioritized risks, trade-offs, and validation steps."
post_date: "2026-04-01"
---

<!-- markdownlint-disable MD041 -->

## 🎯 Scope And Decision Context

This review covers the current architecture of `z00z_core`, `z00z_crypto`,
`z00z_networks`, `z00z_simulator`, `z00z_storage`, `z00z_utils`, and
`z00z_wallets`.

The decision context is not whether the code compiles today. The real
question is whether these crate boundaries will remain stable, testable,
operable, and extensible as Z00Z moves from scaffolded phases into production
behavior.

The review is evidence-based. Findings are grounded in crate manifests,
crate root modules, README files, and directly visible boundary code such as
facade exports, transport traits, placeholder seams, and dependency edges.

## ⚙️ Goals And Constraints

The primary quality drivers are maintainability, boundary clarity, vendor
isolation, deterministic behavior, testability, and safe evolution of public
APIs.

The strongest architectural constraint is the Design Foundation rule that
business crates should route low-level concerns through `z00z_utils` instead of
spreading direct filesystem, serialization, time, RNG, and logging policy.

A second hard constraint is vendor isolation around Tari. The codebase says
that vendor code under `z00z_crypto/tari/` is read-only and that the rest of
the system should depend on Z00Z-owned abstractions rather than on Tari-owned
types and semantics.

A third constraint is phase asymmetry. Some crates already carry production
logic, while others are intentionally scaffolded or stub-heavy. That is an
acceptable temporary state only if placeholder seams are narrow, explicit, and
hard to accidentally treat as finished contracts.

A fourth hard constraint for this review is structural stability of the
repository. Recommendations should harden boundaries, exports, ownership, and
internal seams within the current project layout rather than propose crate or
package reshuffles.

Find the version suffixes *V1..*VN, *v1..*vN in the signatures (N is the number) throughout the codebase.
This will retain only the working version and remove all legacy versions.

After removing all non-canonical versions, remove all suffixes *V1..*VN, *v1..*vN from the remaining signatures.

The initial analysis already shows that technically, deep import paths have been almost completely cleared out, but there is no complete ban on the shim/alias layer: public compatible re-exports remain in the facades, and in some places, even explicit compatibility/legacy markings are present.
Fix this to strict criteria.

## 🏗️ Architecture Map

The current structure looks like this under the dependency and boundary lens.

| Crate | Current Architectural Role | Boundary Quality | Highest Structural Risk |
| --- | --- | --- | --- |
| `z00z_core` | Consensus and asset-domain core | Moderate | Broad root facade and tooling mixed into core package |
| `z00z_crypto` | Cryptographic boundary and vendor adapter | Moderate | Tari concrete types leak through the public facade |
| `z00z_networks` | Intended networking namespace | Weak | Not a complete crate boundary yet; only `rpc` exists |
| `z00z_simulator` | End-to-end orchestration and scenario harness | Moderate | Integration hub couples directly to nearly every domain crate |
| `z00z_storage` | State roots, proofs, persistence mechanics | Moderate | Consensus and backend concerns live in one package |
| `z00z_utils` | Cross-cutting abstraction layer | Strong | Risk of becoming a policy megacrate |
| `z00z_wallets` | Wallet domain, adapters, DB, UI, WASM, RPC | Weak to moderate | Stub-heavy service facade is exposed as if it were stable |

The good news is that the repository is converging around a recognizable
layering model: `z00z_utils` as the cross-cutting foundation, `z00z_core` and
`z00z_crypto` as domain primitives, `z00z_storage` as state machinery,
`z00z_wallets` as application-facing orchestration, and `z00z_simulator` as
the integration harness.

The main architectural problem is maturity mismatch across boundaries.
Several crates present clean package-level stories, but some of the exported
surfaces are still broader, more placeholder-driven, or more vendor-coupled
than the stated architecture implies.

## 🚨 Cross-Crate Findings Ordered By Severity

- `High | Confirmed risk`: Stable-looking root facades are broader than the
actual stable contracts. `z00z_core` re-exports `assets::*`, `z00z_crypto`
re-exports many Tari concrete types and services, and `z00z_wallets` re-exports
service and core domains while explicitly documenting that large parts of the
stack are still stub-heavy. This creates a contract-governance problem: callers
can bind to unstable or placeholder semantics long before the project wants
them to.

- `High | Confirmed risk`: `z00z_networks` is currently a namespace rather than
a complete crate boundary. Only `crates/z00z_networks/rpc/Cargo.toml` exists,
while `crates/z00z_networks/onionnet/` is empty. That means the repository has
already committed to a conceptual network layer without a coherent package-level
ownership model for transport, identity, retries, peer state, or failure
containment.

- `High | Likely risk`: Simulator and wallet orchestration are becoming the
coupling amplifiers of the workspace. `z00z_simulator` depends directly on
`z00z_core`, `z00z_crypto`, `z00z_networks_rpc`, `z00z_storage`,
`z00z_utils`, and `z00z_wallets`, while `z00z_wallets` itself spans core,
services, adapters, persistence, desktop UI, and WASM. Without tighter public
contracts, future change cost will concentrate in those two packages.

- `Medium | Confirmed risk`: The repository has a clean policy statement about
`z00z_utils`, but not all crates are equally strict about separating wire
formats, tool-facing helpers, and domain surfaces. The direction is right; the
boundary discipline is not yet uniform.

## 🔍 Detailed Review By Crate

### 🧩 `z00z_core`

`z00z_core` is the domain-heavy package that owns assets, genesis, domains,
and hashing, and it depends on `z00z_crypto` and `z00z_utils`.

The architecture intent is coherent. The crate README and root module say the
package is responsible for core protocol behavior, deterministic genesis, and
asset configuration. That is the correct shape for a blockchain core package.

#### 📌 `z00z_core` Boundary Summary

- `src/lib.rs` exposes four main modules: `assets`, `domains`, `genesis`, and
  `hashing`.
- The root also performs a very broad re-export through `pub use assets::*`.
- `Cargo.toml` enables configuration, export, and CLI-oriented features inside
  the same package that also owns consensus-facing domain logic.
- The README emphasizes YAML-driven genesis and asset configuration and compile-
  time embedding for reproducibility.

#### 🚨 `z00z_core` Findings Ordered By Severity

- `High | Confirmed risk`: The root facade is broader than the domain boundary.
  `pub use assets::*` makes the entire asset module tree a de facto public API.
  That weakens contract control and makes internal refactors expensive.

- `Medium | Likely risk`: Consensus logic, genesis configuration loading,
  artifact export, and optional CLI concerns live in one crate. That is not yet
  a failure, but it increases the chance that deployment tooling and consensus
  policy will evolve together instead of through separate seams.

- `Low | Needs validation`: Compile-time embedding of configuration is strong
  for determinism and auditability, but it can become operationally rigid if
  network rollout needs faster config-only evolution than full binary rebuilds.

#### ⚖️ `z00z_core` Trade-Offs And Rationale

The crate chooses pragmatic consolidation over early structural
decomposition. That avoids the complexity of fragmenting the current core
surface before the public contract is better understood.

The cost of that choice is API sprawl. Broad root exports are cheap early and
expensive later because they silently turn implementation details into external
dependencies.

#### ✅ `z00z_core` Recommended Changes

- Replace `pub use assets::*` with an explicit curated export list.
- Keep consensus-domain exports at the crate root and keep tooling or
  generation helpers behind their owning submodules instead of re-exporting
  them as peer root-surface contracts.
- Keep genesis and configuration loaders off the broad root facade; expose them
  only through narrow, named module entrypoints.

#### ❓ `z00z_core` Validation Plan And Open Questions

- Run a workspace usage audit for `z00z_core::*` imports and measure how much of
  the current surface is actually needed. ANS: YES
- Check whether production code depends on any type that only exists because of
  the broad `assets::*` re-export. ANS: YES
- Open question: should network-specific or deployment-specific genesis loading
  remain in this crate, or become a higher-level composition concern. ANS: higher-level

### 🔐 `z00z_crypto`

`z00z_crypto` is intended to be the cryptographic abstraction boundary, and
the crate says that backend choice should remain hidden behind Z00Z-owned APIs.

The crate has strong internal organization around claims, commitments, hash
policy, range proofs, AEAD, KDF, and secret handling. It also keeps backend
modules private with `backend` and `backend_tari`.

#### 📌 `z00z_crypto` Boundary Summary

- The root declares many crypto submodules and private backend modules.
- The crate README explicitly says the backend should be hidden.
- The root facade re-exports large parts of Tari, including commitment
  factories, range proof services, Ristretto types, signature types, and
  utility traits.
- The crate README also assigns one production sealing path to a wallet facade,
  which splits production ownership between `z00z_crypto` and `z00z_wallets`.

#### 🚨 `z00z_crypto` Findings Ordered By Severity

- `Critical | Confirmed risk`: Vendor isolation is partial and inconsistent.
  Some Tari types are directly re-exported at the crate root, including
  `PedersenCommitmentFactory`, `CommitmentSignature`, and
  `DiffieHellmanSharedSecret`, while others are exposed through Z00Z-named type
  aliases. Neither approach creates real backend isolation, and downstream code
  can still couple to Tari semantics even though the package documentation says
  backend choice is abstracted.

- `Medium | Needs validation`: Production ownership may be ambiguous for
  encrypted pack handling. The crypto crate says the stable wire contract is
  `ZkPackEncrypted`, but its documentation also points to a wallet facade as the
  blessed production seal/open path. That looks like a cross-crate ownership
  question and should be clarified before more encryption behavior accumulates
  on top of it.

- `Medium | Likely risk`: The root facade is overloaded. Claims, hashes, KDF,
  commitments, AEAD, experimental features, and Tari passthroughs all converge
  at the crate root. This makes API governance and deprecation discipline much
  harder than it needs to be.

#### ⚖️ `z00z_crypto` Trade-Offs And Rationale

Re-exporting Tari is convenient because it avoids writing thin wrappers for
every primitive and keeps integration cost low in the short term.

The cost is strategic lock-in. A convenient passthrough facade is almost the
same as no isolation once downstream code starts naming vendor types.

#### ✅ `z00z_crypto` Recommended Changes

- Introduce a clear in-crate separation between `stable facade`,
  `expert facade`, and `vendor passthrough` namespaces.
- Remove direct Tari concrete type re-exports from the root facade wherever a
  Z00Z-owned alias or wrapper already exists, and confine remaining vendor
  passthroughs to an explicitly named vendor-facing namespace.
- Declare one canonical production encryption path in the crate docs and root
  exports, and mark every alternate path as compatibility-only or experimental.

#### ❓ `z00z_crypto` Validation Plan And Open Questions

- Audit the workspace for direct dependencies on Tari-exported types through
  `z00z_crypto`. ANS: YES
- Identify which re-exports are actually used in public surfaces of other
  crates. ANS: YES
- Open question: is backend substitution a real future requirement or only a
  design aspiration. The answer should determine how much isolation overhead is
  justified now.  ANS: a design aspiration; justified now

### 🌐 `z00z_networks`

`z00z_networks` is architecturally the weakest reviewed area, not because the
RPC crate is bad, but because the umbrella boundary is incomplete.

#### 📌 `z00z_networks` Boundary Summary

- The folder contains `rpc/` and `onionnet/`.
- Only `rpc/` has a `Cargo.toml`; `onionnet/` is empty.
- `z00z_networks_rpc` provides a clean `RpcTransport` trait over
  `z00z_utils::codec::Value` and supports native and WASM targets.
- The RPC crate does not yet represent the entire networking architecture. It
  is only one slice of it.

#### 🚨 `z00z_networks` Findings Ordered By Severity

- `High | Confirmed risk`: `z00z_networks` is not yet a coherent crate boundary.
  The repository name suggests a network subsystem, but the actual package-level
  implementation exists only for RPC transport. OnionNet has no package,
  interface, or shared ownership model.

- `Medium | Likely risk`: The current transport abstraction is message-shaped,
  not system-shaped. `RpcTransport::call(method, params)` is fine for request/
  response wiring, but it does not define lifecycle, retry posture, timeout
  policy, streaming, peer identity, or network fault containment.

- `Low | Confirmed trade-off`: The RPC crate correctly uses a transport trait and
  a codec-owned `Value` instead of hard-wiring consumers to one runtime or one
  JSON transport implementation.

#### ⚖️ `z00z_networks` Trade-Offs And Rationale

Starting with RPC first is reasonable. It provides immediate utility to
wallet and simulator flows.

The risk is semantic overreach. Teams may start treating `z00z_networks_rpc`
as if the network architecture were already defined, even though transport is
only one dimension of networking.

#### ✅ `z00z_networks` Recommended Changes

- Keep `z00z_networks` in the current layout and document explicit ownership
  rules for what belongs to `rpc` versus future OnionNet code.
- Add OnionNet interface placeholders and namespace-level boundary notes inside
  the existing `z00z_networks` tree so new code lands behind named seams rather
  than as ad hoc modules.
- Add explicit network policy seams for timeout, retry, authentication, and
  connection lifecycle before more crates depend on RPC as the network default.

#### ❓ `z00z_networks` Validation Plan And Open Questions

- Validate whether upcoming features need streaming, subscriptions, or peer identity beyond request/response RPC. ANS: `Yes. The current request/response RPC seam is sufficient for simple wallet commands, but upcoming OnionNet features require additional network primitives beyond RpcTransport::call(method, params): peer identity and certificate pinning for node-to-node QUIC, descriptor and bridge refresh flows, health and backpressure exchange, and likely long-lived control or telemetry streams. Public wallet RPC may remain request/response in MVP, with polling for reply/status if needed, but the network layer itself cannot be modeled as RPC-only.` 

- Decide who owns transport observability and retry policy: RPC itself, callers, or a higher-level network orchestrator. ANS: `A higher-level network orchestrator should own end-to-end transport observability, backpressure posture, and retry policy. RPC should own only per-call mechanics such as serialization, request timeouts, and local error surfacing; callers may choose UX behavior, but they should not independently define transport retry semantics. Retry, queue shedding, temporary rejection, and health aggregation must be coordinated at the OnionNet or node control-plane level so that bridge, edge, relay, exit, and phase 120 behave consistently under saturation.` 

- Open question: will OnionNet be a peer transport, a privacy overlay, or a separate application service. Each option implies different ownership and interface requirements inside the current layout. ANS: `OnionNet should be treated as a node-owned privacy overlay with its own internal peer transport, not as a separate application service. Concretely, it is a transport and privacy shell around phase 100 ingress, hosted by phase 120 as a node transport plane, with a dedicated crate under crates/z00z_networks/onionnet. It terminates at exit normalization and hands off only canonical WorkItem values into AggregatorIngress::admit(). Wallets own only transport selection and control-plane hooks; runtime aggregators own only canonical intake and ordering; phase 120 remains the composition root.` 


### 🧪 `z00z_simulator`

`z00z_simulator` is the integration harness of the workspace. It is the most
valuable crate for proving system behavior and the most dangerous crate for
accumulating accidental coupling.

#### 📌 `z00z_simulator` Boundary Summary

- The crate depends directly on core, crypto, RPC, storage, utils, and wallets.
- The root module is clean and scenario-oriented.
- `scenario_1` is split into many stage modules and utility lanes.
- The README currently contains only whitespace, so the intended architectural
  contract of the simulator is undocumented.

#### 🚨 `z00z_simulator` Findings Ordered By Severity

- `High | Confirmed risk`: The simulator is a direct dependency hub for nearly
  every major reviewed crate. That is normal for an integration harness, but it
  also means architectural drift will first become normalized here. If the
  simulator reaches through unstable internals instead of stable facades, it can
  silently turn temporary seams into permanent dependencies.

- `Medium | Confirmed risk`: The crate lacks architectural documentation. An
  empty README is not just a docs problem here; it means there is no explicit
  boundary statement for what the simulator is allowed to depend on, what it is
  allowed to fake, and which surfaces are considered public integration seams.

- `Medium | Likely risk`: The `scenario_1` stage decomposition is readable, but
  without a higher-level scenario contract it can ossify into workflow-specific
  coupling. Stage utilities and public re-exports can become de facto APIs for
  new scenarios instead of a consciously designed scenario framework.

#### ⚖️ `z00z_simulator` Trade-Offs And Rationale

A simulator must touch many crates. That is its job.

The trade-off is that its dependency breadth must be balanced by stronger
rules about where it enters each crate. Otherwise it becomes the place where
every layering exception is rationalized.

#### ✅ `z00z_simulator` Recommended Changes

- State in the README that `z00z_simulator` is the integration harness for
  scenario execution, and document which crate-root facades are allowed import
  surfaces.
- Restrict simulator imports to stable facades wherever possible.
- Add a documented scenario contract so new scenarios reuse the same stage and
  artifact conventions instead of copying `scenario_1` internals ad hoc.

#### ❓ `z00z_simulator` Validation Plan And Open Questions

- Audit whether simulator code imports private-leaning modules instead of stable
  crate-root facades. ANS: BOTH
- Add one architectural check that fails if simulator starts depending on known
  implementation-detail modules. ANS: YES
- Open question: is the simulator intended for tests only, operator demos, or
  future reproducible research workflows. ANS BOTH

### 💾 `z00z_storage`

`z00z_storage` owns a meaningful and mostly coherent domain: hierarchical
asset paths, proofs, snapshots, serialization, and state-root machinery.

#### 📌 `z00z_storage` Boundary Summary

- The root exposes `assets`, `checkpoint`, `serialization`, and `snapshot`.
- The README clearly states the canonical path and minimal committed leaf rules.
- The crate uses JMT heavily for proofs and tree semantics.
- The same crate also contains a RedB backend under
  `src/assets/store_internal/redb_backend.rs`.
- Dev dependencies reach upward into `z00z_wallets`.

#### 🚨 `z00z_storage` Findings Ordered By Severity

- `Medium | Confirmed risk`: `z00z_storage` combines consensus-root semantics
  with backend-specific persistence mechanics inside one package. JMT-based proof
  ownership and RedB backend ownership are both legitimate, but they are not the
  same architectural concern.

- `Medium | Likely risk`: The dev-dependency on `z00z_wallets` suggests reverse
  knowledge flow in tests. Runtime layering stays intact, but test-time coupling
  can still normalize storage behavior that is wallet-shaped instead of storage-
  shaped.

- `Low | Confirmed strength`: The README is unusually precise about what counts
  toward committed roots and what must remain side-index metadata. That is good
  architecture because it makes invariants explicit.

#### ⚖️ `z00z_storage` Trade-Offs And Rationale

Keeping JMT logic and concrete persistence in one crate makes the proof system
easier to implement and benchmark.

The cost is that backend changes and consensus-invariant changes travel
together. If not managed carefully, a storage engine refactor can begin to feel
like a consensus refactor.

#### ✅ `z00z_storage` Recommended Changes

- Treat RedB as an internal backend seam and keep it behind dedicated internal
  interfaces instead of letting backend details leak into stable storage-facing
  contracts.
- Keep consensus-root types and proof semantics in the most stable namespace;
  move backend-specific helpers deeper behind internal modules.
- Remove wallet-shaped assertions from storage tests where they are not needed
  to verify storage invariants.

#### ❓ `z00z_storage` Validation Plan And Open Questions

- Benchmark whether alternate storage backends are a realistic future need. If
  not, an internal RedB coupling may be an acceptable trade-off. ANS:  a realistic future need
- Audit storage tests for direct wallet-domain assumptions. ANS: YES
- Open question: should checkpoint and snapshot stay in the same crate as asset
  tree storage, or should they be separated by stricter internal seams. ANS: in the same crate

### 🧰 `z00z_utils`

`z00z_utils` is currently the strongest architectural crate in the reviewed
set because it actually embodies the repository's stated design constitution.

#### 📌 `z00z_utils` Boundary Summary

- The crate groups codec, config, I/O, logger, metrics, RNG, time,
  compression, and OS hardening.
- The `prelude` provides curated reuse for common abstractions.
- `io/mod.rs` intentionally re-exports `std::fs::File` and `std::io` traits
  alongside higher-level helpers.
- The crate has no top-level README file at `crates/z00z_utils/README.md`.

#### 🚨 `z00z_utils` Findings Ordered By Severity

- `Medium | Likely risk`: `z00z_utils` is on a path toward megacrate status.
  Cross-cutting concerns already include serialization, file I/O, config,
  logging, metrics, compression, RNG, time, and hardening. This is still
  coherent, but only if admission criteria remain strict.

- `Low | Confirmed trade-off`: Re-exporting `File`, `copy`, `Read`, `Write`, and
  related primitives through the I/O module is defensible because the Design
  Foundation explicitly allows streaming-heavy operations to use standard types.
  This is a good example of pragmatic abstraction rather than dogmatic wrapping.

- `Low | Confirmed risk`: Documentation is lighter than the crate's importance.
  The crate already has module-map and quick-reference documents, but it lacks a
  root-level `README.md` landing page. That is not fatal, yet it still reduces
  discoverability of the project's most important architectural seam.

#### ⚖️ `z00z_utils` Trade-Offs And Rationale

A strong utilities crate improves consistency across the workspace.

The risk is over-aggregation. Once every low-level concern lands here,
`z00z_utils` can become the place where unrelated policy debates are forced to
coexist and every crate-level change waits on one foundational package.

#### ✅ `z00z_utils` Recommended Changes

- Add a concise root README that explains what belongs in `z00z_utils` and what
  does not, and link it to the existing module-map and quick-reference docs.
- Add an admission checklist for new modules so the crate stays focused on
  cross-cutting abstractions rather than becoming a general dumping ground.
- Reject new low-level convenience helpers unless they strengthen an existing
  abstraction boundary or remove duplication across multiple business crates.

#### ❓ `z00z_utils` Validation Plan And Open Questions

- Review recent additions to `z00z_utils` and classify whether they are true
  cross-cutting policy or just convenient shared code. ANS: they are true cross-cutting policy
  ANS: recent additions in z00z_utils are not one uniform class. The crate itself is a real cross-cutting policy seam, as the review already notes in [031-architecture-review.md](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-browser/workbench/workbench.html), but the two additions differ in kind. The stronger true policy module is os_hardening, because it centralizes audited unsafe, OS-level hardening, secret-memory handling, and fail-closed behavior in [os_hardening.rs](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-browser/workbench/workbench.html), and it is consumed in wallet runtime/session key handling in [wallet_service_actions_hardening.rs](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-browser/workbench/workbench.html) and [redb_wallet_store_open_session.rs](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-browser/workbench/workbench.html). Compression is narrower: it includes important safety policy such as bounded decompression in [compression.rs](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-browser/workbench/workbench.html), but in practice it is mostly shared infrastructure for wallet backup and wallet object storage flows, not a workspace-wide constitutional boundary, as shown by [redb_wallet_store_codecs.rs](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-browser/workbench/workbench.html) and [backup_importer_impl.rs](vscode-file://vscode-app/usr/share/code/resources/app/out/vs/code/electron-browser/workbench/workbench.html).
- Open question: should OS hardening and compression stay here permanently, or
  should one of them be governed by stricter submodule-level admission rules.
  ANS: os_hardening should stay in z00z_utils, but only under stricter submodule-level admission and review rules. It is exactly the kind of low-level, security-sensitive, platform-specific policy that benefits from one audited home. Compression should not be treated as permanently admitted on the same basis. It may remain in z00z_utils for now, but under a narrower rule: only generic bounded compression and streaming primitives belong there. If the module keeps accumulating wallet-format semantics, backup-container rules, or storage-specific policy, it should move down to the owning storage or wallet boundary instead of expanding the utilities crate.

### 👛 `z00z_wallets`

`z00z_wallets` has the largest architectural surface of the reviewed crates
and therefore the highest structural risk.

The crate declares a clean architecture story with core, adapters, services,
database backends, optional desktop UI, and WASM-facing surfaces. That is a
credible target architecture. The problem is that the exported surface already
looks stable while major parts of the implementation are still explicitly
placeholder-driven.

#### 📌 `z00z_wallets` Boundary Summary

- The root exposes `core`, `adapters`, `services`, `wasm`, optional `db`, and
  optional `egui_views`.
- The crate root and README both say that services and adapters are still
  largely stub-heavy.
- `src/services/wallet_service.rs` uses `include!` to splice multiple service
  fragments and tests into one compilation unit.
- `Cargo.toml` pulls in core, crypto, storage, utils, RPC, serialization,
  cryptography, GUI, async, and state-machine dependencies.
- `core/tx/multi_io.rs` still uses `rand::thread_rng()` on non-seeded paths.

#### 🚨 `z00z_wallets` Findings Ordered By Severity

- `High | Confirmed risk`: Placeholder semantics are exported through the same
  facades that future production callers are expected to use. The crate and
  README openly describe services, chain logic, and RPC methods as partial or
  stub-heavy, yet the crate root re-exports them as normal integration surfaces.
  That is the most serious architectural issue in the reviewed set.

- `High | Confirmed risk`: The service layer is structurally opaque because
  `wallet_service.rs` is assembled with `include!` from types, actions, session,
  store, and tests. This weakens module boundaries, increases merge friction,
  and makes review harder because the visible file boundary is not the real
  architecture boundary.

- `High | Confirmed risk`: The crate is functionally a mini-monolith. It owns
  domain logic, orchestration, RPC, persistence, optional GUI, and WASM-facing
  surfaces inside one package. That may be acceptable temporarily, but it means
  almost any change in wallet behavior can ripple across multiple layers without
  a package boundary to slow the spread.

- `Medium | Confirmed risk`: Boundary discipline around deterministic RNG and
  abstraction policy is inconsistent. `core/tx/multi_io.rs` falls back to
  `rand::thread_rng()` when no seed is provided, which weakens the repository's
  deterministic and centralized RNG story.

- `Medium | Confirmed risk`: Concrete JSON types appear in both core and adapter
  code paths. That is expected at the RPC edge, but the same pattern also shows
  up in wallet verification and persistence-adjacent code such as
  `core/tx/tx_verifier.rs` and `db/redb_wallet_store.rs`. This suggests the
  wire/domain split is not yet fully hardened.

#### ⚖️ `z00z_wallets` Trade-Offs And Rationale

A single wallet crate keeps the phase-to-phase development loop fast. It
avoids the overhead of multiple micro-crates while the product surface is still
forming.

The cost is architectural debt concentration. The more the crate succeeds,
the more expensive future disentangling becomes unless the public seams are
narrowed before production behavior fully lands.

#### ✅ `z00z_wallets` Recommended Changes

- Stop treating stub-heavy services as normal stable crate-root contracts.
  Gate them behind explicit compatibility or experimental surfaces and narrow
  the top-level exports accordingly.
- Replace `include!`-assembled services with explicit submodules and one thin
  facade module.
- Define which layers are allowed to use concrete JSON values and which must use
  domain types or `z00z_utils` codec abstractions only.
- Harden the wallet core facade, the RPC facade, and the persistence facade as
  three explicit in-crate boundaries with separate export rules and
  compatibility policies.

#### ❓ `z00z_wallets` Validation Plan And Open Questions

- Audit all downstream imports of `z00z_wallets` to see which placeholders are
  already being depended on. ANS: YES
- Measure compile-time and review-surface impact of the current `include!`
  service assembly. ANS: YES
- Open question: how strict should the long-term single-crate wallet facade be,
  and which compatibility lanes should remain feature-gated instead of public by
  default.

## ⚖️ Trade-Off Summary

The repository is making one coherent strategic bet: preserve momentum by
keeping several large packages internally modular before tightening their
exports, ownership rules, and internal seams.

That bet is still viable for `z00z_core`, `z00z_storage`, and `z00z_utils`.
It is becoming risky for `z00z_crypto`, and it is already structurally risky for
`z00z_wallets` and for the still-incomplete `z00z_networks` namespace.

The root cause is not code quality in the narrow sense. The root cause is
contract maturity. The codebase exposes more surface area than it is ready to
govern as stable architecture.

## ✅ Recommended Change Order

1. Narrow exported surfaces in `z00z_core`, `z00z_crypto`, and `z00z_wallets`.
2. Keep `z00z_networks` in its current shape, but define explicit ownership,
    policy, and interface rules so the namespace is no longer ambiguous.
3. Make `z00z_wallets` placeholder seams explicit and non-default.
4. Add simulator architecture rules so it consumes only stable facades.
5. Decide whether `z00z_storage` needs a clearer in-crate seam between
    consensus proof ownership and backend mechanics.
6. Protect `z00z_utils` from scope creep with written admission rules.

## 🧪 Validation Plan

The highest-value validation work is architectural, not algorithmic.

1. Run import-graph audits for each reviewed crate root and identify which
   re-exports are actually used across the workspace.
2. Create one policy check that forbids new Tari concrete type leakage through
   `z00z_crypto` root exports unless explicitly approved.
3. Create one policy check that fails if new simulator code imports known
   implementation-detail wallet or storage modules.
4. Prototype a non-`include!` wallet service layout and measure whether review
   clarity improves without harming development velocity.
5. Decide, in writing, which boundaries are stable now and which are still
   intentionally provisional.

## 🚩 Final Verdict

- The workspace has a recognizable architecture and a credible direction.

- The next architectural bottleneck is not in `z00z_utils`, `z00z_core`, or
`z00z_storage`. It is in surface-governance around `z00z_crypto`, package
completeness around `z00z_networks`, and boundary maturity inside
`z00z_wallets`.

- If those three areas are tightened before more product logic lands on top of
them, the current design remains salvageable and scalable. If not, the project
will keep compiling while its real architectural contracts drift out of control.

---



# аудит безопасности для z00z_crypto: 

сначала соберу архитектурный и dependency-контекст, затем проверю секреты, типичные крипто- и Rust-риски, и в конце прогоню отдельную верификацию выводов.

Провёл статический security-аудит z00z_crypto. Подтверждён один существенный finding.

1. Medium: test-only AEAD helper доступен в non-test сборках через feature-флаги. В aead.rs публично реэкспортируется `seal_with_nonce_TEST_ONLY` под `feature = "test-fast"` и `feature = "test-utils"`, а сама функция в aead_test_only.rs принимает caller-supplied nonce для XChaCha20-Poly1305. Это не ломает безопасность автоматически, но выносит опасный primitive в сборки, которые уже не ограничены `cfg(test)`. Для AEAD повторный nonce под тем же ключом критичен, поэтому такой API не должен быть доступен в потенциально ship-ready feature-профилях.

Что я сознательно не подтвердил как уязвимость. `SecretBytes32::into_inner()` в secret_bytes.rs выглядит как осознанный escape hatch, а не скрытая утечка: он действительно выводит секрет за границы zeroize-wrapper, но это семантически ожидаемо для `into_inner`. Аналогично, `ClaimSourceProof::from_bytes()` в v2.rs сам по себе не оказался unbounded decode: общий `BincodeCodec` в bincode.rs ограничивает десериализацию 10 MB и режет trailing bytes. Это скорее кандидат на ужесточение policy для конкретного типа, но не подтверждённый security bug. `SignBackend(String)` в v2.rs тоже не тянет на finding без доказательства, что туда реально попадают чувствительные данные; по увиденному sign path в lib_api.rs наружу уходит фиксированный backend context, а не секретный материал.

Ограничения аудита: это был read-only статический обзор, без запуска fuzzing, runtime instrumentation и без сборки матрицы feature-комбинаций. Отдельный secret scan через GitHub Advanced Security не выполнился, потому что для репозитория он не включён. Для повышения уверенности следующий естественный шаг один: убрать публичный реэкспорт `seal_with_nonce_TEST_ONLY` из любых non-test feature-конфигураций и оставить его только под `cfg(test)` или в закрытом test-support модуле.

---



# аудит безопасности для z00z_core: 

архитектура, зависимости, точки входа и потенциально опасные зоны, чтобы аудит был по реальным trust boundaries, а не по grep-шаблонам.

Final closeout verification

Финальный набор стабилен: один low-severity hardening finding, остальное снято. Закрываю аудит в формате security report с явной пометкой, что `doublecheck` gate выполнен и автоматический secret-scan был недоступен из-за выключенного GHAS.

Ran terminal command:  ./scripts/play_tone.sh

**Findings Summary**

| Severity   | Count |
| ---------- | ----: |
| 🔴 CRITICAL |     0 |
| 🟠 HIGH     |     0 |
| 🟡 MEDIUM   |     0 |
| 🔵 LOW      |     1 |
| ⚪ INFO     |     3 |

**Scope**
Аудит выполнен для z00z_core и связанных import/export границ внутри этого crate, включая `src/`, `bin/`, `tests/`, локальный Cargo.lock и ключевые shared codec paths в z00z_utils, на которые `z00z_core` опирается.

**Constraints**

- Только статический аудит, без intrusive testing и без правок кода.
- `doublecheck` verification gate выполнен.
- Автоматический GitHub secret scan через MCP не смог отработать по репозиторию, потому что для него не включён GitHub Advanced Security.

**Assets and Trust Boundaries**

- Human-readable JSON import/export boundary для `AssetPkgWire` в wire_pkg.rs.
- Genesis import/export boundary в serde.rs.
- Secret-bearing internal asset state и blinding material в assets.rs и tooling around asset generation.
- Local file I/O, archive creation, config loading, and artifact generation in genesis and CLI helpers.

**Evidence Reviewed**

- Cargo.toml
- README.md
- secret.rs
- wire.rs
- wire_pkg.rs
- wire_pkg_serde.rs
- wire_pkg_serde_impls.rs
- wire_pkg_serde_parse.rs
- assets.rs
- asset_crypto.rs
- genesis_config.rs
- serde.rs
- genesis_output_support.rs
- genesis_run.rs
- assets_generation_cli.rs
- assets_generation_cli_phase.rs
- assets_generation_cli_report.rs
- json.rs
- bincode.rs
- deterministic.rs

**Findings**

**🔵 LOW - Input Handling / DoS Hardening: unbounded JSON deserialization on import-facing paths**

- Confidence: Medium
- Evidence:
  - Shared JSON codec deserializes arbitrary byte slices without an explicit size cap in json.rs.
  - `AssetPkgWire` is documented as a canonical human-readable JSON boundary in wire_pkg.rs.
  - `decode_asset_pkg_json` in wire_pkg_serde_impls.rs first probes JSON for a forbidden `secret` field and then performs typed deserialization.
  - Additional JSON import helpers rely on the same unbounded codec in serde.rs, serde.rs, and serde.rs.
- Exact code:

```rust
fn deserialize<T: DeserializeOwned>(&self, bytes: &[u8]) -> Result<T, Self::Error> {
    let mut stream = serde_json::Deserializer::from_slice(bytes).into_iter::<T>();
```

```rust
pub fn decode_asset_pkg_json(bytes: &[u8]) -> Result<AssetPkgWire, AssetError> {
    if payload_has_secret_field(bytes)? {
        return Err(AssetError::InvalidAsset(Cow::Borrowed(
            "asset pkg json decode failed: forbidden field: secret",
        )));
    }

    let codec = JsonCodec;
    codec.deserialize(bytes)
```

- Risk in plain English:
  Если внешний caller не ограничивает размер входного JSON заранее, oversized payload может вызвать избыточные аллокации и CPU load на import/decode пути. По текущим данным это не доказанный remote exploit, а реальный resource-exhaustion hardening gap.
- Attack-path note:
  Уязвимость зависит от того, есть ли upstream request-size/file-size cap. Внутри самого `JsonCodec` такого ограничения нет.
- Recommended remediation:
  1. Добавить bounded JSON decode в `z00z_utils::codec::JsonCodec`, аналогично bounded bincode.
  2. Для `AssetPkgWire` и genesis import paths ввести type-specific size ceilings.
  3. Если публичный boundary уже ограничивается выше по стеку, зафиксировать это как явный contract и покрыть тестами.
- Residual risk:
  Пока не доказано, что эти функции вызываются из неограниченного сетевого пути. Поэтому severity оставлен низким.

**Info / Dropped After Verification**

- ⚪ `AssetPkgWire` secret-injection guard выглядит в целом корректным:
  - top-level unknown fields запрещены через `#[serde(deny_unknown_fields)]` в wire_pkg_serde.rs
  - `AssetWire::validate()` дополнительно запрещает `secret` в импортируемом wire object в wire.rs

- ⚪ `create_genesis_snapshot_zip()` пишет `cli_command` в `run_genesis.sh`, но это выглядит как self-injection/tooling artifact, а не подтверждённая privilege-crossing уязвимость:
  - genesis_output_support.rs
  - genesis_run.rs

- ⚪ `assets_generation_cli` использует `DeterministicRngProvider`, но по коду и документации это intentional reproducibility tooling, gated by feature, а не подтверждённый production-crypto flaw:
  - Cargo.toml
  - assets_generation_cli_phase.rs
  - deterministic.rs

**Dependency Audit**
Явных dependency findings по просмотренным пакетам не подтвердилось. В scoped Cargo.lock не всплыло очевидного watchlist-level попадания среди ключевых зависимостей `z00z_core`.

**Secrets & Exposure Scan**

- Ручной scoped scan по коду не выявил захардкоженных ключей или токенов в reviewed paths.
- Автоматический secret scanning MCP tool вернул: GitHub Advanced Security для репозитория не включён, поэтому repo-backed secret scan не выполнился.
- Это validation gap только для автоматической GitHub secret-scanning части, не для ручного обзора кода.

**Doublecheck Gate**

- `doublecheck` выполнен на черновом и финальном наборе findings.
- Результат: retained только один low-severity finding; спорные кандидаты про script reflection и deterministic RNG были сняты.

**Conclusion**
По z00z_core подтверждён один низкоприоритетный hardening issue: shared JSON import paths не имеют собственного payload-size limit и потенциально могут использоваться для resource exhaustion, если выше по стеку отсутствуют ограничения размера входа. Критических, high или medium findings по просмотренному объёму не подтверждено.

**Next verification step**

1. Проверить, есть ли upstream size caps на все вызовы `decode_asset_pkg_json` и genesis JSON import paths.
2. Если таких caps нет, ввести bounded JSON codec в `z00z_utils` и narrow tests на oversized payload rejection.
3. Если хочешь, следующим сообщением я подготовлю уже не аудит, а точный patch plan для устранения этого finding без лишнего рефакторинга.

---



# аудит безопасности для z00z_storage

Compacted conversationНачинаю новый аудит для z00z_storage: сначала доберу обязательный контекст по правилам репозитория, памяти и спецификации, затем построю карту поверхностей атаки и критических end-to-end путей, чтобы не сканировать вслепую.

**Security Review Report**

Project: `z00z_storage`  
Scan Date: `2026-04-03`  
Scope: crates/z00z_storage  
Languages Detected: Rust  
Frameworks Detected: Rust workspace crate, RedB, JMT

┌──────────────────────────────────────────────┐
│              FINDINGS SUMMARY                │
├──────────────┬───────────────────────────────┤
│ 🔴 CRITICAL  │ 0                             │
│ 🟠 HIGH      │ 0                             │
│ 🟡 MEDIUM    │ 2                             │
│ 🔵 LOW       │ 0                             │
│ ⚪ INFO      │ 1                             │
├──────────────┼───────────────────────────────┤
│ TOTAL        │ 3                             │
└──────────────┴───────────────────────────────┘

Dependency Audit: `0` actionable package findings in manifest review  
Secrets Scan: `0` secrets found in scoped source/test review

> [!WARNING]
> **MEDIUM — File-store seal path can publish an unreplayable checkpoint bundle**
>
> **Confidence:** HIGH  
> **Location:** store.rs, store_fs.rs, test_checkpoint_store_api.rs, redb_backend_validate.rs, test_redb_rehydrate.rs
>
> `CheckpointFsStore::seal_artifact()` only checks that the proof statement tuple matches the caller-supplied `snap_id` and `exec_id`, then persists artifact plus link. `save_link()` only requires artifact presence, statement/link tuple consistency, and exec-id uniqueness. It does **not** require that the referenced snapshot row or exec-input row already exists in persistent storage.
>
> The strongest proof is the existing regression test test_checkpoint_store_api.rs: it seals a checkpoint successfully, then immediately proves that `load_exec_input()` fails because no exec row exists. In contrast, the canonical RedB reload path rejects this exact class of state drift by requiring snapshot, draft, checkpoint, link, and exec rows to all be present and mutually consistent in redb_backend_validate.rs.
>
> **Risk:** a caller can create a checkpoint/link pair that looks canonical to generic file-store consumers but cannot be replayed or fully audited end to end. That is an integrity and control-gap issue, not just a convenience tradeoff, because the durable backend already treats missing replay evidence as invalid state.
>
> **Recommended fix:** make `seal_artifact()` fail closed unless the referenced snapshot and exec-input artifacts are already persisted, or add a distinct “draft-only / non-canonical” persistence API and reserve `seal_artifact()` for fully materialized replay bundles.

> [!CAUTION]
> **MEDIUM — Generic checkpoint artifact APIs treat `cp_proof` as opaque bytes; proof binding is enforced only on RedB rehydrate**
>
> **Confidence:** MEDIUM-HIGH  
> **Location:** artifact_proof_draft.rs, artifact_proof_draft.rs, artifact_proof_draft.rs, artifact_final.rs, artifact_final.rs, test_checkpoint_finalization.rs, redb_backend.rs, redb_backend_validate.rs, test_redb_rehydrate.rs
>
> `CheckpointProof::new()` and `new_attest()` reject only empty `cp_proof`. `CheckpointDraft::finalize()` verifies `pub_in` and optional statement/draft alignment, then stores raw proof bytes into the final artifact. The existing test test_checkpoint_finalization.rs proves that changing `cp_proof` changes serialized bytes but **does not** change the derived checkpoint ID.
>
> The canonical durable backend does something stricter: it synthesizes proof bytes from `(exec_id || state_root)` in redb_backend.rs, and on reload it rejects checkpoint rows whose statement or proof bytes drift from the persisted draft/exec/state tuple in redb_backend_validate.rs and redb_backend_validate.rs.
>
> **Risk:** outside the RedB rehydrate path, callers can persist or transport artifacts whose proof bytes are structurally accepted but semantically unbound to any verified replay/input root. This is weaker than a direct forged-proof exploit because this crate is mostly storing artifacts, not verifying zk soundness itself, so I am not rating it HIGH. But it is still a real integrity gap because one backend treats proof binding as mandatory while the public artifact layer does not.
>
> **Recommended fix:** either bind `cp_proof` into the artifact ID / compatibility contract, or expose an explicit verifier/attestor boundary so that storage APIs cannot be mistaken for proof-validation APIs.

> [!NOTE]
> **INFO — Nullifier keying by `ClaimNullifier` alone is a hardening note, not a finding**
>
> **Confidence:** HIGH  
> **Location:** store_rows.rs, nullifier.rs, nullifier.rs, claim_tx_verifier_impl_proof.rs
>
> Storage enforces replay by `ClaimNullifier`, while the row also records `chain_id` for audit. That would be suspicious if the nullifier were chain-agnostic. But upstream wallet logic derives the nullifier with `chain_id` included and explicitly verifies it against `derive_nullifier(claim_id, recipient_owner_hex, chain_id)`. So this should stay a hardening observation only, not a vulnerability.

**Critical End-to-End Verification Map**

> [!IMPORTANT]
> Ниже перечислены именно те user journeys и state transitions, которые нужно доказывать end to end. Здесь важна не только корректность happy path, но и то, что storage отвергает частично-связанные или подменённые артефакты.

1. **Asset insert → witness proof blob → replay validation**

   What must be proven: `AssetPath {definition_id, serial_id, asset_id}` остаётся каноническим идентификатором leaf, semantic root совпадает с ожидаемым root, `root_bind` связывает semantic root и backend root, а все три JMT branches доказывают существование нужных leaves. Это реализовано в proof.rs и проверяется в test_store_api.rs и test_checkpoint_root_binding.rs.

   Success example: valid blob passes `chk_blob()` and preserves root/path/leaf agreement.  
   Negative scenarios: wrong semantic root, wrong backend root, wrong path, wrong definition leaf, wrong serial leaf, tampered `root_bind`, wrong bind version.  
   Measurable pass/fail: `chk_blob()` must return `Ok(_)` only when all root/path/leaf/proof constraints hold; otherwise it must return the precise typed error such as `RootMix`, `PathMix`, `DefMix`, `SerMix`, `RootBindMix`, `BindVerMix`, `DefProofMix`, `SerProofMix`, or `AssetProofMix`.

2. **Snapshot persistence → snapshot validation → replay entry recovery**

   What must be proven: stored snapshot bytes round-trip canonically, `prev_root` is preserved, replay entries can be reconstructed without consulting simulator internals, and duplicated or malformed witness/path state is rejected. Evidence sits in test_replay_bound.rs and the snapshot path/bind suites already enumerated by the crate tests.

   Success example: `validate_snapshot()` passes and `replay_entries()` returns exactly one entry per snapshot row.  
   Negative scenarios: duplicate path, duplicate asset id, malformed witness blob, corrupted transport wrapper, version mismatch.  
   Measurable pass/fail: replay count must equal stored entry count; any witness/root/path mismatch must hard-fail before replay is accepted.

3. **Snapshot + exec input + draft → finalized checkpoint artifact + bound link**

   What must be proven: `snap_id`, `exec_id`, `prev_root`, public inputs, statement tuple, and final checkpoint artifact all describe the same execution boundary. Positive and negative evidence is in test_checkpoint_replay_inputs.rs, test_checkpoint_link_injective.rs, and test_checkpoint_finalization.rs.

   Success example: matching `snap_id` and `exec_id` produce a stable final artifact and link round-trip.  
   Negative scenarios: `snap_id` mismatch, `exec_id` mismatch, `prev_root` mismatch, tampered `pub_in`, conflicting exec reuse, tampered link tuple, legacy opaque artifact used with bound link.  
   Measurable pass/fail: typed failures must remain stable as `LinkMix`, `ReplayMix`, `RootMix`, `ProofMix`, or `ArtifactCompatMix`; a successful seal must not be possible unless the full replay bundle exists.

4. **Durable RedB reload → canonical metadata rehydrate**

   What must be proven: on process restart, persisted state is accepted only if snapshot row, draft row, checkpoint row, link row, and exec row all exist; statement equals reconstructed draft boundary; checkpoint proof bytes equal the canonical storage-owned encoding; mixed legacy/new ID eras are rejected. This is enforced in redb_backend_validate.rs and proven by test_redb_rehydrate.rs, test_redb_rehydrate.rs, and test_redb_rehydrate.rs.

   Success example: reload keeps root and items stable across restart cycles.  
   Negative scenarios: missing snapshot row, tampered checkpoint proof bytes, statement drift from draft, mixed checkpoint-id eras, legacy checkpoint artifact with persisted link metadata.  
   Measurable pass/fail: `AssetStore::load(...)` must reject any metadata bundle that cannot reconstruct the exact durable checkpoint tuple.

5. **Claim source root/proof → nullifier replay protection**

   What must be proven: claim source root and claim source proof reuse the same bound witness machinery, and replay is blocked by nullifier uniqueness. Positive evidence is in test_claim_source_proof.rs; replay rejection is enforced in store_rows.rs.

   Success example: claim proof blob decodes, root versions match, and `root_bind` is non-zero.  
   Negative scenarios: repeated nullifier already present in storage, duplicate nullifier in one publish batch.  
   Measurable pass/fail: duplicate claim publication must fail with `ClaimReplay`, and cross-chain replay relies on upstream chain-separated nullifier derivation remaining unchanged.

> [!TIP]
> Для regression suite я бы считал минимально обязательными следующие сценарии:
>
> 1. seal path rejects checkpoint creation when exec row is absent  
> 2. seal path rejects checkpoint creation when snapshot row is absent  
> 3. generic file-store load rejects artifact/link bundle whose `cp_proof` bytes drift from `(exec_id, state_root)` if artifact is attested  
> 4. claim nullifier cross-chain collision stays impossible as long as wallet derivation includes `chain_id`

> [!NOTE]
> Это был статический аудит по коду и существующим тестам. Я не запускал test suite. Субагент для независимого double-check в этой сессии не смог прочитать workspace, поэтому финальная уверенность основана на ручной cross-check проверке исходников и тестов.

Natural next steps:

1. Ужесточить `CheckpointFsStore::seal_artifact()` до полного replay-evidence gate.
2. Выделить явный API boundary между “store opaque artifact” и “store verified attested checkpoint”.
3. После правки прогнать focused tests по checkpoint/snapshot/redb rehydrate и повторить security review только для z00z_storage.

---



# аудит безопасности для z00z_wallets

Продолжаю с критических wallet-путей: сейчас доберу точные доказательства по session auth, wallet identity drift и seed/backup throttling, затем сверю это с тестами и вынесу только подтверждённые findings.

**Итог**
Ниже итоговый deep security report по z00z_wallets на основе read-only аудита кода и точечных exact test runs. В этой фазе я не менял файлы.

> [!IMPORTANT]
> Самый сильный подтверждённый риск в этом проходе не в криптопримитивах, а в нарушении единого источника истины для wallet identity: storage-слой строго привязан к persisted `network/chain`, а часть service/RPC-путей снова читает runtime env/config и смешивает эти модели.

> [!WARNING]
> На RPC boundary подтверждён auth-free `app.wallet.list_wallets`, а `wallet.session.lock_wallet` зарегистрирован и реализован без session/password gate. Это образует реальный компонентный DoS chain; его итоговая severity зависит от того, открыт ли `wallet.*` наружу тем же policy layer, что и `app.*`.

> [!CAUTION]
> У меня есть прямой transport proof для `app.wallet.list_wallets`, но нет отдельного runtime E2E теста, который показывает внешний вызов `wallet.session.lock_wallet` через тот же transport. Поэтому эта часть зафиксирована честно: code-level exposure confirmed, deployment-level reachability partially verified.

**Findings Summary**

- `HIGH`: Persisted wallet identity не является single source of truth во всех open/unlock/derive путях.
- `MEDIUM`: Неаутентифицированное перечисление wallet ids подтверждено; lock RPC не требует session/password и может дать DoS при доступности метода через runtime dispatcher.
- `LOW`: Тестовое покрытие не закрывает mismatch-сценарии для `open_wallet_source(Path|Bytes)` при runtime chain/network drift.

**Finding 1: Identity Drift Between Persisted Wallet And Runtime Chain**
Вердикт: `confirmed`

Storage path строго валидирует identity из `.wlt`: тест test_redb_wlt_open.rs подтверждает `WalletNetworkMismatch` и `WalletChainMismatch`, а service helper wallet_service_store_support.rs умеет восстановить persisted identity через discovery.

Но выше по стеку инвариант ломается:

- Path import сохраняет `network` из runtime identity, а `chain` из discovery в wallet_service_store_create_unlock_open.rs.
- Bytes import вообще не пишет `wallet_identities` перед возвратом из wallet_service_store_create_unlock_open.rs.
- Unlock fallback при отсутствии cached identity снова уходит в runtime env/config через wallet_service_store_create_unlock_open.rs.
- Derivation path создаёт chain binding из runtime config в wallet_service_session_derivation.rs, а RPC address encoding повторно читает runtime chain в server_derive.rs. Источник этого поведения находится в wallet_paths.rs.

Почему это важно:

- Correctness risk: один и тот же persisted wallet может отдавать derive/address поведение, зависящее от текущего runtime chain, а не от собственной identity.
- Availability risk: при drift runtime chain unlock/derive начинают fail-closed не потому, что wallet повреждён, а потому что service layer смотрит не туда.
- Boundary inconsistency: часть flows уже правильно использует persisted identity, часть нет.

Proof path:

1. `.wlt` metadata discovery даёт persisted `{wallet_id, network, chain}` в wallet_service_store_create_unlock_open.rs.
2. Low-level open reject’ит mismatch в test_redb_wlt_open.rs.
3. Higher-level derive path читает runtime chain в wallet_service_session_derivation.rs и server_derive.rs.

Failure path:

1. Wallet создан/импортирован под persisted `devnet`.
2. Runtime chain меняется через env/config.
3. Unlock или derive начинают зависеть от runtime chain вместо persisted identity.
4. Получаем либо reject, либо wrong-chain address semantics.

Test evidence:

- Тест `cargo test -p z00z_wallets --test test_redb_wlt_open test_open_fails_identity_mismatch -- --exact` прошёл: storage mismatch reject подтверждён.
- Тест `cargo test -p z00z_wallets --test test_wallet_service_errors derive_public_key_rejects_invalid_chain_config -- --exact` прошёл; сам сценарий описан в test_wallet_service_errors.rs, где derive ломается после `broken-chain`.

**Finding 2: Wallet Enumeration Is Auth-Free, Lock RPC Has No Session Gate**
Вердикт: `partially confirmed`, с сильным code-level evidence

Что подтверждено прямо:

- `app.wallet.list_wallets` не принимает session и wired напрямую в dispatcher в app.rs, app_impl.rs, app_dispatcher_wiring.rs.
- Возврат содержит wallet ids через wallet_service_store_create_unlock.rs и структуру wallet.rs.
- Exact transport test test_rpc_wiring_spec_a.rs подтверждает, что `app.wallet.list_wallets` succeeds без auth context.
- `wallet.session.lock_wallet` объявлен без session/password в wallet.rs, зарегистрирован напрямую в wallet_dispatcher_wiring_register.rs, и service method просто лочит state по `wallet_id` в wallet_service_session_guards.rs.
- Прямой unit-path показывает, что lock immediately blocks export в wallet_impl_tests.rs.

Что не подтверждено до конца:

- Нет отдельного runtime transport test, где внешний клиент вызывает `wallet.session.lock_wallet` и получает success без дополнительного policy layer.

Почему это важно:

- Если runtime policy допускает тот же caller class к `wallet.*`, что и к `app.*`, тогда attacker может:
  1. вызвать `app.wallet.list_wallets`,
  2. получить valid wallet ids,
  3. вызвать `wallet.session.lock_wallet`,
  4. сорвать экспорт/seed/derive для активных wallets.
- Даже если внешний transport это блокирует, на компонентной границе API already weakly designed: sensitive state transition не требует possession of session.

Proof path:

1. `app.wallet.list_wallets` wired без auth params: app.rs.
2. Returned payload includes `PersistWalletInfo.id`: wallet.rs.
3. `wallet.session.lock_wallet` accepts only `PersistWalletId`: wallet.rs.
4. Service lock path has no `verify_session*` call before state mutation: wallet_service_session_guards.rs.

Failure path:

1. Caller lists wallets without session.
2. Caller selects `wallet_id`.
3. Caller hits lock RPC.
4. Legit user attempts export or another session-gated operation and receives locked-state denial.

Test evidence:

- Тест `cargo test -p z00z_wallets --test test_rpc_wiring_spec_a app_wallet_list_wallets_accepts -- --exact` прошёл.
- Existing unit evidence for lock side effect is in wallet_impl_tests.rs.

> [!NOTE]
> В текущем evidence set это не “полностью спекулятивный DoS”, но и не полный remote exploit proof. Точная формулировка должна оставаться такой: `enumeration is transport-confirmed; lock is auth-free at the RPC/component boundary; end-to-end runtime reachability of lock needs one additional transport test`.

**Critical User Journeys**

1. Import existing `.wlt` by path: discover identity, copy file, lock state created, later unlock by id.
2. Import existing `.wlt` by bytes: discover identity, copy file, unlock by id, then derive/export.
3. Unlock existing wallet and derive address after runtime env/config change.
4. Enumerate wallets via app RPC, then attempt disruptive wallet state transition via session RPC.

**State Transitions That Matter**

- `file only -> discovered -> managed locked wallet -> unlocked session -> derived address`
- `unlocked session -> lock_wallet -> locked state -> export/seed/derive denial`
- `persisted identity(devnet) + runtime identity(mainnet) -> unlock/derive divergence`
- `bytes-import without cached identity -> unlock fallback -> runtime identity selected`

**Positive Scenarios**

- Storage layer correctly rejects network/chain mismatch in test_redb_wlt_open.rs.
- `show_seed_phrase` path is session/password/confirmation aware and has precheck logic in wallet_service_session_guards.rs.
- Export-support paths already know how to recover persisted identity via wallet_service_store_support.rs.

**Negative Scenarios**

- `open_wallet_source(Path)` caches mixed identity fields.
- `open_wallet_source(Bytes)` omits identity cache entirely.
- Derive/address path is not pinned to persisted wallet identity.
- Lock transition is not bound to session ownership.

**Cryptographic / Security Invariants**

- A wallet’s `network/chain` must come from persisted metadata after discovery, not from mutable runtime env.
- Any address emitted for a wallet must be encoded under that wallet’s persisted chain.
- Sensitive state transitions on a wallet should require proof of current authorization, ideally possession of the active session or fresh password.
- Enumeration of stable wallet identifiers should not be wider than the set of principals allowed to mutate wallet state.

**Measurable Pass/Fail Criteria**

1. `PASS` only if all open/import paths write exactly `discovery.network` and `discovery.chain` into the in-memory identity cache.
2. `PASS` only if unlock/derive/address/export read wallet identity through one persisted-identity resolver, never directly from runtime env for an already-known wallet.
3. `PASS` only if a transport-level test proves `wallet.session.lock_wallet` rejects callers lacking a valid session, or the method signature itself is changed to require session ownership.
4. `PASS` only if a mismatch test exists for both `WalletSource::Path` and `WalletSource::Bytes` under non-default runtime chain/network.
5. `FAIL` if `resolve_wallet_chain_type_checked()` remains in per-wallet derive/address paths without persisted-identity reconciliation.

**Recommended Fix Direction**

1. Replace per-wallet runtime identity reads with a single persisted-identity resolution path, reusing wallet_service_store_support.rs.
2. Make `wallet.session.lock_wallet` session-bound, or split it into authenticated self-lock and privileged admin-lock with explicit authorization semantics.
3. Add end-to-end tests for:
   - imported wallet under runtime chain drift,
   - bytes import under drift,
   - transport denial of unauthenticated `wallet.session.lock_wallet`.

> [!TIP]
> Самый короткий безопасный refactor здесь не “чинить derive отдельно”, а централизовать wallet identity resolution и запретить любому per-wallet path читать `Z00Z_WALLET_*` после того, как `wallet_id` уже известен.

**Verification**
Я заново прогнал три точечных exact tests:

- `test_open_fails_identity_mismatch`: passed
- `derive_public_key_rejects_invalid_chain_config`: passed
- `app_wallet_list_wallets_accepts`: passed

Прямого E2E transport proof для `wallet.session.lock_wallet` в этой сессии не найдено; это и есть главный caveat отчёта.

Если нужно, следующим сообщением я могу:

1. превратить это в короткий PR-style review с 2 findings и residual risk,
2. или сразу предложить минимальный patch plan для исправления обоих findings без лишнего рефакторинга.

---



# аудит безопасности для  z00z_simulator

Перехожу на новый аудит по z00z_simulator: сначала соберу архитектурный и trust-boundary контекст, чтобы не делать pattern-only выводы, затем сведу подтверждённые findings в формальный отчёт.

**Findings**

> [!IMPORTANT]
> Severity summary for z00z_simulator: `0` confirmed `🔴 CRITICAL`, `0` confirmed `🟠 HIGH`, `1` confirmed `🟡 MEDIUM`, `1` confirmed `🔵 LOW`, `2` `⚪ INFO`.

1. `🟡 MEDIUM` Stage 2 persists wallet passwords, seed phrases, and receiver secret material in plaintext as part of the normal simulator contract.  
   Evidence: stage_2.rs unconditionally writes wlt_secrets_debug.md; artifacts.rs serializes `password`, `seed_phrase`, and `receiver_secret_hex`; scenario_design.yaml requires this file to exist; test_scenario1_stage_surface.rs enforces that requirement. The crate already has a private-file primitive in atomic_write.rs, but the secrets table uses plain `write_file`. The contrast is explicit: artifacts.rs rejects leaked passwords and seed phrases in logs, while the simulator deliberately writes them to disk. A real generated artifact exists at wlt_secrets_debug.md.
   Why not `HIGH`: the strongest evidence points to simulator/test exposure-at-rest, not a demonstrated attacker-reachable production path. I also ran an adversarial recheck with the `Doublecheck` subagent; it agreed this is supported but better classified as `MEDIUM` than `HIGH`.

2. `🔵 LOW` The runner recursively deletes the configured output directory without sandbox validation.  
   Evidence: runner.rs calls `reset_outputs_dir(&cfg.outputs.dir)` before execution; runner.rs turns the string into a `PathBuf` and deletes it with `remove_dir_all`; file_read.rs is just a thin wrapper over `std::fs::remove_dir_all`; test_wallet_integration.rs shows tests patch `cfg.outputs.dir` dynamically.  
   This is a real destructive footgun, but on current evidence it is mainly an operational risk that requires control over config or API invocation.

3. `⚪ INFO` The receive/claim split is intentionally explicit and is a good control boundary.  
   Evidence: runner_contract_table.in defines stage 7 as a report-only receive bridge that does not mutate claimed state, and runner_contract_table.in defines stage 8 as the explicit claim transition. This is the right shape for auditable state transition boundaries.

4. `⚪ INFO` The checkpoint proof path has dedicated end-to-end binding checks across stages 9–12.  
   Evidence: runner_contract_table.in defines the bundle/checkpoint handoff stages, mod.rs persists `exec_input_id_hex` into the bridge, and test_stage8_proof_path.rs verifies that `exec_input`, `draft`, final publication, `cp_proof`, and `checkpoint_id` stay bound.

**Critical Flows**

> [!WARNING]
> The most important end-to-end path is not just “scenario runs ok”. It is: wallet creation → secret display/export checks → claim publish → Bob receive bridge → explicit claim → checkpoint bridge build → storage-backed apply → finalized publication.

1. Wallet lifecycle proof path.  
   Use stage_2.rs, scenario_design.yaml, and the exact passing test `cargo test -p z00z_simulator --release --features test-fast --test test_wallet_integration stage2_rpc_no_secrets -- --exact --nocapture`.
   Pass criteria:

- `list_wallets` returns all expected actors.
- recovered seed phrase matches the creation response.
- address listing works before lifecycle backgrounding and is rejected after backgrounding.
- restart determinism reproduces the same derived public key.
- RPC log contains no plaintext password or seed phrase.
  Fail criteria:
- any mismatch in recovered seed phrase;
- address listing succeeds after lifecycle lock;
- RPC log contains password or mnemonic.

2. Receive/claim state transition.  
   Use runner_contract_table.in.  
   Pass criteria:

- stage 7 only builds Bob’s receive handoff and keeps claimed state unchanged;
- stage 8 alone performs the claim mutation and writes the downstream leaf/snapshot artifacts.
  Fail criteria:
- claimed state changes during stage 7;
- stage 8 succeeds without a valid receive handoff.

3. Checkpoint/proof binding path.  
   Use mod.rs and test_stage8_proof_path.rs.  
   Pass criteria:

- `checkpoint_s7.json` and `checkpoint_s8.json` carry the same `exec_input_id_hex` and `draft_id_hex`;
- final `checkpoint_id_hex` matches the link artifact;
- `exec.txs()[0].tx_proof()` equals `art.cp_proof()`;
- audit references the same checkpoint id as the link artifact.
  Fail criteria:
- drift between `exec_input_id_hex`, `draft_id_hex`, or `checkpoint_id_hex`;
- `cp_proof` diverges from the transaction proof bytes;
- artifact/link/audit chain is incomplete.

4. Negative-path expectations that should stay enforced.  
   Use scenario_design.yaml and the stage/test surface already present in the crate.
   Required failures:

- `list_addresses` rejected after lifecycle backgrounding;
- missing or tampered handoff artifacts block later stages;
- missing `exec_input` blocks stage 10 publish path;
- final publication must not claim success if artifact/link/audit files are absent.

**Assessment**

> [!CAUTION]  
> The main confirmed risk is not cryptographic unsoundness. It is secret-handling policy inside the simulator itself.

The crate has a reasonably strong proof-binding story for stages 7–12, and the wallet/log privacy checks are more disciplined than average. The weak point is that Stage 2 normalizes plaintext secret persistence as a first-class artifact even though the same codebase already treats those values as too sensitive for logs. That is the inconsistency worth fixing first.

**Validation**

I ran two exact release-style checks:

1. `cargo test -p z00z_simulator --release --features test-fast --test test_wallet_integration stage2_rpc_no_secrets -- --exact --nocapture`  
   Result: passed. This confirms the RPC log privacy control is live.

2. `cargo test -p z00z_simulator --release --features test-fast --test test_scenario1_stage_surface test_scenario1_stage_surface -- --exact --nocapture`  
   Result: passed. This confirms wlt_secrets_debug.md is encoded as an official stage-2 contract, not incidental debug drift.

Next natural steps:

1. Remove plaintext secret persistence from the default stage-2 contract and gate it behind an explicit debug-only path that uses private file permissions.
2. Add path guardrails around runner.rs so output reset is constrained to a known sandbox root.
3. If you want, I can turn this into a full polished markdown audit file under reports with the same severity table, alerts, and pass/fail matrix.
