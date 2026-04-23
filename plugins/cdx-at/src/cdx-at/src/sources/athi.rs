use clap::Args;

use crate::sources::common::{insert_string, DateRangeArgs, JsonMap, SearchBaseArgs, SearchPayloadArgs};

pub(crate) const SEARCH_ATHI_HELP: &str = r#"Example:
  cdx-at search ATHI --abbreviation ASVG --limit 5
  cdx-at search ATHI --document-type BG"#;

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchAthiArgs {
    #[command(flatten)]
    pub(crate) base: SearchBaseArgs,

    #[arg(long, help = "Law abbreviation (e.g. \"ASVG\")")]
    pub(crate) abbreviation: Option<String>,

    #[arg(long = "document-type", help = "Document type (e.g. \"BG\")")]
    pub(crate) document_type: Option<String>,

    #[command(flatten)]
    pub(crate) dates: DateRangeArgs,
}

impl SearchPayloadArgs for SearchAthiArgs {
    fn base(&self) -> &SearchBaseArgs {
        &self.base
    }

    fn extend_payload(&self, payload: &mut JsonMap) {
        insert_string(payload, "abbreviation", &self.abbreviation);
        insert_string(payload, "documentType", &self.document_type);
        self.dates.insert_into(payload);
    }
}
