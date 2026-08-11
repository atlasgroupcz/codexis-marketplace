use clap::Args;

use crate::sources::common::{
    insert_string, insert_string_list, JsonMap, OffsetLimitArgs, SearchBaseArgs, SearchPayloadArgs,
};

pub(crate) const SEARCH_NLCVDR_HELP: &str = r#"Examples:
  cdx-nl search NLCVDR --query "algemene plaatselijke verordening" --limit 5
  cdx-nl search NLCVDR --work-id CVDR72510
  cdx-nl search NLCVDR --organisatie-type Gemeente --organisatie Amsterdam
  cdx-nl search NLCVDR --valid-at 2024-01-01
  cdx-nl search NLCVDR --query "geluidhinder" --search-pages --sort date --order desc
See references/search-nlcvdr.md for the full filter cheat-sheet."#;

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchNlcvdrArgs {
    #[command(flatten)]
    pub(crate) base: SearchBaseArgs,

    #[command(flatten)]
    pub(crate) pagination: OffsetLimitArgs,

    #[arg(long = "work-id", help = "CVDR work identifier (e.g. CVDR72510)")]
    pub(crate) work_id: Option<String>,

    #[arg(
        long = "type",
        help = "dcterms:type filter (DOCUMENT mode only; usually 'regeling')"
    )]
    pub(crate) type_: Option<String>,

    #[arg(
        long = "organisatie",
        help = "Issuing body, exact match (DOCUMENT mode only)"
    )]
    pub(crate) organisatie: Option<String>,

    #[arg(
        long = "organisatie-type",
        help = "Issuing-body type: Gemeente, Provincie, Waterschap, Koninkrijksdeel, \
                CaribischOpenbaarLichaam, NederlandseAntillen, RegionaalSamenwerkingsorgaan \
                (DOCUMENT mode only)"
    )]
    pub(crate) organisatie_type: Option<String>,

    #[arg(
        long = "rechtsgebied",
        help = "Legal area (repeatable; DOCUMENT mode only)"
    )]
    pub(crate) rechtsgebied: Vec<String>,

    #[arg(long = "valid-at", help = "Validity date (YYYY-MM-DD)")]
    pub(crate) valid_at: Option<String>,

    #[arg(long = "valid-from-from", help = "Valid-from lower bound (YYYY-MM-DD)")]
    pub(crate) valid_from_from: Option<String>,

    #[arg(long = "valid-from-to", help = "Valid-from upper bound (YYYY-MM-DD)")]
    pub(crate) valid_from_to: Option<String>,

    #[arg(
        long = "search-pages",
        help = "If set, search page-level consolidated-text markdown instead of metadata only. \
                  NOTE: type/organisatie/organisatie-type/rechtsgebied are ignored in this mode"
    )]
    pub(crate) search_pages: bool,
}

impl SearchPayloadArgs for SearchNlcvdrArgs {
    fn base(&self) -> &SearchBaseArgs {
        &self.base
    }

    fn extend_payload(&self, payload: &mut JsonMap) {
        self.pagination.insert_into(payload);
        insert_string(payload, "workId", &self.work_id);
        insert_string(payload, "type", &self.type_);
        insert_string(payload, "organisatie", &self.organisatie);
        insert_string(payload, "organisatieType", &self.organisatie_type);
        insert_string_list(payload, "rechtsgebied", &self.rechtsgebied);
        insert_string(payload, "validAt", &self.valid_at);
        insert_string(payload, "validFromFrom", &self.valid_from_from);
        insert_string(payload, "validFromTo", &self.valid_from_to);
        if self.search_pages {
            payload.insert("searchPages".into(), true.into());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct Wrapper {
        #[command(flatten)]
        args: SearchNlcvdrArgs,
    }

    #[test]
    fn parses_query_and_workid() {
        let w = Wrapper::try_parse_from(["t", "--query", "apv", "--work-id", "CVDR72510"]).unwrap();
        assert_eq!(w.args.base.query.as_deref(), Some("apv"));
        assert_eq!(w.args.work_id.as_deref(), Some("CVDR72510"));
    }

    #[test]
    fn parses_repeatable_rechtsgebied() {
        let w = Wrapper::try_parse_from([
            "t",
            "--rechtsgebied",
            "openbare orde en veiligheid",
            "--rechtsgebied",
            "ruimtelijke ordening",
        ])
        .unwrap();
        assert_eq!(
            w.args.rechtsgebied,
            vec![
                "openbare orde en veiligheid".to_string(),
                "ruimtelijke ordening".to_string()
            ]
        );
    }

    #[test]
    fn payload_emits_camelcase_keys() {
        let args = SearchNlcvdrArgs {
            organisatie_type: Some("Gemeente".into()),
            valid_from_from: Some("2020-01-01".into()),
            ..Default::default()
        };
        let json = args.build_payload("NLCVDR").unwrap();
        assert!(json.contains("\"organisatieType\":\"Gemeente\""));
        assert!(json.contains("\"validFromFrom\":\"2020-01-01\""));
    }

    #[test]
    fn search_pages_flag_emits_bool() {
        let args = SearchNlcvdrArgs {
            search_pages: true,
            ..Default::default()
        };
        let json = args.build_payload("NLCVDR").unwrap();
        assert!(json.contains("\"searchPages\":true"));
    }
}
