//! Telling two different 404s apart for a right of superficies.
//!
//! ČÚZK returns the same status and the same shape for "the parcel exists but
//! carries no right of superficies" and for "the parcel does not exist". Only
//! the free text in `detail` differs. Most parcels carry no such right, so the
//! first case is a normal state and must not end as an ERROR - otherwise the
//! "is it clean?" workflow would stop on every single task.
//!
//! The distinction rests on English free text, which is fragile. So it fails
//! to the safe side: anything unrecognised is an error. The other way round, a
//! reworded message would tell the client a plot is clean without us knowing
//! that.

/// The marker ČÚZK uses for "the object exists but carries no right of
/// superficies".
const WITHOUT_MARKER: &str = "without PravoStavby";

#[derive(Debug)]
pub enum Missing {
    /// The object exists, it just carries no right of superficies. Normal.
    NoRight,
    /// The object was not found, or the body was not recognised. An error.
    NotFound,
}

/// Takes either the raw body or the message `cuzk_error::human_message` built
/// from it - that one passes `detail` through verbatim, so the marker survives
/// in both. Hence a substring search rather than parsing: the caller does not
/// have to care which of the two it is holding.
pub fn interpret_404(text: &str) -> Missing {
    if text.contains(WITHOUT_MARKER) {
        Missing::NoRight
    } else {
        Missing::NotFound
    }
}

#[cfg(test)]
mod tests {
    use super::{interpret_404, Missing};

    const NO_RIGHT: &str = r#"{"title":"Not found","status":404,"detail":"Parcela [ID:2034508804] was found but without PravoStavby"}"#;
    const NOT_FOUND: &str =
        r#"{"title":"Not found","status":404,"detail":"Parcela [ID:1] was not found"}"#;
    const REWORDED: &str = r#"{"title":"Not found","status":404,"detail":"neco uplne jineho"}"#;

    #[test]
    fn existing_parcel_without_right_is_a_normal_state() {
        assert!(matches!(interpret_404(NO_RIGHT), Missing::NoRight));
    }

    #[test]
    fn missing_parcel_is_an_error() {
        assert!(matches!(interpret_404(NOT_FOUND), Missing::NotFound));
    }

    #[test]
    fn unrecognised_wording_fails_to_the_safe_side() {
        assert!(
            matches!(interpret_404(REWORDED), Missing::NotFound),
            "unknown detail must be treated as an error, never as 'no right'"
        );
    }

    #[test]
    fn html_body_fails_to_the_safe_side() {
        assert!(matches!(interpret_404("<html>"), Missing::NotFound));
    }
}
