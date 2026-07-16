use clap::Args;

use crate::sources::common::{insert_string, parse_ico, JsonMap, SearchBaseArgs, SearchPayloadArgs};

pub(crate) const SEARCH_OSOBY_HELP: &str = r#"Examples:
  rzpro-cli search CZRZP-OS --query "Medtronic"
  rzpro-cli search CZRZP-OS --ico 12345678"#;

/// Search filters for RZPRO registered persons/companies. Mirrors
/// `RzproPersonSearchRequest` — all optional. The fulltext key is `nazev`.
#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchOsobyArgs {
    #[command(flatten)]
    pub(crate) base: SearchBaseArgs,

    #[arg(long, value_parser = parse_ico, help = "IČO filter, exact match (canonicalized to 8 digits)")]
    pub(crate) ico: Option<String>,
}

impl SearchPayloadArgs for SearchOsobyArgs {
    fn base(&self) -> &SearchBaseArgs {
        &self.base
    }

    fn extend_payload(&self, payload: &mut JsonMap) {
        // Person search fulltext key is `nazev` (RzproPersonSearchRequest), not `q`.
        insert_string(payload, "nazev", &self.base.query);
        insert_string(payload, "ico", &self.ico);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn query_maps_to_nazev_not_q() {
        let args = SearchOsobyArgs {
            base: SearchBaseArgs { query: Some("Medtronic".to_string()), ..SearchBaseArgs::default() },
            ico: Some("12345678".to_string()),
        };
        let p: Value = serde_json::from_str(&args.build_payload().unwrap()).unwrap();
        assert_eq!(p["nazev"], "Medtronic");
        assert_eq!(p["ico"], "12345678");
        assert!(p.get("q").is_none());
    }
}
