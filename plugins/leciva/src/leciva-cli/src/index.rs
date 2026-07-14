use anyhow::{anyhow, Result};

#[derive(Clone, Debug, serde::Serialize)]
pub struct Entry {
    #[serde(rename = "kodSukl")] // align with `latka` output and the SKILL.md contract
    pub kod: String,
    pub nazev: String,
    pub sila: String,
    pub doplnek: String,
}

pub struct Index {
    pub entries: Vec<Entry>,
}

fn split_semicolon(line: &str) -> Vec<&str> {
    line.split(';').collect()
}

fn col_index(header: &[&str], name: &str) -> Result<usize> {
    header
        .iter()
        .position(|h| h.trim() == name)
        .ok_or_else(|| anyhow!("CSV missing column {name}"))
}

pub fn build_tsv_from_csv(csv: &str) -> Result<(String, usize)> {
    let mut lines = csv.lines();
    let header_line = lines.next().ok_or_else(|| anyhow!("empty CSV"))?;
    let header = split_semicolon(header_line);
    let i_kod = col_index(&header, "KOD_SUKL")?;
    let i_nazev = col_index(&header, "NAZEV")?;
    let i_sila = col_index(&header, "SILA")?;
    let i_doplnek = col_index(&header, "DOPLNEK")?;

    let mut out = String::new();
    let mut count = 0usize;
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let cols = split_semicolon(line);
        let get = |i: usize| cols.get(i).map(|s| s.trim()).unwrap_or("");
        // replace any tab in the data — it would break the TSV layout
        let row = format!(
            "{}\t{}\t{}\t{}",
            get(i_kod).replace('\t', " "),
            get(i_nazev).replace('\t', " "),
            get(i_sila).replace('\t', " "),
            get(i_doplnek).replace('\t', " "),
        );
        out.push_str(&row);
        out.push('\n');
        count += 1;
    }
    Ok((out, count))
}

pub fn parse_tsv(tsv: &str) -> Index {
    let entries = tsv
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| {
            let mut p = l.split('\t');
            Some(Entry {
                kod: p.next()?.to_string(),
                nazev: p.next()?.to_string(),
                sila: p.next().unwrap_or("").to_string(),
                doplnek: p.next().unwrap_or("").to_string(),
            })
        })
        .collect();
    Index { entries }
}

/// One query term: a Number matches a whole-number strength in name+strength;
/// Text matches as a substring in name+strength+packaging.
enum Term {
    Number(String),
    Text(String),
}

/// Split the folded query into terms. Any non-alphanumeric char (`/`, `,`, …) and
/// whitespace break tokens, and a number glued to a unit is split too
/// ("50mcg" → 50 + "mcg"), so the strength matches as a whole number, not a
/// substring of "150mcg". Empty query → no terms → matches everything.
fn query_terms(query: &str) -> Vec<Term> {
    let normalized: String = crate::text::fold(query)
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect();
    normalized
        .split_whitespace()
        .flat_map(|tok| {
            let digits: String = tok.chars().take_while(|c| c.is_ascii_digit()).collect();
            if digits.is_empty() {
                return vec![Term::Text(tok.to_string())];
            }
            let rest = &tok[digits.len()..]; // digits are ASCII → byte len == char count
            let mut out = vec![Term::Number(digits)];
            if !rest.is_empty() {
                out.push(Term::Text(rest.to_string()));
            }
            out
        })
        .collect()
}

/// Matches `num` in `hay` only as a whole number: `"50"` hits `"50MG"` but not
/// `"150MG"` nor `"500MG"`. Byte indexing is UTF-8-safe (a non-ASCII byte is not
/// an ascii digit, so it counts as a boundary).
fn contains_whole_number(hay: &str, num: &str) -> bool {
    if num.is_empty() {
        return false; // empty would make `find` spin forever
    }
    let hb = hay.as_bytes();
    let n = num.len();
    let mut start = 0;
    while let Some(off) = hay[start..].find(num) {
        let pos = start + off;
        let before_ok = pos == 0 || !hb[pos - 1].is_ascii_digit();
        let after_ok = pos + n >= hb.len() || !hb[pos + n].is_ascii_digit();
        if before_ok && after_ok {
            return true;
        }
        start = pos + 1;
    }
    false
}

impl Entry {
    /// All terms must match (AND). A Number is matched as a whole number in
    /// `nazev + sila` only — never the package size, so "euthyrox 50" won't hit
    /// 50-count packs of other strengths. Text is a substring of
    /// `nazev + sila + doplnek` (so form words like "sup"/"tbl" match). The code is
    /// deliberately not in the haystack: `search` is a name→code resolver.
    /// Haystacks are folded lazily — most queries are text-only and never build `strength`.
    fn matches(&self, terms: &[Term]) -> bool {
        let mut strength: Option<String> = None;
        let mut full: Option<String> = None;
        terms.iter().all(|term| match term {
            Term::Number(n) => {
                let h = strength
                    .get_or_insert_with(|| crate::text::fold(&format!("{} {}", self.nazev, self.sila)));
                contains_whole_number(h, n)
            }
            Term::Text(t) => {
                let h = full.get_or_insert_with(|| {
                    crate::text::fold(&format!("{} {} {}", self.nazev, self.sila, self.doplnek))
                });
                h.contains(t.as_str())
            }
        })
    }
}

