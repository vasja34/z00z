# Phase 059: Core-Upgrade - Context

**Gathered:** 2026-06-16
**Status:** Ready for source audit and detailed planning

<domain>
## Phase Boundary

Phase 059 turns `059-TODO.md` from a whitepaper-grade object model into a
crate-by-crate implementation plan for Assets, Vouchers, and Rights across
`z00z_core`, `z00z_storage`, `z00z_wallets`, and `z00z_simulator`.

The phase is not a narrow core-only addition. The new core object classes have
wide-front impact: storage proof semantics, wallet persistence and receiver
safety, simulator scenarios, runtime admission, validator checks, watcher
alerts, and tests must all be expanded together.

The phase must preserve the current Z00Z design foundation:
- no duplicate authority layers;
- no parallel storage tree for vouchers or rights;
- no wallet-only source of truth;
- no value-bearing rights;
- no arbitrary custom actions on native cash;
- no concept drift from the Asset/Voucher/Right triad in `059-TODO.md`.

</domain>

<decisions>
## Implementation Decisions

### Scope And Source Of Truth

- **D-01:** `.planning/phases/059-Core-Upgrade/059-TODO.md` is the canonical
  Phase 059 planning inventory. Planning must cover it section by section and
  must not rename, soften, or skip its task meaning.
- **D-02:** Phase 059 must treat Assets, Vouchers, and Rights as sibling
  settlement objects under one generalized settlement state. Vouchers and
  Rights must not be implemented as nested data inside Assets.
- **D-03:** `z00z_core`, `z00z_storage`, `z00z_wallets`, and `z00z_simulator`
  are all in scope. `z00z_runtime`, `z00z_rollup_node`, watcher/admission
  surfaces, docs, and test gates are integration scope where they consume or
  validate settlement objects.
- **D-04:** The implementation must be planned at micro-level. Every paragraph
  of `059-TODO.md` must map to a module, invariant, test, simulation, explicit
  deferral, or non-goal.

### Object Semantics

- **D-05:** `Asset` means final spendable value. It is clean native cash.
  Asset balances are the only wallet-visible spendable cash balance.
- **D-06:** `Voucher` means conditional value claim over Z00Z. It may represent
  reserved or redeemable value, but it is not final cash until redeemed through
  a validator-checked action.
- **D-07:** `Right` means authority without value. Rights can authorize bounded
  actions, delegation, consumption, revocation, expiry, or challenge paths, but
  must contribute zero to value conservation.
- **D-08:** `CashPolicy` for native assets is fixed and narrow. Native cash must
  not accept arbitrary `ActionPool` semantics.
- **D-09:** `VoucherPolicy`, `ActionPool`, and richer policy descriptors are
  target architecture names. Planning must distinguish live code from target
  semantics and avoid claiming missing code already exists.

### Genesis Strategy

- **D-10:** Keep `z00z_core::genesis` as the single genesis orchestration
  boundary because chain identity, seed handling, deterministic outputs, weak
  seed checks, exports, and cross-object genesis invariants are shared.
- **D-11:** Inside that single boundary, use per-object generators:
  `genesis_assets`, existing `genesis_rights`, new `genesis_vouchers`, and new
  policy/action-pool generation support. Each object class gets its own typed
  birth semantics without creating separate genesis authorities.
- **D-12:** Assets keep finite-supply genesis semantics based on class, serials,
  nominal value, and policy flags.
- **D-13:** Rights use authority-instance genesis semantics: right class, scope,
  holder/control/beneficiary fixtures, quotas or use nonce, validity window,
  policy ids, payload commitment, and metadata. They never mint or reserve
  value.
- **D-14:** Vouchers use conditional-claim genesis semantics. Most vouchers
  should be created by runtime issuance actions after genesis, but genesis must
  support initial bootstrap vouchers when explicitly configured. Every genesis
  voucher must bind to backing/reserve evidence or a genesis-reserve source.
- **D-15:** Policy generation is content-addressed and deterministic. Policies,
  action pools, and condition descriptors should have canonical hashes and
  exported descriptors so wallets, validators, storage witnesses, and
  simulator fixtures all bind the same semantics.

### Storage And Settlement

- **D-16:** `z00z_storage::settlement` remains the canonical storage authority.
  `SettlementStateRoot`, `SettlementPath`, proofs, and `SettlementLeaf` must be
  extended in place.
- **D-17:** Current live leaf families are `Terminal(TerminalLeaf)` and
  `Right(RightLeaf)`. Voucher support must extend the same leaf-family model,
  likely through `SettlementLeaf::Voucher(VoucherLeaf)` plus corresponding
  `SettlementLeafFamily::Voucher`, encoding tags, proof markers, batch proof
  tags, nonexistence proofs, recovery, journal, and serialization inspection.
- **D-18:** Committed state stores commitments, policy hashes, lifecycle state,
  roots, path bindings, counters, and public verifier data. Wallet payloads
  store secrets/openings/private metadata. Witnesses carry descriptors, proofs,
  signatures, attestations, and selected action data.
- **D-19:** Typed remove/create delta across assets, vouchers, and rights is the
  canonical execution shape. Conservation is checked across value-bearing input
  objects and output objects; rights are excluded from value totals.
- **D-20:** `FeeEnvelope` remains a separate processing-support boundary. Fees
  must not smuggle value into Rights or authority into Vouchers.

### Wallet Model

- **D-21:** Wallet structures must move from asset-only persistence toward a
  typed owned-object inventory while preserving asset balance projections for
  existing spendable-cash APIs.
- **D-22:** `WalletAssetStore`, `OwnedAssetPayload`, receive scanning, reserve
  state, quarantine state, and RPC transfer paths must either be generalized or
  paired with equivalent Voucher and Right stores behind one object inventory
  facade.
- **D-23:** Unknown or unavailable policy descriptors must quarantine the object.
  Quarantined vouchers and rights must not appear as spendable cash; unknown
  assets must also remain non-spendable until accepted by the existing policy
  rules.
- **D-24:** Wallet UI/RPC/service boundaries must present three projections:
  cash balance for Assets, conditional claims for Vouchers, and authority
  inventory for Rights.
- **D-25:** Wallet transaction/package builders must choose an action from the
  relevant policy/action pool, bind live inputs and planned outputs, bind
  required rights, attach policy descriptors, attach proofs/attestations, and
  produce validator-checkable witness data.

### Simulator Model

- **D-26:** The simulator must stop proving only asset transfer paths. It must
  validate end-to-end object flows for all object classes and their combined
  interactions between Alice, Bob, and Charlie.
- **D-27:** Simulator scenarios must include at minimum: Asset transfer,
  Voucher issue/offer/accept/reject, Voucher transfer where allowed, Voucher
  full redeem, Voucher partial redeem, Voucher refund, Voucher expiry, Right
  grant, Right transfer/delegation where allowed, Right consume, Right revoke,
  Right expiry, Right challenge, fee support, and negative invalid-action
  cases.
- **D-28:** Combined simulator scenarios must include voucher actions that
  require rights, right delegation that authorizes a downstream voucher action,
  and failures where the right is missing, expired, out of scope, consumed, or
  revoked.
- **D-29:** Existing staged `scenario_1` should be adapted rather than replaced:
  genesis/config stages, transaction preparation, bundle/checkpoint stages,
  scan/apply stages, Charlie handoff stages, HJMT examples, reports, and
  design/config YAML must stay synchronized.

### Runtime, Validators, Watchers

- **D-30:** Validators must verify object presence, policy/template match,
  action membership, required rights and scope, signatures/control keys,
  attestations where applicable, quota/expiry, typed deletion/creation,
  conservation, and fee support separation.
- **D-31:** Aggregators/runtime planners may route and package work, but must
  not become a second semantic authority beside storage and validator checks.
- **D-32:** Watchers must detect unknown policy usage, invalid voucher backing,
  rights used as value, stale roots, invalid lifecycle transitions, replay,
  duplicate redemption, expired object use, and acceptance/refund boundary
  violations.

### Tests And Verification

- **D-33:** Tests must expand by object class and by interaction class. It is
  insufficient to add isolated unit tests for new structs.
