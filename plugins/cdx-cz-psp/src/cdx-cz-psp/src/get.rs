use clap::Args;

const GET_HELP: &str = "\
Common document resources:
  cdx-cz-psp://doc/<DOC_ID>/meta
  cdx-cz-psp://doc/<DOC_ID>/text
  cdx-cz-psp://doc/<DOC_ID>/attachment/<FILE>

Resolve display ID:
  cdx-cz-psp://resolve/<ID>

Examples:
  cdx-cz-psp get cdx-cz-psp://doc/CZPSPDOK1234/meta
  cdx-cz-psp get cdx-cz-psp://doc/CZPSPPRE5678/text
  cdx-cz-psp get cdx-cz-psp://resolve/CZPSPDOK1234";

#[derive(Args, Debug, Clone)]
#[command(after_help = GET_HELP)]
pub(crate) struct GetArgs {
    #[arg(value_name = "CDX_CZ_PSP_URL", help = "Resource URL in cdx-cz-psp:// protocol")]
    pub(crate) resource: String,

    #[arg(long, help = "Print the translated curl command and exit")]
    pub(crate) dry_run: bool,
}
