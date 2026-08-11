//! The ureq plumbing shared by the lookup plugins and by `cdx-notify`: one
//! configured agent, one way to sort a result, one retry policy.
//!
//! [`crate::http`] next door holds the stateless helpers - URL joining, which
//! statuses are worth retrying, how long to wait between attempts. This module
//! is what turns those into actual requests.
//!
//! What stays with the caller is the judgement: whether a 404 is an error or an
//! empty result, whether an HTTP 200 carrying HTML counts as a failure, and how
//! to word any of it. This module reports an [`Outcome`] and stops there.

use std::time::Duration;

use crate::http::{backoff, is_failure, retryable_status};

/// A response as ureq 3 models it: the `http` crate's type over ureq's
/// streaming body.
pub type Response = ureq::http::Response<ureq::Body>;

/// How many times a call is issued before giving up.
const MAX_ATTEMPTS: usize = 3;

/// An agent configured the way every lookup needs it.
///
/// ureq keeps a connection pool inside the agent, so a client that holds on to
/// one reuses the TCP+TLS connection for follow-up requests to the same host.
///
/// `user_agent` is sent with every request the agent makes; a single request can
/// still override it with its own `User-Agent` header. Plugins pass their own
/// `name/version`, except where the server demands something else - the Slovak
/// cadastre's WAF rejects anything that does not look like a browser.
///
/// `http_status_as_error(false)` is the load-bearing part: a 4xx/5xx arrives as
/// an ordinary response, so [`outcome`] can read its body and hand it to the
/// caller. ureq's own error carries the status as a bare number, which would drop
/// the server's explanation. Anything using this agent must therefore judge the
/// status itself - [`outcome`] and [`call_with_retry`] are how.
pub fn agent(user_agent: &str, connect: Duration, recv: Duration) -> ureq::Agent {
    ureq::Agent::config_builder()
        .user_agent(user_agent)
        .timeout_connect(Some(connect))
        .timeout_recv_response(Some(recv))
        .http_status_as_error(false)
        .build()
        .into()
}

/// A status rendered for a human: `"404 Not Found"`, or just the number for a
/// code with no canonical phrase.
///
/// ureq 3 does not keep the server's own reason phrase, so the phrase is derived
/// from the code - which is why this is one function rather than a phrase lookup
/// the caller has to remember to pair with a separate formatter.
pub fn status_text(code: u16) -> String {
    let reason = ureq::http::StatusCode::from_u16(code)
        .ok()
        .and_then(|status| status.canonical_reason())
        .unwrap_or_default();
    crate::http::format_status(code, reason)
}

/// Which transport failures are worth another attempt.
///
/// [`retryable_status`] covers failures that carry an HTTP status; this is its
/// counterpart for the ones that carry none - a TLS reset, a timeout, a refused
/// connection, a name that does not resolve. A malformed URI or a protocol error
/// is not retried: the second attempt fails the same way.
pub fn is_retryable_transport(error: &ureq::Error) -> bool {
    matches!(
        error,
        ureq::Error::Io(_)
            | ureq::Error::Timeout(_)
            | ureq::Error::ConnectionFailed
            | ureq::Error::HostNotFound
    )
}

/// How a call ended once retrying stopped making sense.
pub enum Outcome {
    /// A status below 400. The body has NOT been read, so the caller can stream
    /// it, parse it as JSON, or read it as text.
    Ok(Response),
    /// A status of 400 or above, and the body, already read. The body is always
    /// read so the connection returns to the pool even when the caller drops it.
    Status(u16, String),
    /// The request never produced a response.
    Transport(ureq::Error),
}

impl Outcome {
    /// The response, or a message naming the method, status and URL.
    ///
    /// The server's body is left out on purpose: a caller reaching for the
    /// response wants to read the body itself, and services that answer a
    /// failure with a full HTML error page add nothing by quoting it.
    ///
    /// The status survives only as text, so match on [`Outcome`] instead when
    /// the code itself decides what happens next - a 404 that means "no such
    /// record" rather than "broken", say.
    pub fn into_response(self, method: &str, url: &str) -> Result<Response, String> {
        match self {
            Outcome::Ok(resp) => Ok(resp),
            Outcome::Status(code, _) => {
                Err(format!("HTTP {} on {method} {url}", status_text(code)))
            }
            Outcome::Transport(error) => Err(format!("{method} {url} failed: {error}")),
        }
    }

    /// The body as text, or a message naming the method, status and URL - with
    /// the server's own explanation appended when it sent one.
    ///
    /// This is the shape every JSON lookup wants, which is why the wording lives
    /// here rather than being written out per plugin. As with [`Self::into_response`],
    /// the status survives only as text; match on [`Outcome`] to branch on it.
    pub fn into_text(self, method: &str, url: &str) -> Result<String, String> {
        match self {
            Outcome::Ok(mut resp) => resp
                .body_mut()
                .read_to_string()
                .map_err(|e| format!("{method} {url}: could not read the body: {e}")),
            Outcome::Status(code, body) if !body.trim().is_empty() => Err(format!(
                "HTTP {} on {method} {url}\n{}",
                status_text(code),
                body.trim()
            )),
            // No body worth quoting - the status line says it all, and that
            // wording lives in `into_response`.
            failed => Err(failed.into_response(method, url).unwrap_err()),
        }
    }
}

