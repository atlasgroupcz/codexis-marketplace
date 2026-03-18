use clap::Args;

use crate::sources::common::{
    insert_string_array, IssuedDateArgs, JsonMap, SearchBaseArgs, SearchPayloadArgs,
    StandardSortArgs,
};

pub(crate) const SEARCH_ES_HELP: &str = r#"Search EU court decisions and ECHR-related rulings exposed by CODEXIS.

Key flags:
  --query STRING
  --sort RELEVANCE|DATE|NAME  default: RELEVANCE
  --sort-order ASC|DESC       default: DESC
  --type VALUE           repeatable
  --issued-from YYYY-MM-DD
  --issued-to YYYY-MM-DD

Example:
  cdx-cli search ES --query "ochrana spotřebitele" --type Rozsudek --limit 5"#;

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchEsArgs {
    #[command(flatten)]
    pub(crate) base: SearchBaseArgs,

    #[command(flatten)]
    pub(crate) sort: StandardSortArgs,

    #[arg(
        long = "type",
        visible_alias = "typ",
        help = "Decision type",
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

    fn has_source_filters(&self) -> bool {
        self.sort.is_present() || !self.types.is_empty() || self.issued.is_present()
    }
}
