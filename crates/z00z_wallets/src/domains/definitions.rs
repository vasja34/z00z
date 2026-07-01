//! Centralized domain definitions for cryptographic operations.
//!
//! This module is the single source of truth for `hash_domain!()` usage in
//! `z00z_wallets`. Domain identifiers are part of persistent and/or wire
//! contracts; change with extreme care.

use z00z_crypto::expert::hash_domain;
use z00z_crypto::expert::traits::DomainSeparation;

// ============================================================================
// KEY DERIVATION DOMAINS (HD WALLET)
// ============================================================================

hash_domain!(WalletMasterKeyDomain, "z00z.wallets.master_key", 1);
#[rustfmt::skip]
hash_domain!(WalletKeyDerivationDomain, "z00z.wallets.key.derivation", 1);
hash_domain!(WalletBIP44PaymentDomain, "z00z.wallets.bip44.payment", 1);
hash_domain!(WalletBIP44ChangeDomain, "z00z.wallets.bip44.change", 1);
hash_domain!(WalletBIP44Domain, "z00z.wallets.bip44", 1);
hash_domain!(WalletPaymentDomain, "z00z.wallets.payment", 1);
hash_domain!(WalletChangeDomain, "z00z.wallets.change", 1);
#[rustfmt::skip]
hash_domain!(WalletMessageSigningDomain, "z00z.wallets.message_signing", 1);
hash_domain!(WalletBlindingDomain, "z00z.wallets.blinding", 1);
hash_domain!(WalletSessionDomain, "z00z.wallets.session", 1);

// Deterministic nonce domain for Schnorr signing.
// Separate prod/test to prevent domain reuse across environments.
hash_domain!(WalletSignNonceProdDomain, "z00z.wallets.sign_nonce.prod", 1);
hash_domain!(WalletSignNonceTestDomain, "z00z.wallets.sign_nonce.test", 1);

// ============================================================================
// WALLET HASHING DOMAINS (CHECKSUMS, IDS, HELPERS)
// ============================================================================

// Note: PaymentKeyDomain, ChangeKeyDomain, Bip44Domain, SessionKeyDomain, BlindingKeyDomain
// were duplicates of WalletPaymentDomain, WalletChangeDomain, WalletBIP44Domain, etc.
// They have been removed to ensure exactly one domain per operation.

hash_domain!(TxIdDomain, "z00z.wallets.tx_id", 1);
// Canonical Tx hash domain used by RPC and storage.
hash_domain!(TxHashDomain, "z00z.wallets.tx.hash", 1);
// Schnorr challenge domain for transaction signing
// Note: This is Z00Z-specific and differs from Tari's "com.tari.schnorr_signature"
// Z00Z uses "z00z.wallets.schnorr_challenge" for protocol separation
#[rustfmt::skip]
hash_domain!(SchnorrChallengeDomain, "z00z.wallets.schnorr_challenge", 1);

#[rustfmt::skip]
hash_domain!(EncryptionChecksumDomain, "z00z.wallets.encryption_checksum", 1);
hash_domain!(PasswordBloomDomain, "z00z.wallets.password_bloom", 1);
hash_domain!(IndexMacDomain, "z00z.wallets.index_mac", 1);
#[rustfmt::skip]
hash_domain!(SnapshotChecksumDomain, "z00z.wallets.snapshot_checksum", 1);
hash_domain!(WalletIntegrityDomain, "z00z.wallets.wallet_integrity", 1);

hash_domain!(WalletFileIdDomain, "z00z.wallets.file_id", 1);
hash_domain!(WalletSeedSaltDomain, "z00z.wallets.seed_salt", 1);
hash_domain!(WalletFingerprintDomain, "z00z.wallets.fingerprint", 1);
hash_domain!(PayRefDomain, "z00z.wallets.tx.pay_ref", 1);
hash_domain!(CardEntryDomain, "z00z.wallets.chain.card_entry", 1);
hash_domain!(
    RetryDigestDomain,
    "z00z.wallet.stealth.retry_digest.prod",
    1
);

// Address hashing domain for cryptographic contexts (prod).
hash_domain!(
    WalletAddressHashProdDomain,
    "z00z.wallets.address_hash.prod",
    1
);

// ============================================================================
// WALLET ENTITY DOMAINS
// ============================================================================

hash_domain!(WalletIdDomain, "z00z.wallets.wallet.wallet_id", 2);

