# Z00Z Project - GitHub Copilot Instructions

**Version:** 2.0  
**Last Updated:** 2025-12-09  
**Project:** Z00Z - Privacy-focused blockchain with confidential transactions

---

## 🎯 Core Principles (@MUST reference)

### 1. English-Only Policy

**MANDATORY:** All code, comments, documentation, commit messages, and technical content MUST be written exclusively in English.

**Applies to:**
- ✅ Source code and inline comments
- ✅ API documentation and README files
- ✅ Commit messages and pull request descriptions
- ✅ Technical specifications and architecture documents
- ✅ Error messages and logging output
- ✅ Configuration files and environment variables

**Rationale:** Ensures consistency, maintainability, and accessibility for international development teams.

---

### 2. Safe File Operations

**CRITICAL:** NEVER use destructive deletion commands without explicit user confirmation.

**On Linux:**
- ✅ Use `trash-put <path>` (trash-cli package)
- ✅ Use `gio trash <path>` (GNOME utilities)
- ❌ NEVER use `rm -rf`
- ❌ NEVER use `rm -r`
- ❌ NEVER use `rm` with wildcards (`rm *.tmp`) without confirmation

**If trash utilities unavailable:** Ask user for preferred safe delete method.

---

### 3. Version Management

**MANDATORY:** Use `./.github/skills/z00z-git-versioning/scripts/version-manager.sh` for ALL version updates and git operations.

**Commands:**
```bash
# Increment versions
./.github/skills/z00z-git-versioning/scripts/version-manager.sh patch -m "Bug fix description"
./.github/skills/z00z-git-versioning/scripts/version-manager.sh minor -m "New feature description"
./.github/skills/z00z-git-versioning/scripts/version-manager.sh major -m "Breaking change description"

# Update specific crate
./.github/skills/z00z-git-versioning/scripts/version-manager.sh crate <name> <version> -m "Update description"

# Sync to GitHub (force push, NOT PR)
./.github/skills/z00z-git-versioning/scripts/version-manager.sh sync -f -b <current-branch>
```

**Automatic actions:**
- Updates `versions.yaml` with new version
- Creates git commit with conventional format
- Creates version tag (`vX.Y.Z`)
- Pushes to GitHub on specified branch

**See:** `.github/skills/z00z-git-versioning/scripts/VERSION_MANAGEMENT.md` for complete documentation

---

### 4. Protected Directories

**CRITICAL:** The `z00z_crypto/tari/` directory is READ-ONLY.

- ✅ Can compile and use Tari libraries
- ✅ Can export new functionality through `z00z_crypto/src/lib.rs`
- ❌ NEVER modify source files in `tari/` subdirectories

**Rationale:** Maintains clean vendor boundary and simplifies upstream updates.

---

### 5. User Interaction Signals

**MANDATORY:** At the end of each Copilot cycle, when user interaction is expected, execute:

```bash
./scripts/play_tone.sh
```

**Purpose:** Audible cue signals that AI has completed work and is awaiting user input.

---

### 6. Documentation Standards

**MANDATORY:** Use emojis at the beginning of markdown paragraphs for improved readability.

**Suggested emojis:** 📌🎯⏰💥⚙️🔑♨️⭐👍👎☢️🚫💯💫👁️‍🗨️🚨🛑🔔🚩⚠️⛔✅❌‼️❓

**Date format:** ISO 8601 (`YYYY-MM-DD`)

---

## 🏗️ Architecture Guidelines

### ONE SOURCE OF TRUTH Principle

**Core concept:** Each low-level operation (file I/O, serialization, time, logging) MUST have ONE centralized abstraction layer in `z00z_utils`.

#### Architecture Layers

```
┌─────────────────────────────────────────┐
│    z00z_core (Business Logic)           │
│    ❌ NO direct std::fs                 │
│    ❌ NO direct serde_json/yaml         │
│    ❌ NO direct SystemTime              │
└─────────────┬───────────────────────────┘
              ↓ uses abstractions
┌─────────────────────────────────────────┐
│    z00z_utils (Abstraction Layer)       │
│    ✅ io::*, codec::*, time::*          │
└─────────────┬───────────────────────────┘
              ↓ wraps
┌─────────────────────────────────────────┐
│    std::fs, serde_*, SystemTime         │
└─────────────────────────────────────────┘
```

