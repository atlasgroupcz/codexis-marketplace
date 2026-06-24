//! Skládání textu pro vyhledávání: lowercase + odstranění diakritiky,
//! ať `"listova"` najde `"listová"` a `"paracetamol"` najde `"paracetamol"`.
//! Pokrývá českou a slovenskou diakritiku (data SÚKL jsou cz/sk).

/// Lowercase + fold diakritiky na holé ASCII písmeno.
pub fn fold(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ä' | 'ã' | 'å' => 'a',
            'č' | 'ç' | 'ć' => 'c',
            'ď' => 'd',
            'é' | 'è' | 'ê' | 'ë' | 'ě' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ĺ' | 'ľ' | 'ł' => 'l',
            'ň' | 'ñ' => 'n',
            'ó' | 'ò' | 'ô' | 'ö' | 'õ' | 'ø' => 'o',
            'ŕ' | 'ř' => 'r',
            'š' | 'ś' => 's',
            'ť' => 't',
            'ú' | 'ù' | 'û' | 'ü' | 'ů' | 'ű' => 'u',
            'ý' | 'ÿ' => 'y',
            'ž' | 'ź' | 'ż' => 'z',
            other => other,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folds_czech_diacritics_and_lowercases() {
        assert_eq!(fold("Listová"), "listova");
        assert_eq!(fold("KYSELINA LISTOVÁ"), "kyselina listova");
        assert_eq!(fold("Paracetamol"), "paracetamol");
    }

    #[test]
    fn folds_slovak_diacritics() {
        assert_eq!(fold("ľanový"), "lanovy");
        assert_eq!(fold("ťava"), "tava");
    }

    #[test]
    fn accent_insensitive_substring_match() {
        // dotaz bez háčků chytne data s háčky
        assert!(fold("kyselina listová").contains(&fold("listova")));
    }
}
