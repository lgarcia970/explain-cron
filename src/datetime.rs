use std::time::{SystemTime, UNIX_EPOCH};

/// A plain calendar timestamp with minute resolution, always UTC.
/// No timezone or DST handling - cron itself has no concept of either.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateTime {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
}

const WEEKDAY_NAMES: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

impl DateTime {
    pub fn new(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> Option<Self> {
        if !(1..=12).contains(&month)
            || day == 0
            || day > days_in_month(year, month)
            || hour > 23
            || minute > 59
        {
            return None;
        }
        Some(DateTime { year, month, day, hour, minute })
    }

    /// The current time, read from the system clock and truncated to the
    /// minute (cron has no finer resolution than that).
    pub fn now() -> Self {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let days = secs.div_euclid(86_400);
        let remainder = secs.rem_euclid(86_400);
        let (year, month, day) = civil_from_days(days);
        DateTime {
            year,
            month,
            day,
            hour: (remainder / 3600) as u32,
            minute: (remainder / 60 % 60) as u32,
        }
    }

    /// Parses "YYYY-MM-DD HH:MM".
    pub fn parse(text: &str) -> Result<Self, String> {
        let (date_part, time_part) = text
            .split_once(' ')
            .ok_or_else(|| format!("'{text}' is not in the form YYYY-MM-DD HH:MM"))?;

        let mut date_fields = date_part.split('-');
        let (Some(y), Some(m), Some(d), None) =
            (date_fields.next(), date_fields.next(), date_fields.next(), date_fields.next())
        else {
            return Err(format!("'{date_part}' is not a YYYY-MM-DD date"));
        };
        let mut time_fields = time_part.split(':');
        let (Some(hh), Some(mm), None) = (time_fields.next(), time_fields.next(), time_fields.next())
        else {
            return Err(format!("'{time_part}' is not an HH:MM time"));
        };

        let year: i32 = y.parse().map_err(|_| format!("'{y}' is not a valid year"))?;
        let month: u32 = m.parse().map_err(|_| format!("'{m}' is not a valid month"))?;
        let day: u32 = d.parse().map_err(|_| format!("'{d}' is not a valid day"))?;
        let hour: u32 = hh.parse().map_err(|_| format!("'{hh}' is not a valid hour"))?;
        let minute: u32 = mm.parse().map_err(|_| format!("'{mm}' is not a valid minute"))?;

        DateTime::new(year, month, day, hour, minute)
            .ok_or_else(|| format!("'{text}' is not a valid date and time"))
    }

    pub fn weekday(&self) -> u32 {
        weekday_from_days(days_from_civil(self.year, self.month, self.day))
    }

    /// Advances the clock by exactly one minute, rolling over hour, day,
    /// month, and year as needed.
    pub fn add_minute(&mut self) {
        self.minute += 1;
        if self.minute == 60 {
            self.minute = 0;
            self.hour += 1;
        }
        if self.hour == 24 {
            self.hour = 0;
            self.day += 1;
        }
        if self.day > days_in_month(self.year, self.month) {
            self.day = 1;
            self.month += 1;
        }
        if self.month == 13 {
            self.month = 1;
            self.year += 1;
        }
    }
}

impl std::fmt::Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {:04}-{:02}-{:02} {:02}:{:02}",
            WEEKDAY_NAMES[self.weekday() as usize],
            self.year,
            self.month,
            self.day,
            self.hour,
            self.minute,
        )
    }
}

pub fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

pub fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if is_leap_year(year) { 29 } else { 28 },
        _ => 30, // unreachable for validated input; avoids a panic on bad data
    }
}

/// Civil calendar date to days since 1970-01-01. Howard Hinnant's
/// `days_from_civil` algorithm: exact, integer-only, valid for any
/// proleptic Gregorian date.
fn days_from_civil(y: i32, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y as i64 - 1 } else { y as i64 };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = if m > 2 { m as i64 - 3 } else { m as i64 + 9 };
    let doy = (153 * mp + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The inverse of `days_from_civil`.
fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m, d)
}

fn weekday_from_days(z: i64) -> u32 {
    ((z % 7 + 11) % 7) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_is_a_thursday() {
        let dt = DateTime::new(1970, 1, 1, 0, 0).unwrap();
        assert_eq!(dt.weekday(), 4);
    }

    #[test]
    fn known_date_matches_known_weekday() {
        // 2000-01-01 was a Saturday.
        let dt = DateTime::new(2000, 1, 1, 0, 0).unwrap();
        assert_eq!(dt.weekday(), 6);
    }

    #[test]
    fn leap_year_detection() {
        assert!(is_leap_year(2000));
        assert!(!is_leap_year(1900));
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(2023));
    }

    #[test]
    fn add_minute_rolls_over_hour() {
        let mut dt = DateTime::new(2024, 1, 1, 23, 59).unwrap();
        dt.add_minute();
        assert_eq!(dt, DateTime::new(2024, 1, 2, 0, 0).unwrap());
    }

    #[test]
    fn add_minute_rolls_over_month_end() {
        let mut dt = DateTime::new(2024, 2, 29, 23, 59).unwrap();
        dt.add_minute();
        assert_eq!(dt, DateTime::new(2024, 3, 1, 0, 0).unwrap());
    }

    #[test]
    fn add_minute_rolls_over_year() {
        let mut dt = DateTime::new(2024, 12, 31, 23, 59).unwrap();
        dt.add_minute();
        assert_eq!(dt, DateTime::new(2025, 1, 1, 0, 0).unwrap());
    }

    #[test]
    fn non_leap_february_rejects_the_29th() {
        assert!(DateTime::new(2023, 2, 29, 0, 0).is_none());
    }

    #[test]
    fn parse_reads_valid_datetime() {
        let dt = DateTime::parse("2024-03-05 09:30").unwrap();
        assert_eq!(dt, DateTime::new(2024, 3, 5, 9, 30).unwrap());
    }

    #[test]
    fn parse_rejects_malformed_input() {
        assert!(DateTime::parse("2024-03-05").is_err());
        assert!(DateTime::parse("2024-13-05 09:30").is_err());
        assert!(DateTime::parse("2024-03-05 25:00").is_err());
    }
}
