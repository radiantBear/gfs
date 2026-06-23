use std::fmt::Display;

use chrono::{DateTime, Days, Local, Months, NaiveDate, TimeZone, Utc, offset::LocalResult};

/// Picks dates out of the provided list that should be dropped to maintain a
/// Grandfather-Father-Son backup scheme. This means it keeps all dates in the last week,
/// one date per week in the last month, and one date per month before then.
///
/// When there are two or more candidate dates for a given range (i.e. multiple
/// possibilities in the same week or month), this function will preserve the oldest one.
/// This prevents gaps from forming when this function is used to delete records on many
/// days in a row. If the newest one were preserved instead, after 15 days, there would
/// still only be records for the latest 8 days (with all the older ones deleted) and no
/// record for the week prior could be preserved.
pub(crate) fn pick_dates_to_drop(
    last_week: DateTime<Local>,
    last_month: DateTime<Local>,
    mut dates_present: Vec<DateTime<Local>>,
) -> Option<Vec<DateTime<Local>>> {
    let one_week = Days::new(7);
    let one_month = Months::new(1);

    let mut dates_to_drop = Vec::new();

    let mut next_weekly = DateTime::<Utc>::MIN_UTC;
    let mut next_monthly = DateTime::<Utc>::MIN_UTC;

    dates_present.sort();

    for date in dates_present {
        if date > last_week {
            // Keep all for 7 days
            continue;
        }

        if date > last_month {
            // Keep 1 per week for the last month
            if date >= next_weekly {
                next_weekly = date.checked_add_days(one_week)?.to_utc();
            } else {
                dates_to_drop.push(date);
            }
        } else {
            // Keep 1 per month in perpetuity
            if date >= next_monthly {
                next_monthly = date.checked_add_months(one_month)?.to_utc();
            } else {
                dates_to_drop.push(date);
            }
        }
    }

    Some(dates_to_drop)
}

/// Converts the given date to a filename in `YYYY-MM-DD` format
pub(crate) fn date_to_filename(date: DateTime<Local>, suffix: &String) -> String {
    date.format(format!("%Y-%m-%d{}", suffix).as_str())
        .to_string()
}

