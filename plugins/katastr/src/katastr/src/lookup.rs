//! Builds ČÚZK REST API paths from named arguments.
//!
//! All pure, no network - deliberately. The model used to assemble these paths
//! from markdown, so a typo in a parameter name only surfaced as an error from
//! ČÚZK. Here the type system and the tests catch it.

/// Diagnostic paths. They take no arguments, so constants - but they belong
/// here for the same reason as the rest: one module owns the API's paths, or a
/// handful stay scattered across the callers.
pub const STAV_UCTU: &str = "/api/v1/AplikacniSluzby/StavUctu";
pub const PROVOZNI_INFORMACE: &str = "/api/v1/AplikacniSluzby/ProvozniInformace";
pub const AKTUALNOST_DAT: &str = "/api/v1/AplikacniSluzby/AktualnostDat";
pub const HEALTH: &str = "/api/v1/AplikacniSluzby/Health";

/// `puvod_ze` is the ZE source of record. The swagger marks it optional, but
/// the server rejects `TypParcely=PZE` without it: "The property is required
/// 'PuvodParcelyZE' when 'TypParcely' == 'PZE'" - verified live.
pub fn parcela_search(
    ku: u32,
    cislo: u32,
    poddeleni: Option<u32>,
    druh: u8,
    typ: &str,
    puvod_ze: Option<u8>,
) -> String {
    let mut p = format!(
        "/api/v1/Parcely/Vyhledani?KodKatastralnihoUzemi={ku}&TypParcely={typ}&DruhCislovaniParcely={druh}&KmenoveCisloParcely={cislo}"
    );
    if let Some(pod) = poddeleni {
        p.push_str(&format!("&PoddeleniCislaParcely={pod}"));
    }
    if let Some(zdroj) = puvod_ze {
        p.push_str(&format!("&PuvodParcelyZE={zdroj}"));
    }
    p
}

pub fn parcela_detail(id: &str) -> String {
    format!("/api/v1/Parcely/{id}")
}

pub fn stavba_search(cast_obce: u32, typ_stavby: u8, cislo_domovni: u32) -> String {
    format!(
        "/api/v1/Stavby/Vyhledani?KodCastiObce={cast_obce}&TypStavby={typ_stavby}&CisloDomovni={cislo_domovni}"
    )
}

pub fn stavba_detail(id: &str) -> String {
    format!("/api/v1/Stavby/{id}")
}

pub fn stavba_adresni_misto(kod: u32) -> String {
    format!("/api/v1/Stavby/AdresniMisto/{kod}")
}

pub fn jednotka_search(
    cast_obce: u32,
    typ_stavby: u8,
    cislo_domovni: u32,
    cislo_jednotky: u32,
) -> String {
    format!(
        "/api/v1/Jednotky/Vyhledani?KodCastiObce={cast_obce}&TypStavby={typ_stavby}&CisloDomovni={cislo_domovni}&CisloJednotky={cislo_jednotky}"
    )
}

pub fn jednotka_detail(id: &str) -> String {
    format!("/api/v1/Jednotky/{id}")
}

pub fn rizeni_search(typ: &str, cislo: u32, rok: u32, pracoviste: u32) -> String {
    format!(
        "/api/v1/Rizeni/Vyhledani?TypRizeni={typ}&Cislo={cislo}&Rok={rok}&KodPracoviste={pracoviste}"
    )
}

pub fn rizeni_detail(id: &str) -> String {
    format!("/api/v1/Rizeni/{id}")
}

pub fn rizeni_prijate_dne(typ: &str, pracoviste: u32, datum: &str) -> String {
    format!(
        "/api/v1/Rizeni/PrijateDne?TypRizeni={typ}&KodPracoviste={pracoviste}&DatumPrijeti={datum}"
    )
}

pub fn pravo_stavby_detail(id: &str) -> String {
    format!("/api/v1/PravaStavby/{id}")
}

pub fn pravo_stavby_parcela(id: &str) -> String {
    format!("/api/v1/PravaStavby/Parcela/{id}")
}

pub fn pravo_stavby_stavba(id: &str) -> String {
    format!("/api/v1/PravaStavby/Stavba/{id}")
}

pub fn sousedni_parcely(id: &str) -> String {
    format!("/api/v1/Parcely/SousedniParcely/{id}")
}

pub fn pracoviste_list() -> String {
    "/api/v1/CiselnikyISKN/Pracoviste".to_string()
}

pub fn pracoviste_detail(kod: u32) -> String {
    format!("/api/v1/CiselnikyISKN/Pracoviste/{kod}")
}

fn polygon_path(entity: &str, coords: &str) -> String {
    format!(
        "/api/v1/{entity}/Polygon?SeznamSouradnic={}",
        percent_encode(coords)
    )
}

/// `coords` is the point list from [`crate::geom::square_around`].
pub fn parcely_polygon(coords: &str) -> String {
    polygon_path("Parcely", coords)
}

pub fn stavby_polygon(coords: &str) -> String {
    polygon_path("Stavby", coords)
}

