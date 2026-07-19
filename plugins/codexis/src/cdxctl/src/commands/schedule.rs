use crate::error::CdxctlError;
use serde_json::{json, Value};

const DAY_PARTS: [&str; 4] = ["NIGHT", "MORNING", "AFTERNOON", "EVENING"];
const REPEATS: [&str; 3] = ["DAILY", "WEEKLY", "MONTHLY"];
const MAX_DAY_OF_MONTH: u32 = 28;

/// Schedule flags as given on the command line. Exactly one of the structured
/// `repeat` or the legacy `cron` has to be present when a schedule is required.
#[derive(Default)]
pub struct ScheduleArgs<'a> {
    pub repeat: Option<&'a str>,
    pub day_part: Option<&'a str>,
    pub days_of_week: &'a [u32],
    pub day_of_month: Option<u32>,
    pub cron: Option<&'a str>,
}

impl ScheduleArgs<'_> {
    pub fn is_empty(&self) -> bool {
        self.repeat.is_none()
            && self.day_part.is_none()
            && self.days_of_week.is_empty()
            && self.day_of_month.is_none()
            && self.cron.is_none()
    }

    /// Builds the `AutomationScheduleInput` payload. Returns `None` when no schedule
    /// flag was given, so partial updates can leave the current schedule untouched.
    pub fn to_input(&self) -> Result<Option<Value>, CdxctlError> {
        if self.is_empty() {
            return Ok(None);
        }
        if self.repeat.is_some() && self.cron.is_some() {
            return Err(CdxctlError::Parse(
                "--cron cannot be combined with --repeat; use one or the other".to_string(),
            ));
        }
        if let Some(cron) = self.cron {
            return schedule_from_cron(cron).map(Some);
        }
        let repeat = normalize(
            self.repeat.ok_or_else(|| {
                CdxctlError::Parse("--repeat is required (DAILY, WEEKLY or MONTHLY)".to_string())
            })?,
            &REPEATS,
            "repeat",
        )?;
        let day_part = normalize(self.day_part.unwrap_or("MORNING"), &DAY_PARTS, "day part")?;
        build(&repeat, &day_part, self.days_of_week, self.day_of_month).map(Some)
    }
}

fn build(
    repeat: &str,
    day_part: &str,
    days_of_week: &[u32],
    day_of_month: Option<u32>,
) -> Result<Value, CdxctlError> {
    let mut input = json!({ "repeat": repeat, "dayPart": day_part });
    match repeat {
        "WEEKLY" => {
            if days_of_week.is_empty() {
                return Err(CdxctlError::Parse(
                    "--days-of-week is required for a WEEKLY schedule (0=Sunday..6=Saturday)"
                        .to_string(),
                ));
            }
            let mut days: Vec<u32> = days_of_week.to_vec();
            days.sort_unstable();
            days.dedup();
            if let Some(bad) = days.iter().find(|day| **day > 6) {
                return Err(CdxctlError::Parse(format!(
                    "day of week must be within 0-6 (0=Sunday), got {bad}"
                )));
            }
            input["daysOfWeek"] = json!(days);
        }
        "MONTHLY" => {
            let day = day_of_month.ok_or_else(|| {
                CdxctlError::Parse(format!(
                    "--day-of-month is required for a MONTHLY schedule (1-{MAX_DAY_OF_MONTH})"
                ))
            })?;
            if !(1..=MAX_DAY_OF_MONTH).contains(&day) {
                return Err(CdxctlError::Parse(format!(
                    "day of month must be within 1-{MAX_DAY_OF_MONTH}, got {day}"
                )));
            }
            input["dayOfMonth"] = json!(day);
        }
        _ => {}
    }
    Ok(input)
}

fn normalize(raw: &str, allowed: &[&str], label: &str) -> Result<String, CdxctlError> {
    let upper = raw.trim().to_uppercase();
    if allowed.contains(&upper.as_str()) {
        return Ok(upper);
    }
    Err(CdxctlError::Parse(format!(
        "unknown {label} '{raw}', expected one of {}",
        allowed.join(", ")
    )))
}

// TODO: delete `--cron` and this translation once every shipped plugin automation and
// every prompt/skill that still emits a cron expression has moved to --repeat/--day-part.
// The daemon dropped `AutomationInput.cron` entirely, so this is a client-side shim only.
fn schedule_from_cron(cron: &str) -> Result<Value, CdxctlError> {
    let fields: Vec<&str> = cron.split_whitespace().collect();
    if fields.len() != 5 {
        return Err(cron_error(cron));
    }
    let (minute, hour, day_of_month, month, day_of_week) =
        (fields[0], fields[1], fields[2], fields[3], fields[4]);
    if parse_number(minute, 0, 59).is_none() || month != "*" {
        return Err(cron_error(cron));
    }
    let hour = parse_number(hour, 0, 23).ok_or_else(|| cron_error(cron))?;
    let day_part = DAY_PARTS[(hour / 6) as usize];

    if day_of_month == "*" && day_of_week == "*" {
        return build("DAILY", day_part, &[], None);
    }
    if day_of_month == "*" {
        let days = parse_days_of_week(day_of_week).ok_or_else(|| cron_error(cron))?;
        return build("WEEKLY", day_part, &days, None);
    }
    if day_of_week == "*" {
        let day = parse_number(day_of_month, 1, 31).ok_or_else(|| cron_error(cron))?;
        return build("MONTHLY", day_part, &[], Some(day.min(MAX_DAY_OF_MONTH)));
    }
    Err(cron_error(cron))
}

