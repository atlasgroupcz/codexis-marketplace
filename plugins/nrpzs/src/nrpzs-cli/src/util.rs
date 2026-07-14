pub(crate) use cdx_common::ico::normalize_ico;

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
    fn norm_strips_czech_diacritics_and_lowercases() {
        assert_eq!(norm("Středočeský"), "stredocesky");
        assert_eq!(norm("UROLOGIE"), "urologie");
        assert_eq!(norm("Příbram"), "pribram");
    }
}
