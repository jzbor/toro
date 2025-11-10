use chrono::{Datelike, Days, NaiveDate, Utc};

use crate::error::{ToroError, ToroResult};

pub fn format_date(date: NaiveDate, nice: bool) -> String {
    let today: NaiveDate = Utc::now().naive_local().into();
    let diff = date - today;

    if nice && date == today {
        String::from("today")
    } else if nice && today.checked_sub_days(Days::new(1)).map(|n| date == n).unwrap_or(false) {
        String::from("yesterday")
    } else if nice && today.checked_add_days(Days::new(1)).map(|n| date == n).unwrap_or(false) {
        String::from("tomorrow")
    } else if nice && diff.num_days().unsigned_abs() < 14 {
        format!("{}d", diff.num_days())
    } else if nice && diff.num_weeks().unsigned_abs() < 6 {
        format!("{}w", diff.num_weeks())
    } else {
        format!("{:0>4}-{:0>2}-{:0>2}", date.year(), date.month(), date.day())
    }
}

pub fn parse_date(input: &str) -> ToroResult<NaiveDate> {
    if input == "today" {
        Ok(Utc::now().naive_local().into())
    } else if input == "tomorrow" {
        let date: NaiveDate = Utc::now().naive_local().into();
        date.checked_add_days(Days::new(1))
            .ok_or(ToroError::DateOverflowError())
    } else if input == "yesterday" {
        let date: NaiveDate = Utc::now().naive_local().into();
        date.checked_sub_days(Days::new(1))
            .ok_or(ToroError::DateOverflowError())
    } else if let Some(ds) = input.strip_suffix("d") {
        let date: NaiveDate = Utc::now().naive_local().into();
        let days = ds.parse().map_err(|_| ToroError::DateInputError(input.to_owned()))?;
        date.checked_add_days(Days::new(days))
            .ok_or(ToroError::DateOverflowError())
    } else if let Some(ws) = input.strip_suffix("w") {
        let date: NaiveDate = Utc::now().naive_local().into();
        let weeks = ws.parse::<u64>().map_err(|_| ToroError::DateInputError(input.to_owned()))?;
        date.checked_add_days(Days::new(weeks * 7))
            .ok_or(ToroError::DateOverflowError())
    } else {
        let mut parts = input.splitn(3, "-");

        let year_str = parts.next().ok_or_else(|| ToroError::DateInputError(input.to_owned()))?;
        let month_str = parts.next().ok_or_else(|| ToroError::DateInputError(input.to_owned()))?;
        let day_str = parts.next().ok_or_else(|| ToroError::DateInputError(input.to_owned()))?;

        let year = str::parse(year_str).map_err(|_| ToroError::DateInputError(input.to_owned()))?;
        let month = str::parse(month_str).map_err(|_| ToroError::DateInputError(input.to_owned()))?;
        let day = str::parse(day_str).map_err(|_| ToroError::DateInputError(input.to_owned()))?;

        NaiveDate::default()
            .with_year(year)
            .ok_or_else(|| ToroError::DateInputError(input.to_owned()))?
            .with_month(month)
            .ok_or_else(|| ToroError::DateInputError(input.to_owned()))?
            .with_day(day)
            .ok_or_else(|| ToroError::DateInputError(input.to_owned()))
    }
}
