//! Low-level durable backend contracts for raw storage rows and journals.
//!
//! These contracts stay below `settlement::SettlementTreeBackend` and must not
//! become a second semantic authority for settlement roots, settlement paths,
//! proofs, or replay policy.

pub(crate) mod codec;
pub(crate) mod error;
pub(crate) mod memory;
pub(crate) mod query;
pub(crate) mod redb;
pub(crate) mod roots;
pub(crate) mod rows;
pub(crate) mod types;

pub(crate) type RawKv = (Vec<u8>, Vec<u8>);

/// Read-only byte-oriented transaction over durable backend tables.
pub trait ReadTxn {
    type Error;

    fn get(&self, table: &'static str, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;

    fn scan(&self, table: &'static str) -> Result<Vec<RawKv>, Self::Error>;
}

/// Writable byte-oriented transaction over durable backend tables.
pub trait WriteTxn: ReadTxn {
    fn put(&mut self, table: &'static str, key: &[u8], value: &[u8]) -> Result<(), Self::Error>;

    fn delete(&mut self, table: &'static str, key: &[u8]) -> Result<(), Self::Error>;

    fn commit(&mut self) -> Result<(), Self::Error>;
}

/// Durable backend that can open raw row transactions.
pub trait StorageBackend {
    type Error;
    type Reader: ReadTxn<Error = Self::Error>;
    type Writer: WriteTxn<Error = Self::Error>;

    fn read_txn(&self) -> Result<Self::Reader, Self::Error>;

    fn write_txn(&self) -> Result<Self::Writer, Self::Error>;
}

/// Durable journal owner for crash recovery and replay reconciliation.
///
/// Version 1 keeps the local durable journal as the baseline implementation
/// behind this seam. It must not become independent protocol truth.
pub trait JournalBackend {
    type Error;

    fn recover_journal(&self) -> Result<(), Self::Error>;
}
