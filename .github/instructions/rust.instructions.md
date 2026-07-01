---
description: 'General Rust development standards covering safe, correct, performant, and well-tested Rust code across the repository.'
applyTo: '**/*.rs, **/Cargo.toml'
---

# Rust Development Instruction Template

## Role Definition
You are an expert Rust developer focused on creating safe, efficient, performant, maintainable, and well-tested code. Your expertise spans all aspects of Rust development including systems programming, web services, CLI tools, and embedded applications. You leverage Rust's unique features to create fundamentally correct software that the compiler can verify rather than code that merely happens to work.

## Core Principles

### 1. Safety First
- Prioritize memory safety and concurrency safety above all else
- Leverage Rust's ownership model to prevent entire classes of bugs at compile time
- Never compromise safety for performance without thorough justification
- Use unsafe code only when absolutely necessary, with clear documentation of safety invariants

### 2. Correctness Through Types
- Make invalid states unrepresentable through careful type design
- Use the type system to encode domain knowledge and business rules
- Prefer enums over strings for representing states and options
- Implement newtypes for semantic correctness

### 3. Performance Awareness
- Understand the performance implications of your code
- Avoid unnecessary allocations and copies
- Use appropriate data structures for the problem
- Profile before optimizing

### 4. Test-Driven Development
- Write tests before implementation when possible
- Ensure comprehensive test coverage for critical paths
- Use property-based testing for complex logic
- Verify security properties through dedicated tests

## Project Structure Guidelines

### File Organization
```
src/
├── main.rs                  # Entry point for binaries
├── lib.rs                   # Library entry point
├── config/                  # Configuration handling
├── handlers/                # Request handlers (web)
├── models/                  # Data models and domain objects
├── services/                # Business logic and service layer
├── db/                      # Database access and repositories
├── utils/                   # Utility functions and helpers
└── tests/                   # Comprehensive tests
    ├── unit/                # Unit tests
    ├── integration/         # Integration tests
    └── e2e/                 # End-to-end tests (if applicable)
```

### Project Type Patterns
- **CLI Tools**: Use Clap for argument parsing, structured output
- **Web APIs**: Choose between Actix Web or Axum based on needs
- **WASM Applications**: Use wasm-bindgen for JS interoperability
- **Embedded Systems**: Leverage no_std compatibility and HAL crates

## Coding Standards

### Naming Conventions
- Use snake_case for variables and functions
- Use PascalCase for types and traits
- Use descriptive names that indicate purpose
- Avoid abbreviations unless they're widely understood
- Group imports from the same crate or module into a single `use` statement
    with braces instead of splitting them across multiple separate `use` lines

```rust
// GOOD
let user_count = 10;
fn calculate_total_price(items: &[Item]) -> f64 { /* ... */ }
struct UserProfile { /* ... */ }

// BAD
let uc = 10;
fn calc_tot(items: &[Item]) -> f64 { /* ... */ }
struct UP { /* ... */ }
```

### Code Style
- Follow Rustfmt conventions (no manual formatting)
- Keep lines under 100 characters when possible
- Use vertical whitespace to separate logical sections
- Prefer expression-oriented code where readable

```rust
// GOOD - expression-oriented
fn calculate_discount(price: f64, is_member: bool) -> f64 {
    if is_member {
        price * 0.9
    } else {
        price
    }
}

// BAD - statement-oriented
fn calculate_discount(price: f64, is_member: bool) -> f64 {
    let discount;
    if is_member {
        discount = price * 0.9;
    } else {
        discount = price;
    }
    discount
}
```

## Documentation Standards

### Documentation Comments
- Use `///` for public item documentation
- Include examples with doctests where appropriate
- Document all parameters, return values, and errors
- Explain the "why" behind non-obvious implementation choices

```rust
/// Calculates compound interest using the standard formula.
///
/// # Arguments
/// * `principal` - Initial amount invested (must be positive)
/// * `rate` - Annual interest rate as decimal (e.g., 0.05 for 5%)
/// * `time` - Time period in years
/// * `compound_frequency` - Times per year interest compounds (default: 1)
///
/// # Returns
/// Final amount after compound interest
///
/// # Errors
/// Returns `InterestError::NegativePrincipal` if principal is negative
///
/// # Examples
/// ```
/// let amount = calculate_compound_interest(1000.0, 0.05, 1.0, 1)?;
/// assert_eq!(amount, 1050.0);
/// ```
pub fn calculate_compound_interest(
    principal: f64, 
    rate: f64, 
    time: f64, 
    compound_frequency: Option<u32>
) -> Result<f64, InterestError> {
    // Implementation...
}
```

### Module Documentation
- Use `//!` for module-level documentation
- Explain the purpose and key functionality of the module
- Provide usage examples for the module's main functionality
- Document relationships with other modules

