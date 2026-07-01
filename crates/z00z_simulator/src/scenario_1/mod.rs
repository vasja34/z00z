//! Scenario 1 modules.

use std::{
    sync::{Condvar, Mutex, OnceLock},
    thread::ThreadId,
};

pub mod claim_pkg_consumer;
mod claim_pkg_store;
pub mod runner;
mod runner_contract;
mod runner_verify;
mod runtime_observability;
pub mod stage_1;
pub mod stage_10;
pub mod stage_11;
pub mod stage_12;
pub mod stage_13;
pub mod stage_2;
pub mod stage_3;
pub mod stage_4;
pub mod stage_5;
pub mod stage_6;
pub mod stage_7;
pub mod stage_8;
pub mod stage_9;
pub mod support;

pub use stage_10::run_bundle_publish;
pub use stage_3::{run_claim_genesis, run_claim_prepare};
pub use stage_4::run_claim_publish;
pub use stage_5::run_tx_plan;
pub use stage_6::run_tx_prepare;
pub use stage_7::run_transfer_receive;
pub use stage_8::run_transfer_claim;
pub use stage_9::run_bundle_build;

#[derive(Default)]
struct ScenarioProcessState {
    owner: Option<ThreadId>,
    depth: usize,
}

struct ScenarioProcessLock {
    state: Mutex<ScenarioProcessState>,
    condvar: Condvar,
}

pub(crate) struct ScenarioProcessGuard<'a> {
    lock: &'a ScenarioProcessLock,
}

impl ScenarioProcessLock {
    fn new() -> Self {
        Self {
            state: Mutex::new(ScenarioProcessState::default()),
            condvar: Condvar::new(),
        }
    }

    fn lock(&self) -> ScenarioProcessGuard<'_> {
        let current = std::thread::current().id();
        let mut state = self.state.lock().unwrap_or_else(|err| err.into_inner());
        loop {
            match state.owner {
                Some(owner) if owner == current => {
                    state.depth += 1;
                    break;
                }
                Some(_) => {
                    state = self
                        .condvar
                        .wait(state)
                        .unwrap_or_else(|err| err.into_inner());
                }
                None => {
                    state.owner = Some(current);
                    state.depth = 1;
                    break;
                }
            }
        }
        ScenarioProcessGuard { lock: self }
    }

    fn unlock(&self) {
        let current = std::thread::current().id();
        let mut state = self.state.lock().unwrap_or_else(|err| err.into_inner());
        debug_assert_eq!(
            state.owner,
            Some(current),
            "scenario process lock released by non-owner thread"
        );
        if state.depth > 1 {
            state.depth -= 1;
            return;
        }
        state.owner = None;
        state.depth = 0;
        self.condvar.notify_all();
    }
}

impl Drop for ScenarioProcessGuard<'_> {
    fn drop(&mut self) {
        self.lock.unlock();
    }
}

fn scenario_process_lock() -> &'static ScenarioProcessLock {
    static LOCK: OnceLock<ScenarioProcessLock> = OnceLock::new();
    LOCK.get_or_init(ScenarioProcessLock::new)
}

pub(crate) fn acquire_scenario_process_guard() -> ScenarioProcessGuard<'static> {
    scenario_process_lock().lock()
}
