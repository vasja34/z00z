# Core Examples

Version: 2026-06-27
Status: planning inventory
Scope: reusable YAML examples for assets, rights, policies, vouchers, wallet
profiles, and policy profiles.

## 🎯 Decision

Use one small catalog envelope for every example: `id`, `family`,
`materialize_as`, `use_case`, and `yaml`.

Do not force common payload fields across different object families. The live
validators do not share one schema: assets carry monetary policy, rights carry
authority and policy ids, policies carry action pools, and vouchers carry
backing plus lifecycle state. Over-unifying those inner fields would create
false, unclear fields.

## ⚖️ Pros And Cons

| Option | Pros | Cons | Decision |
| --- | --- | --- | --- |
| Unified catalog envelope only | Every named example has the same searchable wrapper; every example points to one concrete YAML payload; no field is invented for unrelated families. | The `yaml` payload still differs by family. | Use it. |
| Fully unified object payload | Looks tidy and could make one generic UI easier. | Creates vague fields like `settings`, `policy`, or `scope` that mean different things for assets, rights, vouchers, and profiles. | Do not use it. |
| Separate per-family sections only | Matches live genesis sections directly. | Profile examples become detached from genesis examples and are easier to leave dead. | Use only inside each `yaml` payload. |

## 🧭 Source Anchors

- Live genesis fixture:
  [devnet_genesis_config.yaml](../../../crates/z00z_core/z00z_config/devnet_genesis_config.yaml).
- Split genesis fixture:
  [devnet_genesis_config.yaml](../../../crates/z00z_core/z00z_config/devnet_genesis_config.yaml).
- Wallet catalog:
  [WALLET-GUIDE.md](../../../crates/z00z_wallets/docs/WALLET-GUIDE.md).
- Profile ideas:
  [TODO-Wallet-idea.md](../../../docs/tech-papers/TODO-Wallet-idea.md) and
  [articles-review-ideas.md](../../../docs/tech-papers/articles-review-ideas.md).

## 🪙 Asset Examples

```yaml
core_examples:
  - id: z00z
    family: asset
    materialize_as: assets[]
    use_case: Native settlement cash and fee lane.
    yaml:
      id: z00z
      symbol: Z00Z
      class: Coin
      name: Z00Z Native Coin
      description: Native confidential coin
      policy:
        allow_burn: false
        decimals: 8
        serials: 20
        nominal: 10000
        is_gas: true
        domain_name: z00z.core.assets.coin.devnet.v1
      metadata: {}

  - id: zUSD
    family: asset
    materialize_as: assets[]
    use_case: Confidential stable-token style asset for non-native value tests.
    yaml:
      id: zUSD
      symbol: zUSD
      class: Token
      name: Z00Z USD Stablecoin
      description: Confidential USD-pegged stablecoin
      policy:
        allow_burn: true
        decimals: 6
        serials: 20
        nominal: 10000
        domain_name: z00z.core.assets.token.devnet.v1
      metadata: {}

  - id: zNFT
    family: asset
    materialize_as: assets[]
    use_case: Event-ticket or unique-right anchor with one nominal unit.
    yaml:
      id: zNFT
      symbol: zNFT
      class: Nft
      name: Z00Z Event Ticket
      description: NFT event ticket
      policy:
        allow_burn: true
        decimals: 0
        serials: 20
        nominal: 1
        domain_name: z00z.core.assets.nft.devnet.v1
      metadata: {}

  - id: zBurnSink
    family: asset
    materialize_as: assets[]
    use_case: Explicit void asset for burn-sink and zero-value checks.
    yaml:
      id: zBurnSink
      symbol: zBurnSink
      class: Void
      name: Burn Sink
      description: Special void asset for burning coins
      policy:
        allow_burn: true
        decimals: 0
        serials: 20
        nominal: 0
        domain_name: z00z.core.assets.void.devnet.v1
      metadata: {}
```

## 🔑 Right Examples

