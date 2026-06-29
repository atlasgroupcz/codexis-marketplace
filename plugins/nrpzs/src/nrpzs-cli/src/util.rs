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

/// Lowercase + strip Czech diacritics, for accent-insensitive matching.
pub fn norm(s: &str) -> String {
    s.chars()
        .map(|c| {
            let lower = c.to_lowercase().next().unwrap_or(c);
            match lower {
                'á' | 'à' | 'â' | 'ä' => 'a',
                'č' => 'c',
                'ď' => 'd',
                'é' | 'ě' | 'ê' | 'ë' => 'e',
                'í' | 'î' | 'ï' => 'i',
                'ň' => 'n',
                'ó' | 'ô' | 'ö' => 'o',
                'ř' => 'r',
                'š' => 's',
                'ť' => 't',
                'ú' | 'ů' | 'û' | 'ü' => 'u',
                'ý' => 'y',
                'ž' => 'z',
                other => other,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ico_pads_to_eight() {
        assert_eq!(normalize_ico("6196306").unwrap(), "06196306");
        assert_eq!(normalize_ico("27085031").unwrap(), "27085031");
    }

    #[test]
    fn ico_rejects_nondigits_and_overflow() {
        assert!(normalize_ico("12a45").is_err());
        assert!(normalize_ico("123456789").is_err());
        assert!(normalize_ico("  ").is_err());
    }

    #[test]
    fn norm_strips_czech_diacritics_and_lowercases() {
        assert_eq!(norm("Středočeský"), "stredocesky");
        assert_eq!(norm("UROLOGIE"), "urologie");
        assert_eq!(norm("Příbram"), "pribram");
    }
}
