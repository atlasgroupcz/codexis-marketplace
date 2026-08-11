//! The reqwest counterpart to [`crate::http_client`]: one TLS policy, in one
//! place, for the two plugins that need an async-capable client.
//!
//! Those two need reqwest rather than ureq for `multipart`, whose `Form::file`
//! assembles the body and derives each part's media type - so they pay for a
//! tokio runtime they otherwise would not.
//!
//! What this module exists to settle is TLS. reqwest 0.13's `rustls` feature
//! pulls aws-lc-rs, a C library built with cmake that costs ~2 MB of binary, and
//! verifies against the system trust store, which it reads from disk when the
//! client is built. Neither suits a short-lived CLI or a per-request CGI. This
//! builds the same policy the ureq plugins get from ureq's own `rustls` feature:
//! the ring provider, and the Mozilla roots compiled into the binary.

use std::sync::Arc;

/// The Mozilla root set, pre-parsed at compile time.
///
/// `webpki-roots` hands back `TrustAnchor` values rather than DER, so this costs
/// one allocation instead of parsing 121 certificates on every client.
fn root_store() -> rustls::RootCertStore {
    rustls::RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
    }
}

/// A blocking client builder with this repo's TLS policy already applied.
///
/// Callers add their own timeouts, user agent and headers, then `.build()`.
/// Handing back the builder rather than a finished client is what lets the two
/// callers differ where they genuinely do.
pub fn client_builder() -> reqwest::blocking::ClientBuilder {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let tls = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .expect("ring supports rustls' default protocol versions")
        .with_root_certificates(root_store())
        .with_no_client_auth();
    reqwest::blocking::ClientBuilder::new().use_preconfigured_tls(tls)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_root_store_is_not_empty() {
        // An empty store builds fine and then fails every handshake, which is a
        // confusing way to find out the roots went missing.
        assert!(root_store().roots.len() > 100);
    }

    #[test]
    fn the_builder_produces_a_usable_client() {
        assert!(client_builder().build().is_ok());
    }
}
