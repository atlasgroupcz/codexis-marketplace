use clap::Args;

use crate::sources::common::{
    insert_string_array, JsonMap, SearchBaseArgs, SearchPayloadArgs, StandardSortArgs,
};

pub(crate) const SEARCH_VS_HELP: &str = r#"Search contract specimens and templates.

Key flags:
  --query STRING
  --sort RELEVANCE|NAME|DATE  default: RELEVANCE
  --sort-order ASC|DESC       default: DESC
  --author VALUE         repeatable
  --category VALUE       repeatable

Example:
  cdx-cli search VS --query "pracovní smlouva" --category "Pracovní právo (vzory dle zákoníku práce)" --limit 5"#;

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchVsArgs {
    #[command(flatten)]
    pub(crate) base: SearchBaseArgs,

    #[command(flatten)]
    pub(crate) sort: StandardSortArgs,

    #[arg(
        long = "author",
        visible_alias = "autor",
        help = "Author",
        value_name = "VALUE"
    )]
    pub(crate) authors: Vec<String>,

    #[arg(
        long = "category",
        visible_alias = "kategorie",
        help = "Category",
        value_name = "VALUE"
    )]
    pub(crate) categories: Vec<String>,
}

impl SearchPayloadArgs for SearchVsArgs {
    fn base(&self) -> &SearchBaseArgs {
        &self.base
    }

    fn extend_payload(&self, payload: &mut JsonMap) {
        self.sort.insert_into(payload);
        insert_string_array(payload, "autor", &self.authors);
        insert_string_array(payload, "kategorie", &self.categories);
    }

    fn has_source_filters(&self) -> bool {
        self.sort.is_present() || !self.authors.is_empty() || !self.categories.is_empty()
    }
}
