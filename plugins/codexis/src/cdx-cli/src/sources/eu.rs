use clap::Args;

use crate::sources::common::{
    insert_string_array, ApprovedDateArgs, EffectiveDateArgs, IssuedDateArgs, JsonMap,
    SearchBaseArgs, SearchFacetArgs, SearchPayloadArgs, StandardSortArgs,
};

pub(crate) const SEARCH_EU_HELP: &str = r#"Example:
  cdx-cli search EU --query GDPR --type Nařízení --series L --limit 5"#;

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchEuArgs {
    #[command(flatten)]
    pub(crate) base: SearchBaseArgs,

    #[command(flatten)]
    pub(crate) sort: StandardSortArgs,

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
        long = "source",
        visible_alias = "zdroj",
        help = "Source (repeatable)",
        value_name = "VALUE"
    )]
    pub(crate) sources: Vec<String>,

    #[arg(
        long = "series",
        visible_alias = "zdroj-uveu",
        help = "Official journal series (repeatable)",
        value_name = "VALUE"
    )]
    pub(crate) series: Vec<String>,

    #[arg(
        long = "author",
        visible_alias = "autor",
        help = "Author (repeatable)",
        value_name = "VALUE"
    )]
    pub(crate) authors: Vec<String>,

    #[arg(
        long = "domain",
        visible_alias = "oblast",
        help = "Domain (repeatable)",
        value_name = "VALUE"
    )]
    pub(crate) domains: Vec<String>,

    #[command(flatten)]
    pub(crate) issued: IssuedDateArgs,

    #[command(flatten)]
    pub(crate) approved: ApprovedDateArgs,

    #[command(flatten)]
    pub(crate) effective: EffectiveDateArgs,
}

impl SearchPayloadArgs for SearchEuArgs {
    fn base(&self) -> &SearchBaseArgs {
        &self.base
    }

    fn extend_payload(&self, payload: &mut JsonMap) {
        self.sort.insert_into(payload);
        insert_string_array(payload, "typ", &self.types);
        insert_string_array(payload, "zdroj", &self.sources);
        insert_string_array(payload, "zdrojUveu", &self.series);
        insert_string_array(payload, "autor", &self.authors);
        insert_string_array(payload, "oblast", &self.domains);
        self.issued.insert_into(payload);
        self.approved.insert_into(payload);
        self.effective.insert_into(payload);
    }

    fn facet_mode(&self) -> crate::core::http::SearchFacetMode {
        self.facets.mode()
    }
}