/// Converts the given filename (in `YYYY-MM-DD` format) to a date
pub(crate) fn filename_to_date(
    filename: impl AsRef<str>,
    suffix: impl Display,
) -> Result<DateTime<Local>, String> {
    let date = NaiveDate::parse_from_str(filename.as_ref(), format!("%Y-%m-%d{}", suffix).as_str())
        .map_err(|err| err.to_string())?;
    let date = date
        .and_hms_opt(0, 0, 0)
        .ok_or("Unable to add time component to date".to_string())?;
    let date = Local.from_local_datetime(&date);

    match date {
        LocalResult::Single(date) => Ok(date),
        LocalResult::Ambiguous(date1, date2) => Err(format!(
            "{:#?} could be either {:#?} or {:#?} when localized",
            filename.as_ref(),
            date1,
            date2
        )),
        LocalResult::None => Err(format!(
            "{:#?} could not be converted to a localized date",
            filename.as_ref()
        )),
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn test_preserves_dates_within_1_week() {
        let data = vec![
            Local.with_ymd_and_hms(2025, 3, 2, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 3, 3, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 3, 5, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 3, 6, 0, 0, 0).unwrap(),
        ];
        let last_week = Local.with_ymd_and_hms(2025, 3, 1, 0, 0, 0).unwrap();
        let last_month = Local.with_ymd_and_hms(2025, 2, 7, 0, 0, 0).unwrap();

        let result = pick_dates_to_drop(last_week, last_month, data);

        assert_eq!(result.unwrap(), Vec::<DateTime<Local>>::new());
    }

    #[test]
    fn test_preserves_1_date_per_week_in_last_month() {
        let data = vec![
            Local.with_ymd_and_hms(2025, 3, 2, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 3, 3, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 3, 5, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 3, 6, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 3, 14, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 3, 18, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 3, 28, 0, 0, 0).unwrap(),
        ];
        let last_week = Local.with_ymd_and_hms(2025, 3, 24, 0, 0, 0).unwrap();
        let last_month = Local.with_ymd_and_hms(2025, 3, 1, 0, 0, 0).unwrap();

        let result = pick_dates_to_drop(last_week, last_month, data);

        assert_eq!(
            result.unwrap(),
            vec![
                Local.with_ymd_and_hms(2025, 3, 3, 0, 0, 0).unwrap(),
                Local.with_ymd_and_hms(2025, 3, 5, 0, 0, 0).unwrap(),
                Local.with_ymd_and_hms(2025, 3, 6, 0, 0, 0).unwrap(),
                Local.with_ymd_and_hms(2025, 3, 18, 0, 0, 0).unwrap(),
            ]
        );
    }

    #[test]
    fn test_preserves_1_date_per_month_before_last_month() {
        let data = vec![
            Local.with_ymd_and_hms(2025, 3, 2, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 3, 3, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 3, 5, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 3, 6, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 3, 14, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 3, 18, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 3, 28, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 4, 3, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 4, 17, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 4, 29, 0, 0, 0).unwrap(),
        ];
        let last_week = Local.with_ymd_and_hms(2025, 5, 24, 0, 0, 0).unwrap();
        let last_month = Local.with_ymd_and_hms(2025, 5, 1, 0, 0, 0).unwrap();

        let result = pick_dates_to_drop(last_week, last_month, data);

        assert_eq!(
            result.unwrap(),
            vec![
                Local.with_ymd_and_hms(2025, 3, 3, 0, 0, 0).unwrap(),
                Local.with_ymd_and_hms(2025, 3, 5, 0, 0, 0).unwrap(),
                Local.with_ymd_and_hms(2025, 3, 6, 0, 0, 0).unwrap(),
                Local.with_ymd_and_hms(2025, 3, 14, 0, 0, 0).unwrap(),
                Local.with_ymd_and_hms(2025, 3, 18, 0, 0, 0).unwrap(),
                Local.with_ymd_and_hms(2025, 3, 28, 0, 0, 0).unwrap(),
                Local.with_ymd_and_hms(2025, 4, 17, 0, 0, 0).unwrap(),
                Local.with_ymd_and_hms(2025, 4, 29, 0, 0, 0).unwrap(),
            ]
        );
    }

    #[test]
    fn test_works_on_typical_day_to_week_transition() {
        let data = vec![
            Local.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 2, 1, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 3, 1, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 3, 8, 0, 0, 0).unwrap(), // Recently was week 4; now redundant for month
            Local.with_ymd_and_hms(2025, 3, 15, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 3, 22, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 3, 29, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 4, 5, 0, 0, 0).unwrap(), // Yesterday was day 7; now week 1
            Local.with_ymd_and_hms(2025, 4, 6, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 4, 7, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 4, 8, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 4, 9, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 4, 10, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 4, 11, 0, 0, 0).unwrap(),
        ];
        let last_week = Local.with_ymd_and_hms(2025, 4, 5, 0, 0, 0).unwrap();
        let last_month = Local.with_ymd_and_hms(2025, 3, 12, 0, 0, 0).unwrap();

        let result = pick_dates_to_drop(last_week, last_month, data);

        assert_eq!(
            result.unwrap(),
            vec![Local.with_ymd_and_hms(2025, 3, 8, 0, 0, 0).unwrap()]
        );
    }

    #[test]
    fn test_works_on_typical_week_to_month_transition() {
        let data = vec![
            Local.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 2, 1, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 3, 1, 0, 0, 0).unwrap(), // Yesterday was week 4; now month 1
            Local.with_ymd_and_hms(2025, 3, 8, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 3, 15, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 3, 22, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 3, 25, 0, 0, 0).unwrap(), // Yesterday was day 7; now redundant for week
            Local.with_ymd_and_hms(2025, 3, 26, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 3, 27, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 3, 28, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 3, 29, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 3, 30, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 3, 31, 0, 0, 0).unwrap(),
        ];
        let last_week = Local.with_ymd_and_hms(2025, 3, 25, 0, 0, 0).unwrap();
        let last_month = Local.with_ymd_and_hms(2025, 3, 1, 0, 0, 0).unwrap();

        let result = pick_dates_to_drop(last_week, last_month, data);

        assert_eq!(
            result.unwrap(),
            vec![Local.with_ymd_and_hms(2025, 3, 25, 0, 0, 0).unwrap()]
        );
    }

    #[test]
    fn test_works_on_midmonth_day_to_week_transition() {
        let data = vec![
            Local.with_ymd_and_hms(2025, 1, 14, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 2, 14, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 3, 14, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 3, 21, 0, 0, 0).unwrap(), // Recently was week 4; now redundant for month
            Local.with_ymd_and_hms(2025, 3, 28, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 4, 4, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 4, 11, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 4, 18, 0, 0, 0).unwrap(), // Yesterday was day 7; now week 1
            Local.with_ymd_and_hms(2025, 4, 19, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 4, 20, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 4, 21, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 4, 22, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 4, 23, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 4, 24, 0, 0, 0).unwrap(),
        ];
        let last_week = Local.with_ymd_and_hms(2025, 4, 18, 0, 0, 0).unwrap();
        let last_month = Local.with_ymd_and_hms(2025, 3, 25, 0, 0, 0).unwrap();

        let result = pick_dates_to_drop(last_week, last_month, data);

        assert_eq!(
            result.unwrap(),
            vec![Local.with_ymd_and_hms(2025, 3, 21, 0, 0, 0).unwrap()]
        );
    }

    #[test]
    fn test_works_with_midmonth_week_to_month_transition() {
        let data = vec![
            Local.with_ymd_and_hms(2025, 1, 14, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 2, 14, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 3, 14, 0, 0, 0).unwrap(), // Yesterday was week 4; now month 1
            Local.with_ymd_and_hms(2025, 3, 21, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 3, 28, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 4, 4, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 4, 7, 0, 0, 0).unwrap(), // Yesterday was day 7; now redundant for week
            Local.with_ymd_and_hms(2025, 4, 8, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 4, 9, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 4, 10, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 4, 11, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 4, 12, 0, 0, 0).unwrap(),
            Local.with_ymd_and_hms(2025, 4, 13, 0, 0, 0).unwrap(),
        ];
        let last_week = Local.with_ymd_and_hms(2025, 4, 7, 0, 0, 0).unwrap();
        let last_month = Local.with_ymd_and_hms(2025, 3, 14, 0, 0, 0).unwrap();

        let result = pick_dates_to_drop(last_week, last_month, data);

        assert_eq!(
            result.unwrap(),
            vec![Local.with_ymd_and_hms(2025, 4, 7, 0, 0, 0).unwrap()]
        );
    }

    #[test]
    fn test_date_to_filename_conversion_works_without_suffix() {
        let date = Local.with_ymd_and_hms(2025, 3, 5, 0, 0, 0).unwrap();
        let result = date_to_filename(date, &"".to_string());

        assert_eq!(result, "2025-03-05".to_string());
    }

    #[test]
    fn test_date_to_filename_conversion_works_with_suffix() {
        let date = Local.with_ymd_and_hms(2025, 3, 5, 0, 0, 0).unwrap();
        let result = date_to_filename(date, &".sql".to_string());

        assert_eq!(result, "2025-03-05.sql".to_string());
    }

    #[test]
    fn test_filename_to_date_conversion_works_without_suffix() -> Result<(), String> {
        let result = filename_to_date(&"2025-03-05".to_string(), &"".to_string())?;

        assert_eq!(result, Local.with_ymd_and_hms(2025, 3, 5, 0, 0, 0).unwrap());
        Ok(())
    }

    #[test]
    fn test_filename_to_date_conversion_works_with_suffix() -> Result<(), String> {
        let result = filename_to_date(&"2025-03-05.sql".to_string(), &".sql".to_string())?;

        assert_eq!(result, Local.with_ymd_and_hms(2025, 3, 5, 0, 0, 0).unwrap());
        Ok(())
    }
}
