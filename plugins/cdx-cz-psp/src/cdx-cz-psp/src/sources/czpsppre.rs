use clap::Args;

use crate::sources::common::{insert_string, insert_u64, DateRangeArgs, JsonMap, SearchBaseArgs, SearchPayloadArgs};

pub(crate) const SEARCH_CZPSPPRE_HELP: &str = r#"Example:
  cdx-cz-psp search CZPSPPRE --query "daně" --type "Vládní návrh zákona" --limit 5
  cdx-cz-psp search CZPSPPRE --election-period 10 --sbirka-number "459/2022 Sb.""#;

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchCzpsppreArgs {
    #[command(flatten)]
    pub(crate) base: SearchBaseArgs,

    #[arg(long = "type", help = "Document type (e.g. \"Vládní návrh zákona\", \"Poslanecký návrh zákona\")")]
    pub(crate) doc_type: Option<String>,

    #[arg(long = "election-period", help = "Election period number (e.g. 10)")]
    pub(crate) election_period: Option<u64>,

    #[arg(long = "state-class", help = "State class (approved, unapproved, lightgray, in_progress)")]
    pub(crate) state_class: Option<String>,

    #[arg(long = "press-number", help = "Print/press number")]
    pub(crate) press_number: Option<String>,

    #[arg(long, help = "Submitter (e.g. \"Vláda\")")]
    pub(crate) submitter: Option<String>,

    #[arg(long = "current-state", help = "Current state text")]
    pub(crate) current_state: Option<String>,

    #[arg(long = "sbirka-number", help = "Publication number in Sbírka zákonů (e.g. \"459/2022 Sb.\")")]
    pub(crate) sbirka_number: Option<String>,

    #[arg(long = "eurovoc-descriptor", help = "EUROVOC descriptor keyword")]
    pub(crate) eurovoc_descriptor: Option<String>,

    #[command(flatten)]
    pub(crate) dates: DateRangeArgs,
}

impl SearchPayloadArgs for SearchCzpsppreArgs {
    fn base(&self) -> &SearchBaseArgs {
        &self.base
    }

    fn extend_payload(&self, payload: &mut JsonMap) {
        insert_string(payload, "type", &self.doc_type);
        insert_u64(payload, "electionPeriod", self.election_period);
        insert_string(payload, "stateClass", &self.state_class);
        insert_string(payload, "pressNumber", &self.press_number);
        insert_string(payload, "submitter", &self.submitter);
        insert_string(payload, "currentState", &self.current_state);
        insert_string(payload, "sbirkaNumber", &self.sbirka_number);
        insert_string(payload, "eurovocDescriptor", &self.eurovoc_descriptor);
        insert_string(payload, "submissionDateFrom", &self.dates.date_from);
        insert_string(payload, "submissionDateTo", &self.dates.date_to);
    }
}
