# Léčiva v1.1 — čitelný výstup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `detail`/`slozeni` vracejí čitelné názvy místo kódů, `detail` je kurátorovaný (bez interního šumu), nový `latka` hledá léky podle účinné látky — vše lokálně z cache, bez API volání navíc.

**Architecture:** `refresh` navíc vytáhne z DLP dumpu číselníky (kód→název) do `cis/*.tsv` a postaví reverzní index látka→léky (`latka-drugs.tsv`). Nový modul `codelist` staví i načítá tyhle tabulky. `detail`/`slozeni` joinují živou odpověď API na lokální číselníky; `latka` jede čistě z cache.

**Tech Stack:** Rust 2021 (navazuje na hotové leciva-cli v1.0.0), `std::collections::HashMap`, existující `zip`/`encoding_rs`/`serde_json`. Žádné nové dependency.

## Global Constraints

- Navazuje na hotový `plugins/leciva/src/leciva-cli/` (moduly: paths, decode, index, manifest, http, dataset, cli). Branch `lm-117-leciva`.
- Výstup vždy JSON na stdout; chyby `ERROR: …` na stderr + exit 2.
- Vše lokálně z měsíčního dumpu — **žádné nové API volání**. `slozeni` používá živé `/slozeni/{kod}` (jako dosud) + lokální join na názvy.
- Číselníky v dumpu: oddělovač `;`, kódování cp1250, join podle prvního sloupce. Parsovat podle **názvu sloupce** (hlavička).
- Data v `~/.cdx/apps/leciva/`: nově `cis/*.tsv` + `latka-drugs.tsv`. Atomicky (temp → rename), manifest až nakonec.
- Graceful: kód, co v číselníku není → `null` + syrový kód zachovat. Chybějící `cis/` → resolvuje na kódy, nespadne.
- Verze po dokončení: `1.1.0` (Cargo.toml, plugin.json, SKILL.md). Rebuild binárky přes `src/build.sh`.

## File Structure

```
plugins/leciva/src/leciva-cli/src/
  codelist.rs     # NOVÝ — build cis tsv + latka-drugs + Codelists load/resolve
  dataset.rs      # MOD — refresh navíc staví cis/*.tsv + latka-drugs.tsv
  paths.rs        # MOD — cesty cis_dir/cis_path/latka_drugs_path
  cli.rs          # MOD — detail kurátor + --raw, slozeni názvy, nový latka
  main.rs         # MOD — mod codelist
plugins/leciva/skills/leciva/SKILL.md   # MOD — latka, detail --raw, prezentace
plugins/leciva/.claude-plugin/plugin.json  # MOD — verze 1.1.0
.claude-plugin/marketplace.json            # beze změny (popis stačí)
```

---

## Task 1: Cesty pro číselníky (`paths.rs`)

**Files:**
- Modify: `plugins/leciva/src/leciva-cli/src/paths.rs`

**Interfaces:**
- Produces: `paths::cis_dir() -> PathBuf`, `paths::cis_path(name: &str) -> PathBuf`, `paths::latka_drugs_path() -> PathBuf`.

- [ ] **Step 1: Napiš failing test (přidej do `mod tests` v paths.rs)**

```rust
    #[test]
    fn cis_paths_under_app_dir() {
        std::env::set_var("CODEXIS_PUBLIC_USER_HOME", "/tmp/fh2");
        assert_eq!(cis_dir(), PathBuf::from("/tmp/fh2/.cdx/apps/leciva/cis"));
        assert_eq!(cis_path("latky.tsv"), PathBuf::from("/tmp/fh2/.cdx/apps/leciva/cis/latky.tsv"));
        assert_eq!(latka_drugs_path(), PathBuf::from("/tmp/fh2/.cdx/apps/leciva/latka-drugs.tsv"));
        std::env::remove_var("CODEXIS_PUBLIC_USER_HOME");
    }
```

