//! Logger trait and implementations
//!
//! This module provides a logging abstraction to enable dependency injection
//! of logger implementations. The goal is to eliminate hardcoded `tracing::`
//! calls throughout the codebase.
//!
//! # Examples
//!
//! ```
//! use z00z_utils::logger::{Logger, TracingLogger};
//!
//! let logger = TracingLogger;
//! logger.info("Starting asset registry");
//! logger.debug("Debug message");
//! ```

mod file_logger;
mod macros;
mod noop;
mod rotating_file_logger;
mod stdout;
mod structured;
mod tracing_logger;
mod traits;
mod vec_logger;

fn sanitize_message(msg: &str) -> String {
    use std::fmt::Write as _;

    let mut out = String::with_capacity(msg.len());
    let bytes = msg.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == 0x1b {
            if let Some(end) = find_complete_ansi_end(bytes, index) {
                index = end;
                continue;
            }

            out.push_str("\\x1B");
            index += 1;
            continue;
        }

        let ch = msg[index..]
            .chars()
            .next()
            .expect("valid UTF-8 character boundary");

        match ch {
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\0' => {}
            control if control.is_control() => {
                let _ = write!(out, "\\x{:02X}", control as u32);
            }
            other => out.push(other),
        }

        index += ch.len_utf8();
    }

    out
}

fn find_complete_ansi_end(bytes: &[u8], start: usize) -> Option<usize> {
    if start + 1 >= bytes.len() {
        return None;
    }

    match bytes[start + 1] {
        b'[' => find_complete_csi_end(bytes, start + 2),
        b']' => find_complete_osc_end(bytes, start + 2),
        _ => None,
    }
}

fn find_complete_csi_end(bytes: &[u8], mut index: usize) -> Option<usize> {
    while index < bytes.len() {
        let next = bytes[index];
        if (0x40..=0x7e).contains(&next) {
            return Some(index + 1);
        }
        index += 1;
    }

    None
}

fn find_complete_osc_end(bytes: &[u8], mut index: usize) -> Option<usize> {
    while index < bytes.len() {
        match bytes[index] {
            0x07 => return Some(index + 1),
            0x1b if index + 1 < bytes.len() && bytes[index + 1] == b'\\' => {
                return Some(index + 2);
            }
            _ => index += 1,
        }
    }

    None
}

pub use file_logger::FileLogger;
pub use noop::NoopLogger;
pub use rotating_file_logger::{RotatingFileLogger, RotationPolicy};
pub use stdout::StdoutLogger;
pub use structured::{EventLevel, LogEvent, LogLevel, StructuredLogger};
pub use tracing_logger::TracingLogger;
pub use traits::Logger;
pub use vec_logger::VecLogger;

#[cfg(test)]
mod test_logger;
