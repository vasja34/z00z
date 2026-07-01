//! RNG provider trait and implementations
//!
//! This module provides a random number generator provider abstraction.
//!
//! # Examples
//!
//! ```no_run
//! use z00z_utils::rng::{SecureRngProvider, SystemRngProvider};
//! use rand::RngCore;
//!
//! let provider = SystemRngProvider;
//! let mut rng = provider.rng();
//! let mut random_bytes = [0u8; 32];
//! rng.fill_bytes(&mut random_bytes);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#[cfg(any(
    test,
    feature = "deterministic-rng",
    feature = "test-utils",
    feature = "test-params-fast"
))]
mod deterministic;
mod ext;
#[cfg(any(
    test,
    feature = "deterministic-rng",
    feature = "test-utils",
    feature = "test-params-fast"
))]
mod mock;
mod system;
mod traits;

#[cfg(any(
    test,
    feature = "deterministic-rng",
    feature = "test-utils",
    feature = "test-params-fast"
))]
pub use deterministic::DeterministicRngProvider;
pub use ext::RngCoreExt;
#[cfg(any(
    test,
    feature = "deterministic-rng",
    feature = "test-utils",
    feature = "test-params-fast"
))]
pub use mock::MockRngProvider;
pub use system::SystemRngProvider;
pub use traits::{DeterministicRngSource, SecureRngProvider};

#[cfg(test)]
mod test_rng;
