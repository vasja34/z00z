# Z00Z Utils Phase 1 - Quick Reference

## Module Overview

### 📦 io - File I/O Abstraction

**Import:**
```rust
use z00z_utils::{IoError, save_json, load_json, save_yaml, load_yaml, save_bincode, load_bincode};
use z00z_utils::io::fs::{save_with_codec, load_with_codec};
```

**Common Use Cases:**

```rust
// Save/load JSON
save_json("data.json", &my_data)?;
let data: MyType = load_json("data.json")?;

// Save/load YAML
save_yaml("config.yaml", &my_config)?;
let config: ConfigType = load_yaml("config.yaml")?;

// Save/load Bincode
save_bincode("data.bin", &my_data)?;
let data: MyType = load_bincode("data.bin")?;

// Custom codec
use z00z_utils::JsonCodec;
save_with_codec::<MyType, JsonCodec>("custom.json", &my_data)?;
let data = load_with_codec::<MyType, JsonCodec>("custom.json")?;
```

**Features:**
- ✅ Automatic directory creation
- ✅ Atomic writes (tmpfile + rename)
- ✅ Generic over codec implementation
- ✅ Full error context

---

### ⏰ time - Time Provider Abstraction

**Import:**
```rust
use z00z_utils::{TimeProvider, SystemTimeProvider, MockTimeProvider};
```

**Production Use:**
```rust
use z00z_utils::SystemTimeProvider;
use std::time::SystemTime;

let provider = SystemTimeProvider::default();
let now: SystemTime = provider.now();
let unix_ts: u64 = provider.unix_timestamp();
let unix_ms: u64 = provider.unix_timestamp_millis();
```

**Testing Use:**
```rust
use z00z_utils::MockTimeProvider;
use std::time::{SystemTime, Duration};

// Create with specific time
let now = SystemTime::now();
let provider = MockTimeProvider::new(now);

// Set fixed time
provider.set_time(start_time);

// Advance time (e.g., for testing timeouts)
provider.advance_by(Duration::from_secs(60));

// Share across threads
let provider_clone = provider.clone();
// Same state accessible from clone
```

**Trait Usage:**
```rust
fn process_with_time<T: TimeProvider>(provider: &T) {
    let now = provider.now();
    println!("Current time: {:?}", now);
}

// Works with both SystemTimeProvider and MockTimeProvider
process_with_time(&SystemTimeProvider::default());
process_with_time(&MockTimeProvider::new(start_time));
```

**Thread Safety:**
- ✅ Safe to share across threads (Arc<Mutex> internally)
- ✅ Clones share same state
- ✅ Mutation safe via mutex

---

### 🎲 rng - Random Number Provider Abstraction

**Import:**
```rust
use z00z_utils::rng::{DeterministicRng, MockRngProvider, SecureRngProvider, SystemRngProvider};
use rand::RngCore;
```

**Production Use:**
```rust
use z00z_utils::rng::SystemRngProvider;
use rand::RngCore;

let provider = SystemRngProvider::default();
let mut rng = provider.rng();

// Generate random bytes
let mut buf = [0u8; 32];
rng.fill_bytes(&mut buf);

// Generate random numbers (RngCore methods)
let random_u32: u32 = rng.next_u32();
let random_u64: u64 = rng.next_u64();
```

**Testing Use:**
```rust
use z00z_utils::rng::{DeterministicRng, MockRngProvider};

// Create with specific seed
let provider = MockRngProvider::with_u64_seed(42);
let mut rng = provider.rng();

// Same seed = same sequence
let provider2 = MockRngProvider::with_u64_seed(42);
let mut rng2 = provider2.rng();

// Both produce same values
assert_eq!(rng.next_u32(), rng2.next_u32());
```

**Trait Usage:**
```rust
fn generate_key<P: SecureRngProvider>(provider: &P) -> [u8; 32] {
    let mut rng = provider.rng();
    let mut key = [0u8; 32];
    rng.fill_bytes(&mut key);
    key
}

// Production: secure RNG
let prod_key = generate_key(&SystemRngProvider::default());

// Testing: deterministic RNG uses a different trait
fn generate_test_key<P: DeterministicRng>(provider: &P) -> [u8; 32] {
    let mut rng = provider.rng();
    let mut key = [0u8; 32];
    rng.fill_bytes(&mut key);
    key
}

let test_key = generate_test_key(&MockRngProvider::with_u64_seed(123));
```

**Reproducibility:**
- ✅ Same seed = same sequence (deterministic testing)
- ✅ Different seeds = different sequences
- ✅ Production uses OsRng for cryptographic security

---

## Phase 0 Modules (Already Complete)

### Logger Module
```rust
use z00z_utils::{Logger, NoopLogger, StdoutLogger, TracingLogger};

let logger: Box<dyn Logger> = Box::new(NoopLogger);
logger.info("Application started");
logger.error("Something went wrong");
```

### MetricsSink Module
```rust
use z00z_utils::{MetricsSink, NoopMetrics};
use std::sync::Arc;

let metrics: Arc<dyn MetricsSink> = Arc::new(NoopMetrics);
metrics.inc_counter("requests_total", 1);
metrics.observe_histogram("response_time", 42.5);
metrics.set_gauge("active_connections", 10.0);
```

