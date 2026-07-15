use clap::Args;

use crate::sources::common::{
    insert_number, insert_string, JsonMap, SearchBaseArgs, SearchPayloadArgs,
};

pub(crate) const SEARCH_ZMLUVY_HELP: &str = r#"Example:
  register-zmluv search SKCRZ --query "údržba" --limit 5
  register-zmluv search SKCRZ --ico 00151513 --date-from 2024-01-01
  register-zmluv search SKCRZ --query "nájomná zmluva" --value-min 100000"#;

/// Search filters for the Slovak Register of Contracts (crz.gov.sk).
/// Mirrors `CrzSearchRequest` — every field is optional.
#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchZmluvyArgs {
    #[command(flatten)]
    pub(crate) base: SearchBaseArgs,

    #[arg(long, help = "Party IČO — matches either contract party, exact match")]
    pub(crate) ico: Option<String>,
    #[arg(long, help = "Publishing sector/ministry id (rezort), exact match")]
    pub(crate) rezort: Option<String>,
    #[arg(long = "value-min", help = "Minimum total contract value in EUR (sumaSpolu >=)")]
    pub(crate) value_min: Option<f64>,
    #[arg(long = "value-max", help = "Maximum total contract value in EUR (sumaSpolu <=)")]
    pub(crate) value_max: Option<f64>,
    #[arg(long = "date-from", help = "Contract date from (YYYY-MM-DD)")]
    pub(crate) date_from: Option<String>,
    #[arg(long = "date-to", help = "Contract date to (YYYY-MM-DD)")]
    pub(crate) date_to: Option<String>,
}

impl SearchPayloadArgs for SearchZmluvyArgs {
    fn base(&self) -> &SearchBaseArgs {
        &self.base
    }

    fn extend_payload(&self, payload: &mut JsonMap) {
        insert_string(payload, "ico", &self.ico);
        insert_string(payload, "rezort", &self.rezort);
        insert_number(payload, "valueMin", self.value_min);
        insert_number(payload, "valueMax", self.value_max);
        insert_string(payload, "dateFrom", &self.date_from);
        insert_string(payload, "dateTo", &self.date_to);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn builds_full_filter_payload_with_correct_field_names_and_types() {
        let args = SearchZmluvyArgs {
            base: SearchBaseArgs {
                query: Some("údržba".to_string()),
                ..SearchBaseArgs::default()
            },
            ico: Some("00151513".to_string()),
            rezort: Some("114573".to_string()),
            value_min: Some(100_000.0),
            value_max: Some(500_000.0),
            date_from: Some("2024-01-01".to_string()),
            date_to: Some("2024-12-31".to_string()),
        };

        let payload = args.build_payload().unwrap();
        let p: Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(p["q"], "údržba");
        assert_eq!(p["ico"], "00151513");
        assert_eq!(p["rezort"], "114573");
        assert_eq!(p["valueMin"], 100_000.0);
        assert_eq!(p["valueMax"], 500_000.0);
        assert_eq!(p["dateFrom"], "2024-01-01");
        assert_eq!(p["dateTo"], "2024-12-31");
    }

    #[test]
    fn omits_absent_filters() {
        let args = SearchZmluvyArgs {
            base: SearchBaseArgs {
                query: Some("test".to_string()),
                ..SearchBaseArgs::default()
            },
            ..SearchZmluvyArgs::default()
        };

        let payload = args.build_payload().unwrap();
        let p: Value = serde_json::from_str(&payload).unwrap();
        assert!(p.get("ico").is_none());
        assert!(p.get("rezort").is_none());
        assert!(p.get("valueMin").is_none());
        assert!(p.get("dateFrom").is_none());
    }

    // The SK register indexes one record per contract (upsert by id), so there
    // is no CZ-style `platnyZaznam` version flag and no publisher/counterparty
    // split — a single `ico` matches either party.
    #[test]
    fn does_not_emit_czech_only_filters() {
        let args = SearchZmluvyArgs {
            base: SearchBaseArgs {
                query: Some("test".to_string()),
                ..SearchBaseArgs::default()
            },
            ico: Some("00151513".to_string()),
            ..SearchZmluvyArgs::default()
        };

        let payload = args.build_payload().unwrap();
        let p: Value = serde_json::from_str(&payload).unwrap();
        assert!(p.get("onlyValid").is_none());
        assert!(p.get("publisherIco").is_none());
        assert!(p.get("counterpartyIco").is_none());
    }
}
