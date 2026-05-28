use clap::Args;

const GET_HELP: &str = "\
Common document resources:
  cdx-at://doc/<DOC_ID>/meta
  cdx-at://doc/<DOC_ID>/text
  cdx-at://doc/<DOC_ID>/attachment/<FILE>

Consolidated law / point-in-time (History domain):
  cdx-at://law/<LAW_KEY>                            LAW_KEY = Gesetzesnummer or eli~<stem>
  cdx-at://law/<LAW_KEY>/at?date=<YYYY-MM-DD>
  cdx-at://law/<LAW_KEY>/versions
  cdx-at://law/<LAW_KEY>/toc?all=true
  cdx-at://law/<LAW_KEY>/parts
  cdx-at://law/<LAW_KEY>/paragraph/<PARA>/versions
  cdx-at://law/<LAW_KEY>/text?part=<PARA>

Natural-key resolvers (-> {docId, domain, url}):
  cdx-at://bgbl/<TYPE>/<YEAR>/<NR>                  Bundesrecht gazette (TYPE = I|II|III)
  cdx-at://lgbl/<STATE>/<YEAR>/<NR>                 Landesrecht gazette (STATE e.g. WI)
  cdx-at://by-ecli/<ECLI>                           Judikatur by ECLI
  cdx-at://by-document-number/<DOMAIN>/<DN>         DOMAIN = history|judikatur|bundesrecht|landesrecht

Examples:
  cdx-at get cdx-at://doc/ATBR1234/meta
  cdx-at get cdx-at://doc/ATJD5678/text
  cdx-at get cdx-at://law/10008147/at?date=2024-06-01
  cdx-at get cdx-at://bgbl/I/2026/62";

#[derive(Args, Debug, Clone)]
#[command(after_help = GET_HELP)]
pub(crate) struct GetArgs {
    #[arg(value_name = "CDX_AT_URL", help = "Resource URL in cdx-at:// protocol")]
    pub(crate) resource: String,

    #[arg(long, help = "Print the translated curl command and exit")]
    pub(crate) dry_run: bool,
}