### ConfigSource Module
```rust
use z00z_utils::{ConfigSource, EnvConfig, YamlConfig, LayeredConfig};

// Environment variables
let env = EnvConfig;
let value: Option<String> = env.get("MY_VAR")?;

// YAML file
let yaml = YamlConfig::from_file("config.yaml")?;
let nested_value: Option<u32> = yaml.get_typed("assets.default.decimals")?;

// Layered fail-closed path with an explicit trusted file location
let config = LayeredConfig::with_yaml("/etc/z00z/config.yaml")?;
let value = config.get("key")?;  // Checks the environment first with the same key string, then YAML

// `new()` is a cwd-relative convenience constructor for controlled CLI use only.
// Platform environment semantics still apply to layered lookups.

// Explicit missing-file-only downgrade
let optional = LayeredConfig::with_optional_yaml("config.yaml")?;
let fallback = optional.get("key")?;
```

### Codec Module

```rust
use z00z_utils::{Codec, JsonCodec, YamlCodec, BincodeCodec};

let json: JsonCodec<MyType> = JsonCodec::default();
let encoded = json.encode(&data)?;
let decoded = json.decode(&encoded)?;

// Works with all I/O functions
save_with_codec::<MyType, JsonCodec>("file.json", &data)?;
```

---

## Testing Patterns

### Unit Test with Mocks

```rust
#[cfg(test)]
mod tests {
    use z00z_utils::MockTimeProvider;
    use std::time::Duration;

    #[test]
    fn test_timing_logic() {
        let start = std::time::SystemTime::now();
        let provider = MockTimeProvider::new(start);
        
        // Your code using time provider
        provider.advance_by(Duration::from_secs(10));
        
        // Verify: time advanced exactly 10 seconds
        let ts1 = provider.unix_timestamp();
        provider.advance_by(Duration::from_secs(5));
        let ts2 = provider.unix_timestamp();
        
        assert_eq!(ts2 - ts1, 5);
    }
}
```

### Unit Test with Deterministic RNG

```rust
#[cfg(test)]
mod tests {
    use rand::RngCore;
    use z00z_utils::rng::{DeterministicRng, MockRngProvider};

    #[test]
    fn test_crypto_with_known_seed() {
        let provider = MockRngProvider::with_u64_seed(12345);
        let mut rng = provider.rng();
        
        let val1 = rng.next_u32();
        
        // Same seed produces same value
        let provider2 = MockRngProvider::with_u64_seed(12345);
        let mut rng2 = provider2.rng();
        assert_eq!(val1, rng2.next_u32());
    }
}
```

### Thread Safety Test

```rust
#[test]
fn test_provider_thread_safety() {
    use std::sync::Arc;
    use std::thread;

    let provider = Arc::new(MockTimeProvider::new(SystemTime::now()));
    let provider2 = Arc::clone(&provider);
    
    let handle = thread::spawn(move || {
        provider2.advance_by(Duration::from_secs(10));
    });
    
    handle.join().unwrap();
    
    // Main thread sees the advancement from spawned thread
    let ts = provider.unix_timestamp();
    assert!(ts >= /* expected value */);
}
```

---

## Error Handling

### I/O Errors

```rust
use z00z_utils::IoError;

match save_json("data.json", &data) {
    Ok(()) => println!("Saved successfully"),
    Err(IoError::Io(e)) => eprintln!("File I/O error: {}", e),
    Err(IoError::Serialization(e)) => eprintln!("Serialization error: {}", e),
    Err(IoError::Deserialization(e)) => eprintln!("Deserialization error: {}", e),
}
```

### Config Errors

```rust
use z00z_utils::{ConfigSource, ConfigError};

match config.get_typed::<u32>("key") {
    Ok(value) => println!("Got: {}", value),
    Err(ConfigError::NotFound { key }) => eprintln!("Key not found: {}", key),
    Err(ConfigError::Parse { key, value, error }) => {
        eprintln!("Failed to parse {} = '{}': {}", key, value, error)
    }
    Err(ConfigError::Io(e)) => eprintln!("I/O error: {}", e),
    Err(ConfigError::FileTooLarge { size, max }) => {
        eprintln!("Config file too large: {} > {}", size, max)
    }
}
```

---

## Performance Notes

### Zero-Cost Abstractions

- ✅ SystemTimeProvider: No allocation, direct `SystemTime::now()` call
- ✅ SystemRngProvider: No allocation, direct `OsRng` usage
- ✅ NoopLogger: Inlined to nothing in release builds
- ✅ NoopMetrics: Inlined to nothing in release builds

### Mock Implementations

- ⚠️ MockTimeProvider: Uses `Arc<Mutex<_>>` for thread safety
- ⚠️ MockRngProvider: Each call creates new StdRng (but deterministic)

For production, always use System* implementations.

---

## Integration Checklist

When integrating into existing code:

- [ ] Replace `std::time::SystemTime` usages with `TimeProvider` trait
- [ ] Replace `rand` usages with `SecureRngProvider` (or `DeterministicRngProvider` for genesis/tests)
- [ ] Replace direct file I/O with `save_*/load_*` functions
- [ ] Replace hardcoded config with `ConfigSource` trait
- [ ] Add Logger trait parameter to key functions
- [ ] Add MetricsSink trait parameter for instrumentation
- [ ] Update tests to use Mock implementations
- [ ] Verify all tests pass
- [ ] Run clippy: `cargo clippy --all-targets`
- [ ] Format code: `cargo fmt`

---

**Last Updated:** Phase 1 Completion  
**Version:** 1.0.0  
**Status:** ✅ Production Ready
