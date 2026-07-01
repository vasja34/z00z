# Design Foundation for Z00Z Blockchain Projects

**Version:** 1.3.0
**Date:** 2026-04-02
**Status:** Architectural Constitution
**Reference Implementation:** Z00Z Blockchain
**Purpose:** Universal design patterns and architectural principles for Rust blockchain/cryptographic systems

---

## 📋 Document Overview

🎯 **Purpose:** This document establishes foundational design principles, architectural patterns, and best practices for Rust blockchain/cryptographic projects. **All principles are universal** and applicable to any similar project, with Z00Z blockchain serving as a concrete reference implementation.

🔍 **Scope:** Universal design patterns applicable to:

- Core business logic (blockchain, smart contracts, consensus)
- Cryptographic primitives (signatures, commitments, proofs)
- Abstraction layers (I/O, time, RNG, logging)
- Runtime, storage, networking, and wallet modules
- External integrations and tools

✅ **Authority:** This document describes universal principles first, followed by "See also" references to Z00Z implementations as concrete examples.

---

## 🏛️ Core Architectural Principles

### 1. ONE SOURCE OF TRUTH

**Principle:** Each low-level operation MUST have exactly ONE centralized abstraction layer.

**Rationale:**

- Prevents fragmentation and inconsistent implementations
- Preserves consistency across crates and layers
- Enables global behavior changes (logging, metrics, testing)
- Improves testability through centralized abstractions
- Simplifies maintenance and refactoring
- Strengthens security and observability at one boundary

**Architecture:**

```text
┌──────────────────────────────────────────┐
│  Business Logic (z00z_core, z00z_*)      │
│  ❌ NO direct std::fs, serde_*, time::*  │
└────────────────┬─────────────────────────┘
                 ↓ uses abstractions
┌──────────────────────────────────────────┐
│  Abstraction Layer (z00z_utils)          │
│  ✅ io::*, codec::*, time::*, logger::*  │
└────────────────┬─────────────────────────┘
                 ↓ wraps
┌──────────────────────────────────────────┐
│  Standard Library & Dependencies         │
│  std::fs, serde_*, SystemTime, log, etc. │
└──────────────────────────────────────────┘

```

> **Note:** The above diagram shows Z00Z's implementation. Apply the same layering pattern to your project (e.g., `your_project_core` → `your_project_utils` → stdlib).

**Implementation Map (Z00Z Example):**

| Low-Level Operation | ONE SOURCE | Anti-Pattern |
| --------------------- | ------------ | -------------- |
| File I/O | `z00z_utils::io::*` | ❌ `std::fs` |
| Serialization | `z00z_utils::codec::{JsonCodec, YamlCodec}` | ❌ `serde_json`, `serde_yaml` |
| Time/Clock | `z00z_utils::time::TimeProvider` | ❌ `SystemTime::now()` |
| Random Numbers | `z00z_utils::rng::RngProvider` | ❌ `rand::thread_rng()` |
| Logging | `z00z_utils::logger::Logger` | ❌ `log::info!()` |
| Metrics | `z00z_utils::metrics::MetricsSink` | ❌ Direct prometheus calls |
| Configuration | `z00z_utils::config::ConfigSource` | ❌ Direct YAML parsing |

**Correct Usage Patterns:**

#### File I/O

```rust
// ❌ WRONG
use std::fs;
let data = fs::read_to_string("config.yaml")?;
fs::write("output.json", json)?;

// ✅ CORRECT
use z00z_utils::io::{read_to_string, write_file};
let data = read_to_string("config.yaml")?;
write_file("output.json", &json)?;

```

**Available functions:**

- `write_file(path, data)` - Atomic write with temp+rename semantics
- `read_file(path)` - Read bytes from file
- `read_to_string(path)` - Read file as UTF-8 string
- `remove_file(path)` - Delete file through the abstraction layer
- `rename_file(from, to)` - Rename or move a file
- `create_dir_all(path)` - Create directories recursively

#### Serialization And YAML

```rust
// ❌ WRONG
use serde_json;
use serde_yaml::Value;

let json = serde_json::to_string(&data)?;
let policy: Value = config.get("policy")?;

// ✅ CORRECT
use z00z_utils::codec::{Codec, JsonCodec};
use z00z_utils::config::{YamlValue, from_yaml_value};

let codec = JsonCodec;
let json_bytes = codec.serialize(&data)?;
let policy: YamlValue = config.get("policy")?;
let parsed = from_yaml_value(policy)?;

```

**Approved wrappers:**

- `JsonCodec` - Replace direct `serde_json` usage
- `YamlCodec` - Replace direct `serde_yaml` usage where codec-style access fits
- `YamlValue` and `from_yaml_value()` - Replace public exposure of `serde_yaml::Value`
- Pass slice-like values by reference when using codec helpers, for example `codec.serialize(&assets)?`, so the abstraction remains uniform for borrowed collections.

#### Time And Clock

```rust
// ❌ WRONG
use std::time::{SystemTime, UNIX_EPOCH};
let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

// ✅ CORRECT
use z00z_utils::time::{SystemTimeProvider, TimeProvider};

let time_provider = SystemTimeProvider::default();
let now = time_provider.unix_timestamp();

```

**Boundary Rules:**

- Do not mix direct low-level library calls and abstraction-layer calls for the same concern inside one business-logic module.
- Do not expose `serde_*`, raw time, or other low-level dependency types from public APIs when a project abstraction already exists.
- Apply the same abstraction rules to tests and doctests; do not reintroduce direct `serde_json`, ad-hoc YAML parsing, or raw time helpers in test-only code unless the test explicitly verifies compatibility with that low-level dependency.
- `std::fs::File` remains acceptable for streaming-heavy operations such as ZIP handling or large-file pipelines where the wrapper would only add indirection.

**Enforcement (adapt paths to your project):**

```bash
# Verification commands (MUST return 0 matches in business logic crates)
grep -rn "use std::fs" crates/your_core/src/ | grep -v "std::fs::File"
grep -rn "serde_yaml::Value" crates/your_core/src/
grep -rn "serde_json::" crates/your_core/src/
grep -rn "SystemTime::" crates/your_core/src/
grep -rn "rand::thread_rng" crates/your_core/src/
grep -rn "log::" crates/your_core/src/

# Z00Z example:
# grep -rn "use std::fs" crates/z00z_core/src/

```

---

### 2. TRAIT-BASED DEPENDENCY INJECTION

**Principle:** All external dependencies MUST be injected via traits, enabling testability and flexibility.

**Benefits:**

- Production code uses real implementations
- Tests use **real implementations** with deterministic/synthetic input data
- No mocks/stubs needed - inject alternative real implementations instead
- No runtime overhead (monomorphization)
- Clear separation of interface vs. implementation

**Pattern:**

```rust
// Define trait for dependency (abstraction)
pub trait ExternalService {
    fn get_current_value(&self) -> u64;
    fn get_data(&self) -> Data;
}

// Production implementation (uses real external source)
pub struct RealService;
impl ExternalService for RealService {
    fn get_current_value(&self) -> u64 {
        // Calls actual system/network/database
        actual_system_call()
    }
    fn get_data(&self) -> Data {
        fetch_from_real_source()
    }
}

// Test implementation (deterministic, but still real code)
pub struct DeterministicService {
    fixed_value: u64,
    test_data: Data,
}
impl ExternalService for DeterministicService {
    fn get_current_value(&self) -> u64 {
        self.fixed_value  // Real implementation, deterministic data
    }
    fn get_data(&self) -> Data {
        self.test_data.clone()  // Real implementation, controlled data
    }
}

```

**Usage Pattern:**

```rust
// Business logic is generic over trait
pub fn process_entity<S: ExternalService>(
    config: Config,
    service: &S,  // Injected dependency
) -> Entity {
    let value = service.get_current_value();
    let data = service.get_data();
    // ... business logic uses real implementations
    Entity::new(value, data, config)
}

// Production: uses real external service
let entity = process_entity(config, &RealService);

// Testing: uses deterministic service (still real code, no mocks)
let test_service = DeterministicService {
    fixed_value: 42,
    test_data: create_test_data(),
};
let entity = process_entity(config, &test_service);
assert_eq!(entity.value(), 42);

```

**Key Difference from Mocking:**

- ❌ **Mocks/Stubs:** Fake objects that verify calls, don't execute real logic
- ✅ **Alternative Implementations:** Real code with different data sources (deterministic, in-memory, file-based)

**Common Patterns:**

| Dependency Type | Real Implementation | Test Implementation |
| ---------------- | --------------------- | --------------------- |
| Time/Clock | System clock | Fixed time or controllable clock |
| Random Numbers | Crypto RNG | Seeded deterministic RNG |
| Database | PostgreSQL/RocksDB | In-memory BTreeMap or temp DB |
| Network | HTTP client | In-memory request/response |
| File System | OS file system | In-memory virtual FS |
| Logger | File/stdout logger | Collecting logger (real logging to buffer) |

#### Example: Common Abstractions

```rust
// Time: not a mock, real implementations with different sources
pub trait TimeSource {
    fn now(&self) -> Timestamp;
}

// RNG: not a mock, real RNG with different seeds
pub trait RandomSource {
    fn next_bytes(&mut self, buf: &mut [u8]);
}

// Storage: not a mock, real storage with different backends
pub trait Storage {
    fn read(&self, key: &[u8]) -> Option<Vec<u8>>;
    fn write(&mut self, key: &[u8], value: &[u8]);
}

```

**See also:**

- `z00z_utils::time::TimeProvider` - Time abstraction example
- `z00z_utils::rng::RngProvider` - RNG abstraction example
- `z00z_utils::logger::Logger` - Logging abstraction example

---

### 3. ZERO-OVERHEAD ABSTRACTIONS

**Principle:** Abstractions MUST compile to optimal code with zero runtime cost.

**Techniques:**

**A. Noop Implementations for Testing:**

```rust
/// Logger that compiles to nothing in tests
pub struct NoopLogger;

impl Logger for NoopLogger {
    #[inline(always)]
    fn info(&self, _msg: &str) {}

    #[inline(always)]
    fn error(&self, _msg: &str) {}
}

```

**Result:** Compiler eliminates all logging calls (dead code elimination).

**B. Monomorphization:**

```rust
// Generic function - specialized per concrete type
pub fn process<T: TimeProvider>(provider: &T) -> u64 {
    provider.unix_timestamp()
}

// Compiler generates TWO specialized versions:
// 1. process_SystemTimeProvider() - calls real clock
// 2. process_MockTimeProvider() - reads fixed value
// No virtual dispatch, no overhead

```

**C. Inline + Const Propagation:**

```rust
#[inline]
pub fn validate_range(value: u64) -> bool {
    value >= MIN_VALUE && value <= MAX_VALUE
}

// Inlined at call site, constants propagated
// May optimize to single comparison

```

**Evidence:** `z00z_utils` migration achieved 100% backward compatibility with NoopLogger/NoopMetrics eliminating conditional compilation (`#[cfg(test)]`).

---

### 4. DOMAIN SEPARATION (Cryptographic Security)

**Principle:** All cryptographic operations MUST use domain-separated hashing to prevent cross-context attacks.

**Attack Vector:** Same key/nonce reused in different contexts (entity creation, signature generation, key derivation) allows cryptographic attacks (key recovery, signature forgery).

**Solution:** Domain-specific hash functions with unique identifiers per operation context.

**Universal Pattern:**

```rust
use crypto_lib::{hash_domain, DomainHasher};

// Define domain at module level
hash_domain!(
    InitializationDomainProduction,
    "app/initialization/blinding/production",  // Unique domain string
    1  // Version number
);

// Use in function
pub fn derive_secret(seed: &[u8; 32], index: u64) -> SecretKey {
    let hasher = InitializationDomainProduction::hasher();
    hasher
        .chain(seed)
        .chain(&index.to_le_bytes())
        .finalize_into_scalar()
}

```

**Naming Convention:**

- **Format:** `{Operation}Domain{Environment}` (e.g., `InitializationDomainProduction`)
- **String:** `"app/{module}/{operation}/{environment}"` (hierarchical namespace)
- **Version:** Increment when domain purpose changes

**Environment Separation:**

```rust
// Production domain
hash_domain!(
    SignatureDomainProduction,
    "app/signatures/production",
    1
);

// Testing domain (DIFFERENT output for same inputs)
hash_domain!(
    SignatureDomainTesting,
    "app/signatures/testing",
    1
);

```

**Result:** Cryptographic outputs from testing environment cannot be replayed in production (different domain separators produce distinct values).

**Verification:**

```bash
# All hash_domain! usages must follow convention
grep -rn "hash_domain!" src/ --include="*.rs"

```

**Domain Categories (examples):**

- `MetadataHashDomain` - Data commitments
- `OwnershipSignatureDomain` - Ownership proofs
- `InitializationDomain{Env}` - Deterministic initialization values
- `RandomSeedDomain{Env}` - Deterministic RNG seeding
- `IdentifierDerivationDomain{Env}` - Unique ID generation

**See also:** `crates/z00z_crypto/src/hash_domain.rs` - Domain separation implementation for Z00Z cryptographic operations.

---

### 5. VENDOR CODE ISOLATION

**Principle:** External vendored code MUST be isolated in READ-ONLY directories with clear boundaries.

**Policy - Vendor Directory Pattern:**

- ✅ **Allowed:** Compile, link, use vendored libraries
- ✅ **Allowed:** Export functionality via project's public API
- ❌ **FORBIDDEN:** Modify source files in vendor subtree
- ❌ **FORBIDDEN:** Add files to vendor directory

**Boundary Rules:**

- Stable root facades MUST NOT present vendor concrete types as ordinary project-owned contracts.
- If passthrough access is unavoidable, confine it to an explicitly named `vendor`, `expert`, or similarly narrow namespace so downstream code opts into vendor lock-in consciously.
- Prefer project-owned wrappers, aliases, or thin adapter types for the default public surface, and document every remaining vendor passthrough as compatibility-sensitive.

**Rationale:**

- Simplifies upstream updates (clean git subtree)
- Clear ownership boundary (external vs. internal code)
- Prevents accidental modifications
- Makes fork points explicit

**Integration Pattern:**

