use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use serde_json::json;

use std::collections::HashMap;

use crate::codelist::Codelists;
use crate::dataset;
use crate::http::SuklClient;
use crate::index::{self, Index};
use crate::manifest;
use crate::paths;

const ROOT_HELP: &str = "\
leciva-cli — vyhledávání léků (SÚKL DLP).

Výstup: JSON na stdout (zpracuj `jq`); při chybě `ERROR:` na stderr + nenulový exit.
Hledání podle názvu jede z lokálního indexu (stáhne se sám při prvním použití),
detail/složení z živého API podle kódu SÚKL.

Příklady:
  leciva-cli search \"paralen\"
  leciva-cli detail 0182362
  leciva-cli detail 0182362 --all
  leciva-cli slozeni 0182362
  leciva-cli latka \"ibuprofen\"
  leciva-cli refresh
  leciva-cli index status
";

#[derive(Parser, Debug)]
#[command(name = "leciva-cli", version, long_about = ROOT_HELP, arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Hledání léku podle názvu (lokální index).
    Search {
        query: String,
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// Detail léku podle kódu SÚKL (živé API + lokální názvy).
    Detail {
        kod: String,
        /// Vypíše všechna pole (s dopsanými názvy), ne jen kurátorovaný výběr.
        #[arg(long)]
        all: bool,
    },
    /// Složení léku podle kódu SÚKL (živé API + názvy látek).
    Slozeni { kod: String },
    /// Hledání léků podle účinné látky (lokální index).
    Latka {
        query: String,
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// Ceny a úhrady léku podle kódu SÚKL (živé API).
    Ceny { kod: String },
    /// Dokumenty léku — příbalový leták / SPC (živé API).
    Dokumenty { kod: String },
    /// Stáhnout/přestavět lokální index.
    Refresh {
        #[arg(long)]
        force: bool,
    },
    /// Stav lokálního indexu.
    #[command(subcommand)]
    Index(IndexCommand),
}

#[derive(Subcommand, Debug)]
pub enum IndexCommand {
    /// Vypíše manifest (verze/stáří indexu).
    Status,
}

fn ensure_index(client: &SuklClient) -> Result<()> {
    if paths::index_path().exists() && paths::manifest_path().exists() {
        return Ok(());
    }
    dataset::refresh(client, true)?;
    Ok(())
}

fn load_index() -> Result<Index> {
    let tsv = std::fs::read_to_string(paths::index_path())
        .map_err(|e| anyhow!("index not available: {e}"))?;
    Ok(index::parse_tsv(&tsv))
}

fn print_json(value: &serde_json::Value) {
    println!(
        "{}",
        serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
    );
}

fn s<'a>(v: &'a serde_json::Value, k: &str) -> Option<&'a str> {
    v.get(k).and_then(|x| x.as_str()).filter(|s| !s.is_empty())
}

fn resolved(map: &HashMap<String, String>, code: Option<&str>) -> serde_json::Value {
    match code.and_then(|c| map.get(c)) {
        Some(name) => serde_json::Value::String(name.clone()),
        None => serde_json::Value::Null,
    }
}

pub fn curate_detail(raw: &serde_json::Value, cl: &Codelists) -> serde_json::Value {
    use serde_json::{json, Value};
    let forma_kod = s(raw, "lekovaFormaKod");
    let cesta_kod = s(raw, "cestaKod");
    let vydej_kod = s(raw, "zpusobVydejeKod");
    let stav_kod = s(raw, "stavRegistraceKod");
    let drz_kod = s(raw, "drzitelKod");
    let atc_kod = s(raw, "ATCkod");
    let zav_kod = s(raw, "zavislostKod");

    // účinné látky: pole kódů → názvy (zachovej i syrové kódy)
    let kody: Vec<i64> = raw
        .get("leciveLatky")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_i64()).collect())
        .unwrap_or_default();
    let nazvy: Vec<Value> = kody
        .iter()
        .map(|k| match cl.latky.get(&k.to_string()) {
            Some(n) => Value::String(n.clone()),
            None => Value::Null,
        })
        .collect();

    let mut out = json!({
        "kodSukl": raw.get("kodSUKL").cloned().unwrap_or(Value::Null),
        "nazev": raw.get("nazev").cloned().unwrap_or(Value::Null),
        "sila": raw.get("sila").cloned().unwrap_or(Value::Null),
        "doplnek": raw.get("doplnek").cloned().unwrap_or(Value::Null),
        "baleni": raw.get("baleni").cloned().unwrap_or(Value::Null),
        "forma": resolved(&cl.formy, forma_kod),
        "formaKod": forma_kod,
        "cestaPodani": resolved(&cl.cesty, cesta_kod),
        "atc": { "kod": atc_kod, "nazev": resolved(&cl.atc, atc_kod) },
        "vydej": resolved(&cl.vydej, vydej_kod),
        "stavRegistrace": resolved(&cl.stavyreg, stav_kod),
        "drzitelKod": drz_kod,
        "registracniCislo": raw.get("registracniCislo").cloned().unwrap_or(Value::Null),
        // jeDodavka = dodáván do lékáren/zdr. zařízení v posledních 6 měsících → prakticky „k dostání" (SÚKL field, zachován)
        "jeDodavka": raw.get("jeDodavka").cloned().unwrap_or(Value::Null),
        "ucinneLatky": nazvy,
        "ucinneLatkyKody": kody,
    });
    // závislost jen když je vyplněná
    if zav_kod.is_some() {
        out["zavislost"] = resolved(&cl.zavislost, zav_kod);
    }
    out
}

