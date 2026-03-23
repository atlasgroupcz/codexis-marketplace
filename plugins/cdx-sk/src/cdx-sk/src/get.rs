use clap::Args;

const GET_HELP: &str = "\
Common document resources:
  cdx-sk://doc/<DOC_ID>/meta[?includeAllAssets=true]
  cdx-sk://doc/<DOC_ID>/text[?timecutId=X&part=Y&part=Z]
  cdx-sk://doc/<DOC_ID>/toc[?timecutId=X]
  cdx-sk://doc/<DOC_ID>/parts[?timecutId=X&search=S&offset=N&limit=N]
  cdx-sk://doc/<DOC_ID>/versions
  cdx-sk://doc/<DOC_ID>/related?type=T[&offset=N&limit=N&sort=S&order=O]
  cdx-sk://doc/<DOC_ID>/related/counts
  cdx-sk://doc/<DOC_ID>/attachment/<FILE>

Direct Slovak law fetches (SKEZ only):
  cdx-sk://law/SK/<NUM>/<YEAR>
  cdx-sk://law/SK/<NUM>/<YEAR>/meta[?includeAllAssets=true]
  cdx-sk://law/SK/<NUM>/<YEAR>/text[?timecutId=X&part=Y]
  cdx-sk://law/SK/<NUM>/<YEAR>/toc[?timecutId=X]
  cdx-sk://law/SK/<NUM>/<YEAR>/parts[?search=X&offset=N&limit=N]
  cdx-sk://law/SK/<NUM>/<YEAR>/versions
  cdx-sk://law/SK/<NUM>/<YEAR>/related?type=T[&offset=N&limit=N&sort=S&order=O]
  cdx-sk://law/SK/<NUM>/<YEAR>/related/counts

Resolve display ID:
  cdx-sk://resolve/<ID>

Relation types: IMPLEMENTING, AMENDS, AMENDED_BY, REPEALS, REFERENCING_DECISION, REFERENCED_LAW

Examples:
  cdx-sk get cdx-sk://doc/SKEZ1234/meta
  cdx-sk get cdx-sk://doc/SKVS5678/text
  cdx-sk get cdx-sk://law/SK/40/1964
  cdx-sk get cdx-sk://law/SK/40/1964/text
  cdx-sk get 'cdx-sk://doc/SKEZ1/text?part=paragraf-1&part=paragraf-2'
  cdx-sk get 'cdx-sk://doc/SKEZ1/parts?search=paragraf-23'
  cdx-sk get 'cdx-sk://doc/SKEZ1/related?type=IMPLEMENTING&limit=10'";

#[derive(Args, Debug, Clone)]
#[command(after_help = GET_HELP)]
pub(crate) struct GetArgs {
    #[arg(value_name = "CDX_SK_URL", help = "Resource URL in cdx-sk:// protocol")]
    pub(crate) resource: String,

    #[arg(long, help = "Print the translated curl command and exit")]
    pub(crate) dry_run: bool,
}
