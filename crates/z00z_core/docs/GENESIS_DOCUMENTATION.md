# Genesis Module - Comprehensive Documentation

**Version:** 3.8 (Security Hardening Release)
**Status:** Production-Ready
**Last Updated:** 2025-12-10

---

## 📋 Table of Contents

1. [Module Overview](#module-overview)
2. [Architecture](#architecture)
3. [Core Components](#core-components)
4. [API Reference](#api-reference)
5. [Security Model](#security-model)
6. [Usage Examples](#usage-examples)
7. [Performance Characteristics](#performance-characteristics)
8. [Testing Strategy](#testing-strategy)
9. [Configuration Reference](#configuration-reference)
10. [Troubleshooting](#troubleshooting)

---

## 📖 Module Overview

### Purpose

The Genesis module generates the **initial state** of the Z00Z blockchain: a
deterministic bootstrap corpus of assets, policies, rights, and vouchers.
Every node MUST derive the same live bootstrap objects to prevent chain
splits.

The one authoritative object-family semantics matrix lives in
`OBJECT_FAMILY_SEMANTICS.md`. Use that file for the current bootstrap-vs-runtime
contract, `VoucherBootstrapEntryV1` semantics, and the narrowed `mintable`
wording.

`z00z_core::ObjectFamily` is the canonical caller-visible family vocabulary.
`z00z_core::assets::ObjectFamily` remains a compatibility facade only.

### Key Features

✅ **Deterministic Generation**: Same seed → identical bootstrap objects across all nodes
✅ **Nested Parallelism**: Efficient multi-core bootstrap generation (10× speedup)
✅ **Security Enhancements**:
- **M1**: Genesis seed validation (200-bit entropy minimum)
- **M2**: Batch proof verification (O(log n) complexity)
- **C2**: Genesis state integrity hash (prevents chain splits)
- **L4**: Resource limits (DoS protection)
- **L5**: Atomic file writes (RAII cleanup)

✅ **Multi-Format Export**: JSON + Bincode serialization
✅ **Comprehensive Testing**: 96 tests (27 unit + 69 integration) + performance benchmarks

### Design Philosophy

The Genesis module follows these core principles:

1. **Determinism First**: All operations are deterministic from the genesis seed
2. **Network Isolation**: Different networks (devnet/testnet/mainnet) use distinct cryptographic domains
3. **Security by Default**: All security enhancements enabled without configuration
4. **Performance**: Parallel generation using Rayon for maximum throughput
5. **Type Safety**: Compile-time family separation prevents mixing asset,
   right, voucher, and policy semantics

### Live Contract Snapshot

- `z00z_core::genesis::GenesisConfig` is the one canonical bootstrap authority.
  Split manifests must rehydrate back into that typed shape before validation
  or generation.
- `GenesisConfig` owns four live bootstrap sections: `assets`, `rights`,
  `policies`, and `vouchers`.
- `VoucherBootstrapEntryV1` is bootstrap input only. The live runtime voucher
  object is `VoucherLeaf`.
- Policies are part of the bootstrap corpus, but runtime policy handling binds
  `PolicyDescriptorV1` by hash rather than materializing a policy settlement
  leaf.
- `mintable` remains a definition/catalog flag on the current tree. The public
  runtime object RPC does not expose a generic post-genesis asset-mint
  selector.

---

## 🏗️ Architecture

### High-Level Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                    Genesis Orchestration                     │
│                      (genesis.rs)                            │
│  - Coordinates generation workflow                           │
│  - Manages parallel asset creation                           │
│  - Handles dependency injection (RNG, Time, Logger)         │
└─────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
   ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
   │   Config     │  │ Cryptography │  │  Validation  │
   │ (YAML parse) │  │  (blinding,  │  │   (proofs,   │
   │              │  │   nonces)    │  │    state)    │
   └──────────────┘  └──────────────┘  └──────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │  Serialization   │
                    │ (JSON, Bincode)  │
                    └──────────────────┘
```

### Module Structure

```text
genesis/
├── mod.rs                 # Module documentation and public API
├── genesis.rs             # Core generation logic (2048 lines)
├── genesis_config.rs      # YAML configuration (276 lines)
├── validator.rs           # Validation and verification (1069 lines)
├── serde.rs              # Serialization operations
├── README.md             # Quick start guide
└── *.yaml                # Network configuration files
```

### Data Flow

```text
1. Load YAML Config
   └─> Parse network parameters, asset definitions, output paths

2. Validate Genesis Seed
   └─> M1: Check entropy (≥200 bits)
   └─> Reject known test seeds in production

3. Materialize Policy And Profile Inputs
   ├─> Resolve policy descriptors and action-pool bindings
   ├─> Validate wallet/policy profiles used by live docs and fixtures
   └─> Bind object-family semantics before generation

4. Generate Typed Bootstrap Objects
   ├─> Assets: derive blinding factors, nonces, commitments, and range proofs
   ├─> Rights: derive deterministic authority leaves with zero-value semantics
   ├─> Vouchers: materialize backing-aware voucher config from bootstrap input
   └─> Policies: export deterministic descriptors for runtime hash binding

5. Validate Typed Outputs
   ├─> Batch verify asset commitments and range proofs (M2)
   ├─> Verify voucher backing and runtime-shape materialization contracts
   ├─> Verify right/policy cross-links
   └─> Check state hash and settlement manifest consistency (C2)

6. Export Results
   ├─> Per-asset JSON/Bincode outputs
   ├─> `genesis_rights.json`, `genesis_policies.json`, `genesis_vouchers.json`
   ├─> `genesis_settlement_manifest.json`
   └─> Write atomically with RAII cleanup (L5)
```

---

## 🔧 Core Components

### 1. Genesis Generation (`genesis.rs`)

**Purpose**: Orchestrates the entire genesis generation process.

**Key Functions**:

```rust
/// Main entry point for genesis generation
///
/// Loads configuration, generates assets with parallel processing,
/// validates all cryptographic proofs, and exports results.
///
/// # Arguments
/// * `config_path` - Path to YAML configuration file
/// * `injected_deps` - Optional dependency injection (for testing)
///
/// # Returns
/// * `Ok(())` - Genesis generation successful
/// * `Err(GenesisError)` - If any step fails
///
/// # Examples
/// ```rust
/// use z00z_core::genesis::run_genesis;
///
/// // Generate mainnet genesis
/// run_genesis("configs/devnet_genesis_config.yaml", None)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn run_genesis(
    config_path: &str,
    injected_deps: Option<InjectedDependencies>,
) -> Result<(), GenesisError>
```

**Key Types**:

```rust
/// Complete genesis state containing all generated assets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisAssets {
    pub network_id: u32,
    pub network_type: String,
    pub genesis_seed: [u8; 32],
    pub assets: Vec<Asset>,
    pub genesis_state_hash: [u8; 32],  // C2: Integrity hash
    pub total_assets: usize,
    pub timestamp: u64,
}

/// Network type for cryptographic domain separation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChainType {
    Devnet,   // Development network
    Testnet,  // Test network
    Mainnet,  // Production network
}
```

**Cryptographic Domain Separation**:

Each network uses distinct hash domains to prevent cross-network replay attacks:

```rust
// Blinding factor domains (network-specific)
hash_domain!(GenesisBlindingDomainDevnet, "z00z_core/genesis/blinding/devnet", 1);
hash_domain!(GenesisBlindingDomainTestnet, "z00z_core/genesis/blinding/testnet", 1);
hash_domain!(GenesisBlindingDomainMainnet, "z00z_core/genesis/blinding/mainnet", 1);

// RNG seed domains (network-specific)
hash_domain!(GenesisRngSeedDomainDevnet, "z00z_core/genesis/rng_seed/devnet", 1);
hash_domain!(GenesisRngSeedDomainTestnet, "z00z_core/genesis/rng_seed/testnet", 1);
hash_domain!(GenesisRngSeedDomainMainnet, "z00z_core/genesis/rng_seed/mainnet", 1);

// Asset ID domains (network-specific)
hash_domain!(GenesisAssetIdDomainDevnet, "z00z_core/genesis/asset_id/devnet", 1);
hash_domain!(GenesisAssetIdDomainTestnet, "z00z_core/genesis/asset_id/testnet", 1);
hash_domain!(GenesisAssetIdDomainMainnet, "z00z_core/genesis/asset_id/mainnet", 1);
```

**Parallel Generation**:

```rust
/// Generate assets in parallel using Rayon
///
/// Two-level parallelism:
/// 1. Level 1: Parallel across asset definitions
/// 2. Level 2: Parallel across serial IDs within each definition
///
/// Performance: ~1000 assets/sec on 16-core system
fn generate_assets_parallel(
    config: &GenesisConfig,
    network_type: ChainType,
) -> Result<Vec<Asset>, GenesisError>
```

---

### 2. Configuration (`genesis_config.rs`)

**Purpose**: Parse and validate YAML configuration files.

**Key Types**:

```rust
/// Top-level genesis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisConfig {
    pub chain: ChainConfig,
    pub assets: Vec<AssetConfigEntry>,
    pub rights: Vec<RightsConfigEntry>,
    pub policies: Vec<PolicyConfigEntryV1>,
    pub vouchers: Vec<VoucherBootstrapEntryV1>,
    pub wallet_profiles: Vec<WalletProfileConfig>,
    pub policy_profiles: Vec<PolicyProfileConfig>,
    pub outputs: OutputsConfig,
    pub performance: PerformanceConfig,
}

/// Network identification and cryptographic parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub id: u32,
    pub chain_type: String,
    pub name: String,
    pub magic_bytes: [u8; 4],
    pub domains: DomainsConfig,
}

/// Genesis seed configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainsConfig {
    pub genesis_seed: [u8; 32],
}

/// Asset bootstrap entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetConfigEntry {
    pub id: String,
    pub class: AssetClass,
    pub name: String,
    pub symbol: String,
    pub policy: PolicyConfig,
    pub metadata: Option<BTreeMap<String, String>>,
}
```

**Configuration Loading**:

```rust
/// Load genesis configuration from YAML file
///
/// Uses z00z_utils::io for consistent I/O operations.
/// Validates schema and performs basic sanity checks.
///
/// # Examples
/// ```rust
/// use z00z_core::genesis::genesis_config::load_genesis_config;
///
/// let config = load_genesis_config("configs/devnet_genesis_config.yaml")?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn load_genesis_config(path: &str) -> Result<GenesisConfig, GenesisError>
```

---

### 3. Validation (`validator.rs`)

**Purpose**: Comprehensive validation for genesis assets and state.

**Key Functions**:

```rust
/// Validate genesis seed entropy (M1 Security Enhancement)
///
/// Requirements:
/// - Minimum 200 bits entropy for production
/// - Rejects known test patterns ([42; 32], all-zeros, all-ones)
/// - Statistical tests for randomness
///
/// # Security
/// Using weak seeds can compromise entire blockchain security.
///
/// # Examples
/// ```rust
/// use z00z_core::genesis::validator::validate_genesis_seed;
///
/// let seed = [0x42; 32];  // Test seed
/// validate_genesis_seed(&seed, "devnet")?;  // OK for devnet
///
/// // validate_genesis_seed(&seed, "mainnet")?;  // ERROR: Test seed not allowed
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn validate_genesis_seed(
    seed: &[u8; 32],
    network: &str,
) -> Result<(), GenesisError>

/// Batch verify all genesis asset commitments (M2 Security Enhancement)
///
/// Uses O(log n) batch verification instead of O(n) sequential:
/// - 100 Assets: ~10ms (vs 100ms sequential) - 10× faster
/// - 1000 Assets: ~50ms (vs 5000ms sequential) - 100× faster
///
/// # Examples
/// ```rust
/// use z00z_core::genesis::validator::verify_genesis_assets;
///
/// let assets = generate_assets(...);
/// verify_genesis_assets(&assets)?;  // Batch verification
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn verify_genesis_assets(
    assets: &[Asset],
) -> Result<(), GenesisError>

/// Compute genesis state hash (C2 Security Enhancement)
///
/// Deterministic hash over all asset commitments for state verification.
/// Ensures all nodes arrive at identical genesis state.
///
/// # Examples
/// ```rust
/// use z00z_core::genesis::validator::compute_genesis_state_hash;
///
/// let hash = compute_genesis_state_hash(&assets, network_id);
/// // Store in GenesisAssets for consensus verification
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn compute_genesis_state_hash(
    assets: &[Asset],
    network_id: u32,
) -> [u8; 32]
```

**Error Types**:

```rust
/// Comprehensive genesis error types
#[derive(Debug, Error)]
pub enum GenesisError {
    #[error("config load failed: {0}")]
    ConfigLoadFailed(String),

    #[error("insecure genesis seed: {0}")]
    InsecureGenesisSeed(String),

    #[error("low entropy seed: {entropy} bits (minimum: {minimum} bits)")]
    LowEntropySeed { entropy: f64, minimum: f64 },

    #[error("test seed not allowed in production")]
    TestSeedInProduction,

    #[error("blinding derivation failed for asset {asset_id:?} serial {serial_id}: {error}")]
    BlindingDerivationFailed {
        asset_id: [u8; 32],
        serial_id: u32,
        error: String,
    },

    #[error("proof verification failed for asset {asset_id:?} serial {serial_id}: {error}")]
    ProofVerificationFailed {
        asset_id: [u8; 32],
        serial_id: u32,
        error: String,
    },

    #[error("genesis state mismatch for {network}: expected {expected:?}, computed {computed:?}")]
    GenesisStateMismatch {
        expected: [u8; 32],
        computed: [u8; 32],
        network: String,
    },

    // ... additional error variants
}
```

---

### 4. Serialization (`serde.rs`)

**Purpose**: Export genesis assets to multiple formats.

**Key Functions**:

```rust
/// Export genesis assets to JSON and Bincode formats
///
/// Atomic writes with RAII cleanup (L5 security enhancement):
/// - Writes to temporary file first
/// - Renames on success (atomic operation)
/// - Cleans up on error
///
/// # Examples
/// ```rust
/// use z00z_core::genesis::serde::export_genesis_assets;
///
/// export_genesis_assets(&genesis_assets, "outputs/genesis/")?;
/// // Creates: genesis_<SYMBOL>.json, genesis_<SYMBOL>.bin
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn export_genesis_assets(
    genesis: &GenesisAssets,
    output_dir: &str,
) -> Result<(), GenesisError>
```

---

## 📚 API Reference

### Public API

The genesis module exposes the following public API through `mod.rs`:

```rust
// Core generation
pub use genesis::{
    run_genesis,                    // Main entry point
    GenesisAssets,                  // Complete genesis state
    ChainType,                    // Network type enum
    derive_genesis_blinding,        // Blinding factor derivation
    derive_deterministic_rng_seed,  // RNG seed derivation
};

// Configuration
pub use genesis_config::{
    load_genesis_config,            // Load YAML config
    GenesisConfig,                  // Top-level config
    AssetConfigEntry,               // Asset definition
    PolicyConfig,                   // Asset policy
};

// Validation
pub use validator::{
    validate_genesis_seed,          // M1: Seed validation
    verify_genesis_assets,          // M2: Batch verification
    compute_genesis_state_hash,     // C2: State hash
    GenesisError,                   // Error type
};

// Serialization
pub use serde::{
    export_genesis_assets,          // Multi-format export
};
```

### Common Usage Patterns

#### Pattern 1: Standard Genesis Generation

```rust
use z00z_core::genesis::run_genesis;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load config, generate, verify, export
    run_genesis("configs/devnet_genesis_config.yaml", None)?;

    println!("Genesis generation complete!");
    println!("Output: outputs/genesis/genesis_<SYMBOL>.json");

    Ok(())
}
```

#### Pattern 2: Custom Dependency Injection (Testing)

```rust
use z00z_core::genesis::{run_genesis, InjectedDependencies};
use z00z_utils::time::MockTimeProvider;
use z00z_utils::rng::MockRngProvider;

fn test_genesis_with_mocks() -> Result<(), Box<dyn std::error::Error>> {
    let deps = InjectedDependencies {
        time_provider: Box::new(MockTimeProvider::new(SystemTime::now())),
        rng_provider: Box::new(MockRngProvider::with_seed(42)),
        logger: Box::new(NoopLogger),
        metrics: Box::new(NoopMetrics),
    };

    run_genesis("test_config.yaml", Some(deps))?;

    Ok(())
}
```

#### Pattern 3: Manual Asset Generation

```rust
use z00z_core::genesis::{
    derive_genesis_blinding,
    derive_deterministic_rng_seed,
    ChainType,
};
use z00z_core::assets::{Asset, AssetDefinition};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

fn generate_single_asset(
    definition: Arc<AssetDefinition>,
    genesis_seed: &[u8; 32],
    serial_id: u32,
) -> Result<Asset, Box<dyn std::error::Error>> {
    // Derive blinding factor
    let blinding = derive_genesis_blinding(
        genesis_seed,
        &definition.id,
        serial_id,
        ChainType::Mainnet,
    )?;

    // Derive nonce
    let nonce = derive_nonce(genesis_seed, serial_id as u64, 0, &[0u8; 32]);

    // Create deterministic RNG
    let rng_seed = derive_deterministic_rng_seed(
        genesis_seed,
        &definition.id,
        serial_id,
        ChainType::Mainnet,
    );
    let mut rng = ChaCha20Rng::from_seed(rng_seed);

    // Create asset with range proof
    let asset = Asset::new(
        definition,
        serial_id,
        definition.nominal,
        &blinding,
        nonce,
        &mut rng,
    )?;

    Ok(asset)
}
```

---

## 🔒 Security Model

### Security Enhancements (v3.8)

#### M1: Genesis Seed Validation

**Purpose**: Ensure genesis seed has sufficient entropy to prevent attacks.

**Requirements**:
- ✅ Minimum 200 bits entropy for production networks
- ✅ Reject known test seeds ([42; 32], all-zeros, all-ones)
- ✅ Statistical randomness tests
- ✅ Network-specific enforcement (stricter for mainnet)

**Implementation**:
```rust
pub fn validate_genesis_seed(seed: &[u8; 32], network: &str) -> Result<(), GenesisError> {
    // 1. Check for known weak patterns
    if is_weak_pattern(seed) {
        return Err(GenesisError::InsecureGenesisSeed(...));
    }

    // 2. Calculate entropy
    let entropy = calculate_entropy(seed);
    if entropy < 200.0 && is_production(network) {
        return Err(GenesisError::LowEntropySeed { entropy, minimum: 200.0 });
    }

    // 3. Reject test seeds in production
    if is_test_seed(seed) && is_production(network) {
        return Err(GenesisError::TestSeedInProduction);
    }

    Ok(())
}
```

#### M2: Batch Proof Verification

**Purpose**: Efficient verification of all range proofs in O(log n) time.

**Performance**:
| Assets | Sequential | Batch | Speedup |
|--------|-----------|-------|---------|
| 100    | 100ms     | 10ms  | 10×     |
| 1000   | 5000ms    | 50ms  | 100×    |
| 50000  | 50s       | 5s    | 10×     |

**Implementation**:
```rust
pub fn verify_genesis_assets(assets: &[Asset]) -> Result<(), GenesisError> {
    // Batch verify all range proofs at once
    validate_genesis_commitments_batch(assets)?;

    // Additional validations
    verify_commitment_homomorphism(assets)?;
    check_no_duplicate_commitments(assets)?;

    Ok(())
}
```

#### C2: Genesis State Integrity Hash

**Purpose**: Deterministic hash over all assets for consensus verification.

**Properties**:
- ✅ Collision-resistant (Blake2b-256)
- ✅ Deterministic (same assets → same hash)
- ✅ Includes network ID (prevents cross-network confusion)

**Implementation**:
```rust
pub fn compute_genesis_state_hash(assets: &[Asset], network_id: u32) -> [u8; 32] {
    let mut hasher = Blake2b::<U32>::new();

    // Hash network ID
    hasher.update(&network_id.to_le_bytes());

    // Hash all commitments in deterministic order
    for asset in assets {
        hasher.update(asset.commitment.as_bytes());
    }

    hasher.finalize().into()
}
```

#### L4: Resource Limits (DoS Protection)

**Purpose**: Prevent resource exhaustion attacks during generation.

**Limits**:
- ✅ Maximum 1,000,000 assets per genesis
- ✅ Maximum 100 MB config file size
- ✅ Timeout: 10 minutes for generation
- ✅ Memory: Bounded by parallel chunk size

#### L5: Atomic File Writes (RAII Cleanup)

**Purpose**: Ensure file writes succeed completely or not at all.

**Mechanism**:
```rust
pub fn export_genesis_assets(genesis: &GenesisAssets, dir: &str) -> Result<(), GenesisError> {
    // 1. Write to temporary file
    let temp_path = format!("{}.tmp", final_path);
    write_file(&temp_path, &data)?;

    // 2. Atomic rename (POSIX guarantee)
    rename_file(&temp_path, &final_path)?;

    // 3. RAII cleanup on error (temp file deleted)
    Ok(())
}
```

### Cryptographic Guarantees

1. **Domain Separation**: Different networks use distinct hash domains
2. **Determinism**: Same seed → identical outputs (bit-for-bit)
3. **Confidentiality**: Asset values hidden by Pedersen commitments
4. **Integrity**: Range proofs guarantee values in valid range [0, 2^64)
5. **Non-malleability**: Commitments cannot be modified without detection

---

## 💡 Usage Examples

### Example 1: Generate Mainnet Genesis

```rust
use z00z_core::genesis::run_genesis;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating mainnet genesis...");

    // Load mainnet config and generate
    run_genesis("configs/devnet_genesis_config.yaml", None)?;

    println!("✅ Mainnet genesis complete!");
    println!("   Output: outputs/genesis/genesis_<SYMBOL>.json");
    println!("   Binary: outputs/genesis/genesis_<SYMBOL>.bin");

    Ok(())
}
```

### Example 2: Verify Genesis Assets

```rust
use z00z_core::genesis::GenesisAssets;
use z00z_core::genesis::validator::{verify_genesis_assets, compute_genesis_state_hash};
use z00z_utils::io::load_json;

fn verify_genesis_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Load genesis from JSON
    let genesis: GenesisAssets = load_json(path)?;

    println!("Verifying {} assets...", genesis.assets.len());

    // Verify all proofs (batch verification)
    verify_genesis_assets(&genesis.assets)?;

    // Verify state hash
    let computed_hash = compute_genesis_state_hash(&genesis.assets, genesis.network_id);
    if computed_hash != genesis.genesis_state_hash {
        return Err("State hash mismatch!".into());
    }

    println!("✅ Genesis verification passed!");

    Ok(())
}
```

### Example 3: Custom Network Configuration

Start from a copy of the live devnet manifest and keep the split-manifest
shape intact:

```yaml
# configs/custom_genesis_config.yaml
version: 1
manifest_refs:
  assets: custom_assets_catalog.yaml
  rights: custom_rights_config.yaml
  policies: custom_policies_config.yaml
  vouchers: custom_vouchers_config.yaml
