use clap::Args;

use crate::sources::common::{insert_string, IssuedDateArgs, JsonMap, SearchBaseArgs, SearchPayloadArgs};

pub(crate) const SEARCH_SKEZ_HELP: &str = r#"Example:
  cdx-sk search SKEZ --query "občiansky zákonník" --limit 5
  cdx-sk search SKEZ --doc-number "40/1964 Zb."
  cdx-sk search SKEZ --valid-at 2026-01-01"#;

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchSkezArgs {
    #[command(flatten)]
    pub(crate) base: SearchBaseArgs,

    #[arg(long = "doc-number", help = "Document number (e.g. \"40/1964 Zb.\")")]
    pub(crate) doc_number: Option<String>,

    #[arg(long, help = "Document type (e.g. \"Zákon\")")]
    pub(crate) typ: Option<String>,

    #[arg(long = "valid-at", help = "Validity date (YYYY-MM-DD)")]
    pub(crate) valid_at: Option<String>,

    #[command(flatten)]
    pub(crate) issued: IssuedDateArgs,
}

impl SearchPayloadArgs for SearchSkezArgs {
    fn base(&self) -> &SearchBaseArgs {
        &self.base
    }

    fn extend_payload(&self, payload: &mut JsonMap) {
        insert_string(payload, "docNumber", &self.doc_number);
        insert_string(payload, "typ", &self.typ);
        insert_string(payload, "validAt", &self.valid_at);
        self.issued.insert_into(payload);
    }
}
