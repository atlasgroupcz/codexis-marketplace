//! Parsing the or.justice.cz register pages.
//!
//! The portal is an Apache Wicket app - every page is server-rendered HTML with
//! no JSON behind it (verified 2026-07-20 across the search, listiny and detail
//! pages: the only XHR is an unrelated support chatbot). So the HTML *is* the
//! API, and these functions turn it into data.
//!
//! The markup is a stable table layout, not a fragile modern one, so targeted
//! regex extraction is reliable. Everything below was checked against real
//! responses on 2026-07-20.

use std::sync::OnceLock;

use regex::Regex;

/// One row of the IČO/name search results (`rejstrik-$firma`).
#[derive(Debug, Clone, PartialEq)]
pub struct SearchHit {
    pub subjekt_id: String,
    pub ico: String,
    pub nazev: String,
    pub spisovaznacka: Option<String>,
    pub sidlo: Option<String>,
}

/// One row of the Sbírka listin table (`vypis-sl-firma`).
#[derive(Debug, Clone, PartialEq)]
pub struct ListinaRow {
    /// The full číslo listiny, e.g. `B 8573/SL95/MSPH` (nbsp normalized to space).
    /// This is the document's only identifier - the register's "Číslo listiny".
    pub cislo: String,
    pub typ: String,
    pub vznik: Option<String>,
    pub doslo: Option<String>,
    pub zalozeno: Option<String>,
    pub stran: Option<u32>,
    /// The `dokument`/`spis` ids from the row's detail link - the download chain
    /// needs both alongside the subjektId.
    pub dokument: String,
    pub spis: String,
}

