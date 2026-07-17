//! UTC timestamp helper shared by the watcher plugins' state and change logs.
//!
//! The format matches Python's
//! `datetime.now(UTC).isoformat(timespec="seconds").replace("+00:00", "Z")`,
//! so state files written by the Python and Rust versions stay comparable.

use chrono::Utc;

/// ISO-8601 UTC timestamp, second precision, `Z` suffix
/// (e.g. `2026-07-17T13:04:05Z`).
pub fn now_utc() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn now_utc_is_iso8601_seconds_with_z_suffix() {
        let ts = now_utc();
        assert_eq!(ts.len(), 20, "YYYY-MM-DDTHH:MM:SSZ is 20 chars, got {ts:?}");
        assert!(ts.ends_with('Z'), "must end with Z: {ts:?}");
        assert!(ts.contains('T'), "must have date/time separator T: {ts:?}");
        assert!(!ts.contains('+'), "no timezone offset: {ts:?}");
        assert!(!ts.contains('.'), "no fractional seconds: {ts:?}");
    }
}
