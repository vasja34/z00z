use std::{
    path::Path,
    sync::{Mutex, MutexGuard, OnceLock},
};

use z00z_utils::io::path_exists;

pub fn assert_absent(path: &Path) {
    assert!(
        !path_exists(path).expect("path_exists"),
        "path must be absent: {}",
        path.display()
    );
}

pub fn lock_stage4_tamper() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|err| err.into_inner())
}
