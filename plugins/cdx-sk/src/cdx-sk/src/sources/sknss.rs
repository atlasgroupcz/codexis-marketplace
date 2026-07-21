use clap::Args;

use crate::core::error::CliError;
use crate::sources::common::{
    build_court_search_payload, insert_string, insert_string_vec, JsonMap, SearchBaseArgs,
    SearchPayloadArgs,
};

pub(crate) const SEARCH_SKNSS_HELP: &str = r#"Example:
  cdx-sk search SKNSS --query "kasačná sťažnosť" --forma Rozsudok --limit 5
  cdx-sk search SKNSS --case-number "1Vs/1/2021"
  cdx-sk search SKNSS --court-first-instance "Krajský súd v Košiciach" --date-from 2024-01-01"#;

/// Search arguments for nssud_sk - Najvyšší správny súd SR (Supreme Administrative Court).
/// The `/search` body pages with `from`/`size` and every JSON key is the camelCase field name.
#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchSknssArgs {
    #[command(flatten)]
    pub(crate) base: SearchBaseArgs,

    #[arg(long = "forma", help = "Decision form (repeatable): e.g. Rozsudok, Uznesenie")]
    pub(crate) forma: Vec<String>,

    #[arg(
        long = "court-first-instance",
        help = "First-instance court (repeatable), e.g. \"Krajský súd v Košiciach\""
    )]
    pub(crate) sud1stupna: Vec<String>,

    #[arg(long = "zr", help = "Zbierka (collection) category flag, e.g. A")]
    pub(crate) zr: Option<String>,

    #[arg(
        long = "case-number",
        help = "Exact NSS case number (spisová značka), e.g. \"1Vs/1/2021\""
    )]
    pub(crate) case_number: Option<String>,

    #[arg(long = "date-from", help = "Decision date from (YYYY-MM-DD)")]
    pub(crate) date_from: Option<String>,

    #[arg(long = "date-to", help = "Decision date to (YYYY-MM-DD)")]
    pub(crate) date_to: Option<String>,
}

impl SearchPayloadArgs for SearchSknssArgs {
    fn base(&self) -> &SearchBaseArgs {
        &self.base
    }

    fn extend_payload(&self, payload: &mut JsonMap) {
        insert_string_vec(payload, "forma", &self.forma);
        insert_string_vec(payload, "sud1stupna", &self.sud1stupna);
        insert_string(payload, "zr", &self.zr);
        insert_string(payload, "caseNumber", &self.case_number);
        insert_string(payload, "decisionDateFrom", &self.date_from);
        insert_string(payload, "decisionDateTo", &self.date_to);
    }

    fn build_payload(&self, _source_code: &'static str) -> Result<String, CliError> {
        build_court_search_payload(&self.base, |payload| self.extend_payload(payload))
    }
}
