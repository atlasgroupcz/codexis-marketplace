use clap::Args;

use crate::sources::common::{insert_string, DateRangeArgs, JsonMap, SearchBaseArgs, SearchPayloadArgs};

pub(crate) const SEARCH_SKVS_HELP: &str = r#"Example:
  cdx-sk search SKVS --query "náhrada škody" --court OSBA1 --limit 5
  cdx-sk search SKVS --decision-form Rozsudok --date-from 2024-01-01"#;

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchSkvsArgs {
    #[command(flatten)]
    pub(crate) base: SearchBaseArgs,

    #[arg(long, help = "Court code (e.g. \"OSBA1\")")]
    pub(crate) court: Option<String>,

    #[arg(long = "court-name", help = "Court name (e.g. \"Okresný súd Bratislava I\")")]
    pub(crate) court_name: Option<String>,

    #[arg(long, help = "Judge name")]
    pub(crate) judge: Option<String>,

    #[arg(long = "spisova-znacka", help = "Case file number")]
    pub(crate) spisova_znacka: Option<String>,

    #[arg(long = "decision-form", help = "Decision form (e.g. \"Rozsudok\")")]
    pub(crate) decision_form: Option<String>,

    #[arg(long = "decision-nature", help = "Decision nature")]
    pub(crate) decision_nature: Option<String>,

    #[arg(long, help = "ECLI identifier")]
    pub(crate) ecli: Option<String>,

    #[command(flatten)]
    pub(crate) dates: DateRangeArgs,
}

impl SearchPayloadArgs for SearchSkvsArgs {
    fn base(&self) -> &SearchBaseArgs {
        &self.base
    }

    fn extend_payload(&self, payload: &mut JsonMap) {
        insert_string(payload, "court", &self.court);
        insert_string(payload, "courtName", &self.court_name);
        insert_string(payload, "judge", &self.judge);
        insert_string(payload, "spisovaZnacka", &self.spisova_znacka);
        insert_string(payload, "decisionForm", &self.decision_form);
        insert_string(payload, "decisionNature", &self.decision_nature);
        insert_string(payload, "ecli", &self.ecli);
        self.dates.insert_into(payload);
    }
}
