use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_millis() as u64)
        .unwrap_or(0)
}

// days since 1970-01-01 to a civil date (Howard Hinnant's algorithm)
fn civil(days: i64) -> (i64, u64, u64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u64;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u64;
    let year = yoe + era * 400 + i64::from(month <= 2);

    (year, month, day)
}

/// `2026-10-15T05:01:02.123Z`, as JavaScript's `toISOString`.
pub fn iso(ms: u64) -> String {
    let secs = ms / 1000;
    let (year, month, day) = civil((secs / 86_400) as i64);
    let rem = secs % 86_400;

    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{:03}Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60,
        ms % 1000
    )
}

/// To the minute, the way handoffs and records are dated.
pub fn minute(ms: u64) -> String {
    format!("{}Z", &iso(ms)[..16])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_matches_javascript() {
        let cases = [
            (0, "1970-01-01T00:00:00.000Z"),
            (951_782_400_000, "2000-02-29T00:00:00.000Z"),
            (1_791_090_062_123, "2026-10-04T05:01:02.123Z"),
            (4_102_444_799_999, "2099-12-31T23:59:59.999Z"),
        ];

        for (ms, expected) in cases {
            assert_eq!(iso(ms), expected);
        }
        assert_eq!(minute(0), "1970-01-01T00:00Z");
    }
}