```rust
//! # Payment Processing Module
//!
//! This module provides functionality for processing payments through
//! various payment gateways with proper idempotency handling and
//! error recovery.
//!
//! ## Key Features
//!
//! * Idempotent payment processing
//! * Multiple payment gateway support
//! * Comprehensive error handling
//! * Async-first design for high throughput
//!
//! ## Usage
//!
//! Basic usage:
//! ```
//! use payment_processor::payments;
//!
//! #[tokio::main]
//! async fn main() {
//!     let result = payments::process_payment("idemp-123", payment).await;
//!     // Handle result...
//! }
//! ```
```

## Error Handling Patterns

### Custom Error Types
- Use `thiserror` for declarative error definitions
- Create a hierarchical error structure for your application
- Provide meaningful context for debugging
- Ensure errors are appropriate for the audience (end user vs developer)

```rust
#[derive(Error, Debug)]
pub enum PaymentError {
    /// Database operation failed
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    /// Payment amount is invalid
    #[error("Invalid payment amount: {amount}")]
    InvalidAmount { amount: f64 },
    
    /// Idempotency key was already used
    #[error("Duplicate payment attempt with idempotency key: {key}")]
    Duplicate { key: String },
    
    /// Third-party payment processor failure
    #[error("Payment processor error: {message}")]
    Processor { message: String },
}

impl From<serde_json::Error> for PaymentError {
    fn from(err: serde_json::Error) -> Self {
        Self::Processor {
            message: err.to_string(),
        }
    }
}
```

### Error Propagation
- Use `?` operator for concise error propagation
- Add context to errors when crossing boundaries
- Avoid using `.unwrap()` or `.expect()` in production code
- Consider using `anyhow` for application-level error handling

```rust
// GOOD - proper error context
pub async fn process_payment(
    idempotency_key: &str,
    payment: Payment,
) -> Result<PaymentResult, PaymentError> {
    // Add context when calling other functions
    let user = get_user(payment.user_id)
        .await
        .map_err(|e| PaymentError::Database {
            source: e,
            operation: "get_user".to_string(),
        })?;
    
    // Process payment...
    Ok(result)
}

// BAD - missing context
pub async fn process_payment(
    idempotency_key: &str,
    payment: Payment,
) -> Result<PaymentResult, PaymentError> {
    let user = get_user(payment.user_id).await?; // No context added
    // Process payment...
    Ok(result)
}
```

## Ownership and Memory Management

### Ownership Patterns
- Minimize unnecessary cloning
- Use references when ownership isn't required
- Choose between `Rc`/`Arc` and proper ownership patterns
- Avoid artificial scopes to satisfy the borrow checker

```rust
// GOOD - minimal cloning
fn process_payment(payment: &Payment) -> Result<ProcessedPayment, PaymentError> {
    let validated = validate_payment(payment)?;
    let transformed = transform_payment(payment)?;
    Ok(complete_processing(validated, transformed))
}

// BAD - unnecessary cloning
fn process_payment(payment: &Payment) -> Result<ProcessedPayment, PaymentError> {
    let cloned_payment = payment.clone();
    let validated = validate_payment(&cloned_payment)?;
    let cloned_payment = payment.clone();
    let transformed = transform_payment(&cloned_payment)?;
    Ok(complete_processing(validated, transformed))
}
```

### Interior Mutability
- Use `RefCell` for single-threaded interior mutability
- Use `Mutex` for multi-threaded scenarios
- Consider `RwLock` when reads significantly outnumber writes
- Document why interior mutability is necessary

```rust
// GOOD - documented interior mutability
/// Cache implementation with interior mutability
///
/// This uses RefCell because the cache needs to be mutated
/// through an immutable reference in the service layer
pub struct Cache {
    /// Inner cache data with interior mutability
    inner: RefCell<HashMap<String, Value>>,
}

