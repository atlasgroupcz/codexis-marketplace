use clap::Args;

use crate::sources::common::{
    insert_string_array, IssuedDateArgs, JsonMap, SearchBaseArgs, SearchPayloadArgs,
    StandardSortArgs,
};

pub(crate) const SEARCH_JD_HELP: &str = r#"Search judicial decisions from Czech courts.

Key flags:
  --query STRING
  --sort RELEVANCE|CITEX|DATE|NAME  default: RELEVANCE
  --sort-order ASC|DESC             default: DESC
  --court VALUE          repeatable
  --city VALUE           repeatable
  --type VALUE           repeatable
  --issued-from YYYY-MM-DD
  --issued-to YYYY-MM-DD

Example:
  cdx-cli search JD --query "náhrada škody" --court "Nejvyšší soud" --type Rozsudek --limit 5"#;

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchJdArgs {
    #[command(flatten)]
    pub(crate) base: SearchBaseArgs,

    #[command(flatten)]
    pub(crate) sort: StandardSortArgs,

    #[arg(
        long = "court",
        visible_alias = "soud",
        help = "Court",
        value_name = "VALUE"
    )]
    pub(crate) courts: Vec<String>,

    #[arg(
        long = "city",
        visible_alias = "mesto",
        help = "City",
        value_name = "VALUE"
    )]
    pub(crate) cities: Vec<String>,

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

impl SearchPayloadArgs for SearchJdArgs {
    fn base(&self) -> &SearchBaseArgs {
        &self.base
    }

    fn extend_payload(&self, payload: &mut JsonMap) {
        self.sort.insert_into(payload);
        insert_string_array(payload, "soud", &self.courts);
        insert_string_array(payload, "mesto", &self.cities);
        insert_string_array(payload, "typ", &self.types);
        self.issued.insert_into(payload);
    }

    fn has_source_filters(&self) -> bool {
        self.sort.is_present()
            || !self.courts.is_empty()
            || !self.cities.is_empty()
            || !self.types.is_empty()
            || self.issued.is_present()
    }
}