```rust
// project_crypto/src/lib.rs
// Re-export vendor components with project-specific wrappers

// Direct re-exports (stable API)
pub use vendor_crypto::keys::{
    SecretKey,
    PublicKey,
};

// Wrapped re-exports (add project-specific behavior)
pub mod commitment {
    use vendor_crypto::commitment::CommitmentFactory as VendorFactory;

    pub type CommitmentFactory = VendorFactory;

    pub fn default_factory() -> CommitmentFactory {
        VendorFactory::default()
    }
}

```

**Enforcement:**

```bash
# Verify no modifications in vendor directory
cd lib/vendor_crypto/
git status  # Must be clean (or managed via git subtree/submodule)

```

**Extension:** If vendor library lacks needed functionality, implement in project's own source (NOT in vendor directory), then contribute upstream.

**See also:** `crates/z00z_crypto/tari/` - Example of vendored Tari cryptography library with clean integration boundaries.

---

### 6. ENGLISH-ONLY POLICY

**Principle:** All technical artifacts MUST be written exclusively in English.

**Scope:**

- ✅ Source code (Rust, scripts)
- ✅ Comments and documentation
- ✅ Commit messages and PR descriptions
- ✅ Error messages and log output
- ✅ Configuration files
- ✅ Technical specifications
- ✅ API documentation

**Rationale:**

- International team accessibility
- Standard tooling compatibility (linters, analyzers)
- Long-term maintainability
- Open-source contribution readiness

**Enforcement:** Code review MUST reject non-English technical content.

**Exception:** User-facing content (UI translations, marketing) may use localization.

---

### 7. PARALLELISM & CONCURRENCY

**Principle:** Use appropriate concurrency model for the workload type - data parallelism for CPU-bound tasks, async/await for I/O-bound tasks.

**Problem:** Incorrect concurrency model wastes resources (thread pool for I/O waits, async overhead for CPU work).

**Solution:** Choose model based on bottleneck type.

**Universal Pattern:**

| Workload Type | Model | Library | Use Case |
| --------------- | ------- | --------- | ---------- |
| CPU-bound (compute) | Data parallelism | `rayon` | Cryptographic operations, batch processing, validation |
| I/O-bound (network/disk) | Async/await | `tokio` | Network requests, database queries, file I/O |
| Mixed | Hybrid | `rayon` + `tokio` | Async I/O coordinator spawning parallel CPU work |

**CPU-Bound Pattern (Data Parallelism with rayon):**

```rust
use rayon::prelude::*;

// Parallel batch processing
pub fn process_batch(items: &[Item]) -> Result<Vec<Output>> {
    items.par_iter()
        .map(|item| process_expensive_computation(item))
        .collect()
}

// Parallel validation (fail-fast on error)
pub fn validate_batch(entities: &[Entity]) -> Result<()> {
    entities.par_iter()
        .try_for_each(|entity| {
            // Expensive crypto verification runs in parallel
            verify_signature(&entity.signature)?;
            verify_proof(&entity.proof)?;
            Ok(())
        })
}

// Parallel aggregation
pub fn compute_merkle_root(leaves: &[Hash]) -> Hash {
    leaves.par_chunks(2)
        .map(|pair| hash_pair(pair[0], pair.get(1)))
        .reduce(|| Hash::default(), |a, b| hash_pair(a, b))
}

```

**I/O-Bound Pattern (Async with tokio):**

```rust
use tokio::task;

// Async I/O operations
pub async fn fetch_data_from_network(urls: &[Url]) -> Result<Vec<Data>> {
    let futures = urls.iter()
        .map(|url| async move {
            let client = HttpClient::new();
            client.get(url).await
        });

    // Concurrent I/O (not parallel - single thread can handle many)
    futures::future::try_join_all(futures).await
}

// Async database queries
pub async fn load_entities(ids: &[EntityId], db: &Database) -> Result<Vec<Entity>> {
    let queries = ids.iter()
        .map(|id| db.query("SELECT * FROM entities WHERE id = $1", id));

    futures::future::try_join_all(queries).await
}

```

**Hybrid Pattern (Async coordinator + Parallel workers):**

```rust
use tokio::task;
use rayon::prelude::*;

// Async I/O fetches data, rayon processes it
pub async fn fetch_and_process(urls: &[Url]) -> Result<Vec<Output>> {
    // 1. Async: Fetch data from network (I/O-bound)
    let data = fetch_data_from_network(urls).await?;

    // 2. Spawn blocking: Hand off to rayon for CPU work
    let result = task::spawn_blocking(move || {
        data.par_iter()
            .map(|item| expensive_cryptographic_operation(item))
            .collect::<Result<Vec<_>>>()
    })
    .await??;

    Ok(result)
}

// Async server handling parallel crypto verification
pub async fn handle_request(request: Request) -> Result<Response> {
    // Async: Receive request (I/O)
    let payload = request.read_body().await?;

    // Blocking pool: Verify signatures in parallel (CPU)
    let verified = task::spawn_blocking(move || {
        payload.signatures.par_iter()
            .try_for_each(verify_signature)
    })
    .await??;

    // Async: Send response (I/O)
    Ok(Response::success())
}

```

**Guidelines:**

**rayon (CPU-bound):**

- ✅ Cryptographic operations (signatures, proofs, hashing)
- ✅ Batch validation/processing
- ✅ Data transformations (serialization, compression)
- ✅ Mathematical computations
- ❌ Network I/O, file I/O, database queries

**tokio (I/O-bound):**

- ✅ Network servers/clients (HTTP, WebSocket, P2P)
- ✅ Database operations
- ✅ File I/O (when using async fs)
- ✅ Concurrent timers/delays
- ❌ Heavy CPU computations (blocks event loop)

**Hybrid (rayon + tokio):**

- ✅ Web servers processing crypto requests
- ✅ Blockchain nodes (network I/O + block verification)
- ✅ API endpoints with computation
- Pattern: `tokio::task::spawn_blocking` for CPU work from async context

**Anti-Patterns:**

- ❌ Using async for pure CPU work (unnecessary overhead)
- ❌ Blocking tokio runtime with CPU-intensive tasks
- ❌ Using threads for I/O when async is available
- ❌ Mixing sync and async without clear boundaries

**Performance Considerations:**

```rust
// ❌ BAD: Blocking tokio runtime
async fn bad_crypto_in_async(data: &[u8]) -> Hash {
    expensive_hash(data)  // Blocks event loop!
}

// ✅ GOOD: Offload to thread pool
async fn good_crypto_in_async(data: Vec<u8>) -> Hash {
    tokio::task::spawn_blocking(move || {
        expensive_hash(&data)
    })
    .await
    .unwrap()
}

// ❌ BAD: Async overhead for CPU work
async fn bad_parallel_compute(items: &[Item]) -> Vec<Output> {
    let futures = items.iter().map(|item| async {
        compute(item)  // Pure CPU, no I/O!
    });
    futures::future::join_all(futures).await
}

// ✅ GOOD: Data parallelism for CPU work
fn good_parallel_compute(items: &[Item]) -> Vec<Output> {
    items.par_iter()
        .map(|item| compute(item))
        .collect()
}

```

**See also:**

