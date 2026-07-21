use clap::Args;

use crate::core::error::CliError;
use crate::sources::common::{
    build_court_search_payload, insert_string, insert_string_vec, JsonMap, SearchBaseArgs,
    SearchPayloadArgs,
};

pub(crate) const SEARCH_SKUS_HELP: &str = r#"Example:
  cdx-sk search SKUS --query "základné práva a slobody" --form-of-decision Nález --limit 5
  cdx-sk search SKUS --case-number "III. ÚS 307/2026"
  cdx-sk search SKUS --judge-reporter "Robert Šorl" --date-from 2024-01-01"#;

/// Search arguments for ustavnysud_sk - Ústavný súd SR (Constitutional Court).
/// The `/search` body pages with `from`/`size` and every JSON key is the camelCase field name.
#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchSkusArgs {
    #[command(flatten)]
    pub(crate) base: SearchBaseArgs,

    #[arg(
        long = "form-of-decision",
        help = "Decision form (repeatable): e.g. Uznesenie, Nález"
    )]
    pub(crate) form_of_decision: Vec<String>,

    #[arg(
        long = "type-of-decision",
        help = "Decision type (repeatable), e.g. \"Uznesenie o odmietnutí\""
    )]
    pub(crate) type_of_decision: Vec<String>,

    #[arg(
        long = "judge-reporter",
        help = "Judge-rapporteur / sudca spravodajca (repeatable)"
    )]
    pub(crate) judge_reporter: Vec<String>,

    #[arg(
        long = "case-number",
        help = "Exact case number (spisová značka), e.g. \"III. ÚS 307/2026\""
    )]
    pub(crate) case_number: Option<String>,

    #[arg(long = "date-from", help = "Decision date from (YYYY-MM-DD)")]
    pub(crate) date_from: Option<String>,

    #[arg(long = "date-to", help = "Decision date to (YYYY-MM-DD)")]
    pub(crate) date_to: Option<String>,
}

impl SearchPayloadArgs for SearchSkusArgs {
    fn base(&self) -> &SearchBaseArgs {
        &self.base
    }

    fn extend_payload(&self, payload: &mut JsonMap) {
        insert_string_vec(payload, "formOfDecision", &self.form_of_decision);
        insert_string_vec(payload, "typeOfDecision", &self.type_of_decision);
        insert_string_vec(payload, "judgeReporter", &self.judge_reporter);
        insert_string(payload, "caseNumber", &self.case_number);
        insert_string(payload, "decisionDateFrom", &self.date_from);
        insert_string(payload, "decisionDateTo", &self.date_to);
    }

    fn build_payload(&self, _source_code: &'static str) -> Result<String, CliError> {
        build_court_search_payload(&self.base, |payload| self.extend_payload(payload))
    }
}
