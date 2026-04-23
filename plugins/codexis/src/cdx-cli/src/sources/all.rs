use clap::Args;

use crate::sources::common::{JsonMap, SearchBaseArgs, SearchPayloadArgs, StandardSortArgs};

pub(crate) const SEARCH_ALL_HELP: &str = r#"Use ALL for orientation when the relevant source is not clear yet.
Follow up with a source-specific search before citing or extracting.

Examples:
  cdx-cli search ALL --query "insolvence" --limit 5
  cdx-cli search ALL '{"query":"insolvence","limit":5}'"#;

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchAllArgs {
    #[command(flatten)]
    pub(crate) base: SearchBaseArgs,

    #[command(flatten)]
    pub(crate) sort: StandardSortArgs,
}

impl SearchPayloadArgs for SearchAllArgs {
    fn base(&self) -> &SearchBaseArgs {
        &self.base
    }

    fn extend_payload(&self, payload: &mut JsonMap) {
        self.sort.insert_into(payload);
    }
}
