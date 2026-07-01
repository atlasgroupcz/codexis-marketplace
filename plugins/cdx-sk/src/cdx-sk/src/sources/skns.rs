use clap::Args;

use crate::core::error::CliError;
use crate::sources::common::{
    build_court_search_payload, insert_string, insert_string_vec, JsonMap, SearchBaseArgs,
    SearchPayloadArgs,
};

pub(crate) const SEARCH_SKNS_HELP: &str = r#"Example:
  cdx-sk search SKNS --query "dovolanie" --kolegium Trestnoprávne --limit 5
  cdx-sk search SKNS --case-number "1Tdo/22/2026"
  cdx-sk search SKNS --chamber-type velky_senat --date-from 2024-01-01"#;

/// Search arguments for nsud_sk — Najvyšší súd SR (Supreme Court).
/// The `/search` body pages with `from`/`size` and every JSON key is the camelCase field name.
#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchSknsArgs {
    #[command(flatten)]
    pub(crate) base: SearchBaseArgs,

    #[arg(
        long = "kolegium",
        help = "Collegium (repeatable): Občianskoprávne, Obchodnoprávne, Správne, Trestnoprávne"
    )]
    pub(crate) kolegium: Vec<String>,

    #[arg(
        long = "chamber-type",
        help = "Special-chamber flag: velky_senat | kompetencny_senat | normal"
    )]
    pub(crate) chamber_type: Option<String>,

    #[arg(
        long = "case-number",
        help = "Exact case number (spisová značka), e.g. \"1Tdo/22/2026\""
    )]
    pub(crate) case_number: Option<String>,

    #[arg(long = "date-from", help = "Decision date from (YYYY-MM-DD)")]
    pub(crate) date_from: Option<String>,

    #[arg(long = "date-to", help = "Decision date to (YYYY-MM-DD)")]
    pub(crate) date_to: Option<String>,
}

impl SearchPayloadArgs for SearchSknsArgs {
    fn base(&self) -> &SearchBaseArgs {
        &self.base
    }

    fn extend_payload(&self, payload: &mut JsonMap) {
        insert_string_vec(payload, "kolegium", &self.kolegium);
        insert_string(payload, "chamberType", &self.chamber_type);
        insert_string(payload, "caseNumber", &self.case_number);
        insert_string(payload, "decisionDateFrom", &self.date_from);
        insert_string(payload, "decisionDateTo", &self.date_to);
    }

    fn build_payload(&self, _source_code: &'static str) -> Result<String, CliError> {
        build_court_search_payload(&self.base, |payload| self.extend_payload(payload))
    }
}
