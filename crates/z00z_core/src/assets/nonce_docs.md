# Nonce Generation - Deterministic nonce derivation for Asset privacy

Provides cryptographically secure nonce generation with:
- Deterministic derivation from wallet seed
- Persistent counter to prevent reuse
- Wallet recovery support
- Thread-safe counter management

## 🎯 Nonce Derivation Strategies

Z00Z supports three nonce derivation approaches for different use cases:

### Strategy 1: Full Derivation (RECOMMENDED for production wallets)

```rust
use z00z_core::assets::nonce::derive_nonce;

let wallet_seed = [42u8; 32]; // From HD wallet
let counter = 1u64; // From NonceCounter
let timestamp = 1701388800000000u64; // Current time in microseconds
let prev_output = [0u8; 32]; // Previous output hash in chain

let nonce = derive_nonce(&wallet_seed, counter, timestamp, &prev_output);
```

**Use when:** Creating transaction outputs in production wallets
**Security:** Requires persistent NonceCounter in database transaction
**Privacy:** Maximum entropy (counter + timestamp + prev_output)
**Features:**
- Wallet recovery support (deterministic from seed)
- Transaction chaining (prev_output links outputs)
- Audit trail (timestamp tracking)

### Strategy 2: Simple Derivation (for testing and lightweight wallets)

```rust
use z00z_core::assets::nonce::derive_nonce_simple;
use z00z_utils::prelude::SystemTimeProvider;

let wallet_seed = [42u8; 32];
let counter = 1u64;
let time = SystemTimeProvider;

let nonce = derive_nonce_simple(&wallet_seed, counter, &time).expect("simple nonce");
```

**Use when:** Testing or lightweight wallet implementations
**Security:** Simpler but still requires persistent counter
**Privacy:** Good (counter provides uniqueness)
**Features:**
- No timestamp dependency (simpler testing)
- No transaction chaining (independent outputs)

### Strategy 3: Minimal Derivation (for genesis and benchmarks)

```rust
use z00z_core::assets::nonce::derive_nonce_minimal;
use z00z_utils::time::SystemTimeProvider;
use rand::rngs::OsRng;

let time_provider = SystemTimeProvider;
let nonce = derive_nonce_minimal(&mut OsRng, &time_provider).expect("minimal nonce");
```

**Use when:**
- Genesis asset generation (deterministic + unique)
- Benchmarks and performance tests
- When wallet seed is not available

**Security:** RNG-based + timestamp for uniqueness
**Privacy:** Good (cryptographically secure RNG)
**Features:**
- No persistent state required
- Timestamp ensures uniqueness

### Strategy 4: Genesis-Specific (for reproducible genesis)

```rust
use z00z_core::assets::nonce::derive_genesis_nonce;

let genesis_seed = [0xABu8; 32]; // From genesis config
let definition_id = [0x01u8; 32]; // Asset definition ID
let serial_id = 100u32; // Serial number

let nonce = derive_genesis_nonce(&genesis_seed, &definition_id, serial_id);
```

**Use when:** Generating deterministic genesis state (ONE-TIME ONLY)
**Security:** Deterministic for reproducibility
**Privacy:** Genesis outputs are semi-public by design
**Features:**
- Reproducible across nodes (deterministic)
- Network-aware domain separation
- No RNG or timestamp dependency

## ⚠️ CRITICAL Security Warnings

### 🚫 NEVER use placeholder nonces in production

```rust,ignore
// ❌ WRONG - breaks privacy!
let nonce = [42u8; 32];
let asset = Asset::new(def, serial, amount, &blinding, nonce, &mut rng)?;
```

**Why dangerous:**
- Reused nonces leak privacy (linkable outputs)
- Violates protocol uniqueness requirement
- Could enable double-spend detection bypass

### 🔒 ALWAYS persist counter before using nonce

```rust,ignore
// ✅ CORRECT