- `crates/z00z_core/src/genesis.rs` - Parallel genesis asset generation with rayon
- `crates/z00z_rollup_node/src/server.rs` - Tokio async server with blocking crypto
- Rayon docs: [https://docs.rs/rayon](https://docs.rs/rayon)
- Tokio docs: [https://tokio.rs](https://tokio.rs)

---

## 🔧 Rust Implementation Standards

### Module Organization

**Structure Guidelines:**

- **Depth:** Maximum 1-2 levels (avoid deep nesting)
- **Visibility:** Use `pub(crate)` for internal APIs
- **Facade:** Export public API only through `src/lib.rs`
- **Shared Types:** Place in module to avoid cycles

**Standard Layout:**

```text
📁 crate/
├── 📁 benches/
│   ├── 📁 module_a/
│   │   ├── 📄 a1_bench.rs
│   │   ├── 📄 a2_bench.rs
│   │    └── 📄 README.md
│   └── 📁 module_b/
│       ├── 📄 b1_bench.rs
│       └── 📄 b2_bench.rs
│       └── 📄 README.md
├── 📁 bin/
│   ├── 📁 module_a/
│   │   └── 📄 a1_cli.rs
│   └── 📁 module_b/
│       ├── 📄 a1_cli.rs
│       └── 📄 README.md
├── 📄 Cargo.lock
├── 📄 Cargo.toml
├── 📁 docs/
│   ├── 📁 module_a/
│   │   ├── 📄 A1.md
│   │   └── 📄 A2.md
│   └── 📁 module_b/
│       ├── 📄 B1.md
│       └── 📄 B2.md
│
├── 📁 examples/
│   ├── 📁 module_a/
│   │   ├── 📄 a1.rs
│   │   └── 📄 README.md
│   ├── 📁 module_b/
│   │   ├── 📄 b_config.yaml
│   │   ├── 📄 b_example.rs
│   │   └── 📄 README.md
│   └── 📄 README.md
│
├── 📁 outputs/
│   ├── 📁 module_a/
│   └──📁 module_b/

├── 📁 scripts/
│   └── 📄 f1.sh
├── 📁 src/
│   ├── 📁 module_a/
│   │   ├── 📄 a_config.rs
│   │   ├── 📄 a_config_schema.yaml
│   │   ├── 📄 mod.rs
│   ├── 📁 module_b/
│   │   ├── 📄 b_config.rs
│   │   ├── 📄 b_config_schema.yaml
│   │   ├── 📄 mod.rs
│   └── 📄 lib.rs
└── 📁 tests/
    ├── 📁 module_a/
    │   ├── 📄 a_integration.rs
    │   ├── 📄 a_fixtures.rs
    └── 📁 module_b/
        ├── 📄 b_integration
        └── 📄 b_fixtures.rs


```

**lib.rs Pattern:**

```rust
//! Crate documentation
//!
//! # Overview
//! ...
//!
//! # Examples
//! ...

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod common;
mod module_a;
mod module_b;

// Public facade - explicit re-exports only
pub use common::{Error, Result};
pub use module_a::{TypeA, FunctionA};
pub use module_b::{TypeB, FunctionB};

// DO NOT re-export internal types

```

**Benefits:**

- Clear public API surface
- Prevents accidental exposure of internal types
- Enables refactoring without breaking changes
- Clean documentation (`cargo doc`)

---

### 🧭 Facade, Compatibility, And Split Governance

📌 **Mandatory policy:** Structural refactors and public-surface cleanup MUST
preserve contract maturity, canonical ownership, and caller-visible stability.
A crate may still be evolving internally, but it MUST NOT export placeholder,
stub-heavy, vendor-coupled, or duplicate seams as if they were the project's
default stable contract.

#### Stable Facade Rules

- Keep crate roots and shallow facade modules narrow and explicit. Avoid broad
    wildcard re-exports such as `pub use module::*` when they turn an entire
    subtree into a de facto public API.

- The default caller path MUST expose only governed, production-ready
    contracts. Compatibility-only, experimental, placeholder, or stub-heavy
    surfaces MUST live behind explicit lanes such as `compat`, `experimental`,
    `vendor`, or feature-gated modules.

- Prefer one stable facade plus explicitly named secondary lanes over multiple
    public entrypoints for the same capability.

- If a helper becomes public during normalization, add rustdoc in the same wave
    so the public surface, examples, and verification gates stay aligned.

- After a split stabilizes, hide deep implementation modules as crate-private or
    internal when external consumers no longer need them. The shallow facade,
    not the split file path, must remain the canonical caller entrypoint.

#### Vendor And Backend Isolation Rules

- Vendor isolation is not achieved by read-only source layout alone. It also
    requires the default public surface to avoid naming vendor concrete types as
    if they were project-native contracts.

- Backend-specific helpers and persistence mechanics MUST remain behind
    dedicated internal seams. Stable namespaces should expose project-owned
    abstractions, aliases, wrappers, or documented expert lanes only.

- When backend substitution is only aspirational, do not scatter leaky
    passthroughs across the root facade. Keep the lock-in explicit and localized.

#### Responsibility-First Split Policy

- Split by responsibility seams first and line counts second.
- Preferred facade band is roughly `80-220` lines, with a normal facade target
    below `300` lines. Larger facades are acceptable only when they remain
    cohesive and are clearly acting as rustdoc, re-export, orchestration, or
    compatibility layers rather than mixed-concern execution files.

- Size bands are heuristics, not quotas. Do not cut one homogeneous
    responsibility into brittle shards merely to satisfy a numeric target.

- If a planned module is likely to land above `900` lines, add another seam
    before implementation starts. If any resulting module reaches `1000+` lines,
    the split is incomplete.

- Extract pure types and helpers first, then move stateful logic and impl
    blocks, and shrink the facade last.

- Reject shard-style layouts where many tiny files still form one concept that
    must always be read together.

#### Path Normalization And Alias Rules

- Preserve caller-visible paths during structural split waves through existing
    shallow aliases, facade modules, or explicit re-exports.

- External path normalization is a dedicated, inventory-backed closeout wave.
    Do not combine the first internal split and the first caller-visible path
    rewrite in the same wave for protected seams.

- Normalize consumers to existing shallow aliases before moving or renaming deep
    modules.

- Prefer existing shallow facades over temporary compatibility shims or
    duplicate public APIs.

- Deep implementation walkthroughs, migration notes, or file-trace documents
    MUST be labeled as internal detail so they do not compete with the canonical
    public contract.

#### Protected Seams And Canonical Ownership

- Cryptographic surfaces MUST preserve one canonical owner for domain tags,
    transcript framing helpers, AAD builders, KDF info constants, and public
    crypto entrypoints. Do not create parallel public owners during a refactor.

- Boundary-sensitive wallet or persistence symbols, such as session, lock, scan
    state, and store helpers, MUST keep stable caller-visible contracts while
    internals move.

- Consensus aliases and other shallow domain entrypoints MUST become canonical
    before deep module cleanup.

- Concrete JSON or wire-format values belong at transport or adapter edges.
    Domain cores should prefer domain types or project codec abstractions.

- `include!` MUST NOT be used to assemble long-lived service, application, or
    boundary-defining modules. Use explicit submodules plus one thin facade.

#### Integration Harness And Namespace Rules

- Integration harnesses such as simulators may depend on many crates, but they
    MUST enter those crates through stable crate-root facades or other explicitly
    approved seams rather than through implementation-detail modules.

- A directory or namespace name is not a complete architecture boundary. Before
    a subsystem grows, define ownership, lifecycle, timeout, retry,
    authentication, and fault-containment seams in writing.

- Transport or networking abstractions are incomplete if they define only
    message-shaped calls but leave timeout policy, retry posture, streaming,
    peer identity, authentication, connection lifecycle, and fault containment
    implicit.

- Cross-cutting utility crates need an explicit admission checklist. Add a new
    helper there only when it strengthens an existing abstraction boundary or
    removes duplication across multiple business crates; do not use the shared
    foundation as a general dumping ground for convenience code.

#### Wave Verification, Sync, And Rollback Rules

- Every structural wave MUST start with a fail-fast regression gate, then run
    targeted tests for the touched crate or seam in the same wave.

- Protected-seam or normalization waves MUST also run a broader release-style
    verification gate plus a grep-backed audit for legacy deep imports, stale
    rustdoc paths, and stale planning or migration references.

- Docs, rustdoc examples, YAML or tool wiring, and planning artifacts MUST be
    updated in the same wave as module, alias, or public-path changes.

- Protected-seam and normalization waves MUST name exact verification anchors in
    the plan and closeout notes. Generic statements such as `cargo test` are
    insufficient where boundary-sensitive behavior is at stake.

- A normalization wave is incomplete unless it records the caller inventory
    source, the exact caller updates performed, and the verification evidence
    that no legacy deep-import or stale rustdoc path survived in the governed
    surface.

- Do not parallelize waves that touch the same facade root, the same public
    re-export surface, or the same boundary-sensitive symbols.

- Stop widening a wave and roll back to the last stable path-preserving seam if
    the current plan creates duplicate public entrypoints, multi-crate path churn
    without caller inventory, circular imports, hidden boundary breakage, or
    shard-like modules with no strong responsibility boundary.

**Enforcement (adapt paths to your project):**

```bash
# Broad wildcard facades should be rare and intentional
rg -n "pub use .*::\\*" crates/*/src/lib.rs crates/*/src/**/mod.rs

# Boundary-defining modules should not be assembled from include! fragments
rg -n "include!" crates/*/src -g '*.rs'

# Closeout audits should prove deep-path cleanup and shallow-facade adoption
rg -n "crate::[A-Za-z0-9_]+::[A-Za-z0-9_]+::" crates/*/src -g '*.rs'

```

**See also:**

- Phase 030 refactor rules for facade-preserving seam extraction,
    normalization-wave discipline, and same-wave sync requirements.

- Phase 031 architecture review for contract-maturity, vendor-isolation,
    simulator-boundary, namespace-ownership, and placeholder-surface findings.

---

### NASA Rules

📌 **Mandatory policy:** Z00Z adopts a Rust-oriented adaptation of the NASA JPL
"Power of Ten" rules as a project-wide engineering standard. These rules are
mandatory for all new code and all substantial modifications, with the strictest
enforcement on crypto, consensus, storage, validation, wallet persistence, and
network-facing logic. Legacy exceptions must be explicit, justified, and tracked
as remediation work.

📌 **Interpretation note:** The original rules were written for C. In Z00Z we
apply their intent to Rust, preserving the verification goals while mapping the
rules to project conventions such as `Result`-based error handling,
`#![forbid(unsafe_code)]`, split policy, and repository verify gates.

1. **Keep control flow simple.** Avoid recursion in production critical paths,

prefer straight-line logic with explicit branches, and use early error returns
when they make failure handling clearer. Any exception that obscures the call
graph or makes bounded execution harder to prove must be redesigned or justified.

1. **Bound every loop.** Every loop outside deliberate schedulers, servers, or

event pumps must have a statically obvious upper bound or an explicit runtime
budget guard. Iteration over attacker-controlled, untrusted, or persistent data
must fail closed once the bound is exceeded.

1. **Do not rely on unbounded runtime allocation in critical paths.** Prefer

pre-sized buffers, bounded growth, object reuse, and initialization-time
allocation for hot or safety-relevant code. If runtime heap allocation remains
necessary, cap it, measure it, and document why a bounded alternative was not
used.

1. **Keep functions small enough to audit as a unit.** For safety-critical

logic, target roughly `<= 60` normalized lines per worker function. If a
function grows beyond that, split by responsibility seam rather than by syntax.
Facade modules and compatibility layers remain governed by the broader split
policy above; this rule applies primarily to executable logic units.

1. **Encode invariants aggressively.** Non-trivial functions should carry

explicit precondition, postcondition, or loop-invariant checks. In Z00Z, prefer
typed validation and error returns for user- or data-driven failures, and use
side-effect-free `debug_assert!` or `assert!` only for internal invariants that
must never be false.

1. **Keep data at the narrowest useful scope.** Declare variables in the

smallest block that needs them, avoid reusing one variable for multiple roles,
and keep file-local state private unless a wider scope is demonstrably needed.

1. **Check every meaningful return path and validate every parameter.** All

`Result`, `Option`, and other non-void outcomes must be handled explicitly.
Input parameters, indexes, lengths, encodings, and state transitions must be
validated before use. Ignoring a return value is allowed only when the discard
is intentional, harmless, and documented in code.

1. **Minimize macro and conditional-compilation complexity.** Prefer functions,

traits, and modules over macro-generated control flow. Keep macros simple,
syntactically complete, and local in purpose; do not hide pointer-like access,
unsafe behavior, or meaningful control flow inside macros. Keep `#[cfg(...)]`
surface area small and justified because each additional variant multiplies the
verification matrix.

1. **Keep indirection shallow and analyzable.** Raw pointers and `unsafe` code

are forbidden by default in Z00Z and must remain isolated when unavoidable.
Avoid deep ownership chains, opaque callback graphs, and dynamic dispatch that
makes call targets difficult to enumerate in critical paths. Humans and tools
must be able to reconstruct the call graph with modest effort.

1. **Run with zero-warning discipline.** All code must build with warnings

enabled and must pass repository verification without new warnings. In practice
this means `cargo fmt`, `cargo clippy --all-targets --all-features`, relevant
tests, and the repository verification gate are part of the definition of done.
If a tool reports a confusing warning, rewrite the code until the warning is
either resolved or the intent becomes mechanically obvious.

📌 **Z00Z enforcement rule:** These NASA-style rules complement, not replace,
the existing Z00Z rules in this document. When two rules overlap, follow the
stricter interpretation that improves analyzability, bounded execution,
observability, and auditability.

---

### Error Handling

**Standard Pattern:**

```rust
use thiserror::Error;

/// Errors for asset operations
#[derive(Debug, Error)]
pub enum AssetError {
    #[error("invalid commitment: {0}")]
    InvalidCommitment(String),

    #[error("amount exceeds maximum: {amount} > {max}")]
    AmountTooLarge { amount: u64, max: u64 },

    #[error("cryptographic operation failed")]
    CryptoError(#[from] CryptoError),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, AssetError>;

```

**Rules:**

- ✅ Use `thiserror::Error` for all error enums
- ✅ Provide context in error messages
- ✅ Use `#[from]` for error conversions
- ✅ Define crate-specific `Result<T>` type alias
- ❌ NEVER use `unwrap()` in production code
- ❌ NEVER use `expect()` without documented safety invariant
- ❌ NEVER use `panic!()` for error conditions

**Exception:** `expect()` is allowed when invariant is proven:

```rust
// SAFE: Config validated at initialization, key guaranteed to exist
let value = config.get("required_key")
    .expect("BUG: required_key missing after validation");

```

---

### Public API Design

**Principles:**

- **Small Surface:** Expose minimum necessary API
- **Stability:** Changes require semver major bump
- **Builder Pattern:** For types with many optional parameters
- **Interface Segregation:** Return traits, hide implementations

**Example:**

```rust
/// Transaction builder for confidential transfers
pub struct TransactionBuilder { /* private fields */ }

impl TransactionBuilder {
    pub fn new() -> Self { /* ... */ }

    pub fn sender(mut self, key: &SecretKey) -> Self {
        self.sender = Some(key.clone());
        self
    }

    pub fn recipient(mut self, key: &PublicKey) -> Self {
        self.recipient = Some(key.clone());
        self
    }

    pub fn amount(mut self, amount: u64) -> Self {
        self.amount = Some(amount);
        self
    }

    pub fn build(self) -> Result<Transaction, Error> {
        // Validate required fields
        let sender = self.sender.ok_or(Error::MissingSender)?;
        let recipient = self.recipient.ok_or(Error::MissingRecipient)?;
        let amount = self.amount.ok_or(Error::MissingAmount)?;

        // Construct transaction
        Ok(Transaction::new(sender, recipient, amount))
    }
}

```

**Benefits:**

- Fluent API (chainable methods)
- Compile-time enforcement of required fields
- Clear error messages for missing data
- Easy to extend with new optional parameters

---

### Naming Conventions

| Element | Convention | Example |
| --------- | ----------- | --------- |
| Types/Structs | `PascalCase` (nouns) | `TransactionBuilder`, `Asset` |
| Traits | `PascalCase` (capabilities) | `TimeProvider`, `Serializable` |
| Functions/Methods | `snake_case` (verbs) | `create_asset()`, `verify()` |
| Boolean Functions | `is_*` or `has_*` prefix | `is_valid()`, `has_signature()` |
| Constants | `SCREAMING_SNAKE_CASE` | `DEFAULT_TIMEOUT`, `MAX_AMOUNT` |
| Modules | `snake_case` (nouns) | `transaction`, `asset_registry` |
| Type Parameters | Single uppercase letter or `PascalCase` | `T`, `E`, `TimeProvider` |

Rust imports from the same crate or module MUST be grouped into a single
`use` statement with braces instead of being spread across multiple separate
`use` lines.

**Example:**

```rust
pub const MAX_TRANSACTION_SIZE: usize = 1024;

pub trait AssetValidator {
    fn is_valid(&self, asset: &Asset) -> bool;
}

pub struct StandardValidator;

impl AssetValidator for StandardValidator {
    fn is_valid(&self, asset: &Asset) -> bool {
        asset.amount() > 0 && asset.has_valid_commitment()
    }
}

pub fn validate_asset(asset: &Asset) -> Result<()> {
    // ...
}

```

### Identifier Length Rule

The naming conventions above are mandatory for all crates, modules, and APIs.

**Rule:** No identifiers longer than 5 words.

#### 1) What is checked

The rule MUST validate these identifier types:

- **Functions**: free functions, inherent methods, trait methods, async fns.
- **“Signatures” identifiers**: constants / failpoint IDs / metric IDs / any signature-like variable/const names used as identifiers in code (NOT comments).

#### 2) Word counting definition (mandatory)

An identifier’s **word count** is computed by splitting its name using **all** of the following boundaries:

- `_` (snake_case / SCREAMING_SNAKE_CASE)
- `-`
- **camelCase / PascalCase transitions** (e.g., `walletIdIsDeterministic` → `wallet`, `Id`, `Is`, `Deterministic`)

Empty tokens are ignored.

#### 3) Violation condition

An identifier is a violation if:

- `word_count(identifier) > 5`

Examples of violations:

- `CREATE_WALLET_WLT_FAILPOINT_AFTER_DB_CREATE` (too many words)
- `wallet_id_is_deterministic_with_mock_rng_provider` (too many words)

#### 4) Enforcement requirement

 If violations already exist, they MUST be reported and scheduled for rename.

#### 5) Rename recommendation constraints

For every violating identifier, `recommended_rename` MUST:

- have **≤ 5 words** under the same word-count rules,
- reflect the **real behavior/purpose** from the implementation logic,
- keep project style:
  - functions/methods: `snake_case`
  - constants/signatures: `SCREAMING_SNAKE_CASE` (when applicable)

---

### Documentation Requirements

**Mandatory Documentation:**

1. **Crate-level (lib.rs):**

```rust
//! # Z00Z Core - Confidential Transaction System
//!
//! Core business logic for Z00Z blockchain including assets, transactions,
//! and genesis block generation.
//!
//! ## Features
//!
//! - Confidential transactions using Pedersen commitments
//! - Asset definitions with configurable policies
//! - Deterministic genesis block generation
//!
//! ## Examples
//!
//! ```
//! use z00z_core::{Asset, AssetDefinition};
//!
//! let definition = AssetDefinition::new("Token", "TKN")?;
//! let asset = Asset::create(100, &definition)?;
//! # Ok::<_, Box<dyn std::error::Error>>(())
//! ```

#![doc = include_str!("../README.md")]

```

1. **Public Items:**

```rust
/// Creates a confidential asset with Pedersen commitment.
///
/// # Arguments
///
/// * `amount` - Asset amount (must be > 0)
/// * `blinding` - Cryptographic blinding factor for privacy
/// * `definition` - Asset policy and metadata
///
/// # Returns
///
/// Returns `Ok(Asset)` on success, or `AssetError` if validation fails.
///
/// # Errors
///
/// - `AssetError::InvalidAmount` - If amount is zero
/// - `AssetError::InvalidCommitment` - If commitment construction fails
///
/// # Examples
///
/// ```
/// use z00z_core::{create_asset, AssetDefinition, BlindingFactor};
/// use z00z_crypto::RistrettoSecretKey;
///
/// let definition = AssetDefinition::new("Token", "TKN")?;
/// let blinding = BlindingFactor::random(&mut rng);
/// let asset = create_asset(1000, blinding, &definition)?;
/// assert_eq!(asset.amount(), 1000);
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
///
/// # Security
///
/// Blinding factor MUST be generated from cryptographically secure RNG.
/// Reusing blinding factors compromises privacy.
pub fn create_asset(
    amount: u64,
    blinding: BlindingFactor,
    definition: &AssetDefinition,
) -> Result<Asset, AssetError> {
    // Implementation
}

```

**Required Sections:**

- **Summary** - One-line description
- **Arguments** - Each parameter documented
- **Returns** - Success/error cases
- **Errors** - Specific error variants
- **Examples** - Compilable, runnable example
- **Security** (if applicable) - Critical security considerations

**Verification:**

```bash
cargo doc --no-deps --document-private-items
# Must build with zero warnings

```

---

### Testing Strategy

**Test Pyramid:**

1. **Unit Tests (70%)** - Next to code in `mod tests {}`
   - Test individual functions/methods
   - Cover edge cases and error paths
   - Fast execution (no I/O, no network)

2. **Integration Tests (25%)** - In `tests/*.rs`
   - Test through public API only
   - Realistic end-to-end scenarios
   - May use file system, test fixtures

3. **Doc Tests (5%)** - In `///` documentation
   - Verify examples compile and run
   - Ensure API usage is correct
   - Catch breaking changes early

**Example:**

```rust
/// Parse genesis configuration from YAML file
///
/// # Examples
///
/// ```
/// use z00z_core::genesis::load_genesis_config;
///
/// let config = load_genesis_config("config/genesis.yaml")?;
/// assert_eq!(config.network(), "testnet");
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
pub fn load_genesis_config(path: &str) -> Result<GenesisConfig> {
    // Implementation
}

#[cfg(test)]
mod tests {
    use super::*;
    use z00z_utils::time::MockTimeProvider;

    #[test]
    fn test_load_valid_config() {
        let config = load_genesis_config("fixtures/valid.yaml").unwrap();
        assert_eq!(config.network(), "testnet");
        assert_eq!(config.version(), "1.0.0");
    }

    #[test]
    fn test_load_invalid_config() {
        let result = load_genesis_config("fixtures/invalid.yaml");
        assert!(result.is_err());
        assert!(matches!(result, Err(Error::InvalidSchema(_))));
    }

    #[test]
    fn test_deterministic_generation() {
        let mock_time = MockTimeProvider::fixed(/* specific time */);
        let config1 = generate_with_time(&mock_time);
        let config2 = generate_with_time(&mock_time);
        assert_eq!(config1, config2);  // Same inputs → same outputs
    }
}