- **D-34:** Core tests must cover parsing, schema validation, deterministic
  genesis, policy hash stability, impossible config rejection, genesis exports,
  and cross-object invariants.
- **D-35:** Storage tests must cover encoding/decoding, leaf family tags,
  proof/nonexistence proof validation, model/store equivalence, apply handoff,
  batch proof compatibility, recovery/journal, typed deltas, conservation, and
  failure cases for all object families.
- **D-36:** Wallet tests must cover persistence, migrations, scan/receive,
  quarantine, reservation, package building, accepted/rejected voucher states,
  right inventory states, spendable cash projection, and RPC/service behavior.
- **D-37:** Simulator tests must include Alice/Bob/Charlie full paths and
  release-mode E2E evidence. The simulator must surface problems and report
  concrete fixes, not only demonstrate happy paths.

### Second-Pass Source-Audit Expansion

- **D-38:** The next artifact should be a source audit before numbered plans.
  It must list every live code path that assumes only assets, only terminal
  leaves, or only terminal/right leaf families. Do not jump directly from this
  context to implementation.
- **D-39:** The audit must separate three columns for every item: `live`,
  `target`, and `migration concern`. This is mandatory because Phase 059 uses
  target terms such as `VoucherPolicy` and `ActionPool` that are not yet live
  implementation surfaces.
- **D-40:** Existing rights code is live enough to reuse, but not complete
  enough to treat as finished. `RightsConfigEntry`, `GenesisRightRecord`,
  `GenesisRightLeaf`, `RightLeaf`, `RightAction`, and right proof support must
  be audited for missing interactions before any rename or broad refactor.
- **D-41:** Existing voucher code should be assumed absent until the source
  audit proves otherwise. Planning must create voucher semantics explicitly
  rather than routing vouchers through asset metadata or wallet-only payloads.
- **D-42:** Current `GenesisConfig` requires `rights` and validates
  `rights.is_empty()` as an error. Phase 059 must decide compatibility for
  existing configs before adding `vouchers` and `policies`; adding new required
  arrays without defaults can break current simulator and genesis fixtures.
- **D-43:** Current wallet `ObjectKindId` has `OwnedAsset` but no owned voucher
  or owned right kinds. Planning must decide whether to add `OwnedObject` as a
  new generalized payload kind or add `OwnedVoucher` and `OwnedRight` kinds
  behind a shared query facade. Reusing `OwnedAsset` for all classes is
  rejected.

### Object Shape Requirements

- **D-44:** `VoucherLeaf` is a target storage leaf and should be designed as a
  committed conditional-claim object. Minimum semantics: object id/terminal id,
  issuer commitment, holder commitment, beneficiary commitment, backing
  commitment or reserve reference, face value, remaining value, policy id,
  action pool id, lifecycle state, validity window, acceptance/refund metadata,
  replay nonce, and disclosure/audit commitments.
- **D-45:** Voucher value fields must be explicit enough for conservation:
  validators need to know which amount is being redeemed, which residual claim
  remains, and which backing/reserve commitment is consumed or preserved.
- **D-46:** Voucher holder and beneficiary are separate concepts. Planning must
  support voucher transfer where policy allows without silently changing the
  beneficiary or refund authority.
- **D-47:** `RightLeaf` remains zero-value. Any field named budget, fee, payer,
  sponsor, reserve, amount, nominal, or backing belongs outside right committed
  state unless source audit proves it is only metadata and cannot affect value.
- **D-48:** Asset, Voucher, Right, and FeeEnvelope must stay different object
  roles: final value, conditional value, authority, and processing support.
  Any implementation path that makes one role substitute for another is a
  design-foundation violation.

### Policy And Action Model

- **D-49:** Policies should be content-addressed descriptors with stable
  canonical bytes and hashes. Planning should introduce or identify concrete
  types for policy id, action id, condition descriptor, and action-pool id.
- **D-50:** Native cash policy stays special: asset actions are fixed cash
  actions, not arbitrary user policy. Voucher and right policies may use action
  pools, but those action pools must be deterministic and validator-readable.
- **D-51:** Core-safe MVP condition classes are Tier 0 deterministic and Tier 1
  verifier-safe attested conditions. Registry/oracle/subjective conditions must
  be deferred unless represented as deterministic descriptor commitments plus
  explicit verifier-safe attestations.
- **D-52:** Every policy descriptor must declare allowed input object families,
  allowed output object families, required rights, required signatures,
  lifecycle preconditions, lifecycle postconditions, expiry rules, replay keys,
  and conservation contribution.
- **D-53:** Unknown policy descriptors are fail-closed for validators and
  quarantined for wallets. Simulator must contain unknown-policy cases for all
  object families, not only assets.

### Genesis And Publication Details

- **D-54:** Genesis should export object-class artifacts and a single settlement
  manifest. Existing artifacts include `genesis_rights.json` and
  `genesis_settlement_manifest.json`; Phase 059 should add voucher and policy
  artifacts without replacing the manifest role.
- **D-55:** Deterministic genesis derivation must be domain-separated by
  network, chain id, object class, object id, index, root generation, and
  policy/action descriptor hash. Voucher derivations must not reuse right
  derivation labels.
- **D-56:** Assets use finite-supply config. Rights use count/scope/validity
  config. Vouchers use backing/reserve/lifecycle config. Policies use
  descriptor config. These are separate typed sections under one genesis
  orchestration boundary.
- **D-57:** Genesis vouchers are bootstrap exceptions, not the ordinary voucher
  issuance path. Runtime issuance must be able to create vouchers after genesis
  under a validator-checked policy and reserve/backing rule.
- **D-58:** Genesis and simulator fixtures must include at least one clean
  native asset, one fully backed voucher, one transferable or redeemable
  voucher, one non-transferable voucher, one one-time right, one delegable
  right, and one expired/revoked negative fixture.

### Storage And Proof Details

- **D-59:** Adding voucher support requires touching all leaf-family boundaries:
  `SettlementLeaf`, human-readable serde tags, binary family tags, `From`
  conversions, `family_tag`, `terminal_id`, `serial_id`, `as_*` accessors,
  `check_path`, `SettlementLeafFamily`, marker leaves, proof blobs,
  nonexistence proofs, batch proof tags, HJMT cache encode/decode, fuzz seeds,
  recovery, listing, and docs.
- **D-60:** `SettlementPath` stays one shape. Vouchers and rights must still be
  addressable under `definition_id -> serial_id -> terminal_id`; any object
  class that lacks asset-style serial supply must still define a deterministic
  serial bucket rule.
- **D-61:** Storage tests must prove that an asset path cannot validate a
  voucher or right leaf, a voucher path cannot validate an asset or right leaf,
  and nonexistence proofs are family-specific.
- **D-62:** Typed execution deltas must explicitly list deleted objects,
  created objects, updated residual objects, attached fee envelope, selected
  action, policy descriptor hash, prior root, and expected new root.
- **D-63:** Storage must not interpret wallet payload secrets. It validates
  committed leaves, paths, roots, proof family, and typed deltas only.

### Wallet Details

- **D-64:** Wallet persistence should expose one object inventory facade and
  three projections: spendable cash assets, conditional voucher claims, and
  authority rights. Only the asset projection contributes to spendable balance.
- **D-65:** Wallet migration must preserve existing `OwnedAssetPayload` rows and
  `PAYLOAD_VERSION_OWNED_ASSET = 1`. New object support should be additive or
  versioned; it must not require rewriting all existing asset rows unless a
  reversible migration is explicitly planned and tested.
- **D-66:** Wallet scan/receive should classify settlement leaves by object
  family before attempting object-specific payload recovery. Asset scanning may
  continue to use stealth output recovery; voucher/right discovery needs
  separate payload-opening and policy-descriptor checks.
- **D-67:** Wallet package building must reject packages where a voucher is used
  as cash input, a right is used as value input, an unknown-policy object is
  treated as spendable, or a required right is present but out of scope,
  expired, revoked, consumed, or delegated incorrectly.
- **D-68:** Wallet RPC should not overload `asset.send`/`asset.receive` for
  voucher/right semantics. Planning should add typed RPC surfaces or an object
  RPC namespace while preserving existing asset methods as cash-only.

### Simulator Details

