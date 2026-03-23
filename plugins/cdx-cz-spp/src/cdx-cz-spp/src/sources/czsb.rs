use clap::Args;

use crate::sources::common::{insert_string, JsonMap, SearchBaseArgs, SearchPayloadArgs};

pub(crate) const SEARCH_CZSB_HELP: &str = r#"Example:
  cdx-cz-spp search CZSB --query "vyhláška" --publikujici "Praha" --limit 5
  cdx-cz-spp search CZSB --hlavni-typ pp --platnost Platné"#;

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchCzsbArgs {
    #[command(flatten)]
    pub(crate) base: SearchBaseArgs,

    #[arg(long = "druh-predpisu", help = "Document type (e.g. \"Obecně závazná vyhláška\")")]
    pub(crate) druh_predpisu: Option<String>,
    #[arg(long, help = "Publisher/municipality name")]
    pub(crate) publikujici: Option<String>,
    #[arg(long = "oblast-pravni-upravy", help = "Legal area (e.g. \"Odpady\")")]
    pub(crate) oblast_pravni_upravy: Option<String>,
    #[arg(long, help = "Validity status (e.g. \"Platné\")")]
    pub(crate) platnost: Option<String>,
    #[arg(long = "cislo-predpisu", help = "Regulation number (e.g. \"1/2026\")")]
    pub(crate) cislo_predpisu: Option<String>,
    #[arg(long, help = "Municipality ICO")]
    pub(crate) ico: Option<String>,
    #[arg(long = "zakonne-zmocneni", help = "Legal authorization")]
    pub(crate) zakonne_zmocneni: Option<String>,
    #[arg(long = "hlavni-typ", help = "Main type: \"pp\" (legal regs) or \"oa\" (other acts)")]
    pub(crate) hlavni_typ: Option<String>,
    #[arg(long = "datum-vydani-from", help = "Issue date from (YYYY-MM-DD)")]
    pub(crate) datum_vydani_from: Option<String>,
    #[arg(long = "datum-vydani-to", help = "Issue date to (YYYY-MM-DD)")]
    pub(crate) datum_vydani_to: Option<String>,
    #[arg(long = "datum-zverejneni-from", help = "Publication date from (YYYY-MM-DD)")]
    pub(crate) datum_zverejneni_from: Option<String>,
    #[arg(long = "datum-zverejneni-to", help = "Publication date to (YYYY-MM-DD)")]
    pub(crate) datum_zverejneni_to: Option<String>,
    #[arg(long = "datum-ucinnosti-from", help = "Effective date from (YYYY-MM-DD)")]
    pub(crate) datum_ucinnosti_from: Option<String>,
    #[arg(long = "datum-ucinnosti-to", help = "Effective date to (YYYY-MM-DD)")]
    pub(crate) datum_ucinnosti_to: Option<String>,
}

impl SearchPayloadArgs for SearchCzsbArgs {
    fn base(&self) -> &SearchBaseArgs {
        &self.base
    }

    fn extend_payload(&self, payload: &mut JsonMap) {
        insert_string(payload, "druhPredpisu", &self.druh_predpisu);
        insert_string(payload, "publikujici", &self.publikujici);
        insert_string(payload, "oblastPravniUpravy", &self.oblast_pravni_upravy);
        insert_string(payload, "platnost", &self.platnost);
        insert_string(payload, "cisloPredpisu", &self.cislo_predpisu);
        insert_string(payload, "ico", &self.ico);
        insert_string(payload, "zakonneZmocneni", &self.zakonne_zmocneni);
        insert_string(payload, "hlavniTyp", &self.hlavni_typ);
        insert_string(payload, "datumVydaniFrom", &self.datum_vydani_from);
        insert_string(payload, "datumVydaniTo", &self.datum_vydani_to);
        insert_string(payload, "datumZverejneniFrom", &self.datum_zverejneni_from);
        insert_string(payload, "datumZverejneniTo", &self.datum_zverejneni_to);
        insert_string(payload, "datumUcinnostiFrom", &self.datum_ucinnosti_from);
        insert_string(payload, "datumUcinnostiTo", &self.datum_ucinnosti_to);
    }
}