chain:
  id: 99
  type: custom
  chain: custom-net-1
  magic_bytes: [0xCA, 0xFE, 0xBA, 0xBE]
outputs:
  assets_export_path: crates/z00z_core/outputs/custom_genesis/
  snapshot_export_path: crates/z00z_core/outputs/custom_genesis/
  logging_path: crates/z00z_core/outputs/log/
performance:
  num_threads: auto
```

```rust
// Generate custom network genesis
run_genesis("configs/custom_genesis_config.yaml", None)?;
```

---

## ⚡ Performance Characteristics

### Benchmark Results (16-core System)

#### Single Asset Generation
- **Time**: 6.28 ms/asset
- **Components**:
  - Blinding derivation: 0.1 ms
  - Nonce generation: 0.05 ms
  - RNG setup: 0.03 ms
  - Asset + range proof: 6.1 ms (98% of time)

#### Parallel Generation (1000 Assets)
- **Time**: 1.015 seconds
- **Throughput**: ~985 assets/sec
- **Speedup**: 10× vs sequential
- **CPU Utilization**: ~95% (excellent parallelism)

#### Batch Verification (100 Proofs)
- **Time**: 109 ms
- **Per-proof**: 1.09 ms
- **Speedup**: 10× vs individual verification (100ms → 10ms)

### Scalability

| Assets | Generation | Verification | Total | Throughput |
|--------|-----------|--------------|-------|-----------|
| 10     | 0.06s     | 0.01s        | 0.07s | 143/sec   |
| 100    | 0.10s     | 0.11s        | 0.21s | 476/sec   |
| 1000   | 1.02s     | 1.09s        | 2.11s | 474/sec   |
| 2200   | 2.10s     | 2.20s        | 4.30s | 511/sec   |
| 10000  | 10.5s     | 11.0s        | 21.5s | 465/sec   |

### Memory Usage

- **Per Asset**: ~2 KB (commitment + proof + metadata)
- **1000 Assets**: ~2 MB
- **50000 Assets**: ~100 MB
- **Parallel Overhead**: ~50 MB (thread pool + buffers)

### Optimization Tips

1. **Use Release Mode**: `cargo run --release` (10× faster than debug)
2. **Increase Genesis Pool**: Set `performance.num_threads` in the genesis manifest
3. **Reduce Serial Count**: For testing, use smaller `serials` value
4. **SSD Storage**: Use SSD for faster file I/O (10× faster writes)

---

## 🧪 Testing Strategy

### Test Organization

```text
tests/
├── fixtures/                              # Generated helper artifacts only
├── test_genesis.rs                        # Canonical genesis test entry point
├── test_genesis_mod.rs                    # Root-level genesis module map
├── test_genesis_manifest.rs               # Lane selection and partial-manifest rules
├── test_genesis_manifest_refs.rs          # Canonical root-manifest and ref guards
├── test_genesis_manifest_goldens.rs       # Canonical golden and byte-stability checks
├── test_genesis_validation.rs             # Validation and reject-path coverage
├── test_genesis_rights.rs                 # Rights materialization coverage
├── test_genesis_vouchers.rs               # Voucher materialization coverage
└── test_live_guardrails.rs                # Documentation and naming guards
```

All owned genesis integration files now live directly under
`crates/z00z_core/tests/`. `tests/fixtures/` is the only allowed owned nested
support directory.

### Test Coverage

Coverage is split across the flat `genesis_tests` entry target, focused
manifest-reference and manifest-golden targets, and `test_live_guardrails.rs`
for documentation and naming contracts.

**By Category**:
- Reproducibility: 3 tests
- Validation: 7 tests
- Configuration: 9 tests
- Integration: 8 tests
- Cryptographic Security: 3 tests
- Security Validation: 8 tests (property-based)
- Determinism: 2 tests
- Multi-asset: 4 tests
- Range Proofs: 4 tests
- Commitment Homomorphism: 4 tests
- Network Isolation: 5 tests
- Batch Verification: 13 tests

### Key Test Scenarios

#### 1. Determinism Tests

```rust
#[test]
fn test_reproducible_genesis_same_seed() {
    let config = load_test_config();

    // Generate twice with same seed
    let genesis1 = generate_genesis(&config)?;
    let genesis2 = generate_genesis(&config)?;

    // Must be identical (byte-for-byte)
    assert_eq!(genesis1.assets, genesis2.assets);
    assert_eq!(genesis1.genesis_state_hash, genesis2.genesis_state_hash);
}
```

#### 2. Security Validation Tests

```rust
#[test]
fn test_genesis_seed_entropy_requirement() {
    let weak_seed = [0u8; 32];  // All zeros
    let result = validate_genesis_seed(&weak_seed, "mainnet");

    assert!(result.is_err());
    assert!(matches!(result, Err(GenesisError::InsecureGenesisSeed(_))));
}

