// crates/z00z_core/src/assets/registry.rs
//
//! AssetDefinitionRegistry - Centralized registry for AssetDefinition instances
//!
//! Provides singleton-style access to AssetDefinition objects with Arc sharing
//! for memory efficiency. Thread-safe for concurrent access.
//!
//! ## Architecture
//!
//! This module provides the core storage and access layer for asset definitions.
//! Related functionality has been extracted to separate modules:
//!
//! - **Config Parsing**: `registry_catalog` handles secondary catalog YAML parsing (internal)
//! - **Snapshot Types**: [`snapshot`] module defines RegistrySnapshot/RegistryVersion
//! - **Wire Format**: [`wire`] module handles serialization DTOs
//! - **Test Fixtures**: `crates/z00z_core/tests/test_assets_fixtures.rs` provides test utilities
//!
//! ## Core Responsibilities
//!
//! - **Storage**: Thread-safe BTreeMap with RwLock for concurrent access
//! - **Versioning**: Atomic version tracking for snapshot updates
//! - **Arc Sharing**: Efficient memory usage via `Arc<AssetDefinition>`
//! - **Explicit Owners**: passed AssetDefinitionRegistry instances own primary writes
//! - **Global Access**: GLOBAL_ASSET_REGISTRY stays a read-mostly fallback seam
//!
//! ## Catalog Loading
//!
//! Secondary catalog parsing delegates to `registry_catalog`:
//! ```rust,no_run
//! use z00z_core::assets::registry::AssetDefinitionRegistry;
//! use z00z_utils::prelude::{NoopLogger, NoopMetrics, SystemTimeProvider};
//! use std::sync::Arc;
//! use std::path::Path;
//!
//! let registry = AssetDefinitionRegistry::load_catalog_from_yaml(
//!     Path::new("config/assets.yaml"),
//!     Arc::new(NoopLogger),
//!     Arc::new(NoopMetrics),
//!     Arc::new(SystemTimeProvider)
//! )?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Snapshot Operations
//!
//! Support for validator → wallet synchronization:
//! ```rust,no_run
//! # use z00z_core::assets::registry::AssetDefinitionRegistry;
//! # use z00z_utils::prelude::{NoopLogger, NoopMetrics, SystemTimeProvider};
//! # use std::sync::Arc;
//! let registry = AssetDefinitionRegistry::new(
//!     Arc::new(NoopLogger),
//!     Arc::new(NoopMetrics),
//!     Arc::new(SystemTimeProvider)
//! );
//!
//! // Create snapshot for transmission
//! let snapshot = registry.create_snapshot()?;
//!
//! // Update from received snapshot
//! registry.update_from_snapshot(snapshot)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Thread Safety
//!
//! All operations use RwLock for concurrent access. Multiple readers can access
//! simultaneously, writers have exclusive access. Lock ordering is documented
//! in the struct to prevent deadlocks.
//!
//! [`snapshot`]: super::snapshot
//! [`wire`]: super::wire

// ============================================================================
// Imports — Organized by convention
// ============================================================================

// External crates
use once_cell::sync::Lazy;

// ALWAYS use z00z_utils abstractions - ONE SOURCE OF TRUTH
use z00z_utils::prelude::{
    Logger, MetricsSink, NoopLogger, NoopMetrics, SystemTimeProvider, TimeProvider,
};

// Standard library
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Arc, RwLock};

// Internal modules
use super::assets::AssetError;
use super::definition::AssetDefinition;
#[allow(unused_imports)] // Used in #[cfg(test)]
use super::policy_flags::BURNABLE;
use super::registry_catalog; // Secondary registry-catalog parsing helpers

#[path = "registry_config.rs"]
mod registry_config;
#[path = "registry_core.rs"]
mod registry_core;
#[path = "registry_snapshot.rs"]
mod registry_snapshot;

