use clap::Args;

const GET_HELP: &str = "\
Medicine document resources:
  leciva://doc/<DOC_ID>/meta          Full medicine record (add ?full=true for all raw columns)
  leciva://doc/<DOC_ID>/cena          Live price / reimbursement (proxied from SÚKL)
  leciva://doc/values/<FIELD>         Distinct values + counts (forma|vydej|stav|atc|typ|indikace)

Resolve a display ID:
  leciva://resolve/<ID>

Examples:
  leciva-cli get leciva://doc/CZSUKL0000009/meta
  leciva-cli get \"leciva://doc/CZSUKL0000009/meta?full=true\"
  leciva-cli get leciva://doc/CZSUKL0000009/cena
  leciva-cli get leciva://doc/values/forma";

#[derive(Args, Debug, Clone)]
#[command(after_help = GET_HELP)]
pub(crate) struct GetArgs {
    #[arg(value_name = "LECIVA_URL", help = "Resource URL in leciva:// protocol")]
    pub(crate) resource: String,

    #[arg(long, help = "Print the translated curl command and exit")]
    pub(crate) dry_run: bool,
}
