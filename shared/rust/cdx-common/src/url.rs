//! Percent-encoding for values that go into a URL.
//!
//! Three plugins had grown their own copy of the same RFC 3986 set. The rule is
//! not plugin-specific, so it lives here; what differs between the two functions
//! below is one character, and that difference is the reason there are two.

use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};

/// RFC 3986's unreserved set: everything else is escaped.
///
/// Deliberately the conservative choice. A space becomes `%20`, which every
/// decoder reads as a space; form encoding would make it `+`, which only a
/// form-urlencoded parser reads that way.
const UNRESERVED: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'~');

/// The same set, but `/` survives - it is legal inside a query value and keeps
/// a path readable in logs and error messages.
const UNRESERVED_KEEPING_SLASH: &AsciiSet = &UNRESERVED.remove(b'/');

/// Encodes a value for use as a query parameter.
pub fn encode_query_value(value: &str) -> String {
    utf8_percent_encode(value, UNRESERVED).to_string()
}

/// Encodes a filesystem path for use as a query parameter, leaving the
/// separators alone.
pub fn encode_path_value(path: &str) -> String {
    utf8_percent_encode(path, UNRESERVED_KEEPING_SLASH).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_query_value_keeps_only_the_unreserved_characters() {
        assert_eq!(encode_query_value("/tmp/a b.pdf"), "%2Ftmp%2Fa%20b.pdf");
        assert_eq!(encode_query_value("a-b_c.d~e"), "a-b_c.d~e");
    }

    #[test]
    fn query_structure_characters_are_escaped_so_the_value_stays_one_value() {
        assert_eq!(encode_query_value("a&b=c?d"), "a%26b%3Dc%3Fd");
    }

    #[test]
    fn a_path_value_keeps_its_separators() {
        assert_eq!(encode_path_value("/tmp/a b.mp4"), "/tmp/a%20b.mp4");
    }

    #[test]
    fn non_ascii_is_escaped_byte_by_byte_as_utf8() {
        assert_eq!(encode_query_value("závěrka"), "z%C3%A1v%C4%9Brka");
        assert_eq!(encode_path_value("závěrka"), "z%C3%A1v%C4%9Brka");
    }
}