```yaml
core_examples:
  - id: machine_compute_capability
    family: right
    materialize_as: rights[]
    use_case: Compute scheduler capability that can be transferred and consumed.
    yaml:
      id: machine_compute_capability
      right_class: machine_capability
      issuer_scope: issuer.devnet.compute
      provider_scope: provider.devnet.compute
      holder_fixture: wallet.alice
      control_fixture: service.compute.scheduler
      beneficiary_fixture: wallet.alice
      count: 3
      domain_name: z00z.core.rights.machine_capability.devnet.v1
      valid_from: 0
      valid_until: 4102444800
      challenge_from: 0
      challenge_until: 0
      revocation_policy_id: policy.revoke.devnet.compute
      transition_policy_id: policy.transition.devnet.compute
      challenge_policy_id: policy.challenge.devnet.compute
      disclosure_policy_id: policy.disclosure.devnet.compute
      retention_policy_id: policy.retention.devnet.compute
      payload_commitment_seed: payload.devnet.machine_compute_capability
      metadata:
        purpose: create, transfer, consume, replay reject

  - id: confidential_data_access
    family: right
    materialize_as: rights[]
    use_case: Time-bounded access to protected data.
    yaml:
      id: confidential_data_access
      right_class: data_access
      issuer_scope: issuer.devnet.data
      provider_scope: provider.devnet.data
      holder_fixture: wallet.bob
      control_fixture: service.data.guard
      beneficiary_fixture: wallet.bob
      count: 3
      domain_name: z00z.core.rights.data_access.devnet.v1
      valid_from: 0
      valid_until: 4102444800
      challenge_from: 0
      challenge_until: 0
      revocation_policy_id: policy.revoke.devnet.data
      transition_policy_id: policy.transition.devnet.data
      challenge_policy_id: policy.challenge.devnet.data
      disclosure_policy_id: policy.disclosure.devnet.data
      retention_policy_id: policy.retention.devnet.data
      payload_commitment_seed: payload.devnet.confidential_data_access
      metadata:
        purpose: create, expire, absence proof after expiry cleanup

  - id: service_entitlement
    family: right
    materialize_as: rights[]
    use_case: Subscription or service-access entitlement.
    yaml:
      id: service_entitlement
      right_class: service_entitlement
      issuer_scope: issuer.devnet.service
      provider_scope: provider.devnet.service
      holder_fixture: wallet.charlie
      control_fixture: service.billing.router
      beneficiary_fixture: wallet.charlie
      count: 3
      domain_name: z00z.core.rights.service_entitlement.devnet.v1
      valid_from: 0
      valid_until: 4102444800
      challenge_from: 0
      challenge_until: 0
      revocation_policy_id: policy.revoke.devnet.service
      transition_policy_id: policy.transition.devnet.service
      challenge_policy_id: policy.challenge.devnet.service
      disclosure_policy_id: policy.disclosure.devnet.service
      retention_policy_id: policy.retention.devnet.service
      payload_commitment_seed: payload.devnet.service_entitlement
      metadata:
        purpose: create, transfer, revoke

  - id: validator_mandate
    family: right
    materialize_as: rights[]
    use_case: Validator or operator mandate with an active challenge window.
    yaml:
      id: validator_mandate
      right_class: validator_mandate
      issuer_scope: issuer.devnet.validator
      provider_scope: provider.devnet.validator
      holder_fixture: service.validator.primary
      control_fixture: service.validator.orchestrator
      beneficiary_fixture: service.validator.primary
      count: 2
      domain_name: z00z.core.rights.validator_mandate.devnet.v1
      valid_from: 0
      valid_until: 4102444800
      challenge_from: 1
      challenge_until: 4102444800
      revocation_policy_id: policy.revoke.devnet.validator
      transition_policy_id: policy.transition.devnet.validator
      challenge_policy_id: policy.challenge.devnet.validator
      disclosure_policy_id: policy.disclosure.devnet.validator
      retention_policy_id: policy.retention.devnet.validator
      payload_commitment_seed: payload.devnet.validator_mandate
      metadata:
        purpose: create, challenge, revoke

  - id: one_time_agent_action
    family: right
    materialize_as: rights[]
    use_case: One-shot agent action that must reject replayed consumption.
    yaml:
      id: one_time_agent_action
      right_class: one_time_use
      issuer_scope: issuer.devnet.agent
      provider_scope: provider.devnet.agent
      holder_fixture: wallet.charlie
      control_fixture: service.agent.scheduler
      beneficiary_fixture: wallet.charlie
      count: 3
      domain_name: z00z.core.rights.one_time_use.devnet.v1
      valid_from: 0
      valid_until: 4102444800
      challenge_from: 0
      challenge_until: 0
      revocation_policy_id: policy.revoke.devnet.agent
      transition_policy_id: policy.transition.devnet.agent
      challenge_policy_id: policy.challenge.devnet.agent
      disclosure_policy_id: policy.disclosure.devnet.agent
      retention_policy_id: policy.retention.devnet.agent
      payload_commitment_seed: payload.devnet.one_time_agent_action
      metadata:
        purpose: create, consume, second consume reject
```