impl Index {
    /// Single pass → (true match count, first `limit` results). Combined so the
    /// index isn't scanned (and folded) twice for the count and the listing.
    pub fn search_counted(&self, query: &str, limit: usize) -> (usize, Vec<Entry>) {
        let terms = query_terms(query);
        let mut total = 0;
        let mut hits = Vec::new();
        for e in &self.entries {
            if e.matches(&terms) {
                total += 1;
                if hits.len() < limit {
                    hits.push(e.clone());
                }
            }
        }
        (total, hits)
    }

    #[cfg(test)]
    pub fn search(&self, query: &str, limit: usize) -> Vec<Entry> {
        self.search_counted(query, limit).1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_CSV: &str = "KOD_SUKL;H;NAZEV;SILA;FORMA;BALENI;CESTA;DOPLNEK\n\
0000009;;ACYLCOFFIN;450MG/50MG;TBL NOB;10;POR;450MG/50MG TBL NOB 10\n\
0182362;;PARALEN;500MG;TBL NOB;24;POR;500MG TBL NOB 24\n";

    #[test]
    fn builds_tsv_from_csv_by_header() {
        let (tsv, count) = build_tsv_from_csv(SAMPLE_CSV).unwrap();
        assert_eq!(count, 2);
        assert!(tsv.contains("0000009\tACYLCOFFIN\t450MG/50MG\t450MG/50MG TBL NOB 10"));
        assert!(tsv.contains("0182362\tPARALEN\t500MG\t500MG TBL NOB 24"));
    }

    #[test]
    fn build_fails_on_missing_column() {
        let bad = "FOO;BAR\n1;2\n";
        assert!(build_tsv_from_csv(bad).is_err());
    }

    #[test]
    fn search_is_case_insensitive_substring_on_name() {
        let (tsv, _) = build_tsv_from_csv(SAMPLE_CSV).unwrap();
        let idx = parse_tsv(&tsv);
        let hits = idx.search("paralen", 10);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].kod, "0182362");
    }

    #[test]
    fn search_respects_limit() {
        let (tsv, _) = build_tsv_from_csv(SAMPLE_CSV).unwrap();
        let idx = parse_tsv(&tsv);
        assert_eq!(idx.search("", 1).len(), 1);
    }

    #[test]
    fn search_counted_returns_total_and_capped_hits() {
        let (tsv, _) = build_tsv_from_csv(SAMPLE_CSV).unwrap();
        let idx = parse_tsv(&tsv);
        let (total, hits) = idx.search_counted("", 1);
        assert_eq!(total, 2);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn search_matches_name_plus_strength() {
        let (tsv, _) = build_tsv_from_csv(SAMPLE_CSV).unwrap();
        let idx = parse_tsv(&tsv);
        let hits = idx.search("paralen 500", 10);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].kod, "0182362");
        assert_eq!(idx.search("paralen 999", 10).len(), 0);
    }

    #[test]
    fn search_tokens_are_order_independent() {
        let (tsv, _) = build_tsv_from_csv(SAMPLE_CSV).unwrap();
        let idx = parse_tsv(&tsv);
        assert_eq!(idx.search("500 paralen", 10).len(), 1);
    }

    #[test]
    fn numeric_token_matches_whole_number_only() {
        // ACYLCOFFIN 450MG/50MG: "50" hits the /50MG component, "45"/"500"/"5" don't —
        // the whole-number rule keeps "euthyrox 50" off 150MCG and pack sizes.
        let (tsv, _) = build_tsv_from_csv(SAMPLE_CSV).unwrap();
        let idx = parse_tsv(&tsv);
        assert_eq!(idx.search("acylcoffin 50", 10).len(), 1);
        assert_eq!(idx.search("acylcoffin 450", 10).len(), 1);
        assert_eq!(idx.search("acylcoffin 45", 10).len(), 0);
        assert_eq!(idx.search("acylcoffin 5", 10).len(), 0);
    }

    #[test]
    fn numeric_token_ignores_packaging_count() {
        // PARALEN's doplnek is "...TBL NOB 24"; "paralen 24" must not match on pack size.
        let (tsv, _) = build_tsv_from_csv(SAMPLE_CSV).unwrap();
        let idx = parse_tsv(&tsv);
        assert_eq!(idx.search("paralen 24", 10).len(), 0);
    }

    #[test]
    fn strength_with_attached_unit_is_whole_number() {
        // "500mg" (unit glued, no space) must still be a whole-number strength match —
        // "50mg" must NOT hit "500MG" the way a substring would.
        let (tsv, _) = build_tsv_from_csv(SAMPLE_CSV).unwrap();
        let idx = parse_tsv(&tsv);
        assert_eq!(idx.search("paralen 500mg", 10).len(), 1);
        assert_eq!(idx.search("paralen 50mg", 10).len(), 0);
        assert_eq!(idx.search("acylcoffin 50mg", 10).len(), 1); // /50MG component
    }

    #[test]
    fn slash_separates_combined_strengths() {
        // ACYLCOFFIN 450MG/50MG — a slash splits like a space.
        let (tsv, _) = build_tsv_from_csv(SAMPLE_CSV).unwrap();
        let idx = parse_tsv(&tsv);
        assert_eq!(idx.search("acylcoffin 450/50", 10).len(), 1);
        assert_eq!(idx.search("acylcoffin 450/99", 10).len(), 0);
    }
}
