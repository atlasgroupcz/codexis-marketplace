use clap::Args;

const GET_HELP: &str = "\
Common document resources:
  cdx://doc/<DOC_ID>/meta
  cdx://doc/<DOC_ID>/toc
  cdx://doc/<DOC_ID>/text
  cdx://doc/<DOC_ID>/versions          supported for CR documents only
  cdx://doc/<DOC_ID>/related
  cdx://doc/<DOC_ID>/related/counts

Direct Czech law fetches:
  cdx://cz_law/<NUM>/<YEAR>/meta
  cdx://cz_law/<NUM>/<YEAR>/toc
  cdx://cz_law/<NUM>/<YEAR>/text
  cdx://cz_law/<NUM>/<YEAR>/versions
  cdx://cz_law/<NUM>/<YEAR>/related
  cdx://cz_law/<NUM>/<YEAR>/related/counts

Direct EU law fetches (TYPE is R regulation, L directive, D decision):
  cdx://eu_law/<TYPE>/<YEAR>/<NUM>/meta
  cdx://eu_law/<TYPE>/<YEAR>/<NUM>/toc
  cdx://eu_law/<TYPE>/<YEAR>/<NUM>/text[?part=PART]
  cdx://eu_law/<TYPE>/<YEAR>/<NUM>/versions
  cdx://eu_law/<TYPE>/<YEAR>/<NUM>/related[?type=TYPE&part=PART&offset=N&limit=N&sort=FIELD&order=ORDER]
  cdx://eu_law/<TYPE>/<YEAR>/<NUM>/related/counts[?part=PART]
  cdx://eu_law/by-celex/<CELEX>/...           same suffixes, when a CELEX is already at hand

Examples:
  cdx-cli get cdx://doc/CR10_2025_01_01/text
  cdx-cli get 'cdx://doc/CR26785/related?type=SOUVISEJICI_JUDIKATURA&limit=3'
  cdx-cli get cdx://cz_law/89/2012/meta
  cdx-cli get cdx://cz_law/89/2012/text?part=paragraf1
  cdx-cli get cdx://eu_law/R/2016/679/meta";

#[derive(Args, Debug, Clone)]
#[command(after_help = GET_HELP)]
pub(crate) struct GetArgs {
    #[arg(value_name = "CDX_URL", help = "Resource URL in cdx:// protocol")]
    pub(crate) resource: String,

    #[arg(long, help = "Print the translated curl command and exit")]
    pub(crate) dry_run: bool,
}
