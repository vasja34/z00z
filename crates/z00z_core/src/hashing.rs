//! Convenience hashing aliases for Z00Z Core.
//!
//! These aliases exist to keep call sites concise while preserving the exact
//! hash behavior (Blake2b-512 vs Blake2b-256) currently used in the protocol.

use z00z_crypto::{DomainHasher, DomainHasher256};

use crate::domains::{
    ActionDescriptorHashDomain, ActionPoolDescriptorHashDomain, GenesisAssetIdDomainDevnet,
    GenesisAssetIdDomainMainnet, GenesisAssetIdDomainTestnet, GenesisBlindingDomainDevnet,
    GenesisBlindingDomainMainnet, GenesisBlindingDomainTestnet, GenesisNonceDomain,
    GenesisRngSeedDomainDevnet, GenesisRngSeedDomainMainnet, GenesisRngSeedDomainTestnet,
    GenesisStateHashDomain, MetadataHashDomain, NonceDerivationDomain, OwnerSignatureDomain,
    PolicyDescriptorHashDomain, RegistryHashDomain, TestAssetIdDomain,
};

use z00z_crypto::domains::{AssetIdHashDomain, ChecksumHashDomain};

// Assets
pub type MetadataHasher = DomainHasher<MetadataHashDomain>;
pub type OwnerSigHasher = DomainHasher<OwnerSignatureDomain>;
pub type RegistryHasher = DomainHasher<RegistryHashDomain>;
pub type ActionDescriptorHasher = DomainHasher<ActionDescriptorHashDomain>;
pub type ActionPoolDescriptorHasher = DomainHasher<ActionPoolDescriptorHashDomain>;
pub type PolicyDescriptorHasher = DomainHasher<PolicyDescriptorHashDomain>;
pub type NonceDeriveHasher = DomainHasher<NonceDerivationDomain>;
pub type GenesisNonceHasher = DomainHasher<GenesisNonceDomain>;

// Asset ID / checksum (domain types live in z00z_crypto)
pub type AssetIdHasher = DomainHasher<AssetIdHashDomain>;
pub type ChecksumHasher = DomainHasher256<ChecksumHashDomain>;

// Test-only
pub type TestAssetIdHasher = DomainHasher<TestAssetIdDomain>;

// Genesis
pub type GenesisStateHasher = DomainHasher<GenesisStateHashDomain>;

pub type GenesisBlindDevnetHasher = DomainHasher<GenesisBlindingDomainDevnet>;
pub type GenesisBlindTestnetHasher = DomainHasher<GenesisBlindingDomainTestnet>;
pub type GenesisBlindMainnetHasher = DomainHasher<GenesisBlindingDomainMainnet>;

pub type GenesisRngDevnetHasher = DomainHasher<GenesisRngSeedDomainDevnet>;
pub type GenesisRngTestnetHasher = DomainHasher<GenesisRngSeedDomainTestnet>;
pub type GenesisRngMainnetHasher = DomainHasher<GenesisRngSeedDomainMainnet>;

pub type GenesisAssetIdDevnetHasher = DomainHasher<GenesisAssetIdDomainDevnet>;
pub type GenesisAssetIdTestnetHasher = DomainHasher<GenesisAssetIdDomainTestnet>;
pub type GenesisAssetIdMainnetHasher = DomainHasher<GenesisAssetIdDomainMainnet>;
