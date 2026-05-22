use clap::Args;

use crate::sources::common::{
    insert_string, insert_string_list, JsonMap, OffsetLimitArgs, SearchBaseArgs, SearchPayloadArgs,
};

pub(crate) const SEARCH_NLBWB_HELP: &str = r#"Examples:
  cdx-nl search NLBWB --query "Burgerlijk Wetboek" --limit 5
  cdx-nl search NLBWB --bwb-id BWBR0001827
  cdx-nl search NLBWB --afkorting BW
  cdx-nl search NLBWB --valid-at 2026-01-01
  cdx-nl search NLBWB --rechtsgebied "burgerlijk recht" --rechtsgebied "huurrecht"
  cdx-nl search NLBWB --pdf-kind stb_publication --search-pages
See references/search-nlbwb.md for the full filter cheat-sheet."#;

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchNlbwbArgs {
    #[command(flatten)]
    pub(crate) base: SearchBaseArgs,

    #[command(flatten)]
    pub(crate) pagination: OffsetLimitArgs,

    #[arg(long = "bwb-id", help = "BWB document identifier (e.g. BWBR0001827)")]
    pub(crate) bwb_id: Option<String>,

    #[arg(long = "type", help = "BWB type letter: R (Regelgeving), V (Verdragen), W (Wetgeving)")]
    pub(crate) type_: Option<String>,

    #[arg(long = "regeling-soort", help = "Regeling soort (e.g. Wet, Besluit, Verdrag)")]
    pub(crate) regeling_soort: Option<String>,

    #[arg(long = "issuing-authority", help = "Issuing authority")]
    pub(crate) issuing_authority: Option<String>,

    #[arg(long = "afkorting", help = "Common abbreviation (e.g. BW, Sr, AWB)")]
    pub(crate) afkorting: Option<String>,

    #[arg(long = "rechtsgebied", help = "Legal area (repeatable)")]
    pub(crate) rechtsgebied: Vec<String>,

    #[arg(long = "valid-at", help = "Validity date (YYYY-MM-DD)")]
    pub(crate) valid_at: Option<String>,

    #[arg(long = "valid-from-from", help = "Valid-from lower bound (YYYY-MM-DD)")]
    pub(crate) valid_from_from: Option<String>,

    #[arg(long = "valid-from-to", help = "Valid-from upper bound (YYYY-MM-DD)")]
    pub(crate) valid_from_to: Option<String>,

    #[arg(long = "search-pages", help = "If set, search page-level markdown instead of metadata only")]
    pub(crate) search_pages: bool,

    #[arg(long = "pdf-kind", value_parser = ["consolidated", "stb_publication"],
          help = "Restrict matches to a PDF kind")]
    pub(crate) pdf_kind: Option<String>,
}

impl SearchPayloadArgs for SearchNlbwbArgs {
    fn base(&self) -> &SearchBaseArgs { &self.base }

    fn extend_payload(&self, payload: &mut JsonMap) {
        self.pagination.insert_into(payload);
        insert_string(payload, "bwbId",            &self.bwb_id);
        insert_string(payload, "type",             &self.type_);
        insert_string(payload, "regelingSoort",    &self.regeling_soort);
        insert_string(payload, "issuingAuthority", &self.issuing_authority);
        insert_string(payload, "afkorting",        &self.afkorting);
        insert_string_list(payload, "rechtsgebied", &self.rechtsgebied);
        insert_string(payload, "validAt",          &self.valid_at);
        insert_string(payload, "validFromFrom",    &self.valid_from_from);
        insert_string(payload, "validFromTo",      &self.valid_from_to);
        if self.search_pages {
            payload.insert("searchPages".into(), true.into());
        }
        insert_string(payload, "pdfKind",          &self.pdf_kind);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct Wrapper { #[command(flatten)] args: SearchNlbwbArgs }

    #[test]
    fn parses_basic_query_and_pagination() {
        let w = Wrapper::try_parse_from(["t", "--query", "BW", "--limit", "5", "--offset", "10"]).unwrap();
        assert_eq!(w.args.base.query.as_deref(), Some("BW"));
        assert_eq!(w.args.pagination.limit, Some(5));
        assert_eq!(w.args.pagination.offset, Some(10));
    }

    #[test]
    fn parses_repeatable_rechtsgebied() {
        let w = Wrapper::try_parse_from(["t",
            "--rechtsgebied", "burgerlijk recht",
            "--rechtsgebied", "huurrecht"]).unwrap();
        assert_eq!(w.args.rechtsgebied, vec!["burgerlijk recht".to_string(), "huurrecht".to_string()]);
    }

    #[test]
    fn parses_pdf_kind_enum() {
        assert!(Wrapper::try_parse_from(["t", "--pdf-kind", "consolidated"]).is_ok());
        assert!(Wrapper::try_parse_from(["t", "--pdf-kind", "stb_publication"]).is_ok());
        assert!(Wrapper::try_parse_from(["t", "--pdf-kind", "invalid"]).is_err());
    }

    #[test]
    fn payload_emits_camelcase_keys() {
        let args = SearchNlbwbArgs {
            bwb_id: Some("BWBR0001827".into()),
            valid_from_from: Some("2020-01-01".into()),
            ..Default::default()
        };
        let json = args.build_payload("NLBWB").unwrap();
        assert!(json.contains("\"bwbId\":\"BWBR0001827\""));
        assert!(json.contains("\"validFromFrom\":\"2020-01-01\""));
    }
}
