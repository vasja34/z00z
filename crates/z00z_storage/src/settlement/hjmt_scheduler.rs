use std::{
    any::Any,
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use rayon::{prelude::*, ThreadPoolBuilder};

use crate::backend::redb::StoragePlane;

use super::{hjmt_config::env_value, SettlementStore, SettlementStoreError};

const SCHED_CPU_ENV: &str = "Z00Z_STORAGE_SCHED_CPU";
const SCHED_QUEUE_ENV: &str = "Z00Z_STORAGE_SCHED_QUEUE";

#[derive(Clone, Copy)]
struct SchedCfg {
    cpu: usize,
    queue: usize,
}

impl SchedCfg {
    fn new() -> Self {
        let cpu = env_usize(SCHED_CPU_ENV).unwrap_or_else(default_cpu);
        let queue = env_usize(SCHED_QUEUE_ENV).unwrap_or(cpu.saturating_mul(8).max(1));
        Self {
            cpu: cpu.max(1),
            queue: queue.max(1),
        }
    }
}

#[derive(Clone, Copy, Default)]
struct SchedHook {
    cancel_after: Option<usize>,
    skew_ms: u64,
}

struct PoolState {
    cpu: usize,
    pool: Arc<rayon::ThreadPool>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ForestSchedulerMetrics {
    pub max_active: usize,
    pub reject_count: u64,
    pub cancel_count: u64,
    pub last_batch: usize,
    pub last_queued: usize,
    pub max_queued: usize,
    pub last_ordered: bool,
    pub last_blocking_thread: Option<String>,
    pub last_blocking_wait_us: u64,
}

pub(super) struct ForestScheduler {
    cfg: Mutex<SchedCfg>,
    hook: Mutex<SchedHook>,
    pool: Mutex<Option<PoolState>>,
    max_active: AtomicUsize,
    reject_count: AtomicU64,
    cancel_count: AtomicU64,
    last_batch: AtomicUsize,
    last_queued: AtomicUsize,
    max_queued: AtomicUsize,
    last_ordered: AtomicBool,
    block_seq: AtomicUsize,
    last_blocking_wait_us: AtomicU64,
    last_blocking: Mutex<Option<String>>,
}

impl ForestScheduler {
    pub(super) fn new() -> Self {
        Self {
            cfg: Mutex::new(SchedCfg::new()),
            hook: Mutex::new(SchedHook::default()),
            pool: Mutex::new(None),
            max_active: AtomicUsize::new(0),
            reject_count: AtomicU64::new(0),
            cancel_count: AtomicU64::new(0),
            last_batch: AtomicUsize::new(0),
            last_queued: AtomicUsize::new(0),
            max_queued: AtomicUsize::new(0),
            last_ordered: AtomicBool::new(true),
            block_seq: AtomicUsize::new(0),
            last_blocking_wait_us: AtomicU64::new(0),
            last_blocking: Mutex::new(None),
        }
    }

    pub(super) fn run_local<T, E, F>(&self, stage: &'static str, work: F) -> Result<T, E>
    where
        E: From<SettlementStoreError>,
        F: FnOnce() -> Result<T, E>,
    {
        self.last_ordered.store(true, Ordering::SeqCst);
        self.run_local_queued(stage, 1, work)
    }

    pub(super) fn run_local_queued<T, E, F>(
        &self,
        stage: &'static str,
        queued: usize,
        work: F,
    ) -> Result<T, E>
    where
        E: From<SettlementStoreError>,
        F: FnOnce() -> Result<T, E>,
    {
        self.last_ordered.store(true, Ordering::SeqCst);
        self.check_queue::<E>(stage, queued.max(1))?;
        self.check_cancel::<E>(stage, 0)?;
        work()
    }

    pub(super) fn run_serial_batch<T, E, F>(
        &self,
        stage: &'static str,
        batch: usize,
        work: F,
    ) -> Result<T, E>
    where
        E: From<SettlementStoreError>,
        F: FnOnce() -> Result<T, E>,
    {
        self.last_batch.store(batch, Ordering::SeqCst);
        self.last_ordered.store(true, Ordering::SeqCst);
        if batch == 0 {
            return work();
        }
        self.check_queue::<E>(stage, batch)?;
        self.check_cancel::<E>(stage, 0)?;
        self.observe_active(1);
        work()
    }

    pub(super) fn one<T, R, E, F>(&self, stage: &'static str, job: T, work: F) -> Result<R, E>
    where
        T: Send,
        R: Send,
        E: From<SettlementStoreError> + Send,
        F: Fn(T) -> Result<R, E> + Sync + Send,
    {
        self.map(stage, vec![job], work)?
            .into_iter()
            .next()
            .ok_or_else(|| {
                SettlementStoreError::Sched {
                    stage,
                    reason: "missing scheduler result".to_string(),
                }
                .into()
            })
    }

    pub(super) fn map<T, R, E, F>(
        &self,
        stage: &'static str,
        jobs: Vec<T>,
        work: F,
    ) -> Result<Vec<R>, E>
    where
        T: Send,
        R: Send,
        E: From<SettlementStoreError> + Send,
        F: Fn(T) -> Result<R, E> + Sync + Send,
    {
        if jobs.is_empty() {
            return Ok(Vec::new());
        }

        self.check_queue::<E>(stage, jobs.len())?;
        self.last_batch.store(jobs.len(), Ordering::SeqCst);
        self.last_ordered.store(true, Ordering::SeqCst);
        let cfg = self.cfg();
        let hook = self.hook();
        let active = AtomicUsize::new(0);
        let started = AtomicUsize::new(0);
        let total = jobs.len();
        let pool = self.thread_pool(stage, cfg.cpu).map_err(E::from)?;

        let result = pool.install(|| {
            jobs.into_par_iter()
                .enumerate()
                .map(|(idx, job)| {
                    self.check_cancel::<E>(stage, started.fetch_add(1, Ordering::SeqCst))?;
                    let now = active.fetch_add(1, Ordering::SeqCst).saturating_add(1);
                    self.observe_active(now);
                    maybe_skew(hook, idx, total);
                    let out = work(job).map(|value| (idx, value));
                    active.fetch_sub(1, Ordering::SeqCst);
                    out
                })
                .collect::<Vec<_>>()
        });
        let ordered = result
            .iter()
            .enumerate()
            .all(|(pos, item)| item.as_ref().ok().is_some_and(|(idx, _)| *idx == pos));
        self.last_ordered.store(ordered, Ordering::SeqCst);

        let mut ordered = Vec::with_capacity(total);
        for item in result {
            let (idx, value) = item?;
            ordered.push((idx, value));
        }
        ordered.sort_by_key(|(idx, _)| *idx);
        Ok(ordered.into_iter().map(|(_, value)| value).collect())
    }

    pub(super) fn run_blocking<T, E, F>(&self, stage: &'static str, work: F) -> Result<T, E>
    where
        E: From<SettlementStoreError> + Send,
        F: FnOnce() -> Result<T, E> + Send,
        T: Send,
    {
        self.last_ordered.store(true, Ordering::SeqCst);
        self.check_queue::<E>(stage, 1)?;
        self.check_cancel::<E>(stage, 0)?;
        let thread_name = format!(
            "z00z-hjmt-blocking-{}",
            self.block_seq.fetch_add(1, Ordering::SeqCst)
        );

        let wait_started = Instant::now();
        let result = thread::scope(|scope| {
            let handle = thread::Builder::new()
                .name(thread_name)
                .spawn_scoped(scope, || {
                    let current = thread::current().name().unwrap_or("unnamed").to_string();
                    *self
                        .last_blocking
                        .lock()
                        .unwrap_or_else(|poison| poison.into_inner()) = Some(current);
                    work()
                })
                .map_err(|err| {
                    E::from(SettlementStoreError::Sched {
                        stage,
                        reason: err.to_string(),
                    })
                })?;

            handle.join().map_err(|panic| {
                E::from(SettlementStoreError::Sched {
                    stage,
                    reason: panic_text(panic),
                })
            })?
        });
        self.last_blocking_wait_us
            .store(elapsed_micros_u64(wait_started.elapsed()), Ordering::SeqCst);
        result
    }

    pub(super) fn metrics(&self) -> ForestSchedulerMetrics {
        ForestSchedulerMetrics {
            max_active: self.max_active.load(Ordering::SeqCst),
            reject_count: self.reject_count.load(Ordering::SeqCst),
            cancel_count: self.cancel_count.load(Ordering::SeqCst),
            last_batch: self.last_batch.load(Ordering::SeqCst),
            last_queued: self.last_queued.load(Ordering::SeqCst),
            max_queued: self.max_queued.load(Ordering::SeqCst),
            last_ordered: self.last_ordered.load(Ordering::SeqCst),
            last_blocking_wait_us: self.last_blocking_wait_us.load(Ordering::SeqCst),
            last_blocking_thread: self
                .last_blocking
                .lock()
                .unwrap_or_else(|poison| poison.into_inner())
                .clone(),
        }
    }

    fn cfg(&self) -> SchedCfg {
        *self.cfg.lock().unwrap_or_else(|poison| poison.into_inner())
    }

    fn hook(&self) -> SchedHook {
        *self
            .hook
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }

    fn thread_pool(
        &self,
        stage: &'static str,
        cpu: usize,
    ) -> Result<Arc<rayon::ThreadPool>, SettlementStoreError> {
        let mut guard = self
            .pool
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if let Some(state) = guard.as_ref() {
            if state.cpu == cpu {
                return Ok(Arc::clone(&state.pool));
            }
        }

        let pool = Arc::new(
            ThreadPoolBuilder::new()
                .num_threads(cpu)
                .thread_name(|idx| format!("z00z-hjmt-worker-{idx}"))
                .build()
                .map_err(|err| SettlementStoreError::Sched {
                    stage,
                    reason: err.to_string(),
                })?,
        );
        *guard = Some(PoolState {
            cpu,
            pool: Arc::clone(&pool),
        });
        Ok(pool)
    }

    fn observe_active(&self, now: usize) {
        let mut seen = self.max_active.load(Ordering::SeqCst);
        while now > seen {
            match self
                .max_active
                .compare_exchange(seen, now, Ordering::SeqCst, Ordering::SeqCst)
            {
                Ok(_) => return,
                Err(next) => seen = next,
            }
        }
    }

    fn check_queue<E>(&self, stage: &'static str, queued: usize) -> Result<(), E>
    where
        E: From<SettlementStoreError>,
    {
        self.last_queued.store(queued, Ordering::SeqCst);
        self.observe_queued(queued);
        let limit = self.cfg().queue;
        if queued <= limit {
            return Ok(());
        }
        self.reject_count.fetch_add(1, Ordering::SeqCst);
        Err(SettlementStoreError::SchedBackpressure {
            stage,
            queued,
            limit,
        }
        .into())
    }

    fn observe_queued(&self, queued: usize) {
        let mut seen = self.max_queued.load(Ordering::SeqCst);
        while queued > seen {
            match self
                .max_queued
                .compare_exchange(seen, queued, Ordering::SeqCst, Ordering::SeqCst)
            {
                Ok(_) => return,
                Err(next) => seen = next,
            }
        }
    }
    fn check_cancel<E>(&self, stage: &'static str, started: usize) -> Result<(), E>
    where
        E: From<SettlementStoreError>,
    {
        let Some(limit) = self.hook().cancel_after else {
            return Ok(());
        };
        if started < limit {
            return Ok(());
        }
        self.cancel_count.fetch_add(1, Ordering::SeqCst);
        Err(SettlementStoreError::SchedCancel { stage }.into())
    }

    #[cfg(debug_assertions)]
    pub(super) fn set_limits_for_test(&self, cpu: usize, queue: usize) {
        *self.cfg.lock().unwrap_or_else(|poison| poison.into_inner()) = SchedCfg {
            cpu: cpu.max(1),
            queue: queue.max(1),
        };
        *self
            .pool
            .lock()
            .unwrap_or_else(|poison| poison.into_inner()) = None;
    }

    #[cfg(debug_assertions)]
    pub(super) fn set_cancel_after_for_test(&self, cancel_after: Option<usize>) {
        self.hook
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .cancel_after = cancel_after;
    }

    #[cfg(debug_assertions)]
    pub(super) fn set_skew_ms_for_test(&self, skew_ms: u64) {
        self.hook
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .skew_ms = skew_ms;
    }

    #[cfg(debug_assertions)]
    pub(super) fn reset_for_test(&self) {
        self.max_active.store(0, Ordering::SeqCst);
        self.reject_count.store(0, Ordering::SeqCst);
        self.cancel_count.store(0, Ordering::SeqCst);
        self.last_batch.store(0, Ordering::SeqCst);
        self.last_queued.store(0, Ordering::SeqCst);
        self.max_queued.store(0, Ordering::SeqCst);
        self.last_ordered.store(true, Ordering::SeqCst);
        self.block_seq.store(0, Ordering::SeqCst);
        self.last_blocking_wait_us.store(0, Ordering::SeqCst);
        *self
            .pool
            .lock()
            .unwrap_or_else(|poison| poison.into_inner()) = None;
        *self
            .last_blocking
            .lock()
            .unwrap_or_else(|poison| poison.into_inner()) = None;
        *self
            .hook
            .lock()
            .unwrap_or_else(|poison| poison.into_inner()) = SchedHook::default();
    }

    pub(super) fn fork_view(&self) -> Self {
        Self {
            cfg: Mutex::new(self.cfg()),
            hook: Mutex::new(self.hook()),
            pool: Mutex::new(None),
            max_active: AtomicUsize::new(0),
            reject_count: AtomicU64::new(0),
            cancel_count: AtomicU64::new(0),
            last_batch: AtomicUsize::new(0),
            last_queued: AtomicUsize::new(0),
            max_queued: AtomicUsize::new(0),
            last_ordered: AtomicBool::new(true),
            block_seq: AtomicUsize::new(0),
            last_blocking_wait_us: AtomicU64::new(0),
            last_blocking: Mutex::new(None),
        }
    }
}

fn elapsed_micros_u64(elapsed: Duration) -> u64 {
    u64::try_from(elapsed.as_micros()).unwrap_or(u64::MAX)
}

impl SettlementStore {
    pub(crate) fn sched_run_local<T, E, F>(&self, stage: &'static str, work: F) -> Result<T, E>
    where
        E: From<SettlementStoreError>,
        F: FnOnce() -> Result<T, E>,
    {
        self.scheduler.run_local(stage, work)
    }

    pub(crate) fn sched_run_local_queued<T, E, F>(
        &self,
        stage: &'static str,
        queued: usize,
        work: F,
    ) -> Result<T, E>
    where
        E: From<SettlementStoreError>,
        F: FnOnce() -> Result<T, E>,
    {
        self.scheduler.run_local_queued(stage, queued, work)
    }

    pub(crate) fn sched_run_serial_batch<T, E, F>(
        &self,
        stage: &'static str,
        batch: usize,
        work: F,
    ) -> Result<T, E>
    where
        E: From<SettlementStoreError>,
        F: FnOnce() -> Result<T, E>,
    {
        self.scheduler.run_serial_batch(stage, batch, work)
    }

    pub(crate) fn sched_one<T, R, E, F>(&self, stage: &'static str, job: T, work: F) -> Result<R, E>
    where
        T: Send,
        R: Send,
        E: From<SettlementStoreError> + Send,
        F: Fn(T) -> Result<R, E> + Sync + Send,
    {
        self.scheduler.one(stage, job, work)
    }

    pub(crate) fn sched_map<T, R, E, F>(
        &self,
        stage: &'static str,
        jobs: Vec<T>,
        work: F,
    ) -> Result<Vec<R>, E>
    where
        T: Send,
        R: Send,
        E: From<SettlementStoreError> + Send,
        F: Fn(T) -> Result<R, E> + Sync + Send,
    {
        self.scheduler.map(stage, jobs, work)
    }

    pub(crate) fn sched_block<T, E, F>(&self, stage: &'static str, work: F) -> Result<T, E>
    where
        E: From<SettlementStoreError> + Send,
        F: FnOnce() -> Result<T, E> + Send,
        T: Send,
    {
        self.scheduler.run_blocking(stage, work)
    }

    pub(super) fn fork_sched_view(&self) -> Self {
        let mut fork =
            Self::build_with_policy(StoragePlane::off(), self.backend_mode, self.bucket_policy);
        fork.scheduler = self.scheduler.fork_view();
        fork.restore_store(self.snap_store());
        fork.hjmt_store.restore(self.hjmt_store.snap());
        fork.forest_cache.restore(self.forest_cache.snapshot());
        fork.hjmt_roots = self.hjmt_roots.clone();
        fork
    }

    pub fn forest_scheduler_metrics(&self) -> ForestSchedulerMetrics {
        self.scheduler.metrics()
    }

    #[cfg(debug_assertions)]
    pub fn set_sched_limits_for_test(&self, cpu: usize, queue: usize) {
        self.scheduler.set_limits_for_test(cpu, queue);
    }

    #[cfg(debug_assertions)]
    pub fn set_sched_cancel_for_test(&self, cancel_after: Option<usize>) {
        self.scheduler.set_cancel_after_for_test(cancel_after);
    }

    #[cfg(debug_assertions)]
    pub fn set_sched_test_skew_ms(&self, skew_ms: u64) {
        self.scheduler.set_skew_ms_for_test(skew_ms);
    }

    #[cfg(debug_assertions)]
    pub fn reset_sched_for_test(&self) {
        self.scheduler.reset_for_test();
    }
}

fn default_cpu() -> usize {
    thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1)
        .max(1)
}

fn env_usize(key: &str) -> Option<usize> {
    env_value(key)
        .ok()
        .flatten()
        .and_then(|raw| raw.trim().parse::<usize>().ok())
        .filter(|value| *value > 0)
}

fn panic_text(panic: Box<dyn Any + Send + 'static>) -> String {
    if let Some(text) = panic.downcast_ref::<&str>() {
        return (*text).to_string();
    }
    if let Some(text) = panic.downcast_ref::<String>() {
        return text.clone();
    }
    "unknown panic".to_string()
}

fn maybe_skew(hook: SchedHook, idx: usize, total: usize) {
    if hook.skew_ms == 0 {
        return;
    }
    let step = total.saturating_sub(idx);
    let delay = hook
        .skew_ms
        .saturating_mul(u64::try_from(step).unwrap_or(u64::MAX));
    thread::sleep(Duration::from_millis(delay));
}