// ============================================================================
// WALLET SERVICE / RPC DOMAINS
// ============================================================================

hash_domain!(AeadEnvelopeDomain, "z00z.crypto.aead.envelope", 1);
#[rustfmt::skip]
hash_domain!(WalletPasswordVerifierDomain, "z00z.wallets.wallet_service.password_verifier", 1);
#[rustfmt::skip]
hash_domain!(WalletKeyFingerprintDomain, "z00z.wallets.rpc.key_fingerprint", 1);

// ============================================================================
// ENCRYPTION DOMAINS
// ============================================================================

#[rustfmt::skip]
hash_domain!(WalletEncryptionKeyDomain, "z00z.wallets.encryption.key_derivation", 1);
#[rustfmt::skip]
hash_domain!(WalletEncryptionHkdfInfoDomain, "z00z.wallets.encryption.hkdf_info", 1);

// ============================================================================
// BACKUP DOMAINS
// ============================================================================

#[rustfmt::skip]
hash_domain!(WalletBackupAadTagDomain, "z00z.crypto.wallet_backup.aad", 1);
#[rustfmt::skip]
hash_domain!(WalletBackupChecksumDomain, "z00z.crypto.wallet_backup.checksum", 1);

// ============================================================================
// SEED (CIPHER SEED) DOMAINS
// ============================================================================

hash_domain!(CipherSeedAadTagDomain, "z00z.crypto.cipher_seed.aad", 1);
#[rustfmt::skip]
hash_domain!(CipherSeedChecksumDomain, "z00z.crypto.cipher_seed.checksum", 1);

// ============================================================================
// REDB WALLET CRYPTO DOMAINS
// ============================================================================

#[rustfmt::skip]
hash_domain!(Z00ZRedbWalletAadIdDomain, "z00z.crypto.redb_wallet_crypto.wallet_aad_id", 1);
#[rustfmt::skip]
hash_domain!(RedbWalletDataKeyDomain, "z00z.crypto.redb_wallet_crypto.hkdf.data_key", 1);
#[rustfmt::skip]
hash_domain!(RedbWalletIndexKeyDomain, "z00z.crypto.redb_wallet_crypto.hkdf.index_key", 1);
#[rustfmt::skip]
hash_domain!(RedbWalletIntegrityKeyDomain, "z00z.crypto.redb_wallet_crypto.hkdf.integrity_key", 1);

// ============================================================================
// DATABASE INDEX DOMAINS
// ============================================================================

hash_domain!(WalletDbIndexDomain, "z00z.wallets.db.index", 1);

// ============================================================================
// ADDRESS DOMAINS
// ============================================================================

// Domain for Z00Z bech32 address checksums
hash_domain!(AddressChecksumDomain, "z00z.wallets.address.checksum", 1);

// ============================================================================
// ADDRESS MANAGER CACHE DOMAINS
// ============================================================================

/// Domain for receiver-cache state HMAC in test builds.
pub struct ReceiverCacheHmacTestDomain;

/// Domain for receiver-cache state HMAC in non-test builds.
pub struct ReceiverCacheHmacProdDomain;

// IMPORTANT: These HMAC domains are slash-separated and must remain stable.
// They predate the dotted domain style used by `hash_domain!()`.

impl DomainSeparation for ReceiverCacheHmacTestDomain {
    fn version() -> u8 {
        1
    }

    fn domain() -> &'static str {
        "app/z00z_wallets/address/receiver_cache/test"
    }
}

impl DomainSeparation for ReceiverCacheHmacProdDomain {
    fn version() -> u8 {
        1
    }

    fn domain() -> &'static str {
        "app/z00z_wallets/address/receiver_cache/production"
    }
}

// ============================================================================
// BIP-32 TO RISTRETTO BRIDGE DOMAINS
// ============================================================================

// Domain separation for Ristretto bridge
hash_domain!(RistrettoBridgeDomain, "z00z.wallet.bip32_to_ristretto");

// ============================================================================
// STEALTH PROTOCOL DOMAINS (PHASE 1)
// ============================================================================