- **D-69:** Existing `scenario_1` remains the executable scenario home. It must
  grow object-family lanes while keeping stage order, config YAML, design YAML,
  reports, release gate, and runner contract synchronized.
- **D-70:** Stage 1 must emit genesis assets, rights, policies, vouchers, and a
  settlement manifest. Stage 4 must prepare typed object packages and witnesses,
  not only asset spend witnesses. Stage 5 must exercise receiver behavior by
  object family. Stage 6 must carry typed deltas into bundle/checkpoint logic.
  Stage 11 must scan/apply all object families. Stage 13 must include HJMT
  examples for all object leaf families and negative proof-family checks.
- **D-71:** Alice/Bob/Charlie scenarios must include positive and negative
  paths for all object interactions. A simulator path is incomplete if it only
  shows object creation without wallet persistence, validator acceptance or
  rejection, storage root update, scan/apply, and watcher/report evidence.
- **D-72:** Simulator artifacts should report detected problems and proposed
  fixes. Phase 059 evidence must not be only a success log; it must include
  failure artifacts for invalid policy, invalid backing, missing right, expired
  right, replayed right, double redemption, wrong family proof, and forced
  voucher acceptance.

### the agent's Discretion

The user requested comprehensive treatment rather than interactive narrowing.
Where exact schema names are not present in live code, planners may choose the
least disruptive names consistent with existing module conventions, but must
document each current-vs-target distinction and bind every change to a test.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 059 Source Material

- `.planning/phases/059-Core-Upgrade/059-TODO.md` - canonical Phase 059
  whitepaper/task inventory for Assets, Rights, Vouchers, policies, storage,
  wallet safety, simulator scope, and MVP boundaries.
- `.planning/phases/059-Core-Upgrade/059-Untitled Document.md.zip` - preserved
  user-provided source artifact in the existing phase directory; do not unpack
  or replace unless explicitly needed during source audit.

### GSD And Project State

- `.planning/GSD-Workflow.md` - Phase 059 planning workflow constraints and
  requirement that `059-TODO.md` is canonical planning inventory.
- `.planning/ROADMAP.md` - Phase 059 registration and roadmap position.
- `.planning/STATE.md` - current project/phase state.

### Architecture And Conventions

- `.planning/codebase/ARCHITECTURE.md` - crate ownership, layered topology,
  storage/wallet/simulator integration points.
- `.planning/codebase/STACK.md` - workspace technology, Rust version, storage,
  testing, and feature-gate facts.
- `.planning/codebase/TESTING.md` - canonical gates, simulator release lane,
  property/E2E test patterns.
- `.planning/codebase/CONVENTIONS.md` - naming, module ownership, facade,
  testing, and read-only vendor boundaries.
- `.github/requirements/Z00Z_DESIGN_FOUNDATION.md` - no concept drift, no
  duplicate authority layers, no ad-hoc protocol shortcuts.
- `.github/copilot-instructions.md` - repository-local engineering rules.
- `.github/instructions/rust.instructions.md` - Rust-specific local rules.

### Prior Phase Context

- `.planning/phases/000/056-HJMT-storage- aggregator/056-CONTEXT.md` - storage
  aggregator and settlement root context that Phase 059 must extend in place.
- `.planning/phases/000/057-HJMT-multi-aggregator/057-CONTEXT.md` - multi-
  aggregator routing/watchers context that Phase 059 must not duplicate.
- `.planning/phases/000/058-HJMT-benchmarks/058-CONTEXT.md` - benchmark and
  simulator evidence patterns inherited by Phase 059.

### Whitepapers And Design Docs Cited By Phase 059

- `docs/Z00Z-Main-Whitepaper.md` - base Z00Z economic/protocol model.
- `docs/Z00Z-Smart-Cash-Whitepaper.md` - smart-cash framing relevant to
  vouchers and policy/action semantics.
- `docs/Z00Z-UseCases-Whitepaper.md` - use cases for conditional claims,
  delegation, budgets, allowances, and receiver safety.
- `docs/Z00Z-Uniqueness-Whitepaper.md` - differentiators that constrain object
  model decisions.
- `docs/tech-papers/done/Z00Z-HJMT-Design.md` - current generalized HJMT
  settlement storage design.

### Live Code: Core

- `crates/z00z_core/src/genesis/genesis_config.rs` - current `GenesisConfig`
  has assets and rights, but no vouchers.
- `crates/z00z_core/src/genesis/genesis_rights.rs` - existing right genesis
  record and deterministic right generation logic.
- `crates/z00z_core/src/genesis/genesis_config_validate.rs` - current genesis
  schema validation surface.
- `crates/z00z_core/src/assets/right_config.rs` - current `RightsConfigEntry`,
  right classes, forbidden fee/budget keys, and right config parser.
- `crates/z00z_core/src/genesis/genesis_settlement_manifest.rs` - current
  genesis rights artifact and settlement manifest publication surface.
- `crates/z00z_core/src/genesis/mod.rs` - genesis facade/export boundary.

### Live Code: Storage

- `crates/z00z_storage/src/settlement/README.md` - live storage contract,
  current exported leaf families, root/proof boundary, and no legacy adapter
  rule.
- `crates/z00z_storage/src/settlement/root_types.md` - concise root and
  `SettlementLeaf` contract summary.
- `crates/z00z_storage/src/settlement/record.rs` - current `SettlementLeaf`,
  `RightLeaf`, action contexts, and fee record structures.
- `crates/z00z_storage/src/settlement/leaf.rs` - terminal/right leaf encoding
  tags and leaf serialization boundary.
- `crates/z00z_storage/src/settlement/proof.rs` - current
  `SettlementLeafFamily`, marker leaves, proof binding, and validation.
- `crates/z00z_storage/src/settlement/proof_batch.rs` - batch proof leaf-family
  tags and shared proof compatibility.
- `crates/z00z_storage/src/settlement/proof_batch_verify.rs` - batch proof
  verification family checks.
- `crates/z00z_storage/src/settlement/hjmt_cache.rs` - encoded family routing
  and durable cache implications.
- `crates/z00z_storage/src/settlement/store.rs` - store/apply API and right/fee
  operations.
- `crates/z00z_storage/src/settlement/fee_envelope.rs` - separate fee-support
  validation boundary.

### Live Code: Wallets

- `crates/z00z_wallets/src/db/redb_wallet_store/owned_assets.rs` - current
  asset-owned persistence, filter, reserve, restore, scan-batch, and quarantine
  foundation.
- `crates/z00z_wallets/src/db/redb_wallet_store/tables.rs` - current
  `ObjectKindId`, `OwnedAssetStatus`, `OwnedAssetSource`, `OwnedAssetPolicy`,
  and `OwnedAssetPayload` definitions.
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs`
  - current scan/receive flow and asset cache sync.
- `crates/z00z_wallets/src/adapters/rpc/methods/asset_rpc_history.rs` -
  current in-memory asset quarantine support.
- `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_transfer.rs`
  - current receive/send RPC implementation and quarantined asset behavior.

### Live Code: Simulator

- `crates/z00z_simulator/src/scenario_1/stage_1.rs` - genesis/config scenario
  entrypoint.
- `crates/z00z_simulator/src/scenario_1/stage_4.rs` - transaction
  preparation stage entrypoint.
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_preparation_core.rs`
  - transaction package preparation core.
- `crates/z00z_simulator/src/scenario_1/stage_5.rs` - existing transfer lane.
- `crates/z00z_simulator/src/scenario_1/stage_5_utils/transfer_lane_impl.rs`
  - current asset-centric receive/claim transfer lane.
- `crates/z00z_simulator/src/scenario_1/stage_6.rs` - bundle/checkpoint stage.
- `crates/z00z_simulator/src/scenario_1/stage_11.rs` - scan/apply and Charlie
  handoff stage.
- `crates/z00z_simulator/src/scenario_1/stage_13.rs` - HJMT example/flow stage.
- `crates/z00z_simulator/src/scenario_1/runner_verify.rs` - current genesis
  rights artifact and scenario contract verification surface.
- `crates/z00z_simulator/src/scenario_1/scenario_config.yaml` - scenario config
  that must gain object-policy coverage.
- `crates/z00z_simulator/src/scenario_1/scenario_design.yaml` - scenario design
  contract that must stay synchronized with executable stages.

