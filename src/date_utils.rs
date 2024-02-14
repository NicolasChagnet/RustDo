use anyhow::Result;
use chrono::prelude::*;
use chrono::{Datelike, Duration, NaiveDate, Weekday};
use once_cell::sync::Lazy;
use regex::Regex;

pub const FORMAT_DATE: &str = "%d-%m-%Y";
pub const FORMAT_DATETIME: &str = "%d-%m-%Y %h-%m-%s";
pub const ALLOWEDNONDATE: [&str; 4] = ["today", "tomorrow", "next week", "next month"];
pub const ALLOWEDWEEKDAY: [&str; 7] = [
    "monday",
    "tuesday",
    "wednesday",
    "thursday",
    "friday",
    "saturday",
    "sunday",
];

// Counts the number of occurences of a given character
fn count_char(s: &str, ch: char) -> usize {
    s.chars().filter(|c| *c == ch).count()
}

// Converts a string into the appriopriate date element -- calls the correct function depending on the type of string
pub fn convert_str_valid_date(due_str: &str) -> Result<NaiveDate> {
    // The first match checks whether the user specified the year (otherwise we fill it with the current year)
    let date_ret = match count_char(due_str, '-') {
        1 => {
            NaiveDate::parse_from_str(&format!("{}-{}", due_str, Local::now().year()), FORMAT_DATE)?
        }
        0 => {
            // Deals with special inputs
            if ALLOWEDNONDATE.contains(&due_str) {
                get_language_date(due_str)?
            } else if ALLOWEDWEEKDAY.contains(&due_str) {
                get_next_weekday(due_str)?
            } else {
                anyhow::bail!("Wrong date format!");
            }
        }
        _ => NaiveDate::parse_from_str(due_str, FORMAT_DATE)?,
    };
    Ok(date_ret)
}

// Validates the input from user
pub fn validate_regex(s: &String) -> Result<(), &'static str> {
    // This makes sure the regex only initializes once
    static REWORD: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^(0[1-9]|[12][0-9]|3[01])-(0[1-9]|1[012])(-(19|20)\d\d)?$").unwrap()
    });
    // Allow for regex matches, empty strings or various special inputs
    match REWORD.is_match(s)
        || s.is_empty()
        || ALLOWEDNONDATE.contains(&s.as_str())
        || ALLOWEDWEEKDAY.contains(&s.as_str())
    {
        true => Ok(()),
        false => Err("Invalid date!"),
    }
}

// Returns the next date matching a certain weekday
fn get_next_weekday(day_str: &str) -> Result<NaiveDate> {
    let day = day_str.parse::<Weekday>()?; // Day we aim to get
    let now = Local::now();
    let today = now.weekday(); // Day we start from
                               // We initialize with the next day
    let mut next_day = today.succ();

    let mut counter = 1;
    while next_day != day {
        // Loop until we find the correct weekday, deducing the gap
        counter += 1;
        next_day = next_day.succ();
    }
    // Returns the new date with the correct shift
    Ok((now + Duration::days(counter)).date_naive())
}

// Handles special language date markers
fn get_language_date(due_date: &str) -> Result<NaiveDate> {
    let now: DateTime<Local> = Local::now();
    let ret = match due_date {
        "today" => now.date_naive(),
        "tomorrow" => (now + Duration::days(1)).date_naive(),
        "next week" => (now + Duration::days(7)).date_naive(),
        "next month" => {
            let new_date = now.checked_add_months(chrono::Months::new(1));
            match new_date {
                Some(v) => v.date_naive(),
                None => (now + Duration::days(30)).date_naive(),
            }
        }
        _ => anyhow::bail!("Error in natural language parsing for date (this should not happen)"),
    };
    Ok(ret)
}
