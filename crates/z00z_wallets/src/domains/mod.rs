//! Domain-separation and hashing modules for wallet cryptography.

pub mod definitions;
pub mod hashing;

pub use self::definitions::{
    AddressChecksumDomain, AeadEnvelopeDomain, CardEntryDomain, CipherSeedAadTagDomain,
    CipherSeedChecksumDomain, EncKeyDomain, EncryptionChecksumDomain, IdentitySignatureDomain,
    IndexMacDomain, KdhExtDomain, KdhFlexDomain, MacKeyDomain, PackKeyProdDomain,
    PackNonceProdDomain, PasswordBloomDomain, PayRefDomain, ReceiverCacheHmacProdDomain,
    ReceiverCacheHmacTestDomain, RecoverRDomain, RedbWalletDataKeyDomain, RedbWalletIndexKeyDomain,
    RedbWalletIntegrityKeyDomain, RetryDigestDomain, RistrettoBridgeDomain, SOutProdDomain,
    SchnorrChallengeDomain, SenderSaltDomain, SnapshotChecksumDomain, TxHashDomain, TxIdDomain,
    WalletAddressHashProdDomain, WalletBIP44ChangeDomain, WalletBIP44Domain,
    WalletBIP44PaymentDomain, WalletBackupAadTagDomain, WalletBackupChecksumDomain,
    WalletBlindingDomain, WalletChangeDomain, WalletDbIndexDomain, WalletDhKeyHashProdDomain,
    WalletEncryptionHkdfInfoDomain, WalletEncryptionKeyDomain, WalletEphemeralRHashProdDomain,
    WalletFileIdDomain, WalletFingerprintDomain, WalletIdDomain, WalletIdentityKeyHashProdDomain,
    WalletIntegrityDomain, WalletKeyDerivationDomain, WalletKeyFingerprintDomain,
    WalletLeafAdHashProdDomain, WalletMasterKeyDomain, WalletMessageSigningDomain,
    WalletOwnerTagHashProdDomain, WalletPasswordVerifierDomain, WalletPaymentDomain,
    WalletReceiverIdHashProdDomain, WalletSeedSaltDomain, WalletSessionDomain,
    WalletSignNonceProdDomain, WalletSignNonceTestDomain, WalletTag16HashProdDomain,
    WalletViewKeyHashProdDomain, Z00ZRedbWalletAadIdDomain,
};
#[cfg(test)]
pub use self::definitions::{
    EncKeyTestDomain, IdentitySignatureTestDomain, KdhExtTestDomain, KdhFlexTestDomain,
    MacKeyTestDomain, RecoverRTestDomain, SenderSaltTestDomain, WalletDhKeyHashTestDomain,
    WalletEphemeralRHashTestDomain, WalletIdentityKeyHashTestDomain, WalletLeafAdHashTestDomain,
    WalletOwnerTagHashTestDomain, WalletReceiverIdHashTestDomain, WalletTag16HashTestDomain,
    WalletViewKeyHashTestDomain,
};
pub use self::hashing::{
    canonicalize_bytes, canonicalize_string, canonicalize_u32, canonicalize_u64,
    compute_encryption_checksum, compute_fingerprint, compute_index_mac, compute_password_bloom,
    compute_pay_ref, compute_schnorr_challenge, compute_seed_salt, compute_snapshot_checksum,
    compute_tx_id, compute_wallet_file_id, compute_wallet_integrity, compute_wallet_seed_hash,
    derive_change_key, derive_payment_key, derive_wallet_master_key, redb_wallet_hkdf_info_data,
    redb_wallet_hkdf_info_index, redb_wallet_hkdf_info_integrity, verify_index_mac, Bip44Hasher,
    BlindingKeyHasher, ChallengeBytes, ChallengeSize, ChangeKeyHasher, EncryptionChecksumHasher,
    IndexMacHasher, PasswordBloomHasher, PayRefHasher, PaymentKeyHasher, RedbWalletDataKeyHasher,
    RedbWalletIndexKeyHasher, RedbWalletIntegrityKeyHasher, SchnorrChallengeHasher,
    SessionKeyHasher, SnapshotChecksumHasher, TxHashHasher, TxIdHasher, WalletFileIdHasher,
    WalletFingerprintHasher, WalletIntegrityHasher, WalletMasterKeyHasher, WalletSeedSaltHasher,
};