// ============================================================================
// Debug Lock Ordering Assertions (H3 Task 2)
// ============================================================================
//
// Provides compile-time documentation and runtime checks (in debug mode only)
// to prevent lock ordering violations that could cause deadlocks.
//
// Lock ordering rule: definitions → version (never reverse)
//
// These assertions are enabled only in debug builds (`cfg(debug_assertions)`)
// to prevent performance overhead in release builds.
//
// NOTE: This is a best-effort safety mechanism. The assertions help catch
// obvious violations during development and testing, but cannot guarantee
// complete deadlock-freedom at runtime. Proper code review and testing
// remain essential.

// ============================================================================
// Type Aliases - Foundation for Arc-based Registry Architecture
// ============================================================================

/// Type alias for asset definition ID (32-byte hash)
pub type AssetId = [u8; 32];

/// Single asset definition wrapped in Arc (cheap to clone across threads)
pub type ArcDefinition = Arc<AssetDefinition>;

/// Local registry: AssetId → ArcDefinition mapping (BTreeMap for deterministic ordering)
pub type DefinitionRegistry = BTreeMap<AssetId, ArcDefinition>;

/// Shared immutable snapshot of registry (`Arc<BTreeMap>`) that can be cloned to threads
/// This is what WalletContext would hold - lock-free reads after initialization
pub type SharedRegistry = Arc<DefinitionRegistry>;

/// 🌍 Global AssetDefinition registry singleton
///
/// Thread-safe process-global fallback for accessing AssetDefinition objects.
/// Primary write ownership stays with explicit registry instances; global writes
/// are reserved for narrow compatibility and adapter boundaries.
/// Initialized lazily on first access with Noop implementations for simplicity.
///
/// # Examples
///
/// ```ignore
/// use z00z_core::assets::registry::GLOBAL_ASSET_REGISTRY;
/// use z00z_core::assets::definition::AssetDefinition;
/// use z00z_core::assets::AssetClass;
///
/// // Create definition
/// let def = AssetDefinition::new(
///     [42u8; 32],
///     AssetClass::Coin,
///     "Z00Z".into(),
///     "Z00Z".into(),
///     8,
///     50_000,
///     100_000_000,
///     "z00z.io".into(),
///     1,
///     1,
///     GAS | BURNABLE, // combined flags
///     None,
/// ).unwrap();
///
/// // Get reference to registry and insert
/// let arc_def = GLOBAL_ASSET_REGISTRY.insert(def).ok();
///
/// // Retrieve by ID - will succeed if insert was first call
/// if let Ok(Some(retrieved)) = GLOBAL_ASSET_REGISTRY.get(&[42u8; 32]) {
///     if let Some(arc_def) = arc_def {
///         assert!(std::sync::Arc::ptr_eq(&arc_def, &retrieved));
///     }
/// }
/// ```
pub static GLOBAL_ASSET_REGISTRY: Lazy<AssetDefinitionRegistry> = Lazy::new(|| {
    AssetDefinitionRegistry::new(
        Arc::new(NoopLogger),
        Arc::new(NoopMetrics),
        Arc::new(SystemTimeProvider),
    )
});

