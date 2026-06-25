use std::collections::HashMap;

use anyhow::{anyhow, Result};
use serde_json::{json, Value};

use crate::paths;
use crate::text;

/// Columns each dataset must expose. If KSRZIS renames or drops one, loading
/// fails loudly here instead of silently returning empty strings downstream.
fn required_columns(dataset: &str) -> &'static [&'static str] {
    match dataset {
        "RZPRO_ZP" => &[
            "Evidenční číslo",
            "Název ZP",
            "Obchodní název",
            "Výrobce název",
            "Druh ZP",
            "Míra rizika",
            "Stát výrobce",
            "Stav",
            "Typ evidence",
            "Jedná se o příslušenství",
            "Jedná se o systém/soupravu",
            "Generická skupina",
            "Určený účel",
            "Určený účel v AJ",
            "Číslo certifikátu",
            "Číslo notifikované osoby",
            "Byla provedena KZ",
            "Byl vydán návod",
            "Existuje přidělené katalogové číslo",
            "Datum uvedení na trh",
            "Platnost od",
            "Platnost do",
        ],
        "RZPRO_ZP_VAR" => &[
            "Evidenční číslo",
            "Název ZP",
            "Identifikační kód",
            "Doplněk názvu",
            "Katalogové číslo",
            "Platnost varianty od",
            "Platnost varianty do",
        ],
        "RZPRO_ZP_OS" => &[
            "Evidenční číslo",
            "Název ZP",
            "Osoba název",
            "Osoba registrační číslo",
            "Činnost osoby",
            "Vlastník",
            "Platnost vazby ZP na osobu od",
            "Platnost vazby ZP na osobu do",
        ],
        "RZPRO_OS_CIN" => &[
            "Osoba název",
            "Registrační číslo",
            "IČ",
            "Registrace osoby",
            "Platnost činnosti od",
            "Platnost činnosti do",
        ],
        _ => &[],
    }
}

struct Table {
    headers: HashMap<String, usize>,
    rows: Vec<csv::StringRecord>,
}

impl Table {
    fn load(dataset: &str) -> Result<Table> {
        let path = paths::csv_path(dataset);
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_path(&path)
            .map_err(|e| anyhow!("cannot open {}: {e}", path.display()))?;
        let headers: HashMap<String, usize> = rdr
            .headers()?
            .iter()
            .enumerate()
            .map(|(i, h)| (h.to_string(), i))
            .collect();
        if let Some(missing) = missing_columns(&headers, required_columns(dataset)) {
            return Err(anyhow!(
                "dataset {dataset} is missing expected column(s): {missing} \
                 — the KSRZIS CSV schema may have changed"
            ));
        }
        let rows = rdr.records().collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(Table { headers, rows })
    }

    fn col<'a>(&self, rec: &'a csv::StringRecord, name: &str) -> &'a str {
        self.headers
            .get(name)
            .and_then(|&i| rec.get(i))
            .unwrap_or("")
    }
}

/// Returns the missing columns as a comma-joined string, or None if all present.
fn missing_columns(headers: &HashMap<String, usize>, required: &[&str]) -> Option<String> {
    let missing: Vec<&str> = required
        .iter()
        .copied()
        .filter(|c| !headers.contains_key(*c))
        .collect();
    if missing.is_empty() {
        None
    } else {
        Some(missing.join(", "))
    }
}

/// Evidence numbers are stored zero-padded to 8 digits. Accept "19" or "00000019".
fn norm_evid(input: &str) -> String {
    let t = input.trim();
    if !t.is_empty() && t.chars().all(|c| c.is_ascii_digit()) && t.len() <= 8 {
        format!("{t:0>8}")
    } else {
        t.to_string()
    }
}

fn truthy(s: &str) -> bool {
    matches!(s.trim(), "True" | "true" | "1" | "Ano" | "ano")
}

#[derive(Default)]
pub struct Filters {
    pub druh: Option<String>,
    pub riziko: Option<String>,
    pub stat: Option<String>,
    /// Keep only devices whose validity ends on or before this ISO date
    /// (e.g. "2027-01-01") — i.e. expiring by then.
    pub expiruje_do: Option<String>,
}