```

**Property-Based Testing (Optional but Recommended):**

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_asset_roundtrip(amount in 1u64..1_000_000) {
        let asset = Asset::create(amount, ...)?;
        let serialized = asset.serialize()?;
        let deserialized = Asset::deserialize(&serialized)?;
        prop_assert_eq!(asset, deserialized);
    }
}

```

**Test Coverage Target:** Minimum 80% line coverage, 90%+ for critical paths (genesis, cryptography, validation).

---

### Performance & Benchmarking

**Benchmarking Standard:**

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_crypto_operation(c: &mut Criterion) {
    let factory = CryptoFactory::default();
    let secret = SecretKey::random(&mut rng);
    let input_data = vec![0u8; 32];

    c.bench_function("crypto_operation", |b| {
        b.iter(|| {
            factory.compute(black_box(&secret), black_box(&input_data))
        })
    });
}

fn benchmark_batch_processing(c: &mut Criterion) {
    let items = generate_test_data(1000);

    c.bench_function("batch_processing_1000", |b| {
        b.iter(|| {
            process_batch(black_box(&items))
        })
    });
}

fn benchmark_hash_computation(c: &mut Criterion) {
    let data = vec![0u8; 1024];

    c.bench_function("hash_1kb", |b| {
        b.iter(|| {
            compute_hash(black_box(&data))
        })
    });
}

criterion_group!(benches,
    benchmark_crypto_operation,
    benchmark_batch_processing,
    benchmark_hash_computation
);
criterion_main!(benches);

```

**Performance Guidelines:**

- **Hot Paths:** Accept `&[u8]` / `&str` / iterators (avoid clones)
- **Allocations:** Use `SmallVec` for small collections, arena allocators for graphs
- **Borrowing:** Prefer `Cow<'a, str>` for conditional ownership
- **Parallelism:** Use `rayon` for CPU-bound parallelizable tasks
- **Black Box:** Always use `black_box()` to prevent compiler optimizations in benchmarks

**Example (Efficient API):**

```rust
// ❌ Inefficient (forces clone)
pub fn process(data: Vec<u8>) -> Result<Output> { /* ... */ }

// ✅ Efficient (zero-copy)
pub fn process(data: &[u8]) -> Result<Output> { /* ... */ }

// ✅ Flexible (caller decides ownership)
pub fn process<'a>(data: impl Into<Cow<'a, [u8]>>) -> Result<Output> {
    let data = data.into();
    // Uses borrowed data if provided, owned if needed
}

```

**See also:** `crates/z00z_crypto/benches/` - Pedersen commitment and range proof benchmarks.

---

### Safety & Unsafe Code

**Default Policy:** `#![forbid(unsafe_code)]` in all crates.

**Exception Process:**

1. Prove `unsafe` is necessary (performance, FFI, low-level primitives)
2. Localize to smallest possible scope (single function, not module)
3. Document safety invariants in comments
4. Require architecture review + approval

**Example:**

```rust
/// SAFETY: This function is safe because:
/// 1. `ptr` is guaranteed non-null (validated by caller)
/// 2. `len` is exact byte length of allocation
/// 3. Memory is properly aligned for `u64`
/// 4. Lifetime 'a ensures memory is valid for returned slice
unsafe fn as_u64_slice<'a>(ptr: *const u8, len: usize) -> &'a [u64] {
    debug_assert!(!ptr.is_null());
    debug_assert!(len % 8 == 0);
    std::slice::from_raw_parts(ptr as *const u64, len / 8)
}

```

**Verification:**

```bash
# Check for unsafe usage
rg "unsafe" src/ --type rust
# Every match must have documented safety invariant

```

---

## 🧩 Universal Design Patterns

This section describes reusable architectural patterns applicable to any Rust blockchain, cryptographic system, or distributed application. Each pattern is presented as a universal solution with Z00Z implementation examples in "See also" references.

**Pattern Catalog:**

1. **Policy-State Separation** - Memory-efficient entity design separating shared immutable configuration from per-instance state
2. **Deterministic Initialization** - Reproducible bootstrapping using seed-based generation with domain separation
3. **Unique Identifier Generation** - Multiple strategies for generating unique IDs (hash-based, sequential, time-based, random)
4. **Validation Layering** - Progressive validation stages ordered by cost (schema → business → crypto)
5. **Configuration Management** - Layered configuration with file defaults and environment overrides
6. **Snapshot & Synchronization** - Versioned snapshots for state synchronization with checksums
7. **Wire Format & Protocol Versioning** - Version-tagged serialization with backward compatibility
8. **Global Singleton with Lazy Initialization** - Thread-safe global state management
9. **Bitflag Configuration** - Efficient boolean flags using bitwise operations
10. **Test Fixtures Organization** - Structured test data with shared utilities

---

### Policy-State Separation Pattern

**Problem:** Entities have both immutable configuration (policy) shared across many instances, and mutable per-instance state. Storing policy in each instance wastes memory.

**Solution:** Separate immutable policy (shared via `Arc`) from mutable instance state.

**Universal Pattern:**

```rust
/// Policy: Immutable configuration shared across instances
pub struct EntityPolicy {
    id: [u8; 32],
    name: String,
    rules: Rules,
    constraints: Constraints,
    // All immutable fields
}

/// Instance: Mutable per-instance state
pub struct EntityInstance {
    policy: Arc<EntityPolicy>,  // Shared policy (cheap to clone Arc)
    instance_id: u64,           // Unique per instance
    state: State,               // Mutable state
    metadata: Metadata,         // Instance-specific data
    // Per-instance fields
}

impl EntityInstance {
    pub fn new(
        policy: Arc<EntityPolicy>,
        instance_id: u64,
        initial_state: State,
    ) -> Self {
        Self {
            policy,
            instance_id,
            state: initial_state,
            metadata: Metadata::default(),
        }
    }

    // Policy accessed through Arc (no copy)
    pub fn get_policy(&self) -> &EntityPolicy {
        &self.policy
    }
}

```

**Memory Efficiency:**

```rust
// Without separation: N instances × sizeof(Policy + State)
// With separation: M policies × sizeof(Policy) + N instances × sizeof(Arc + State)
//
// Example: 10,000 instances with 10 unique policies
// - Without: 10,000 × 200 bytes = 2,000,000 bytes
// - With: 10 × 200 bytes + 10,000 × 16 bytes = 162,000 bytes (~92% reduction)

```

**Usage Pattern:**

```rust
// Create shared policies
let policy_a = Arc::new(EntityPolicy::new("TypeA", rules_a)?);
let policy_b = Arc::new(EntityPolicy::new("TypeB", rules_b)?);

// Create many instances sharing same policy
let instances: Vec<EntityInstance> = (0..1000)
    .map(|i| {
        let policy = if i % 2 == 0 { Arc::clone(&policy_a) } else { Arc::clone(&policy_b) };
        EntityInstance::new(policy, i, initial_state())
    })
    .collect();

// Memory: Only 2 EntityPolicy objects, 1000 lightweight instances

```

**Benefits:**

- **Memory Efficiency:** Shared policy across instances (Arc clone is cheap pointer copy)
- **Type Safety:** Policy cannot be modified after creation
- **Immutability:** Thread-safe sharing of policy (`Arc<T>` requires `T: Send + Sync`)
- **Flexibility:** Easy to swap policies by changing Arc reference

**When to Use:**

- ✅ Many instances sharing same configuration
- ✅ Policy is immutable after creation
- ✅ Instance state is much smaller than policy
- ✅ Need to share configuration across threads

**When NOT to Use:**

- ❌ Policy changes frequently (use Cow or strategies instead)
- ❌ Only few instances per policy (overhead not worth it)
- ❌ Policy and state are similar size (no memory benefit)

**See also:** `z00z_core::assets::{Asset, AssetDefinition}` - Example implementation with cryptographic properties

---

### Deterministic Initialization Pattern

**Problem:** System initialization must be reproducible (same inputs → same outputs) for auditing, testing, or consensus protocols.

**Solution:** Derive all randomness and state from a single seed using domain-separated derivation.

**Universal Pattern:**

```rust
/// Initialization accumulator: builds state deterministically
pub struct DeterministicBuilder<T> {
    seed: [u8; 32],
    context: Context,
    items: Vec<T>,
}

impl<T> DeterministicBuilder<T> {
    pub fn new(seed: [u8; 32], context: Context) -> Self {
        Self {
            seed,
            context,
            items: Vec::new(),
        }
    }

    /// Add item using deterministic derivation
    pub fn add_item(&mut self, config: ItemConfig, index: u64) -> Result<()> {
        // Derive deterministic randomness from seed + index
        let item_seed = derive_item_seed(&self.seed, index, &self.context);
        let rng = DeterministicRng::from_seed(item_seed);

        // Create item using deterministic RNG
        let item = T::create_deterministic(config, rng)?;

        self.items.push(item);
        Ok(())
    }

    /// Finalize and verify determinism
    pub fn finalize(self) -> Result<InitialState<T>> {
        // Validate all items
        self.validate_items(&self.items)?;

        // Compute state fingerprint for verification
        let fingerprint = compute_fingerprint(&self.items);

        Ok(InitialState {
            items: self.items,
            fingerprint,
            context: self.context,
        })
    }
}

/// Deterministic RNG from seed
struct DeterministicRng {
    state: [u8; 32],
}

impl DeterministicRng {
    fn from_seed(seed: [u8; 32]) -> Self {
        Self { state: seed }
    }

    fn next_bytes(&mut self, buf: &mut [u8]) {
        // Deterministic generation (e.g., ChaCha20)
        // Same seed → same sequence
    }
}

/// Domain-separated derivation
fn derive_item_seed(
    master_seed: &[u8; 32],
    index: u64,
    context: &Context,
) -> [u8; 32] {
    // Use domain separation to prevent cross-context attacks
    let mut hasher = DomainHasher::new(b"item_seed_derivation_v1");
    hasher.update(master_seed);
    hasher.update(&index.to_le_bytes());
    hasher.update(context.as_bytes());
    hasher.finalize()
}

```

**Verification:**

```rust
#[test]
fn test_deterministic_initialization() {
    let seed = [42u8; 32];
    let context = Context::Test;

    // Build state twice with same inputs
    let state1 = {
        let mut builder = DeterministicBuilder::new(seed, context.clone());
        builder.add_item(config1, 0)?;
        builder.add_item(config2, 1)?;
        builder.finalize()?
    };

    let state2 = {
        let mut builder = DeterministicBuilder::new(seed, context.clone());
        builder.add_item(config1, 0)?;
        builder.add_item(config2, 1)?;
        builder.finalize()?
    };

    // Same inputs → identical outputs
    assert_eq!(state1.fingerprint, state2.fingerprint);
    assert_eq!(state1.items, state2.items);
}

```

**Benefits:**

- **Reproducibility:** Same seed → identical state across runs
- **Auditability:** Anyone can verify initialization with same seed
- **Testability:** Deterministic tests without flakiness
- **Context Isolation:** Different contexts produce different outputs (prevent replay)

**When to Use:**

- ✅ Consensus protocols (all nodes must agree on initial state)
- ✅ Auditable systems (initialization must be verifiable)
- ✅ Testing (reproducible test data)
- ✅ Blockchain genesis blocks

**When NOT to Use:**

- ❌ Production secrets (use crypto RNG, never deterministic)
- ❌ Privacy-sensitive data (determinism can leak information)
- ❌ Dynamic initialization (changing requirements)

**See also:** `z00z_core::genesis::GenesisAssetAccumulator` - Blockchain genesis implementation

---

### Unique Identifier Generation Pattern

**Problem:** Entities need unique, unpredictable identifiers that prevent correlation analysis and collision attacks.

**Solution:** Combine deterministic components (type, index) with non-deterministic entropy (time, randomness) using domain-separated hashing.

**Universal Pattern:**

```rust
/// Unique identifier with security properties
pub struct UniqueId([u8; 32]);

impl UniqueId {
    /// Generate identifier from multiple entropy sources
    pub fn generate<T: TimeSource, R: RandomSource>(
        entity_type: &[u8],
        instance_id: u64,
        time: &T,
        rng: &mut R,
    ) -> Self {
        let timestamp = time.timestamp_micros();

        // Mix random entropy
        let mut random_bytes = [0u8; 16];
        rng.fill_bytes(&mut random_bytes);

        // Domain-separated hash combining all inputs
        let mut hasher = DomainHasher::new(b"unique_id_v1");
        hasher.update(entity_type);
        hasher.update(&instance_id.to_le_bytes());
        hasher.update(&timestamp.to_le_bytes());
        hasher.update(&random_bytes);

        UniqueId(hasher.finalize())
    }

    /// Deterministic generation (for reproducible scenarios)
    pub fn generate_deterministic(
        seed: &[u8; 32],
        entity_type: &[u8],
        index: u64,
        context: &[u8],
    ) -> Self {
        // No randomness - fully deterministic
        let mut hasher = DomainHasher::new(b"unique_id_deterministic_v1");
        hasher.update(seed);
        hasher.update(entity_type);
        hasher.update(&index.to_le_bytes());
        hasher.update(context);

        UniqueId(hasher.finalize())
    }
}

```