#### File I/O

```rust
// ❌ WRONG
use std::fs;
fs::write("data.json", json)?;

// ✅ CORRECT
use z00z_utils::io::write_file;
write_file("data.json", &json)?;
```

**Available functions:**
- `write_file(path, data)` - Atomic write with temp+rename
- `read_file(path)` - Read bytes from file
- `read_to_string(path)` - Read file as UTF-8 string
- `remove_file(path)` - Delete file
- `rename_file(from, to)` - Rename/move file
- `create_dir_all(path)` - Create directory (recursive)

**Exception:** `std::fs::File` for streaming operations (ZIP, large files) is acceptable.

#### Serialization

```rust
// ❌ WRONG
use serde_json;
let json = serde_json::to_string(&data)?;

// ✅ CORRECT
use z00z_utils::codec::{Codec, JsonCodec};
let codec = JsonCodec;
let json_bytes = codec.serialize(&data)?;
```

**Available codecs:**
- `JsonCodec` - JSON serialization (replaces serde_json)
- `YamlCodec` - YAML serialization (replaces serde_yaml)
- `BincodeCodec` - Binary serialization

**Note:** For slice types, use `&reference`:
```rust
let bytes = codec.serialize(&assets)?;  // &[Asset] needs &
```

#### YAML Configuration

```rust
// ❌ WRONG
use serde_yaml::Value;
let policy: Value = config.get("policy")?;

// ✅ CORRECT
use z00z_utils::config::{YamlValue, from_yaml_value};
let policy: YamlValue = config.get("policy")?;
let parsed = from_yaml_value(policy)?;
```

**Key changes:**
- `serde_yaml::Value` → `z00z_utils::config::YamlValue`
- `serde_yaml::from_value()` → `z00z_utils::config::from_yaml_value()`

#### Time Operations

```rust
// ❌ WRONG
use std::time::{SystemTime, UNIX_EPOCH};
let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

// ✅ CORRECT
use z00z_utils::time::{TimeProvider, RealTime};
let time_provider = RealTime;
let now = time_provider.unix_timestamp();
```

**Benefits:**
- Mockable in tests (`MockTimeProvider`)
- Consistent error handling
- Injectable fake time for testing

#### Verification Commands

```bash
# Check for violations
grep -rn "use std::fs" crates/z00z_core/src/ | grep -v "std::fs::File"
# Expected: 0 matches

grep -rn "serde_yaml::Value" crates/z00z_core/src/
# Expected: 0 matches

grep -rn "serde_json::" crates/z00z_core/src/
# Expected: 0 matches
```

**See:** `crates/Z00Z_DESIGN_FOUNDATION.md` (Section 1: ONE SOURCE OF TRUTH) for the complete architecture guide

---

## 🔐 Tari Crypto Integration

### Exported Components

**Core Cryptography:**
- `PedersenCommitmentFactory` - Hide transaction amounts
- `CommitmentSignature` (RistrettoComSig) - Zero-knowledge payment proofs
- `RistrettoSchnorr` - Transaction authorization
- `BulletproofsPlusService` - Range proofs for confidential amounts

**Advanced Features:**
- `DhKeyExchange` - OnionNet secure channels
- `DerivedKeyDomain` - HD wallet key generation
- `hash_domain!` macro - Prevent cross-protocol attacks

**Security Tools:**
- `Hidden<T>` - Prevent sensitive data leaks
- `SafePassword` - Wallet encryption keys

**Developer Tools:**
- `to_hex()`, `from_hex()` - Hex encoding utilities
- `ByteArray` trait - Generic byte operations

### Usage Example

