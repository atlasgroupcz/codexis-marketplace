use clap::Args;

use crate::sources::common::{insert_string, DateRangeArgs, JsonMap, SearchBaseArgs, SearchPayloadArgs};

pub(crate) const SEARCH_SKNUS_HELP: &str = r#"Example:
  cdx-sk search SKNUS --query "ústavné právo" --court NSSR --limit 5
  cdx-sk search SKNUS --decision-type Uznesenie --date-from 2024-01-01"#;

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchSknusArgs {
    #[command(flatten)]
    pub(crate) base: SearchBaseArgs,

    #[arg(long, help = "Court code (e.g. \"NSSR\")")]
    pub(crate) court: Option<String>,

    #[arg(long = "court-name", help = "Court name")]
    pub(crate) court_name: Option<String>,

    #[arg(long = "typ-rozhodnutia", help = "Decision type \u{2014} Slovak alias for --decision-type")]
    pub(crate) typ_rozhodnutia: Option<String>,

    #[arg(long = "decision-type", help = "Decision type \u{2014} English alias for --typ-rozhodnutia")]
    pub(crate) decision_type: Option<String>,

    #[arg(long = "case-number", help = "Case number \u{2014} alias for --spisova-znacka")]
    pub(crate) case_number: Option<String>,

    #[arg(long = "spisova-znacka", help = "Case file number \u{2014} alias for --case-number")]
    pub(crate) spisova_znacka: Option<String>,

    #[arg(long, help = "ECLI identifier")]
    pub(crate) ecli: Option<String>,

    #[command(flatten)]
    pub(crate) dates: DateRangeArgs,
}

impl SearchPayloadArgs for SearchSknusArgs {
    fn base(&self) -> &SearchBaseArgs {
        &self.base
    }

    fn extend_payload(&self, payload: &mut JsonMap) {
        insert_string(payload, "court", &self.court);
        insert_string(payload, "courtName", &self.court_name);
        let decision_type = self
            .typ_rozhodnutia
            .as_ref()
            .or(self.decision_type.as_ref())
            .cloned();
        insert_string(payload, "typRozhodnutia", &decision_type);
        let case_number = self
            .case_number
            .as_ref()
            .or(self.spisova_znacka.as_ref())
            .cloned();
        insert_string(payload, "caseNumber", &case_number);
        insert_string(payload, "ecli", &self.ecli);
        self.dates.insert_into(payload);
    }
}
