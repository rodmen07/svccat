//! Dependency-free UTC timestamp formatting shared by the snapshot listing
//! and the SPDX SBOM renderer.

/// Format seconds since the Unix epoch as an ISO 8601 UTC timestamp,
/// e.g. `2024-02-29T00:00:00Z`.
pub fn iso8601_utc(secs: u64) -> String {
    let (y, mo, d, h, m, s) = ymd_hms(secs);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

/// Format seconds since the Unix epoch as a human-readable UTC timestamp,
/// e.g. `2024-02-29 00:00:00 UTC`.
pub fn human_utc(secs: u64) -> String {
    let (y, mo, d, h, m, s) = ymd_hms(secs);
    format!("{y:04}-{mo:02}-{d:02} {h:02}:{m:02}:{s:02} UTC")
}

fn ymd_hms(secs: u64) -> (u64, u64, u64, u64, u64, u64) {
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let (y, mo, d) = days_to_ymd(secs / 86400);
    (y, mo, d, h, m, s)
}

fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    // Gregorian calendar approximation from day count since 1970-01-01.
    let mut year = 1970u64;
    loop {
        let leap = is_leap(year);
        let days_in_year = if leap { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }
    let leap = is_leap(year);
    let months = if leap {
        [31u64, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31u64, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month = 1u64;
    for &days_in_month in &months {
        if days < days_in_month {
            break;
        }
        days -= days_in_month;
        month += 1;
    }
    (year, month, days + 1)
}

#[allow(clippy::manual_is_multiple_of)]
fn is_leap(y: u64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso8601_epoch_start() {
        assert_eq!(iso8601_utc(0), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn iso8601_leap_day() {
        assert_eq!(iso8601_utc(1709164800), "2024-02-29T00:00:00Z");
    }

    #[test]
    fn iso8601_last_second_of_first_day() {
        assert_eq!(iso8601_utc(86399), "1970-01-01T23:59:59Z");
    }

    #[test]
    fn human_utc_matches_old_snapshot_format() {
        // Byte-parity regression guard for `svccat snapshot list` output.
        assert_eq!(human_utc(1709164800), "2024-02-29 00:00:00 UTC");
    }
}