## 🧾 Policy Examples

```yaml
core_examples:
  - id: cash_policy_v1
    family: policy
    materialize_as: generated_from_assets
    use_case: Native cash policy generated when a gas asset exists.
    yaml:
      required_asset:
        id: z00z
        policy:
          is_gas: true
      generated_policy_id: cash_policy_v1

  - id: voucher_transferable_policy_v1
    family: policy
    materialize_as: policies[]
    use_case: Transferable voucher with accept, transfer, full redeem, partial redeem, and refund.
    yaml:
      action_pool:
        label: voucher_transferable_policy_v1
        actions:
          - label: voucher_accept
            allowed_input_families: [voucher]
            allowed_output_families: [voucher]
            lifecycle_effect: accept
            witness_requirements: [acceptance_proof, replay_nonce]
            receiver_must_accept: true
            preserves_beneficiary: true
            preserves_refund_authority: true
          - label: voucher_transfer
            allowed_input_families: [voucher]
            allowed_output_families: [voucher]
            lifecycle_effect: transfer
            witness_requirements: [acceptance_proof, replay_nonce]
            receiver_must_accept: true
            preserves_beneficiary: true
            preserves_refund_authority: true
          - label: voucher_redeem_full
            allowed_input_families: [voucher]
            allowed_output_families: [asset]
            lifecycle_effect: redeem
            witness_requirements: [replay_nonce]
            receiver_must_accept: false
            preserves_beneficiary: true
            preserves_refund_authority: true
          - label: voucher_partial_redeem
            allowed_input_families: [voucher]
            allowed_output_families: [asset, voucher]
            lifecycle_effect: partial_redeem
            witness_requirements: [replay_nonce]
            receiver_must_accept: false
            preserves_beneficiary: true
            preserves_refund_authority: true
          - label: voucher_refund_after_expiry
            allowed_input_families: [voucher]
            allowed_output_families: [asset]
            lifecycle_effect: refund
            witness_requirements: [prior_state_root, replay_nonce]
            receiver_must_accept: false
            preserves_beneficiary: true
            preserves_refund_authority: true
      template:
        label: voucher_transferable_policy_v1
        primary_family: voucher
        allowed_input_families: [voucher]
        allowed_output_families: [asset, voucher]
        required_signatures: [holder]
        expiry_rule: valid_until
        replay_protection: nonce_and_root
        conservation: conditional_value

  - id: voucher_non_transferable_policy_v1
    family: policy
    materialize_as: policies[]
    use_case: Static voucher that can be accepted, redeemed, or expired, but not transferred.
    yaml:
      action_pool:
        label: voucher_non_transferable_policy_v1
        actions:
          - label: voucher_accept_static
            allowed_input_families: [voucher]
            allowed_output_families: [voucher]
            lifecycle_effect: accept
            witness_requirements: [acceptance_proof, replay_nonce]
            receiver_must_accept: true
            preserves_beneficiary: true
            preserves_refund_authority: true
          - label: voucher_redeem_static
            allowed_input_families: [voucher]
            allowed_output_families: [asset]
            lifecycle_effect: redeem
            witness_requirements: [replay_nonce]
            receiver_must_accept: false
            preserves_beneficiary: true
            preserves_refund_authority: true
          - label: voucher_expire_static
            allowed_input_families: [voucher]
            allowed_output_families: [asset]
            lifecycle_effect: expire
            witness_requirements: [prior_state_root, replay_nonce]
            receiver_must_accept: false
            preserves_beneficiary: true
            preserves_refund_authority: true
      template:
        label: voucher_non_transferable_policy_v1
        primary_family: voucher
        allowed_input_families: [voucher]
        allowed_output_families: [asset, voucher]
        required_signatures: [holder]
        expiry_rule: valid_until
        replay_protection: nonce_and_root
        conservation: conditional_value

  - id: right_delegate_policy_v1
    family: policy
    materialize_as: policies[]
    use_case: Controller-approved delegation, use, and revoke flow for rights.
    yaml:
      action_pool:
        label: right_delegate_policy_v1
        actions:
          - label: right_delegate
            allowed_input_families: [right]
            allowed_output_families: [right]
            lifecycle_effect: delegate
            witness_requirements: [replay_nonce]
            receiver_must_accept: false
            preserves_beneficiary: true
            preserves_refund_authority: true
          - label: right_use
            allowed_input_families: [right]
            allowed_output_families: [right]
            lifecycle_effect: use
            witness_requirements: [replay_nonce]
            receiver_must_accept: false
            preserves_beneficiary: true
            preserves_refund_authority: true
          - label: right_revoke
            allowed_input_families: [right]
            allowed_output_families: [right]
            lifecycle_effect: revoke
            witness_requirements: [replay_nonce]
            receiver_must_accept: false
            preserves_beneficiary: true
            preserves_refund_authority: true
      template:
        label: right_delegate_policy_v1
        primary_family: right
        allowed_input_families: [right]
        allowed_output_families: [right]
        required_signatures: [controller]
        expiry_rule: none
        replay_protection: nonce
        conservation: zero_value_authority

  - id: right_one_time_policy_v1
    family: policy
    materialize_as: policies[]
    use_case: Holder-approved one-time right consumption.
    yaml:
      action_pool:
        label: right_one_time_policy_v1
        actions:
          - label: right_one_time_use
            allowed_input_families: [right]
            allowed_output_families: [right]
            lifecycle_effect: use
            witness_requirements: [replay_nonce]
            receiver_must_accept: false
            preserves_beneficiary: true
            preserves_refund_authority: true
      template:
        label: right_one_time_policy_v1
        primary_family: right
        allowed_input_families: [right]
        allowed_output_families: [right]
        required_signatures: [holder]
        expiry_rule: none
        replay_protection: nonce
        conservation: zero_value_authority
```