```rust
use z00z_crypto::{
    PedersenCommitmentFactory, HomomorphicCommitmentFactory,
    BulletproofsPlusService, RistrettoSecretKey, Hidden
};

// Hide sensitive data
let blinding = Hidden::hide(RistrettoSecretKey::random(&mut rng));

// Create commitment
let factory = PedersenCommitmentFactory::default();
let commitment = factory.commit_value(blinding.reveal(), 1000u64);

// Create range proof
let proof_service = BulletproofsPlusService::default();
let proof = proof_service.construct_proof(blinding.reveal(), 1000u64)?;
```

**See:** `crates/Tari Crypto Integration for Z00Z.md` and `crates/Tari Crypto Components Cookbook.md` for complete reference

---

## 🦀 Rust Standards

### Code Style

**Follow Rust 2021 edition idioms:**
- Run `cargo fmt` before commits
- Run `cargo clippy --all-targets --all-features` with zero warnings
- Run `cargo test --all` - all tests must pass

**Naming conventions:**
- Types/structs: `PascalCase` (nouns) - `Parser`, `Engine`
- Functions/methods: `snake_case` (verbs) - `parse`, `encode`
- Booleans: `is_` or `has_` prefix - `is_ready`, `has_signature`
- Constants: `SCREAMING_SNAKE_CASE` - `DEFAULT_TIMEOUT`

### Error Handling

```rust
// Use thiserror for error enums
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid header: {0}")]
    InvalidHeader(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
```

**Rules:**
- ✅ Prefer `Result<T, E>` with typed errors
- ✅ Use `thiserror` for error definitions
- ❌ NEVER use `unwrap()` in production code
- ❌ NEVER use `expect()` unless safety is documented
- ✅ Provide meaningful error messages

### Public API Design

**Principles:**
- Small, stable surfaces
- Builder pattern for configurable types
- Return interfaces (traits), hide implementations
- Do NOT leak dependency types into public API

**Example:**
```rust
pub struct Engine { /* private fields */ }

impl Engine {
    pub fn builder() -> EngineBuilder { /* ... */ }
    
    pub fn process<I: IntoIterator<Item = u8>>(
        &mut self, 
        data: I
    ) -> Result<Stats, Error> {
        // ...
    }
}
```

### Module Organization

**Guidelines:**
- Keep module depth to 1-2 levels maximum
- Internal items: `crate` or `pub(crate)` visibility
- Public items: export only through `src/lib.rs` facade
- Avoid cyclic dependencies: use `common/` for shared types

**Structure:**
```
crate/
├── lib.rs              (Public facade - re-exports only)
├── common/             (Shared types)
├── module_a/
│   ├── mod.rs          (Module facade)
│   ├── types.rs        (Public types)
│   ├── impl.rs         (Private implementation)
│   └── tests.rs        (Unit tests)
```

### Cargo.toml & Features

**Best practices:**
- Explicitly document MSRV (Minimum Supported Rust Version)
- Mark optional dependencies via `features`
- Split heavy capabilities: `serde`, `rayon`, `tokio`, `simd`

**Example:**
```toml
[features]
default = ["std"]
std = []
serde = ["dep:serde", "dep:serde_json"]
async = ["dep:tokio"]
```

### Testing Strategy

**Test types:**
1. **Unit tests** - Next to code in `mod tests {}`
   - Test invariants and edge cases
   - Cover error conditions
   
2. **Integration tests** - In `tests/*.rs`
   - Test through public API only
   - Realistic scenarios
   
3. **Doc tests** - In `///` documentation
   - Minimum: happy path + error example
   - Ensure examples compile and run
   
4. **Property tests** (optional) - Use `proptest`
   - Test invariants across random inputs
   - Round-trip tests (serialize/deserialize)