/// Percent-encoding for a query value.
pub fn percent_encode(s: &str) -> String {
    cdx_common::url::encode_query_value(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parcela_search_includes_all_four_identifying_parts() {
        let p = parcela_search(638790, 545, None, 2, "PKN", None);
        assert_eq!(
            p,
            "/api/v1/Parcely/Vyhledani?KodKatastralnihoUzemi=638790&TypParcely=PKN&DruhCislovaniParcely=2&KmenoveCisloParcely=545"
        );
    }

    #[test]
    fn parcela_search_appends_subdivision_when_present() {
        let p = parcela_search(638790, 2642, Some(9), 2, "PKN", None);
        assert!(p.ends_with("&PoddeleniCislaParcely=9"), "got: {p}");
    }

    #[test]
    fn pze_lookup_carries_the_source_the_server_demands() {
        let p = parcela_search(638790, 545, None, 1, "PZE", Some(4));
        assert!(p.contains("TypParcely=PZE"), "got: {p}");
        assert!(p.ends_with("&PuvodParcelyZE=4"), "got: {p}");
    }

    #[test]
    fn stavba_search_maps_house_number_type() {
        assert_eq!(
            stavba_search(38792, 1, 66),
            "/api/v1/Stavby/Vyhledani?KodCastiObce=38792&TypStavby=1&CisloDomovni=66"
        );
    }

    #[test]
    fn jednotka_search_adds_unit_number() {
        assert_eq!(
            jednotka_search(38792, 1, 66, 3),
            "/api/v1/Jednotky/Vyhledani?KodCastiObce=38792&TypStavby=1&CisloDomovni=66&CisloJednotky=3"
        );
    }

    #[test]
    fn rizeni_search_uses_all_four_number_parts() {
        assert_eq!(
            rizeni_search("V", 123, 2026, 701),
            "/api/v1/Rizeni/Vyhledani?TypRizeni=V&Cislo=123&Rok=2026&KodPracoviste=701"
        );
    }

    #[test]
    fn detail_paths_take_the_iskn_id() {
        assert_eq!(parcela_detail("2034508804"), "/api/v1/Parcely/2034508804");
        assert_eq!(stavba_detail("387773804"), "/api/v1/Stavby/387773804");
        assert_eq!(jednotka_detail("555"), "/api/v1/Jednotky/555");
        assert_eq!(rizeni_detail("108667944010"), "/api/v1/Rizeni/108667944010");
        assert_eq!(
            sousedni_parcely("123"),
            "/api/v1/Parcely/SousedniParcely/123"
        );
    }

    #[test]
    fn pravo_stavby_has_three_entry_points() {
        assert_eq!(pravo_stavby_detail("1"), "/api/v1/PravaStavby/1");
        assert_eq!(pravo_stavby_parcela("1"), "/api/v1/PravaStavby/Parcela/1");
        assert_eq!(pravo_stavby_stavba("1"), "/api/v1/PravaStavby/Stavba/1");
    }

    #[test]
    fn pracoviste_has_list_and_by_code() {
        assert_eq!(pracoviste_list(), "/api/v1/CiselnikyISKN/Pracoviste");
        assert_eq!(
            pracoviste_detail(802),
            "/api/v1/CiselnikyISKN/Pracoviste/802"
        );
    }

    #[test]
    fn polygon_paths_carry_the_coordinate_list() {
        let p = parcely_polygon("[{\"x\":1.0,\"y\":2.0}]");
        assert!(
            p.starts_with("/api/v1/Parcely/Polygon?SeznamSouradnic="),
            "got: {p}"
        );
        assert!(
            p.contains("%7B"),
            "coordinates must be percent-encoded: {p}"
        );
        assert!(stavby_polygon("[]").starts_with("/api/v1/Stavby/Polygon?SeznamSouradnic="));
    }

    /// The hand-rolled encoder the library call replaced, kept as the reference
    /// the replacement is proved against.
    fn hand_rolled(s: &str) -> String {
        let mut out = String::with_capacity(s.len() * 3);
        for b in s.as_bytes() {
            match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    out.push(*b as char)
                }
                _ => out.push_str(&format!("%{b:02X}")),
            }
        }
        out
    }

    #[test]
    fn library_encoder_matches_the_hand_rolled_one_byte_for_byte() {
        for b in 0u8..=127 {
            let s = (b as char).to_string();
            assert_eq!(percent_encode(&s), hand_rolled(&s), "ASCII byte {b:#04X}");
        }
        // The inputs this actually sees, plus the characters the two candidate
        // encodings disagree on: space (%20 vs +), `~`, `*` and `/`.
        for s in [
            r#"[{"x":-494140.17,"y":-1116462.13}]"#,
            "adresa='Malá Strana 66, 74247 Hladké Životice'",
            "Životice",
            "a b~c*d/e+f",
            "",
        ] {
            assert_eq!(percent_encode(s), hand_rolled(s), "input: {s}");
        }
    }

    #[test]
    fn space_encodes_as_percent_20_not_plus() {
        // Form encoding would yield `+`, which only a form-urlencoded parser
        // reads back as a space. The RÚIAN `where` clause carries spaces.
        assert_eq!(percent_encode("a b"), "a%20b");
    }

    #[test]
    fn percent_encode_leaves_unreserved_and_escapes_the_rest() {
        assert_eq!(percent_encode("abcXYZ019-_.~"), "abcXYZ019-_.~");
        assert_eq!(percent_encode("a b"), "a%20b");
        assert_eq!(percent_encode("{\"x\":1}"), "%7B%22x%22%3A1%7D");
    }

    #[test]
    fn percent_encode_handles_multibyte_utf8() {
        // Diacritics must be encoded byte-wise, not character-wise.
        assert_eq!(percent_encode("Životice"), "%C5%BDivotice");
    }

    #[test]
    fn stavba_can_be_found_by_ruian_address_point() {
        assert_eq!(
            stavba_adresni_misto(8189986),
            "/api/v1/Stavby/AdresniMisto/8189986"
        );
    }

    #[test]
    fn prijate_dne_carries_type_office_and_date() {
        assert_eq!(
            rizeni_prijate_dne("V", 804, "2026-07-29"),
            "/api/v1/Rizeni/PrijateDne?TypRizeni=V&KodPracoviste=804&DatumPrijeti=2026-07-29"
        );
    }
}
