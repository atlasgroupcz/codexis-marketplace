//! Guards the one thing in this plugin that depends on ČÚZK's wording.
//!
//! `PravaStavby` answers 404 twice over: once for "the parcel exists but has no
//! right of superficies" - the normal answer for most parcels - and once for
//! "no such parcel", which is an error. Status, shape and `title` are identical;
//! only the English sentence in `detail` differs, and there is no code to key
//! off. So `pravo_stavby::interpret_404` matches the substring
//! "without PravoStavby".
//!
//! That cannot be made robust in our code - the API gives nothing else. What it
//! can be is *watched*: the day ČÚZK rewords that sentence, every clean parcel
//! would start reporting as an error, and this test says so first.
//!
//! Ignored by default: it needs a live call and a configured API key, and it
//! spends one of the 500 calls per period. Run it deliberately:
//!
//! ```text
//! cargo test -p katastr --test pravo_stavby_marker -- --ignored
//! ```

use katastr_core::{api_client, lookup, pravo_stavby, KatastrError};

/// A parcel that exists and carries no right of superficies. If ČÚZK ever
/// registers one here the test starts failing for a legitimate reason - pick
/// another parcel then, do not weaken the assertion.
const PARCELA_BEZ_PRAVA: &str = "2034508804";

#[test]
#[ignore = "hits the live ČÚZK API and spends one call from the quota"]
fn cuzk_still_words_a_missing_right_the_way_we_detect_it() {
    let err = api_client::get(&lookup::pravo_stavby_parcela(PARCELA_BEZ_PRAVA))
        .expect_err("a parcel without the right must answer 404, not 200");

    let KatastrError::ApiHttp { status, message } = &err else {
        panic!("expected an HTTP error from ČÚZK, got: {err}");
    };
    assert_eq!(*status, 404, "message: {message}");

    assert!(
        matches!(
            pravo_stavby::interpret_404(message),
            pravo_stavby::Missing::NoRight
        ),
        "ČÚZK no longer words a missing right the way interpret_404 detects it, \
         so every parcel without one now reports as an error. Update the marker \
         in pravo_stavby.rs to match. Message was: {message}"
    );
}
