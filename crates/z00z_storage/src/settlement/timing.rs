use std::{
    cell::RefCell,
    path::PathBuf,
    time::{Duration, Instant},
};

use z00z_utils::io::{create_dir_all, read_to_string, write_file};

use super::hjmt_config::env_opt;

#[derive(Clone)]
struct TimeRow {
    stage: &'static str,
    nanos: u128,
}

thread_local! {
    static TIME_ROWS: RefCell<Vec<TimeRow>> = const { RefCell::new(Vec::new()) };
}

pub(super) fn is_on() -> bool {
    env_opt("Z00Z_SETTLEMENT_TIME_OUT").is_some() && env_opt("Z00Z_SETTLEMENT_TIME_RUN").is_some()
}

pub(super) fn clear() {
    if !is_on() {
        return;
    }

    TIME_ROWS.with(|rows| rows.borrow_mut().clear());
}

pub(super) fn push(stage: &'static str, span: Duration) {
    if !is_on() {
        return;
    }

    TIME_ROWS.with(|rows| {
        rows.borrow_mut().push(TimeRow {
            stage,
            nanos: span.as_nanos(),
        });
    });
}

pub(super) fn run<T>(stage: &'static str, work: impl FnOnce() -> T) -> T {
    if !is_on() {
        return work();
    }

    let mark = Instant::now();
    let out = work();
    push(stage, mark.elapsed());
    out
}

pub(super) fn flush() {
    if !is_on() {
        return;
    }

    let Some(path) = out_path() else {
        return;
    };
    let Some(run) = env_opt("Z00Z_SETTLEMENT_TIME_RUN") else {
        return;
    };

    if let Some(parent) = path.parent() {
        create_dir_all(parent).expect("timing dir");
    }

    let mut body = read_to_string(&path).unwrap_or_default();
    TIME_ROWS.with(|rows| {
        for row in rows.borrow().iter() {
            body.push_str(&format!(
                "run={run}\tstage={}\tns={}\n",
                row.stage, row.nanos
            ));
        }
    });
    write_file(path, body.as_bytes()).expect("timing file");
}

fn out_path() -> Option<PathBuf> {
    env_opt("Z00Z_SETTLEMENT_TIME_OUT").map(PathBuf::from)
}