#[test]
fn test_batch_verification_performance() {
    let assets = generate_test_assets(100);

    let start = Instant::now();
    verify_genesis_assets(&assets)?;
    let duration = start.elapsed();

    // Must complete in < 20ms for 100 assets
    assert!(duration < Duration::from_millis(20));
}
```

#### 3. Network Isolation Tests

```rust
#[test]
fn test_cross_network_isolation() {
    let seed = [0x42; 32];
    let asset_id = [0x01; 32];

    // Generate blinding for different networks
    let devnet_blinding = derive_genesis_blinding(&seed, &asset_id, 0, ChainType::Devnet)?;
    let mainnet_blinding = derive_genesis_blinding(&seed, &asset_id, 0, ChainType::Mainnet)?;

    // Must be different (domain separation)
    assert_ne!(devnet_blinding, mainnet_blinding);
}
```

### Running Tests

```bash
# Run all genesis tests
cargo test --package z00z_core genesis

# Run specific test module
cargo test --package z00z_core genesis::reproducibility

# Run with output
cargo test --package z00z_core genesis -- --nocapture

# Run benchmarks
cargo bench --bench genesis_bench
```

---

## ⚙️ Configuration Reference

### YAML Configuration Format

```yaml
# Network Configuration
network:
  id: 1                          # Network identifier (1=mainnet, 2=testnet, 99=devnet)
  type: mainnet                  # Network type: "mainnet", "testnet", "devnet"
  magic_bytes: [0x5A, 0x30, 0x30, 0x5A]  # 4-byte network magic (Z00Z)
  domains:
    genesis_seed: [...]          # 32-byte genesis seed (hex or decimal array)