- [ ] **Step 2: Spusť test, ověř FAIL**

Run: `cd plugins/leciva/src/leciva-cli && cargo test cis_paths -- --nocapture`
Expected: FAIL (`cis_dir` not found).

- [ ] **Step 3: Implementuj (přidej do paths.rs nad `#[cfg(test)]`)**

```rust
pub fn cis_dir() -> PathBuf {
    app_dir().join("cis")
}

pub fn cis_path(name: &str) -> PathBuf {
    cis_dir().join(name)
}

pub fn latka_drugs_path() -> PathBuf {
    app_dir().join("latka-drugs.tsv")
}
```

- [ ] **Step 4: Spusť test, ověř PASS**

Run: `cargo test cis_paths -- --nocapture`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add plugins/leciva/src/leciva-cli/src/paths.rs
git commit -m "feat(leciva): codelist cache paths"
```

---

## Task 2: Stavba číselníků z CSV (`codelist.rs` — build část)

**Files:**
- Create: `plugins/leciva/src/leciva-cli/src/codelist.rs`
- Modify: `plugins/leciva/src/leciva-cli/src/main.rs`

**Interfaces:**
- Produces:
  - `codelist::build_codelist_tsv(csv: &str, key_header: &str, val_header: &str) -> anyhow::Result<String>`
  - `codelist::build_latky_tsv(csv: &str) -> anyhow::Result<String>` (NAZEV, fallback NAZEV_INN)
  - `codelist::build_latka_drugs_tsv(slozeni_csv: &str) -> anyhow::Result<String>` (KOD_LATKY \t KOD_SUKL po řádcích)

- [ ] **Step 1: Přidej `mod codelist;` do main.rs**

V `main.rs` mezi ostatní `mod` přidej řádek (abecedně po `mod cli;`):
```rust
mod codelist;
```

- [ ] **Step 2: Napiš failing testy (vytvoř codelist.rs)**

```rust
use anyhow::{anyhow, Result};

