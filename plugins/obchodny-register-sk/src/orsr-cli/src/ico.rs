use anyhow::{anyhow, Result};

pub fn normalize_ico(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("IČO is empty"));
    }
    if !trimmed.chars().all(|c| c.is_ascii_digit()) {
        return Err(anyhow!("IČO must contain only digits, got {trimmed:?}"));
    }
    if trimmed.len() > 8 {
        return Err(anyhow!(
            "IČO is at most 8 digits long, got {} digits ({trimmed:?})",
            trimmed.len()
        ));
    }
    Ok(format!("{trimmed:0>8}"))
}

pub fn validate_date(raw: &str, name: &str) -> Result<()> {
    let value = raw.trim();
    let valid = value.len() == 10
        && value.as_bytes()[4] == b'-'
        && value.as_bytes()[7] == b'-'
        && value
            .chars()
            .enumerate()
            .all(|(index, c)| index == 4 || index == 7 || c.is_ascii_digit());
    if valid {
        Ok(())
    } else {
        Err(anyhow!("{name} must be a date in YYYY-MM-DD format"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_ico_pads_to_eight_digits() {
        assert_eq!(normalize_ico("123").unwrap(), "00000123");
    }

    #[test]
    fn normalize_ico_rejects_non_digits() {
        assert!(normalize_ico("31A33532").is_err());
    }

    #[test]
    fn normalize_ico_rejects_more_than_eight_digits() {
        assert!(normalize_ico("123456789").is_err());
    }

    #[test]
    fn validate_date_accepts_iso_day() {
        assert!(validate_date("2026-06-10", "--date").is_ok());
    }

    #[test]
    fn validate_date_rejects_other_shapes() {
        assert!(validate_date("10.06.2026", "--date").is_err());
    }
}