**Properties:**

- **Uniqueness:** Combination of time + random + instance ensures no collisions
- **Unpredictability:** Cannot predict future IDs (random component)
- **Unlinkability:** Cannot correlate IDs across different entity types (domain separation)
- **Determinism (optional):** Reproducible IDs for testing/consensus

**Comparison:**

| Approach | Uniqueness | Unpredictability | Deterministic | Use Case |
| ---------- | ----------- | ------------------ | --------------- | ---------- |
| UUID v4 | High | High | No | General purpose |
| Sequential ID | Guaranteed | None | Yes | Database keys (public) |
| Timestamp-based | Medium | Low | No | Ordering required |
| Hash(time+random) | High | High | No | Privacy-sensitive |
| Hash(seed+index) | Guaranteed | None | Yes | Consensus/testing |

**Usage Patterns:**

```rust
// Production: unpredictable IDs
let id = UniqueId::generate(
    b"transaction",
    instance_id,
    &SystemTime,
    &mut SystemRng,
);

// Testing/Consensus: reproducible IDs
let id = UniqueId::generate_deterministic(
    &seed,
    b"transaction",
    index,
    b"testnet",
);

// Batch generation with uniqueness guarantee
let ids: Vec<UniqueId> = (0..1000)
    .map(|i| UniqueId::generate(
        b"batch_item",
        i,
        &SystemTime,
        &mut SystemRng,
    ))
    .collect();
// All IDs are unique (time + random + index combination)

```

**Security Considerations:**

```rust
// ✅ CORRECT: Different domains for different purposes
let user_id = UniqueId::generate(b"user_id", ...);
let session_id = UniqueId::generate(b"session_id", ...);
let transaction_id = UniqueId::generate(b"tx_id", ...);

// ❌ WRONG: Reusing same domain allows correlation
let id1 = UniqueId::generate(b"generic_id", ...);  // Don't do this
let id2 = UniqueId::generate(b"generic_id", ...);  // Linkable!

// ✅ CORRECT: Include context in deterministic generation
let mainnet_id = UniqueId::generate_deterministic(seed, ty, idx, b"mainnet");
let testnet_id = UniqueId::generate_deterministic(seed, ty, idx, b"testnet");
// Different contexts → different IDs (prevent replay attacks)

```

**When to Use:**

- ✅ Privacy-sensitive systems (unlinkable identifiers)
- ✅ Distributed systems (collision-free without coordination)
- ✅ Auditable systems (deterministic mode for verification)
- ✅ Cryptographic protocols (domain separation prevents attacks)

**When NOT to Use:**

- ❌ Simple database auto-increment is sufficient
- ❌ IDs must be human-readable (use formatted strings)
- ❌ Need lexicographic ordering (use ULID/timestamp-prefixed)

**See also:** `z00z_core::assets::nonce` - Privacy-preserving nonce generation

---

### Validation Layering Pattern

**Problem:** Complex validation requires multiple stages (syntax, business rules, cryptographic proofs). Mixing all checks creates unclear error messages and wastes resources on expensive operations when basic checks fail.

**Solution:** Progressive validation with fast-fail stages ordered by cost.

**Universal Pattern:**

```rust
pub trait Validator<T> {
    fn validate(&self, entity: &T) -> Result<()>;
}

// Layer 1: Fast schema/syntax validation (microseconds)
pub struct SchemaValidator;
impl Validator<Entity> for SchemaValidator {
    fn validate(&self, entity: &Entity) -> Result<()> {
        // Check basic structure
        ensure!(!entity.id.is_empty(), "ID required");
        ensure!(entity.amount > 0, "Amount must be positive");
        ensure!(entity.metadata.len() <= MAX_METADATA_SIZE, "Metadata too large");
        Ok(())
    }
}

// Layer 2: Business logic validation (milliseconds)
pub struct BusinessValidator;
impl Validator<Entity> for BusinessValidator {
    fn validate(&self, entity: &Entity) -> Result<()> {
        // Check business rules
        ensure!(entity.amount <= entity.policy.max_amount, "Exceeds limit");
        ensure!(entity.policy.is_active(), "Policy inactive");
        ensure!(entity.timestamp <= current_time(), "Future timestamp");
        Ok(())
    }
}

// Layer 3: Cryptographic validation (seconds, CPU-intensive)
pub struct CryptoValidator;
impl Validator<Entity> for CryptoValidator {
    fn validate(&self, entity: &Entity) -> Result<()> {
        // Verify signatures (expensive)
        entity.signature.verify(&entity.public_key, &entity.payload)?;

        // Verify zero-knowledge proofs (very expensive)
        entity.proof.verify(&entity.commitment, &entity.statement)?;

        Ok(())
    }
}

// Composite validator: Run stages in order, fail fast
pub struct LayeredValidator {
    schema: SchemaValidator,
    business: BusinessValidator,
    crypto: CryptoValidator,
}

impl LayeredValidator {
    pub fn validate_entity(&self, entity: &Entity) -> Result<()> {
        // Stage 1: Fast failure on basic errors (~1μs)
        self.schema.validate(entity)?;

        // Stage 2: Business rules (~1ms)
        self.business.validate(entity)?;

        // Stage 3: Expensive crypto only if previous stages passed (~100ms)
        self.crypto.validate(entity)?;

        Ok(())
    }

    // Batch validation with parallelization
    pub fn validate_batch(&self, entities: &[Entity]) -> Result<()> {
        // Stage 1 & 2: Sequential (fast)
        for entity in entities {
            self.schema.validate(entity)?;
            self.business.validate(entity)?;
        }

        // Stage 3: Parallel (CPU-bound crypto operations)
        entities.par_iter()
            .try_for_each(|entity| self.crypto.validate(entity))
    }
}

```

**Benefits:**

- **Cost Ordering:** ~1μs → ~1ms → ~100ms (fail fast on cheap checks)
- **Clear Errors:** Each layer provides specific error context
- **Parallelization:** Expensive crypto validation runs in parallel after serial checks pass
- **Testability:** Each validator tested independently

**See also:** `crates/z00z_core/src/genesis.rs` - Genesis asset validation with schema → business → cryptographic stages.

---

### Configuration Management Pattern

**Pattern:** Layered configuration with file defaults + environment overrides + programmatic customization.

**Universal Pattern:**

```rust
pub trait ConfigSource {
    fn get<T: FromStr>(&self, key: &str) -> Result<T>;
    fn contains(&self, key: &str) -> bool;
}

// Layer 1: File-based configuration (YAML, TOML, JSON)
pub struct FileConfig {
    data: BTreeMap<String, Value>,
}

impl FileConfig {
    pub fn from_yaml(path: &str) -> Result<Self> {
        let yaml = std::fs::read_to_string(path)?;
        let data = serde_yaml::from_str(&yaml)?;
        Ok(Self { data })
    }
}

// Layer 2: Environment variable configuration
pub struct EnvConfig {
    prefix: String,
}

impl EnvConfig {
    pub fn with_prefix(prefix: &str) -> Self {
        Self { prefix: prefix.to_string() }
    }
}

impl ConfigSource for EnvConfig {
    fn get<T: FromStr>(&self, key: &str) -> Result<T> {
        let env_key = format!("{}_{}", self.prefix, key.to_uppercase().replace('.', "_"));
        std::env::var(&env_key)?.parse()
            .map_err(|_| Error::ParseError(env_key))
    }
}

// Layer 3: Layered configuration (ENV overrides File)
pub struct LayeredConfig<F, E> {
    file: F,
    env: E,
}

impl<F: ConfigSource, E: ConfigSource> LayeredConfig<F, E> {
    pub fn new(file: F, env: E) -> Self {
        Self { file, env }
    }

    pub fn get<T: FromStr>(&self, key: &str) -> Result<T> {
        // Try ENV first (highest priority)
        self.env.get(key)
            // Fall back to file defaults
            .or_else(|_| self.file.get(key))
    }
}

```

**Example Configuration File:**

```yaml
# config/app.yaml
version: "1.0.0"
network:
  type: "testnet"
  timeout_ms: 5000
database:
  url: "postgresql://localhost/app"
  max_connections: 20
crypto:
  seed: "0x1234..."  # Hex-encoded seed

```

**Environment Override:**

```bash
APP_NETWORK_TYPE=mainnet \
APP_DATABASE_MAX_CONNECTIONS=50 \
./application
# Overrides YAML `network.type` and `database.max_connections`

```

**Usage:**

```rust
// Load layered configuration
let file = FileConfig::from_yaml("config/app.yaml")?;
let env = EnvConfig::with_prefix("APP");
let config = LayeredConfig::new(file, env);

// Get values (ENV overrides YAML)
let network: String = config.get("network.type")?;
let timeout: u64 = config.get("network.timeout_ms")?;
let seed: [u8; 32] = config.get("crypto.seed")?;

```

**Benefits:**

- **Defaults in Files:** Version controlled, documented, reviewed
- **Overrides in ENV:** CI/CD, Docker, Kubernetes deployments
- **Type Safety:** `ConfigSource` trait enforces parsing
- **Testability:** Use `InMemoryConfig` for tests (no file I/O)
- **Flexibility:** Add more layers (CLI args, remote config, secrets)

**See also:** `crates/z00z_utils/src/config/` - Layered configuration with YAML + ENV support for Z00Z network settings.

---

### Snapshot & Synchronization Pattern

**Problem:** Systems need to synchronize state between components (validator ↔ wallet, node ↔ node) efficiently and safely.

**Solution:** Versioned snapshot pattern with checksums and atomic updates.

**Universal Pattern:**

```rust
/// Versioned snapshot for state synchronization
pub struct StateSnapshot<T> {
    data: Vec<T>,
    version: Version,
    checksum: [u8; 32],
    timestamp: u64,
}

impl<T> StateSnapshot<T> {
    /// Create snapshot from current state
    pub fn create<S: StateProvider<T>>(
        provider: &S,
        version: Version,
    ) -> Result<Self> {
        let data = provider.collect_state()?;
        let checksum = compute_checksum(&data);
        let timestamp = current_timestamp();

        Ok(Self { data, version, checksum, timestamp })
    }

    /// Verify snapshot integrity before applying
    pub fn verify(&self) -> Result<()> {
        let computed = compute_checksum(&self.data);
        if computed != self.checksum {
            return Err(Error::ChecksumMismatch);
        }
        Ok(())
    }
}

/// State provider trait for snapshot creation
pub trait StateProvider<T> {
    fn collect_state(&self) -> Result<Vec<T>>;
    fn update_from_snapshot(&self, snapshot: StateSnapshot<T>) -> Result<()>;
}

```

**Usage Pattern:**

```rust
// Producer creates snapshot
let snapshot = StateSnapshot::create(&registry, current_version())?;
let serialized = serialize(&snapshot)?;
send_to_peer(serialized);

// Consumer receives and applies
let snapshot: StateSnapshot<Definition> = deserialize(received_data)?;
snapshot.verify()?;  // Check integrity
registry.update_from_snapshot(snapshot)?;  // Atomic update

```

**Key Properties:**

- **Versioned:** Track snapshot compatibility
- **Checksummed:** Detect corruption/tampering
- **Atomic:** All-or-nothing updates
- **Generic:** Reusable for any state type

**Use Cases:**

- Wallet synchronization (asset definitions, blockchain state)
- Checkpoint/restore (crash recovery, fast startup)
- Offline updates (export/import via files)
- P2P state synchronization

---

### Wire Format & Protocol Versioning Pattern

**Problem:** Network transmission requires different representation than in-memory structures (efficiency, compatibility, evolution).

**Solution:** Separate wire format (DTO) from domain models with explicit versioning.

**Universal Pattern:**

```rust
/// Domain model (rich behavior, internal structure)
pub struct DomainEntity {
    // Complex internal structure
    shared_reference: Arc<SharedData>,
    cached_computation: Option<Result>,
    // Private invariants
}

/// Wire format (serialization-optimized, stable API)
#[derive(Serialize, Deserialize)]
pub struct EntityWire {
    version: u8,  // Protocol version
    // Flattened, self-contained data
    id: [u8; 32],
    data: Vec<u8>,
    // No references, no computed fields
}

impl From<DomainEntity> for EntityWire {
    fn from(entity: DomainEntity) -> Self {
        Self {
            version: CURRENT_PROTOCOL_VERSION,
            id: entity.id(),
            data: entity.serialize_data(),
        }
    }
}

impl TryFrom<EntityWire> for DomainEntity {
    type Error = Error;

    fn try_from(wire: EntityWire) -> Result<Self> {
        match wire.version {
            1 => Self::from_v1(wire),
            2 => Self::from_v2(wire),
            v => Err(Error::UnsupportedVersion(v)),
        }
    }
}

```

**Versioning Strategy:**

```rust
pub const PROTOCOL_VERSION_V1: u8 = 1;  // Initial format
pub const PROTOCOL_VERSION_V2: u8 = 2;  // Added optional_field
pub const CURRENT_PROTOCOL_VERSION: u8 = PROTOCOL_VERSION_V2;

/// Version-aware deserialization
fn deserialize_wire(bytes: &[u8]) -> Result<DomainEntity> {
    let version = bytes[0];  // First byte is version

    let wire: EntityWire = match version {
        1 => deserialize_v1(&bytes[1..])?,
        2 => deserialize_v2(&bytes[1..])?,
        v => return Err(Error::UnsupportedVersion(v)),
    };

    DomainEntity::try_from(wire)
}

```

**Benefits:**

- **Separation of Concerns:** Wire format isolated from domain logic
- **Forward Compatibility:** New versions can read old formats
- **Network Efficiency:** Optimized for transmission size
- **Evolution Safety:** Explicit version handling

**Anti-patterns to Avoid:**

- ❌ Serializing domain models directly (leaks internals)
- ❌ No version field (impossible to evolve)
- ❌ Breaking changes without version bump
- ❌ Complex nested structures in wire format

---

### Global Singleton with Lazy Initialization Pattern

**Problem:** Need globally accessible state (registry, cache, connection pool) without initialization timing issues.

**Solution:** Lazy static initialization with thread-safe interior mutability.

**Universal Pattern:**

