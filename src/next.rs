use crate::datetime::DateTime;
use crate::schedule::{matches, CronSchedule};

// About four years of minutes: long enough to see every weekday, month,
// and leap-year combination at least once, but bounded so a schedule
// that can never actually fire (day-of-month 31 in February, say)
// doesn't search forever.
const MAX_MINUTES_SEARCHED: u32 = 4 * 366 * 24 * 60;

/// Finds the next `count` times, strictly after `start`, at which
/// `schedule` fires. Returns fewer than `count` entries if the search
/// window runs out first.
pub fn next_runs(schedule: &CronSchedule, start: DateTime, count: usize) -> Vec<DateTime> {
    let mut results = Vec::with_capacity(count);
    let mut current = start;
    current.add_minute();

    for _ in 0..MAX_MINUTES_SEARCHED {
        if results.len() >= count {
            break;
        }
        if matches(
            schedule,
            current.minute,
            current.hour,
            current.day,
            current.month,
            current.weekday(),
        ) {
            results.push(current);
        }
        current.add_minute();
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_cron_expression;

    #[test]
    fn finds_next_matching_minutes() {
        let schedule = parse_cron_expression("30 9 * * *").unwrap();
        let start = DateTime::new(2024, 1, 1, 0, 0).unwrap();
        let runs = next_runs(&schedule, start, 3);
        assert_eq!(
            runs,
            vec![
                DateTime::new(2024, 1, 1, 9, 30).unwrap(),
                DateTime::new(2024, 1, 2, 9, 30).unwrap(),
                DateTime::new(2024, 1, 3, 9, 30).unwrap(),
            ]
        );
    }

    #[test]
    fn search_starts_strictly_after_the_given_minute() {
        let schedule = parse_cron_expression("0 * * * *").unwrap();
        let start = DateTime::new(2024, 1, 1, 9, 0).unwrap();
        let runs = next_runs(&schedule, start, 1);
        assert_eq!(runs, vec![DateTime::new(2024, 1, 1, 10, 0).unwrap()]);
    }

    #[test]
    fn impossible_schedule_returns_fewer_than_requested() {
        // Day-of-month 31 in February never happens.
        let schedule = parse_cron_expression("0 0 31 2 *").unwrap();
        let start = DateTime::new(2024, 1, 1, 0, 0).unwrap();
        let runs = next_runs(&schedule, start, 5);
        assert!(runs.is_empty());
    }

    #[test]
    fn respects_weekday_field() {
        let schedule = parse_cron_expression("0 12 * * 1").unwrap(); // every Monday at noon
        let start = DateTime::new(2024, 3, 1, 0, 0).unwrap(); // a Friday
        let runs = next_runs(&schedule, start, 1);
        assert_eq!(runs[0].weekday(), 1);
    }
}
