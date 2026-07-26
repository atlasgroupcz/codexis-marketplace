//! The `cdx://` address prefixes.
//!
//! `schema.rs` builds its patterns from these. The two help blocks (`cli.rs`,
//! `get.rs`) stay hand-written text - they carry alignment and per-line notes a
//! generator would lose - so a test in `cli.rs` asserts every prefix below
//! appears in both. Without it a route can be added to one surface and missed on
//! another, which is exactly what happened when `eu_law` arrived.

/// Any document, addressed by its CODEXIS id.
pub(crate) const DOC: &str = concat!("cdx://", "doc/<DOC_ID>");
/// A Czech act, by the number and year its citation carries.
pub(crate) const CZ_LAW: &str = concat!("cdx://", "cz_law/<NUM>/<YEAR>");
/// An EU act, by the type, year and number its citation carries.
pub(crate) const EU_LAW: &str = concat!("cdx://", "eu_law/<TYPE>/<YEAR>/<NUM>");
/// An EU act, by a CELEX already in hand.
pub(crate) const EU_LAW_BY_CELEX: &str = concat!("cdx://", "eu_law/by-celex/<CELEX>");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::http::CDX_SCHEME;

    #[test]
    fn every_prefix_uses_the_scheme_the_translator_strips() {
        // concat! cannot take a const, so the literal above is repeated - this ties
        // it to the one CDX_SCHEME that build_cdx_url actually strips.
        for prefix in [DOC, CZ_LAW, EU_LAW, EU_LAW_BY_CELEX] {
            assert!(prefix.starts_with(CDX_SCHEME), "{prefix} does not start with {CDX_SCHEME}");
        }
    }
}