fn col_index(header: &[&str], name: &str) -> Result<usize> {
    header
        .iter()
        .position(|h| h.trim() == name)
        .ok_or_else(|| anyhow!("CSV missing column {name}"))
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
    fn latky_uses_nazev_with_inn_fallback() {
        let csv = "KOD_LATKY;ZDROJ;NAZEV_INN;NAZEV_EN;NAZEV;ZAV;DOP;NARVLA\n\
1064;;PARACETAMOLUM;;PARACETAMOL;;;\n\
9999;;ONLYINN;;;;;\n";
        let tsv = build_latky_tsv(csv).unwrap();
        assert!(tsv.contains("1064\tPARACETAMOL\n"));
        assert!(tsv.contains("9999\tONLYINN\n")); // prázdné NAZEV → fallback INN
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
}
```

- [ ] **Step 3: Spusť testy, ověř FAIL**

Run: `cargo test codelist -- --nocapture`
Expected: FAIL (funkce nedefinované).

- [ ] **Step 4: Implementuj (do codelist.rs nad `#[cfg(test)]`)**

```rust
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
        out.push_str(latka);
        out.push('\t');
        out.push_str(kod);
        out.push('\n');
    }
    Ok(out)
}
```

- [ ] **Step 5: Spusť testy, ověř PASS**

Run: `cargo test codelist -- --nocapture`
Expected: PASS (3 testy).

- [ ] **Step 6: Commit**

```bash
git add plugins/leciva/src/leciva-cli/src/codelist.rs plugins/leciva/src/leciva-cli/src/main.rs
git commit -m "feat(leciva): build codelist + substance-to-drug tsv from DLP dump"
```

---

## Task 3: Načítání číselníků (`codelist.rs` — load část)

**Files:**
- Modify: `plugins/leciva/src/leciva-cli/src/codelist.rs`

**Interfaces:**
- Consumes: `paths::cis_path`.
- Produces:
  - `codelist::load_map(path: &std::path::Path) -> std::collections::HashMap<String, String>` (chybějící soubor → prázdná mapa)
  - `codelist::Codelists { latky, formy, cesty, vydej, stavyreg, organizace, atc, zavislost }` (vše `HashMap<String,String>`)
  - `codelist::Codelists::load() -> Codelists`
  - `Codelists::name(map_field, kod) -> Option<String>` — viz použití přímo přes pole

- [ ] **Step 1: Napiš failing test**

```rust
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
```

- [ ] **Step 2: Spusť test, ověř FAIL**

Run: `cargo test load_map -- --nocapture`
Expected: FAIL.

- [ ] **Step 3: Implementuj (do codelist.rs)**

```rust
use std::collections::HashMap;
use std::path::Path;

use crate::paths;

pub fn load_map(path: &Path) -> HashMap<String, String> {
    let data = match std::fs::read_to_string(path) {
        Ok(d) => d,
        Err(_) => return HashMap::new(),
    };
    data.lines()
        .filter_map(|l| {
            let mut p = l.splitn(2, '\t');
            let k = p.next()?.to_string();
            let v = p.next().unwrap_or("").to_string();
            if k.is_empty() {
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
    pub organizace: HashMap<String, String>,
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
            organizace: load_map(&paths::cis_path("organizace.tsv")),
            atc: load_map(&paths::cis_path("atc.tsv")),
            zavislost: load_map(&paths::cis_path("zavislost.tsv")),
        }
    }
}
```

- [ ] **Step 4: Spusť test, ověř PASS**

Run: `cargo test load_map -- --nocapture`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add plugins/leciva/src/leciva-cli/src/codelist.rs
git commit -m "feat(leciva): load codelists into lookup maps"
```

---

## Task 4: `refresh` staví číselníky + reverzní index (`dataset.rs`)

**Files:**
- Modify: `plugins/leciva/src/leciva-cli/src/dataset.rs`

**Interfaces:**
- Consumes: `codelist::{build_codelist_tsv, build_latky_tsv, build_latka_drugs_tsv}`, `decode::cp1250_to_utf8`, `paths`.
- Produces: rozšířený `dataset::refresh` (beze změny signatury) — vedle indexu zapíše `cis/*.tsv` + `latka-drugs.tsv`.

- [ ] **Step 1: Přidej helper `extract_csv` + test pro výběr souboru ze zipu**

V `dataset.rs` zobecni extrakci. Nahraď `extract_lecivepripravky_csv` voláním nového obecného helperu a přidej ho:

```rust
pub fn extract_csv(zip_bytes: &[u8], name: &str) -> Result<Vec<u8>> {
    let mut archive = zip::ZipArchive::new(Cursor::new(zip_bytes))
        .map_err(|e| anyhow!("cannot open DLP zip: {e}"))?;
    let mut file = archive
        .by_name(name)
        .map_err(|e| anyhow!("{name} not in DLP zip: {e}"))?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}

pub fn extract_lecivepripravky_csv(zip_bytes: &[u8]) -> Result<Vec<u8>> {
    extract_csv(zip_bytes, CSV_NAME)
}
```

(Test extrakce už pokrývá Task 6 z v1.0 nepřímo; nová unit-test pro `extract_csv` netřeba — kryje ji smoke test refreshe v kroku 4.)

- [ ] **Step 2: Přidej stavbu číselníků do `refresh` (za zápis index.tsv, před manifest)**

V `dataset::refresh`, hned po `std::fs::rename(&tmp, paths::index_path())?;` a před vytvořením manifestu, vlož:

```rust
    // Číselníky (kód → název) + reverzní index látka→léky. Neblokující: chyba
    // jednoho číselníku nezruší celý refresh (detail prostě nepřeloží ten kód).
    build_codelists(&zip_bytes)?;
```

A přidej funkci do `dataset.rs`:

```rust
use crate::codelist;

fn write_atomic(path: std::path::PathBuf, contents: String) -> Result<()> {
    let tmp = path.with_extension("tmp");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&tmp, contents)?;
    std::fs::rename(&tmp, &path)?;
    Ok(())
}

fn build_one(zip: &[u8], csv: &str, out: std::path::PathBuf, key: &str, val: &str) {
    let tsv = match extract_csv(zip, csv) {
        Ok(bytes) => match codelist::build_codelist_tsv(&cp1250_to_utf8(&bytes), key, val) {
            Ok(t) => t,
            Err(e) => { eprintln!("WARN: codelist {csv}: {e}"); return; }
        },
        Err(e) => { eprintln!("WARN: codelist {csv}: {e}"); return; }
    };
    if let Err(e) = write_atomic(out, tsv) {
        eprintln!("WARN: write codelist {csv}: {e}");
    }
}

fn build_codelists(zip: &[u8]) -> Result<()> {
    std::fs::create_dir_all(paths::cis_dir())?;
    // látky mají fallback NAZEV→INN, zvlášť
    if let Ok(bytes) = extract_csv(zip, "dlp_latky.csv") {
        if let Ok(tsv) = codelist::build_latky_tsv(&cp1250_to_utf8(&bytes)) {
            let _ = write_atomic(paths::cis_path("latky.tsv"), tsv);
        }
    }
    build_one(zip, "dlp_formy.csv", paths::cis_path("formy.tsv"), "FORMA", "NAZEV");
    build_one(zip, "dlp_cesty.csv", paths::cis_path("cesty.tsv"), "CESTA", "NAZEV");
    build_one(zip, "dlp_vydej.csv", paths::cis_path("vydej.tsv"), "VYDEJ", "NAZEV");
    build_one(zip, "dlp_stavyreg.csv", paths::cis_path("stavyreg.tsv"), "REG", "NAZEV");
    build_one(zip, "dlp_organizace.csv", paths::cis_path("organizace.tsv"), "ZKR_ORG", "NAZEV");
    build_one(zip, "dlp_atc.csv", paths::cis_path("atc.tsv"), "ATC", "NAZEV");
    build_one(zip, "dlp_zavislost.csv", paths::cis_path("zavislost.tsv"), "ZAV", "NAZEV");
    // reverzní index látka → léky
    if let Ok(bytes) = extract_csv(zip, "dlp_slozeni.csv") {
        if let Ok(tsv) = codelist::build_latka_drugs_tsv(&cp1250_to_utf8(&bytes)) {
            let _ = write_atomic(paths::latka_drugs_path(), tsv);
        }
    }
    Ok(())
}
```

Pozn.: `zip_bytes` v `refresh` je dostupné (stažený zip). Pokud bylo dříve přesunuto, drž ho v proměnné až do téhle fáze.

- [ ] **Step 3: Ověř build**

Run: `cargo build`
Expected: OK.

- [ ] **Step 4: Smoke test refreshe naživo (stáhne dump, postaví číselníky)**

Run:
```bash
target/debug/leciva-cli refresh --force >/dev/null 2>&1
ls ~/.cdx/apps/leciva/cis/
head -2 ~/.cdx/apps/leciva/cis/latky.tsv
wc -l ~/.cdx/apps/leciva/latka-drugs.tsv
```
Expected: `cis/` obsahuje latky.tsv, formy.tsv, cesty.tsv, vydej.tsv, stavyreg.tsv, organizace.tsv, atc.tsv, zavislost.tsv; `latky.tsv` má řádky `kod\tNÁZEV`; `latka-drugs.tsv` má statisíce řádků.

- [ ] **Step 5: Commit**

```bash
git add plugins/leciva/src/leciva-cli/src/dataset.rs
git commit -m "feat(leciva): refresh builds codelists + substance-to-drug index"
```

---

## Task 5: `detail` kurátorovaný + `--raw` (`cli.rs`)

**Files:**
- Modify: `plugins/leciva/src/leciva-cli/src/cli.rs`

**Interfaces:**
- Consumes: `codelist::Codelists`, `http::SuklClient`.
- Produces: `cli::curate_detail(raw: &serde_json::Value, cl: &Codelists) -> serde_json::Value` (čistá funkce, testovatelná); změněný `Command::Detail` s `--raw`.

- [ ] **Step 1: Napiš failing test pro `curate_detail`**

Přidej do `cli.rs` (vznikne `#[cfg(test)] mod tests`):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::codelist::Codelists;
    use std::collections::HashMap;
    use serde_json::json;

    fn cl() -> Codelists {
        let mk = |pairs: &[(&str, &str)]| pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect::<HashMap<_, _>>();
        Codelists {
            latky: mk(&[("1593", "IBUPROFEN")]),
            formy: mk(&[("TBL FLM", "Potahovaná tableta")]),
            cesty: mk(&[("POR", "Perorální podání")]),
            vydej: mk(&[("F", "volně prodejné léčivé přípravky")]),
            stavyreg: mk(&[("R", "registrovaný léčivý přípravek")]),
            organizace: mk(&[("OHCCZ", "Sanofi")]),
            atc: mk(&[("M01AE01", "ibuprofen")]),
            zavislost: HashMap::new(),
        }
    }

    #[test]
    fn curate_resolves_and_drops_noise() {
        let raw = json!({
            "kodSUKL": "0254299", "nazev": "IBALGIN", "sila": "400MG", "doplnek": "400MG TBL FLM 24",
            "baleni": "24", "lekovaFormaKod": "TBL FLM", "cestaPodaniKod": "POR",
            "ATCkod": "M01AE01", "zpusobVydejeKod": "F", "stavRegistraceKod": "R",
            "drzitelKod": "OHCCZ", "registracniCislo": "29/154/88-C", "leciveLatky": [1593],
            "jeDodavka": true, "MRPcislo": "X", "braillovoPismo": "S"
        });
        let out = curate_detail(&raw, &cl());
        assert_eq!(out["forma"], "Potahovaná tableta");
        assert_eq!(out["vydej"], "volně prodejné léčivé přípravky");
        assert_eq!(out["atc"]["nazev"], "ibuprofen");
        assert_eq!(out["drzitel"], "Sanofi");
        assert_eq!(out["ucinneLatky"][0], "IBUPROFEN");
        assert!(out.get("jeDodavka").is_none());
        assert!(out.get("MRPcislo").is_none());
    }

    #[test]
    fn curate_keeps_raw_code_when_codelist_missing() {
        let raw = json!({ "kodSUKL": "1", "nazev": "X", "lekovaFormaKod": "ZZZ", "leciveLatky": [999] });
        let out = curate_detail(&raw, &cl());
        assert_eq!(out["forma"], serde_json::Value::Null);
        assert_eq!(out["formaKod"], "ZZZ");
        assert_eq!(out["ucinneLatkyKody"][0], 999);
    }
}
```

- [ ] **Step 2: Spusť testy, ověř FAIL**

Run: `cargo test curate -- --nocapture`
Expected: FAIL (`curate_detail` not found).

- [ ] **Step 3: Implementuj `curate_detail` + helpery (do cli.rs)**

```rust
use crate::codelist::Codelists;
use std::collections::HashMap;

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
    let cesta_kod = s(raw, "cestaPodaniKod");
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
        "drzitel": resolved(&cl.organizace, drz_kod),
        "registracniCislo": raw.get("registracniCislo").cloned().unwrap_or(Value::Null),
        "ucinneLatky": nazvy,
        "ucinneLatkyKody": kody,
    });
    // závislost jen když je vyplněná
    if zav_kod.is_some() {
        out["zavislost"] = resolved(&cl.zavislost, zav_kod);
    }
    out
}
```

- [ ] **Step 4: Spusť testy, ověř PASS**

Run: `cargo test curate -- --nocapture`
Expected: PASS (2 testy).

- [ ] **Step 5: Zapoj do `Command::Detail` (přidej `--raw`, použij curate)**

V `cli.rs` změň variantu Detail:
```rust
    /// Detail léku podle kódu SÚKL (živé API + lokální názvy).
    Detail {
        kod: String,
        #[arg(long)]
        raw: bool,
    },
```
A v `run()` nahraď větev `Command::Detail`:
```rust
        Command::Detail { kod, raw } => {
            let body = client.get(&format!("/lecive-pripravky/{kod}"))?;
            if raw {
                print_raw(&body);
            } else {
                let parsed: serde_json::Value = serde_json::from_str(&body)
                    .map_err(|e| anyhow!("cannot parse detail: {e}"))?;
                // API vrací pole s jedním prvkem
                let rec = parsed.as_array().and_then(|a| a.first()).unwrap_or(&parsed);
                let cl = crate::codelist::Codelists::load();
                print_json(&curate_detail(rec, &cl));
            }
        }
```

- [ ] **Step 6: Ověř build + testy + smoke**

Run:
```bash
cargo build && cargo test
target/debug/leciva-cli detail 0254297 2>/dev/null
```
Expected: build OK, všechny testy PASS; `detail` vrátí čistý objekt s `"forma":"Potahovaná tableta"`, `"ucinneLatky":["IBUPROFEN"]`, bez `jeDodavka`.

- [ ] **Step 7: Commit**

```bash
git add plugins/leciva/src/leciva-cli/src/cli.rs
git commit -m "feat(leciva): curated detail with resolved names + --raw flag"
```

---

## Task 6: `slozeni` s názvy + nový `latka` (`cli.rs`)

**Files:**
- Modify: `plugins/leciva/src/leciva-cli/src/cli.rs`

**Interfaces:**
- Consumes: `codelist::{Codelists, load_map}`, `paths`, `index`.
- Produces: `cli::resolve_slozeni(raw: &serde_json::Value, cl: &Codelists) -> serde_json::Value`; nová varianta `Command::Latka`.

- [ ] **Step 1: Napiš failing test pro `resolve_slozeni`**

Přidej do `cli.rs` testů:
```rust
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
        assert_eq!(out[1]["latka"], serde_json::Value::Null); // 999 není v číselníku
        assert_eq!(out[1]["kodLatky"], 999);
    }
```

- [ ] **Step 2: Spusť test, ověř FAIL**

Run: `cargo test slozeni_adds -- --nocapture`
Expected: FAIL.

- [ ] **Step 3: Implementuj `resolve_slozeni`**

```rust
pub fn resolve_slozeni(raw: &serde_json::Value, cl: &Codelists) -> serde_json::Value {
    use serde_json::{json, Value};
    let items = match raw.as_array() {
        Some(a) => a,
        None => return json!([]),
    };
    // dedup: API vrací duplicitní řádky složení
    let mut seen = std::collections::HashSet::new();
    let mut out: Vec<Value> = Vec::new();
    for it in items {
        let kod = it.get("kodLatky").and_then(|v| v.as_i64());
        let mnozstvi = it.get("mnozstvi").and_then(|v| v.as_str()).unwrap_or("");
        let jednotka = it.get("jednotkaKod").and_then(|v| v.as_str()).unwrap_or("");
        let key = format!("{:?}|{}|{}", kod, mnozstvi, jednotka);
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
```

- [ ] **Step 4: Spusť test, ověř PASS**

Run: `cargo test slozeni_adds -- --nocapture`
Expected: PASS.

- [ ] **Step 5: Zapoj `slozeni` + přidej `latka` příkaz**

V `cli.rs` změň větev `Command::Slozeni`:
```rust
        Command::Slozeni { kod } => {
            let body = client.get(&format!("/slozeni/{kod}"))?;
            let parsed: serde_json::Value = serde_json::from_str(&body)
                .map_err(|e| anyhow!("cannot parse slozeni: {e}"))?;
            let cl = crate::codelist::Codelists::load();
            print_json(&resolve_slozeni(&parsed, &cl));
        }
```
Přidej variantu do `enum Command`:
```rust
    /// Hledání léků podle účinné látky (lokální index).
    Latka {
        query: String,
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
```
A větev v `run()`:
```rust
        Command::Latka { query, limit } => {
            let cl = crate::codelist::Codelists::load();
            let q = query.to_lowercase();
            // látky, jejichž název obsahuje dotaz
            let latka_kody: std::collections::HashSet<String> = cl
                .latky
                .iter()
                .filter(|(_, name)| name.to_lowercase().contains(&q))
                .map(|(k, _)| k.clone())
                .collect();
            if latka_kody.is_empty() {
                print_json(&json!({ "query": query, "count": 0, "results": [] }));
                return Ok(());
            }
            // reverzní index látka → léky
            let ld = crate::codelist::load_map(&paths::latka_drugs_path());
            // load_map drží jen poslední hodnotu na klíč → pro multi-mapping čteme soubor přímo
            let mut drug_kody: std::collections::HashSet<String> = std::collections::HashSet::new();
            if let Ok(text) = std::fs::read_to_string(paths::latka_drugs_path()) {
                for line in text.lines() {
                    let mut p = line.splitn(2, '\t');
                    let (lk, dk) = (p.next().unwrap_or(""), p.next().unwrap_or(""));
                    if latka_kody.contains(lk) && !dk.is_empty() {
                        drug_kody.insert(dk.to_string());
                    }
                }
            }
            let _ = ld; // load_map ponecháno pro čitelnost; hlavní průchod je řádkový výše
            // názvy léků z indexu
            let idx = load_index()?;
            let by_kod: std::collections::HashMap<&str, &index::Entry> =
                idx.entries.iter().map(|e| (e.kod.as_str(), e)).collect();
            let mut results: Vec<serde_json::Value> = drug_kody
                .iter()
                .filter_map(|k| by_kod.get(k.as_str()))
                .take(limit)
                .map(|e| json!({ "kodSukl": e.kod, "nazev": e.nazev, "sila": e.sila }))
                .collect();
            results.sort_by(|a, b| a["kodSukl"].as_str().cmp(&b["kodSukl"].as_str()));
            print_json(&json!({ "query": query, "count": drug_kody.len(), "results": results }));
        }
```

Pozn.: `latka` potřebuje index i číselníky → na začátku větve zavolej `ensure_index(&client)?;` před `load_index()`. Přidej ho hned za `let cl = ...`. Pokud `latka-drugs.tsv` chybí, výsledek bude prázdný — to je OK (lazy refresh ho příště dostaví).

- [ ] **Step 6: Ověř build + testy + smoke**

Run:
```bash
cargo build && cargo test
target/debug/leciva-cli slozeni 0254297 2>/dev/null
target/debug/leciva-cli latka "ibuprofen" 2>/dev/null | head -c 300
```
Expected: build OK, testy PASS; `slozeni` vrací `[{"latka":"IBUPROFEN",...}]`; `latka ibuprofen` vrací léky s ibuprofenem.

- [ ] **Step 7: Commit**

```bash
git add plugins/leciva/src/leciva-cli/src/cli.rs
git commit -m "feat(leciva): slozeni with substance names + latka search by active substance"
```

---

## Task 7: SKILL.md, verze 1.1.0, rebuild, packaging

**Files:**
- Modify: `plugins/leciva/skills/leciva/SKILL.md`
- Modify: `plugins/leciva/.claude-plugin/plugin.json`
- Modify: `plugins/leciva/src/leciva-cli/Cargo.toml`
- Modify: `plugins/leciva/bin/leciva-cli` (rebuild)

- [ ] **Step 1: SKILL.md — přidej `latka`, `detail --raw`, prezentační větu**

V sekci `## Příkazy` přidej řádky:
```
leciva-cli latka "ibuprofen"       # léky obsahující danou účinnou látku
leciva-cli detail 0182362 --raw    # plná nefiltrovaná odpověď API
```
Za sekci `## Postup` přidej:
```markdown
## Výstup

`detail` a `slozeni` vracejí čistý JSON s názvy (forma, cesta, výdej, držitel,
účinné látky). Podej ho uživateli čitelně — lék, síla, forma, výdej (na předpis /
volně prodejné), účinné látky — **nedumpuj syrový JSON**. Plný záznam je přes
`detail <kod> --raw`.
```

- [ ] **Step 2: Verze na 1.1.0**

V `plugins/leciva/.claude-plugin/plugin.json` změň `"version": "1.0.0"` → `"version": "1.1.0"`.
V `plugins/leciva/skills/leciva/SKILL.md` frontmatter změň `version: 1.0.0` → `version: 1.1.0`.
V `plugins/leciva/src/leciva-cli/Cargo.toml` změň `version = "1.0.0"` → `version = "1.1.0"`.

- [ ] **Step 3: Sync Cargo.lock + rebuild binárky**

Run:
```bash
(cd plugins/leciva/src/leciva-cli && cargo build -q)
bash plugins/leciva/src/build.sh
plugins/leciva/bin/leciva-cli --help | head -1
```
Expected: build projde, `Completed plugin build: leciva`.

- [ ] **Step 4: Validace + plný test**

Run:
```bash
python3 -c "import json,yaml;json.load(open('plugins/leciva/.claude-plugin/plugin.json'));yaml.safe_load(open('plugins/leciva/skills/leciva/SKILL.md').read().split('---',2)[1]);print('OK')"
(cd plugins/leciva/src/leciva-cli && cargo test 2>&1 | grep 'test result')
```
Expected: `OK`; všechny testy PASS.

- [ ] **Step 5: Commit**

```bash
git add plugins/leciva
git commit -m "feat(leciva): v1.1.0 — readable output, latka search, SKILL docs"
```

---

## Self-Review

**Spec coverage:**
- §4 číselníky → Task 2 (build) + Task 3 (load). ✓
- §5 refresh staví cis/*.tsv + latka-drugs → Task 4. ✓
- §6 detail kurátor + resolved + --raw → Task 5. ✓ (drop noise, keep raw codes, atc{kod,nazev})
- §7 slozeni názvy → Task 6. ✓
- §8 latka → Task 6. ✓
- §9 SKILL.md → Task 7. ✓
- §10 edge-case: kód není v číselníku → `null` + kód (Task 5 test `curate_keeps_raw_code_when_codelist_missing`, Task 6 slozeni test 999); chybějící cis/ → load_map prázdná mapa (Task 3 test), graceful. ✓

**Placeholder scan:** Žádné TBD. `ensure_index` v latka větvi je explicitně popsán (krok 5 pozn.). Hodnoty číselníků (F/R/TBL FLM) ověřeny z dumpu, ne vymyšlené.

**Type consistency:** `Codelists` pole (latky/formy/cesty/vydej/stavyreg/organizace/atc/zavislost) konzistentní Task 3 → 5 → 6. `curate_detail`/`resolve_slozeni`/`load_map`/`build_*` signatury sedí napříč tasky. `index::Entry` (kod/nazev/sila) z v1.0 použit v Task 6.

**Pozn. k `load_map` v latka (Task 6):** `load_map` drží 1 hodnotu na klíč, ale reverzní index je many-to-one obráceně (látka→více léků), proto se `latka-drugs.tsv` čte **řádkově** přímo (ne přes `load_map`). To je v kódu ošetřené; `let _ = ld;` jen drží import bez varování — při implementaci klidně vynech `load_map` volání a čti jen soubor.

---

## Execution Handoff