fn re(cell: &'static OnceLock<Regex>, pattern: &str) -> &'static Regex {
    // Each pattern is compiled once into its own static cell.
    cell.get_or_init(|| Regex::new(pattern).expect("static regex"))
}

/// Collapse tags, entities and whitespace of an HTML fragment to plain text.
fn text_of(fragment: &str) -> String {
    static TAG: OnceLock<Regex> = OnceLock::new();
    let no_tags = re(&TAG, r"(?s)<[^>]*>").replace_all(fragment, " ");
    let unescaped = no_tags
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace('\u{a0}', " ");
    unescaped.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// `Počet nalezených subjektů: N`. `None` when the marker is absent (a malformed
/// page), which the caller treats differently from a real `0`.
pub fn search_count(html: &str) -> Option<usize> {
    static RE: OnceLock<Regex> = OnceLock::new();
    re(&RE, r"(?s)nalezených subjektů:\s*<span>\s*(\d+)")
        .captures(html)
        .and_then(|c| c[1].parse().ok())
}

/// Every result block on a search page. Each `<li class="result …">` carries one
/// subject's identity and its Sbírka listin link (`vypis-sl-firma?subjektId=`).
pub fn search_hits(html: &str) -> Vec<SearchHit> {
    static SUBJ: OnceLock<Regex> = OnceLock::new();
    static NAZEV: OnceLock<Regex> = OnceLock::new();
    static ICO: OnceLock<Regex> = OnceLock::new();
    static SPIS: OnceLock<Regex> = OnceLock::new();
    static SIDLO: OnceLock<Regex> = OnceLock::new();

    let mut hits = Vec::new();
    // Split on the result-item boundary; the first chunk is the page/form header.
    for block in html.split("<li class=\"result").skip(1) {
        let Some(subjekt_id) = re(&SUBJ, r"vypis-sl-firma\?subjektId=(\d+)")
            .captures(block)
            .map(|c| c[1].to_string())
        else {
            continue;
        };
        let nazev = re(&NAZEV, r"(?s)Název subjektu:.*?<strong[^>]*>(.*?)</strong>")
            .captures(block)
            .map(|c| text_of(&c[1]))
            .unwrap_or_default();
        // The IČO cell interleaves the digit groups with spacer spans whose
        // `style="…0.5em"` carries stray digits - strip tags first, then keep
        // only the digits of the resulting text.
        let ico = re(&ICO, r"(?s)IČO:.*?<strong>(.*?)</strong>")
            .captures(block)
            .map(|c| {
                text_of(&c[1])
                    .chars()
                    .filter(|ch| ch.is_ascii_digit())
                    .collect()
            })
            .unwrap_or_default();
        let spisovaznacka = re(&SPIS, r"(?s)Spisová značka:.*?<span[^>]*>(.*?)</span>")
            .captures(block)
            .map(|c| text_of(&c[1]))
            .filter(|s| !s.is_empty());
        let sidlo = re(&SIDLO, r"(?s)Sídlo:.*?<td[^>]*>(.*?)</td>")
            .captures(block)
            .map(|c| text_of(&c[1]))
            .filter(|s| !s.is_empty());
        hits.push(SearchHit {
            subjekt_id,
            ico,
            nazev,
            spisovaznacka,
            sidlo,
        });
    }
    hits
}

/// The subject identity printed above the listiny table (name + spisová značka +
/// sídlo). Absent fields stay `None`.
pub struct SubjektHlavicka {
    pub nazev: Option<String>,
    pub spisovaznacka: Option<String>,
    pub sidlo: Option<String>,
}

pub fn subjekt_hlavicka(html: &str) -> SubjektHlavicka {
    static NAZEV: OnceLock<Regex> = OnceLock::new();
    static SPIS: OnceLock<Regex> = OnceLock::new();
    static SIDLO: OnceLock<Regex> = OnceLock::new();
    SubjektHlavicka {
        nazev: re(&NAZEV, r"(?s)Obchodní firma:.*?<td[^>]*>(.*?)</td>")
            .captures(html)
            .map(|c| text_of(&c[1]))
            .filter(|s| !s.is_empty()),
        spisovaznacka: re(&SPIS, r"(?s)Spisová značka:.*?<td[^>]*>(.*?)</td>")
            .captures(html)
            .map(|c| text_of(&c[1]))
            .filter(|s| !s.is_empty()),
        sidlo: re(&SIDLO, r"(?s)Sídlo:.*?<td[^>]*>(.*?)</td>")
            .captures(html)
            .map(|c| text_of(&c[1]))
            .filter(|s| !s.is_empty()),
    }
}

/// Every document row of a `vypis-sl-firma` page. The table is one `<tr>` per
/// listina, seven cells: číslo (+ detail link), typ, vznik, došlo, založeno,
/// stran, PDF icon.
pub fn listiny(html: &str) -> Vec<ListinaRow> {
    static ROW: OnceLock<Regex> = OnceLock::new();
    static LINK: OnceLock<Regex> = OnceLock::new();
    static CISLO: OnceLock<Regex> = OnceLock::new();
    static CELL: OnceLock<Regex> = OnceLock::new();

    let row_re = re(&ROW, r"(?s)<tr>(.*?)</tr>");
    let link_re = re(
        &LINK,
        r"vypis-sl-detail\?dokument=(\d+)&(?:amp;)?subjektId=\d+&(?:amp;)?spis=(\d+)",
    );
    let cislo_re = re(&CISLO, r"(?s)vypis-sl-detail[^>]*>\s*<span>(.*?)</span>");
    let cell_re = re(&CELL, r"(?s)<td[^>]*>(.*?)</td>");

    let mut rows = Vec::new();
    for row in row_re.captures_iter(html) {
        let inner = &row[1];
        // A data row is exactly one that links to a document detail.
        let Some(link) = link_re.captures(inner) else {
            continue;
        };
        let dokument = link[1].to_string();
        let spis = link[2].to_string();
        let Some(cislo) = cislo_re.captures(inner).map(|c| text_of(&c[1])) else {
            continue;
        };
        let cells: Vec<String> = cell_re
            .captures_iter(inner)
            .map(|c| text_of(&c[1]))
            .collect();
        let cell = |i: usize| cells.get(i).cloned().filter(|s| !s.is_empty());
        rows.push(ListinaRow {
            cislo,
            typ: cell(1).unwrap_or_default(),
            vznik: cell(2),
            doslo: cell(3),
            zalozeno: cell(4),
            stran: cell(5).and_then(|s| s.parse().ok()),
            dokument,
            spis,
        });
    }
    rows
}

/// The `content/download?id=<32-hex>` handle on a `vypis-sl-detail` page - the
/// only place the actual download link exists. `None` when the document has no
/// digital form.
pub fn download_id(html: &str) -> Option<String> {
    static RE: OnceLock<Regex> = OnceLock::new();
    re(&RE, r"content/download\?id=([a-f0-9]{32})")
        .captures(html)
        .map(|c| c[1].to_string())
}

/// Normalize a číslo listiny for comparison: uppercase, single-spaced, nbsp
/// folded. So `b\u{a0}8573/sl95/msph` and `B 8573/SL95/MSPH` compare equal.
pub fn normalize_key(raw: &str) -> String {
    raw.replace('\u{a0}', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_uppercase()
}

/// A filesystem-safe stem for a číslo listiny: `B 8573/SL95/MSPH` → `B-8573-SL95-MSPH`.
/// Slashes and spaces (both awkward in filenames) become dashes; this keeps the
/// full identifier on disk rather than a bare SL number.
pub fn file_stem(cislo: &str) -> String {
    normalize_key(cislo)
        .chars()
        .map(|c| if c == '/' || c == ' ' { '-' } else { c })
        .collect()
}

/// Fold a Czech/Slovak diacritic to its ASCII base; pass anything else through.
/// Callers lowercase first, so only lowercase forms need listing.
fn fold_diacritic(c: char) -> char {
    match c {
        'á' | 'à' | 'â' | 'ä' => 'a',
        'č' | 'ç' => 'c',
        'ď' => 'd',
        'é' | 'ě' | 'ê' | 'ë' => 'e',
        'í' | 'î' | 'ï' => 'i',
        'ĺ' | 'ľ' => 'l',
        'ň' => 'n',
        'ó' | 'ô' | 'ö' => 'o',
        'ř' | 'ŕ' => 'r',
        'š' => 's',
        'ť' => 't',
        'ú' | 'ů' | 'ü' | 'û' => 'u',
        'ý' => 'y',
        'ž' => 'z',
        other => other,
    }
}

/// A filesystem-safe, human-readable slug for a document `typ`: lowercased,
/// diacritics folded to ASCII, every run of non-alphanumerics turned into a
/// single `-`, edges trimmed, length capped. `"účetní závěrka [2024]"` ->
/// `"ucetni-zaverka-2024"`. Returns an empty string when the typ has no
/// alphanumerics (the caller then falls back to the bare číslo).
pub fn slugify_typ(typ: &str) -> String {
    const MAX: usize = 80;
    let mut out = String::new();
    let mut pending_dash = false;
    for ch in typ.chars() {
        for lc in ch.to_lowercase() {
            let folded = fold_diacritic(lc);
            if folded.is_ascii_alphanumeric() {
                if pending_dash && !out.is_empty() {
                    out.push('-');
                }
                pending_dash = false;
                out.push(folded);
            } else {
                pending_dash = true;
            }
        }
    }
    // The slug is pure ASCII (`[a-z0-9-]`), so truncating on a byte boundary is
    // safe; strip any dash the cut leaves dangling.
    if out.len() > MAX {
        out.truncate(MAX);
        while out.ends_with('-') {
            out.pop();
        }
    }
    out
}

/// The on-disk stem for a downloaded document: `<typ-slug>__<cislo-safe>`, or
/// just `<cislo-safe>` when the typ slugs to nothing. The `__` is unambiguous -
/// neither the slug (`[a-z0-9-]`) nor `file_stem` (`[A-Z0-9-]`) contains `_`.
pub fn doc_file_stem(typ: &str, cislo: &str) -> String {
    let cislo_safe = file_stem(cislo);
    let slug = slugify_typ(typ);
    if slug.is_empty() {
        cislo_safe
    } else {
        format!("{slug}__{cislo_safe}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_typ_folds_diacritics_and_keeps_the_year() {
        assert_eq!(slugify_typ("účetní závěrka [2024]"), "ucetni-zaverka-2024");
        assert_eq!(slugify_typ("výroční zpráva"), "vyrocni-zprava");
        assert_eq!(slugify_typ("Účetní ZÁVĚRKA"), "ucetni-zaverka");
    }

    #[test]
    fn slugify_typ_collapses_punctuation_and_trims_edges() {
        assert_eq!(
            slugify_typ("smlouva o převodu obchodního podílu, notářský zápis"),
            "smlouva-o-prevodu-obchodniho-podilu-notarsky-zapis"
        );
        assert_eq!(slugify_typ("  [stanovy]  "), "stanovy");
    }

    #[test]
    fn slugify_typ_is_empty_when_no_alphanumerics() {
        assert_eq!(slugify_typ(""), "");
        assert_eq!(slugify_typ("[]  -- ,"), "");
    }

    #[test]
    fn slugify_typ_caps_length_without_a_trailing_dash() {
        let long = "slovo ".repeat(40); // ~240 chars of "slovo slovo ..."
        let s = slugify_typ(&long);
        assert!(s.len() <= 80, "len was {}", s.len());
        assert!(!s.ends_with('-'), "ended with dash: {s}");
        assert!(!s.starts_with('-'), "started with dash: {s}");
    }

    #[test]
    fn doc_file_stem_puts_the_typ_before_the_cislo() {
        assert_eq!(
            doc_file_stem("účetní závěrka [2024]", "B 8573/SL95/MSPH"),
            "ucetni-zaverka-2024__B-8573-SL95-MSPH"
        );
    }

    #[test]
    fn doc_file_stem_falls_back_to_cislo_when_typ_is_blank() {
        assert_eq!(doc_file_stem("", "B 8573/SL95/MSPH"), "B-8573-SL95-MSPH");
        assert_eq!(doc_file_stem("   ", "B 8573/SL95/MSPH"), "B-8573-SL95-MSPH");
    }

    // Real responses captured 2026-07-20 (Alza.cz a.s., IČO 27082440).
    const SEARCH_ICO: &str = include_str!("../testdata/search-alza-ico.html");
    const SEARCH_NAME: &str = include_str!("../testdata/search-alza-name.html");
    const SEARCH_NOTFOUND: &str = include_str!("../testdata/search-notfound.html");
    const LISTINY: &str = include_str!("../testdata/listiny-alza.html");
    const DETAIL: &str = include_str!("../testdata/detail-alza-sl95.html");

    #[test]
    fn search_count_reads_the_result_total() {
        assert_eq!(search_count(SEARCH_ICO), Some(1));
        assert_eq!(search_count(SEARCH_NAME), Some(9));
        assert_eq!(search_count(SEARCH_NOTFOUND), Some(0));
    }

    #[test]
    fn search_hits_parses_identity_and_subjekt_id() {
        let hits = search_hits(SEARCH_ICO);
        assert_eq!(hits.len(), 1);
        let h = &hits[0];
        assert_eq!(h.subjekt_id, "701502");
        assert_eq!(h.ico, "27082440");
        assert_eq!(h.nazev, "Alza.cz a.s.");
        assert!(h.spisovaznacka.as_deref().unwrap().contains("B 8573"));
        assert!(h.sidlo.as_deref().unwrap().contains("Jankovcova"));
    }

    #[test]
    fn search_hits_finds_every_result_and_keeps_ico_per_block() {
        let hits = search_hits(SEARCH_NAME);
        assert_eq!(hits.len(), 9);
        // Every hit carries an 8-digit IČO and a subjektId, so ambiguity can be
        // resolved by exact IČO rather than position.
        assert!(hits
            .iter()
            .all(|h| h.ico.len() == 8 && !h.subjekt_id.is_empty()));
        assert!(hits.iter().any(|h| h.ico == "27082440")); // Alza.cz a.s.
    }

    #[test]
    fn search_hits_empty_for_no_match() {
        assert!(search_hits(SEARCH_NOTFOUND).is_empty());
    }

    #[test]
    fn listiny_parses_every_row_with_download_coordinates() {
        let rows = listiny(LISTINY);
        assert_eq!(rows.len(), 93);
        let first = &rows[0];
        assert_eq!(first.cislo, "B 8573/SL95/MSPH");
        assert!(!first.typ.is_empty());
        assert_eq!(first.dokument, "90818001");
        assert_eq!(first.spis, "80343");
        // Within a subject only the SL segment varies; the číslo is the full id.
        assert!(rows
            .iter()
            .all(|r| r.cislo.starts_with("B 8573/") && r.cislo.ends_with("/MSPH")));
    }

    #[test]
    fn download_id_extracts_the_guid() {
        // The GUID rotates per request, so this asserts the value in the captured
        // fixture - which is why the download must fetch the detail fresh and use
        // the GUID immediately rather than storing it.
        assert_eq!(
            download_id(DETAIL).as_deref(),
            Some("23ac77e26442462492521fce848aebee")
        );
    }

    #[test]
    fn text_of_strips_tags_entities_and_nbsp() {
        assert_eq!(text_of("B&nbsp;8573/SL95/MSPH"), "B 8573/SL95/MSPH");
        assert_eq!(text_of("<span>  A &amp; B  </span>"), "A & B");
    }

    #[test]
    fn normalize_key_folds_spacing_and_case() {
        assert_eq!(normalize_key("b\u{a0}8573/sl95/msph"), "B 8573/SL95/MSPH");
    }

    #[test]
    fn file_stem_is_the_full_cislo_made_filesystem_safe() {
        assert_eq!(file_stem("B 8573/SL95/MSPH"), "B-8573-SL95-MSPH");
        assert_eq!(file_stem("b\u{a0}8573/sl95/msph"), "B-8573-SL95-MSPH");
    }
}
