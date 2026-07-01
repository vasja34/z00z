use super::{
    ActionPoolId, BTreeMap, BTreeSet, ChainType, DomainHasher, GenesisError, GenesisPolicyRecord,
    ObjectFamily, PolicyId,
};

use crate::{
    domains::{
        GenesisVoucherDerivationDomainDevnet, GenesisVoucherDerivationDomainMainnet,
        GenesisVoucherDerivationDomainTestnet,
    },
    vouchers::VoucherConfigEntry,
};

pub const GENESIS_VOUCHERS_FILE: &str = "genesis_vouchers.json";

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GenesisVoucherRecord {
    pub voucher_index: u32,
    pub root_generation: u64,
    pub terminal_id: [u8; 32],
    pub issuer_commitment: [u8; 32],
    pub holder_commitment: [u8; 32],
    pub beneficiary_commitment: [u8; 32],
    pub refund_target_commitment: [u8; 32],
    pub config: VoucherConfigEntry,
}

fn derive_voucher_hash(network_type: ChainType, label: &'static str, parts: &[&[u8]]) -> [u8; 32] {
    let digest = match network_type {
        ChainType::Devnet => {
            let mut hasher =
                DomainHasher::<GenesisVoucherDerivationDomainDevnet>::new_with_label(label);
            for part in parts {
                hasher.update(*part);
            }
            hasher.finalize()
        }
        ChainType::Testnet => {
            let mut hasher =
                DomainHasher::<GenesisVoucherDerivationDomainTestnet>::new_with_label(label);
            for part in parts {
                hasher.update(*part);
            }
            hasher.finalize()
        }
        ChainType::Mainnet => {
            let mut hasher =
                DomainHasher::<GenesisVoucherDerivationDomainMainnet>::new_with_label(label);
            for part in parts {
                hasher.update(*part);
            }
            hasher.finalize()
        }
    };

    let mut out = [0u8; 32];
    out.copy_from_slice(&digest.as_ref()[..32]);
    out
}

fn derive_fixture_commitment(
    network_type: ChainType,
    label: &'static str,
    fixture: &str,
    chain_id: u32,
    policy_id: PolicyId,
    action_pool_id: ActionPoolId,
) -> [u8; 32] {
    let chain_id_bytes = chain_id.to_le_bytes();
    derive_voucher_hash(
        network_type,
        label,
        &[
            fixture.as_bytes(),
            &chain_id_bytes,
            &policy_id.bytes(),
            &action_pool_id.bytes(),
        ],
    )
}

pub fn generate_genesis_vouchers(
    vouchers: &[crate::vouchers::VoucherBootstrapEntryV1],
    policy_lookup: &BTreeMap<String, GenesisPolicyRecord>,
    genesis_seed: &[u8; 32],
    chain_id: u32,
    network_type: ChainType,
    root_generation: u64,
) -> Result<Vec<GenesisVoucherRecord>, GenesisError> {
    let mut records = Vec::with_capacity(vouchers.len());
    let mut seen_terminals = BTreeSet::new();

    for (voucher_index, voucher) in vouchers.iter().enumerate() {
        voucher.validate()?;
        let policy = policy_lookup
            .get(voucher.policy_label.as_str())
            .ok_or_else(|| {
                GenesisError::InvalidConfig(format!(
                    "voucher {} references unknown policy {}",
                    voucher.id, voucher.policy_label
                ))
            })?;

        if policy.descriptor.primary_family != ObjectFamily::Voucher {
            return Err(GenesisError::InvalidConfig(format!(
                "voucher {} must resolve to a voucher policy, got {}",
                voucher.id,
                policy.descriptor.primary_family.as_str()
            )));
        }

        let config = voucher.materialize(policy.policy_id, policy.action_pool_id)?;
        if config.audit_commitment.is_none() {
            return Err(GenesisError::InvalidConfig(format!(
                "voucher {} must declare an audit commitment in genesis",
                voucher.id
            )));
        }

        let chain_id_bytes = chain_id.to_le_bytes();
        let voucher_index_bytes = (voucher_index as u32).to_le_bytes();
        let root_generation_bytes = root_generation.to_le_bytes();
        let class_bytes = ObjectFamily::Voucher.as_str().as_bytes();
        let policy_bytes = &policy.policy_id.bytes();
        let pool_bytes = &policy.action_pool_id.bytes();
        let terminal_id = derive_voucher_hash(
            network_type,
            "voucher_terminal",
            &[
                genesis_seed,
                &chain_id_bytes,
                class_bytes,
                config.domain_name.as_bytes(),
                voucher.id.as_bytes(),
                &voucher_index_bytes,
                &root_generation_bytes,
                policy_bytes,
                pool_bytes,
            ],
        );

        if !seen_terminals.insert(terminal_id) {
            return Err(GenesisError::TerminalCollision {
                terminal_id,
                error: format!(
                    "duplicate generated voucher terminal id for {}:{}",
                    voucher.id, voucher_index
                ),
            });
        }

        records.push(GenesisVoucherRecord {
            voucher_index: voucher_index as u32,
            root_generation,
            terminal_id,
            issuer_commitment: derive_fixture_commitment(
                network_type,
                "voucher_issuer_binding",
                config.issuer_fixture.as_str(),
                chain_id,
                config.policy_id,
                config.action_pool_id,
            ),
            holder_commitment: derive_fixture_commitment(
                network_type,
                "voucher_holder_binding",
                config.holder_fixture.as_str(),
                chain_id,
                config.policy_id,
                config.action_pool_id,
            ),
            beneficiary_commitment: derive_fixture_commitment(
                network_type,
                "voucher_beneficiary_binding",
                config.beneficiary_fixture.as_str(),
                chain_id,
                config.policy_id,
                config.action_pool_id,
            ),
            refund_target_commitment: derive_fixture_commitment(
                network_type,
                "voucher_refund_binding",
                config.acceptance.refund_target_fixture.as_str(),
                chain_id,
                config.policy_id,
                config.action_pool_id,
            ),
            config: VoucherConfigEntry {
                replay_nonce: derive_voucher_hash(
                    network_type,
                    "voucher_replay_nonce",
                    &[
                        genesis_seed,
                        &config.replay_nonce,
                        &chain_id_bytes,
                        class_bytes,
                        config.domain_name.as_bytes(),
                        config.id.as_bytes(),
                        &voucher_index_bytes,
                        &root_generation_bytes,
                        policy_bytes,
                        pool_bytes,
                    ],
                ),
                ..config
            },
        });
    }

    Ok(records)
}
