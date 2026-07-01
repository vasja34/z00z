//! Shared wallet export and verifier types.
//!
//! The legacy in-wallet snapshot record has been removed. This module now
//! carries only the shared password-verifier, receiver-deriver, and explicit
//! export-pack types used by the canonical backup and restore paths.

use serde::{Deserialize, Serialize};

include!("persistence_types.rs");