/// Plný záznam (nic nezahozeno) + ke kódům dopsané čitelné názvy.
pub fn detail_all(raw: &serde_json::Value, cl: &Codelists) -> serde_json::Value {
    use serde_json::Value;
    let mut out = raw.clone();
    out["forma"] = resolved(&cl.formy, s(raw, "lekovaFormaKod"));
    out["cestaPodani"] = resolved(&cl.cesty, s(raw, "cestaKod"));
    out["vydej"] = resolved(&cl.vydej, s(raw, "zpusobVydejeKod"));
    out["stavRegistrace"] = resolved(&cl.stavyreg, s(raw, "stavRegistraceKod"));
    out["atcNazev"] = resolved(&cl.atc, s(raw, "ATCkod"));
    if s(raw, "zavislostKod").is_some() {
        out["zavislost"] = resolved(&cl.zavislost, s(raw, "zavislostKod"));
    }
    let nazvy: Vec<Value> = raw
        .get("leciveLatky")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_i64())
                .map(|k| match cl.latky.get(&k.to_string()) {
                    Some(n) => Value::String(n.clone()),
                    None => Value::Null,
                })
                .collect()
        })
        .unwrap_or_default();
    out["ucinneLatkyNazvy"] = Value::Array(nazvy);
    out
}

/// Složení z živého API + dopsané názvy účinných látek (s deduplikací).
pub fn resolve_slozeni(raw: &serde_json::Value, cl: &Codelists) -> serde_json::Value {
    use serde_json::{json, Value};
    let items = match raw.as_array() {
        Some(a) => a,
        None => return json!([]),
    };
    let mut seen = std::collections::HashSet::new();
    let mut out: Vec<Value> = Vec::new();
    for it in items {
        let kod = it.get("kodLatky").and_then(|v| v.as_i64());
        let mnozstvi = it.get("mnozstvi").and_then(|v| v.as_str()).unwrap_or("");
        let jednotka = it.get("jednotkaKod").and_then(|v| v.as_str()).unwrap_or("");
        let key = format!("{kod:?}|{mnozstvi}|{jednotka}");
        if !seen.insert(key) {
            continue;
        }
        let latka = kod
            .and_then(|k| cl.latky.get(&k.to_string()))
            .map(|n| Value::String(n.clone()))
            .unwrap_or(Value::Null);
        out.push(json!({
            "latka": latka,
            "kodLatky": kod,
            "mnozstvi": mnozstvi,
            "jednotka": jednotka,
        }));
    }
    Value::Array(out)
}

