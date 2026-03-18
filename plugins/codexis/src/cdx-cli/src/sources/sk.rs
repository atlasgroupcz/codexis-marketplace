use clap::Args;

use crate::sources::common::{
    insert_string_array, EffectiveDateArgs, IssuedDateArgs, JsonMap, SearchBaseArgs,
    SearchPayloadArgs, StandardSortArgs,
};

pub(crate) const SEARCH_SK_HELP: &str = r#"Search Slovak legislation and regulations.

Key flags:
  --query STRING
  --sort RELEVANCE|DATE|NAME  default: RELEVANCE
  --sort-order ASC|DESC       default: DESC
  --type VALUE           repeatable
  --issued-from / --issued-to
  --effective-from / --effective-to

Example:
  cdx-cli search SK --query "občiansky zákonník" --type Zákon --limit 5"#;

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchSkArgs {
    #[command(flatten)]
    pub(crate) base: SearchBaseArgs,

    #[command(flatten)]
    pub(crate) sort: StandardSortArgs,

    #[arg(
        long = "type",
        visible_alias = "typ",
        help = "Document type",
        value_name = "VALUE"
    )]
    pub(crate) types: Vec<String>,

    #[command(flatten)]
    pub(crate) issued: IssuedDateArgs,

    #[command(flatten)]
    pub(crate) effective: EffectiveDateArgs,
}

impl SearchPayloadArgs for SearchSkArgs {
    fn base(&self) -> &SearchBaseArgs {
        &self.base
    }

    fn extend_payload(&self, payload: &mut JsonMap) {
        self.sort.insert_into(payload);
        insert_string_array(payload, "typ", &self.types);
        self.issued.insert_into(payload);
        self.effective.insert_into(payload);
    }

    fn has_source_filters(&self) -> bool {
        self.sort.is_present()
            || !self.types.is_empty()
            || self.issued.is_present()
            || self.effective.is_present()
    }
}