## 🎟️ Voucher Examples

```yaml
core_examples:
  - id: voucher.transferable.devnet
    family: voucher
    materialize_as: vouchers[]
    use_case: Active transferable claim backed by a genesis reserve.
    yaml:
      id: voucher.transferable.devnet
      issuer_fixture: wallet.alice
      holder_fixture: wallet.bob
      beneficiary_fixture: wallet.charlie
      backing:
        genesis_reserve:
          reserve_id: reserve.devnet.transferable
      face_value: 100
      remaining_value: 100
      policy_label: voucher_transferable_policy_v1
      lifecycle: active
      validity:
        valid_from: 0
        valid_until: 1000
      acceptance:
        receiver_must_accept: true
        allow_reject: true
        refund_target_fixture: wallet.alice
      replay_nonce: [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]
      disclosure_commitment: [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2]
      audit_commitment: [3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3]

  - id: voucher.non_transferable.devnet
    family: voucher
    materialize_as: vouchers[]
    use_case: Pending receiver-acceptance voucher that cannot transfer.
    yaml:
      id: voucher.non_transferable.devnet
      issuer_fixture: wallet.alice
      holder_fixture: wallet.charlie
      beneficiary_fixture: wallet.charlie
      backing:
        genesis_reserve:
          reserve_id: reserve.devnet.non_transferable
      face_value: 75
      remaining_value: 75
      policy_label: voucher_non_transferable_policy_v1
      lifecycle: pending_acceptance
      validity:
        valid_from: 0
        valid_until: 500
      acceptance:
        receiver_must_accept: true
        allow_reject: true
        refund_target_fixture: wallet.alice
      replay_nonce: [4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4]
      disclosure_commitment: [5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5]
      audit_commitment: [6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 6]

  - id: voucher.expired.devnet
    family: voucher
    materialize_as: vouchers[]
    use_case: Negative-path expired voucher with no remaining value.
    yaml:
      id: voucher.expired.devnet
      issuer_fixture: wallet.alice
      holder_fixture: wallet.bob
      beneficiary_fixture: wallet.bob
      backing:
        genesis_reserve:
          reserve_id: reserve.devnet.expired
      face_value: 25
      remaining_value: 0
      policy_label: voucher_non_transferable_policy_v1
      lifecycle: expired
      validity:
        valid_from: 0
        valid_until: 1
      acceptance:
        receiver_must_accept: true
        allow_reject: true
        refund_target_fixture: wallet.alice
      replay_nonce: [7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7]
      disclosure_commitment: [8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8]
      audit_commitment: [9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9]
```

