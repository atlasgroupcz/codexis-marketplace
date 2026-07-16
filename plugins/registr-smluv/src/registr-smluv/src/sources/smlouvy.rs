use clap::Args;

use crate::sources::common::{
    insert_bool, insert_number, insert_string, parse_ico, JsonMap, SearchBaseArgs,
    SearchPayloadArgs,
};

pub(crate) const SEARCH_SMLOUVY_HELP: &str = r#"Example:
  registr-smluv search CZRS --query "rekonstrukce silnice" --limit 5
  registr-smluv search CZRS --publisher-ico 00064581 --date-from 2024-01-01
  registr-smluv search CZRS --ico 27074358 --value-min 1000000 --only-valid"#;

/// Search filters for the Czech Register of Contracts (smlouvy.gov.cz).
/// Mirrors `SmlouvySearchRequest` — every field is optional.
#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchSmlouvyArgs {
    #[command(flatten)]
    pub(crate) base: SearchBaseArgs,

    #[arg(
        long,
        value_parser = parse_ico,
        help = "IČO of the publisher OR the counterparty (canonicalized to 8 digits)"
    )]
    pub(crate) ico: Option<String>,
    #[arg(
        long = "publisher-ico",
        value_parser = parse_ico,
        help = "Publisher (contracting authority) IČO, exact match (canonicalized to 8 digits)"
    )]
    pub(crate) publisher_ico: Option<String>,
    #[arg(
        long = "counterparty-ico",
        value_parser = parse_ico,
        help = "Counterparty (supplier) IČO, exact match (canonicalized to 8 digits)"
    )]
    pub(crate) counterparty_ico: Option<String>,
    #[arg(long = "value-min", help = "Minimum contract value incl. VAT (hodnotaVcetneDph >=)")]
    pub(crate) value_min: Option<f64>,
    #[arg(long = "value-max", help = "Maximum contract value incl. VAT (hodnotaVcetneDph <=)")]
    pub(crate) value_max: Option<f64>,
    #[arg(long = "date-from", help = "Conclusion date from (YYYY-MM-DD)")]
    pub(crate) date_from: Option<String>,
    #[arg(long = "date-to", help = "Conclusion date to (YYYY-MM-DD)")]
    pub(crate) date_to: Option<String>,
    #[arg(long = "only-valid", help = "Only currently valid contract records")]
    pub(crate) only_valid: bool,
}

impl SearchPayloadArgs for SearchSmlouvyArgs {
    fn base(&self) -> &SearchBaseArgs {
        &self.base
    }

    fn extend_payload(&self, payload: &mut JsonMap) {
        insert_string(payload, "ico", &self.ico);
        insert_string(payload, "publisherIco", &self.publisher_ico);
        insert_string(payload, "counterpartyIco", &self.counterparty_ico);
        insert_number(payload, "valueMin", self.value_min);
        insert_number(payload, "valueMax", self.value_max);
        insert_string(payload, "dateFrom", &self.date_from);
        insert_string(payload, "dateTo", &self.date_to);
        insert_bool(payload, "onlyValid", self.only_valid);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn builds_full_filter_payload_with_correct_field_names_and_types() {
        let args = SearchSmlouvyArgs {
            base: SearchBaseArgs {
                query: Some("silnice".to_string()),
                ..SearchBaseArgs::default()
            },
            ico: Some("27074358".to_string()),
            publisher_ico: Some("00064581".to_string()),
            counterparty_ico: Some("12345678".to_string()),
            value_min: Some(1_000_000.0),
            value_max: Some(5_000_000.0),
            date_from: Some("2024-01-01".to_string()),
            date_to: Some("2024-12-31".to_string()),
            only_valid: true,
        };

        let payload = args.build_payload().unwrap();
        let p: Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(p["q"], "silnice");
        assert_eq!(p["ico"], "27074358");
        assert_eq!(p["publisherIco"], "00064581");
        assert_eq!(p["counterpartyIco"], "12345678");
        assert_eq!(p["valueMin"], 1_000_000.0);
        assert_eq!(p["valueMax"], 5_000_000.0);
        assert_eq!(p["dateFrom"], "2024-01-01");
        assert_eq!(p["dateTo"], "2024-12-31");
        assert_eq!(p["onlyValid"], true);
    }

    #[test]
    fn omits_absent_filters_and_only_valid_when_false() {
        let args = SearchSmlouvyArgs {
            base: SearchBaseArgs {
                query: Some("test".to_string()),
                ..SearchBaseArgs::default()
            },
            only_valid: false,
            ..SearchSmlouvyArgs::default()
        };

        let payload = args.build_payload().unwrap();
        let p: Value = serde_json::from_str(&payload).unwrap();
        assert!(p.get("ico").is_none());
        assert!(p.get("publisherIco").is_none());
        assert!(p.get("valueMin").is_none());
        assert!(p.get("onlyValid").is_none());
    }
}
