use std::collections::HashMap;
use std::path::Path;

use anyhow::{anyhow, Result};

use crate::paths;

fn col_index(header: &[&str], name: &str) -> Result<usize> {
    header
        .iter()
        .position(|h| h.trim() == name)
        .ok_or_else(|| anyhow!("CSV missing column {name}"))
}

pub fn load_map(path: &Path) -> HashMap<String, String> {
    let data = match std::fs::read_to_string(path) {
        Ok(d) => d,
        Err(_) => return HashMap::new(),
    };
    data.lines()
        .filter_map(|l| {
            let mut parts = l.splitn(2, '\t');
            let k = parts.next()?.to_string();
            let v = parts.next().unwrap_or("").to_string();
            // skip rows with an empty key OR value, so resolved() returns null
            // instead of an empty name
            if k.is_empty() || v.is_empty() {
                None
            } else {
                Some((k, v))
            }
        })
        .collect()
}

pub struct Codelists {
    pub latky: HashMap<String, String>,
    pub formy: HashMap<String, String>,
    pub cesty: HashMap<String, String>,
    pub vydej: HashMap<String, String>,
    pub stavyreg: HashMap<String, String>,
    pub atc: HashMap<String, String>,
    pub zavislost: HashMap<String, String>,
}

impl Codelists {
    pub fn load() -> Self {
        Self {
            latky: load_map(&paths::cis_path("latky.tsv")),
            formy: load_map(&paths::cis_path("formy.tsv")),
            cesty: load_map(&paths::cis_path("cesty.tsv")),
            vydej: load_map(&paths::cis_path("vydej.tsv")),
            stavyreg: load_map(&paths::cis_path("stavyreg.tsv")),
            atc: load_map(&paths::cis_path("atc.tsv")),
            zavislost: load_map(&paths::cis_path("zavislost.tsv")),
        }
    }
}

pub fn build_codelist_tsv(csv: &str, key_header: &str, val_header: &str) -> Result<String> {
    let mut lines = csv.lines();
    let header_line = lines.next().ok_or_else(|| anyhow!("empty CSV"))?;
    let header: Vec<&str> = header_line.split(';').collect();
    let i_key = col_index(&header, key_header)?;
    let i_val = col_index(&header, val_header)?;
    let mut out = String::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split(';').collect();
        let key = cols.get(i_key).map(|s| s.trim()).unwrap_or("");
        let val = cols.get(i_val).map(|s| s.trim()).unwrap_or("");
        if key.is_empty() {
            continue;
        }
        out.push_str(&key.replace('\t', " "));
        out.push('\t');
        out.push_str(&val.replace('\t', " "));
        out.push('\n');
    }
    Ok(out)
}

pub fn build_latky_tsv(csv: &str) -> Result<String> {
    let mut lines = csv.lines();
    let header_line = lines.next().ok_or_else(|| anyhow!("empty CSV"))?;
    let header: Vec<&str> = header_line.split(';').collect();
    let i_kod = col_index(&header, "KOD_LATKY")?;
    let i_nazev = col_index(&header, "NAZEV")?;
    let i_inn = col_index(&header, "NAZEV_INN")?;
    let mut out = String::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split(';').collect();
        let kod = cols.get(i_kod).map(|s| s.trim()).unwrap_or("");
        if kod.is_empty() {
            continue;
        }
        let nazev = cols.get(i_nazev).map(|s| s.trim()).unwrap_or("");
        let name = if nazev.is_empty() {
            cols.get(i_inn).map(|s| s.trim()).unwrap_or("")
        } else {
            nazev
        };
        if name.is_empty() {
            continue;
        }
        out.push_str(&kod.replace('\t', " "));
        out.push('\t');
        out.push_str(&name.replace('\t', " "));
        out.push('\n');
    }
    Ok(out)
}

/// Group substance synonyms (dlp_synonyma.csv: KOD_LATKY;SQ;ZDROJ;NAZEV, many rows
/// per substance) into a `code → "syn1 syn2 …"` map (lowercase) for search.
pub fn build_synonyma_map(csv: &str) -> Result<HashMap<String, String>> {
    let mut lines = csv.lines();
    let header_line = lines.next().ok_or_else(|| anyhow!("empty CSV"))?;
    let header: Vec<&str> = header_line.split(';').collect();
    let i_kod = col_index(&header, "KOD_LATKY")?;
    let i_nazev = col_index(&header, "NAZEV")?;
    let mut map: HashMap<String, String> = HashMap::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split(';').collect();
        let kod = cols.get(i_kod).map(|s| s.trim()).unwrap_or("");
        let nazev = cols.get(i_nazev).map(|s| s.trim()).unwrap_or("");
        if kod.is_empty() || nazev.is_empty() {
            continue;
        }
        let entry = map.entry(kod.to_string()).or_default();
        if !entry.is_empty() {
            entry.push(' ');
        }
        entry.push_str(&nazev.to_lowercase().replace('\t', " "));
    }
    Ok(map)
}