**Example:**
```rust
/// Parse configuration from YAML
///
/// # Examples
///
/// ```
/// use z00z_core::Config;
///
/// let config = Config::from_yaml("config.yaml")?;
/// assert_eq!(config.version(), 1);
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
pub fn from_yaml(path: &str) -> Result<Config, Error> {
    // ...
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_yaml_valid() {
        let config = Config::from_yaml("fixtures/valid.yaml").unwrap();
        assert_eq!(config.version(), 1);
    }

    #[test]
    fn test_from_yaml_invalid() {
        let result = Config::from_yaml("fixtures/invalid.yaml");
        assert!(result.is_err());
    }
}
```

### Safety & Performance

**Safety:**
- `#![forbid(unsafe_code)]` by default
- If `unsafe` is needed:
  - Localize in smallest possible scope
  - Document safety invariants in comments
  - Prove why unsafe code is correct

**Performance:**
- Use Criterion for benchmarks
- "Hot paths" - avoid allocations where possible
- Accept `&[u8]`/`&str`/iterators instead of owned types
- Use `Cow`, `SmallVec`, arena allocators when appropriate

**Example benchmark:**
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_parse(c: &mut Criterion) {
    c.bench_function("parse_transaction", |b| {
        let data = generate_test_data();
        b.iter(|| parse_transaction(black_box(&data)))
    });
}

criterion_group!(benches, benchmark_parse);
criterion_main!(benches);
```

### Documentation Requirements

**Required documentation:**
- `#![doc = include_str!("../README.md")]` in `lib.rs`
- Each public item: `///` doc comment with example
- `cargo doc --no-deps` must build without warnings
- README: purpose, quick-start, MSRV, features, semver guarantees

**Example:**
```rust
/// Transaction builder for creating confidential transfers.
///
/// # Examples
///
/// ```
/// use z00z_core::{TransactionBuilder, Asset};
///
/// let tx = TransactionBuilder::new()
///     .sender(sender_key)
///     .recipient(recipient_key)
///     .amount(1000)
///     .asset(Asset::default())
///     .build()?;
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
pub struct TransactionBuilder { /* ... */ }
```

### Versioning

**Semantic Versioning (Strict):**
- `MAJOR` - Breaking changes to public API
- `MINOR` - New features (backward compatible)
- `PATCH` - Bug fixes (backward compatible)

**Deprecation process:**
1. Mark old API with `#[deprecated(since = "X.Y.Z", note = "Use new_api instead")]`
2. Document migration path
3. Remove in next MAJOR version

**Example:**
```rust
#[deprecated(since = "1.5.0", note = "Use `process_transaction` instead")]
pub fn process_tx(tx: Transaction) -> Result<Receipt, Error> {
    process_transaction(tx)
}
```

---

## 🧰 Z00Z Utils Reference

### Module Map

**Available modules:**
- `codec` - Serialization abstraction (JsonCodec, YamlCodec, BincodeCodec)
- `config` - Configuration sources (EnvConfig, YamlConfig, LayeredConfig)
- `io` - File I/O operations (atomic writes, directory creation)
- `logger` - Logging abstraction (TracingLogger, NoopLogger, StdoutLogger)
- `metrics` - Metrics sink (NoopMetrics, future: PrometheusMetrics)
- `rng` - Random number generation (SystemRngProvider, MockRngProvider)
- `time` - Time provider (SystemTimeProvider, MockTimeProvider)

### Quick Reference Examples

**File I/O:**
```rust
use z00z_utils::io::{save_json, load_json};

// Save/load with automatic codec selection
save_json("data.json", &my_data)?;
let data: MyType = load_json("data.json")?;
```

**Time (production):**
```rust
use z00z_utils::time::{TimeProvider, SystemTimeProvider};

let provider = SystemTimeProvider::default();
let now = provider.unix_timestamp();
```

**Time (testing):**
```rust
use z00z_utils::time::MockTimeProvider;
use std::time::{SystemTime, Duration};

let provider = MockTimeProvider::new(SystemTime::now());
provider.advance_by(Duration::from_secs(60));  // Simulate time passing
```

**RNG (production):**
```rust
use z00z_utils::rng::{RngProvider, SystemRngProvider};
use rand::RngCore;

let provider = SystemRngProvider::default();
let mut rng = provider.rng();
rng.fill_bytes(&mut buffer);
```

