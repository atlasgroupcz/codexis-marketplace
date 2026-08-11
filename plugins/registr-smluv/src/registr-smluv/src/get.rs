use clap::Args;

const GET_HELP: &str = "\
Contract document resource:
  registr-smluv://doc/<DOC_ID>/meta   Full contract metadata (raw record from smlouvy.gov.cz)

Resolve a display ID:
  registr-smluv://resolve/<ID>

Examples:
  registr-smluv get registr-smluv://doc/CZRS34840741/meta
  registr-smluv get registr-smluv://resolve/CZRS34840741";

#[derive(Args, Debug, Clone)]
#[command(after_help = GET_HELP)]
pub(crate) struct GetArgs {
    #[arg(
        value_name = "REGISTR_SMLUV_URL",
        help = "Resource URL in registr-smluv:// protocol"
    )]
    pub(crate) resource: String,

    #[arg(long, help = "Print the translated curl command and exit")]
    pub(crate) dry_run: bool,
}
