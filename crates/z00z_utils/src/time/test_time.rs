//! TimeProvider trait tests

use crate::time::{MockTimeProvider, SystemTimeProvider, TimeProvider};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};

#[test]
fn test_system_time_provider_returns() {
    let time = SystemTimeProvider;
    let now = time.now();
    assert!(now > SystemTime::UNIX_EPOCH);
}

#[test]
fn test_system_time_provider_timestamp() {
    let time = SystemTimeProvider;
    let ts1 = time.compat_unix_timestamp();
    thread::sleep(Duration::from_millis(10));
    let ts2 = time.compat_unix_timestamp();
    assert!(ts2 >= ts1);
}

#[test]
fn test_system_time_timestamp_millis() {
    let time = SystemTimeProvider;
    let ts_secs = time.compat_unix_timestamp() as f64;
    let ts_millis = time.compat_unix_timestamp_millis() as f64 / 1000.0;
    // Should be very close
    assert!((ts_secs - ts_millis).abs() < 1.0);
}

#[test]
fn test_mock_time_provider_deterministic() {
    let time = MockTimeProvider::new(SystemTime::UNIX_EPOCH);
    assert_eq!(time.compat_unix_timestamp(), 0);
    assert_eq!(time.compat_unix_timestamp(), 0);
    assert_eq!(time.compat_unix_timestamp(), 0);
}

#[test]
fn test_mock_time_provider_controllable() {
    let time = MockTimeProvider::new(SystemTime::UNIX_EPOCH);
    time.advance_by(Duration::from_secs(100));
    assert_eq!(time.compat_unix_timestamp(), 100);
    time.set_time(SystemTime::UNIX_EPOCH + Duration::from_secs(200));
    assert_eq!(time.compat_unix_timestamp(), 200);
}

#[test]
fn test_mock_time_provider_micros() {
    let time = MockTimeProvider::new(SystemTime::UNIX_EPOCH);
    time.advance_by(Duration::from_micros(123));
    assert_eq!(time.compat_unix_timestamp_micros(), 123);
}

#[test]
fn test_mock_time_provider_object() {
    let _: Box<dyn TimeProvider> = Box::new(SystemTimeProvider);
    let _: Box<dyn TimeProvider> = Box::new(MockTimeProvider::new(SystemTime::UNIX_EPOCH));
}

#[test]
fn test_mock_time_provider_thread() {
    let time = Arc::new(MockTimeProvider::new(SystemTime::UNIX_EPOCH));
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let time = Arc::clone(&time);
            thread::spawn(move || {
                time.advance_by(Duration::from_secs(i));
                time.compat_unix_timestamp()
            })
        })
        .collect();

    for handle in handles {
        let _ = handle.join().unwrap();
    }
}

#[test]
fn test_mock_time_provider_default() {
    let time = MockTimeProvider::default();
    assert_eq!(time.compat_unix_timestamp(), 0);
}

#[test]
fn test_mock_time_provider_clone() {
    let time1 = MockTimeProvider::new(SystemTime::UNIX_EPOCH);
    let time2 = time1.clone();

    time1.advance_by(Duration::from_secs(50));
    assert_eq!(time2.compat_unix_timestamp(), 50);

    time2.advance_by(Duration::from_secs(25));
    assert_eq!(time1.compat_unix_timestamp(), 75);
}

#[test]
fn test_timestamp_pre_epoch_zero() {
    let initial = SystemTime::UNIX_EPOCH
        .checked_sub(Duration::from_secs(1))
        .unwrap();
    let time = MockTimeProvider::new(initial);

    assert_eq!(time.compat_unix_timestamp(), 0);
    assert_eq!(time.compat_unix_timestamp_millis(), 0);
    assert_eq!(time.compat_unix_timestamp_micros(), 0);
}

#[test]
fn test_system_time_provider_send() {
    fn test_assert_send_sync_r339<T: Send + Sync>() {}
    test_assert_send_sync_r339::<SystemTimeProvider>();
    test_assert_send_sync_r339::<MockTimeProvider>();
}

#[test]
fn test_try_unix_timestamp_success() {
    let time = SystemTimeProvider;
    let ts = time.try_unix_timestamp().unwrap();
    assert!(ts > 0);
}

#[test]
fn test_mock_time_before_epoch() {
    let initial = SystemTime::UNIX_EPOCH
        .checked_sub(Duration::from_secs(100))
        .unwrap();
    let time = MockTimeProvider::new(initial);

    let result = time.try_unix_timestamp();
    assert!(result.is_err());
}

#[test]
fn test_unix_timestamp_micros_success() {
    let time = SystemTimeProvider;
    let ts = time.try_unix_timestamp_micros().unwrap();
    assert!(ts > 0);
}

#[test]
fn test_time_pre_epoch_micros() {
    let initial = SystemTime::UNIX_EPOCH
        .checked_sub(Duration::from_secs(100))
        .unwrap();
    let time = MockTimeProvider::new(initial);

    let result = time.try_unix_timestamp_micros();
    assert!(result.is_err());
}

#[test]
fn test_format_unix_timestamp_millis() {
    // 1970-01-01 00:00:00.000 UTC
    let s = crate::time::format_unix_timestamp_millis_utc(0);
    assert_eq!(s, "1970-01-01 00:00:00.000");
}

#[test]
fn test_timestamp_secs_utc_renders() {
    let s = crate::time::format_unix_timestamp_secs_utc(0);
    assert_eq!(s, "1970-01-01 00:00:00.000");
}

#[test]
fn test_timestamp_millis_utc_compact() {
    // 1970-01-01 00:00:00 UTC
    let s = crate::time::format_unix_timestamp_milliseconds_compact(0);
    assert_eq!(s, "19700101_000000");
}

#[test]
fn test_system_time_local_shape() {
    let s = crate::time::format_system_time_local(SystemTime::UNIX_EPOCH);
    assert_eq!(s.len(), 23);
    assert_eq!(s.as_bytes()[4], b'-');
    assert_eq!(s.as_bytes()[7], b'-');
    assert_eq!(s.as_bytes()[10], b' ');
    assert_eq!(s.as_bytes()[13], b':');
    assert_eq!(s.as_bytes()[16], b':');
    assert_eq!(s.as_bytes()[19], b'.');
}
