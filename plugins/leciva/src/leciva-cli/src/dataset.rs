use std::io::{Cursor, Read};

use anyhow::{anyhow, Result};

use crate::codelist;
use crate::decode::cp1250_to_utf8;
use crate::http::{opendata_base, SuklClient};
use crate::index::build_tsv_from_csv;
use crate::manifest::{is_stale, Manifest};
use crate::paths;

const CSV_NAME: &str = "dlp_lecivepripravky.csv";
const MIN_ROWS: usize = 10_000;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshOutcome {
    pub updated: bool,
    pub batch: String,
    pub row_count: usize,
}

pub fn zip_url_from_datum(opendata_base: &str, datum_vydani: &str) -> Result<String> {
    // "2026-05-27T00:00:00" nebo "2026-05-27" → "20260527"
    let date = datum_vydani
        .get(0..10)
        .ok_or_else(|| anyhow!("bad datumVydani: {datum_vydani}"))?;
    let compact: String = date.chars().filter(|c| c.is_ascii_digit()).collect();
    if compact.len() != 8 {
        return Err(anyhow!("bad datumVydani: {datum_vydani}"));
    }
    Ok(format!(
        "{}/soubory/SOD{}/DLP{}.zip",
        opendata_base.trim_end_matches('/'),
        compact,
        compact
    ))
}

pub fn batch_label(platnost_od: &str) -> String {
    // "2026-06-01T00:00:00" → "2026.06"
    let date = platnost_od.get(0..7).unwrap_or(platnost_od); // "2026-06"
    date.replace('-', ".")
}

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

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

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
        let csv = cp1250_to_utf8(&bytes);
        if let Ok(tsv) = codelist::build_latky_tsv(&csv) {
            let _ = write_atomic(paths::cis_path("latky.tsv"), tsv);
        }
        // vyhledávací blob (cz + en + inn + synonyma) pro příkaz `latka`
        let synonyma = extract_csv(zip, "dlp_synonyma.csv")
            .ok()
            .and_then(|b| codelist::build_synonyma_map(&cp1250_to_utf8(&b)).ok())
            .unwrap_or_default();
        if let Ok(tsv) = codelist::build_latky_search_tsv(&csv, &synonyma) {
            let _ = write_atomic(paths::cis_path("latky-search.tsv"), tsv);
        }
    }
    build_one(zip, "dlp_formy.csv", paths::cis_path("formy.tsv"), "FORMA", "NAZEV");
    build_one(zip, "dlp_cesty.csv", paths::cis_path("cesty.tsv"), "CESTA", "NAZEV");
    build_one(zip, "dlp_vydej.csv", paths::cis_path("vydej.tsv"), "VYDEJ", "NAZEV");
    build_one(zip, "dlp_stavyreg.csv", paths::cis_path("stavyreg.tsv"), "REG", "NAZEV");
    build_one(zip, "dlp_atc.csv", paths::cis_path("atc.tsv"), "ATC", "NAZEV");
    build_one(zip, "dlp_zavislost.csv", paths::cis_path("zavislost.tsv"), "ZAV", "NAZEV_CS");
    // reverzní index látka → léky
    if let Ok(bytes) = extract_csv(zip, "dlp_slozeni.csv") {
        if let Ok(tsv) = codelist::build_latka_drugs_tsv(&cp1250_to_utf8(&bytes)) {
            let _ = write_atomic(paths::latka_drugs_path(), tsv);
        }
    }
    Ok(())
}

pub fn refresh(client: &SuklClient, force: bool) -> Result<RefreshOutcome> {
    let dlpo = client.aktualni_dlpo()?;
    let batch = batch_label(&dlpo.platnost_od);

    if !force && !is_stale(&dlpo.platnost_od) {
        return Ok(RefreshOutcome {
            updated: false,
            batch,
            row_count: 0,
        });
    }

    let url = zip_url_from_datum(&opendata_base(), &dlpo.datum_vydani)?;
    let zip_bytes = client.get_bytes(&url)?;
    let csv_bytes = extract_lecivepripravky_csv(&zip_bytes)?;
    let csv = cp1250_to_utf8(&csv_bytes);
    let (tsv, row_count) = build_tsv_from_csv(&csv)?;

    if row_count < MIN_ROWS {
        return Err(anyhow!("refused index: only {row_count} rows (< {MIN_ROWS})"));
    }

    let app_dir = paths::app_dir();
    std::fs::create_dir_all(&app_dir)?;
    // 1) zapiš index do temp, 2) atomicky přejmenuj, 3) až pak manifest
    let tmp = app_dir.join("index.tsv.tmp");
    std::fs::write(&tmp, tsv)?;
    std::fs::rename(&tmp, paths::index_path())?;

    // Číselníky (kód → název) + reverzní index látka→léky. Neblokující: chyba
    // číselníků nesmí shodit refresh ani zabránit zápisu manifestu níž
    // (jinak by nový index zůstal bez manifestu). Detail pak prostě nepřeloží kód.
    if let Err(e) = build_codelists(&zip_bytes) {
        eprintln!("WARN: codelists: {e}");
    }

    let manifest = Manifest {
        batch: batch.clone(),
        platnost_od: dlpo.platnost_od.clone(),
        downloaded_at: now_unix(),
        row_count,
    };
    manifest.save()?;

    Ok(RefreshOutcome {
        updated: true,
        batch,
        row_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_zip_url() {
        let url = zip_url_from_datum("https://opendata.sukl.cz", "2026-05-27T00:00:00").unwrap();
        assert_eq!(
            url,
            "https://opendata.sukl.cz/soubory/SOD20260527/DLP20260527.zip"
        );
    }

    #[test]
    fn zip_url_rejects_bad_date() {
        assert!(zip_url_from_datum("https://x", "nope").is_err());
    }

    #[test]
    fn batch_label_from_platnost() {
        assert_eq!(batch_label("2026-06-01T00:00:00"), "2026.06");
    }
}