## 🗂️ Wallet Profile Examples

These are live parser-owned `GenesisConfig` sections. They map to live asset,
right, policy, or voucher anchors, and the loader must parse and validate
every field even before runtime materialization consumes every profile field.
`live_anchor` should be read as either one concrete live id/right class or one
documented live object model when the wallet surface does not expose a single
canonical object id.

```yaml
core_examples:
  - id: fee_credit_v1
    family: wallet_profile
    materialize_as: wallet_profiles[]
    use_case: Non-cash processing capacity backed by a voucher path.
    yaml:
      id: fee_credit_v1
      object_family: voucher
      live_anchor: "existing voucher object model"
      product_anchor: FeeCredit
      transitions: [issue, accept, redeem, expire, refund]
      backing_kind: genesis_reserve
      transferability: non_transferable
      redeem_target: fee_lane
      expiry_rule: valid_until
      fail_closed: [voucher_as_cash, missing_backing, replay, expired_use, unknown_policy]

  - id: service_entitlement_v1
    family: wallet_profile
    materialize_as: wallet_profiles[]
    use_case: Private service subscription or access entitlement.
    yaml:
      id: service_entitlement_v1
      object_family: right
      live_anchor: service_entitlement
      transitions: [grant, delegate, consume, revoke, expire]
      disclosure_policy: policy.disclosure.devnet.service
      retention_policy: policy.retention.devnet.service
      provider_scope: scoped
      beneficiary_scope: scoped
      fail_closed: [right_as_value, out_of_scope_use, revoked_or_expired, unknown_policy]

  - id: data_access_v1
    family: wallet_profile
    materialize_as: wallet_profiles[]
    use_case: Private data-room or API access with audit and retention hooks.
    yaml:
      id: data_access_v1
      object_family: right
      live_anchor: data_access
      transitions: [grant, consume, challenge, revoke, expire]
      disclosure_policy: policy.disclosure.devnet.data
      retention_policy: policy.retention.devnet.data
      audit_trail: required
      challenge_window: bounded
      fail_closed: [expired_access, wrong_beneficiary, challenge_window_misuse, unknown_policy]

  - id: agent_budget_v1
    family: wallet_profile
    materialize_as: wallet_profiles[]
    use_case: Agent receives bounded authority without free wallet balance.
    yaml:
      id: agent_budget_v1
      object_family: right
      live_anchor: [machine_capability, one_time_use]
      transitions: [delegate, consume_quota, revoke, expire]
      action_whitelist: [compute, submit, reconcile]
      quota: bounded
      service_scope: bounded
      expiry_rule: valid_until
      replay_mode: nonce
      fail_closed: [over_budget_action, unauthorized_action_family, consumed_right_reuse, unknown_policy]

  - id: validator_mandate_lock_v1
    family: wallet_profile
    materialize_as: wallet_profiles[]
    use_case: Private lock or validator mandate that blocks ordinary spend selection.
    yaml:
      id: validator_mandate_lock_v1
      object_family: right
      live_anchor: validator_mandate
      locked_asset_id: z00z
      locked_amount: committed
      binding_surfaces: [holder, control, beneficiary, payload]
      payload_binding: payload_commitment
      lock_window:
        valid_from: 0
        valid_until: 4102444800
      transitions: [lock, unlock, redelegate, reward_claim, revoke, expire]
      revoke_mode: challenge_bounded
      disclosure_policy: policy.disclosure.devnet.validator
      retention_policy: policy.retention.devnet.validator
      ordinary_spend: forbidden
      fail_closed: [soft_lock_only, ordinary_spend_of_locked_asset, replay, wrong_family_proof]

  - id: transferable_claim_v1
    family: wallet_profile
    materialize_as: wallet_profiles[]
    use_case: Claim that can circulate privately before final redemption.
    yaml:
      id: transferable_claim_v1
      object_family: voucher
      live_anchor: "existing Phase 059 voucher object model"
      transitions: [offer, accept, transfer, partial_redeem, redeem, reject, expire]
      backing_kind: genesis_reserve
      accept_policy: receiver_must_accept
      transferability: transferable
      partial_redeem: allowed
      residual_policy: return_residual_voucher
      expiry_rule: valid_until
      fail_closed: [wrong_family_proof, double_redeem, expired_use, residual_mismatch, unknown_policy]
```