</canonical_refs>

<code_context>
## Existing Code Insights

### Current Live Object Support

- Core genesis currently supports assets and rights in `GenesisConfig`; voucher
  config/generation is absent.
- Right genesis already exists and should be extended, not replaced.
- Storage already has generalized settlement roots, paths, terminal leaves,
  right leaves, and fee envelope support.
- Storage does not yet have a voucher leaf family.
- Wallet persistence and RPC receive/send flows are asset-centric.
- Simulator stages are built around existing asset/HJMT transfer lanes and must
  be widened rather than forked.

### Second-Pass Live Seams

- Core `GenesisConfig` has `assets: Vec<AssetConfigEntry>` and
  `rights: Vec<RightsConfigEntry>`, but no `vouchers` or standalone `policies`
  section.
- Core `validate_rights_schema` currently rejects empty rights arrays. Planner
  must decide whether Phase 059 keeps rights mandatory, makes rights optional,
  or adds fixture defaults for simulator configs.
- Core `RightsConfigEntry` already forbids fee/budget/payer/sponsor/support
  keys. Keep that pattern and extend it to reject any value-bearing right
  schema.
- Core `generate_genesis_rights` already derives terminal ids, definition ids,
  policy ids, payload commitments, and use nonces with network-aware domains.
  Voucher and policy generators should follow the pattern but use separate
  labels and object-specific invariants.
- Storage `SettlementLeaf` currently serializes only `Terminal` and `Right`
  human-readable families and only terminal/right binary family tags.
- Storage `SettlementLeafFamily`, `TerminalFamilyTagV1`, and `LeafFamilyTagV1`
  currently expose only asset/terminal and right variants.
- Storage HJMT cache currently encodes leaf families as `1 = Terminal` and
  `2 = Right`; voucher addition requires durable tag compatibility planning.
- Wallet `ObjectKindId` currently assigns `OwnedAsset = 21` and has no voucher
  or right wallet object kind.
- Wallet `OwnedAssetStatus` has `Spendable`, `PendingSpend`, `Spent`,
  `PendingReceive`, `Quarantined`, and `Archived`; voucher/right statuses need
  separate lifecycle vocabularies rather than reusing spend status blindly.
- Wallet `OwnedAssetPayload` embeds `AssetWire`, asset ids, asset definition
  ids, spend refs, scan refs, receive refs, and an asset policy quarantine
  field. It is not a safe container for rights or vouchers without a typed
  wrapper or new payload kind.
- Wallet receive flow calls `recv_claim_asset`, returns `Vec<Asset>`, and
  lists spendable asset rows. All of these are asset-only seams.
- Wallet RPC quarantine is currently asset-id based and in-memory in the asset
  RPC layer. Phase 059 must define durable object quarantine semantics for
  vouchers and rights.
- Simulator Stage 4 `tx_preparation_core` builds asset spend membership
  witnesses from `AssetWire`/`TerminalLeaf`. Voucher/right packages need their
  own witness preparation and proof validation lanes.
- Simulator Stage 5 transfer lane uses `Asset`, `TerminalLeaf`, and
  `RuntimeReceiveAssetResponse`. Object-family transfer and acceptance flows
  must not be forced through that asset response type.

### Reusable Assets

- `GenesisConfig` parsing/validation is the correct entry point for adding
  `vouchers` and `policies` fields.
- `genesis_rights.rs` is the template for deterministic non-asset object
  generation.
- `SettlementLeaf`, `SettlementLeafFamily`, `SettlementPath`, and
  `SettlementStateRoot` are the extension points for voucher storage.
- `FeeEnvelope` already protects the fee boundary and should be used to prevent
  fee semantics from leaking into rights.
- `WalletAssetStore` and `OwnedAssetPayload` show existing RedB persistence,
  reservation, quarantine, restore, and scan patterns.
- `scenario_1` stages already provide Alice/Bob/Charlie flow structure and
  checkpoint/report plumbing.

### Established Patterns

- Public crate facades should remain narrow; do not expose backend internals.
- Storage owns proof and root semantics; runtime and simulator consume them.
- Wallet is local state plus witness construction, not protocol truth.
- Simulator evidence must be executable and synchronized with config/design
  YAML.
- Tests should prefer real domain objects, temp dirs, helpers, and property
  coverage over dedicated mocks.
- Heavy simulator evidence belongs in existing gated/release lanes.

### Integration Points

- Core emits deterministic object definitions, policy descriptors, and genesis
  object records.
- Storage stores and proves typed settlement leaves and validates typed deltas.
- Runtime/aggregators assemble admission batches and pass route-bound semantic
  handoff to storage.
- Wallet scans, persists, classifies, quarantines, and builds action packages.
- Validators verify descriptors, object state, rights, attestation, lifecycle,
  deletion/creation, and value conservation.
- Watchers monitor roots, packages, policies, and lifecycle alerts.
- Simulator exercises complete object paths and writes evidence.

</code_context>

<specifics>
## Specific Ideas

### Required Object State Machines

Asset:
- genesis or valid creation;
- receive/accept into spendable cash only when policy and proof are known;
- transfer/split/merge/burn only through fixed cash policy;
- deletion and creation must conserve final value.

Voucher:
- create/issue with backing or reserve evidence;
- offer to receiver;
- accept or reject;
- transfer if policy permits;
- redeem fully;
- redeem partially with residual voucher or explicit remainder handling;
- refund under bounded conditions;
- expire;
- reject double redemption, unbacked issuance, invalid backing, stale policy,
  expired voucher use, and receiver-unsafe forced acceptance.

Right:
- create/grant;
- transfer or delegate if policy permits;
- consume or mark used;
- expire;
- revoke if allowed;
- challenge if allowed;
- reject value fields, fee/budget semantics, over-scope use, stale holder,
  invalid delegation, consumed nonce reuse, and unauthorized action binding.

### Required Cross-Object Interaction Set

- Asset pays final value with no voucher/right semantics attached.
- Voucher is backed by Asset value or explicit reserve evidence.
- Voucher redemption creates Asset output and deletes or updates Voucher input.
- Partial voucher redemption creates Asset output plus residual Voucher output
  when policy allows.
- Right authorizes a Voucher action but never funds it.
- Right authorizes a governance/service/action path but never changes value
  conservation totals.
- FeeEnvelope pays or sponsors fees without becoming voucher backing or right
  authority.
- Unknown policy prevents wallet spendability and validator acceptance.

### Required Alice/Bob/Charlie Simulator Paths

- Alice sends Asset to Bob; Bob sends Asset to Charlie.
- Alice issues Voucher to Bob; Bob accepts; Bob redeems fully into Asset.
- Alice issues Voucher to Bob; Bob partially redeems; Bob transfers or redeems
  residual depending on policy.
- Alice issues Voucher to Bob; Bob rejects; Alice refunds.
- Alice issues Voucher to Bob; Bob does nothing; expiry path is validated.
- Alice grants Right to Bob; Bob uses Right to authorize a Voucher redeem.
- Alice grants Right to Bob; Bob delegates to Charlie if policy permits;
  Charlie uses it once; replay fails.
- Alice grants Right to Bob; Alice revokes or Right expires; Bob/Charlie action
  fails.
- Alice attempts unbacked Voucher issuance; validator rejects.
- Bob attempts to treat Voucher as spendable cash; wallet/simulator blocks it.
- Bob attempts to treat Right as value; storage/validator/wallet reject.

### Required Module-Level Planning Questions To Answer In Source Audit

- Which existing asset-only wallet APIs must become typed object APIs, and which
  asset APIs should remain as cash projections?
- Which settlement proof APIs need a new voucher family tag and what migration
  compatibility is required?
- Which core policy/action descriptor types are already present, partially
  present, or missing?
- Which simulator stages should be extended directly versus split into helper
  modules while preserving the existing scenario contract?
- Which runtime/watchers currently assume `TerminalLeaf` or `RightLeaf` only?
- Which serialization tests must be added to prevent object-family ambiguity?

### Minimum Target Shapes

These names are target planning vocabulary. The source audit may refine names,
but it must preserve the semantics.

