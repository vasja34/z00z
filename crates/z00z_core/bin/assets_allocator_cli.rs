//! # Z00Z Genesis Assets Allocator CLI
//!
//! **Specialized tool for allocating genesis assets into EpochState.**
//!
//! ## Purpose
//!
//! This CLI is the experimental allocation lane for post-bootstrap processing:
//! - Takes raw genesis exports from `z00z_core::genesis::run_genesis`
//! - Allocates them into proper `EpochState` structure
//! - Validates and exports the final state
//!
//! ## Why This Exists
//!
//! The canonical genesis boundary should focus on:
//! - ✅ Cryptographic asset generation (commitments, proofs)
//! - ✅ Deterministic randomness and serialization
//!
//! It should **NOT** directly use runtime state structures like:
//! - ❌ `EpochState` (runtime state management)
//! - ❌ `EpochMeta` (runtime metadata)
//! - ❌ `UnspentSet` / `SpentSet` (runtime asset tracking)
//!
//! This separation ensures clean architecture:
//! ```text
//! ┌─────────────────────────────────────┐
//! │  z00z_core::genesis                 │
//! │  Generate raw cryptographic assets  │
//! └──────────────┬──────────────────────┘
//!                │ Raw AssetOutputs
//!                ▼
//! ┌─────────────────────────────────────┐
//! │  assets_allocator_cli.rs (THIS)     │
//! │  Allocate into EpochState           │
//! └──────────────┬──────────────────────┘
//!                │ Complete EpochState
//!                ▼
//! ┌─────────────────────────────────────┐
//! │  Runtime / Node Initialization      │
//! └─────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```bash
//! # Planned registered target name when this scaffold becomes live
//! cargo run --bin assets_allocator_cli -- \
//!     --config configs/devnet_genesis_config.yaml \
//!     --output genesis_state.bin
//! ```
//!
//! Today this file is a scaffold source, not a registered `[[bin]]` target.
//!
//! ## Implementation Status
//!
//! This CLI is currently a scaffold and exits with a non-zero status.
//!
//! Current state:
//! - CLI scaffolding created
//! - Deprecation warnings added to `epoch.rs`
//! - Migration path documented
//!
//! Next steps:
//! 1. Move `EpochState` creation logic from the genesis runtime here
//! 2. Create allocation API that takes raw outputs
//! 3. Update `genesis_cli.rs` to use this tool
//! 4. Remove deprecated functions from genesis module
//!
//! ## Architecture Notes
//!
//! The allocator should:
//! - Accept configuration (network params, epoch ID, etc.)
//! - Call the canonical genesis boundary to get raw outputs
//! - Create `EpochState` with proper metadata
//! - Validate all invariants
//! - Export serialized state for node initialization

use std::process;

fn main() {
    eprintln!("🚧 Z00Z Genesis Assets Allocator CLI");
    eprintln!("═══════════════════════════════════════\n");
    eprintln!("⚠️  This tool is currently under development.");
    eprintln!();
    eprintln!("Purpose:");
    eprintln!("  Allocate genesis assets into EpochState for runtime initialization.");
    eprintln!();
    eprintln!("Current Status:");
    eprintln!("  • EpochState structures marked as deprecated in genesis");
    eprintln!("  • Migration path documented");
    eprintln!("  • CLI scaffolding created");
    eprintln!();
    eprintln!("Next Steps:");
    eprintln!("  1. Move EpochState creation from the genesis runtime");
    eprintln!("  2. Implement allocation logic");
    eprintln!("  3. Add validation and export");
    eprintln!();
    eprintln!("For now, please use:");
    eprintln!("  cargo run --bin genesis_cli -- --config <config.yaml>");
    eprintln!();
    eprintln!("See: crates/z00z_core/src/state/epoch.rs (deprecation notice)");

    process::exit(1);
}