```rust
use once_cell::sync::Lazy;
use std::sync::{Arc, RwLock};

/// Global singleton with lazy initialization
pub static GLOBAL_REGISTRY: Lazy<Registry> = Lazy::new(|| {
    Registry::new(
        Arc::new(DefaultLogger),
        Arc::new(DefaultMetrics),
        Arc::new(SystemTimeProvider),
    )
});

/// Registry with interior mutability for updates
pub struct Registry {
    data: RwLock<BTreeMap<Key, Arc<Value>>>,
    logger: Arc<dyn Logger>,
    metrics: Arc<dyn MetricsSink>,
    time: Arc<dyn TimeProvider>,
}

impl Registry {
    pub fn new(
        logger: Arc<dyn Logger>,
        metrics: Arc<dyn MetricsSink>,
        time: Arc<dyn TimeProvider>,
    ) -> Self {
        Self {
            data: RwLock::new(BTreeMap::new()),
            logger,
            metrics,
            time,
        }
    }

    /// Thread-safe read access
    pub fn get(&self, key: &Key) -> Option<Arc<Value>> {
        let data = self.data.read().unwrap();
        data.get(key).cloned()  // Arc clone (cheap)
    }

    /// Thread-safe write access
    pub fn insert(&self, key: Key, value: Arc<Value>) {
        let mut data = self.data.write().unwrap();
        data.insert(key, value);
        self.metrics.increment("registry.insert", 1);
    }
}

```

**Testing Strategy:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Problem: Can't reset global singleton between tests
    // Solution: Test-specific registries

    #[test]
    fn test_with_isolated_registry() {
        // Create test-specific instance (not global)
        let registry = Registry::new(
            Arc::new(NoopLogger),
            Arc::new(NoopMetrics),
            Arc::new(MockTimeProvider::fixed(/* time */)),
        );

        // Test with isolated state
        registry.insert(key, value);
        assert_eq!(registry.get(&key), Some(value));
    }

    // For integration tests that MUST use global
    #[test]
    #[serial]  // Use serial_test crate to prevent parallel execution
    fn test_global_registry() {
        // Tests run sequentially, avoiding state conflicts
        GLOBAL_REGISTRY.insert(key, value);
    }
}

```

**Lock Ordering for Deadlock Prevention:**

```rust
/// CRITICAL: Always acquire locks in this order:
/// 1. GLOBAL_REGISTRY.data (if needed)
/// 2. Individual Value internal locks (if any)
///
/// NEVER acquire in reverse order!

// ✅ CORRECT
let registry_data = GLOBAL_REGISTRY.data.read().unwrap();
if let Some(value) = registry_data.get(&key) {
    let value_data = value.internal_data.read().unwrap();
    // Process
}

// ❌ WRONG - DEADLOCK RISK
let value = get_value_somehow();
let value_data = value.internal_data.read().unwrap();  // Lock 1
let registry_data = GLOBAL_REGISTRY.data.read().unwrap();  // Lock 2 - DEADLOCK!

```

**When to Use:**

- ✅ Truly global state (configuration, registries, caches)
- ✅ Expensive initialization (database pools, network connections)
- ✅ Shared across entire application

**When NOT to Use:**

- ❌ State that should be scoped (use passed parameters)
- ❌ Test-only fixtures (use test-specific instances)
- ❌ Mutable state without clear ownership (prefer channels/actors)

---

### Bitflag Configuration Pattern

**Problem:** Entity has multiple boolean properties that are frequently checked together or serialized compactly.

**Solution:** Bitflag-based configuration with type-safe operations.

**Universal Pattern:**

```rust
/// Policy flags as const values (not enum for bitwise ops)
pub mod flags {
    pub type Flags = u8;  // or u16, u32 for more flags

    pub const NONE: Flags       = 0b0000_0000;
    pub const PROPERTY_A: Flags = 0b0000_0001;
    pub const PROPERTY_B: Flags = 0b0000_0010;
    pub const PROPERTY_C: Flags = 0b0000_0100;
    pub const PROPERTY_D: Flags = 0b0000_1000;

    // Composite flags (common combinations)
    pub const BASIC: Flags = PROPERTY_A | PROPERTY_B;
    pub const ADVANCED: Flags = BASIC | PROPERTY_C;
    pub const ALL: Flags = PROPERTY_A | PROPERTY_B | PROPERTY_C | PROPERTY_D;
}

/// Entity with bitflag configuration
pub struct ConfigurableEntity {
    flags: flags::Flags,
    // Other fields
}

impl ConfigurableEntity {
    /// Check if specific flag is set
    pub fn has_property_a(&self) -> bool {
        self.flags & flags::PROPERTY_A != 0
    }

    /// Check if all flags in mask are set
    pub fn has_all(&self, mask: flags::Flags) -> bool {
        self.flags & mask == mask
    }

    /// Check if any flag in mask is set
    pub fn has_any(&self, mask: flags::Flags) -> bool {
        self.flags & mask != 0
    }

    /// Set specific flag
    pub fn set_property_a(&mut self) {
        self.flags |= flags::PROPERTY_A;
    }

    /// Clear specific flag
    pub fn clear_property_a(&mut self) {
        self.flags &= !flags::PROPERTY_A;
    }

    /// Validate flag combinations
    pub fn validate_flags(&self) -> Result<()> {
        // Example: PROPERTY_C requires PROPERTY_A
        if self.has_property_c() && !self.has_property_a() {
            return Err(Error::InvalidFlagCombination);
        }
        Ok(())
    }
}

```

**Builder Pattern Integration:**

```rust
pub struct EntityBuilder {
    flags: flags::Flags,
    // Other fields
}

impl EntityBuilder {
    pub fn new() -> Self {
        Self { flags: flags::NONE }
    }

    pub fn with_property_a(mut self) -> Self {
        self.flags |= flags::PROPERTY_A;
        self
    }

    pub fn with_basic_config(mut self) -> Self {
        self.flags |= flags::BASIC;
        self
    }

    pub fn build(self) -> Result<ConfigurableEntity> {
        let entity = ConfigurableEntity { flags: self.flags };
        entity.validate_flags()?;
        Ok(entity)
    }
}

// Usage
let entity = EntityBuilder::new()
    .with_property_a()
    .with_property_b()
    .build()?;

```

**Serialization:**

```rust
// Compact representation (1 byte for 8 flags)
#[derive(Serialize, Deserialize)]
pub struct EntityWire {
    flags: u8,  // Serialize as single byte
    // Other fields
}

```

**Benefits:**

- **Memory Efficient:** 8 booleans in 1 byte
- **Fast Checks:** Bitwise operations (single CPU instruction)
- **Composite Queries:** Check multiple properties at once
- **Compact Serialization:** Minimal wire format

**When to Use:**

- ✅ Multiple related boolean properties
- ✅ Properties frequently checked together
- ✅ Configuration needs serialization
- ✅ Up to 8/16/32/64 flags (depending on integer size)

**When NOT to Use:**

- ❌ Unrelated boolean properties (use separate fields)
- ❌ More than 64 flags needed (use bitflags crate or BitVec)
- ❌ Enum-like values (use actual enum)

---

### Test Fixtures Organization Pattern

**Problem:** Tests need realistic, reusable data structures without duplicating setup code.

**Solution:** Centralized fixtures module with builder patterns and property-based generation.

**Universal Pattern:**

```rust
// tests/common/fixtures.rs

/// Fixture builder for test entities
pub struct TestEntityBuilder {
    id: Option<[u8; 32]>,
    value: u64,
    flags: u8,
    // Configurable fields
}

impl TestEntityBuilder {
    pub fn new() -> Self {
        Self {
            id: None,  // Will generate if not set
            value: 100,  // Reasonable default
            flags: 0,
        }
    }

    pub fn with_id(mut self, id: [u8; 32]) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_value(mut self, value: u64) -> Self {
        self.value = value;
        self
    }

    pub fn build(self) -> TestEntity {
        TestEntity {
            id: self.id.unwrap_or_else(|| generate_test_id()),
            value: self.value,
            flags: self.flags,
        }
    }
}

/// Named fixture constructors for common scenarios
pub mod entities {
    use super::*;

    /// Minimal valid entity
    pub fn minimal() -> TestEntity {
        TestEntityBuilder::new().build()
    }

    /// Entity with specific configuration
    pub fn with_advanced_features() -> TestEntity {
        TestEntityBuilder::new()
            .with_value(1000)
            .with_flags(flags::ADVANCED)
            .build()
    }

    /// Collection of related entities
    pub fn family() -> Vec<TestEntity> {
        vec![
            minimal(),
            with_advanced_features(),
            TestEntityBuilder::new().with_value(500).build(),
        ]
    }
}

```

**Property-Based Fixture Generation:**

```rust
use proptest::prelude::*;

/// Strategy for generating valid test entities
pub fn entity_strategy() -> impl Strategy<Value = TestEntity> {
    (any::<[u8; 32]>(), 1u64..1_000_000, any::<u8>())
        .prop_map(|(id, value, flags)| {
            TestEntityBuilder::new()
                .with_id(id)
                .with_value(value)
                .with_flags(flags & flags::ALL)  // Mask to valid flags
                .build()
        })
}

proptest! {
    #[test]
    fn test_any_valid_entity_serializes(entity in entity_strategy()) {
        let serialized = serialize(&entity)?;
        let deserialized: TestEntity = deserialize(&serialized)?;
        prop_assert_eq!(entity, deserialized);
    }
}

```

**Fixture Organization:**

```text
tests/
├── common/
│   ├── mod.rs
│   ├── fixtures.rs        // Entity builders
│   ├── providers.rs       // Mock trait implementations
│   └── assertions.rs      // Custom assert helpers
├── integration/
│   ├── scenario_a_test.rs
│   └── scenario_b_test.rs
└── property/
    └── entity_properties_test.rs

```

**Usage in Tests:**

```rust
// tests/integration/scenario_a_test.rs
mod common;  // Import shared fixtures

use common::fixtures::entities;

#[test]
fn test_basic_scenario() {
    let entity = entities::minimal();
    // Test with minimal fixture
}

#[test]
fn test_complex_scenario() {
    let entities = entities::family();
    // Test with collection
}

#[test]
fn test_custom_scenario() {
    let entity = TestEntityBuilder::new()
        .with_value(specific_value)
        .build();
    // Test with customized fixture
}

```

**Benefits:**

- **DRY:** Reuse setup code across tests
- **Flexibility:** Builder pattern allows customization
- **Discoverability:** Named fixtures document common scenarios
- **Evolution:** Change fixture implementation without updating tests

---

### Advanced Testing Patterns

#### Pattern 1: Testing Cryptographic Code

**Challenge:** Crypto operations involve randomness, making tests non-deterministic.

**Solution:** Deterministic RNG with fixed seeds for reproducibility.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use z00z_utils::rng::{RngProvider, MockRngProvider};

    #[test]
    fn test_commitment_reproducibility() {
        // Fixed seed for deterministic behavior
        let rng1 = MockRngProvider::with_seed(42);
        let rng2 = MockRngProvider::with_seed(42);

        let blinding1 = BlindingFactor::random(&mut rng1.rng());
        let blinding2 = BlindingFactor::random(&mut rng2.rng());

        // Same seed → same blinding factor
        assert_eq!(blinding1, blinding2);

        // Create commitments
        let commitment1 = factory.commit_value(&blinding1, 1000);
        let commitment2 = factory.commit_value(&blinding2, 1000);

        // Same inputs → same outputs (deterministic)
        assert_eq!(commitment1, commitment2);
    }

    #[test]
    fn test_different_seeds_produce_different_results() {
        let rng1 = MockRngProvider::with_seed(42);
        let rng2 = MockRngProvider::with_seed(43);  // Different seed

        let blinding1 = BlindingFactor::random(&mut rng1.rng());
        let blinding2 = BlindingFactor::random(&mut rng2.rng());

        // Different seeds → different blinding factors
        assert_ne!(blinding1, blinding2);
    }
}

```

#### Pattern 2: Testing Multi-Dependency Injection

**Challenge:** Functions with multiple injected dependencies create complex test setup.

**Solution:** Fixture struct that bundles all dependencies.

```rust
/// Test fixture bundling all dependencies
struct TestContext {
    time: Arc<MockTimeProvider>,
    rng: Arc<MockRngProvider>,
    logger: Arc<NoopLogger>,
    metrics: Arc<NoopMetrics>,
}

impl TestContext {
    fn new() -> Self {
        Self {
            time: Arc::new(MockTimeProvider::fixed(SystemTime::UNIX_EPOCH)),
            rng: Arc::new(MockRngProvider::with_seed(42)),
            logger: Arc::new(NoopLogger),
            metrics: Arc::new(NoopMetrics),
        }
    }

    fn with_time(mut self, time: SystemTime) -> Self {
        self.time = Arc::new(MockTimeProvider::fixed(time));
        self
    }

    fn with_seed(mut self, seed: u64) -> Self {
        self.rng = Arc::new(MockRngProvider::with_seed(seed));
        self
    }
}

#[test]
fn test_complex_operation() {
    let ctx = TestContext::new()
        .with_time(specific_time)
        .with_seed(deterministic_seed);

    // Use bundled dependencies
    let result = create_entity(
        config,
        ctx.time.as_ref(),
        ctx.rng.as_ref(),
        ctx.logger.as_ref(),
    );

    // Verify time-dependent behavior
    assert_eq!(result.timestamp(), specific_time);
}

```

#### Pattern 3: Verification-Based Testing (Properties)

**Challenge:** Testing all edge cases with manual examples is exhaustive.

**Solution:** Property-based testing with invariant verification.

```rust
use proptest::prelude::*;

// Define invariants as properties
proptest! {
    /// Property: Serialization roundtrip preserves data
    #[test]
    fn prop_serialization_roundtrip(
        value in 1u64..1_000_000,
        seed in any::<u64>()
    ) {
        let ctx = TestContext::new().with_seed(seed);

        let original = create_entity(value, ctx.rng.as_ref());
        let serialized = serialize(&original)?;
        let deserialized: Entity = deserialize(&serialized)?;

        prop_assert_eq!(original, deserialized);
    }

    /// Property: Commitment validation never false-positive
    #[test]
    fn prop_commitment_validation_sound(
        amount in 1u64..1_000_000,
        wrong_amount in 1u64..1_000_000,
        seed in any::<u64>()
    ) {
        // Different amounts should produce different commitments
        prop_assume!(amount != wrong_amount);

        let ctx = TestContext::new().with_seed(seed);
        let blinding = BlindingFactor::random(&mut ctx.rng.rng());

        let commitment = factory.commit_value(&blinding, amount);

        // Verifying wrong amount must fail
        let valid = verify_commitment(&commitment, &blinding, wrong_amount);
        prop_assert!(!valid);
    }
}

```