pub fn search(query: &str, f: &Filters, limit: usize) -> Result<Value> {
    let has_query = query.split_whitespace().next().is_some();
    let has_filter = f.druh.is_some()
        || f.riziko.is_some()
        || f.stat.is_some()
        || f.expiruje_do.is_some();
    if !has_query && !has_filter {
        return Err(anyhow!(
            "empty query: provide search text or at least one filter (--druh/--riziko/--stat/--expiruje-do)"
        ));
    }
    // druh/stat match as folded substrings; riziko as an exact folded value
    // (so --riziko I does not also match IIa/III/IVD).
    let druh = f.druh.as_deref().map(text::fold);
    let stat = f.stat.as_deref().map(text::fold);
    let riziko = f.riziko.as_deref().map(text::fold);

    let t = Table::load("RZPRO_ZP")?;
    let mut results = Vec::new();
    let mut count = 0usize;
    for rec in &t.rows {
        if let Some(d) = &druh {
            if !text::fold(t.col(rec, "Druh ZP")).contains(d.as_str()) {
                continue;
            }
        }
        if let Some(r) = &riziko {
            if text::fold(t.col(rec, "Míra rizika")) != *r {
                continue;
            }
        }
        if let Some(s) = &stat {
            if !text::fold(t.col(rec, "Stát výrobce")).contains(s.as_str()) {
                continue;
            }
        }
        if let Some(ed) = &f.expiruje_do {
            // ISO dates compare lexicographically; "Platnost do" is "YYYY-MM-DD…".
            let pd = t.col(rec, "Platnost do");
            let pd = pd.get(..10).unwrap_or(pd);
            if pd.is_empty() || pd > ed.as_str() {
                continue;
            }
        }
        if has_query {
            let nazev = t.col(rec, "Název ZP");
            let obchodni = t.col(rec, "Obchodní název");
            let vyrobce = t.col(rec, "Výrobce název");
            let hay = text::fold(&format!("{nazev} {obchodni} {vyrobce}"));
            if !text::matches_all_tokens(&hay, query) {
                continue;
            }
        }
        count += 1;
        if results.len() < limit {
            results.push(json!({
                "evidencniCislo": t.col(rec, "Evidenční číslo"),
                "nazev": t.col(rec, "Název ZP"),
                "obchodniNazev": t.col(rec, "Obchodní název"),
                "vyrobce": t.col(rec, "Výrobce název"),
                "druh": t.col(rec, "Druh ZP"),
                "miraRizika": t.col(rec, "Míra rizika"),
                "statVyrobce": t.col(rec, "Stát výrobce"),
                "stav": t.col(rec, "Stav"),
            }));
        }
    }
    Ok(json!({
        "query": query,
        "count": count,
        "shown": results.len(),
        "results": results,
    }))
}

/// Map a filter key to the CSV column whose distinct values it lists.
fn values_column(key: &str) -> Result<&'static str> {
    match key {
        "druh" => Ok("Druh ZP"),
        "riziko" => Ok("Míra rizika"),
        "stat" => Ok("Stát výrobce"),
        _ => Err(anyhow!("unknown column {key} (use druh/riziko/stat)")),
    }
}

/// Distinct values of a filterable column, with counts — so callers don't have to
/// guess the exact value a `search` filter expects. Read live from the data.
pub fn values(key: &str) -> Result<Value> {
    let header = values_column(key)?;
    let t = Table::load("RZPRO_ZP")?;
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for rec in &t.rows {
        let v = t.col(rec, header);
        if !v.is_empty() {
            *counts.entry(v).or_insert(0) += 1;
        }
    }
    let mut items: Vec<(&str, usize)> = counts.into_iter().collect();
    // most common first; ties broken alphabetically for stable output
    items.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    let values: Vec<Value> = items
        .iter()
        .map(|(v, c)| json!({ "hodnota": v, "count": c }))
        .collect();
    Ok(json!({
        "sloupec": key,
        "count": values.len(),
        "values": values,
    }))
}

