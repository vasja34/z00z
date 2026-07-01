/// Hook invoked when cache entries are evicted from the receiver manager.
pub trait CacheEvictionListener: std::fmt::Debug + Send + Sync {
    /// Called when an entry is evicted from the cache.
    ///
    /// Notes:
    /// - This callback may be invoked from multiple threads.
    /// - The receiver manager invokes the listener without holding the cache lock.
    /// - Implementations should avoid blocking and should not panic.
    fn on_evict(&self, path: Bip44Path, key: &Z00ZRistrettoPoint);
}

/// Default eviction listener that performs no action.
#[derive(Debug, Default)]
pub struct NoopEvictionListener;

impl CacheEvictionListener for NoopEvictionListener {
    fn on_evict(&self, _path: Bip44Path, _key: &Z00ZRistrettoPoint) {}
}

/// Eviction listener that logs evictions via the project logger abstraction.
#[derive(Debug, Default)]
pub struct LoggingEvictionListener;

impl CacheEvictionListener for LoggingEvictionListener {
    fn on_evict(&self, path: Bip44Path, _key: &Z00ZRistrettoPoint) {
        log_evict_path(&path);
    }
}

#[cfg(all(debug_assertions, feature = "eviction-logs"))]
fn log_evict_path(path: &Bip44Path) {
    z00z_utils::logger::Logger::debug(
        &z00z_utils::logger::TracingLogger,
        &format!("Receiver cache LRU eviction: {}", path),
    );
}

#[cfg(not(all(debug_assertions, feature = "eviction-logs")))]
fn log_evict_path(_path: &Bip44Path) {}

/// Eviction listener that persists evicted entries to disk (best-effort).
///
/// WARNING: Persisting eviction records can leak wallet metadata (derivation paths and keys).
/// This listener is disabled by default and requires explicit opt-in via compile-time gating.
///
/// Enable only for local debugging, never in production.
#[cfg(all(
    not(target_arch = "wasm32"),
    debug_assertions,
    feature = "eviction-logs"
))]
pub struct PersistenceEvictionListener {
    path: std::path::PathBuf,
    tx: SyncSender<String>,
}

#[cfg(all(
    not(target_arch = "wasm32"),
    debug_assertions,
    feature = "eviction-logs"
))]
impl std::fmt::Debug for PersistenceEvictionListener {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PersistenceEvictionListener")
            .field("path", &self.path)
            .finish()
    }
}

#[cfg(all(
    not(target_arch = "wasm32"),
    debug_assertions,
    feature = "eviction-logs"
))]
impl PersistenceEvictionListener {
    /// Create a new persistence listener that appends JSONL records to `path`.
    ///
    /// This listener is designed to be non-blocking on the eviction path: evictions are
    /// enqueued into a bounded in-memory queue and written by a background worker thread.
    /// If the queue is full or the worker has stopped, new events are dropped.
    pub fn new(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let rotation = z00z_utils::logger::RotationPolicy {
            max_bytes: 10 * 1024 * 1024,
            keep_files: 3,
        };

        let logger = z00z_utils::logger::RotatingFileLogger::new(&path, rotation)?;
        let (tx, rx) = std::sync::mpsc::sync_channel::<String>(1024);

        let worker_path = path.clone();
        std::thread::Builder::new()
            .name(format!(
                "z00z_wallets.cache_evict_persist:{}",
                worker_path.display()
            ))
            .spawn(move || {
                while let Ok(line) = rx.recv() {
                    z00z_utils::logger::Logger::info(&logger, &line);
                }
            })?;

        Ok(Self { path, tx })
    }
}

#[cfg(all(
    not(target_arch = "wasm32"),
    debug_assertions,
    feature = "eviction-logs"
))]
impl CacheEvictionListener for PersistenceEvictionListener {
    fn on_evict(&self, path: Bip44Path, key: &Z00ZRistrettoPoint) {
        let key_hex = z00z_crypto::expert::encoding::to_hex(key.as_bytes());
        let line = format!(r#"{{"path":"{}","spend_key":"{}"}}"#, path, key_hex);
        let _ = self.tx.try_send(line);
    }
}

#[cfg(all(
    not(target_arch = "wasm32"),
    not(all(debug_assertions, feature = "eviction-logs"))
))]
#[derive(Debug, Default)]
/// Disabled-by-default stub for `PersistenceEvictionListener`.
///
/// This type exists to keep the API surface stable, but `new()` returns
/// `ErrorKind::Unsupported` unless compiled with `--features eviction-logs` in a
/// debug-assertions build.
pub struct PersistenceEvictionListener;

#[cfg(all(
    not(target_arch = "wasm32"),
    not(all(debug_assertions, feature = "eviction-logs"))
))]
impl PersistenceEvictionListener {
    /// Create a new persistence listener.
    ///
    /// This is intentionally disabled unless compiled with `--features eviction-logs` in a
    /// debug-assertions build.
    pub fn new(_path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "PersistenceEvictionListener is disabled by default; enable with --features eviction-logs in a debug build",
        ))
    }
}

#[cfg(all(
    not(target_arch = "wasm32"),
    not(all(debug_assertions, feature = "eviction-logs"))
))]
impl CacheEvictionListener for PersistenceEvictionListener {
    fn on_evict(&self, _path: Bip44Path, _key: &Z00ZRistrettoPoint) {}
}