#### Pattern 4: Coverage-Driven Test Expansion

**Approach:** Use coverage metrics to identify untested code paths.

```bash
# Install tarpaulin (coverage tool)
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage/

# View report
xdg-open coverage/index.html

```

**Test Expansion Strategy:**

```rust
// Step 1: Write initial tests (happy path)
#[test]
fn test_basic_operation() {
    let result = operation(valid_input);
    assert!(result.is_ok());
}

// Step 2: Run coverage, identify untested branches
// Coverage shows: Error path at line 42 not covered

// Step 3: Add test for uncovered path
#[test]
fn test_operation_with_invalid_input() {
    let result = operation(invalid_input);
    assert!(matches!(result, Err(Error::InvalidInput)));
}

// Step 4: Run coverage again, verify improvement
// Target: 80% line coverage minimum, 90% for critical paths

```

**CI Integration:**

```yaml
# .github/workflows/coverage.yml
name: Coverage
on: [push, pull_request]
jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
      - run: cargo install cargo-tarpaulin
      - run: cargo tarpaulin --out Xml
      - uses: codecov/codecov-action@v3
        with:
          files: ./cobertura.xml
          fail_ci_if_error: true
      # Fail if coverage drops below threshold
      - run: |
          COVERAGE=$(grep -oP 'line-rate="\K[0-9.]+' cobertura.xml | head -1)
          if (( $(echo "$COVERAGE < 0.80" | bc -l) )); then
            echo "Coverage $COVERAGE below threshold 0.80"
            exit 1
          fi

```

---

## 🛠️ Development Workflow

### Version Management

**CRITICAL:** All version updates MUST use `./.github/skills/z00z-git-versioning/scripts/version-manager.sh`.

**Commands:**

```bash
# Increment patch version (bug fixes)
./.github/skills/z00z-git-versioning/scripts/version-manager.sh patch -m "Fix commitment verification bug"

# Increment minor version (new features, backward compatible)
./.github/skills/z00z-git-versioning/scripts/version-manager.sh minor -m "Add batch validation support"

# Increment major version (breaking changes)
./.github/skills/z00z-git-versioning/scripts/version-manager.sh major -m "Refactor Asset API"

# Update specific crate version
./.github/skills/z00z-git-versioning/scripts/version-manager.sh crate z00z_core 2.0.0 -m "Breaking: Remove deprecated API"

# Sync to GitHub (force push to branch)
./.github/skills/z00z-git-versioning/scripts/version-manager.sh sync -f -b "$(git branch --show-current)"

```

**Actions Performed Automatically:**

1. Updates `versions.yaml` with new version
2. Updates all `Cargo.toml` files with new version
3. Creates git commit with conventional format
4. Creates git tag (`vX.Y.Z`)
5. Pushes to GitHub (force push to specified branch)

**See:** `.github/skills/z00z-git-versioning/scripts/VERSION_MANAGEMENT.md` for complete documentation.

---

### Safe File Operations

**CRITICAL:** NEVER use `rm -rf` or destructive deletion commands.

**Linux Safe Delete:**

```bash
# Install trash-cli (if not present)
sudo apt install trash-cli

# Move to trash (recoverable)
trash-put /path/to/file
trash-put /path/to/directory

# Alternative: GNOME utilities
gio trash /path/to/file

```

**Verification Before Delete:**

```bash
# List files to be deleted BEFORE executing
find ./target -name "*.tmp" | tee /tmp/files-to-delete.txt
# Review /tmp/files-to-delete.txt
cat /tmp/files-to-delete.txt | xargs trash-put

```

**If Trash Utilities Unavailable:** Ask user for preferred safe delete method before proceeding.

---

### Code Quality Checks

**Pre-Commit Checklist:**

```bash
# 1. Format code
cargo fmt --all

# 2. Run Clippy (zero warnings)
cargo clippy --all-targets --all-features -- -D warnings

# 3. Run tests (all pass)
cargo test --all

# 4. Run benchmarks (ensure no regressions)
cargo bench --no-run

# 5. Build documentation (zero warnings)
cargo doc --no-deps --document-private-items

# 6. Verify ONE SOURCE OF TRUTH compliance (adapt paths to your project)
grep -rn "use std::fs" crates/your_core/src/ | grep -v "std::fs::File"
grep -rn "serde_yaml::Value" crates/your_core/src/
grep -rn "serde_json::" crates/your_core/src/

# 7. Check for unsafe code
rg "unsafe" src/ --type rust

```

**CI Pipeline (GitHub Actions):**

```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: rustfmt, clippy
      - run: cargo fmt -- --check
      - run: cargo clippy --all-targets --all-features -- -D warnings
      - run: cargo test --all
      - run: cargo doc --no-deps

```

---

### User Interaction Signal

**MANDATORY:** At the end of each Copilot work cycle, execute:

```bash
./scripts/play_tone.sh

```

**Purpose:** Audible cue that AI has completed work and is awaiting user input.

**Implementation:** Script plays system tone or beep (configurable).

---

## 📚 Documentation Standards

### Emoji Usage

**MANDATORY:** Use emojis at the beginning of markdown paragraphs for improved readability.

**Recommended Emojis:**

- 📌 **Important Point** - Key information
- 🎯 **Goal/Objective** - Purpose or target
- ⏰ **Time-Sensitive** - Deadlines or timestamps
- 💥 **Breaking Change** - API-breaking changes
- ⚙️ **Configuration** - Setup or configuration
- 🔑 **Key Concept** - Fundamental idea
- ♨️ **Hot/Critical** - Urgent attention needed
- ⭐ **Highlight** - Notable feature
- 👍 **Allowed/Correct** - Good practice
- 👎 **Discouraged** - Anti-pattern
- ☢️ **Dangerous** - Security risk
- 🚫 **Forbidden** - Absolute prohibition
- 💯 **Complete** - Fully implemented
- 💫 **New Feature** - Recently added
- 👁️‍🗨️ **Note** - Additional information
- 🚨 **Warning** - Potential issue
- 🛑 **Stop** - Do not proceed
- 🔔 **Reminder** - Don't forget
- 🚩 **Flag** - Mark for review
- ⚠️ **Caution** - Proceed carefully
- ⛔ **Error** - Mistake or failure
- ✅ **Success/Allowed** - Correct approach
- ❌ **Failure/Forbidden** - Wrong approach
- ‼️ **Very Important** - Critical information
- ❓ **Question** - Open issue or query

**Example:**

```markdown
## Feature Overview

🎯 **Purpose:** This feature enables confidential transactions using Pedersen commitments.

⚙️ **Configuration:** Set `enable_confidential = true` in `config.yaml`.

✅ **Allowed:** Use `AssetBuilder` for creating assets.
❌ **Forbidden:** Direct `Asset::new()` bypasses validation.

⚠️ **Caution:** Reusing blinding factors compromises privacy.

```

---

### Date Format

**MANDATORY:** ISO 8601 format (`YYYY-MM-DD`).

**Examples:**

- ✅ `2025-01-19`
- ✅ `2025-01-19T14:30:00Z` (with time)
- ❌ `01/19/2025` (ambiguous)
- ❌ `19-Jan-2025` (non-standard)

---

### Markdown Structure

**Headers:**

```markdown
# Document Title (H1 - once per document)

## Main Section (H2)

### Subsection (H3)

#### Detail (H4 - use sparingly)

```

**Code Blocks:**

````markdown

```rust

pub fn example() -> Result<()> {
    // Code here
    Ok(())
}

```text

````

### Repository-Derived Naming Patterns

This section complements the generic Rust naming rules above. It captures the
naming patterns that are already repeated across Z00Z crates and SHOULD be
treated as the preferred repository style for new code and refactors.

#### General Rule

Names MUST express domain role first and implementation detail second. Public
names SHOULD be explicit about what they do, what boundary they represent, or
what data shape they carry.

Prefer direct action verbs such as `build_`, `create_`, `derive_`, `verify_`,
`validate_`, `lookup_`, `select_`, `load_`, `store_`, `scan_`, and `detect_`.

Avoid ambiguous public names such as:

- `*_for`
- `do_*`
- `handle_*`
- `process_*`

when a more exact verb is available.

#### Traits

Traits in Z00Z are primarily named as capabilities or trust-boundary seams, not
as implementations.

Preferred trait suffixes:

- `*Provider` for injectable dependencies and system facilities
  - `TimeProvider`
  - `SecureRngProvider`
  - `DeterministicRngProvider`
- `*Service` for coarse-grained application or runtime capabilities
  - `WatcherService`
  - `ValidatorService`
  - `AggregatorService`
- `*Store` for persistence seams
  - `CheckpointStore`
  - `PrepSnapshotStore`
  - `SecureKeyStore`
  - `SecretStore`
- `*Rpc` for JSON-RPC surfaces
  - `WalletRpc`
  - `ChainRpc`
  - `TxRpc`
  - `AssetRpc`
- `*Verifier` for explicit verification boundaries
  - `TxProofVerifier`
  - `TxVerifier`
  - `ClaimTxVerifier`
- `*Resolver` for lookup or pre-state resolution seams
  - `InputResolver`
- `*Adapter` for external integration boundaries
  - `DaAdapter`
- `*Manager` for mutable orchestration components
  - `AddressManager`
  - `AsyncAddressManager`
- `*Index` for query/index abstractions
  - `SpentIndex`
  - `MemberIndex`
- `*Sink` / `*Source` for directional data flow boundaries
  - `MetricsSink`
  - `ConfigSource`

Trait names SHOULD be nouns or noun phrases. Public traits SHOULD NOT use
`Impl` naming.

#### Structs and Enums

Public types in Z00Z consistently use suffixes that reveal data shape,
lifecycle role, or error class.

Preferred suffixes:

- `*Error` for failure enums
    - `CompressionError`
    - `GenesisError`
    - `CryptoError`
    - `ChainServiceError`
- `*Config` / `*Settings` for configuration objects
    - `GenesisConfig`
    - `ChainConfig`
    - `YamlConfig`
    - `NodeConfig`
- `*Policy` / `*Rules` for enforcement or strategy definitions
    - `RotationPolicy`
    - `PolicyRules`
    - `SphinxPathPolicy`
- `*Request` / `*Response` / `*Params` / `*Status` for RPC and external I/O
    payloads
    - `PublicationRequest`
    - `RuntimeCreateWalletResponse`
    - `RuntimeRecoverFromSeedParams`
    - `RuntimeOperationStatus`
- `*State` / `*Snapshot` / `*Record` / `*Report` / `*Result` / `*Outcome` for
    computed or persisted views
    - `StageState`
    - `ObservationSnapshot`
    - `EvidenceRecord`
    - `ValidationReport`
    - `ScenarioResult`
    - `ProviderOutcome`
- `*Proof` / `*Bundle` / `*Root` / `*Id` for cryptographic and storage
    artifacts
    - `CheckpointProof`
    - `ClaimTxBundle`
    - `AssetStateRoot`

Types SHOULD remain domain-specific. Prefer `ValidationReport` over generic
names like `Report`, and `ObservationSnapshot` over generic names like
`StateData`.

#### RPC Type Naming

Wallet RPC types follow a very strong repository pattern and new RPC payloads
SHOULD match it exactly:

- `Runtime*Params` for request payloads
- `Runtime*Response` for response payloads
- `Runtime*Status` for lifecycle/status payloads
- `Persist*` for persisted or storage-facing data shapes

Examples:

- `RuntimeCreateWalletParams`
- `RuntimeCreateWalletResponse`
- `RuntimeStartScanResponse`
- `RuntimeJobStatus`
- `PersistWalletInfo`

This prefixing is important because it separates runtime transport types from
core domain types.

#### Functions and Methods

Function names are consistently `snake_case` and verb-led.

Preferred verb families:

- Construction: `build_*`, `create_*`, `new`, `derive_*`
- Validation: `validate_*`, `verify_*`, `detect_*`
- Retrieval: `get_*`, `list_*`, `lookup_*`, `resolve_*`
- State transition: `switch_to_*`, `start_*`, `clear_*`, `reset_*`
- Persistence and transport: `load_*`, `save_*`, `export_*`, `import_*`,
    `scan_*`

Examples:

- `calculate_fee`
- `validate_genesis_commitments_batch`
- `detect_chain_type`
- `create_dual_address`
- `scan_checkpoint`
- `get_rpc_endpoint`
- `switch_to_mainnet`

Boolean-returning functions SHOULD use `is_*` or `has_*`:

- `is_ok`
- `is_supported`
- `is_path_used`
- `has_asset`
- `has_ops`

#### Variables and Parameters

Local names are usually semantic noun phrases that reflect domain position, not
temporary mechanics.

Preferred patterns:

- State transitions: `prev_root`, `next_roots`, `target_height`
- Collections: `asset_batches`, `serial_batches`, `minimum_value_promises`
- Dependency handles: `time_provider`, `rng_provider`, `proof_chk`
- File/system paths: `config_path`, `log_path`, `proof_path`
- Stored views: `validation_report`, `alert_counts`, `path_batch`, `def_rows`

Variables SHOULD be specific enough to show role in the pipeline. Prefer:

- `next_roots` over `roots2`
- `config_path` over `path`
- `validation_report` over `report`
- `minimum_value_promises` over `values`

Short domain abbreviations are acceptable only when they are already stable and
ubiquitous in the repository, such as:

- `tx`
- `rpc`
- `rng`
- `id`
- `pk`
- `sk`
- `jmt`
- `ndef`
- `bip44`

New abbreviations SHOULD NOT be introduced if the full word is clearer.

#### Constants

Constants and statics follow `SCREAMING_SNAKE_CASE` and usually encode one of
these categories:

- limits and bounds
  - `RANGE_PROOF_BITS`
  - `MIN_VALUE_PROMISE`
- environment/configuration keys
  - `ALLOW_DEBUG_RANGE_PROOF`
- identity and seed fixtures
  - `BOB_SEED`
  - `SEND_SEED`
  - `ASSET_ID`