/// Search blob for `latka`: code → lower(NAZEV + NAZEV_EN + NAZEV_INN + synonyms),
/// so search hits English / Latin (INN) / any synonym, not just the Czech name.
pub fn build_latky_search_tsv(csv: &str, synonyma: &HashMap<String, String>) -> Result<String> {
    let mut lines = csv.lines();
    let header_line = lines.next().ok_or_else(|| anyhow!("empty CSV"))?;
    let header: Vec<&str> = header_line.split(';').collect();
    let i_kod = col_index(&header, "KOD_LATKY")?;
    let i_nazev = col_index(&header, "NAZEV")?;
    let i_en = col_index(&header, "NAZEV_EN")?;
    let i_inn = col_index(&header, "NAZEV_INN")?;
    let mut out = String::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split(';').collect();
        let kod = cols.get(i_kod).map(|s| s.trim()).unwrap_or("");
        if kod.is_empty() {
            continue;
        }
        let get = |i: usize| cols.get(i).map(|s| s.trim()).unwrap_or("");
        let mut blob = format!("{} {} {}", get(i_nazev), get(i_en), get(i_inn)).to_lowercase();
        if let Some(s) = synonyma.get(kod) {
            blob.push(' ');
            blob.push_str(s);
        }
        let blob = blob.replace('\t', " ");
        out.push_str(kod);
        out.push('\t');
        out.push_str(blob.split_whitespace().collect::<Vec<_>>().join(" ").trim());
        out.push('\n');
    }
    Ok(out)
}

pub fn build_latka_drugs_tsv(slozeni_csv: &str) -> Result<String> {
    let mut lines = slozeni_csv.lines();
    let header_line = lines.next().ok_or_else(|| anyhow!("empty CSV"))?;
    let header: Vec<&str> = header_line.split(';').collect();
    let i_kod = col_index(&header, "KOD_SUKL")?;
    let i_latka = col_index(&header, "KOD_LATKY")?;
    let mut out = String::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split(';').collect();
        let kod = cols.get(i_kod).map(|s| s.trim()).unwrap_or("");
        let latka = cols.get(i_latka).map(|s| s.trim()).unwrap_or("");
        if kod.is_empty() || latka.is_empty() {
            continue;
        }
        out.push_str(&latka.replace('\t', " "));
        out.push('\t');
        out.push_str(&kod.replace('\t', " "));
        out.push('\n');
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_simple_codelist_by_header() {
        let csv = "VYDEJ;NAZEV\nF;volně prodejné\nR;na předpis\n";
        let tsv = build_codelist_tsv(csv, "VYDEJ", "NAZEV").unwrap();
        assert!(tsv.contains("F\tvolně prodejné\n"));
        assert!(tsv.contains("R\tna předpis\n"));
    }

    #[test]
    fn codelist_skips_empty_key_rows() {
        let csv = "VYDEJ;NAZEV\n;prázdný klíč\nF;volně\n";
        let tsv = build_codelist_tsv(csv, "VYDEJ", "NAZEV").unwrap();
        assert!(!tsv.contains("prázdný"));
        assert!(tsv.contains("F\tvolně\n"));
    }

    #[test]
    fn codelist_errors_on_missing_column() {
        let csv = "FOO;BAR\n1;2\n";
        assert!(build_codelist_tsv(csv, "VYDEJ", "NAZEV").is_err());
    }

    #[test]
    fn latky_uses_nazev_with_inn_fallback() {
        let csv = "KOD_LATKY;ZDROJ;NAZEV_INN;NAZEV_EN;NAZEV;ZAV;DOP;NARVLA\n\
1064;;PARACETAMOLUM;;PARACETAMOL;;;\n\
9999;;ONLYINN;;;;;\n";
        let tsv = build_latky_tsv(csv).unwrap();
        assert!(tsv.contains("1064\tPARACETAMOL\n"));
        assert!(tsv.contains("9999\tONLYINN\n")); // empty NAZEV → INN fallback
    }

    #[test]
    fn latky_search_blob_has_cz_en_inn() {
        let csv = "KOD_LATKY;ZDROJ;NAZEV_INN;NAZEV_EN;NAZEV;ZAV;DOP;NARVLA\n\
1064;;PARACETAMOLUM;PARACETAMOL;PARACETAMOL;;;\n";
        let tsv = build_latky_search_tsv(csv, &HashMap::new()).unwrap();
        assert!(tsv.starts_with("1064\t"));
        assert!(tsv.contains("paracetamol"));
        assert!(tsv.contains("paracetamolum")); // INN
    }

    #[test]
    fn synonyma_grouped_and_folded_into_search() {
        let syn_csv = "KOD_LATKY;SQ;ZDROJ;NAZEV\n\
1064;21;X43;ACETAMINOPHEN\n\
1064;22;SLES;PARAZETAMOL\n";
        let syn = build_synonyma_map(syn_csv).unwrap();
        assert!(syn.get("1064").unwrap().contains("acetaminophen"));
        assert!(syn.get("1064").unwrap().contains("parazetamol"));

        let latky = "KOD_LATKY;ZDROJ;NAZEV_INN;NAZEV_EN;NAZEV;ZAV;DOP;NARVLA\n\
1064;;PARACETAMOLUM;PARACETAMOL;PARACETAMOL;;;\n";
        let tsv = build_latky_search_tsv(latky, &syn).unwrap();
        assert!(tsv.contains("acetaminophen")); // synonymum se dostalo do blobu
    }

    #[test]
    fn latka_drugs_maps_substance_to_drug() {
        let csv = "KOD_SUKL;KOD_LATKY;SQ;S;AMNT_OD;AMNT;UN\n\
0000009;12;1;L;;450;MG\n\
0182362;1064;1;L;;500;MG\n";
        let tsv = build_latka_drugs_tsv(csv).unwrap();
        assert!(tsv.contains("12\t0000009\n"));
        assert!(tsv.contains("1064\t0182362\n"));
    }

    #[test]
    fn load_map_reads_tsv_and_missing_is_empty() {
        let dir = std::env::temp_dir().join("leciva_cltest");
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("x.tsv");
        std::fs::write(&p, "F\tvolně prodejné\nR\tna předpis\n").unwrap();
        let m = load_map(&p);
        assert_eq!(m.get("F").map(String::as_str), Some("volně prodejné"));
        let missing = load_map(&dir.join("nope.tsv"));
        assert!(missing.is_empty());
    }
}
