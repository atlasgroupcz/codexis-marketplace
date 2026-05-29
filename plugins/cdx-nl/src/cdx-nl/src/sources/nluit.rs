use clap::Args;

use crate::sources::common::{
    insert_string, insert_string_list, FromSizeArgs, JsonMap, SearchBaseArgs, SearchPayloadArgs,
};

pub(crate) const SEARCH_NLUIT_HELP: &str = r#"Examples:
  cdx-nl search NLUIT --query "huurrecht" --court HR --size 3
  cdx-nl search NLUIT --case-number "21/00123"
  cdx-nl search NLUIT --decision-type Uitspraak --decision-date-from 2024-01-01
  cdx-nl search NLUIT --legal-area "Civiel recht" --legal-area "Huurrecht"
  cdx-nl search NLUIT --procedure Cassatie
NB: ECLI lookup is a separate route — use cdx-nl get cdx-nl://ecli/<ECLI>.
See references/search-nluit.md for the full filter cheat-sheet."#;

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchNluitArgs {
    #[command(flatten)]
    pub(crate) base: SearchBaseArgs,

    #[command(flatten)]
    pub(crate) pagination: FromSizeArgs,

    #[arg(long, help = "Court code (repeatable: HR, RBAMS, GHARL, …)")]
    pub(crate) court: Vec<String>,

    #[arg(long = "decision-type", help = "Decision type (Uitspraak, Conclusie, …)")]
    pub(crate) decision_type: Option<String>,

    #[arg(long = "decision-date-from", help = "Decision date lower bound (YYYY-MM-DD)")]
    pub(crate) decision_date_from: Option<String>,

    #[arg(long = "decision-date-to", help = "Decision date upper bound (YYYY-MM-DD)")]
    pub(crate) decision_date_to: Option<String>,

    #[arg(long = "legal-area", help = "Legal area (repeatable, e.g. \"Civiel recht\")")]
    pub(crate) legal_area: Vec<String>,

    #[arg(long, help = "Procedure type (Cassatie, Hoger beroep, Eerste aanleg)")]
    pub(crate) procedure: Option<String>,

    #[arg(long = "case-number", help = "Exact case number (e.g. \"21/00123\"). For ECLI lookup use `cdx-nl get cdx-nl://ecli/<ECLI>`.")]
    pub(crate) case_number: Option<String>,
}

impl SearchPayloadArgs for SearchNluitArgs {
    fn base(&self) -> &SearchBaseArgs { &self.base }

    fn extend_payload(&self, payload: &mut JsonMap) {
        self.pagination.insert_into(payload);
        insert_string_list(payload, "court", &self.court);
        insert_string(payload, "decisionType",     &self.decision_type);
        insert_string(payload, "decisionDateFrom", &self.decision_date_from);
        insert_string(payload, "decisionDateTo",   &self.decision_date_to);
        // NB: backend DTO field is `legalAreas` (plural) — see RechtspraakSearchRequest.java
        insert_string_list(payload, "legalAreas", &self.legal_area);
        insert_string(payload, "procedure",        &self.procedure);
        insert_string(payload, "caseNumber",       &self.case_number);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct Wrapper { #[command(flatten)] args: SearchNluitArgs }

    #[test]
    fn parses_basic_query_and_from_size() {
        let w = Wrapper::try_parse_from(["t", "--query", "huurrecht", "--from", "10", "--size", "5"]).unwrap();
        assert_eq!(w.args.base.query.as_deref(), Some("huurrecht"));
        assert_eq!(w.args.pagination.from, Some(10));
        assert_eq!(w.args.pagination.size, Some(5));
    }

    #[test]
    fn parses_repeatable_court() {
        let w = Wrapper::try_parse_from(["t", "--court", "HR", "--court", "RBAMS"]).unwrap();
        assert_eq!(w.args.court, vec!["HR".to_string(), "RBAMS".to_string()]);
    }

    #[test]
    fn payload_emits_camelcase_keys_and_from_size() {
        let args = SearchNluitArgs {
            decision_date_from: Some("2024-01-01".into()),
            pagination: FromSizeArgs { from: Some(0), size: Some(20) },
            legal_area: vec!["Civiel recht".to_string()],
            ..Default::default()
        };
        let json = args.build_payload("NLUIT").unwrap();
        assert!(json.contains("\"decisionDateFrom\":\"2024-01-01\""));
        assert!(json.contains("\"from\":0"));
        assert!(json.contains("\"size\":20"));
        // backend field is plural — must be legalAreas, not legalArea
        assert!(json.contains("\"legalAreas\""));
        assert!(!json.contains("\"legalArea\":"));   // singular form forbidden
        // must NOT contain offset/limit
        assert!(!json.contains("\"offset\""));
        assert!(!json.contains("\"limit\""));
        // sort/order live in URL query, never in body
        assert!(!json.contains("\"sort\""));
        assert!(!json.contains("\"order\""));
    }
}
