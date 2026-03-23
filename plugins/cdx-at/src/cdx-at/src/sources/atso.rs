use clap::Args;

use crate::sources::common::{insert_string, DateRangeArgs, JsonMap, SearchBaseArgs, SearchPayloadArgs};

pub(crate) const SEARCH_ATSO_HELP: &str = r#"Example:
  cdx-at search ATSO --query "Rundschreiben" --application Erlaesse --limit 5"#;

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchAtsoArgs {
    #[command(flatten)]
    pub(crate) base: SearchBaseArgs,

    #[arg(long, help = "Application (e.g. \"Erlaesse\")")]
    pub(crate) application: Option<String>,

    #[arg(long = "document-type", help = "Document type")]
    pub(crate) document_type: Option<String>,

    #[arg(long, help = "Ministry")]
    pub(crate) ministry: Option<String>,

    #[command(flatten)]
    pub(crate) dates: DateRangeArgs,
}

impl SearchPayloadArgs for SearchAtsoArgs {
    fn base(&self) -> &SearchBaseArgs {
        &self.base
    }

    fn extend_payload(&self, payload: &mut JsonMap) {
        insert_string(payload, "application", &self.application);
        insert_string(payload, "documentType", &self.document_type);
        insert_string(payload, "ministry", &self.ministry);
        self.dates.insert_into(payload);
    }
}