pub fn detail(evid: &str) -> Result<Value> {
    let key = norm_evid(evid);
    let t = Table::load("RZPRO_ZP")?;
    let rec = t
        .rows
        .iter()
        .find(|r| t.col(r, "Evidenční číslo") == key)
        .ok_or_else(|| anyhow!("no device with Evidenční číslo {key}"))?;
    Ok(json!({
        "evidencniCislo": t.col(rec, "Evidenční číslo"),
        "nazevZp": t.col(rec, "Název ZP"),
        "obchodniNazev": t.col(rec, "Obchodní název"),
        "stav": t.col(rec, "Stav"),
        "druhZp": t.col(rec, "Druh ZP"),
        "miraRizika": t.col(rec, "Míra rizika"),
        "typEvidence": t.col(rec, "Typ evidence"),
        "prislusenstvi": t.col(rec, "Jedná se o příslušenství"),
        "systemSouprava": t.col(rec, "Jedná se o systém/soupravu"),
        "generickaSkupina": t.col(rec, "Generická skupina"),
        "urcenyUcel": t.col(rec, "Určený účel"),
        "urcenyUcelAj": t.col(rec, "Určený účel v AJ"),
        "cisloCertifikatu": t.col(rec, "Číslo certifikátu"),
        "cisloNotifikovaneOsoby": t.col(rec, "Číslo notifikované osoby"),
        "bylaProvedenaKz": t.col(rec, "Byla provedena KZ"),
        "bylVydanNavod": t.col(rec, "Byl vydán návod"),
        "prideleneKatalogoveCislo": t.col(rec, "Existuje přidělené katalogové číslo"),
        "datumUvedeniNaTrh": t.col(rec, "Datum uvedení na trh"),
        "platnostOd": t.col(rec, "Platnost od"),
        "platnostDo": t.col(rec, "Platnost do"),
        "vyrobceNazev": t.col(rec, "Výrobce název"),
        "statVyrobce": t.col(rec, "Stát výrobce"),
    }))
}

pub fn varianty(evid: &str) -> Result<Value> {
    let key = norm_evid(evid);
    let t = Table::load("RZPRO_ZP_VAR")?;
    let mut nazev = String::new();
    let mut out = Vec::new();
    for rec in &t.rows {
        if t.col(rec, "Evidenční číslo") != key {
            continue;
        }
        if nazev.is_empty() {
            nazev = t.col(rec, "Název ZP").to_string();
        }
        out.push(json!({
            "identifikacniKod": t.col(rec, "Identifikační kód"),
            "doplnekNazvu": t.col(rec, "Doplněk názvu"),
            "katalogoveCislo": t.col(rec, "Katalogové číslo"),
            "platnostOd": t.col(rec, "Platnost varianty od"),
            "platnostDo": t.col(rec, "Platnost varianty do"),
        }));
    }
    Ok(json!({
        "evidencniCislo": key,
        "nazev": nazev,
        "count": out.len(),
        "varianty": out,
    }))
}

/// Reverse lookup: find the device(s) a catalogue number belongs to.
/// Catalogue numbers in the source sometimes carry stray surrounding spaces,
/// so both sides are trimmed; matching is case-insensitive.
pub fn varianta(katalog: &str) -> Result<Value> {
    let needle = katalog.trim();
    let t = Table::load("RZPRO_ZP_VAR")?;
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for rec in &t.rows {
        if !t
            .col(rec, "Katalogové číslo")
            .trim()
            .eq_ignore_ascii_case(needle)
        {
            continue;
        }
        let evid = t.col(rec, "Evidenční číslo").to_string();
        // a catalogue number can repeat across a device's variants; one hit per code
        if !seen.insert((evid.clone(), t.col(rec, "Identifikační kód").to_string())) {
            continue;
        }
        out.push(json!({
            "evidencniCislo": evid,
            "nazev": t.col(rec, "Název ZP"),
            "katalogoveCislo": t.col(rec, "Katalogové číslo").trim(),
            "identifikacniKod": t.col(rec, "Identifikační kód"),
            "doplnekNazvu": t.col(rec, "Doplněk názvu"),
        }));
    }
    Ok(json!({
        "katalog": needle,
        "count": out.len(),
        "vysledky": out,
    }))
}

pub fn osoby(evid: &str) -> Result<Value> {
    let key = norm_evid(evid);

    // Manufacturer-of-record from the device card itself — always present, incl.
    // foreign makers. RZPRO_ZP_OS below only links devices to Czech-registered
    // persons, so it is empty for ~72% of devices (foreign manufacturers are not
    // registered persons). The `vyrobce` field keeps `osoby` from being a dead end.
    let zp = Table::load("RZPRO_ZP")?;
    let device = zp.rows.iter().find(|r| zp.col(r, "Evidenční číslo") == key);
    let nazev = device
        .map(|r| zp.col(r, "Název ZP").to_string())
        .unwrap_or_default();
    let vyrobce = match device {
        Some(r) => json!({
            "nazev": zp.col(r, "Výrobce název"),
            "stat": zp.col(r, "Stát výrobce"),
        }),
        None => Value::Null,
    };

    // Czech-registered persons linked to the device (notifier / distributor).
    let t = Table::load("RZPRO_ZP_OS")?;
    let mut out = Vec::new();
    for rec in &t.rows {
        if t.col(rec, "Evidenční číslo") != key {
            continue;
        }
        out.push(json!({
            "osobaNazev": t.col(rec, "Osoba název"),
            "registracniCislo": t.col(rec, "Osoba registrační číslo"),
            "cinnost": t.col(rec, "Činnost osoby"),
            "vlastnik": truthy(t.col(rec, "Vlastník")),
            "platnostOd": t.col(rec, "Platnost vazby ZP na osobu od"),
            "platnostDo": t.col(rec, "Platnost vazby ZP na osobu do"),
        }));
    }
    Ok(json!({
        "evidencniCislo": key,
        "nazev": nazev,
        "vyrobce": vyrobce,
        "count": out.len(),
        "osoby": out,
    }))
}