**RNG (testing):**
```rust
use z00z_utils::rng::MockRngProvider;

let provider = MockRngProvider::with_seed(42);  // Deterministic
let mut rng = provider.rng();
```

**Config:**
```rust
use z00z_utils::config::{ConfigSource, LayeredConfig, YamlConfig, EnvConfig};

// YAML + ENV override
let yaml = YamlConfig::from_file("config.yaml")?;
let env = EnvConfig;
let config = LayeredConfig::new(yaml, env);

// Get values (ENV overrides YAML)
let value: String = config.get("database.url")?;
```

**See:** `crates/Z00Z_UTILS_MODULE_MAP.md` and `crates/Z00Z_UTILS_QUICK_REFERENCE.md` for complete reference

---

## 📋 Project Structure

```
z00z/
├── Cargo.toml                      (Workspace root)
├── crates/
│   ├── z00z_core/                  (Core business logic)
│   │   ├── src/
│   │   │   ├── assets/             (Asset management)
│   │   │   ├── genesis/            (Genesis block)
│   │   │   ├── tx/                 (Transactions)
│   │   │   └── lib.rs              (Public facade)
│   │   └── tests/                  (Integration tests)
│   │
│   ├── z00z_crypto/                (Cryptographic primitives)
│   │   ├── src/lib.rs              (Public exports)
│   │   └── tari/                   (⚠️ READ-ONLY vendor code)
│   │
│   ├── z00z_utils/                 (Abstraction layer)
│   │   └── src/
│   │       ├── codec/              (Serialization)
│   │       ├── config/             (Configuration)
│   │       ├── io/                 (File I/O)
│   │       ├── logger/             (Logging)
│   │       ├── metrics/            (Metrics)
│   │       ├── rng/                (Random numbers)
│   │       └── time/               (Time provider)
│   │
│   ├── z00z_wallets/               (Wallet implementations)
│   ├── z00z_rollup_node/           (Rollup node)
│   ├── z00z_da_celestia/           (Celestia DA layer)
│   ├── z00z_storage/               (Persistent storage)
│   ├── z00z_telemetry/             (Observability)
│   └── z00z_runtime/               (Runtime environment)
│
├── docs/                           (Design documents)
├── scripts/                        (Build and utility scripts)
│   └── play_tone.sh                (User interaction signal)
├── .github/skills/z00z-git-versioning/scripts/
│   ├── version-manager.sh          (⚠️ Use for all versioned git ops)
│   └── VERSION_MANAGEMENT.md       (Version management guide)
│
└── tests/                          (Workspace-level integration tests)
```

### Module Boundaries

**Clear separation of concerns:**
- `z00z_core` - Pure business logic, NO I/O, NO external dependencies
- `z00z_crypto` - Cryptographic operations, exports Tari components
- `z00z_utils` - Abstraction layer for I/O, time, config, logging
- `z00z_wallets` - Wallet UX, key management, transaction building
- `z00z_storage` - Database, persistence, state management
- `z00z_rollup_node` - Network, consensus, block production

**Dependency flow:**
```
z00z_wallets → z00z_core → z00z_crypto
     ↓              ↓            ↓
          z00z_utils (foundation)
```

---

## 🎯 Common Tasks

### Task 1: Implement New Feature

**Process:**
1. Read existing patterns in relevant crate
2. Design (if non-trivial, discuss with user first)
3. Implement with TDD approach:
   - Write failing test
   - Implement feature
   - Verify test passes
4. Add documentation (doc comments + examples)
5. Update README if public API changed
6. Run quality checks:
   ```bash
   cargo fmt
   cargo clippy --all-targets --all-features
   cargo test --all
   cargo doc --no-deps
   ```

### Task 2: Refactor Code

**Process:**
1. Identify responsibilities to extract
2. Write tests to guard current behavior
3. Perform behavior-preserving refactors
4. Verify tests still pass
5. Keep public API stable (or clearly document breaking changes)
6. Remove dead code and unused abstractions