| Shape | Minimum required semantics |
|---|---|
| `AssetLeaf` / `TerminalLeaf` | Final value object; fixed cash policy; amount commitment; owner/receiver data; serial/nullifier path; no custom action pool. |
| `VoucherLeaf` | Conditional claim object; issuer/holder/beneficiary bindings; backing or reserve commitment; face and remaining value commitments; lifecycle state; policy/action ids; expiry/refund/redeem metadata; replay key. |
| `RightLeaf` | Authority object; class, scopes, holder/control/beneficiary commitments, payload commitment, validity/challenge windows, policy ids, use nonce; zero value. |
| `FeeEnvelope` | Processing support; payer/sponsor commitments, budget units, expiry, transition id, replay key, support ref; not an object class and not value/authority semantics. |
| `PolicyDescriptorV1` | Canonical bytes and hash, object-family inputs/outputs, action ids, condition descriptors, required rights, signatures, attestations, expiry/replay rules, conservation contribution. |
| `ActionPoolDescriptorV1` | Set of named action descriptors; must bind object family, allowed lifecycle transitions, required witnesses, and output construction rules. |
| `ObjectWitness` | Presence proof, prior root, leaf family, selected action, descriptor hash, delete/create/update delta, signatures, optional attestation, optional disclosure/audit artifacts. |
| `WalletOwnedObject` | Wallet-local payload envelope with object family, object id, policy availability, lifecycle status, private openings, labels, refs, checksum, quarantine reason. |

### Canonical Object Lifecycle Vocabulary

| Object | Wallet-visible states | Validator/storage states | Rejection states |
|---|---|---|---|
| Asset | detected, spendable, pending_spend, spent, quarantined, archived | live, deleted, created, fee_consumed | unknown policy, invalid proof, duplicate id, conservation mismatch |
| Voucher | offered, pending_accept, accepted, rejected, redeemable, partially_redeemed, redeemed, refunded, expired, quarantined | live, accepted, updated_residual, deleted, expired | unbacked, double redeem, forced acceptance, invalid refund, stale policy |
| Right | granted, held, delegated, consumed, revoked, expired, challenged, quarantined | live, transferred, consumed, revoked, expired, challenged | out of scope, expired, revoked, nonce replay, missing transition policy |
| FeeEnvelope | attached, accepted, expired, replayed | support-only validation record | budget mismatch, replay, expired support, missing payer/sponsor proof |

### Action Interaction Matrix

| Action | Inputs | Outputs | Required checks | Required evidence |
|---|---|---|---|---|
| Asset transfer | Asset | Asset outputs, optional fee | fixed cash policy, ownership, conservation | spend proof, membership proof, signatures |
| Asset split/merge | Assets | Assets | fixed cash policy, no custom action pool, conservation | amount proof, membership proof, delete/create delta |
| Voucher issue | Asset reserve or reserve commitment, policy | Voucher | backing exists, issuer authorized, receiver not forced to accept | reserve proof, policy descriptor, issuer signature |
| Voucher offer | Voucher | Voucher pending receiver decision | policy allows offer, receiver identity bound | voucher proof, offer envelope |
| Voucher accept | Voucher offer | Accepted Voucher | receiver consent, policy known, no hidden cash spendability | receiver signature, policy descriptor |
| Voucher reject | Voucher offer | Rejected/returned claim | receiver rejection allowed, issuer/refund path bounded | receiver rejection witness |
| Voucher transfer | Voucher, optional Right | Voucher with new holder | policy allows transfer, beneficiary/refund authority preserved | holder signature, optional right proof |
| Voucher full redeem | Voucher | Asset output, voucher deleted | backing consumed, amount equals remaining claim, no double redeem | voucher proof, backing proof, redemption nonce |
| Voucher partial redeem | Voucher | Asset output, residual Voucher | redeemed amount valid, residual value exact, replay prevented | voucher proof, residual descriptor, conservation proof |
| Voucher refund | Voucher | Asset/reserve returned to issuer/refund target | refund condition true, not arbitrary clawback | refund condition proof, lifecycle witness |
| Voucher expire | Voucher | expired/deleted Voucher or locked refund path | now past expiry, no accepted redemption conflict | time/root witness, policy descriptor |
| Right grant | issuer authority | Right | issuer scope, class config, zero value | issuer signature, policy ids |
| Right delegate | Right | Right with narrowed holder/control | policy permits, scope cannot widen | prior right proof, delegation signature |
| Right consume | Right | consumed/deleted Right | nonce unused, action bound, within validity | use nonce, selected action witness |
| Right revoke | Right | revoked/deleted Right | revocation policy present and authorized | revocation proof/signature |
| Right challenge | Right | challenged/updated Right | active challenge window and policy | challenge proof, disclosure as needed |
| Fee attach | FeeEnvelope | support record only | budget commitment, expiry, replay key, no object role confusion | fee envelope and payer/sponsor proof |

### Per-Crate Micro Planning Drilldown

| Crate/module | Micro tasks planner must cover |
|---|---|
| `z00z_core::assets` | Decide object-family public types, right/voucher config modules, policy/action descriptor home, identifier domains, error vocabulary. |
| `z00z_core::genesis::genesis_config` | Add or default `policies` and `vouchers`; preserve config compatibility; validate no empty/malformed target arrays unless intentionally required. |
| `z00z_core::genesis::genesis_config_validate` | Add voucher/policy schema validation; preserve weak-seed/network checks; reject value-bearing rights and unbacked vouchers. |
| `z00z_core::genesis::genesis_rights` | Keep current derivation logic; add missing tests for policy ids, challenge windows, forbidden value fields, and deterministic replay. |
| `z00z_core::genesis::genesis_vouchers` | New generator for bootstrap voucher records with reserve/backing, lifecycle state, policy/action ids, holder/beneficiary bindings, and collision checks. |
| `z00z_core::genesis::genesis_policies` | New generator/exporter for deterministic policy/action descriptors and descriptor hashes. |
| `z00z_core::genesis::genesis_settlement_manifest` | Extend manifest with policy and voucher artifacts while keeping current rights artifact semantics. |
| `z00z_storage::settlement::record` | Add voucher leaf, voucher action/context/errors, accessors, serde variants, binary decode, path checks, and leaf-family tests. |
| `z00z_storage::settlement::leaf` | Add stable voucher family tag and encoding; add roundtrip and unknown-tag tests. |
| `z00z_storage::settlement::proof` | Add `SettlementLeafFamily::Voucher`, marker leaf, family-specific nonexistence and inclusion validation. |
| `z00z_storage::settlement::proof_batch` | Add voucher terminal/leaf family tags without reusing asset/right tag values; update decode/reject tests. |
| `z00z_storage::settlement::hjmt_cache` | Extend family encode/decode and durable migration tests; prove old terminal/right rows still decode. |
| `z00z_storage::settlement::store` | Add voucher lifecycle operations or generic typed object operations; prevent wallet-only state transitions. |
| `z00z_storage::settlement::model` | Update semantic model and root determinism for all object families. |
| `z00z_wallets::db::redb_wallet_store` | Add object inventory kind(s), payload versions, indexes by object family/status/policy, migration/backcompat tests, durable quarantine. |
| `z00z_wallets::services::wallet` | Add object scan classification, voucher/right lifecycle operations, object package builder, and cash projection boundaries. |
| `z00z_wallets::adapters::rpc` | Keep asset RPC cash-only; add typed voucher/right/object RPC methods or namespace; expose quarantine and lifecycle states safely. |
| `z00z_wallets::backup` | Include voucher/right payloads and policy descriptor references without leaking secrets. |
| `z00z_runtime::validators` | Add policy/action/value/right/lifecycle verification; expose precise rejection reasons. |
| `z00z_runtime::aggregators` | Carry typed object packages and route-bound deltas without semantic authority. |
| `z00z_runtime::watchers` | Alert on invalid backing, wrong family proof, forced acceptance, replay, double redemption, and value-bearing rights. |
| `z00z_rollup_node` | Include object-family verdict/publication fields where checkpoint verification depends on them. |
| `z00z_simulator::scenario_1` | Expand stages and YAML contracts for assets/vouchers/rights/policies, positive paths, negative paths, and artifact evidence. |

### Wallet Adaptation Matrix

