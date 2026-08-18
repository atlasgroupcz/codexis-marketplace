//! The `cdx://` address prefixes.
//!
//! `schema.rs` builds its patterns from these. The two help blocks (`cli.rs`,
//! `get.rs`) stay hand-written text - they carry alignment and per-line notes a
//! generator would lose - so a test in `cli.rs` asserts every prefix below
//! appears in both. Without it a route can be added to one surface and missed on
//! another.
//!
//! Only prefixes the API serves belong here. Advertising one it does not turns
//! every call built from it into a 404 that reads as a broken tool.

/// Any document, addressed by its CODEXIS id.
pub(crate) const DOC: &str = concat!("cdx://", "doc/<DOC_ID>");
/// A Czech act, by the number and year its citation carries.
pub(crate) const CZ_LAW: &str = concat!("cdx://", "cz_law/<NUM>/<YEAR>");

/// Every prefix, for the code that has to treat them as a set.
pub(crate) const ALL: &[&str] = &[DOC, CZ_LAW];

/// The namespace each prefix claims, i.e. the segment after `cdx://`.
#[cfg(test)]
pub(crate) fn namespaces() -> impl Iterator<Item = &'static str> {
    ALL.iter().filter_map(|prefix| {
        prefix
            .strip_prefix(crate::core::http::CDX_SCHEME)?
            .split('/')
            .next()
    })
}

/// The namespace of every `cdx://` URL written in `text`, in order of appearance.
///
/// Lets a test read back what a help block or a schema document advertises and
/// hold it against the vocabulary above, rather than naming one route it must
/// not mention.
#[cfg(test)]
pub(crate) fn cdx_namespaces_in(text: &str) -> Vec<&str> {
    text.match_indices(crate::core::http::CDX_SCHEME)
        .filter_map(|(at, scheme)| {
            let rest = &text[at + scheme.len()..];
            let end = rest
                .find(|c: char| c == '/' || c.is_whitespace())
                .unwrap_or(rest.len());
            Some(&rest[..end]).filter(|segment| !segment.is_empty())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::http::CDX_SCHEME;

    #[test]
    fn every_prefix_uses_the_scheme_the_translator_strips() {
        // concat! cannot take a const, so the literal above is repeated - this ties
        // it to the one CDX_SCHEME that build_cdx_url actually strips.
        for prefix in ALL {
            assert!(
                prefix.starts_with(CDX_SCHEME),
                "{prefix} does not start with {CDX_SCHEME}"
            );
        }
    }
}
