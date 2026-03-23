use clap::Args;

const GET_HELP: &str = "\
Common document resources:
  cdx-at://doc/<DOC_ID>/meta
  cdx-at://doc/<DOC_ID>/text
  cdx-at://doc/<DOC_ID>/attachment/<FILE>

Resolve display ID:
  cdx-at://resolve/<ID>

Examples:
  cdx-at get cdx-at://doc/ATBR1234/meta
  cdx-at get cdx-at://doc/ATJD5678/text
  cdx-at get cdx-at://resolve/ATBR1234";

#[derive(Args, Debug, Clone)]
#[command(after_help = GET_HELP)]
pub(crate) struct GetArgs {
    #[arg(value_name = "CDX_AT_URL", help = "Resource URL in cdx-at:// protocol")]
    pub(crate) resource: String,

    #[arg(long, help = "Print the translated curl command and exit")]
    pub(crate) dry_run: bool,
}
