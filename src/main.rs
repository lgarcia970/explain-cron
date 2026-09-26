use std::env;
use std::process::ExitCode;

use explain_cron::DateTime;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        print_usage();
        return ExitCode::from(2);
    }

    let mut next_count: Option<usize> = None;
    let mut from: Option<String> = None;
    let mut expression_parts: Vec<String> = Vec::new();

    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--next" => {
                let Some(value) = args.next() else {
                    eprintln!("error: --next requires a number");
                    return ExitCode::from(2);
                };
                let Ok(count) = value.parse() else {
                    eprintln!("error: '{value}' is not a valid count for --next");
                    return ExitCode::from(2);
                };
                next_count = Some(count);
            }
            "--from" => {
                let Some(value) = args.next() else {
                    eprintln!("error: --from requires a \"YYYY-MM-DD HH:MM\" value");
                    return ExitCode::from(2);
                };
                from = Some(value);
            }
            other => expression_parts.push(other.to_string()),
        }
    }

    if expression_parts.is_empty() {
        print_usage();
        return ExitCode::from(2);
    }

    // Accept the expression either quoted as one argument or as five
    // separate ones, so `explain-cron */15 9-17 '*' '*' 1-5` also works.
    let expression = expression_parts.join(" ");

    let schedule = match explain_cron::parse_cron_expression(&expression) {
        Ok(schedule) => schedule,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };

    println!("{}", explain_cron::describe(&schedule));

    if let Some(count) = next_count {
        let start = match from {
            Some(text) => match DateTime::parse(&text) {
                Ok(dt) => dt,
                Err(err) => {
                    eprintln!("error: {err}");
                    return ExitCode::FAILURE;
                }
            },
            None => DateTime::now(),
        };

        println!("\nnext {count} run(s) after {start}:");
        for run in explain_cron::next_runs(&schedule, start, count) {
            println!("{run}");
        }
    }

    ExitCode::SUCCESS
}

fn print_usage() {
    eprintln!(
        "usage: explain-cron [--next N] [--from \"YYYY-MM-DD HH:MM\"] \"<minute> <hour> <day-of-month> <month> <day-of-week>\""
    );
    eprintln!("example: explain-cron --next 3 \"*/15 9-17 * * 1-5\"");
}