**Example:**
```rust
// Before: Complex function
fn process(data: &[u8]) -> Result<Output, Error> {
    // 200 lines of mixed concerns
}

// After: Extracted responsibilities
fn process(data: &[u8]) -> Result<Output, Error> {
    let parsed = parse_input(data)?;
    let validated = validate(parsed)?;
    let transformed = transform(validated)?;
    Ok(Output::from(transformed))
}

fn parse_input(data: &[u8]) -> Result<Input, Error> { /* ... */ }
fn validate(input: Input) -> Result<Validated, Error> { /* ... */ }
fn transform(validated: Validated) -> Transformed { /* ... */ }
```

### Task 3: Add Tests

**Coverage priorities:**
1. **Happy path** - Normal operation
2. **Error cases** - Invalid input, I/O errors
3. **Edge cases** - Empty input, max values, boundary conditions
4. **Concurrent behavior** - Race conditions, thread safety (if applicable)

**Example:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_transaction() {
        let tx = parse_transaction(&valid_data()).unwrap();
        assert_eq!(tx.amount, 1000);
    }

    #[test]
    fn test_parse_invalid_signature() {
        let result = parse_transaction(&invalid_sig_data());
        assert!(matches!(result, Err(Error::InvalidSignature)));
    }

    #[test]
    fn test_parse_empty_input() {
        let result = parse_transaction(&[]);
        assert!(result.is_err());
    }
}
```

### Task 4: Update Documentation

**When to update:**
- Public API added/changed/removed
- Breaking changes
- New features
- Architecture changes

**What to update:**
- Doc comments (`///`)
- README.md
- CHANGELOG.md (if exists)
- Migration guides (for breaking changes)

---

## ❌ Anti-Patterns to Avoid

### Don't: Mix Concerns

```rust
// ❌ WRONG: HTTP handler in core business logic
// z00z_core/src/assets/registry.rs
pub fn create_asset_http(req: HttpRequest) -> HttpResponse {
    // parsing HTTP, validation, business logic all mixed
}

// ✅ CORRECT: Separate concerns
// z00z_api/src/handlers/assets.rs
pub fn create_asset_handler(req: HttpRequest) -> HttpResponse {
    let params = parse_request(req)?;
    let asset = z00z_core::assets::create_asset(params)?;
    Response::json(asset)
}
```

### Don't: Bypass Abstractions

```rust
// ❌ WRONG: Direct std::fs in z00z_core
use std::fs;
fs::write("state.json", data)?;

// ✅ CORRECT: Use z00z_utils abstraction
use z00z_utils::io::write_file;
write_file("state.json", &data)?;
```

### Don't: Expose Implementation Details

```rust
// ❌ WRONG: Leaking dependency types
pub fn parse_config(value: serde_yaml::Value) -> Result<Config, Error>;

// ✅ CORRECT: Use wrapper types
use z00z_utils::config::YamlValue;
pub fn parse_config(value: YamlValue) -> Result<Config, Error>;
```

### Don't: Overgeneralize

```rust
// ❌ WRONG: Excessive generics
pub fn process<T, U, V, W>(
    input: T,
    processor: U,
    validator: V,
    serializer: W
) -> Result<Vec<u8>, Error>
where
    T: Input + Clone + Debug,
    U: Processor<T> + Send + Sync,
    V: Validator<U::Output>,
    W: Serializer<V::Output>,
{
    // ...
}

// ✅ CORRECT: Concrete types with clear purpose
pub fn process_transaction(tx: Transaction) -> Result<ProcessedTx, Error> {
    // ...
}
```

### Don't: Ignore Errors

```rust
// ❌ WRONG
let config = load_config().unwrap();  // Can panic!
let data = parse(bytes).expect("failed");  // Unclear error

// ✅ CORRECT
let config = load_config()
    .map_err(|e| Error::ConfigLoad(e))?;
let data = parse(bytes)?;  // Propagate error
```

---

## 🔒 Security Guidelines

### Input Validation