fn parse_days_of_week(field: &str) -> Option<Vec<u32>> {
    field
        .split(',')
        .map(|raw| parse_number(raw, 0, 7).map(|day| if day == 7 { 0 } else { day }))
        .collect()
}

fn parse_number(raw: &str, min: u32, max: u32) -> Option<u32> {
    let value: u32 = raw.parse().ok()?;
    (value >= min && value <= max).then_some(value)
}

fn cron_error(cron: &str) -> CdxctlError {
    CdxctlError::Parse(format!(
        "cron expression '{cron}' cannot be expressed as a day-part schedule; \
         use --repeat/--day-part instead"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args<'a>(repeat: Option<&'a str>, day_part: Option<&'a str>) -> ScheduleArgs<'a> {
        ScheduleArgs {
            repeat,
            day_part,
            ..ScheduleArgs::default()
        }
    }

    #[test]
    fn empty_args_produce_no_schedule() {
        assert!(ScheduleArgs::default().to_input().unwrap().is_none());
    }

    #[test]
    fn daily_defaults_to_morning() {
        let input = args(Some("daily"), None).to_input().unwrap().unwrap();
        assert_eq!(input["repeat"], "DAILY");
        assert_eq!(input["dayPart"], "MORNING");
        assert!(input.get("daysOfWeek").is_none());
    }

    #[test]
    fn weekly_sorts_and_dedups_days() {
        let days = [5, 1, 5];
        let input = ScheduleArgs {
            repeat: Some("WEEKLY"),
            day_part: Some("evening"),
            days_of_week: &days,
            ..ScheduleArgs::default()
        }
        .to_input()
        .unwrap()
        .unwrap();
        assert_eq!(input["daysOfWeek"], json!([1, 5]));
        assert_eq!(input["dayPart"], "EVENING");
    }

    #[test]
    fn weekly_without_days_is_rejected() {
        assert!(args(Some("WEEKLY"), None).to_input().is_err());
    }

    #[test]
    fn monthly_requires_day_of_month_within_28() {
        assert!(args(Some("MONTHLY"), None).to_input().is_err());
        let input = ScheduleArgs {
            repeat: Some("MONTHLY"),
            day_of_month: Some(15),
            ..ScheduleArgs::default()
        }
        .to_input()
        .unwrap()
        .unwrap();
        assert_eq!(input["dayOfMonth"], 15);
        assert!(ScheduleArgs {
            repeat: Some("MONTHLY"),
            day_of_month: Some(31),
            ..ScheduleArgs::default()
        }
        .to_input()
        .is_err());
    }

    #[test]
    fn unknown_enum_values_are_rejected() {
        assert!(args(Some("HOURLY"), None).to_input().is_err());
        assert!(args(Some("DAILY"), Some("NOON")).to_input().is_err());
    }

    #[test]
    fn cron_daily_maps_to_day_part() {
        let cron = "0 9 * * *";
        let input = ScheduleArgs {
            cron: Some(cron),
            ..ScheduleArgs::default()
        }
        .to_input()
        .unwrap()
        .unwrap();
        assert_eq!(input["repeat"], "DAILY");
        assert_eq!(input["dayPart"], "MORNING");
    }

    #[test]
    fn cron_weekly_maps_sunday_seven_to_zero() {
        let input = ScheduleArgs {
            cron: Some("0 20 * * 7,1"),
            ..ScheduleArgs::default()
        }
        .to_input()
        .unwrap()
        .unwrap();
        assert_eq!(input["repeat"], "WEEKLY");
        assert_eq!(input["dayPart"], "EVENING");
        assert_eq!(input["daysOfWeek"], json!([0, 1]));
    }

    #[test]
    fn cron_monthly_clamps_day_31_to_28() {
        let input = ScheduleArgs {
            cron: Some("0 2 31 * *"),
            ..ScheduleArgs::default()
        }
        .to_input()
        .unwrap()
        .unwrap();
        assert_eq!(input["repeat"], "MONTHLY");
        assert_eq!(input["dayPart"], "NIGHT");
        assert_eq!(input["dayOfMonth"], 28);
    }

    #[test]
    fn sub_daily_cron_is_rejected() {
        for cron in ["*/5 * * * *", "0 * * * *", "0 9 1-5 * *", "0 9 * *"] {
            assert!(
                ScheduleArgs {
                    cron: Some(cron),
                    ..ScheduleArgs::default()
                }
                .to_input()
                .is_err(),
                "expected {cron} to be rejected"
            );
        }
    }

    #[test]
    fn cron_and_repeat_together_are_rejected() {
        assert!(ScheduleArgs {
            repeat: Some("DAILY"),
            cron: Some("0 9 * * *"),
            ..ScheduleArgs::default()
        }
        .to_input()
        .is_err());
    }
}
