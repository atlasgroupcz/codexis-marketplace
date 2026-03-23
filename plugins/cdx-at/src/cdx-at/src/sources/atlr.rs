use clap::Args;

use crate::sources::common::{insert_string, DateRangeArgs, JsonMap, SearchBaseArgs, SearchPayloadArgs};

pub(crate) const SEARCH_ATLR_HELP: &str = r#"Example:
  cdx-at search ATLR --query "Landesgesetz" --state Tirol --limit 5"#;

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchAtlrArgs {
    #[command(flatten)]
    pub(crate) base: SearchBaseArgs,

    #[arg(long = "document-type", help = "Document type")]
    pub(crate) document_type: Option<String>,

    #[arg(long, help = "State (Bundesland, e.g. \"Tirol\")")]
    pub(crate) state: Option<String>,

    #[arg(long = "gazette-number", help = "Gazette number (LGBl number)")]
    pub(crate) gazette_number: Option<String>,

    #[command(flatten)]
    pub(crate) dates: DateRangeArgs,
}

impl SearchPayloadArgs for SearchAtlrArgs {
    fn base(&self) -> &SearchBaseArgs {
        &self.base
    }

    fn extend_payload(&self, payload: &mut JsonMap) {
        insert_string(payload, "documentType", &self.document_type);
        insert_string(payload, "state", &self.state);
        insert_string(payload, "gazetteNumber", &self.gazette_number);
        self.dates.insert_into(payload);
    }
}
