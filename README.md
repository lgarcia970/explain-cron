# explain-cron

Cron syntax is compact enough that it's easy to write an expression that
doesn't mean what you think it means. `0 0 * * 1-5/2` looks like "every
other weekday" but the step is just the range `1-5` sampled every 2 steps,
so it fires on Monday, Wednesday, and Friday, not on alternating weeks or
anything calendar-aware. This tool parses a five-field cron expression,
tells you exactly what it will run on, or tells you why it doesn't parse.

## Usage

```
$ explain-cron "*/15 9-17 * * 1-5"
minute: 0,15,30,45
hour: 9,10,11,12,13,14,15,16,17
day of month: every value
month: every value
day of week: 1,2,3,4,5

$ explain-cron "0 0 1 1 *"
minute: 0
hour: 0
day of month: 1
month: 1
day of week: every value

$ explain-cron "70 * * * *"
error: minute: 70 is out of range (0-59)
```

The expression can be passed as one quoted string or as five separate
arguments; both are joined with spaces before parsing.

## Supported syntax

- `*` - every value
- `5` - a single value
- `1-5` - an inclusive range
- `1,5,10` - a list
- `*/15` or `1-30/5` - a step applied to a wildcard or a range

Day-of-week accepts both `0` and `7` for Sunday. Field ranges follow
standard cron: minute 0-59, hour 0-23, day of month 1-31, month 1-12,
day of week 0-7.

Not yet supported: named months/days (`JAN`, `MON`), the `@daily` /
`@hourly` style shorthands, and the `?` placeholder some implementations
use for day fields.

## Design

The parsing and matching logic lives in `src/parser.rs` and
`src/schedule.rs` as plain functions with no I/O: given a string (or a
`CronSchedule` plus some calendar fields), they return a value or an
error, nothing else. `src/main.rs` is the only place that touches
`std::env` or a terminal. That split is what makes the logic worth unit
testing - the tests in each module call the parsing and matching
functions directly, no process spawning or fixture files required.

## Building

Standard library only, no dependencies to fetch:

```
cargo build --release
```

## License

MIT, see LICENSE.