# Asset Definitions
assets:
  - id: Z00Z                     # Unique asset identifier
    class: Coin                  # Asset class: Coin, Token, Nft, Void
    name: "Z00Z Coin"            # Human-readable name
    symbol: Z00Z                 # Trading symbol
    policy:
      serials: 2200              # Number of serial instances to generate
      nominal: 1000000           # Value per instance (in smallest unit)
      min_reveal: 1              # Minimum reveal threshold
      max_reveal: 1000000        # Maximum reveal threshold
      fungibility: 1             # Fungibility level (1=fully fungible)
    metadata:                    # Optional metadata (key-value pairs)
      description: "Native Z00Z blockchain coin"
      website: "https://z00z.network"
      supply: "2,200,000,000"

  - id: ETH
    class: Token
    name: "Ethereum"
    symbol: ETH
    policy:
      serials: 1000
      nominal: 1000000000000000000  # 1 ETH in wei
      min_reveal: 1
      max_reveal: 1000000000000000000
      fungibility: 1

# Output Configuration
outputs:
  assets_export_path: "outputs/genesis/"  # Directory for output files
```

### Configuration Validation Rules

1. **Network ID**: Must be > 0
2. **Network Type**: Must be "mainnet", "testnet", or "devnet"
3. **Magic Bytes**: Must be exactly 4 bytes
4. **Genesis Seed**: Must be exactly 32 bytes with sufficient entropy (M1)
5. **Asset ID**: Must be unique within genesis
6. **Asset Class**: Must be valid enum value
7. **Serials**: Must be > 0 and < 1,000,000 (L4)
8. **Nominal**: Must be > 0 and < 2^64
9. **Output Path**: Must be valid filesystem path

### Pre-configured Networks

#### Devnet Core Fixture
- **Network ID**: 3
- **Purpose**: Minimal Phase 059 core manifest, policy, rights, and voucher validation
- **Seed Requirements**: Dedicated deterministic devnet fixture seed
- **Config**: `configs/devnet_genesis_config.yaml`

#### Devnet Simulator Fixture
- **Network ID**: 3
- **Purpose**: Compact multi-asset Phase 059 simulator and scenario fixture
- **Seed Requirements**: Dedicated deterministic devnet simulator seed
- **Config**: `configs/devnet_genesis_config.yaml`

---

## 🔍 Troubleshooting

### Common Errors and Solutions

#### Error: "insecure genesis seed"

**Cause**: Genesis seed has insufficient entropy or matches known weak pattern.

**Solution**:
```bash
# Generate secure random seed (Linux)
head -c 32 /dev/urandom | xxd -p -c 32