| Wallet seam | Current risk | Required Phase 059 behavior |
|---|---|---|
| Owned object persistence | `OwnedAssetPayload` can only represent assets safely. | Add object inventory facade with typed payloads or new object kinds. |
| Spendable listing | `list_spendable_assets` filters asset spend states. | Keep cash-only spendable list; add voucher/right inventory lists. |
| Receive scanning | `recv_claim_asset` returns assets. | Classify leaves by family, then use family-specific payload recovery. |
| Reservation | `reserve_asset_inputs` marks assets pending spend. | Add voucher action reservation and right use reservation without cash semantics. |
| Confirmation | `confirm_asset_spend` updates spent assets and new outputs. | Confirm typed deltas: asset spent/output, voucher residual/redeemed/refunded, right consumed/delegated/revoked. |
| Restore/import | Restores assets and payloads. | Restore object inventory plus policy descriptor availability and quarantine states. |
| Quarantine | Asset RPC has in-memory quarantine id set plus payload reason. | Make quarantine durable, family-aware, policy-aware, and excluded from balance. |
| RPC send/receive | Asset methods return asset response types. | Add typed voucher/right/object methods; do not overload asset methods. |
| Backup/export | Asset payloads and wallet state dominate. | Include typed object payloads, descriptor refs, and private openings with secret scans. |

### Simulator Expansion Matrix

| Scenario | Alice action | Bob action | Charlie action | Expected verdict |
|---|---|---|---|---|
| Clean asset path | sends asset to Bob | accepts/spends asset | receives asset | accepted; asset balance updates only. |
| Voucher full redeem | issues backed voucher to Bob | accepts and redeems | none | accepted; voucher deleted, asset output created. |
| Voucher partial redeem | issues backed voucher | redeems part | receives residual or asset if policy allows | accepted; residual voucher exact. |
| Voucher reject/refund | offers voucher | rejects | none | accepted; refund path bounded. |
| Voucher expiry | offers voucher | no accept/redeem | none | expiry accepted; late redeem rejected. |
| Voucher transfer | issues transferable voucher | transfers to Charlie | accepts/redeems | accepted only if policy permits transfer. |
| Non-transferable voucher | issues non-transferable voucher | attempts transfer | attempts redeem | rejected at transfer or redeem. |
| Right-gated redeem | grants redeem right to Bob | redeems voucher using right | none | accepted; right scope checked. |
| Right delegation | grants delegable right | delegates to Charlie | uses once | first accepted, replay rejected. |
| Revoked right | grants then revokes right | attempts action | none | rejected; watcher alert. |
| Expired right | grants short-lived right | attempts late action | none | rejected; expiry evidence captured. |
| Missing right | issues right-gated voucher | redeems without right | none | rejected; no storage state change. |
| Unbacked voucher | issues without reserve | tries accept/redeem | none | rejected by validator. |
| Wrong family proof | submits asset proof for voucher | none | none | rejected by proof family check. |
| Forced voucher acceptance | sends voucher as if cash | wallet refuses cash balance | none | rejected/quarantined; receiver safety preserved. |
| Fee support boundary | attaches fee envelope | performs action | none | accepted only if fee support separate from object semantics. |

### Test Expansion Grid

| Layer | Required positive tests | Required negative tests |
|---|---|---|
| Core config | assets/rights/vouchers/policies parse and validate; deterministic descriptor hashes. | empty or malformed required sections, duplicate ids, value-bearing right keys, unbacked voucher config, overflow. |
| Core genesis | deterministic assets, rights, vouchers, policies, manifest entries, collision-free ids. | derivation label reuse, terminal collision, wrong root generation, weak seed, missing backing. |
| Storage leaf codec | terminal/right/voucher serde and bincode roundtrip; stable family tags. | unknown tag, cross-family decode, family/path mismatch, malformed voucher lifecycle fields. |
| Storage proofs | inclusion, deletion, nonexistence, batch proof for all families. | wrong family proof, stale root, wrong marker, wrong batch tag, missing deletion fact. |
| Storage model/store | apply typed deltas and root determinism for mixed objects. | conservation mismatch, right value contribution, double redeem, residual mismatch. |
| Wallet DB | persist/list/restore asset/voucher/right payloads and indexes. | old payload breakage, checksum mismatch, unknown descriptor spendability, status transition violation. |
| Wallet services | scan classify, quarantine, reserve, package, confirm typed actions. | voucher counted as cash, right counted as value, expired/revoked/missing right, unknown policy. |
| Wallet RPC | cash-only asset methods; typed voucher/right/object methods. | asset RPC accepting voucher/right id, forced voucher acceptance, quarantine bypass. |
| Runtime validators | action membership, required rights, attestation, lifecycle, conservation. | unbacked issue, double redeem, replay, wrong scope, mutable descriptor. |
| Watchers | alert records for policy/backing/replay/family/lifecycle violations. | silent invalid packages, stale root acceptance, duplicate redemption not reported. |
| Simulator | Alice/Bob/Charlie positive paths for all object families. | all failure paths in simulator expansion matrix with persisted artifacts. |
| Fuzz/property | leaf codec, proof envelopes, amount/residual conservation, policy descriptor hash stability. | malformed inputs must fail closed and never panic. |

### Planning Order Constraint

1. Source audit and live-vs-target inventory.
2. Core schemas, policy descriptors, genesis generators, manifest changes.
3. Storage leaf family, proof, model, store, recovery, and serialization changes.
4. Runtime validator/aggregator/watcher typed package and verdict changes.
5. Wallet persistence, scan, package builder, RPC, backup/restore changes.
6. Simulator scenario/YAML/artifact expansion.
7. Tests and verification gates for every changed layer.
8. Docs and final anti-drift review.

Do not plan wallet or simulator first; they depend on core object descriptors
and storage proof-family semantics.

</specifics>

## Micro Coverage Map For `059-TODO.md`

