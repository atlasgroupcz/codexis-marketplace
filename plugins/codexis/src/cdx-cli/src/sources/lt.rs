use clap::Args;

use crate::sources::common::{
    IssuedDateArgs, JsonMap, SearchBaseArgs, SearchFacetArgs, SearchPayloadArgs, StandardSortArgs,
};

pub(crate) const SEARCH_LT_HELP: &str = r#"Example:
  cdx-cli search LT --query "odpovědnost za škodu" --limit 5"#;

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchLtArgs {
    #[command(flatten)]
    pub(crate) base: SearchBaseArgs,

    #[command(flatten)]
    pub(crate) sort: StandardSortArgs,

    #[command(flatten)]
    pub(crate) facets: SearchFacetArgs,

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

    fn facet_mode(&self) -> crate::core::http::SearchFacetMode {
        self.facets.mode()
    }
}