Constant names SHOULD encode the semantic dimension, not just the unit. Prefer
`MIN_VALUE_PROMISE` over `MIN_VALUE`, and `RANGE_PROOF_BITS` over `BITS`.

#### Tests

Z00Z has a strong and repeated test naming style:

File naming:

- all Rust test files MUST start with `test_`
    - this includes integration, scenario, module-local, suite-root, and
      test-only helper `.rs` files
    - `test_checkpoint_store_api.rs`
    - `test_claim_audit_log_integrity.rs`
    - `test_wallet_integration.rs`
- non-canonical names such as `*_tests.rs`, `*_test.rs`,
    `integration_tests.rs`, or bare `tests.rs` MUST be renamed to the
    `test_*.rs` form

Function naming:

- test functions SHOULD start with `test_`
- the suffix SHOULD describe behavior, rejection condition, or contract
    - `test_yaml_multi_doc_rejected`
    - `test_metrics_thread_safety`
    - `test_lock_bytes_zero_on_drop`
    - `test_bincode_trailing_bytes_rejected`

The preferred shape is:

- `test_<subject>_<behavior>`
- `test_<subject>_<condition>_<expected_outcome>`

#### Examples

Example file names are descriptive, workflow-oriented, and `snake_case`.

Common patterns:

- `*_demo` for illustrative examples
  - `logger_demo`
    - `workflow_demo`
    - `transport_demo`
- `quick_start_*` for onboarding examples
    - `quick_start_runtime`
- workflow or scenario naming
    - `scenario_runner`
    - `stage_flow`
    - `transfer_lane`
    - `bundle_workflow`

Examples SHOULD be named after the user-visible workflow or teaching goal, not
after internal implementation details.

#### Benches

Bench files are typically named with subject-first `snake_case`, often ending
in `_bench` or `_timing`.

Examples:

- `range_proof_bench.rs`
- `hash_policy_bench.rs`
- `wallet_service_bench.rs`
- `mac_timing.rs`

Bench functions SHOULD start with `bench_`:

- `bench_single_proof`
- `bench_batch_verify`
- `bench_registry_batch_insert`
- `bench_metadata_verify_hash`

The preferred shape is:

- `bench_<subject>`
- `bench_<subject>_<operation>`
- `bench_<subject>_<scenario>`

#### Module and File Naming

Modules and files SHOULD stay in `snake_case` and reflect one of these
repository-native roles:

- domain object
  - `transport.rs`
  - `alerts.rs`
  - `status_view.rs`
- boundary or seam
  - `da_adapter.rs`
  - `state_traits.rs`
- focused helper/type partition
  - `store_types.rs`
  - `stealth_request_types.rs`
- explicit test/support partition
  - `test_config.rs`
  - `test_metrics.rs`
  - `support.rs`

Prefer splitting by responsibility rather than using generic buckets like
`utils.rs`, `misc.rs`, or `helpers.rs` in domain crates.

#### Naming Decision Heuristics

When choosing between candidate names, prefer the name that answers these
questions most directly:

1. Is this a capability boundary, a data shape, or an operation?
2. Is the object persisted, computed, runtime-facing, or cryptographic?
3. Does the name reveal trust level or ownership boundary?
4. Would a new reader understand the role without reading the body?

If two names are both valid, choose the one that matches existing repository
families first.

#### Repository-Specific Anti-Patterns

Avoid introducing:

- generic names such as `ManagerImpl`, `Data`, `Helper`, `Util`, `Processor`
- ambiguous verb phrases such as `process_*`, `handle_*`, `do_*`, or `*_for`
- transport-shaped names inside core domain modules unless the type is truly
    RPC/runtime-facing
- new abbreviations that are not already established in Z00Z
- test names that only restate the function name without behavior

Repository naming SHOULD bias toward:

- precise domain nouns
- explicit capability suffixes
- verb-led operations
- behavior-led test names
- workflow-led example names
- benchmark names that encode measured subject and operation

**Tables:**

```markdown
| Column 1 | Column 2 | Column 3 |
|----------|----------|----------|
| Value A  | Value B  | Value C  |

```

**Lists:**

```markdown
- Item 1
- Item 2
  - Sub-item 2.1
  - Sub-item 2.2
- Item 3

```

---

## 🔒 Security Best Practices

### Cryptographic Keys

**Handling:**

- ✅ Use `Hidden<T>` wrapper for sensitive data (zeroizes on drop)
- ✅ Zeroize keys on drop (automatic with secure wrappers)
- ✅ Never log or print keys (even in debug mode)
- ❌ NEVER store keys in plaintext
- ❌ NEVER commit keys to git

**Universal Pattern:**

```rust
use crypto_lib::{SecretKey, Hidden};

// Wrap sensitive data
let secret_key = Hidden::hide(SecretKey::random(&mut rng));

// Access only when needed
let signature = secret_key.reveal().sign(message);

// Automatic zeroization on drop
drop(secret_key);  // Memory zeroized

```

**See also:** `crates/z00z_crypto/src/hidden.rs` - Secure wrapper for sensitive cryptographic data.

---

### Random Number Generation

**Requirements:**

- ✅ Use abstracted RNG provider interface
- ✅ Production: System-provided CSPRNG (cryptographically secure)
- ✅ Testing: Deterministic, seeded RNG for reproducibility
- ❌ NEVER use `rand::thread_rng()` directly in business logic
- ❌ NEVER use non-crypto RNG for keys/secrets

**Universal Pattern:**

```rust
// Production
let rng_provider = SystemRngProvider::default();
let secret = SecretKey::random(&mut rng_provider.rng());

// Testing (deterministic)
let rng_provider = MockRngProvider::with_seed(42);
let blinding = BlindingFactor::random(&mut rng_provider.rng());

```

---

### Input Validation

**Defense in Depth:**

1. **Schema Validation** - Reject malformed inputs
2. **Range Validation** - Ensure values within bounds
3. **Cryptographic Validation** - Verify proofs/signatures

**Example:**

```rust
pub fn process_transaction(tx: &Transaction) -> Result<()> {
    // Layer 1: Schema validation
    if tx.inputs.is_empty() {
        return Err(Error::NoInputs);
    }
    if tx.outputs.is_empty() {
        return Err(Error::NoOutputs);
    }

    // Layer 2: Range validation
    for output in &tx.outputs {
        if output.amount == 0 {
            return Err(Error::ZeroAmount);
        }
        if output.amount > MAX_AMOUNT {
            return Err(Error::AmountTooLarge);
        }
    }

    // Layer 3: Cryptographic validation
    tx.verify_signatures()?;
    tx.verify_range_proofs()?;

    // Layer 4: Business logic
    verify_balance(tx)?;

    Ok(())
}

```

---

### Error Messages

**Security Considerations:**

- ✅ Provide actionable error messages
- ✅ Include context (what operation failed)
- ❌ NEVER leak sensitive data in errors
- ❌ NEVER expose internal paths/structure

**Example:**

```rust
// ✅ Good: Actionable, no sensitive data
return Err(Error::InvalidSignature {
    key_id: public_key.fingerprint(),  // Public identifier only
});

// ❌ Bad: Leaks secret key
return Err(Error::InvalidSignature {
    secret_key: secret_key.to_string(),  // SECURITY VIOLATION
});

// ✅ Good: Generic database error
return Err(Error::DatabaseError);

// ❌ Bad: Exposes internal structure
return Err(Error::DatabaseError {
    query: "SELECT * FROM users WHERE password = ...",  // Leaks schema
});

```

---

## 📖 References

### Key Documents

- **Section 1 in this document** - Complete ONE SOURCE OF TRUTH architecture and approved abstraction patterns
- **`crates/Z00Z_UTILS_MODULE_MAP.md`** - z00z_utils module reference
- **`crates/Z00Z_UTILS_QUICK_REFERENCE.md`** - Quick API reference
- **`.github/requirements/Tari-Crypto-Integration-Z00Z.md`** - Tari crypto integration guide
- **`.github/requirements/Tari-Crypto-Components-Cookbook.md`** - Tari crypto API reference
- **`.github/github-copilot-instructions.md`** - GitHub Copilot configuration
- **`.github/skills/z00z-git-versioning/scripts/VERSION_MANAGEMENT.md`** - Version management guide

### Module Documentation

- **`crates/z00z_core/src/genesis/GENESIS_DOCUMENTATION.md`** - Genesis module reference
- **`crates/z00z_core/examples/genesis/README.md`** - Genesis examples
- **`crates/z00z_core/benches/genesis/README.md`** - Genesis benchmarks
- **`crates/z00z_core/tests/genesis/README.md`** - Genesis test suite

### External References

- **Rust Book:** [https://doc.rust-lang.org/book/](https://doc.rust-lang.org/book/)
- **Rust API Guidelines:** [https://rust-lang.github.io/api-guidelines/](https://rust-lang.github.io/api-guidelines/)
- **Tari Crypto:** [https://github.com/tari-project/tari-crypto](https://github.com/tari-project/tari-crypto)
- **Criterion Benchmarking:** [https://bheisler.github.io/criterion.rs/book/](https://bheisler.github.io/criterion.rs/book/)

---

## ✅ Compliance Checklist

Before submitting code for review, verify:

- [ ] **English Only** - All technical content in English
- [ ] **ONE SOURCE OF TRUTH** - No direct stdlib I/O/time/serialization in business logic
- [ ] **Trait Injection** - All dependencies injected via traits
- [ ] **Domain Separation** - All crypto operations use domain-separated hashing
- [ ] **Vendor Isolation** - No modifications to vendored code directories
- [ ] **Parallelism** - Correct model chosen (rayon for CPU, tokio for I/O)
- [ ] **Error Handling** - Structured errors with `thiserror`, no `unwrap()` in production
- [ ] **Documentation** - All public items documented with examples
- [ ] **Testing** - Unit tests + integration tests + doc tests
- [ ] **Formatting** - `cargo fmt` applied
- [ ] **Linting** - `cargo clippy` with zero warnings
- [ ] **Benchmarks** - Performance benchmarks for critical paths (use `criterion`)
- [ ] **Version Management** - Semantic versioning for API changes

---

## 📝 Document Maintenance

**Version History:**

| Version | Date | Changes |
| ------- | ---- | ------- |
| 1.0.0 | 2025-01-19 | Initial architectural foundation document |
| 1.1.0 | 2025-12-10 | Added universal design patterns: Snapshot & Synchronization, Wire Format & Protocol Versioning, Global Singleton with Lazy Initialization, Bitflag Configuration, Test Fixtures Organization, Advanced Testing Patterns (crypto testing, multi-dependency injection, property-based testing, coverage-driven expansion) |

**Review Cycle:** This document MUST be reviewed and updated:

- After major architectural changes
- When new patterns emerge
- Quarterly (minimum)

**Change Process:**

1. Propose changes via pull request
2. Require architecture review + approval
3. Update version number (semver)
4. Announce changes to team

---

## 📝 Changelog

### Version 1.2.0 (2025-12-10) - Universal Patterns Refactoring

**Major Changes:**

- 🔄 **Document Purpose:** Transformed from Z00Z-specific constitution to universal design patterns guide
- ✨ **Pattern Generalization:** Refactored all code examples from concrete Z00Z implementations to abstract universal patterns
- 📚 **10 Universal Patterns:** Added comprehensive pattern catalog applicable to any Rust blockchain/crypto project
- 🔗 **Z00Z as Example:** Repositioned Z00Z references as "See also" implementation examples
- ⚡ **NEW: Principle 7 - Parallelism & Concurrency:** Comprehensive guide for choosing rayon (CPU-bound) vs tokio (I/O-bound) with hybrid patterns

**Refactored Sections:**

1. **Testing Principle:** Removed mocks/stubs, emphasized real implementations with deterministic data
2. **Trait-Based DI:** Generalized from `TimeProvider`/`RngProvider` to abstract `ExternalService` traits
3. **Policy-State Separation:** Universal pattern with memory efficiency calculations
4. **Deterministic Initialization:** Generic `DeterministicBuilder<T>` replacing Genesis-specific code
5. **Unique Identifier:** Abstract `UniqueId` pattern with multiple generation strategies
6. **Validation Layering:** Schema → Business → Crypto stages with cost-ordering
7. **Configuration Management:** Layered config pattern (File + ENV overrides)
8. **Benchmarking:** Generic crypto operations instead of `PedersenCommitmentFactory`
9. **Domain Separation:** Abstract hash domain pattern
10. **Vendor Isolation:** Generic vendor directory pattern

**New Content:**

- **Parallelism & Concurrency Principle:** CPU-bound (rayon) vs I/O-bound (tokio) decision matrix with patterns
- **Hybrid Patterns:** `tokio::task::spawn_blocking` for CPU work in async contexts
- **Anti-Patterns:** Common mistakes (blocking tokio runtime, async overhead for CPU)

**Project-Agnostic Improvements:**

- Document title changed to "Design Foundation for Rust Blockchain Projects"
- All enforcement commands generalized (e.g., `your_core` placeholders instead of `z00z_core`)
- Compliance checklist made universal
- References to "Z00Z development" changed to "battle-tested patterns"

**Removed:** Z00Z-specific implementations from pattern definitions (moved to "See also" references)

**Added:** Pattern catalog introduction with 10 universal patterns overview

---

### Version 1.1.0 (2025-12-09) - Pattern Integration

**Added:**

- Snapshot & Synchronization Pattern
- Wire Format & Protocol Versioning Pattern
- Global Singleton with Lazy Initialization Pattern
- Bitflag Configuration Pattern
- Test Fixtures Organization Pattern
- Advanced Testing Patterns section

---

### Version 1.0.0 (2025-12-08) - Initial Foundation

**Created:** Architectural constitution with core principles, testing philosophy, and Z00Z-specific patterns

---

## 🎓 Conclusion

🎯 **Purpose:** This document establishes universal architectural patterns for Rust blockchain and cryptographic systems, with Z00Z serving as a reference implementation demonstrating best practices.

📌 **Authority:** These principles represent battle-tested patterns applicable across projects. Z00Z development adheres strictly to these standards.

💫 **Evolution:** As patterns emerge and improve, this document evolves. Contribute refinements via pull request.

✅ **Applicability:** Use these patterns in any Rust project requiring strong testing, deterministic behavior, or cryptographic operations.

🔔 **Questions?** Consult pattern definitions first, review Z00Z implementation examples second, then seek architecture guidance.
