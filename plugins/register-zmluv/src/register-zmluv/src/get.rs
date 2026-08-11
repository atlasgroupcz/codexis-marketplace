use clap::Args;

const GET_HELP: &str = "\
Contract document resource:
  register-zmluv://doc/<DOC_ID>/meta   Full contract metadata (raw record from crz.gov.sk)

Resolve a display ID:
  register-zmluv://resolve/<ID>

Examples:
  register-zmluv get register-zmluv://doc/SKCRZ123456/meta
  register-zmluv get register-zmluv://resolve/SKCRZ123456";

#[derive(Args, Debug, Clone)]
#[command(after_help = GET_HELP)]
pub(crate) struct GetArgs {
    #[arg(
        value_name = "REGISTER_ZMLUV_URL",
        help = "Resource URL in register-zmluv:// protocol"
    )]
    pub(crate) resource: String,

    #[arg(long, help = "Print the translated curl command and exit")]
    pub(crate) dry_run: bool,
}
