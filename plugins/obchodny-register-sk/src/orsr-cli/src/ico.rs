use anyhow::{anyhow, Result};

pub(crate) use cdx_common::ico::normalize_ico;

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
    fn validate_date_accepts_iso_day() {
        assert!(validate_date("2026-06-10", "--date").is_ok());
    }

    #[test]
    fn validate_date_rejects_other_shapes() {
        assert!(validate_date("10.06.2026", "--date").is_err());
    }
}
