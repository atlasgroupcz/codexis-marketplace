use clap::Args;

const GET_HELP: &str = "\
Device document resources:
  rzpro://doc/<DOC_ID>/meta           Full device record (kind, risk class, purpose, manufacturer, variants …)
  rzpro://doc/values/<FIELD>          Distinct values + counts (druh|riziko|stat)

Registered person/company:
  rzpro://osoba/<KEY>                 Person by IČO or Registrační číslo, with linked devices

Resolve a display ID:
  rzpro://resolve/<ID>

Examples:
  rzpro-cli get rzpro://doc/CZRZP00000019/meta
  rzpro-cli get rzpro://osoba/002276
  rzpro-cli get rzpro://doc/values/riziko";

#[derive(Args, Debug, Clone)]
#[command(after_help = GET_HELP)]
pub(crate) struct GetArgs {
    #[arg(value_name = "RZPRO_URL", help = "Resource URL in rzpro:// protocol")]
    pub(crate) resource: String,

    #[arg(long, help = "Print the translated curl command and exit")]
    pub(crate) dry_run: bool,
}