**Always validate and sanitize:**
- User input (CLI args, API requests)
- File contents (configs, transactions)
- Network data (P2P messages, RPC calls)

**Example:**
```rust
pub fn set_amount(&mut self, amount: u64) -> Result<(), Error> {
    if amount == 0 {
        return Err(Error::InvalidAmount("Amount must be positive"));
    }
    if amount > MAX_AMOUNT {
        return Err(Error::InvalidAmount("Amount exceeds maximum"));
    }
    self.amount = amount;
    Ok(())
}
```

### Cryptography

**Rules:**
- ✅ Use Tari crypto primitives (already audited)
- ❌ NEVER implement custom crypto algorithms
- ❌ NEVER roll your own random number generation
- ✅ Use `Hidden<T>` for sensitive data (keys, seeds)
- ✅ Use `SafePassword` for user passwords

**Example:**
```rust
use z00z_crypto::{Hidden, RistrettoSecretKey};

// ✅ CORRECT: Hide sensitive key
let secret = Hidden::hide(RistrettoSecretKey::random(&mut rng));

// Use reveal() only when necessary
let signature = sign_message(secret.reveal(), &message);

// secret is automatically zeroed on drop
```

### Sensitive Data Handling

**Guidelines:**
- Use `Hidden<T>` wrapper for secrets
- Zero memory on drop (use `zeroize` crate)
- Avoid logging secrets
- Avoid including secrets in error messages
- Use constant-time comparison for secrets

**Example:**
```rust
use z00z_crypto::Hidden;
use zeroize::Zeroize;

#[derive(Zeroize)]
#[zeroize(drop)]
struct WalletSeed([u8; 32]);

impl WalletSeed {
    pub fn new(seed: [u8; 32]) -> Hidden<Self> {
        Hidden::hide(WalletSeed(seed))
    }
}

// Automatically zeroed on drop
```

---

## 🧪 Quality Checklist

### Before Submitting PR

**Code quality:**
- [ ] `cargo fmt` applied
- [ ] `cargo clippy` passes with zero warnings
- [ ] `cargo test --all` passes
- [ ] `cargo doc --no-deps` builds without warnings

**Testing:**
- [ ] Unit tests added for new functionality
- [ ] Integration tests updated if public API changed
- [ ] Doc tests compile and run
- [ ] Error cases covered

**Documentation:**
- [ ] Public items have `///` doc comments with examples
- [ ] README updated if needed
- [ ] Migration guide for breaking changes

**Architecture:**
- [ ] Follows ONE SOURCE OF TRUTH principle
- [ ] No `std::fs` in `z00z_core` (use `z00z_utils::io`)
- [ ] No `serde_json`/`serde_yaml` in `z00z_core` (use `z00z_utils::codec`)
- [ ] No `SystemTime` in `z00z_core` (use `z00z_utils::time`)
- [ ] No modifications to `z00z_crypto/tari/` directory

**Git:**
- [ ] Commit messages in English
- [ ] Used `./.github/skills/z00z-git-versioning/scripts/version-manager.sh` for version updates
- [ ] Executed `./scripts/play_tone.sh` at end of cycle

---

## 📚 Additional Resources

### Internal Documentation

- `crates/Z00Z_DESIGN_FOUNDATION.md` - Design foundation, including the ONE SOURCE OF TRUTH architecture guide
- `crates/Tari Crypto Integration for Z00Z.md` - Crypto components overview
- `crates/Tari Crypto Components Cookbook.md` - Detailed crypto API reference
- `crates/Z00Z_UTILS_MODULE_MAP.md` - Complete z00z_utils structure
- `crates/Z00Z_UTILS_QUICK_REFERENCE.md` - Quick examples for common tasks
- `.github/skills/z00z-git-versioning/scripts/VERSION_MANAGEMENT.md` - Version management workflow
- `.github/github-copilot-instructions.md` - Project-wide Copilot instructions

