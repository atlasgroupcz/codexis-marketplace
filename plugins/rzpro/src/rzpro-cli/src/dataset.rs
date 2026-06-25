use std::collections::BTreeMap;
use std::io::{Cursor, Read};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context, Result};

use crate::http::RzproClient;
use crate::manifest::{self, Manifest};
use crate::paths;

/// Datasets pulled for v1. Order is download order; all are required.
pub const DATASETS: [&str; 4] = ["RZPRO_ZP", "RZPRO_ZP_VAR", "RZPRO_ZP_OS", "RZPRO_OS_CIN"];

pub enum RefreshOutcome {
    UpToDate { datum: String },
    Updated { datum: String, counts: BTreeMap<String, usize> },
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Strip HTML tags, collapsing each tag to a single space.
fn strip_tags(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out
}

/// Find "Datum aktualizace DD.MM.YYYY" on the page and return it as YYYY-MM-DD.
pub fn parse_datum(html: &str) -> Option<String> {
    let text = strip_tags(html);
    let marker = "Datum aktualizace";
    let start = text.find(marker)? + marker.len();
    let tail = &text[start..];
    let (d, m, y) = first_date(tail)?;
    Some(format!("{y:04}-{m:02}-{d:02}"))
}

/// First DD.MM.YYYY occurring in `s`, returned as (day, month, year).
fn first_date(s: &str) -> Option<(u32, u32, u32)> {
    let bytes: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            if let Some((parts, _next)) = try_date_at(&bytes, i) {
                return Some(parts);
            } else {
                // skip this run of digits
                let mut j = i;
                while j < bytes.len() && bytes[j].is_ascii_digit() {
                    j += 1;
                }
                i = j;
                continue;
            }
        }
        i += 1;
    }
    None
}

fn try_date_at(b: &[char], start: usize) -> Option<((u32, u32, u32), usize)> {
    let mut i = start;
    let day = take_num(b, &mut i, 1, 2)?;
    if i >= b.len() || b[i] != '.' {
        return None;
    }
    i += 1;
    let month = take_num(b, &mut i, 1, 2)?;
    if i >= b.len() || b[i] != '.' {
        return None;
    }
    i += 1;
    let year = take_num(b, &mut i, 4, 4)?;
    Some(((day, month, year), i))
}

fn take_num(b: &[char], i: &mut usize, min: usize, max: usize) -> Option<u32> {
    let start = *i;
    while *i < b.len() && *i - start < max && b[*i].is_ascii_digit() {
        *i += 1;
    }
    if *i - start < min {
        return None;
    }
    b[start..*i].iter().collect::<String>().parse().ok()
}

/// Extract the single CSV file from a dataset ZIP, returned as UTF-8 text.
fn extract_csv(zip_bytes: &[u8]) -> Result<String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(zip_bytes))
        .context("dataset is not a valid ZIP archive")?;
    if archive.is_empty() {
        return Err(anyhow!("dataset ZIP is empty"));
    }
    let mut file = archive.by_index(0)?;
    let mut buf = Vec::with_capacity(file.size() as usize);
    file.read_to_end(&mut buf)?;
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

fn count_rows(csv_text: &str) -> usize {
    // rows minus header; tolerate missing trailing newline
    csv_text.lines().count().saturating_sub(1)
}

/// Download all datasets if the remote publication date is newer than the local
/// manifest (or `force`). Returns whether anything was updated.
pub fn refresh(force: bool) -> Result<RefreshOutcome> {
    let client = RzproClient::new()?;
    let page = client.opendata_page()?;
    let datum = parse_datum(&page)
        .ok_or_else(|| anyhow!("could not find 'Datum aktualizace' on OpenData page"))?;

    let local = manifest::load();
    let up_to_date = local
        .as_ref()
        .map(|m| !m.is_older_than(&datum))
        .unwrap_or(false);
    let data_present = DATASETS.iter().all(|d| paths::csv_path(d).exists());

    if up_to_date && data_present && !force {
        return Ok(RefreshOutcome::UpToDate { datum });
    }

    std::fs::create_dir_all(paths::data_dir())?;
    let mut counts = BTreeMap::new();
    for ds in DATASETS {
        let zip = client
            .dataset_zip(ds)
            .with_context(|| format!("downloading {ds}"))?;
        let csv = extract_csv(&zip).with_context(|| format!("extracting {ds}"))?;
        counts.insert(ds.to_string(), count_rows(&csv));
        // write atomically: temp + rename
        let dst = paths::csv_path(ds);
        let tmp = dst.with_extension("csv.tmp");
        std::fs::write(&tmp, csv.as_bytes())?;
        std::fs::rename(&tmp, &dst)?;
    }

    Manifest {
        datum: datum.clone(),
        downloaded_at: now_unix(),
        counts: counts.clone(),
    }
    .save()?;

    Ok(RefreshOutcome::Updated { datum, counts })
}

/// Ensure local CSVs exist; lazily refresh on first use.
pub fn ensure_data() -> Result<()> {
    let present = DATASETS.iter().all(|d| paths::csv_path(d).exists());
    if !present {
        refresh(false)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_datum_from_page_fragment() {
        let html = "<span>Open data</span><b>Datum aktualizace</b> <i>24.6.2026</i> more";
        assert_eq!(parse_datum(html).as_deref(), Some("2026-06-24"));
    }

    #[test]
    fn parses_two_digit_day_month() {
        let html = "Datum aktualizace 03.11.2026";
        assert_eq!(parse_datum(html).as_deref(), Some("2026-11-03"));
    }

    #[test]
    fn none_when_marker_absent() {
        assert_eq!(parse_datum("no date here 1.2.3"), None);
    }
}
