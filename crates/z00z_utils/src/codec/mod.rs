//! Codec module for serialization/deserialization
//!
//! Provides trait-based abstraction for different serialization formats.
//! Supports JSON, Bincode, and YAML out of the box.
//!
//! `z00z_utils` intentionally owns the narrow JSON compatibility boundary for
//! `Value` and `json!()` so the workspace does not depend on accidental raw
//! `serde_json` imports drifting through internal helpers.

pub mod traits;
pub use traits::{Codec, CodecError};

mod canonical_json;
pub use canonical_json::to_canonical_json_bytes;

pub mod json;
pub use json::JsonCodec;

/// Narrow JSON compatibility surface owned by `z00z_utils`.
///
/// This exception remains intentional because the verified workspace blast radius
/// still includes `Value` and `json!()` consumers outside `z00z_utils`. New code
/// inside this crate should route through this surface instead of reaching for
/// raw `serde_json` directly.
pub use serde_json::{json, Map, Value};

pub mod bincode;
pub use bincode::BincodeCodec;

pub mod yaml;
pub use yaml::YamlCodec;

#[cfg(test)]
mod test_codec;
