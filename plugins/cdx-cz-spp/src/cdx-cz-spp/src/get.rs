use clap::Args;

const GET_HELP: &str = "\
Common document resources:
  cdx-cz-spp://doc/<DOC_ID>/meta[?includeAllAssets=true]
  cdx-cz-spp://doc/<DOC_ID>/text[?file=F&timecutId=X&part=Y&part=Z]
  cdx-cz-spp://doc/<DOC_ID>/toc[?file=F&timecutId=X]
  cdx-cz-spp://doc/<DOC_ID>/parts[?file=F&timecutId=X&search=S&offset=N&limit=N]
  cdx-cz-spp://doc/<DOC_ID>/versions
  cdx-cz-spp://doc/<DOC_ID>/related?type=T[&offset=N&limit=N&sort=S&order=O]
  cdx-cz-spp://doc/<DOC_ID>/related/counts
  cdx-cz-spp://doc/<DOC_ID>/attachment/<FILE>

Resolve display ID:
  cdx-cz-spp://resolve/<ID>

Relation types: IMPLEMENTING, AMENDS, AMENDED_BY, REPEALS, REFERENCING_DECISION, REFERENCED_LAW

Examples:
  cdx-cz-spp get cdx-cz-spp://doc/CZSB1234/meta
  cdx-cz-spp get cdx-cz-spp://doc/CZSB5678/text
  cdx-cz-spp get 'cdx-cz-spp://doc/CZSB1/text?part=paragraf-1&part=paragraf-2'
  cdx-cz-spp get 'cdx-cz-spp://doc/CZSB1/parts?search=paragraf-23'
  cdx-cz-spp get 'cdx-cz-spp://doc/CZSB1/related?type=IMPLEMENTING&limit=10'";

#[derive(Args, Debug, Clone)]
#[command(after_help = GET_HELP)]
pub(crate) struct GetArgs {
    #[arg(value_name = "CDX_CZ_SPP_URL", help = "Resource URL in cdx-cz-spp:// protocol")]
    pub(crate) resource: String,

    #[arg(long, help = "Print the translated curl command and exit")]
    pub(crate) dry_run: bool,
}
