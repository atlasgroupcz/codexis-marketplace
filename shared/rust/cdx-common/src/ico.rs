use anyhow::{anyhow, Result};

/// Normalizes a raw IČO string into the canonical 8-digit, zero-padded form.
///
/// Byte-identical logic was copy-pasted across the `ares`, `zrsr`,
/// `obchodny-register-sk` and `nrpzs` plugins.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pads_short_ico() {
        assert_eq!(normalize_ico("12345").unwrap(), "00012345");
    }

    #[test]
    fn keeps_full_ico() {
        assert_eq!(normalize_ico("00012345").unwrap(), "00012345");
    }

    #[test]
    fn rejects_empty() {
        assert!(normalize_ico("   ").is_err());
    }

    #[test]
    fn rejects_non_digits() {
        assert!(normalize_ico("12a45").is_err());
    }

    #[test]
    fn rejects_too_long() {
        assert!(normalize_ico("123456789").is_err());
    }
}