| TODO section | Planning obligation |
|---|---|
| Key Terms Used In This Paper | Freeze glossary terms in plan; prevent target/live confusion. |
| 1. Why This Paper Exists | Treat phase as architecture correction, not feature append. |
| 1.1 Design Problem | Identify all places where value, authority, policy, and storage semantics are currently conflated or asset-only. |
| 1.2 Design Thesis | Preserve triad split across core, storage, wallet, validator, simulator. |
| 1.3 Reader Outcome | Produce plan that implementers can follow without reinterpreting object semantics. |
| 2. Position In The Z00Z Corpus | Cross-check main/smart-cash/use-case/HJMT docs before implementation. |
| 2.1 Corpus Role | Treat `059-TODO.md` as bridge from whitepaper to full spec. |
| 2.2 Current Maturity Versus Target Architecture | Mark every live-vs-target item in source audit before coding. |
| 3. Core Thesis: Asset, Voucher, And Right | Encode triad into type system, storage leaves, wallet views, simulator tests. |
| 3.1 The Minimal Triad | Implement only Asset/Voucher/Right as primary object classes for MVP. |
| 3.2 Why This Split Is Minimal | Reject merged encumbered-cash or value-bearing-right shortcuts. |
| 3.3 Why Voucher Is Not Redundant With Right | Plan Voucher as conditional value and Right as authorization. |
| 3.4 Cross-Object Binding Rules | Specify package/witness bindings for live inputs, outputs, selected action, policy, right references, roots, and disclosure data. |
| 4. Asset: Final Value And Cash Boundary | Keep asset APIs as cash-grade final value. |
| 4.1 What Asset Means | Document and test asset-only spendable balance. |
| 4.2 Why Asset Must Stay Clean | Reject arbitrary native asset action pools. |
| 4.3 Cash-Grade Invariants | Add conservation, ownership, serial/nullifier, and policy tests. |
| 4.4 What This Paper Does Not Claim About Assets | Avoid implementing voucher/right behavior as asset metadata. |
| 5. Voucher: Conditional Value, Not Dirty Cash | Add voucher type, lifecycle, policy, and receiver-safety tests. |
| 5.1 Economic Meaning | Voucher carries conditional claim, not immediate cash. |
| 5.2 Fully Backed Vouchers | Require backing/reserve evidence and reject unbacked vouchers. |
| 5.3 Voucher Is Not Final Cash | Wallet and validator must not count vouchers as spendable cash. |
| 5.4 Voucher Lifecycle | Implement/test create, offer, accept/reject, redeem, refund, expire. |
| 5.5 Partial Redeem | Implement/test value split between asset output and residual voucher or explicit closure. |
| 5.6 Why Vouchers Are Better Than Encumbered Cash | Use vouchers rather than dirty cash extensions. |
| 6. Right: Authority Without Value | Keep right leaf zero-value and authority-only. |
| 6.1 What Right Means | Validate scope, holder/control, action membership, and zero value. |
| 6.2 Stateless And Stateful Rights | Support stateless proof and stateful quota/nonce/expiry planning. |
| 6.3 Rights And Delegation | Test delegation, scope narrowing, revocation, expiry, and replay failure. |
| 6.4 Why Right Does Not Duplicate Voucher | Prevent right from becoming budget, claim, or cash surrogate. |
| 7. Policy, ActionPool, And Condition Model | Define policy descriptor and action selection boundary. |
| 7.1 Fixed CashPolicy For Native Asset | Keep asset cash policy fixed and special. |
| 7.2 VoucherPolicy And ActionPool | Add voucher policy/action semantics without affecting native cash. |
| 7.3 Core-Safe Condition Classes | MVP accepts deterministic and verifier-safe attested conditions only. |
| 7.4 Validator And Wallet Responsibilities | Split wallet UX/package duties from validator proof duties. |
| 7.5 Minimum Policy Contract Surface | Plan exact descriptor fields, hashes, action ids, and condition commitments. |
| 7.6 Minimum Action Semantics | Define per-action inputs, outputs, rights, signatures, attestations, and state transitions. |
| 7.7 Package And Witness Boundary | Plan package schema and validator binding checks. |
| 7.8 Separate Fee-Support Boundary | Keep FeeEnvelope separate and test boundary violations. |
| 8. Payment, Acceptance, And Receiver Safety | Preserve one-sided cash while making voucher acceptance explicit. |
| 8.1 Clean Payment Versus Voucher Transfer | Separate asset transfer UX from voucher offer/accept UX. |
| 8.2 One-Sided Cash Stays | Do not require receiver acceptance for normal cash. |
| 8.3 Refund Is Not Arbitrary Clawback | Refund only through bounded voucher lifecycle rules. |
| 8.4 Unknown Policy And Wallet Quarantine | Quarantine unknown-policy objects and exclude from spendable balance. |
| 9. Storage And Settlement Architecture | Extend one settlement architecture, not a parallel tree. |
| 9.1 One Settlement-Root Contract And Semantic Object View | Preserve one `SettlementStateRoot` and one `SettlementPath` family. |
| 9.2 Live HJMT Leaves And The Voucher Target | Add voucher leaf family after auditing current terminal/right proof assumptions. |
| 9.3 What Belongs In Canonical State | Keep commitments and verifier data on-chain/state; secrets in wallet; descriptors/witness in package/DA. |
| 9.3.1 Per-Object Storage Split | Define committed/payload/witness split per Asset, Voucher, Right. |
| 9.4 Why Policies And ActionPool Live Mostly Outside The Committed State | Store policy commitments/hashes, not arbitrary hidden code. |
| 9.5 Conservation And Supply | Enforce Assets + Vouchers value conservation and exclude Rights. |
| 9.6 Why Not Nested Rights Or Nested Vouchers | Keep sibling object leaves under one root. |
| 9.7 Where Objects Live And Who Uses Them | Map wallet, aggregator, validator, watcher, registry/issuer, root contract responsibilities. |
| 9.8 End-To-End Role Path | Simulate user-to-wallet-to-aggregator-to-validator-to-storage-to-watcher path. |
| 9.9 Admission, Verdict, And Alert Surfaces | Add validator verdicts and watcher alerts for invalid object actions. |
| 10. Security Boundary And Non-Goals | Encode refusal cases and non-goals as tests. |
| 10.1 What Validators Must Verify | Build validator checklist into tests and simulator failure cases. |
| 10.2 What Core Z00Z Should Refuse | Reject arbitrary cash actions, wallet-only policy logic, value rights, unbacked vouchers, hidden logic. |
| 10.3 Residual Risks | Track issuer solvency, policy descriptor availability, UX confusion, and oracle/attestation risk. |
| 10.4 Non-Goals | Defer universal VM, subjective/oracle-heavy conditions, and fully generalized app logic. |
| 11. MVP Recommendation | Plan MVP object set first, with future hooks documented. |
| 11.1 MVP Object Set | Implement clean Asset, fully backed Voucher, generic Right. |
| 11.2 MVP Use-Case Priority | Prioritize budget voucher, contractor/employee spend voucher, grant/allowance, delayed settlement claim. |
| 11.3 Future Expansion | Mark registry/oracle/subjective conditions and richer policy systems as later phases unless needed for MVP tests. |
| 11.4 From Whitepaper To Full Spec | Produce schema, package, verifier, publication, watcher, wallet quarantine, and test artifacts. |
| 12. Conclusion | Keep triad and storage architecture coherent through implementation. |
| Appendix A. Core Claims And Non-Claims | Convert claims/non-claims into acceptance and rejection tests. |
| Appendix B. Reading Map | Use listed docs as required reading during source audit and plan writing. |

## TODO Bullet And Table Coverage Ledger

The micro coverage map above is the section index. This ledger is the stricter
bullet/table-row contract for `059-TODO.md`. The source currently contains 190
Markdown list bullets and 112 Markdown table rows. Execution and final evidence
must treat each bullet, table row, figure obligation, and paragraph-level
recommendation as covered only when it maps to a D-ID, plan action,
test/simulator artifact, corpus constraint, explicit deferral, or non-goal.

| TODO item group | Required closure |
|---|---|
| Key-term bullets for `Asset`, `Voucher`, `Right`, policies, `ActionPool`, `FeeEnvelope`, checkpoint, settlement evidence, HJMT roots/paths/leaves, and typed object view. | D-05 through D-20, D-44 through D-48, D-49 through D-53; plans `059-01`, `059-02`, `059-04`, `059-05`, `059-06`, `059-10`. |
| Section 1 thesis bullets: cash finality, programmable conditional value, delegated authority, refund/partial redeem, supply accounting, bounded auditable settlement surface. | D-01 through D-09, D-16 through D-25, D-30 through D-37; plans `059-01` through `059-10`. |
| Corpus-role table rows and maturity bullets for main, smart-cash, use-case, uniqueness, and HJMT design documents. | Referenced corpus coverage map, D-38 through D-41; plans `059-01` and `059-10`. |
| Minimal-triad table rows, split-minimality bullets, voucher/right non-redundancy bullets, and cross-object binding table rows. | D-05 through D-07, D-19, D-44 through D-48, D-52, D-56 through D-58; plans `059-02`, `059-05`, `059-08`, `059-09`, `059-10`. |
| Asset operations, cash-grade invariant bullets, and "what breaks" table for programmable native cash. | D-05, D-08, D-21, D-24, D-50, D-64 through D-68; plans `059-02`, `059-05`, `059-07`, `059-08`, `059-10`. |
| Voucher economic meaning bullets, fully-backed model, non-final-cash table, lifecycle figure, partial-redeem bullets, and encumbered-cash comparison. | D-06, D-14, D-23, D-44 through D-46, D-57, D-67, D-70 through D-72; plans `059-02`, `059-03`, `059-04`, `059-05`, `059-08`, `059-09`, `059-10`. |
| Right action bullets, stateless/stateful right bullets, delegation weakening bullets, and voucher/right distinction bullets. | D-07, D-13, D-40, D-47, D-58, D-67, D-70 through D-72; plans `059-02`, `059-03`, `059-05`, `059-06`, `059-08`, `059-09`, `059-10`. |
| Policy/action/condition tables and bullets: fixed cash policy, voucher action pool, condition trust tiers, validator/wallet responsibilities, minimum policy contract, and minimum action semantics. | D-08, D-15, D-25, D-30, D-49 through D-53, D-56 through D-58; plans `059-02`, `059-03`, `059-05`, `059-06`, `059-08`, `059-10`. |
| Package/witness bullets and fee-support bullets. | D-18, D-20, D-25, D-30, D-52, D-56, D-63; plans `059-02`, `059-05`, `059-06`, `059-08`, `059-10`. |
| Payment, receiver-safety, refund, and unknown-policy quarantine bullets. | D-21 through D-25, D-46, D-53, D-64 through D-68, D-70 through D-72; plans `059-07`, `059-08`, `059-09`, `059-10`. |
| Storage architecture bullets/tables: one root/path/leaf family, live HJMT leaves, canonical-state split, policy commitments, conservation, no nested vouchers/rights, role map, end-to-end path, admission/verdict/alerts. | D-16 through D-20, D-30 through D-32, D-44 through D-48, D-59 through D-63; plans `059-04`, `059-05`, `059-06`, `059-09`, `059-10`. |
| Security checklist bullets: validators must verify, core must refuse, residual risks, and non-goals. | D-30 through D-37, D-51 through D-53, D-70 through D-72; `059-TEST-SPEC.md`, `059-TESTS-TASKS.md`, plans `059-06`, `059-09`, `059-10`. |
| MVP, use-case, future-expansion, and full-spec bullets. | D-10 through D-15, D-33 through D-37, D-49 through D-58, D-69 through D-72; plans `059-03`, `059-08`, `059-09`, `059-10`. |
| Appendix A claim/non-claim rows and Appendix B reading-map rows. | `059-SOURCE-AUDIT.md`, referenced corpus coverage map, final `059-EVIDENCE-LEDGER.md`; plans `059-01` and `059-10`. |

