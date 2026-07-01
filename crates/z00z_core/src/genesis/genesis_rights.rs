use super::{BTreeMap, BTreeSet, ChainType, DomainHasher, GenesisError, GenesisPolicyRecord};

use crate::domains::{
    GenesisRightDerivationDomainDevnet, GenesisRightDerivationDomainMainnet,
    GenesisRightDerivationDomainTestnet,
};
use crate::rights::{RightClassConfig, RightsConfigEntry};

pub const GENESIS_ROOT_GENERATION: u64 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GenesisRightLeaf {
    pub version: u8,
    pub terminal_id: [u8; 32],
    pub right_class: RightClassConfig,
    pub issuer_scope: [u8; 32],
    pub provider_scope: [u8; 32],
    pub holder_commitment: [u8; 32],
    pub control_commitment: [u8; 32],
    pub beneficiary_commitment: [u8; 32],
    pub payload_commitment: [u8; 32],
    pub valid_from: u64,
    pub valid_until: u64,
    pub challenge_from: u64,
    pub challenge_until: u64,
    pub use_nonce: [u8; 32],
    pub revocation_policy_id: [u8; 32],
    pub transition_policy_id: [u8; 32],
    pub challenge_policy_id: [u8; 32],
    pub disclosure_policy_id: [u8; 32],
    pub retention_policy_id: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GenesisRightRecord {
    pub right_id: String,
    pub right_index: u32,
    pub definition_id: [u8; 32],
    pub serial_id: u32,
    pub domain_name: String,
    pub metadata_purpose: String,
    pub leaf: GenesisRightLeaf,
}

fn derive_definition_id(
    entry: &RightsConfigEntry,
    chain_id: u32,
    network_type: ChainType,
) -> [u8; 32] {
    let chain_id_bytes = chain_id.to_le_bytes();
    derive_right_hash(
        network_type,
        "right_definition_id",
        &[
            entry.id.as_bytes(),
            entry.right_class.as_str().as_bytes(),
            entry.domain_name.as_bytes(),
            &chain_id_bytes,
        ],
    )
}

fn derive_right_hash(network_type: ChainType, label: &'static str, parts: &[&[u8]]) -> [u8; 32] {
    let hash_output = match network_type {
        ChainType::Devnet => {
            let mut hasher =
                DomainHasher::<GenesisRightDerivationDomainDevnet>::new_with_label(label);
            for part in parts {
                hasher.update(*part);
            }
            hasher.finalize()
        }
        ChainType::Testnet => {
            let mut hasher =
                DomainHasher::<GenesisRightDerivationDomainTestnet>::new_with_label(label);
            for part in parts {
                hasher.update(*part);
            }
            hasher.finalize()
        }
        ChainType::Mainnet => {
            let mut hasher =
                DomainHasher::<GenesisRightDerivationDomainMainnet>::new_with_label(label);
            for part in parts {
                hasher.update(*part);
            }
            hasher.finalize()
        }
    };

    let mut result = [0u8; 32];
    result.copy_from_slice(&hash_output.as_ref()[..32]);
    result
}

fn derive_policy_id(
    network_type: ChainType,
    policy_kind: &'static str,
    policy_value: &str,
    chain_id: u32,
    right_class: RightClassConfig,
) -> [u8; 32] {
    let chain_id_bytes = chain_id.to_le_bytes();
    derive_right_hash(
        network_type,
        policy_kind,
        &[
            policy_value.as_bytes(),
            right_class.as_str().as_bytes(),
            &chain_id_bytes,
        ],
    )
}

fn derive_binding(
    network_type: ChainType,
    label: &'static str,
    binding: &str,
    domain_name: &str,
    chain_id: u32,
) -> [u8; 32] {
    let chain_id_bytes = chain_id.to_le_bytes();
    derive_right_hash(
        network_type,
        label,
        &[binding.as_bytes(), domain_name.as_bytes(), &chain_id_bytes],
    )
}

fn derive_right_leaf(
    entry: &RightsConfigEntry,
    right_index: u32,
    genesis_seed: &[u8; 32],
    chain_id: u32,
    network_type: ChainType,
    root_generation: u64,
    policy_lookup: &BTreeMap<String, GenesisPolicyRecord>,
) -> GenesisRightLeaf {
    let chain_id_bytes = chain_id.to_le_bytes();
    let right_index_bytes = right_index.to_le_bytes();
    let root_generation_bytes = root_generation.to_le_bytes();
    let right_class = entry.right_class;
    let class_name = right_class.as_str().as_bytes();
    let right_id_bytes = entry.id.as_bytes();
    let domain_name_bytes = entry.domain_name.as_bytes();
    let beneficiary_binding = entry
        .beneficiary_fixture
        .as_deref()
        .unwrap_or(entry.holder_fixture.as_str());

    GenesisRightLeaf {
        version: 1,
        terminal_id: derive_right_hash(
            network_type,
            "right_terminal",
            &[
                genesis_seed,
                &chain_id_bytes,
                class_name,
                domain_name_bytes,
                right_id_bytes,
                &right_index_bytes,
                &root_generation_bytes,
            ],
        ),
        right_class,
        issuer_scope: derive_binding(
            network_type,
            "right_issuer_scope",
            entry.issuer_scope.as_str(),
            entry.domain_name.as_str(),
            chain_id,
        ),
        provider_scope: derive_binding(
            network_type,
            "right_provider_scope",
            entry.provider_scope.as_str(),
            entry.domain_name.as_str(),
            chain_id,
        ),
        holder_commitment: derive_binding(
            network_type,
            "right_holder_binding",
            entry.holder_fixture.as_str(),
            entry.domain_name.as_str(),
            chain_id,
        ),
        control_commitment: derive_binding(
            network_type,
            "right_control_binding",
            entry.control_fixture.as_str(),
            entry.domain_name.as_str(),
            chain_id,
        ),
        beneficiary_commitment: derive_binding(
            network_type,
            "right_beneficiary_binding",
            beneficiary_binding,
            entry.domain_name.as_str(),
            chain_id,
        ),
        payload_commitment: derive_right_hash(
            network_type,
            "right_payload_commitment",
            &[
                genesis_seed,
                entry.payload_commitment_seed.as_bytes(),
                domain_name_bytes,
                class_name,
                entry.holder_fixture.as_bytes(),
                entry.control_fixture.as_bytes(),
                &chain_id_bytes,
                &right_index_bytes,
                &root_generation_bytes,
            ],
        ),
        valid_from: entry.valid_from,
        valid_until: entry.valid_until,
        challenge_from: entry.challenge_from,
        challenge_until: entry.challenge_until,
        use_nonce: derive_right_hash(
            network_type,
            "right_use_nonce",
            &[
                genesis_seed,
                right_id_bytes,
                &right_index_bytes,
                &chain_id_bytes,
            ],
        ),
        revocation_policy_id: resolve_right_policy_id(
            policy_lookup,
            network_type,
            "right_revocation_policy",
            entry.revocation_policy_id.as_str(),
            chain_id,
            right_class,
        ),
        transition_policy_id: resolve_right_policy_id(
            policy_lookup,
            network_type,
            "right_transition_policy",
            entry.transition_policy_id.as_str(),
            chain_id,
            right_class,
        ),
        challenge_policy_id: resolve_right_policy_id(
            policy_lookup,
            network_type,
            "right_challenge_policy",
            entry.challenge_policy_id.as_str(),
            chain_id,
            right_class,
        ),
        disclosure_policy_id: resolve_right_policy_id(
            policy_lookup,
            network_type,
            "right_disclosure_policy",
            entry.disclosure_policy_id.as_str(),
            chain_id,
            right_class,
        ),
        retention_policy_id: resolve_right_policy_id(
            policy_lookup,
            network_type,
            "right_retention_policy",
            entry.retention_policy_id.as_str(),
            chain_id,
            right_class,
        ),
    }
}

fn resolve_right_policy_id(
    policy_lookup: &BTreeMap<String, GenesisPolicyRecord>,
    network_type: ChainType,
    policy_kind: &'static str,
    policy_value: &str,
    chain_id: u32,
    right_class: RightClassConfig,
) -> [u8; 32] {
    if let Some(policy) = policy_lookup.get(policy_value) {
        return policy.policy_id.bytes();
    }

    derive_policy_id(
        network_type,
        policy_kind,
        policy_value,
        chain_id,
        right_class,
    )
}

pub(crate) fn generate_genesis_rights_with_policies(
    rights: &[RightsConfigEntry],
    policy_lookup: &BTreeMap<String, GenesisPolicyRecord>,
    genesis_seed: &[u8; 32],
    chain_id: u32,
    network_type: ChainType,
    root_generation: u64,
) -> Result<Vec<GenesisRightRecord>, GenesisError> {
    let mut records = Vec::new();
    let mut seen_terminals = BTreeSet::new();

    for entry in rights {
        for right_index in 0..entry.count {
            let leaf = derive_right_leaf(
                entry,
                right_index,
                genesis_seed,
                chain_id,
                network_type,
                root_generation,
                policy_lookup,
            );

            if !seen_terminals.insert(leaf.terminal_id) {
                return Err(GenesisError::TerminalCollision {
                    terminal_id: leaf.terminal_id,
                    error: format!(
                        "duplicate generated right terminal id for {}:{}",
                        entry.id, right_index
                    ),
                });
            }

            records.push(GenesisRightRecord {
                right_id: entry.id.clone(),
                right_index,
                definition_id: derive_definition_id(entry, chain_id, network_type),
                serial_id: right_index,
                domain_name: entry.domain_name.clone(),
                metadata_purpose: entry
                    .metadata
                    .as_ref()
                    .and_then(|metadata| metadata.get("purpose"))
                    .cloned()
                    .ok_or_else(|| GenesisError::RightDerivationFailed {
                        right_id: entry.id.clone(),
                        right_index,
                        error: "rights.metadata.purpose missing after validation".to_string(),
                    })?,
                leaf,
            });
        }
    }

    Ok(records)
}

#[cfg(test)]
pub(crate) fn generate_genesis_rights(
    rights: &[RightsConfigEntry],
    genesis_seed: &[u8; 32],
    chain_id: u32,
    network_type: ChainType,
    root_generation: u64,
) -> Result<Vec<GenesisRightRecord>, GenesisError> {
    generate_genesis_rights_with_policies(
        rights,
        &BTreeMap::new(),
        genesis_seed,
        chain_id,
        network_type,
        root_generation,
    )
}
