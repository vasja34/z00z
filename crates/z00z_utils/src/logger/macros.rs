//! Logging macros for JSON payloads via `Logger`.

/// Log structured JSON at info level.
#[macro_export]
macro_rules! log_info {
    ($logger:expr, $($key:ident = $value:expr),+ $(,)?) => {{
        let payload = $crate::codec::json!({
            $(stringify!($key): $value),+
        });
        ($logger).info(&payload.to_string());
    }};
}

/// Log structured JSON at warn level.
#[macro_export]
macro_rules! log_warn {
    ($logger:expr, $($key:ident = $value:expr),+ $(,)?) => {{
        let payload = $crate::codec::json!({
            $(stringify!($key): $value),+
        });
        ($logger).warn(&payload.to_string());
    }};
}

/// Log structured JSON at error level.
#[macro_export]
macro_rules! log_error {
    ($logger:expr, $($key:ident = $value:expr),+ $(,)?) => {{
        let payload = $crate::codec::json!({
            $(stringify!($key): $value),+
        });
        ($logger).error(&payload.to_string());
    }};
}

/// Log structured JSON at debug level.
#[macro_export]
macro_rules! log_debug {
    ($logger:expr, $($key:ident = $value:expr),+ $(,)?) => {{
        let payload = $crate::codec::json!({
            $(stringify!($key): $value),+
        });
        ($logger).debug(&payload.to_string());
    }};
}

/// Log structured JSON at trace level.
#[macro_export]
macro_rules! log_trace {
    ($logger:expr, $($key:ident = $value:expr),+ $(,)?) => {{
        let payload = $crate::codec::json!({
            $(stringify!($key): $value),+
        });
        ($logger).trace(&payload.to_string());
    }};
}
