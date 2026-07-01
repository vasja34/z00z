use super::{SystemTimeProvider, TimeProvider};

/// Generate timestamp string in format yyyymmdd_hhmmss.
pub(super) fn generate_timestamp() -> String {
    let time_provider = SystemTimeProvider;
    let total_secs = time_provider.compat_unix_timestamp_millis() / 1000;

    const SECONDS_PER_DAY: u64 = 86400;
    const SECONDS_PER_HOUR: u64 = 3600;
    const SECONDS_PER_MINUTE: u64 = 60;

    let days_since_epoch = total_secs / SECONDS_PER_DAY;
    let seconds_today = total_secs % SECONDS_PER_DAY;
    let hours = seconds_today / SECONDS_PER_HOUR;
    let minutes = (seconds_today % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE;
    let seconds = seconds_today % SECONDS_PER_MINUTE;

    let mut year = 1970u64;
    let mut remaining_days = days_since_epoch;
    loop {
        let is_leap =
            year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
        let days_in_year = if is_leap { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let is_leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let days_in_months: [u64; 12] = if is_leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1u64;
    let mut day = remaining_days + 1;
    for &days_in_month in &days_in_months {
        if day <= days_in_month {
            break;
        }
        day -= days_in_month;
        month += 1;
    }

    format!(
        "{:04}{:02}{:02}_{:02}{:02}{:02}",
        year, month, day, hours, minutes, seconds
    )
}

#[path = "genesis_output_support.rs"]
mod genesis_output_support;

pub(crate) use self::genesis_output_support::{
    create_timestamped_output_dir, prepare_genesis_logging_dir, prepare_genesis_snapshot_root,
    write_genesis_report, GenesisReportArgs,
};

#[cfg(not(target_arch = "wasm32"))]
pub(crate) use self::genesis_output_support::create_genesis_snapshot_zip;
