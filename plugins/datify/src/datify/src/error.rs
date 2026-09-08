//! Error type for the Datify plugin.
//!
//! Variants exist per failure the user can actually act on - a missing token, a
//! rejected token, a permission gap - so both the CLI message and the CGI's HTTP
//! status follow from one classification.

use thiserror::Error;

pub type Result<T> = std::result::Result<T, DatifyError>;

#[derive(Debug, Error)]
pub enum DatifyError {
    /// Configuration is invalid (bad URL, empty token on write).
    #[error("{0}")]
    Config(String),

    /// No API token is configured at all.
    #[error("API token pro Datify není nastaven. Vygenerujte ho v Datify na {0} a uložte přes `datify-cli settings set-key <TOKEN>` nebo v nastavení aplikace.")]
    KeyMissing(String),

    /// Datify rejected the token (HTTP 401).
    #[error("Datify odmítlo API token (HTTP 401). Zkontrolujte, že je token opsaný celý, nebyl zneplatněn a patří k aktuálnímu účtu.")]
    Auth,

    /// The token may not touch this resource - or it does not exist. Datify
    /// answers 403 for an unknown id as well as for a real permission gap, so
    /// the message must not send the user looking only at their rights.
    #[error("Datify nepovolilo požadovanou akci (HTTP 403). Buď k tomu objektu token nemá oprávnění, nebo objekt neexistuje - Datify obojí hlásí stejně. Ověřte id výpisem nadřazeného objektu.")]
    Permission,

    /// The dataset, record or attachment does not exist (HTTP 404).
    #[error("Datify nenašlo požadovaný objekt (HTTP 404): {0}")]
    NotFound(String),

    /// Datify rejected the payload (HTTP 400/422).
    #[error("Datify odmítlo data (HTTP {status}): {message}")]
    Validation { status: u16, message: String },

    /// Rate limited (HTTP 429).
    #[error("Datify odmítlo požadavek kvůli rate limitu (HTTP 429). Zkuste to za chvíli.")]
    RateLimit,

    /// Transport failure.
    #[error("Nepodařilo se kontaktovat Datify API: {0}")]
    Network(String),

    /// Any other non-2xx, or an unparseable body.
    #[error("Datify API vrátilo chybu (HTTP {status}): {message}")]
    Http { status: u16, message: String },

    /// `cdxctl appenv` failed to store or remove the token. Maps to HTTP 500 on
    /// the CGI side with the reason, so the settings page can show it.
    #[error("{0}")]
    AppEnv(String),

    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Json(#[from] serde_json::Error),
}

impl DatifyError {
    /// HTTP status the CGI should report for this failure.
    pub fn http_status(&self) -> u16 {
        match self {
            DatifyError::Config(_) => 400,
            DatifyError::KeyMissing(_) => 412,
            DatifyError::Auth => 401,
            DatifyError::Permission => 403,
            DatifyError::NotFound(_) => 404,
            DatifyError::Validation { .. } => 422,
            DatifyError::RateLimit => 429,
            DatifyError::Network(_) => 503,
            DatifyError::Http { status, .. } => *status,
            DatifyError::AppEnv(_) => 500,
            DatifyError::Io(_) | DatifyError::Json(_) => 500,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The settings page turns these into its own HTTP reply, so a wrong status
    /// here shows up in the browser as the wrong kind of failure - a missing
    /// token reported as a server error, say.
    #[test]
    fn every_variant_maps_to_the_status_the_cgi_should_report() {
        let cases: Vec<(DatifyError, u16)> = vec![
            (DatifyError::Config("x".into()), 400),
            (DatifyError::KeyMissing("url".into()), 412),
            (DatifyError::Auth, 401),
            (DatifyError::Permission, 403),
            (DatifyError::NotFound("/p".into()), 404),
            (
                DatifyError::Validation {
                    status: 400,
                    message: "m".into(),
                },
                422,
            ),
            (DatifyError::RateLimit, 429),
            (DatifyError::Network("boom".into()), 503),
            (
                DatifyError::Http {
                    status: 502,
                    message: "m".into(),
                },
                502,
            ),
            (DatifyError::AppEnv("cdxctl selhalo".into()), 500),
            (DatifyError::Io(std::io::Error::other("io")), 500),
            (
                DatifyError::Json(serde_json::from_str::<i32>("nope").unwrap_err()),
                500,
            ),
        ];
        for (error, expected) in cases {
            assert_eq!(error.http_status(), expected, "for {error:?}");
        }
    }

    /// `Http` reports whatever Datify sent, rather than a fixed status.
    #[test]
    fn an_unclassified_http_error_passes_the_servers_status_through() {
        for status in [418, 502, 504] {
            let error = DatifyError::Http {
                status,
                message: String::new(),
            };
            assert_eq!(error.http_status(), status);
        }
    }

    #[test]
    fn the_missing_token_message_names_where_to_get_one() {
        let error = DatifyError::KeyMissing("https://app.datify.cz/tokens".into());
        assert!(
            error.to_string().contains("https://app.datify.cz/tokens"),
            "{error}"
        );
    }

    /// 403 covers two different causes, and the message must not send the user
    /// looking only at their rights.
    #[test]
    fn the_permission_message_mentions_a_missing_object_too() {
        let text = DatifyError::Permission.to_string();
        assert!(text.contains("oprávnění"), "{text}");
        assert!(text.contains("neexistuje"), "{text}");
    }
}
