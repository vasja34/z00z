//! Integration tests for OS-hardening (best-effort, cross-platform).

#[cfg(all(unix, not(target_os = "ios"), not(miri)))]
use libc::{getrlimit, rlimit, RLIMIT_CORE};
#[cfg(all(any(target_os = "linux", target_os = "android"), not(miri)))]
use libc::{prctl, PR_GET_DUMPABLE};
#[cfg(not(miri))]
use z00z_utils::os_hardening::apply_best_effort;
use z00z_utils::os_hardening::{lock_bytes_best_effort, OwnedLockedBytes};

struct OwnedHolder {
    secret: OwnedLockedBytes<32>,
}

#[test]
#[cfg(not(miri))]
fn test_hardening_apply_best_effort() {
    // Should never panic, regardless of platform
    let report = apply_best_effort();
    // Verify struct fields exist
    let _ = report.core_dumps_disabled;
    let _ = report.non_dumpable;
    let _ = report.notes;

    #[cfg(all(unix, not(target_os = "ios")))]
    if report.core_dumps_disabled {
        let mut limit = rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        // SAFETY: `getrlimit` writes into a valid stack-allocated `rlimit`.
        let rc = unsafe { getrlimit(RLIMIT_CORE, &mut limit) };
        assert_eq!(rc, 0);
        assert_eq!(limit.rlim_cur, 0);
        assert_eq!(limit.rlim_max, 0);
    }

    #[cfg(any(target_os = "linux", target_os = "android"))]
    if report.non_dumpable {
        // SAFETY: `prctl(PR_GET_DUMPABLE, ..)` reads process state only.
        let dumpable = unsafe { prctl(PR_GET_DUMPABLE, 0, 0, 0, 0) };
        assert_eq!(dumpable, 0);
    }
}
#[test]
fn test_locked_bytes_holder_shape() {
    let holder = OwnedHolder {
        secret: OwnedLockedBytes::new_best_effort_with(|slot| slot.fill(0x55)),
    };

    assert_eq!(holder.secret.reveal(), &[0x55u8; 32]);

    let debug = format!("{:?}", holder.secret);
    assert!(debug.contains("OwnedLockedBytes"));
    assert!(!debug.contains("0x55"));
    assert!(!debug.contains("ptr"));
}
#[test]
fn test_lock_bytes_empty_slice() {
    // Empty slice should return None
    let result = lock_bytes_best_effort(&mut []);
    assert!(result.is_none());
}