## 🏛️ Policy Profile Examples

These are live parser-owned wallet or enterprise overlays. They belong in live
genesis YAML as typed, validated sections even before every field has a direct
runtime consumer.

```yaml
core_examples:
  - id: corporate_eu_transfer_v1
    family: policy_profile
    materialize_as: policy_profiles[]
    use_case: Corporate wallet proves one transfer followed a declared rule without exposing unrelated inventory.
    yaml:
      id: corporate_eu_transfer_v1
      enforced_actions: [create, transfer, consume, revoke, challenge, disclose]
      selected_fields: [invoice_reference, counterparty_reference]
      require_purpose: true
      require_expiry: true
      bind_policy_id: true
      bind_checkpoint_anchor: true
      bind_retained_document_hash: true
      disclosure_receipt_required: true
      retention_profile: enterprise_retention_7y_v1

  - id: enterprise_retention_7y_v1
    family: policy_profile
    materialize_as: policy_profiles[]
    use_case: Enterprise archive keeps a bounded evidence package for seven years.
    yaml:
      id: enterprise_retention_7y_v1
      retained_object_type: EvidencePackage
      required_bindings: [policy_id, action, field_set, purpose, expiry, document_hash, checkpoint_anchor]
      retention_years: 7
      disclosure_receipt_type: DisclosureReceipt

  - id: sanctions_screened_counterparty_v1
    family: policy_profile
    materialize_as: policy_profiles[]
    use_case: Wallet records that one counterparty reference was screened for one scoped action.
    yaml:
      id: sanctions_screened_counterparty_v1
      applies_to_profiles: [corporate_eu_transfer_v1]
      selected_fields: [counterparty_reference, sanctions_screening_metadata]
      require_expiry: true
      bind_policy_id: true
      bind_retained_document_hash: true
```

## ✅ Materialization Rules

- Put `assets[]`, `rights[]`, `policies[]`, `vouchers[]`, `wallet_profiles[]`,
  and `policy_profiles[]` into live genesis YAML when those sections are
  present in the canonical config.
- Keep `wallet_profiles[]` and `policy_profiles[]` typed and validated through
  the live loader even when some fields are parser-owned only and not yet
  consumed by runtime materialization.
- Treat `cash_policy_v1` as generated behavior from the gas asset. Do not add a
  custom `policies[]` entry for it.
- Do not add a field unless at least one example uses it in YAML and the
  scenario explains why it exists.
