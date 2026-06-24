use anyhow::{anyhow, Result};

#[derive(Clone, Debug, serde::Serialize)]
pub struct Entry {
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
        // tab-separated; tabuly v datech nahraď mezerou pro jistotu
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

impl Index {
    pub fn search(&self, query: &str, limit: usize) -> Vec<Entry> {
        let q = crate::text::fold(query);
        self.entries
            .iter()
            .filter(|e| crate::text::fold(&e.nazev).contains(&q))
            .take(limit)
            .cloned()
            .collect()
    }

    /// Kolik záznamů by `search` našel bez ořezu na `limit`.
    /// Slouží k rozlišení „přesně N výsledků" od „N z mnoha" (truncated).
    pub fn count_matches(&self, query: &str) -> usize {
        let q = crate::text::fold(query);
        self.entries
            .iter()
            .filter(|e| crate::text::fold(&e.nazev).contains(&q))
            .count()
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
    fn count_matches_ignores_limit() {
        let (tsv, _) = build_tsv_from_csv(SAMPLE_CSV).unwrap();
        let idx = parse_tsv(&tsv);
        // search ořízne na 1, count_matches vrátí skutečný počet
        assert_eq!(idx.search("", 1).len(), 1);
        assert_eq!(idx.count_matches(""), 2);
    }
}
