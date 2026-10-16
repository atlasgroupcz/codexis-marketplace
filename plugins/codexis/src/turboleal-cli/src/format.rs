use std::sync::LazyLock;
use std::time::SystemTime;

use chrono::{DateTime, Datelike, Days, NaiveDate, TimeDelta, Utc};
use regex::Regex;
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

static ISO_DAY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d{4})-(\d{2})-(\d{2})").unwrap());
static SECTION_PREFIX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"§|par\.|paragraf|\s+").unwrap());

pub const CHANGE_TYPES: [&str; 5] = [
    "AMENDED",
    "UPCOMING_AMENDMENT",
    "ADDED",
    "REMOVED",
    "REPEALED",
];

/// The change type in Czech words.
pub fn type_label(change_type: &str) -> &str {
    match change_type {
        "AMENDED" => "změna znění (novela)",
        "UPCOMING_AMENDMENT" => "připravovaná změna, zatím nenabyla účinnosti",
        "ADDED" => "nové ustanovení",
        "REMOVED" => "zrušené ustanovení",
        "REPEALED" => "zrušený předpis",
        other => other,
    }
}

/// `2027-01-01` or an ISO instant as `1. 1. 2027`; anything else unchanged.
pub fn czech_date(value: &str) -> String {
    match ISO_DAY.captures(value) {
        Some(day) => format!(
            "{}. {}. {}",
            day[3].parse::<u8>().unwrap_or_default(),
            day[2].parse::<u8>().unwrap_or_default(),
            &day[1]
        ),
        None => value.to_owned(),
    }
}

/// Lower case without diacritics, for forgiving substring matches.
pub fn fold(value: &str) -> String {
    value
        .nfd()
        .filter(|character| !is_combining_mark(*character))
        .collect::<String>()
        .to_lowercase()
}

/// `§ 38g`, `38g`, `par. 38g` and `§ 38g - Daňové přiznání` all reduce to `38g`.
pub fn section_key(value: &str) -> String {
    let label = value.split(" - ").next().unwrap_or_default();
    SECTION_PREFIX.replace_all(&fold(label), "").into_owned()
}

/// The calendar day in Prague of an instant such as a `detectedAt`; a plain
/// `YYYY-MM-DD` or anything unparsable comes back unchanged.
pub fn prague_date(value: &str) -> String {
    match DateTime::parse_from_rfc3339(value) {
        Ok(instant) => prague_day(instant.with_timezone(&Utc)).to_string(),
        Err(_) => value.to_owned(),
    }
}

/// Today in Prague as `YYYY-MM-DD`.
pub fn prague_today() -> String {
    prague_day(DateTime::<Utc>::from(SystemTime::now())).to_string()
}

/// Central European time: UTC+2 from the last Sunday of March to the last
/// Sunday of October, both at 01:00 UTC, otherwise UTC+1.
fn prague_day(instant: DateTime<Utc>) -> NaiveDate {
    let year = instant.year();
    let switch = |month| {
        last_sunday(year, month)
            .and_hms_opt(1, 0, 0)
            .expect("01:00 exists")
            .and_utc()
    };
    let summer = instant >= switch(3) && instant < switch(10);
    (instant + TimeDelta::hours(if summer { 2 } else { 1 })).date_naive()
}

fn last_sunday(year: i32, month: u32) -> NaiveDate {
    let last = NaiveDate::from_ymd_opt(year, month + 1, 1)
        .and_then(|first| first.pred_opt())
        .expect("a valid month");
    last - Days::new(last.weekday().num_days_from_sunday().into())
}

/// Accepts a `YYYY-MM-DD` date on the command line.
pub fn iso_date(value: &str) -> Result<String, String> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map(|date| date.to_string())
        .map_err(|_| format!("expected a date as YYYY-MM-DD, got {value}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_are_written_in_czech() {
        assert_eq!(czech_date("2027-03-09"), "9. 3. 2027");
        assert_eq!(czech_date("2026-10-15T12:00:00Z"), "15. 10. 2026");
        assert_eq!(czech_date("soon"), "soon");
    }

    #[test]
    fn section_labels_reduce_to_their_number() {
        assert_eq!(section_key("§ 38g"), section_key("38g"));
        assert_ne!(section_key("§ 38ga"), section_key("38g"));
        assert_eq!(
            section_key("§ 72 - Nárok na odpočet daně"),
            section_key("72")
        );
        assert_eq!(section_key("par. 52"), "52");
    }

    #[test]
    fn folding_drops_case_and_diacritics() {
        assert_eq!(fold("Zákoník PRÁCE"), "zakonik prace");
        assert_eq!(fold("Zákon o daních z příjmů"), "zakon o danich z prijmu");
    }

    #[test]
    fn detection_is_dated_in_prague() {
        assert_eq!(prague_date("2026-10-15T23:30:00Z"), "2026-10-16");
        assert_eq!(prague_date("2026-10-25T21:59:00Z"), "2026-10-25");
        assert_eq!(prague_date("2026-10-25T23:30:00Z"), "2026-10-26");
        assert_eq!(prague_date("2026-01-15T22:59:59.123456Z"), "2026-01-15");
        assert_eq!(prague_date("2026-01-15T23:00:00Z"), "2026-01-16");
        assert_eq!(prague_date("2026-07-01T22:00:00Z"), "2026-07-02");
        assert_eq!(prague_date("2026-10-15"), "2026-10-15");
    }

    #[test]
    fn only_iso_dates_are_accepted_on_the_command_line() {
        assert_eq!(iso_date("2026-12-31").as_deref(), Ok("2026-12-31"));
        assert!(iso_date("31. 12. 2026").is_err());
    }
}
