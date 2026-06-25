/// Fold a string for diacritics-insensitive, case-insensitive matching.
/// Maps Czech (and common Latin) accented letters to ASCII and lowercases.
pub fn fold(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        for low in ch.to_lowercase() {
            out.push(map_char(low));
        }
    }
    out
}

fn map_char(c: char) -> char {
    match c {
        'á' | 'à' | 'â' | 'ä' | 'ã' | 'å' => 'a',
        'č' | 'ç' | 'ć' => 'c',
        'ď' => 'd',
        'é' | 'è' | 'ê' | 'ë' | 'ě' => 'e',
        'í' | 'ì' | 'î' | 'ï' => 'i',
        'ľ' | 'ł' => 'l',
        'ň' | 'ñ' => 'n',
        'ó' | 'ò' | 'ô' | 'ö' | 'õ' => 'o',
        'ř' => 'r',
        'š' | 'ś' => 's',
        'ť' => 't',
        'ú' | 'ù' | 'û' | 'ü' | 'ů' => 'u',
        'ý' | 'ÿ' => 'y',
        'ž' | 'ź' | 'ż' => 'z',
        other => other,
    }
}

/// Does `haystack` contain every whitespace-separated token of `query`
/// (all comparisons folded)? Empty query matches nothing.
pub fn matches_all_tokens(haystack_folded: &str, query: &str) -> bool {
    let tokens: Vec<String> = query.split_whitespace().map(fold).collect();
    if tokens.is_empty() {
        return false;
    }
    tokens.iter().all(|t| haystack_folded.contains(t.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folds_czech_diacritics() {
        assert_eq!(fold("Zdravotnický Prostředek"), "zdravotnicky prostredek");
        assert_eq!(fold("ŘÁDEK"), "radek");
    }

    #[test]
    fn token_match_is_order_independent() {
        let hay = fold("EIA Gliadin IgA TestLine");
        assert!(matches_all_tokens(&hay, "gliadin eia"));
        assert!(matches_all_tokens(&hay, "TESTLINE"));
        assert!(!matches_all_tokens(&hay, "gliadin xyz"));
        assert!(!matches_all_tokens(&hay, "   "));
    }
}