### External Resources

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust Design Patterns](https://rust-unofficial.github.io/patterns/)
- [The Rustonomicon](https://doc.rust-lang.org/nomicon/) - Unsafe Rust
- [Cargo Book](https://doc.rust-lang.org/cargo/)

---

## 🎓 Definition of Done

A task is **complete** when:

✅ **Functionality** - Feature works as specified  
✅ **Tests** - Unit tests + integration tests pass  
✅ **Documentation** - Public API documented with examples  
✅ **Quality** - `fmt`, `clippy`, `test` all pass  
✅ **Architecture** - Follows ONE SOURCE OF TRUTH  
✅ **Security** - Input validated, sensitive data protected  
✅ **Git** - Proper commit message, version updated if needed  
✅ **Signal** - `play_tone.sh` executed at end of cycle

---

## 🤖 How to Use These Instructions

**For Copilot:**
1. Read and internalize these guidelines
2. Apply them consistently to all code and documentation
3. When uncertain, ask user for clarification
4. Prioritize correctness over cleverness
5. Prefer small, focused changes over large refactors

---

## 📚 Pattern Quick Links

| Pattern | Line # in DESIGN_FOUNDATION.md | Use When |
|---------|-------------------------------|----------|
| Policy-State Separation | ~920 | Entities with shared config |
| Deterministic Initialization | ~1015 | Genesis blocks, reproducible setup |
| Unique Identifier | ~1148 | Generating IDs/nonces |
| Validation Layering | ~1280 | Multi-stage validation |
| Configuration Management | ~1376 | YAML + ENV config |
| Parallelism (rayon/tokio) | ~395 | Choosing concurrency model |

**Open specific pattern:**
```bash
# Jump to line in VS Code
code -g crates/Z00Z_DESIGN_FOUNDATION.md:920
```

---

## 🚨 When to Read Full Design Foundation

**Read full document (`crates/Z00Z_DESIGN_FOUNDATION.md`) when:**
- Adding new architectural patterns
- Unsure which principle applies
- Implementing crypto operations
- Designing new module structure
- Code review finds violations

**Use semantic search:**
```bash
# Search for specific patterns
rg "Policy-State Separation" crates/Z00Z_DESIGN_FOUNDATION.md -A 30
rg "Deterministic Initialization" crates/Z00Z_DESIGN_FOUNDATION.md -A 30
rg "Parallelism" crates/Z00Z_DESIGN_FOUNDATION.md -A 30
```

---

## 🤖 For AI Assistants

**When generating code for Z00Z project:**

1. **Read context:** Use `read_file` tool on `/crates/Z00Z_DESIGN_FOUNDATION.md` for relevant sections
2. **Search patterns:** Use `grep_search` to find similar existing implementations  
3. **Verify compliance:** After changes, run `./scripts/verify_compliance.sh`
4. **Ask when uncertain:** If pattern unclear, request clarification from user

**Never assume** - always verify against Design Foundation before implementing architectural decisions.

**Workflow:**
```bash
# 1. Read relevant sections of Design Foundation
cat crates/Z00Z_DESIGN_FOUNDATION.md | grep -A 20 "YOUR_PATTERN_NAME"

# 2. Check for similar existing patterns
rg "pattern_keyword" crates/ --type rust

# 3. Verify compliance after changes
./scripts/verify_compliance.sh
```

---

## 🎓 How to Use These Instructions

**For Copilot:**
1. Read and internalize these guidelines
2. Apply them consistently to all code and documentation
3. When uncertain, ask user for clarification
4. Prioritize correctness over cleverness
5. Prefer small, focused changes over large refactors

**For Users:**
- Reference this file with: `use crates/github-copilot-instructions.md;`
- Specific sections: `@must`, `@rust.instructions`, `@ONE_SOURCE_OF_TRUTH`
- For crypto: `@Tari-Crypto-Integration-Z00Z` `Tari-Crypto-Components-Cookbook`
- For utils: `@Z00Z_UTILS_MODULE_MAP` `Z00Z_UTILS_QUICK_REFERENCE`

---

**Last Updated:** 2025-12-10  
**Maintainers:** Z00Z Development Team  
**License:** Internal Use Only
