#![doc = include_str!("../README.md")]
#![warn(unsafe_code)]
#![warn(missing_docs)]

//! # Z00Z Utils
//!
//! Generic, reusable utilities for the Z00Z blockchain project.
//!
//! ## Modules
//!
//! - [`logger`] - Logging abstraction (Logger trait)
//! - [`metrics`] - Metrics/telemetry (MetricsSink trait)
//! - [`config`] - Configuration loading (ConfigSource trait)
//! - [`codec`] - Serialization/deserialization (JSON, Bincode, YAML, plus the
//!   explicit narrow JSON compatibility surface for `Value` and `json!()`)
//! - [`io`] - File I/O operations with atomic writes
//! - [`time`] - Time provider abstraction (TimeProvider trait)
//! - [`rng`] - RNG provider abstraction, with deterministic reproducibility gated to explicit domains

pub mod codec;
#[cfg(not(target_arch = "wasm32"))]
pub mod compression;
pub mod config;
pub mod io;
pub mod logger;
pub mod metrics;
pub mod os_hardening;
pub mod rng;
pub mod time;

/// Commonly used imports
pub mod prelude {
    pub use crate::codec::{BincodeCodec, Codec, CodecError, JsonCodec, YamlCodec};
    pub use crate::config::{ConfigError, ConfigSource, LayeredConfig};
    pub use crate::io::{
        load_bincode, load_json, load_yaml, save_bincode, save_json, save_yaml, IoError,
    };
    pub use crate::logger::{
        FileLogger, LogEvent, LogLevel, Logger, NoopLogger, StdoutLogger, StructuredLogger,
        TracingLogger,
    };
    pub use crate::metrics::{MetricsSink, NoopMetrics};
    pub use crate::rng::{
        DeterministicRngSource, RngCoreExt, SecureRngProvider, SystemRngProvider,
    };

    #[cfg(any(
        test,
        feature = "deterministic-rng",
        feature = "test-utils",
        feature = "test-params-fast"
    ))]
    pub use crate::rng::DeterministicRngProvider;

    #[cfg(any(
        test,
        feature = "deterministic-rng",
        feature = "test-utils",
        feature = "test-params-fast"
    ))]
    pub use crate::rng::MockRngProvider;
    pub use crate::time::{MockTimeProvider, SystemTimeProvider, TimeProvider};

    #[cfg(not(target_arch = "wasm32"))]
    pub use crate::compression::{
        lz4_compress, lz4_decompress_bounded, zstd_compress, zstd_decompress_bounded,
        CompressionError,
    };
}