/// 🗂️ Centralized registry for AssetDefinition instances
///
/// Manages all AssetDefinition objects in the system, providing:
/// - `Arc<AssetDefinition>` sharing for memory efficiency
/// - Thread-safe concurrent access via RwLock
/// - Loading from configuration files
/// - Fast lookup by asset ID
///
/// # Architecture
///
/// ```text
/// AssetDefinitionRegistry (singleton)
///     │
///     ├─ definitions: BTreeMap<[u8; 32], Arc<AssetDefinition>>
///     ├─ version: u64 (tracks registry version for downgrade prevention)
///     │
///     └─ Thread-safe access via RwLock
///            ├─ read() → shared access for lookups
///            └─ write() → exclusive access for insertions
/// ```
///
/// # Memory Model
///
/// - Each AssetDefinition stored once as `Arc<AssetDefinition>`
/// - Multiple Asset instances share the same Arc (8 bytes per reference)
/// - Registry holds the "canonical" Arc instance
///
/// # Thread Safety
///
/// - RwLock allows multiple concurrent readers
/// - Single writer at a time for insertions
/// - No deadlocks (no nested locks)
///
/// # Examples
///
/// ```ignore
/// use z00z_core::assets::registry::AssetDefinitionRegistry;
/// use z00z_core::assets::definition::AssetDefinition;
/// use z00z_core::assets::AssetClass;
/// use z00z_core::assets::policy_flags::{GAS, BURNABLE};
/// use z00z_utils::logger::NoopLogger;
/// use z00z_utils::metrics::NoopMetrics;
/// use z00z_utils::time::SystemTimeProvider;
/// use std::sync::Arc;
///
/// // Create registry with dependencies
/// let mut registry = AssetDefinitionRegistry::new(
///     Arc::new(NoopLogger),
///     Arc::new(NoopMetrics),
///     Arc::new(SystemTimeProvider::default()),
/// );
///
/// // Insert definition
/// let def = AssetDefinition::new(
///     [42u8; 32],
///     AssetClass::Coin,
///     "Z00Z".into(),
///     "Z00Z".into(),
///     8,
///     50_000,
///     100_000_000,
///     "z00z.io".into(),
///     1,
///     1,
///     GAS | BURNABLE,  // policy_flags
///     None,
/// ).unwrap();
///
/// let arc_def = registry.insert(def);
///
/// // Retrieve by ID
/// let retrieved = registry.get(&[42u8; 32]).expect("definition exists");
/// assert!(Arc::ptr_eq(&arc_def, &retrieved));
/// ```
///
/// ## Lock Ordering (Deadlock Prevention)
///
/// **CRITICAL**: This struct uses two separate `RwLock`s for concurrent access.
/// To prevent deadlocks, locks MUST be acquired in the following order:
///
/// 1. **First**: `definitions` lock (via `defs_read()` or `defs_write()`)
/// 2. **Second**: `version` lock (via `version_read()` or `version_write()`)
///
/// **Never** acquire locks in reverse order (version → definitions).
///
/// ### Correct Usage
///
/// ```rust,ignore
/// // ✅ CORRECT: Single lock only
/// let defs = self.defs_read()?;
///
/// // ✅ CORRECT: definitions → version order
/// let mut defs = self.defs_write()?;
/// *defs = new_registry;
/// let mut version = self.version_write()?;
/// *version = new_version;
/// ```
///
/// ### Incorrect Usage (Causes Deadlock)
///
/// ```rust,ignore
/// // ❌ WRONG: version → definitions order (DEADLOCK RISK!)
/// let version = self.version_read()?;
/// let defs = self.defs_read()?;  // DEADLOCK if another thread holds defs first
/// ```
///
/// All helper methods (`defs_read`, `defs_write`, `version_read`, `version_write`)
/// document their position in the lock ordering hierarchy.
pub struct AssetDefinitionRegistry {
    /// Internal storage: asset_id → `Arc<AssetDefinition>`
    /// Uses DefinitionRegistry type alias for consistency
    definitions: RwLock<DefinitionRegistry>,

    /// Registry version for downgrade prevention
    /// Incremented on each accepted state change
    version: RwLock<u64>,

    /// Logger for registry operations (MANDATORY - always use z00z_utils)
    logger: Arc<dyn Logger>,

    /// Metrics sink for tracking registry operations (MANDATORY - always use z00z_utils)
    metrics: Arc<dyn MetricsSink>,

    /// Time provider for timestamps (MANDATORY - always use z00z_utils)
    time: Arc<dyn TimeProvider>,
}

impl AssetDefinitionRegistry {}

impl Default for AssetDefinitionRegistry {
    fn default() -> Self {
        Self::new(
            Arc::new(NoopLogger),
            Arc::new(NoopMetrics),
            Arc::new(SystemTimeProvider),
        )
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[path = "test_registry.rs"]
mod tests;
