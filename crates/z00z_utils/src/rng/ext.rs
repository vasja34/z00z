//! RNG extension helpers.
//!
//! This module exists to keep direct `rand::*` imports out of business logic.

use rand::RngCore;

/// Extension trait for filling bytes without importing `rand::RngCore` downstream.
pub trait RngCoreExt {
    /// Fill the provided slice with random bytes.
    fn fill_bytes_ext(&mut self, dest: &mut [u8]);
}

impl<T: RngCore> RngCoreExt for T {
    fn fill_bytes_ext(&mut self, dest: &mut [u8]) {
        RngCore::fill_bytes(self, dest)
    }
}
