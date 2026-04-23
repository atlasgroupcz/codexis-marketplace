use clap::Args;

use crate::sources::common::{
    insert_string_array, IssuedDateArgs, JsonMap, SearchBaseArgs, SearchFacetArgs,
    SearchPayloadArgs, StandardSortArgs,
};

pub(crate) const SEARCH_ES_HELP: &str = r#"Example:
  cdx-cli search ES --query "ochrana spotřebitele" --type Rozsudek --limit 5"#;

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchEsArgs {
    #[command(flatten)]
    pub(crate) base: SearchBaseArgs,

    #[command(flatten)]
    pub(crate) sort: StandardSortArgs,

    #[command(flatten)]
    pub(crate) facets: SearchFacetArgs,

    #[arg(
        long = "type",
        visible_alias = "typ",
        help = "Decision type (repeatable)",
        value_name = "VALUE"
    )]
    pub(crate) types: Vec<String>,

    #[command(flatten)]
    pub(crate) issued: IssuedDateArgs,
}

impl SearchPayloadArgs for SearchEsArgs {
    fn base(&self) -> &SearchBaseArgs {
        &self.base
    }

    fn extend_payload(&self, payload: &mut JsonMap) {
        self.sort.insert_into(payload);
        insert_string_array(payload, "typ", &self.types);
        self.issued.insert_into(payload);
    }

    fn facet_mode(&self) -> crate::core::http::SearchFacetMode {
        self.facets.mode()
    }
}
