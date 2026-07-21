//! The data model plus the fetch → parse orchestration over `http` + `parse`.
//!
//! A subject's Sbírka listin is reached in two GETs: the search page resolves an
//! IČO to the internal `subjektId`, then the listiny page carries the documents.
//! Downloading one adds two more — the detail page holds the only copy of the
//! download link, so it can't be skipped.

use anyhow::{anyhow, bail, Result};
use serde::Serialize;

use crate::http::{Document, PortalClient};
use crate::parse;

/// The public host for citations. The env var may point the fetch at a test
/// server, but a document's URL always names the official portal.
const OFFICIAL_HOST: &str = "https://or.justice.cz";

/// A subject and its filed documents — the shape `listiny` emits and the watcher
/// diffs.
#[derive(Debug, Clone, Serialize)]
pub struct Detail {
    pub ico: String,
    pub nazev: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spisovaznacka: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sidlo: Option<String>,
    /// The internal register id — needed to build download links, not meaningful
    /// to the user, so it stays out of the emitted JSON.
    #[serde(skip)]
    pub subjekt_id: String,
    pub pocet_listin: usize,
    pub listiny: Vec<Listina>,
}

/// One filed document. The full číslo listiny is its only identifier — the
/// register's "Číslo listiny", unique and citable on its own.
#[derive(Debug, Clone, Serialize)]
pub struct Listina {
    /// The full číslo listiny, e.g. `B 8573/SL95/MSPH`.
    pub cislo: String,
    pub typ: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vznik: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doslo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zalozeno: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stran: Option<u32>,
    /// Download coordinates for the detail page — internal, so not serialized.
    #[serde(skip)]
    pub dokument: String,
    #[serde(skip)]
    pub spis: String,
}

impl Listina {
    /// Whether a číslo listiny names this document (case/space-insensitive).
    pub fn matches(&self, cislo: &str) -> bool {
        parse::normalize_key(cislo) == parse::normalize_key(&self.cislo)
    }
}

/// Resolve an IČO to its subject and fetch the listiny.
///
/// The search is exact on IČO, so it returns 0 or 1 subject — but if it ever
/// returns more, the block whose IČO matches is taken rather than the first, and
/// an ambiguous match errors rather than guessing.
pub fn fetch_detail(client: &PortalClient, ico8: &str) -> Result<Detail> {
    let search = client.get_html(&format!("/ias/ui/rejstrik-$firma?ico={ico8}"))?;
    let hit = pick_subject(&search, ico8)?;

    let listiny_html = client.get_html(&format!(
        "/ias/ui/vypis-sl-firma?subjektId={}",
        hit.subjekt_id
    ))?;
    let rows = parse::listiny(&listiny_html);
    // The listiny page repeats the identity; prefer the search hit but fall back
    // to it for anything the search omitted.
    let hlavicka = parse::subjekt_hlavicka(&listiny_html);

    let listiny: Vec<Listina> = rows
        .into_iter()
        .map(|r| Listina {
            cislo: r.cislo,
            typ: r.typ,
            vznik: r.vznik,
            doslo: r.doslo,
            zalozeno: r.zalozeno,
            stran: r.stran,
            dokument: r.dokument,
            spis: r.spis,
        })
        .collect();

    Ok(Detail {
        ico: ico8.to_string(),
        nazev: pick_nazev(&hit, &hlavicka),
        spisovaznacka: hit.spisovaznacka.or(hlavicka.spisovaznacka),
        sidlo: hit.sidlo.or(hlavicka.sidlo),
        subjekt_id: hit.subjekt_id,
        pocet_listin: listiny.len(),
        listiny,
    })
}

/// A downloaded document: which listina it is, its bytes, and its official URL.
pub struct FetchedDocument {
    pub listina: Listina,
    pub document: Document,
    pub url: String,
}

/// Download one document, named by IČO + a listina reference. Resolves the
/// subject, finds the row, follows to the detail page for the download link,
/// then fetches the bytes.
pub fn fetch_document(
    client: &PortalClient,
    ico8: &str,
    reference: &str,
) -> Result<FetchedDocument> {
    let detail = fetch_detail(client, ico8)?;
    let matches: Vec<&Listina> = detail
        .listiny
        .iter()
        .filter(|l| l.matches(reference))
        .collect();
    let listina = match matches.as_slice() {
        [one] => (*one).clone(),
        [] => bail!("listina {reference} u IČO {ico8} nenalezena"),
        _ => bail!("odkaz {reference} je u IČO {ico8} nejednoznačný — použij celé číslo listiny"),
    };

    let detail_html = client.get_html(&format!(
        "/ias/ui/vypis-sl-detail?dokument={}&subjektId={}&spis={}",
        listina.dokument, detail.subjekt_id, listina.spis
    ))?;
    let download_id = parse::download_id(&detail_html)
        .ok_or_else(|| anyhow!("listina {} nemá digitální podobu ke stažení", listina.cislo))?;

    let document = client.get_bytes(&format!("/ias/content/download?id={download_id}"))?;
    Ok(FetchedDocument {
        listina,
        document,
        url: format!("{OFFICIAL_HOST}/ias/content/download?id={download_id}"),
    })
}

/// Choose the subject for an exact-IČO search, honouring the count and matching
/// by IČO when more than one comes back.
fn pick_subject(search_html: &str, ico8: &str) -> Result<parse::SearchHit> {
    let hits = parse::search_hits(search_html);
    if parse::search_count(search_html) == Some(0) || hits.is_empty() {
        bail!("IČO {ico8} nenalezeno");
    }
    if let [only] = hits.as_slice() {
        return Ok(only.clone());
    }
    let mut exact = hits.into_iter().filter(|h| h.ico == ico8);
    match (exact.next(), exact.next()) {
        (Some(h), None) => Ok(h),
        (None, _) => bail!("IČO {ico8} nenalezeno"),
        (Some(_), Some(_)) => bail!("IČO {ico8} vrátilo více subjektů — nejednoznačné"),
    }
}

fn pick_nazev(hit: &parse::SearchHit, hlavicka: &parse::SubjektHlavicka) -> String {
    if !hit.nazev.is_empty() {
        hit.nazev.clone()
    } else {
        hlavicka.nazev.clone().unwrap_or_default()
    }
}