No TODO list bullet or table row may be considered closed only because a broad
section is named. The implementation summary and final evidence ledger must
name the closest D-ID, plan, test/simulator artifact, corpus constraint,
explicit deferral, or non-goal for each group above.

## Referenced Corpus Coverage Map

| Corpus source | Phase 059 planning constraint | Required plan anchor |
|---|---|---|
| `docs/Z00Z-Main-Whitepaper.md` | Preserve wallet-local possession, receiver-native package construction, checkpoint settlement evidence, and the rule that soft confirmation is not final settlement. | `059-06`, `059-08`, `059-09`, `059-10` |
| `docs/Z00Z-Smart-Cash-Whitepaper.md` | Keep the implementation inside smart-cash and bounded-rights semantics; do not turn vouchers, rights, or policies into a universal private VM or hidden arbitrary execution layer. | `059-02`, `059-05`, `059-06`, `059-10` |
| `docs/Z00Z-UseCases-Whitepaper.md` | Cover budget, allowance, grant, contractor/employee spend, aid/community voucher, and service/agent right scenarios as concrete simulator/UAT fuel without claiming every surrounding provider or issuer truth is core protocol truth. | `059-03`, `059-08`, `059-09`, `059-10` |
| `docs/Z00Z-Uniqueness-Whitepaper.md` | Preserve the rights-first model: private wallet-local objects and bounded rights settle through minimal checkpoint evidence, not public-account state, full-wallet authority, or visible contract permissions. | `059-02`, `059-06`, `059-07`, `059-08`, `059-09`, `059-10` |
| `docs/tech-papers/done/Z00Z-HJMT-Design.md` | Extend the existing HJMT settlement contract in place: one `SettlementStateRoot`, one `SettlementPath`, `SettlementLeaf` families, no `AssetStateRoot`/`AssetPath` revival as live authority, no global root registry, and `RightLeaf`/`FeeEnvelope` separation. | `059-01`, `059-04`, `059-05`, `059-06`, `059-10` |

Every implementation summary and the final evidence ledger must map these
corpus constraints to code, tests, simulator artifacts, docs, explicit
deferrals, or non-goals. A plan is incomplete if it implements the local TODO
wording but violates one of these corpus constraints.

## Crate Impact Matrix

| Crate/module | Required impact |
|---|---|
| `z00z_core::assets` | Preserve asset semantics and compatibility exports without becoming the only home for policy/action/right/voucher logic. |
| `z00z_core::actions` | Populate the module root with canonical action ids, action descriptors, and action-pool types, or thinly re-export a single canonical implementation. |
| `z00z_core::policies` | Populate the module root with canonical policy ids, policy descriptors, condition descriptors, and canonical hash rules. |
| `z00z_core::rights` | Populate the module root with right-specific authority policy/action types while preserving zero-value semantics. |
| `z00z_core::vauchers` | Populate the existing module root with voucher config, policy, and lifecycle types; keep the current path spelling unless the implementation performs an audited rename. |
| `z00z_core::genesis` | Add voucher and policy generation under shared orchestration; extend validation and exports. |
| `z00z_core` tests | Add deterministic genesis, policy hash, schema rejection, and object invariant tests. |
| `z00z_storage::settlement` | Add voucher leaf family and extend all proof, model, store, batch, recovery, and serialization paths. |
| `z00z_storage` tests | Add all-family proof and delta tests plus negative conservation/lifecycle cases. |
| `z00z_wallets::db` | Generalize owned object persistence or add typed stores behind a unified facade. |
| `z00z_wallets::services` | Add voucher/right receive, quarantine, package building, state transitions, and cash projection separation. |
| `z00z_wallets::rpc` | Add typed object APIs while preserving existing asset transfer semantics. |
| `z00z_wallets` tests | Add persistence, scan, migration, quarantine, reservation, voucher lifecycle, right lifecycle, RPC tests. |
| `z00z_simulator::scenario_1` | Extend existing staged Alice/Bob/Charlie flows to every object class and combined interaction. |
| `z00z_simulator` tests | Add executable happy and failure paths with release-mode evidence. |
| `z00z_runtime` | Audit admission, planner, validator, watcher assumptions about leaf families and value conservation. |
| `z00z_rollup_node` | Audit checkpoint/publication surfaces for object family and verdict reporting assumptions. |
| `z00z_networks` | Audit serialized package/RPC transport assumptions if new envelope fields cross process boundaries. |
| `z00z_crypto` | Use existing cryptographic primitives; do not edit vendored `tari` code. |
| `z00z_utils` | Reuse codec/config/io helpers; avoid phase-specific utility duplication. |

## Acceptance Criteria For Planning

- Every section in the micro coverage map and every item group in the TODO
  bullet/table coverage ledger is represented in one or more plans, tests,
  explicit deferrals, or non-goals.
- Source audit names all live code paths that assume only Asset/Terminal or
  Right leaves.
- Source audit has a live/target/migration table and explicitly marks every
  target-only type, including Voucher, VoucherPolicy, ActionPool, and any new
  object package envelope.
- Plans are ordered so object schemas and storage family tags land before
  wallet/simulator flows that depend on them.
- Core plans populate or intentionally re-export `assets`, `actions`,
  `policies`, `rights`, `vauchers`, and `genesis` through one canonical
  implementation per concept; empty module roots or duplicate descriptor logic
  cannot close Phase 059.
- Wallet plans preserve spendable cash semantics and quarantine unknown policy
  objects.
- Wallet plans define durable object-family quarantine and typed inventory
  states before adding voucher/right RPC methods.
- Simulator plans cover all object classes and cross-object flows among Alice,
  Bob, and Charlie.
- Simulator plans include negative artifacts for every validator/watcher
  rejection class, not only happy-path transfer artifacts.
- Validator/watcher plans include failure surfaces, not just happy paths.
- Conservation tests explicitly include Assets and Vouchers and explicitly
  exclude Rights.
- Storage tests prove family-specific inclusion, deletion, and nonexistence
  proofs for Terminal, Right, and Voucher leaves.
- Verification plan includes targeted crate tests, property tests where
  applicable, simulator release lane, and the repository full verification
  gate when implementation is complete.

<deferred>
## Deferred Ideas

- Universal VM-like policy execution is not an MVP requirement.
- Subjective/oracle-heavy policy conditions are outside the core-safe MVP unless
  represented only as verifier-safe attestations.
- Cross-chain bridge semantics, external issuer solvency systems, and market
  pricing of vouchers are outside Phase 059 unless needed to reject unsafe
  assumptions.
- UI polish is out of scope except where wallet/RPC semantics require distinct
  Asset/Voucher/Right projections.

</deferred>

---

*Phase: 059-Core-Upgrade*
*Context gathered: 2026-06-16*
