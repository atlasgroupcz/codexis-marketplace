use clap::Args;

use crate::sources::common::{
    insert_bool, insert_string, insert_string_array, ApprovedDateArgs, ChangedDateArgs, CrSortArgs,
    EffectiveDateArgs, IssuedDateArgs, JsonMap, SearchBaseArgs, SearchFacetArgs, SearchPayloadArgs,
};

pub(crate) const SEARCH_CR_HELP: &str = r#"Example:
  cdx-cli search CR --query "občanský zákoník" --type Zákon --current --limit 5"#;

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchCrArgs {
    #[command(flatten)]
    pub(crate) base: SearchBaseArgs,

    #[command(flatten)]
    pub(crate) sort: CrSortArgs,

    #[command(flatten)]
    pub(crate) facets: SearchFacetArgs,

    #[arg(
        long = "type",
        visible_alias = "typ",
        help = "Document type (repeatable)",
        value_name = "VALUE"
    )]
    pub(crate) types: Vec<String>,

    #[arg(
        long = "author",
        visible_alias = "autor",
        help = "Author (repeatable)",
        value_name = "VALUE"
    )]
    pub(crate) authors: Vec<String>,

    #[arg(
        long = "current",
        visible_alias = "valid-now",
        help = "Only currently valid documents"
    )]
    pub(crate) current: bool,

    #[arg(long = "valid-at", help = "Valid at date (YYYY-MM-DD)")]
    pub(crate) valid_at: Option<String>,

    #[command(flatten)]
    pub(crate) issued: IssuedDateArgs,

    #[command(flatten)]
    pub(crate) effective: EffectiveDateArgs,

    #[command(flatten)]
    pub(crate) approved: ApprovedDateArgs,

    #[command(flatten)]
    pub(crate) changed: ChangedDateArgs,
}

impl SearchPayloadArgs for SearchCrArgs {
    fn base(&self) -> &SearchBaseArgs {
        &self.base
    }

    fn extend_payload(&self, payload: &mut JsonMap) {
        self.sort.insert_into(payload);
        insert_string_array(payload, "typ", &self.types);
        insert_string_array(payload, "autor", &self.authors);
        insert_bool(payload, "validNow", self.current);
        insert_string(payload, "validAt", &self.valid_at);
        self.issued.insert_into(payload);
        self.effective.insert_into(payload);
        self.approved.insert_into(payload);
        self.changed.insert_into(payload);
    }

    fn facet_mode(&self) -> crate::core::http::SearchFacetMode {
        self.facets.mode()
    }
}
