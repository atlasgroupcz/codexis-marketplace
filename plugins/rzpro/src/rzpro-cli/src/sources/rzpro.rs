use clap::Args;

use crate::sources::common::{
    insert_bool, insert_string, parse_ico, JsonMap, SearchBaseArgs, SearchPayloadArgs,
};

pub(crate) const SEARCH_RZPRO_HELP: &str = r#"Examples:
  rzpro-cli search CZRZP --query "gliadin"
  rzpro-cli search CZRZP --druh "in vitro" --riziko III --stat Německo
  rzpro-cli search CZRZP --katalogove-cislo G0068486
  rzpro-cli search CZRZP --expiruje-do 2027-01-01"#;

/// Search filters for RZPRO devices. Mirrors `RzproSearchRequest` - all optional.
#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchRzproArgs {
    #[command(flatten)]
    pub(crate) base: SearchBaseArgs,

    #[arg(
        long = "evidencni-cislo",
        help = "Evidenční číslo - exact, 8-digit device id"
    )]
    pub(crate) evidencni_cislo: Option<String>,
    #[arg(long, help = "Device kind (Druh ZP) filter")]
    pub(crate) druh: Option<String>,
    #[arg(
        long,
        help = "Risk class (Míra rizika) filter, e.g. I, IIa, III, IVDR C"
    )]
    pub(crate) riziko: Option<String>,
    #[arg(long, help = "Manufacturer country (Stát výrobce) filter")]
    pub(crate) stat: Option<String>,
    #[arg(long, value_parser = parse_ico, help = "Manufacturer/holder IČO filter, exact match (canonicalized to 8 digits)")]
    pub(crate) ico: Option<String>,
    #[arg(
        long = "katalogove-cislo",
        help = "Catalogue number of a device variant, exact match"
    )]
    pub(crate) katalogove_cislo: Option<String>,
    #[arg(
        long = "expiruje-do",
        help = "Only devices with Platnost do on or before this date (YYYY-MM-DD)"
    )]
    pub(crate) expiruje_do: Option<String>,
    #[arg(
        long = "only-active",
        help = "Only currently valid devices (Stav = Platný)"
    )]
    pub(crate) only_active: bool,
}

impl SearchPayloadArgs for SearchRzproArgs {
    fn base(&self) -> &SearchBaseArgs {
        &self.base
    }

    fn extend_payload(&self, payload: &mut JsonMap) {
        insert_string(payload, "q", &self.base.query);
        insert_string(payload, "evidencniCislo", &self.evidencni_cislo);
        insert_string(payload, "druh", &self.druh);
        insert_string(payload, "miraRizika", &self.riziko);
        insert_string(payload, "statVyrobce", &self.stat);
        insert_string(payload, "ico", &self.ico);
        insert_string(payload, "katalogoveCislo", &self.katalogove_cislo);
        insert_string(payload, "expirujeDo", &self.expiruje_do);
        insert_bool(payload, "onlyActive", self.only_active);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn builds_full_filter_payload_with_correct_keys() {
        let args = SearchRzproArgs {
            base: SearchBaseArgs {
                query: Some("gliadin".to_string()),
                ..SearchBaseArgs::default()
            },
            evidencni_cislo: Some("00000019".to_string()),
            druh: Some("Diagnostický zdravotnický prostředek in vitro".to_string()),
            riziko: Some("III".to_string()),
            stat: Some("Německo".to_string()),
            ico: Some("12345678".to_string()),
            katalogove_cislo: Some("G0068486".to_string()),
            expiruje_do: Some("2027-01-01".to_string()),
            only_active: true,
        };
        let p: Value = serde_json::from_str(&args.build_payload().unwrap()).unwrap();
        assert_eq!(p["q"], "gliadin");
        assert_eq!(p["evidencniCislo"], "00000019");
        assert_eq!(p["druh"], "Diagnostický zdravotnický prostředek in vitro");
        assert_eq!(p["miraRizika"], "III");
        assert_eq!(p["statVyrobce"], "Německo");
        assert_eq!(p["ico"], "12345678");
        assert_eq!(p["katalogoveCislo"], "G0068486");
        assert_eq!(p["expirujeDo"], "2027-01-01");
        assert_eq!(p["onlyActive"], true);
    }

    #[test]
    fn omits_absent_filters_and_false_bool() {
        let args = SearchRzproArgs {
            base: SearchBaseArgs {
                query: Some("x".to_string()),
                ..SearchBaseArgs::default()
            },
            ..SearchRzproArgs::default()
        };
        let p: Value = serde_json::from_str(&args.build_payload().unwrap()).unwrap();
        assert!(p.get("druh").is_none());
        assert!(p.get("miraRizika").is_none());
        assert!(p.get("onlyActive").is_none());
    }
}
