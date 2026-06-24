//! Search text normalization: lowercase + strip cs/sk diacritics, so an
//! accent-free query still matches accented data.

pub fn is_ascii_digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

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
        assert!(fold("kyselina listová").contains(&fold("listova")));
    }

    #[test]
    fn is_ascii_digits_basics() {
        assert!(is_ascii_digits("254045"));
        assert!(!is_ascii_digits("")); // prázdné není číselný token
        assert!(!is_ascii_digits("50mg"));
        assert!(!is_ascii_digits("12 34"));
    }
}