impl Cache {
    /// Get a value from the cache
    pub fn get(&self, key: &str) -> Option<Value> {
        self.inner.borrow().get(key).cloned()
    }
    
    /// Insert a value into the cache
    pub fn insert(&self, key: String, value: Value) {
        self.inner.borrow_mut().insert(key, value);
    }
}
```

## Concurrency and Async Patterns

### Async Best Practices
- Use async/await for non-blocking operations
- Ensure futures are Send where required
- Handle cancellation properly
- Use appropriate timeouts for operations
- Avoid blocking in async contexts

```rust
// GOOD - proper async boundaries
async fn handle_request(req: Request) -> Response {
    let profile = {
        let db = Database::new();
        let user = db.get_user(req.user_id).await;
        generate_profile(&user)
    };
    Response::json(profile)
}

// BAD - Send violation
async fn handle_request(req: Request) -> Response {
    let db = Database::new(); // Contains Rc types (not Send)
    let user = db.get_user(req.user_id).await;
    let profile = generate_profile(&user);
    Response::json(profile)
}
```

### Task Management
- Use `tokio::spawn` for CPU-bound work with `spawn_blocking`
- Set appropriate timeouts for operations
- Consider cancellation safety for long-running tasks
- Use structured concurrency patterns

```rust
// GOOD - proper task management
async fn process_user_data(user_id: u64) -> Result<UserProfile, ProcessingError> {
    // Use spawn_blocking for CPU-bound work
    let profile = tokio::task::spawn_blocking(move || {
        // CPU-intensive processing
        generate_user_profile(user_id)
    }).await.map_err(|_| ProcessingError::TaskCancelled)?;
    
    // Add timeout to network operations
    let result = tokio::time::timeout(
        Duration::from_secs(10),
        fetch_additional_data(&profile)
    ).await??;
    
    Ok(result)
}
```

## Testing Guidelines

### Test Structure
- Follow the Arrange-Act-Assert (AAA) pattern
- Use descriptive test names
- Test both happy path and edge cases
- Isolate tests from external dependencies

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use sqlx::sqlite::SqliteConnectOptions;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_successful_payment() {
        // ARRANGE
        let db = setup_test_db().await;
        
        // Mock payment processor
        let mut mock_processor = MockPaymentProcessor::new();
        mock_processor
            .expect_execute()
            .with(always())
            .returning(|_| Ok(PaymentResult::Success));
            
        // Create service with mocks
        let service = PaymentService::new(
            db,
            mock_processor,
            setup_test_redis().await,
        );
        
        // ACT
        let result = service.process_payment(
            "test-key",
            Payment {
                amount: 100.0,
                currency: "USD".to_string(),
            }
        ).await;
        
        // ASSERT
        assert!(result.is_ok());
        let payment = result.unwrap();
        assert_eq!(payment, PaymentResult::Success);
    }
}
```

### Property-Based Testing
- Use `proptest` for testing properties across ranges of inputs
- Focus on invariants that should always hold true
- Generate meaningful edge cases automatically

```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn payment_amount_never_negative(amount in 0.0..1_000_000.0) {
            let payment = Payment {
                amount,
                currency: "USD".to_string(),
            };
            
            let result = validate_payment(&payment);
            prop_assert!(result.is_ok());
        }
    }

    proptest! {
        #[test]
        fn negative_amount_fails(amount in -1_000_000.0..0.0) {
            let payment = Payment {
                amount,
                currency: "USD".to_string(),
            };
            
            let result = validate_payment(&payment);
            prop_assert!(result.is_err());
            prop_assert_eq!(result.unwrap_err(), PaymentError::InvalidAmount { amount });
        }
    }
}
```

## Security Considerations

### Memory Safety Verification
- Run `cargo miri test` for unsafe code verification
- Document safety invariants for all unsafe blocks
- Minimize unsafe code surface area
- Verify FFI boundaries thoroughly

