use clap::Args;

use crate::sources::common::{insert_string, insert_u64, DateRangeArgs, JsonMap, SearchBaseArgs, SearchPayloadArgs};

pub(crate) const SEARCH_CZPSPDOK_HELP: &str = r#"Example:
  cdx-cz-psp search CZPSPDOK --query "interpelace" --election-period 10 --limit 5
  cdx-cz-psp search CZPSPDOK --document-type "Písemná interpelace" --author "Fiala""#;

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchCzpspdokArgs {
    #[command(flatten)]
    pub(crate) base: SearchBaseArgs,

    #[arg(long = "document-type", help = "Document type (e.g. \"Písemná interpelace\", \"Zpráva a jiné\")")]
    pub(crate) document_type: Option<String>,

    #[arg(long = "election-period", help = "Election period number (e.g. 10)")]
    pub(crate) election_period: Option<u64>,

    #[arg(long = "state-class", help = "State class (approved, unapproved, lightgray, in_progress)")]
    pub(crate) state_class: Option<String>,

    #[arg(long = "press-number", help = "Print/press number")]
    pub(crate) press_number: Option<String>,

    #[arg(long, help = "Author name")]
    pub(crate) author: Option<String>,

    #[arg(long = "current-state", help = "Current state text")]
    pub(crate) current_state: Option<String>,

    #[arg(long, help = "Addressee (interpellations only)")]
    pub(crate) addressee: Option<String>,

    #[command(flatten)]
    pub(crate) dates: DateRangeArgs,
}

impl SearchPayloadArgs for SearchCzpspdokArgs {
    fn base(&self) -> &SearchBaseArgs {
        &self.base
    }

    fn extend_payload(&self, payload: &mut JsonMap) {
        insert_string(payload, "documentType", &self.document_type);
        insert_u64(payload, "electionPeriod", self.election_period);
        insert_string(payload, "stateClass", &self.state_class);
        insert_string(payload, "pressNumber", &self.press_number);
        insert_string(payload, "author", &self.author);
        insert_string(payload, "currentState", &self.current_state);
        insert_string(payload, "addressee", &self.addressee);
        insert_string(payload, "submissionDateFrom", &self.dates.date_from);
        insert_string(payload, "submissionDateTo", &self.dates.date_to);
    }
}