# Generate secure random seed (macOS)
openssl rand -hex 32

# Use in config
genesis_seed: [0xAB, 0xCD, 0xEF, ...]  # Paste generated bytes
```

#### Error: "test seed not allowed in production"

**Cause**: Using test seed ([42; 32]) with mainnet configuration.

**Solution**: Use secure random seed for mainnet (see above).

#### Error: "proof verification failed"

**Cause**: Range proof invalid (likely implementation bug or corrupted data).

**Solution**:
1. Check asset amount is in valid range [0, 2^64)
2. Verify blinding factor derivation is deterministic
3. Run with `--verbose` to see detailed error
4. Check for hardware issues (memory corruption)

#### Error: "genesis state mismatch"

**Cause**: Computed state hash doesn't match expected hash.

**Solution**:
1. Verify all nodes use identical configuration
2. Check genesis seed is exactly the same (byte-for-byte)
3. Ensure same z00z_core version on all nodes
4. Regenerate genesis and distribute to all nodes

#### Error: "config load failed"

**Cause**: YAML parsing error or file not found.

**Solution**:
```bash
# Verify YAML syntax
yamllint configs/devnet_genesis_config.yaml

# Check file exists
ls -l configs/devnet_genesis_config.yaml

# Validate against schema
cargo run --bin validate_config -- configs/devnet_genesis_config.yaml
```

#### Performance Issue: "Generation too slow"

**Cause**: Running in debug mode or single-threaded.

**Solution**:
```bash
# Use release mode (10× faster)
cargo run --release --bin genesis_cli -- --config configs/devnet_genesis_config.yaml

