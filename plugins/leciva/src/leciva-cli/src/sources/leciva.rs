use clap::Args;

use crate::sources::common::{insert_bool, insert_string, JsonMap, SearchBaseArgs, SearchPayloadArgs};

pub(crate) const SEARCH_LECIVA_HELP: &str = r#"Examples:
  leciva-cli search CZSUKL --query "paralen"
  leciva-cli search CZSUKL --latka ibuprofen --only-dodavka
  leciva-cli search CZSUKL --kod-sukl 0000009"#;

/// Search filters for the Czech Medicines Registry (SÚKL). Mirrors
/// `LecivaSearchRequest` - every field is optional.
#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchLecivaArgs {
    #[command(flatten)]
    pub(crate) base: SearchBaseArgs,

    #[arg(long = "kod-sukl", help = "Kód SÚKL - exact match")]
    pub(crate) kod_sukl: Option<String>,
    #[arg(long, help = "ATC code - exact match")]
    pub(crate) atc: Option<String>,
    #[arg(long, help = "Pharmaceutical form (Forma) - resolved Czech name; see `get …/doc/values/forma`")]
    pub(crate) forma: Option<String>,
    #[arg(long, help = "Dispensing category (Způsob výdeje) - resolved name; see `…/doc/values/vydej`")]
    pub(crate) vydej: Option<String>,
    #[arg(long, help = "Registration status (Stav registrace) - resolved name; see `…/doc/values/stav`")]
    pub(crate) stav: Option<String>,
    #[arg(long, help = "Active substance name (fulltext against látky)")]
    pub(crate) latka: Option<String>,
    #[arg(long = "only-dodavka", help = "Only medicines currently on the market (jeDodavka = true)")]
    pub(crate) only_dodavka: bool,
    #[arg(long = "vcetne-zrusenych", help = "Include withdrawn/cancelled registrations")]
    pub(crate) vcetne_zrusenych: bool,
}

impl SearchPayloadArgs for SearchLecivaArgs {
    fn base(&self) -> &SearchBaseArgs {
        &self.base
    }

    fn extend_payload(&self, payload: &mut JsonMap) {
        insert_string(payload, "q", &self.base.query);
        insert_string(payload, "kodSukl", &self.kod_sukl);
        insert_string(payload, "atc", &self.atc);
        insert_string(payload, "forma", &self.forma);
        insert_string(payload, "vydej", &self.vydej);
        insert_string(payload, "stavRegistrace", &self.stav);
        insert_string(payload, "latka", &self.latka);
        insert_bool(payload, "onlyDodavka", self.only_dodavka);
        insert_bool(payload, "vcetneZrusenych", self.vcetne_zrusenych);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn builds_full_filter_payload_with_correct_keys() {
        let args = SearchLecivaArgs {
            base: SearchBaseArgs { query: Some("paralen".to_string()), ..SearchBaseArgs::default() },
            kod_sukl: Some("0000009".to_string()),
            atc: Some("N02BA51".to_string()),
            forma: Some("Tableta".to_string()),
            vydej: Some("volně prodejné léčivé přípravky".to_string()),
            stav: Some("Registrovaný".to_string()),
            latka: Some("ibuprofen".to_string()),
            only_dodavka: true,
            vcetne_zrusenych: true,
        };
        let p: Value = serde_json::from_str(&args.build_payload().unwrap()).unwrap();
        assert_eq!(p["q"], "paralen");
        assert_eq!(p["kodSukl"], "0000009");
        assert_eq!(p["atc"], "N02BA51");
        assert_eq!(p["forma"], "Tableta");
        assert_eq!(p["vydej"], "volně prodejné léčivé přípravky");
        assert_eq!(p["stavRegistrace"], "Registrovaný");
        assert_eq!(p["latka"], "ibuprofen");
        assert_eq!(p["onlyDodavka"], true);
        assert_eq!(p["vcetneZrusenych"], true);
    }

    #[test]
    fn omits_absent_filters_and_false_bools() {
        let args = SearchLecivaArgs {
            base: SearchBaseArgs { query: Some("x".to_string()), ..SearchBaseArgs::default() },
            ..SearchLecivaArgs::default()
        };
        let p: Value = serde_json::from_str(&args.build_payload().unwrap()).unwrap();
        assert!(p.get("kodSukl").is_none());
        assert!(p.get("atc").is_none());
        assert!(p.get("onlyDodavka").is_none());
        assert!(p.get("vcetneZrusenych").is_none());
    }
}