/// Ceny a úhrady z /cau-scau, kurátorované. Bez záznamu (OTC/nehrazený) → null.
pub fn curate_ceny(kod: &str, raw: &serde_json::Value) -> serde_json::Value {
    use serde_json::{json, Value};
    // „bez záznamu": chybový objekt (kodChyby), prázdné/ne-objektové tělo (404 bez JSON),
    // nebo objekt bez cenových polí.
    let has_record = raw.is_object()
        && raw.get("kodChyby").is_none()
        && (raw.get("cenaPuvodce").is_some() || raw.get("uhrady").is_some());
    if !has_record {
        return json!({
            "kodSukl": kod,
            "maZaznam": false,
            "ceny": Value::Null,
            "poznamka": "bez záznamu o ceně/úhradě (např. volně prodejný nebo nehrazený lék)",
        });
    }
    let uhrady: Vec<Value> = raw
        .get("uhrady")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .map(|u| {
                    json!({
                        "uhradaPojistovny": u.get("uhrada").cloned().unwrap_or(Value::Null),
                        "doplatekPacienta": u.get("zapocitatelnyDoplatek").cloned().unwrap_or(Value::Null),
                        "indikacniOmezeni": u.get("indikacniOmezeni").cloned().unwrap_or(Value::Null),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    json!({
        "kodSukl": kod,
        "maZaznam": true,
        "cenaPuvodce": raw.get("cenaPuvodce").cloned().unwrap_or(Value::Null),
        "maxCenaLekarna": raw.get("maxCenaLekarna").cloned().unwrap_or(Value::Null),
        "dph": raw.get("dph").cloned().unwrap_or(Value::Null),
        "uhrady": uhrady,
    })
}

/// Seznam dokumentů z /dokumenty-metadata + odkaz na PDF (typ lowercase).
pub fn dokumenty_list(kod: &str, base: &str, raw: &serde_json::Value) -> serde_json::Value {
    use serde_json::{json, Value};
    let items: Vec<Value> = raw
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|d| {
                    let typ = d.get("typ").and_then(|v| v.as_str())?;
                    Some(json!({
                        "typ": typ,
                        "nazev": d.get("nazev").cloned().unwrap_or(Value::Null),
                        "url": format!("{}/dokumenty/{}/{}", base, kod, typ.to_lowercase()),
                    }))
                })
                .collect()
        })
        .unwrap_or_default();
    json!({ "kodSukl": kod, "dokumenty": items })
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let client = SuklClient::new()?;

    match cli.command {
        Command::Search { query, limit } => {
            ensure_index(&client)?;
            let idx = load_index()?;
            let total = idx.count_matches(&query);
            let hits = idx.search(&query, limit);
            let mut out = json!({
                "query": query,
                "count": total,          // skutečný počet shod (i když je víc než zobrazeno)
                "shown": hits.len(),
                "results": hits,
            });
            if total > hits.len() {
                out["truncated"] = json!(true);
                out["poznamka"] = json!(format!(
                    "zobrazeno {} z {} — na úplný výčet přidej `--limit {}`",
                    hits.len(), total, total
                ));
            }
            print_json(&out);
        }
        Command::Detail { kod, all } => {
            let body = client.get(&format!("/lecive-pripravky/{kod}"))?;
            let parsed: serde_json::Value = serde_json::from_str(&body)
                .map_err(|e| anyhow!("cannot parse detail: {e}"))?;
            // API vrací pole s jedním prvkem
            let rec = parsed.as_array().and_then(|a| a.first()).unwrap_or(&parsed);
            let cl = crate::codelist::Codelists::load();
            if all {
                print_json(&detail_all(rec, &cl));
            } else {
                print_json(&curate_detail(rec, &cl));
            }
        }
        Command::Slozeni { kod } => {
            let body = client.get(&format!("/slozeni/{kod}"))?;
            let parsed: serde_json::Value = serde_json::from_str(&body)
                .map_err(|e| anyhow!("cannot parse slozeni: {e}"))?;
            let cl = crate::codelist::Codelists::load();
            print_json(&resolve_slozeni(&parsed, &cl));
        }
        Command::Latka { query, limit } => {
            ensure_index(&client)?;
            let q = crate::text::fold(&query); // diakritika-insensitivní hledání
            // matchuj proti blobu cz+en+inn+synonyma (latky-search.tsv)
            let search_path = paths::cis_path("latky-search.tsv");
            let index_present = search_path.exists();
            let mut latka_kody: std::collections::HashSet<String> = std::collections::HashSet::new();
            if let Ok(text) = std::fs::read_to_string(&search_path) {
                for line in text.lines() {
                    let mut p = line.splitn(2, '\t');
                    let (k, blob) = (p.next().unwrap_or(""), p.next().unwrap_or(""));
                    if !k.is_empty() && crate::text::fold(blob).contains(&q) {
                        latka_kody.insert(k.to_string());
                    }
                }
            }
            let mut drug_kody: std::collections::HashSet<String> = std::collections::HashSet::new();
            if !latka_kody.is_empty() {
                if let Ok(text) = std::fs::read_to_string(paths::latka_drugs_path()) {
                    for line in text.lines() {
                        let mut p = line.splitn(2, '\t');
                        let (lk, dk) = (p.next().unwrap_or(""), p.next().unwrap_or(""));
                        if latka_kody.contains(lk) && !dk.is_empty() {
                            drug_kody.insert(dk.to_string());
                        }
                    }
                }
            }
            let idx = load_index()?;
            let by_kod: std::collections::HashMap<&str, &index::Entry> =
                idx.entries.iter().map(|e| (e.kod.as_str(), e)).collect();
            let mut results: Vec<serde_json::Value> = drug_kody
                .iter()
                .filter_map(|k| by_kod.get(k.as_str()))
                .map(|e| json!({ "kodSukl": e.kod, "nazev": e.nazev, "sila": e.sila }))
                .collect();
            results.sort_by(|a, b| a["kodSukl"].as_str().cmp(&b["kodSukl"].as_str()));
            let count = results.len();
            results.truncate(limit);
            let shown = results.len();
            let mut out = json!({
                "query": query,
                "count": count,   // skutečný počet shod (i když je víc než zobrazeno)
                "shown": shown,
                "results": results,
            });
            if count > shown {
                out["truncated"] = json!(true);
                out["poznamka"] = json!(format!(
                    "zobrazeno {shown} z {count} — na úplný výčet přidej `--limit {count}`"
                ));
            }
            if !index_present {
                out["poznamka"] =
                    json!("index účinných látek chybí — spusť `leciva-cli refresh`");
            }
            print_json(&out);
        }
        Command::Ceny { kod } => {
            let body = client.get_lenient(&format!("/cau-scau/{kod}"))?;
            // 404 může vrátit prázdné/ne-JSON tělo → Null → curate_ceny vrátí „bez záznamu"
            let parsed = serde_json::from_str(&body).unwrap_or(serde_json::Value::Null);
            print_json(&curate_ceny(&kod, &parsed));
        }
        Command::Dokumenty { kod } => {
            let body = client.get_lenient(&format!("/dokumenty-metadata/{kod}"))?;
            // 404 může vrátit prázdné/ne-JSON tělo → Null → dokumenty_list vrátí prázdný seznam
            let parsed = serde_json::from_str(&body).unwrap_or(serde_json::Value::Null);
            print_json(&dokumenty_list(&kod, client.base_url(), &parsed));
        }
        Command::Refresh { force } => {
            let outcome = dataset::refresh(&client, force)?;
            print_json(&serde_json::to_value(&outcome)?);
        }
        Command::Index(IndexCommand::Status) => match manifest::load() {
            Some(m) => print_json(&serde_json::to_value(&m)?),
            None => print_json(&json!({ "status": "missing" })),
        },
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cl() -> Codelists {
        let mk = |pairs: &[(&str, &str)]| {
            pairs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect::<HashMap<String, String>>()
        };
        Codelists {
            latky: mk(&[("1593", "IBUPROFEN")]),
            formy: mk(&[("TBL FLM", "Potahovaná tableta")]),
            cesty: mk(&[("POR", "Perorální podání")]),
            vydej: mk(&[("F", "volně prodejné léčivé přípravky")]),
            stavyreg: mk(&[("R", "registrovaný léčivý přípravek")]),
            atc: mk(&[("M01AE01", "ibuprofen")]),
            zavislost: HashMap::new(),
        }
    }

    #[test]
    fn curate_resolves_and_drops_noise() {
        let raw = json!({
            "kodSUKL": "0254299", "nazev": "IBALGIN", "sila": "400MG", "doplnek": "400MG TBL FLM 24",
            "baleni": "24", "lekovaFormaKod": "TBL FLM", "cestaKod": "POR",
            "ATCkod": "M01AE01", "zpusobVydejeKod": "F", "stavRegistraceKod": "R",
            "drzitelKod": "OHCCZ", "registracniCislo": "29/154/88-C", "leciveLatky": [1593],
            "jeDodavka": true, "MRPcislo": "X", "braillovoPismo": "S"
        });
        let out = curate_detail(&raw, &cl());
        assert_eq!(out["forma"], "Potahovaná tableta");
        assert_eq!(out["cestaPodani"], "Perorální podání");
        assert_eq!(out["vydej"], "volně prodejné léčivé přípravky");
        assert_eq!(out["atc"]["nazev"], "ibuprofen");
        assert_eq!(out["drzitelKod"], "OHCCZ");
        assert_eq!(out["ucinneLatky"][0], "IBUPROFEN");
        assert_eq!(out["jeDodavka"], true); // SÚKL field zachován (k dostání)
        assert!(out.get("MRPcislo").is_none()); // jiný šum zahozen
    }

    #[test]
    fn curate_keeps_raw_code_when_codelist_missing() {
        let raw = json!({ "kodSUKL": "1", "nazev": "X", "lekovaFormaKod": "ZZZ", "leciveLatky": [999] });
        let out = curate_detail(&raw, &cl());
        assert_eq!(out["forma"], serde_json::Value::Null);
        assert_eq!(out["formaKod"], "ZZZ");
        assert_eq!(out["ucinneLatkyKody"][0], 999);
    }

    #[test]
    fn detail_all_keeps_everything_and_adds_names() {
        let raw = json!({
            "kodSUKL": "1", "lekovaFormaKod": "TBL FLM", "cestaKod": "POR",
            "zpusobVydejeKod": "F", "leciveLatky": [1593], "jeDodavka": true
        });
        let out = detail_all(&raw, &cl());
        // nic se nezahodí
        assert_eq!(out["jeDodavka"], true);
        assert_eq!(out["lekovaFormaKod"], "TBL FLM");
        // názvy dopsané
        assert_eq!(out["forma"], "Potahovaná tableta");
        assert_eq!(out["cestaPodani"], "Perorální podání");
        assert_eq!(out["vydej"], "volně prodejné léčivé přípravky");
        assert_eq!(out["ucinneLatkyNazvy"][0], "IBUPROFEN");
    }

    #[test]
    fn slozeni_adds_substance_names() {
        let raw = json!([
            { "kodLatky": 1593, "mnozstvi": "400", "jednotkaKod": "MG" },
            { "kodLatky": 999, "mnozstvi": "2", "jednotkaKod": "MG" }
        ]);
        let out = resolve_slozeni(&raw, &cl());
        assert_eq!(out[0]["latka"], "IBUPROFEN");
        assert_eq!(out[0]["mnozstvi"], "400");
        assert_eq!(out[0]["jednotka"], "MG");
        assert_eq!(out[1]["latka"], serde_json::Value::Null);
        assert_eq!(out[1]["kodLatky"], 999);
    }

    #[test]
    fn ceny_no_record_returns_null() {
        let raw = json!({ "kodChyby": 26, "popisChyby": "..." });
        let out = curate_ceny("0254297", &raw);
        assert_eq!(out["maZaznam"], false);
        assert_eq!(out["ceny"], serde_json::Value::Null);
        assert!(out["poznamka"].as_str().unwrap().contains("nehrazený"));
    }

    #[test]
    fn ceny_extracts_price_and_reimbursement() {
        let raw = json!({
            "cenaPuvodce": 71.27, "maxCenaLekarna": 95.0, "dph": 10,
            "uhrady": [{ "uhrada": 109.08, "zapocitatelnyDoplatek": 0, "indikacniOmezeni": null }]
        });
        let out = curate_ceny("0243130", &raw);
        assert_eq!(out["maZaznam"], true);
        assert_eq!(out["cenaPuvodce"], 71.27);
        assert_eq!(out["uhrady"][0]["uhradaPojistovny"], 109.08);
        assert_eq!(out["uhrady"][0]["doplatekPacienta"], 0);
    }

    #[test]
    fn dokumenty_builds_lowercase_urls() {
        let raw = json!([{ "id": 1, "typ": "PIL", "nazev": "PI.pdf" }, { "id": 2, "typ": "SPC", "nazev": "SPC.pdf" }]);
        let out = dokumenty_list("0254297", "https://x/dlp/v1", &raw);
        assert_eq!(out["dokumenty"][0]["typ"], "PIL");
        assert_eq!(out["dokumenty"][0]["url"], "https://x/dlp/v1/dokumenty/0254297/pil");
        assert_eq!(out["dokumenty"][1]["url"], "https://x/dlp/v1/dokumenty/0254297/spc");
    }

    #[test]
    fn ceny_empty_or_null_body_is_no_record() {
        // 404 s prázdným/ne-JSON tělem → parse dá Null → graceful „bez záznamu", ne pád
        let out = curate_ceny("0254297", &serde_json::Value::Null);
        assert_eq!(out["ceny"], serde_json::Value::Null);
        assert!(out["poznamka"].is_string());
    }

    #[test]
    fn dokumenty_non_array_body_is_empty_list() {
        let out = dokumenty_list("0254297", "https://x/dlp/v1", &serde_json::Value::Null);
        assert!(out["dokumenty"].as_array().unwrap().is_empty());
    }
}
