use clap::Args;

const GET_HELP: &str = "\
Common document resources:
  cdx-nl://doc/<DOC_ID>/meta
  cdx-nl://doc/<DOC_ID>/text[?toestandId=X&part=Y&part=Z]   # NLBWB
  cdx-nl://doc/<DOC_ID>/text[?page=N]                       # NLUIT
  cdx-nl://doc/<DOC_ID>/toc[?toestandId=X]                  # NLBWB
  cdx-nl://doc/<DOC_ID>/parts[?toestandId=X&search=S&offset=N&limit=N]
  cdx-nl://doc/<NLBWB_ID>/versions[?limit=N&offset=N&from=YYYY-MM-DD&to=YYYY-MM-DD&includeAll=true]
  cdx-nl://doc/<NLBWB_ID>/at?date=YYYY-MM-DD
  cdx-nl://doc/<NLBWB_ID>/cited-by-decisions[?limit=N&offset=N]
  cdx-nl://doc/<DOC_ID>/related[?direction=in|out&offset=N&limit=N]
  cdx-nl://doc/<DOC_ID>/related/counts
  cdx-nl://doc/<NLBWB_ID>/citations?toestandId=X[&page=N]
  cdx-nl://doc/<DOC_ID>/attachment/<FILE>

Direct law fetch (NLBWB only):
  cdx-nl://law/NL/<BWB_ID>[/meta|text|toc|parts|versions|at|cited-by-decisions|related|citations]

Abbreviation resolver (NLBWB only):
  cdx-nl://afkorting/<ABBR>[/meta|text|toc|parts|versions|at|cited-by-decisions|related|related/counts|citations]

ECLI resolver (NLUIT only):
  cdx-nl://ecli/<ECLI>[/meta|text|toc|parts|related|related/counts|attachment/<FILE>]

Publication resolver (NLBWB only):
  cdx-nl://publication/<PUB_ID>/resolve

Universal ID resolver (display ID, BWB-id, or ECLI):
  cdx-nl://resolve/<ID>

Examples:
  cdx-nl get cdx-nl://doc/NLBWB1234/meta
  cdx-nl get cdx-nl://doc/NLUIT5678/text
  cdx-nl get cdx-nl://law/NL/BWBR0001827
  cdx-nl get cdx-nl://afkorting/BW/text
  cdx-nl get cdx-nl://ecli/ECLI:NL:HR:2024:1234
  cdx-nl get cdx-nl://publication/stb-2024-123/resolve
  cdx-nl get 'cdx-nl://doc/NLBWB1/citations?toestandId=X&page=3'
  cdx-nl get 'cdx-nl://doc/NLBWB1/at?date=2020-06-15'
  cdx-nl get cdx-nl://doc/NLBWB1/cited-by-decisions";

#[derive(Args, Debug, Clone)]
#[command(after_help = GET_HELP)]
pub(crate) struct GetArgs {
    #[arg(value_name = "CDX_NL_URL", help = "Resource URL in cdx-nl:// protocol")]
    pub(crate) resource: String,

    #[arg(long, help = "Print the translated curl command and exit")]
    pub(crate) dry_run: bool,
}
