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
