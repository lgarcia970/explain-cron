use std::fmt;

use crate::schedule::CronSchedule;

/// Everything that can go wrong turning cron text into a `CronSchedule`.
/// Kept as data (not a formatted string) so callers can match on the
/// specific problem instead of scraping messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CronError {
    WrongFieldCount { found: usize },
    EmptyPart { field: &'static str },
    NotANumber { field: &'static str, text: String },
    OutOfRange { field: &'static str, value: u32, min: u32, max: u32 },
    BackwardsRange { field: &'static str, start: u32, end: u32 },
    ZeroStep { field: &'static str },
}

impl fmt::Display for CronError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CronError::WrongFieldCount { found } => write!(
                f,
                "expected 5 fields (minute hour day-of-month month day-of-week), found {found}"
            ),
            CronError::EmptyPart { field } => write!(f, "{field}: empty value"),
            CronError::NotANumber { field, text } => {
                write!(f, "{field}: '{text}' is not a whole number")
            }
            CronError::OutOfRange { field, value, min, max } => {
                write!(f, "{field}: {value} is out of range ({min}-{max})")
            }
            CronError::BackwardsRange { field, start, end } => {
                write!(f, "{field}: range {start}-{end} goes backwards")
            }
            CronError::ZeroStep { field } => write!(f, "{field}: step of 0 is not allowed"),
        }
    }
}

impl std::error::Error for CronError {}

/// Parses one comma-separated cron field (e.g. "1-5,10,*/15") into the
/// sorted, deduplicated list of values it selects within `[min, max]`.
pub fn parse_field(spec: &str, min: u32, max: u32, field: &'static str) -> Result<Vec<u32>, CronError> {
    let mut values = Vec::new();
    for part in spec.split(',') {
        values.extend(parse_part(part, min, max, field)?);
    }
    values.sort_unstable();
    values.dedup();
    Ok(values)
}

fn parse_part(part: &str, min: u32, max: u32, field: &'static str) -> Result<Vec<u32>, CronError> {
    if part.is_empty() {
        return Err(CronError::EmptyPart { field });
    }

    let (range_part, step) = match part.split_once('/') {
        Some((r, s)) => {
            let step: u32 = s
                .parse()
                .map_err(|_| CronError::NotANumber { field, text: s.to_string() })?;
            if step == 0 {
                return Err(CronError::ZeroStep { field });
            }
            (r, step)
        }
        None => (part, 1),
    };

    let (start, end) = if range_part == "*" {
        (min, max)
    } else if let Some((a, b)) = range_part.split_once('-') {
        let start = parse_number(a, min, max, field)?;
        let end = parse_number(b, min, max, field)?;
        if start > end {
            return Err(CronError::BackwardsRange { field, start, end });
        }
        (start, end)
    } else {
        let value = parse_number(range_part, min, max, field)?;
        (value, value)
    };

    Ok((start..=end).step_by(step as usize).collect())
}

fn parse_number(text: &str, min: u32, max: u32, field: &'static str) -> Result<u32, CronError> {
    let value: u32 = text
        .parse()
        .map_err(|_| CronError::NotANumber { field, text: text.to_string() })?;
    if value < min || value > max {
        return Err(CronError::OutOfRange { field, value, min, max });
    }
    Ok(value)
}

/// Parses a full five-field cron expression. Day-of-week accepts both 0
/// and 7 for Sunday; 7 is folded down to 0 so callers only ever see 0-6.
pub fn parse_cron_expression(expr: &str) -> Result<CronSchedule, CronError> {
    let fields: Vec<&str> = expr.split_whitespace().collect();
    if fields.len() != 5 {
        return Err(CronError::WrongFieldCount { found: fields.len() });
    }

    let minute = parse_field(fields[0], 0, 59, "minute")?;
    let hour = parse_field(fields[1], 0, 23, "hour")?;
    let day_of_month = parse_field(fields[2], 1, 31, "day-of-month")?;
    let month = parse_field(fields[3], 1, 12, "month")?;

    let mut day_of_week = parse_field(fields[4], 0, 7, "day-of-week")?;
    for d in day_of_week.iter_mut() {
        if *d == 7 {
            *d = 0;
        }
    }
    day_of_week.sort_unstable();
    day_of_week.dedup();

    Ok(CronSchedule { minute, hour, day_of_month, month, day_of_week })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wildcard_covers_full_range() {
        assert_eq!(parse_field("*", 0, 3, "test").unwrap(), vec![0, 1, 2, 3]);
    }

    #[test]
    fn list_is_sorted_and_deduplicated() {
        assert_eq!(parse_field("5,1,5,3", 0, 59, "minute").unwrap(), vec![1, 3, 5]);
    }

    #[test]
    fn range_is_inclusive() {
        assert_eq!(parse_field("2-5", 0, 59, "minute").unwrap(), vec![2, 3, 4, 5]);
    }

    #[test]
    fn step_applies_within_range() {
        assert_eq!(parse_field("0-10/3", 0, 59, "minute").unwrap(), vec![0, 3, 6, 9]);
    }

    #[test]
    fn step_applies_to_wildcard() {
        assert_eq!(parse_field("*/15", 0, 59, "minute").unwrap(), vec![0, 15, 30, 45]);
    }

    #[test]
    fn backwards_range_is_rejected() {
        assert_eq!(
            parse_field("10-5", 0, 59, "minute"),
            Err(CronError::BackwardsRange { field: "minute", start: 10, end: 5 })
        );
    }

    #[test]
    fn out_of_range_value_is_rejected() {
        assert_eq!(
            parse_field("60", 0, 59, "minute"),
            Err(CronError::OutOfRange { field: "minute", value: 60, min: 0, max: 59 })
        );
    }

    #[test]
    fn zero_step_is_rejected() {
        assert_eq!(parse_field("*/0", 0, 59, "minute"), Err(CronError::ZeroStep { field: "minute" }));
    }

    #[test]
    fn wrong_field_count_is_rejected() {
        assert_eq!(
            parse_cron_expression("* * * *"),
            Err(CronError::WrongFieldCount { found: 4 })
        );
    }

    #[test]
    fn sunday_seven_folds_to_zero() {
        let schedule = parse_cron_expression("0 0 * * 7").unwrap();
        assert_eq!(schedule.day_of_week, vec![0]);
    }

    #[test]
    fn full_expression_parses_each_field() {
        let schedule = parse_cron_expression("*/15 9-17 1,15 * 1-5").unwrap();
        assert_eq!(schedule.minute, vec![0, 15, 30, 45]);
        assert_eq!(schedule.hour, (9..=17).collect::<Vec<_>>());
        assert_eq!(schedule.day_of_month, vec![1, 15]);
        assert_eq!(schedule.month, (1..=12).collect::<Vec<_>>());
        assert_eq!(schedule.day_of_week, vec![1, 2, 3, 4, 5]);
    }
}