pub fn osoba(key: &str) -> Result<Value> {
    let k = key.trim();
    let reg_padded = if k.chars().all(|c| c.is_ascii_digit()) && k.len() <= 6 {
        format!("{k:0>6}")
    } else {
        k.to_string()
    };

    let cin = Table::load("RZPRO_OS_CIN")?;
    let mut identity: Option<(String, String, String)> = None;
    let mut cinnosti = Vec::new();
    for rec in &cin.rows {
        let reg = cin.col(rec, "Registrační číslo");
        let ic = cin.col(rec, "IČ");
        if reg != reg_padded && reg != k && ic != k {
            continue;
        }
        if identity.is_none() {
            identity = Some((
                cin.col(rec, "Osoba název").to_string(),
                reg.to_string(),
                ic.to_string(),
            ));
        }
        cinnosti.push(json!({
            "cinnost": cin.col(rec, "Registrace osoby"),
            "platnostOd": cin.col(rec, "Platnost činnosti od"),
            "platnostDo": cin.col(rec, "Platnost činnosti do"),
        }));
    }
    let (nazev, reg, ic) = identity
        .ok_or_else(|| anyhow!("no person with Registrační číslo / IČ {key}"))?;

    // Devices linked to this company, taken from the device↔person table we
    // already load (no separate RZPRO_OS_ZP download needed).
    let zp_os = Table::load("RZPRO_ZP_OS")?;
    let mut seen = std::collections::HashSet::new();
    let mut prostredky = Vec::new();
    for rec in &zp_os.rows {
        if zp_os.col(rec, "Osoba registrační číslo") != reg {
            continue;
        }
        let evid = zp_os.col(rec, "Evidenční číslo").to_string();
        if !seen.insert(evid.clone()) {
            continue;
        }
        prostredky.push(json!({
            "evidencniCislo": evid,
            "nazev": zp_os.col(rec, "Název ZP"),
            "cinnost": zp_os.col(rec, "Činnost osoby"),
            "vlastnik": truthy(zp_os.col(rec, "Vlastník")),
        }));
    }

    Ok(json!({
        "osoba": { "nazev": nazev, "registracniCislo": reg, "ic": ic },
        "cinnosti": cinnosti,
        "pocetProstredku": prostredky.len(),
        "prostredky": prostredky,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evid_padding() {
        assert_eq!(norm_evid("19"), "00000019");
        assert_eq!(norm_evid("00000019"), "00000019");
        assert_eq!(norm_evid(" 19 "), "00000019");
        assert_eq!(norm_evid("abc"), "abc");
    }

    #[test]
    fn truthy_values() {
        assert!(truthy("True"));
        assert!(!truthy("False"));
        assert!(!truthy(""));
    }

    #[test]
    fn missing_columns_detects_schema_drift() {
        let mut headers = HashMap::new();
        headers.insert("Evidenční číslo".to_string(), 0);
        headers.insert("Název ZP".to_string(), 1);
        // present subset → only the absent ones are reported
        let missing = missing_columns(&headers, &["Evidenční číslo", "Stav", "Druh ZP"]);
        assert_eq!(missing.as_deref(), Some("Stav, Druh ZP"));
        // all present → None
        assert_eq!(missing_columns(&headers, &["Evidenční číslo", "Název ZP"]), None);
    }

    #[test]
    fn every_dataset_has_required_columns() {
        for ds in ["RZPRO_ZP", "RZPRO_ZP_VAR", "RZPRO_ZP_OS", "RZPRO_OS_CIN"] {
            assert!(!required_columns(ds).is_empty(), "{ds} has no required columns");
        }
    }
}