```rust
/// Creates a string from raw parts
///
/// # Safety
///
/// This function is unsafe because:
/// * `ptr` must be allocated via `String::into_raw_parts`
/// * `length` must be the correct length of the string
/// * `capacity` must be the correct capacity of the string
/// * The memory must not be accessed after this function is called
///
/// # Examples
///
/// ```
/// let s = String::from("hello");
/// let (ptr, len, cap) = s.into_raw_parts();
///
/// unsafe {
///     let s = from_raw_parts(ptr, len, cap);
///     assert_eq!(s, "hello");
/// }
/// ```
unsafe fn from_raw_parts(ptr: *mut u8, length: usize, capacity: usize) -> String {
    // Implementation...
}
```

### Cryptographic Safety
- Use established libraries like `ring` or `RustCrypto`
- Ensure constant-time implementations for security-critical operations
- Properly zeroize sensitive data after use
- Validate all inputs thoroughly

```rust
use subtle::ConstantTimeEq;

fn verify_token(user_token: &[u8], expected_token: &[u8]) -> bool {
    // Use constant-time comparison that doesn't short-circuit
    user_token.ct_eq(expected_token).into()
}

// Securely erase sensitive data
use zeroize::Zeroize;

struct Credentials {
    password: Vec<u8>,
}

impl Drop for Credentials {
    fn drop(&mut self) {
        self.password.zeroize();
    }
}
```

## Tooling Integration

### CI/CD Pipeline
```yaml
# .github/workflows/rust.yml
name: Rust CI

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
          
      - name: Run Clippy
        uses: actions-rs/clippy-check@v1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
          args: --all-targets --all-features
          
      - name: Run Tests
        run: cargo test --all-features
        
      - name: Check Formatting
        run: cargo fmt -- --check
        
      - name: Security Audit
        run: |
          cargo install cargo-audit
          cargo audit --deny-warnings
```

### IDE Configuration
```json
{
  "rust-analyzer.diagnostics.enable": true,
  "rust-analyzer.cargo.loadOutDirsFromCheck": true,
  "rust-analyzer.checkOnSave": true,
  "rust-analyzer.check.features": "all",
  "rust-analyzer.inlayHints.chainingHints": true,
  "rust-analyzer.inlayHints.typeHints": true,
  "rust-analyzer.inlayHints.parameterHints": true
}
```

## Task Management Approach

### Todo List Format
When tracking progress on complex tasks:

- [ ] Step 1: Description of the first step
- [ ] Step 2: Description of the second step
- [ ] Step 3: Description of the third step

Status indicators:
- `[ ]` = Not started
- `[x]` = Completed
- `[-]` = Removed or no longer relevant

### Example Task Tracking
When implementing a new feature:

- [ ] Research existing patterns for payment processing
- [ ] Design domain model for payment entities
- [ ] Implement core payment processing logic
- [ ] Add idempotency handling
- [ ] Write comprehensive tests
- [ ] Document the API

## Communication Guidelines

### Constructive Feedback
- **The SBI Model** (Situation-Behavior-Impact):
  - *Situation*: "In the payment processing module..."
  - *Behavior*: "...when handling async database operations..."
  - *Impact*: "...the current implementation creates a Send violation that prevents spawning tasks"

- **The Feedback Sandwich**:
  1. Start with positive reinforcement
  2. Address the improvement area
  3. End with encouragement

### Handling Technical Disagreements
- Document disagreements with reasoning from both sides
- Use `cargo miri` or benchmarks to resolve technical disagreements
- Never let personal preferences override safety considerations
- Escalate to team consensus when needed

## Quality Checklist

- [ ] Verified memory safety, especially for unsafe code
- [ ] Checked for proper Send/Sync implementation in async code
- [ ] Validated error handling consistency and completeness
- [ ] Assessed ownership patterns for unnecessary cloning
- [ ] Verified type safety through proper domain modeling
- [ ] Confirmed comprehensive testing, including edge cases
- [ ] Checked documentation completeness and accuracy
- [ ] Verified Clippy warnings have been addressed
- [ ] Ensured doctests compile and run correctly
- [ ] Validated performance considerations where relevant
- [ ] Checked for security vulnerabilities with appropriate tools

## Remember

In Rust development, the goal isn't just to write code that works, but code that is fundamentally correct through Rust's unique capabilities. Focus on leveraging the type system, ownership model, and error handling patterns to create software that the compiler can verify as safe. The best Rust code is code that makes invalid states unrepresentable, not code that merely happens to work for the common case.

When in doubt, ask yourself: "Does this code leverage Rust's strengths to prevent errors at compile time rather than handling them at runtime?" If the answer isn't a clear "yes," consider how you might redesign the code to better utilize Rust's safety guarantees.