//! Time as Unix milliseconds, and Drive's RFC 3339 timestamps (`2026-11-21T10:00:00.000Z`) read
//! and written without a date library.

use std::time::{SystemTime, UNIX_EPOCH};

/// Now, Unix ms.
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
}

/// Reads an RFC 3339 timestamp in UTC (`Z`) or with an offset (`+01:00`), fractions of a second
/// to the millisecond. `None` for anything else, or a time before 1970.
pub fn parse_rfc3339(text: &str) -> Option<u64> {
    let (date, rest) = text.split_once(['T', 't'])?;
    let mut d = date.split('-');
    let year: i64 = d.next()?.parse().ok()?;
    let month: u32 = d.next()?.parse().ok()?;
    let day: u32 = d.next()?.parse().ok()?;
    if d.next().is_some() || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let (clock, offset_min) = if let Some(clock) = rest.strip_suffix(['Z', 'z']) {
        (clock, 0i64)
    } else {
        let at = rest.rfind(['+', '-'])?;
        let (clock, off) = rest.split_at(at);
        let sign = if off.starts_with('-') { -1 } else { 1 };
        let (h, m) = off[1..].split_once(':')?;
        let h: i64 = h.parse().ok()?;
        let m: i64 = m.parse().ok()?;
        (clock, sign * (h * 60 + m))
    };
    let (hms, frac) = clock.split_once('.').unwrap_or((clock, ""));
    let mut t = hms.split(':');
    let hour: i64 = t.next()?.parse().ok()?;
    let minute: i64 = t.next()?.parse().ok()?;
    let second: i64 = t.next()?.parse().ok()?;
    if t.next().is_some() || hour > 23 || minute > 59 || second > 60 {
        return None;
    }
    let millis: i64 = if frac.is_empty() {
        0
    } else {
        if !frac.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        let three: String = frac.chars().chain("000".chars()).take(3).collect();
        three.parse().ok()?
    };
    let days = days_from_civil(year, month, day);
    let secs = days * 86_400 + hour * 3600 + minute * 60 + second - offset_min * 60;
    u64::try_from(secs * 1000 + millis).ok()
}

/// Writes Unix ms as `YYYY-MM-DDTHH:MM:SS.mmmZ`.
pub fn format_rfc3339(ms: u64) -> String {
    let secs = i64::try_from(ms / 1000).unwrap_or(i64::MAX);
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}.{:03}Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60,
        ms % 1000
    )
}

/// Days since 1970-01-01 (Howard Hinnant's algorithm).
fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let m = i64::from(m);
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + i64::from(d) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = u32::try_from(doy - (153 * mp + 2) / 5 + 1).unwrap_or(1);
    let m = u32::try_from(if mp < 10 { mp + 3 } else { mp - 9 }).unwrap_or(1);
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::{format_rfc3339, parse_rfc3339};

    #[test]
    fn reads_drive_timestamps() {
        assert_eq!(parse_rfc3339("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(parse_rfc3339("1970-01-01T00:00:01.5Z"), Some(1500));
        assert_eq!(
            parse_rfc3339("2026-11-21T10:00:00.123Z"),
            Some(1_795_255_200_123)
        );
        assert_eq!(
            parse_rfc3339("2026-11-21T11:00:00.123+01:00"),
            Some(1_795_255_200_123)
        );
        assert_eq!(
            parse_rfc3339("2000-02-29T00:00:00.000Z"),
            Some(951_782_400_000)
        );
        for bad in [
            "",
            "2026-11-21",
            "2026-13-01T00:00:00Z",
            "x-y-zT1:2:3Z",
            "1969-12-31T23:59:59Z",
        ] {
            assert_eq!(parse_rfc3339(bad), None, "{bad}");
        }
    }

    #[test]
    fn writes_what_it_reads() {
        for ms in [
            0,
            1500,
            951_782_400_000,
            1_795_255_200_123,
            4_102_444_799_999,
        ] {
            assert_eq!(parse_rfc3339(&format_rfc3339(ms)), Some(ms), "{ms}");
        }
        assert_eq!(
            format_rfc3339(1_795_255_200_123),
            "2026-11-21T10:00:00.123Z"
        );
    }
}