/// Sorts one call's result into an [`Outcome`], without retrying.
///
/// For a request that must not be issued twice - a non-idempotent POST - this is
/// the whole of what [`call_with_retry`] offers minus the repetition.
pub fn outcome(result: Result<Response, ureq::Error>) -> Outcome {
    match result {
        Ok(mut resp) => {
            let code = resp.status().as_u16();
            if !is_failure(code) {
                return Outcome::Ok(resp);
            }
            // Read even though the caller may not want it: an undrained body
            // keeps the connection out of the pool.
            let body = resp.body_mut().read_to_string().unwrap_or_default();
            Outcome::Status(code, body)
        }
        Err(error) => Outcome::Transport(error),
    }
}

/// Issues the request and retries while that makes sense: [`retryable_status`]
/// for HTTP statuses, [`is_retryable_transport`] for failures that carry none,
/// [`backoff`] in between.
///
/// `call` issues the request afresh on every attempt, because sending a
/// `ureq::RequestBuilder` consumes it. Use [`outcome`] instead for a request
/// that must not be repeated.
pub fn call_with_retry(call: impl Fn() -> Result<Response, ureq::Error>) -> Outcome {
    let mut attempt = 0;
    loop {
        let settled = outcome(call());
        let worth_retrying = match &settled {
            Outcome::Ok(_) => false,
            Outcome::Status(code, _) => retryable_status(*code),
            Outcome::Transport(error) => is_retryable_transport(error),
        };
        if !worth_retrying || attempt + 1 >= MAX_ATTEMPTS {
            return settled;
        }
        std::thread::sleep(backoff(attempt));
        attempt += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    /// Builds a `Response` with the given status and body, the way a server
    /// would - there is no ureq constructor for one.
    fn response(code: u16, body: &str) -> Response {
        ureq::http::Response::builder()
            .status(code)
            .body(ureq::Body::builder().data(body.to_string()))
            .expect("a hand-built response is well-formed")
    }

    #[test]
    fn connection_and_io_failures_are_retryable() {
        // A TLS reset arrives as Io - that is the failure this policy exists for.
        assert!(is_retryable_transport(&ureq::Error::Io(
            std::io::Error::from(std::io::ErrorKind::ConnectionReset)
        )));
        assert!(is_retryable_transport(&ureq::Error::ConnectionFailed));
        assert!(is_retryable_transport(&ureq::Error::HostNotFound));
        assert!(is_retryable_transport(&ureq::Error::Timeout(
            ureq::Timeout::Connect
        )));
    }

    #[test]
    fn bad_uri_is_not_retryable() {
        assert!(!is_retryable_transport(&ureq::Error::BadUri(
            "h ttp://example.invalid".into()
        )));
    }

    #[test]
    fn a_success_is_returned_on_the_first_attempt() {
        let attempts = Cell::new(0);
        let outcome = call_with_retry(|| {
            attempts.set(attempts.get() + 1);
            Ok(response(200, "body"))
        });
        assert_eq!(attempts.get(), 1);
        assert!(matches!(outcome, Outcome::Ok(_)));
    }

    #[test]
    fn a_retryable_status_is_retried_up_to_the_limit() {
        let attempts = Cell::new(0);
        let outcome = call_with_retry(|| {
            attempts.set(attempts.get() + 1);
            Ok(response(503, "unavailable"))
        });
        assert_eq!(attempts.get(), MAX_ATTEMPTS);
        match outcome {
            Outcome::Status(code, body) => {
                assert_eq!(code, 503);
                assert_eq!(body, "unavailable");
            }
            _ => panic!("a 503 must end as Status"),
        }
    }

    #[test]
    fn a_non_retryable_status_gives_up_at_once_and_keeps_the_body() {
        let attempts = Cell::new(0);
        let outcome = call_with_retry(|| {
            attempts.set(attempts.get() + 1);
            Ok(response(404, "not found"))
        });
        assert_eq!(attempts.get(), 1);
        match outcome {
            Outcome::Status(code, body) => {
                assert_eq!(code, 404);
                assert_eq!(body, "not found");
            }
            _ => panic!("a 404 must end as Status"),
        }
    }

    #[test]
    fn a_retryable_transport_failure_is_retried_then_reported() {
        let attempts = Cell::new(0);
        let outcome = call_with_retry(|| {
            attempts.set(attempts.get() + 1);
            Err(ureq::Error::ConnectionFailed)
        });
        assert_eq!(attempts.get(), MAX_ATTEMPTS);
        assert!(matches!(outcome, Outcome::Transport(_)));
    }

    #[test]
    fn a_transport_failure_that_will_not_improve_is_not_retried() {
        let attempts = Cell::new(0);
        let outcome = call_with_retry(|| {
            attempts.set(attempts.get() + 1);
            Err(ureq::Error::BadUri("h ttp://example.invalid".into()))
        });
        assert_eq!(attempts.get(), 1);
        assert!(matches!(outcome, Outcome::Transport(_)));
    }

    #[test]
    fn a_retry_that_then_succeeds_returns_the_success() {
        let attempts = Cell::new(0);
        let outcome = call_with_retry(|| {
            attempts.set(attempts.get() + 1);
            if attempts.get() == 1 {
                Ok(response(503, "warming up"))
            } else {
                Ok(response(200, "body"))
            }
        });
        assert_eq!(attempts.get(), 2);
        assert!(matches!(outcome, Outcome::Ok(_)));
    }

    #[test]
    fn status_text_names_known_codes_and_falls_back_to_the_number() {
        assert_eq!(status_text(404), "404 Not Found");
        assert_eq!(status_text(503), "503 Service Unavailable");
        assert_eq!(status_text(799), "799");
    }
}
