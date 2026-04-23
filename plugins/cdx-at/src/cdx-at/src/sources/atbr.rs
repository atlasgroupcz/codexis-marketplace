use clap::Args;

use crate::sources::common::{insert_string, DateRangeArgs, JsonMap, SearchBaseArgs, SearchPayloadArgs};

pub(crate) const SEARCH_ATBR_HELP: &str = r#"Example:
  cdx-at search ATBR --query "Verordnung" --limit 5
  cdx-at search ATBR --document-type Verordnung --part Teil2"#;

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchAtbrArgs {
    #[command(flatten)]
    pub(crate) base: SearchBaseArgs,

    #[arg(long = "document-type", help = "Document type (e.g. \"Verordnung\")")]
    pub(crate) document_type: Option<String>,

    #[arg(long, help = "Part (e.g. \"Teil2\")")]
    pub(crate) part: Option<String>,

    #[arg(long = "gazette-number", help = "Gazette number (BGBl number)")]
    pub(crate) gazette_number: Option<String>,

    #[command(flatten)]
    pub(crate) dates: DateRangeArgs,
}

impl SearchPayloadArgs for SearchAtbrArgs {
    fn base(&self) -> &SearchBaseArgs {
        &self.base
    }

    fn extend_payload(&self, payload: &mut JsonMap) {
        insert_string(payload, "documentType", &self.document_type);
        insert_string(payload, "part", &self.part);
        insert_string(payload, "gazetteNumber", &self.gazette_number);
        self.dates.insert_into(payload);
    }
}
