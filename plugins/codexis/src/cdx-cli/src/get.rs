use clap::Args;

#[derive(Args, Debug, Clone)]
pub(crate) struct GetArgs {
    #[arg(value_name = "CDX_URL", help = "Resource URL in cdx:// protocol")]
    pub(crate) resource: String,

    #[arg(long, help = "Print the translated curl command and exit")]
    pub(crate) dry_run: bool,
}
