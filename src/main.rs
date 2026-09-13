use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("usage: explain-cron \"<minute> <hour> <day-of-month> <month> <day-of-week>\"");
        eprintln!("example: explain-cron \"*/15 9-17 * * 1-5\"");
        return ExitCode::from(2);
    }

    // Accept the expression either quoted as one argument or as five
    // separate ones, so `explain-cron */15 9-17 '*' '*' 1-5` also works.
    let expression = args.join(" ");

    match explain_cron::parse_cron_expression(&expression) {
        Ok(schedule) => {
            println!("{}", explain_cron::describe(&schedule));
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}
