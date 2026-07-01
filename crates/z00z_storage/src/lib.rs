#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

pub mod backend;
pub mod checkpoint;
mod error;
pub mod serialization;
pub mod settlement;
pub mod snapshot;
// Shared non-production fixtures stay in `src` so integration tests and
// benches can reuse one crate-local surface without widening production APIs.
#[doc(hidden)]
pub mod fixture_support;

pub use self::error::{CheckpointError, SerializationError};
