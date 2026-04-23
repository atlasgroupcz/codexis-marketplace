use clap::Args;

use crate::sources::common::{
    insert_string, IssuedDateArgs, JsonMap, SearchBaseArgs, SearchFacetArgs, SearchPayloadArgs,
    StandardSortArgs,
};

pub(crate) const SEARCH_COMMENT_HELP: &str = r#"Example:
  cdx-cli search COMMENT --query "nájem bytu" --related-doc CR26785 --limit 5"#;

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchCommentArgs {
    #[command(flatten)]
    pub(crate) base: SearchBaseArgs,

    #[command(flatten)]
    pub(crate) sort: StandardSortArgs,

    #[command(flatten)]
    pub(crate) facets: SearchFacetArgs,

    #[command(flatten)]
    pub(crate) issued: IssuedDateArgs,

    #[arg(
        long = "related-doc",
        visible_alias = "related-with-item",
        help = "Related CR docId"
    )]
    pub(crate) related_doc: Option<String>,

    #[arg(
        long = "related-part",
        visible_alias = "related-with-item-part",
        help = "Related CR part id"
    )]
    pub(crate) related_part: Option<String>,
}

impl SearchPayloadArgs for SearchCommentArgs {
    fn base(&self) -> &SearchBaseArgs {
        &self.base
    }

    fn extend_payload(&self, payload: &mut JsonMap) {
        self.sort.insert_into(payload);
        self.issued.insert_into(payload);
        insert_string(payload, "relatedWithItem", &self.related_doc);
        insert_string(payload, "relatedWithItemPart", &self.related_part);
    }

    fn facet_mode(&self) -> crate::core::http::SearchFacetMode {
        self.facets.mode()
    }
}
