mod dates;
mod files;

use std::io;

use crate::dates::*;
use crate::files::*;
use chrono::{DateTime, Days, Local, Months};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(about, author, version)]
struct Args {
    #[arg(
        short,
        long,
        default_value = "",
        help = "Expect and apply this string to filenames"
    )]
    suffix: String,

    #[arg(long, hide = true, value_parser = |s: &str| filename_to_date(s, ""))]
    test_fixed_date: Option<DateTime<Local>>,
}

pub fn main() -> Result<(), String> {
    let args = Args::parse();

    let today = args.test_fixed_date.unwrap_or(Local::now());
    let Some(last_week) = today.checked_sub_days(Days::new(7)) else {
        return Err("Unable to generate date for last week".to_owned());
    };
    let Some(last_month) = today.checked_sub_months(Months::new(1)) else {
        return Err("Unable to generate date for last month".to_owned());
    };

    let dates_present =
        parse_files(".", |n| filename_to_date(n, &args.suffix)).map_err(|err| err.to_string())?;

    copy_input_to_file(&mut io::stdin(), date_to_filename(today, &args.suffix)).unwrap();

    let Some(old_dates) = pick_dates_to_drop(last_week, last_month, dates_present) else {
        return Err("Unable to generate date for a week/month offset".to_owned());
    };
    let old_dates = old_dates
        .into_iter()
        .map(|d| date_to_filename(d, &args.suffix))
        .collect();

    println!("Cleaning up old files: {:#?}", old_dates);
    remove_files(".", old_dates).unwrap();

    Ok(())
}
