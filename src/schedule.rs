/// A parsed cron expression: for each field, the sorted list of values
/// it matches. Day-of-week is always normalized to the 0-6 range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CronSchedule {
    pub minute: Vec<u32>,
    pub hour: Vec<u32>,
    pub day_of_month: Vec<u32>,
    pub month: Vec<u32>,
    pub day_of_week: Vec<u32>,
}

/// Whether the given point in time (as plain calendar fields, no
/// timezone or calendar library involved) satisfies every field of the
/// schedule. `day_of_week` uses 0 = Sunday .. 6 = Saturday.
pub fn matches(
    schedule: &CronSchedule,
    minute: u32,
    hour: u32,
    day_of_month: u32,
    month: u32,
    day_of_week: u32,
) -> bool {
    schedule.minute.contains(&minute)
        && schedule.hour.contains(&hour)
        && schedule.day_of_month.contains(&day_of_month)
        && schedule.month.contains(&month)
        && schedule.day_of_week.contains(&(day_of_week % 7))
}

/// A plain-English rundown of what the schedule matches, one line per
/// field, in the fixed order minute/hour/day-of-month/month/day-of-week.
pub fn describe(schedule: &CronSchedule) -> String {
    format!(
        "minute: {}\nhour: {}\nday of month: {}\nmonth: {}\nday of week: {}",
        describe_field(&schedule.minute, 60),
        describe_field(&schedule.hour, 24),
        describe_field(&schedule.day_of_month, 31),
        describe_field(&schedule.month, 12),
        describe_field(&schedule.day_of_week, 7),
    )
}

fn describe_field(values: &[u32], full_count: usize) -> String {
    if values.len() == full_count {
        "every value".to_string()
    } else {
        values
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schedule(minute: Vec<u32>, hour: Vec<u32>, day_of_month: Vec<u32>, month: Vec<u32>, day_of_week: Vec<u32>) -> CronSchedule {
        CronSchedule { minute, hour, day_of_month, month, day_of_week }
    }

    #[test]
    fn matches_requires_every_field_to_agree() {
        let s = schedule(vec![30], vec![9], vec![1], vec![1], vec![0, 1, 2, 3, 4, 5, 6]);
        assert!(matches(&s, 30, 9, 1, 1, 3));
        assert!(!matches(&s, 31, 9, 1, 1, 3));
    }

    #[test]
    fn day_of_week_wraps_modulo_seven() {
        let s = schedule(vec![0], vec![0], vec![1], vec![1], vec![0]);
        assert!(matches(&s, 0, 0, 1, 1, 7));
    }

    #[test]
    fn describe_reports_full_range_as_every_value() {
        let s = schedule((0..60).collect(), vec![9], (1..=31).collect(), (1..=12).collect(), vec![1]);
        let text = describe(&s);
        assert!(text.contains("minute: every value"));
        assert!(text.contains("day of month: every value"));
        assert!(text.contains("month: every value"));
        assert!(text.contains("day of week: 1"));
    }

    #[test]
    fn describe_lists_specific_values() {
        let s = schedule(vec![0, 30], vec![9], vec![1], vec![1], vec![1, 2, 3, 4, 5]);
        assert_eq!(describe(&s).lines().next().unwrap(), "minute: 0,30");
    }
}
