use clap::Args;

use crate::sources::common::{insert_string, DateRangeArgs, JsonMap, SearchBaseArgs, SearchPayloadArgs};

pub(crate) const SEARCH_ATJD_HELP: &str = r#"Example:
  cdx-at search ATJD --query "Grundrechte" --application Vfgh --limit 5
  cdx-at search ATJD --case-number "G 123/2024" --decision-type Erkenntnis"#;

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchAtjdArgs {
    #[command(flatten)]
    pub(crate) base: SearchBaseArgs,

    #[arg(long, help = "Application (e.g. \"Vfgh\", \"Vwgh\", \"Justiz\")")]
    pub(crate) application: Option<String>,

    #[arg(long = "document-type", help = "Document type")]
    pub(crate) document_type: Option<String>,

    #[arg(long = "decision-type", help = "Decision type (e.g. \"Erkenntnis\")")]
    pub(crate) decision_type: Option<String>,

    #[arg(long = "case-number", help = "Case number (e.g. \"G 123/2024\")")]
    pub(crate) case_number: Option<String>,

    #[arg(long, help = "ECLI identifier")]
    pub(crate) ecli: Option<String>,

    #[arg(long, help = "State (Bundesland)")]
    pub(crate) state: Option<String>,

    #[command(flatten)]
    pub(crate) dates: DateRangeArgs,
}

impl SearchPayloadArgs for SearchAtjdArgs {
    fn base(&self) -> &SearchBaseArgs {
        &self.base
    }

    fn extend_payload(&self, payload: &mut JsonMap) {
        insert_string(payload, "application", &self.application);
        insert_string(payload, "documentType", &self.document_type);
        insert_string(payload, "decisionType", &self.decision_type);
        insert_string(payload, "caseNumber", &self.case_number);
        insert_string(payload, "ecli", &self.ecli);
        insert_string(payload, "state", &self.state);
        self.dates.insert_into(payload);
    }
}