hash_domain!(
    WalletReceiverIdHashProdDomain,
    "z00z.wallet.stealth.receiver_id.prod",
    1
);
hash_domain!(
    WalletViewKeyHashProdDomain,
    "z00z.wallet.stealth.view_key.prod",
    1
);
hash_domain!(
    WalletIdentityKeyHashProdDomain,
    "z00z.wallet.stealth.identity_key.prod",
    1
);
hash_domain!(
    WalletDhKeyHashProdDomain,
    "z00z.wallet.stealth.dh_key.prod",
    1
);
hash_domain!(
    WalletOwnerTagHashProdDomain,
    "z00z.wallet.stealth.owner_tag.prod",
    1
);
hash_domain!(
    WalletTag16HashProdDomain,
    "z00z.wallet.stealth.tag16.prod",
    1
);
hash_domain!(
    WalletLeafAdHashProdDomain,
    "z00z.wallet.stealth.leaf_ad.prod",
    1
);
hash_domain!(
    WalletEphemeralRHashProdDomain,
    "z00z.wallet.stealth.ephemeral_r.prod",
    1
);
hash_domain!(SenderSaltDomain, "z00z.wallet.stealth.sender_salt.prod", 1);
hash_domain!(KdhFlexDomain, "z00z.wallet.stealth.k_dh_flex.prod", 1);
hash_domain!(KdhExtDomain, "z00z.wallet.stealth.k_dh_ext.prod", 1);
hash_domain!(PackKeyProdDomain, "z00z.wallet.stealth.pack_key.prod", 1);
hash_domain!(
    PackNonceProdDomain,
    "z00z.wallet.stealth.pack_nonce.prod",
    1
);
hash_domain!(SOutProdDomain, "z00z.wallet.stealth.s_out.prod", 1);
hash_domain!(EncKeyDomain, "z00z.wallet.stealth.enc_key.prod", 1);
hash_domain!(MacKeyDomain, "z00z.wallet.stealth.mac_key.prod", 1);
hash_domain!(RecoverRDomain, "z00z.wallet.stealth.recover_r.prod", 1);
hash_domain!(
    IdentitySignatureDomain,
    "z00z.wallet.stealth.identity_sig",
    1
);

#[cfg(test)]
hash_domain!(
    WalletReceiverIdHashTestDomain,
    "z00z.wallet.stealth.receiver_id.test",
    1
);
#[cfg(test)]
hash_domain!(
    WalletViewKeyHashTestDomain,
    "z00z.wallet.stealth.view_key.test",
    1
);
#[cfg(test)]
hash_domain!(
    WalletIdentityKeyHashTestDomain,
    "z00z.wallet.stealth.identity_key.test",
    1
);
#[cfg(test)]
hash_domain!(
    WalletDhKeyHashTestDomain,
    "z00z.wallet.stealth.dh_key.test",
    1
);
#[cfg(test)]
hash_domain!(
    WalletOwnerTagHashTestDomain,
    "z00z.wallet.stealth.owner_tag.test",
    1
);
#[cfg(test)]
hash_domain!(
    WalletTag16HashTestDomain,
    "z00z.wallet.stealth.tag16.test",
    1
);
#[cfg(test)]
hash_domain!(
    WalletLeafAdHashTestDomain,
    "z00z.wallet.stealth.leaf_ad.test",
    1
);
#[cfg(test)]
hash_domain!(
    WalletEphemeralRHashTestDomain,
    "z00z.wallet.stealth.ephemeral_r.test",
    1
);
#[cfg(test)]
hash_domain!(
    SenderSaltTestDomain,
    "z00z.wallet.stealth.sender_salt.test",
    1
);
#[cfg(test)]
hash_domain!(KdhFlexTestDomain, "z00z.wallet.stealth.k_dh_flex.test", 1);
#[cfg(test)]
hash_domain!(KdhExtTestDomain, "z00z.wallet.stealth.k_dh_ext.test", 1);
#[cfg(test)]
hash_domain!(EncKeyTestDomain, "z00z.wallet.stealth.enc_key.test", 1);
#[cfg(test)]
hash_domain!(MacKeyTestDomain, "z00z.wallet.stealth.mac_key.test", 1);
#[cfg(test)]
hash_domain!(RecoverRTestDomain, "z00z.wallet.stealth.recover_r.test", 1);
#[cfg(test)]
hash_domain!(
    IdentitySignatureTestDomain,
    "z00z.wallet.stealth.identity_sig.test",
    1
);

// ============================================================================
// DOMAIN UNIQUENESS TESTS
// ============================================================================

#[cfg(test)]
#[path = "test_definitions.rs"]
mod tests;
