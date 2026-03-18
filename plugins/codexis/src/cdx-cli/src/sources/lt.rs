use clap::Args;

use crate::sources::common::{
    IssuedDateArgs, JsonMap, SearchBaseArgs, SearchPayloadArgs, StandardSortArgs,
};

pub(crate) const SEARCH_LT_HELP: &str = r#"Search legal literature, practical guides, and articles.

Key flags:
  --query STRING
  --sort RELEVANCE|DATE|NAME  default: RELEVANCE
  --sort-order ASC|DESC       default: DESC
  --issued-from YYYY-MM-DD
  --issued-to YYYY-MM-DD

Example:
  cdx-cli search LT --query "odpovědnost za škodu" --limit 5"#;

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchLtArgs {
    #[command(flatten)]
    pub(crate) base: SearchBaseArgs,

    #[command(flatten)]
    pub(crate) sort: StandardSortArgs,

    #[command(flatten)]
    pub(crate) issued: IssuedDateArgs,
}

impl SearchPayloadArgs for SearchLtArgs {
    fn base(&self) -> &SearchBaseArgs {
        &self.base
    }

    fn extend_payload(&self, payload: &mut JsonMap) {
        self.sort.insert_into(payload);
        self.issued.insert_into(payload);
    }
}