# Increase genesis pool size through the canonical manifest knob
# performance:
#   num_threads: 16
cargo run --release --bin genesis_cli -- --config configs/devnet_genesis_config.yaml

# Reduce serial count for testing
# Edit config: serials: 100 (instead of 2200)
```

### Debugging Tips

1. **Enable Verbose Logging**:
   ```bash
   RUST_LOG=debug cargo run --bin genesis_cli -- --config config.yaml --verbose
   ```

2. **Check Resource Usage**:
   ```bash
   # Monitor CPU and memory
   htop

   # Check disk space
   df -h outputs/genesis/
   ```

3. **Validate Configuration**:
   ```bash
   # Use schema validation tool
   cargo run --bin validate_config -- configs/devnet_genesis_config.yaml
   ```

4. **Run Tests First**:
   ```bash
   # Verify implementation before generating
   cargo test --package z00z_core genesis
   ```

5. **Compare with Known-Good Genesis**:
   ```bash
   # Compare state hashes
   diff <(jq '.genesis_state_hash' genesis1.json) <(jq '.genesis_state_hash' genesis2.json)
   ```

---

## 📖 Additional Resources

### Related Documentation

- **[Genesis Cryptography Security Review]**: Formal threat model and security proofs
  (`docs/genesis/genesis_spec_crypto_review.md`)
- **[ONE SOURCE OF TRUTH]**: Architecture principle for abstractions
    (`crates/Z00Z_DESIGN_FOUNDATION.md`, Section 1)
- **[Tari Crypto Integration]**: Cryptographic primitives usage
    (`.github/requirements/Tari-Crypto-Integration-Z00Z.md`)
- **[Asset Module Documentation]**: Asset types and operations
  (`crates/z00z_core/docs/ASSETS_DOCUMENTATION.md`)

### Example Files

- **Example target**: `genesis_example`
- **Bench target**: `genesis_bench`
- **Test helper module**: `tests/test_genesis_helpers.rs`

### External References

- [Pedersen Commitments](https://en.wikipedia.org/wiki/Commitment_scheme)
- [Bulletproofs](https://eprint.iacr.org/2017/1066.pdf)
- [Domain Separation](https://en.wikipedia.org/wiki/Domain_separation)
- [BLAKE2](https://www.blake2.net/)

---

## 📝 Version History

### v3.8 (2025-12-10) - Security Hardening Release
- ✅ M1: Genesis seed validation (200-bit entropy)
- ✅ M2: Batch proof verification (O(log n))
- ✅ C2: Genesis state integrity hash
- ✅ L4: Resource limits (DoS protection)
- ✅ L5: Atomic file writes (RAII cleanup)
- 📚 Comprehensive documentation

### v3.7 (2024-12-07)
- Nested parallelism optimization
- Performance improvements (10× speedup)
- 96 tests (27 unit + 69 integration)

### v3.6 (2024-12-01)
- Network-aware cryptographic domains
- Cross-network isolation tests
- Configuration schema validation

### v3.5 (2024-11-15)
- Initial production-ready release
- Basic parallelism support
- YAML configuration

---

**Document Version**: 1.0
**Last Updated**: 2025-12-10
**Maintainers**: Z00Z Core Team
